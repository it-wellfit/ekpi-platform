#![forbid(unsafe_code)]

//! RAG orchestration prototype (Phase 4).
//!
//! Retrieves relevant chunks, constructs a prompt with citation instructions,
//! calls an [`LlmProvider`], verifies an exact `[doc_id@version]` citation and
//! returns an answer or refuses.
//!
//! **The only provider in this repository is [`MockLlmProvider`].** It is a
//! deterministic test double, not an LLM integration. `--llm` therefore does
//! not call an external model and must not be treated as AI evaluation
//! evidence (see `EVAL-GAP-01`).

use crate::docs::DocDatabase;
use crate::retrieval::{Citation, Retriever, SearchResult};

/// Result of a RAG query.
#[derive(Debug, Clone)]
pub enum RagResult {
    Answer {
        text: String,
        citations: Vec<Citation>,
        confidence: f64,
        latency_ms: u64,
    },
    Refused {
        reason: String,
    },
}

impl RagResult {
    pub fn has_answer(&self) -> bool {
        matches!(self, RagResult::Answer { .. })
    }

    pub fn citation_count(&self) -> usize {
        match self {
            RagResult::Answer { citations, .. } => citations.len(),
            RagResult::Refused { .. } => 0,
        }
    }
}

/// Trait for LLM providers (mockable for testing).
pub trait LlmProvider {
    fn generate(&self, prompt: &str) -> Result<String, String>;
}

/// A mock LLM provider for testing.
pub struct MockLlmProvider {
    pub response: String,
}

impl LlmProvider for MockLlmProvider {
    fn generate(&self, _prompt: &str) -> Result<String, String> {
        Ok(self.response.clone())
    }
}

/// RAG orchestrator.
pub struct RagOrchestrator {
    pub threshold: f64,
    pub max_citations: usize,
    pub min_confidence: f64,
}

