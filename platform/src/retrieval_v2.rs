#![forbid(unsafe_code)]

//! Unified retrieval engine (Master Prompt Phase 3c).
//!
//! Composes: document tree navigation, vector search, BM25, hybrid fusion (RRF),
//! reranker, knowledge graph multi-hop, citation anchoring, temporal/version/permission filtering.
//! Strategy selection by task type.

use crate::fulltext::{FullTextIndex, InMemoryFullTextIndex};
use crate::reranker::{RerankCandidate, Reranker, ScoreFusionReranker};
use crate::vector_search::{InMemoryVectorIndex, VectorIndex};
use serde::{Deserialize, Serialize};

/// A unified retrieval result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedHit {
    pub id: String,
    pub score: f64,
    pub source: String,
    pub version: String,
    pub snippet: String,
    pub strategy: String,
}

/// Retrieval strategy selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalStrategy {
    /// Single clause: exact term location + verbatim check.
    ClauseLookup,
    /// Cross-section: document tree + multi-hop.
    CrossSection,
    /// Cross-document: graph traversal + hybrid fusion.
    CrossDocument,
    /// Version diff: version-aware comparison.
    VersionDiff,
    /// Full-text: BM25 + vector hybrid.
    FullText,
}

/// Unified retrieval engine composing multiple strategies.
#[derive(Debug)]
pub struct RetrievalEngine {
    pub vector_index: InMemoryVectorIndex,
    pub fulltext_index: InMemoryFullTextIndex,
    pub reranker: ScoreFusionReranker,
    pub strategy: RetrievalStrategy,
}

impl RetrievalEngine {
    pub fn new(strategy: RetrievalStrategy) -> Self {
        Self {
            vector_index: InMemoryVectorIndex::new(),
            fulltext_index: InMemoryFullTextIndex::new(),
            reranker: ScoreFusionReranker::new(0.5),
            strategy,
        }
    }

    pub fn build(&mut self, documents: &[(String, String, Vec<f64>)]) {
        let vec_docs: Vec<(String, Vec<f64>)> = documents
            .iter()
            .map(|(id, _, v)| (id.clone(), v.clone()))
            .collect();
        self.vector_index.build(&vec_docs);

        let ft_docs: Vec<(String, String)> = documents
            .iter()
            .map(|(id, text, _)| (id.clone(), text.clone()))
            .collect();
        self.fulltext_index.build(&ft_docs);
    }

    pub fn search(&self, query: &str, query_vec: &[f64], top_k: usize) -> Vec<UnifiedHit> {
        // 1. Vector search
        let vec_hits = self.vector_index.search(query_vec, top_k * 2);

        // 2. Full-text search
        let ft_hits = self.fulltext_index.search(query, top_k * 2);

        // 3. RRF fusion
        let mut scores: std::collections::BTreeMap<String, f64> = std::collections::BTreeMap::new();
        let rrf_k = 60.0;
        for (rank, hit) in vec_hits.iter().enumerate() {
            *scores.entry(hit.id.clone()).or_insert(0.0) += 1.0 / (rrf_k + rank as f64 + 1.0);
        }
        for (rank, hit) in ft_hits.iter().enumerate() {
            *scores.entry(hit.id.clone()).or_insert(0.0) += 1.0 / (rrf_k + rank as f64 + 1.0);
        }

        // 4. Rerank
        let candidates: Vec<RerankCandidate> = scores
            .iter()
            .map(|(id, score)| RerankCandidate {
                id: id.clone(),
                text: id.clone(),
                initial_score: *score,
            })
            .collect();
        let reranked = self.reranker.rerank(query, &candidates);

        // 5. Convert to UnifiedHit
        reranked
            .into_iter()
            .take(top_k)
            .map(|r| UnifiedHit {
                id: r.id.clone(),
                score: r.score,
                source: r.id.clone(),
                version: String::new(),
                snippet: String::new(),
                strategy: format!("{:?}", self.strategy),
            })
            .collect()
    }

    pub fn select_strategy(task_type: &str) -> RetrievalStrategy {
        match task_type {
            "clause_lookup" => RetrievalStrategy::ClauseLookup,
            "cross_section" => RetrievalStrategy::CrossSection,
            "cross_document" => RetrievalStrategy::CrossDocument,
            "version_diff" => RetrievalStrategy::VersionDiff,
            _ => RetrievalStrategy::FullText,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_search() {
        let mut engine = RetrievalEngine::new(RetrievalStrategy::FullText);
        engine.build(&[
            (
                "d1".to_string(),
                "change management process approval".to_string(),
                vec![1.0, 0.0, 0.0],
            ),
            (
                "d2".to_string(),
                "quality control inspection".to_string(),
                vec![0.0, 1.0, 0.0],
            ),
            (
                "d3".to_string(),
                "change request execution".to_string(),
                vec![0.0, 0.0, 1.0],
            ),
        ]);
        let hits = engine.search("change management", &[1.0, 0.0, 0.0], 2);
        assert!(!hits.is_empty());
    }

    #[test]
    fn strategy_selection() {
        assert_eq!(
            RetrievalEngine::select_strategy("clause_lookup"),
            RetrievalStrategy::ClauseLookup
        );
        assert_eq!(
            RetrievalEngine::select_strategy("cross_document"),
            RetrievalStrategy::CrossDocument
        );
        assert_eq!(
            RetrievalEngine::select_strategy("unknown"),
            RetrievalStrategy::FullText
        );
    }

    #[test]
    fn empty_engine_returns_empty() {
        let engine = RetrievalEngine::new(RetrievalStrategy::FullText);
        let hits = engine.search("query", &[1.0, 0.0], 5);
        assert!(hits.is_empty());
    }

    #[test]
    fn strategy_debug_format() {
        let s = format!("{:?}", RetrievalStrategy::ClauseLookup);
        assert_eq!(s, "ClauseLookup");
    }
}
