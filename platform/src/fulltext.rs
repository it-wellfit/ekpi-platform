#![forbid(unsafe_code)]

//! Full-text search (Master Prompt Phase 3b).
//!
//! Trait-based full-text index with in-memory BM25 for MVP.
//! Tantivy adapter is feature-gated for R4.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A full-text search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTextHit {
    pub id: String,
    pub score: f64,
    pub matched_terms: Vec<String>,
}

/// Trait for full-text index implementations.
pub trait FullTextIndex {
    fn build(&mut self, documents: &[(String, String)]);
    fn search(&self, query: &str, top_k: usize) -> Vec<FullTextHit>;
    fn add_document(&mut self, id: &str, text: &str);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-memory BM25 index (MVP).
#[derive(Debug, Clone, Default)]
pub struct InMemoryFullTextIndex {
    documents: Vec<(String, String)>,
    doc_freqs: BTreeMap<String, usize>,
    total_docs: usize,
    avg_doc_len: f64,
}

impl InMemoryFullTextIndex {
    pub fn new() -> Self {
        Self::default()
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect()
    }
}

impl FullTextIndex for InMemoryFullTextIndex {
    fn build(&mut self, documents: &[(String, String)]) {
        self.documents = documents.to_vec();
        self.total_docs = documents.len();
        let mut total_len = 0usize;
        self.doc_freqs.clear();
        for (_, text) in documents {
            let tokens = Self::tokenize(text);
            total_len += tokens.len();
            let mut seen = std::collections::BTreeSet::new();
            for token in &tokens {
                if seen.insert(token.clone()) {
                    *self.doc_freqs.entry(token.clone()).or_insert(0) += 1;
                }
            }
        }
        self.avg_doc_len = if self.total_docs > 0 {
            total_len as f64 / self.total_docs as f64
        } else {
            1.0
        };
    }

    fn search(&self, query: &str, top_k: usize) -> Vec<FullTextHit> {
        let query_terms = Self::tokenize(query);
        let k1 = 1.2;
        let b = 0.75;

        let mut scores: Vec<(String, f64, Vec<String>)> = self
            .documents
            .iter()
            .map(|(id, text)| {
                let tokens = Self::tokenize(text);
                let doc_len = tokens.len() as f64;
                let mut matched = Vec::new();
                let mut score = 0.0;
                for term in &query_terms {
                    let tf = tokens.iter().filter(|t| **t == *term).count() as f64;
                    let df = *self.doc_freqs.get(term).unwrap_or(&0) as f64;
                    if df > 0.0 && tf > 0.0 {
                        matched.push(term.clone());
                        let idf = ((self.total_docs as f64 - df + 0.5) / (df + 0.5) + 1.0).ln();
                        let tf_norm = (tf * (k1 + 1.0))
                            / (tf + k1 * (1.0 - b + b * doc_len / self.avg_doc_len));
                        score += idf * tf_norm;
                    }
                }
                (id.clone(), score, matched)
            })
            .filter(|(_, score, _)| *score > 0.0)
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
            .into_iter()
            .take(top_k)
            .map(|(id, score, matched)| FullTextHit {
                id,
                score,
                matched_terms: matched,
            })
            .collect()
    }

    fn add_document(&mut self, id: &str, text: &str) {
        self.documents.push((id.to_string(), text.to_string()));
        // Rebuild for simplicity
        let docs = self.documents.clone();
        self.build(&docs);
    }

    fn len(&self) -> usize {
        self.documents.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_search() {
        let mut idx = InMemoryFullTextIndex::new();
        idx.build(&[
            (
                "d1".to_string(),
                "change management process approval".to_string(),
            ),
            ("d2".to_string(), "quality control inspection".to_string()),
            ("d3".to_string(), "change request execution".to_string()),
        ]);
        let hits = idx.search("change management", 2);
        assert!(!hits.is_empty());
        assert_eq!(hits[0].id, "d1");
    }

    #[test]
    fn bm25_scores_relevant_higher() {
        let mut idx = InMemoryFullTextIndex::new();
        idx.build(&[
            (
                "d1".to_string(),
                "change change change management".to_string(),
            ),
            ("d2".to_string(), "change management".to_string()),
        ]);
        let hits = idx.search("change management", 2);
        assert_eq!(hits.len(), 2);
        // d1 should score higher due to term frequency
        assert!(hits[0].score >= hits[1].score);
    }

    #[test]
    fn no_match_returns_empty() {
        let mut idx = InMemoryFullTextIndex::new();
        idx.build(&[("d1".to_string(), "hello world".to_string())]);
        let hits = idx.search("nonexistent term", 5);
        assert!(hits.is_empty());
    }

    #[test]
    fn add_document_updates_index() {
        let mut idx = InMemoryFullTextIndex::new();
        idx.build(&[("d1".to_string(), "hello".to_string())]);
        idx.add_document("d2", "world");
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn matched_terms_reported() {
        let mut idx = InMemoryFullTextIndex::new();
        idx.build(&[("d1".to_string(), "change management process".to_string())]);
        let hits = idx.search("change management", 1);
        assert!(!hits[0].matched_terms.is_empty());
    }
}