impl RagOrchestrator {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            max_citations: 5,
            min_confidence: 0.5,
        }
    }

    pub fn answer(
        &self,
        question: &str,
        retriever: &Retriever,
        db: &DocDatabase,
        llm: &dyn LlmProvider,
    ) -> RagResult {
        let start = std::time::Instant::now();

        let search_result = retriever.search_with_citations(question, self.max_citations, db);
        if !search_result.has_evidence() {
            return RagResult::Refused {
                reason: match search_result {
                    SearchResult::InsufficientEvidence { reason } => reason,
                    _ => "no evidence found".to_string(),
                },
            };
        }

        let citations = match search_result {
            SearchResult::Hits(c) => c,
            _ => {
                return RagResult::Refused {
                    reason: "no evidence found".to_string(),
                }
            }
        };

        let context = citations
            .iter()
            .enumerate()
            .map(|(i, c)| {
                format!(
                    "[{}] {} ({}@{}): {}",
                    i + 1,
                    c.verbatim_text,
                    c.doc_id,
                    c.version,
                    c.section
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are a helpful assistant. Answer the question using ONLY the provided context. \
             Cite sources using [doc_id@version] format. If the context does not contain \
             sufficient information, say \"I don't have enough information to answer.\"\n\n\
             Context:\n{}\n\nQuestion: {}",
            context, question
        );

        let llm_response = match llm.generate(&prompt) {
            Ok(r) => r,
            Err(e) => {
                return RagResult::Refused {
                    reason: format!("LLM error: {e}"),
                };
            }
        };

        let has_citation = citations
            .iter()
            .any(|citation| has_citation_marker(&llm_response, citation));

        let confidence = if has_citation {
            citations.first().map(|c| c.score).unwrap_or(0.0)
        } else {
            0.0
        };

        if confidence < self.min_confidence && !has_citation {
            return RagResult::Refused {
                reason: "LLM response lacks citation support".to_string(),
            };
        }

        let latency_ms = start.elapsed().as_millis() as u64;

        RagResult::Answer {
            text: llm_response,
            citations,
            confidence,
            latency_ms,
        }
    }

    pub fn verify_citations(answer: &str, citations: &[Citation]) -> bool {
        if citations.is_empty() || answer.trim().is_empty() {
            return false;
        }
        citations
            .iter()
            .any(|citation| has_citation_marker(answer, citation))
    }
}

fn has_citation_marker(answer: &str, citation: &Citation) -> bool {
    answer.contains(&format!("[{}@{}]", citation.doc_id, citation.version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docs::DocDatabase;

    fn make_db() -> (DocDatabase, std::collections::BTreeMap<String, String>) {
        let json = r#"{
            "documents": [
                {
                    "doc_id": "SOP-CHANGE",
                    "title": "Change Management SOP",
                    "permissions": [],
                    "versions": [{
                        "version": "3.2",
                        "effective_from": "2026-03-01",
                        "sections": [
                            {"section": "4.3", "verbatim_text": "quality management system requires document control"}
                        ]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let mut dv = std::collections::BTreeMap::new();
        dv.insert("SOP-CHANGE".to_string(), "3.2".to_string());
        (db, dv)
    }

    #[test]
    fn rag_returns_answer_with_citations() {
        let (db, dv) = make_db();
        let retriever = Retriever::build(&db, &dv, 0.1);
        let llm = MockLlmProvider {
            response: "Based on [SOP-CHANGE@3.2], quality management requires document control."
                .to_string(),
        };
        let rag = RagOrchestrator::new(0.1);
        let result = rag.answer("quality management", &retriever, &db, &llm);
        assert!(result.has_answer());
        assert!(result.citation_count() > 0);
    }

    #[test]
    fn rag_refuses_on_no_evidence() {
        let (db, dv) = make_db();
        let retriever = Retriever::build(&db, &dv, 0.99);
        let llm = MockLlmProvider {
            response: "I can answer anything.".to_string(),
        };
        let rag = RagOrchestrator::new(0.99);
        let result = rag.answer("quantum physics", &retriever, &db, &llm);
        assert!(!result.has_answer());
    }

    #[test]
    fn rag_verifies_citations() {
        let citations = vec![Citation {
            doc_id: "DOC-1".to_string(),
            version: "1.0".to_string(),
            section: "s1".to_string(),
            score: 0.9,
            verbatim_text: "test text".to_string(),
            char_range: (0, 9),
        }];
        let answer = "Based on [DOC-1@1.0], the answer is test text.";
        assert!(RagOrchestrator::verify_citations(answer, &citations));
    }

    #[test]
    fn rag_verify_citations_fails_without_reference() {
        let citations = vec![Citation {
            doc_id: "DOC-1".to_string(),
            version: "1.0".to_string(),
            section: "s1".to_string(),
            score: 0.9,
            verbatim_text: "test text".to_string(),
            char_range: (0, 9),
        }];
        let answer = "I don't know the answer.";
        assert!(!RagOrchestrator::verify_citations(answer, &citations));
    }

    #[test]
    fn rag_refuses_empty_citations() {
        assert!(!RagOrchestrator::verify_citations("any answer", &[]));
    }

    #[test]
    fn rag_refuses_answer_without_versioned_citation() {
        let (db, dv) = make_db();
        let retriever = Retriever::build(&db, &dv, 0.1);
        let llm = MockLlmProvider {
            response: "Based on [SOP-CHANGE], quality management requires document control."
                .to_string(),
        };
        let rag = RagOrchestrator::new(0.1);
        let result = rag.answer("quality management", &retriever, &db, &llm);
        assert!(!result.has_answer());
    }

    #[test]
    fn verify_citations_rejects_unversioned_or_verbatim_only_answers() {
        let citations = vec![Citation {
            doc_id: "DOC-1".to_string(),
            version: "1.0".to_string(),
            section: "s1".to_string(),
            score: 0.9,
            verbatim_text: "test text".to_string(),
            char_range: (0, 9),
        }];
        assert!(!RagOrchestrator::verify_citations(
            "Based on [DOC-1], see test text.",
            &citations
        ));
        assert!(!RagOrchestrator::verify_citations(
            "test text appears here without a citation",
            &citations
        ));
    }

    #[test]
    fn mock_llm_provider_works() {
        let llm = MockLlmProvider {
            response: "test response".to_string(),
        };
        assert_eq!(llm.generate("prompt").unwrap(), "test response");
    }
}
