#![forbid(unsafe_code)]

//! Reranker (Master Prompt Phase 3d).
//!
//! Trait-based reranker with score-fusion for MVP.
//! Cross-encoder adapter is feature-gated for R4.

use serde::{Deserialize, Serialize};

/// A candidate for reranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankCandidate {
    pub id: String,
    pub text: String,
    pub initial_score: f64,
}

/// A reranked result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    pub id: String,
    pub score: f64,
    pub rank: usize,
}

/// Trait for reranker implementations.
pub trait Reranker {
    fn rerank(&self, query: &str, candidates: &[RerankCandidate]) -> Vec<RerankResult>;
}

/// Score-fusion reranker (MVP): combines initial score with term overlap.
#[derive(Debug, Clone, Default)]
pub struct ScoreFusionReranker {
    pub alpha: f64,
}

impl ScoreFusionReranker {
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
        }
    }
}

impl Reranker for ScoreFusionReranker {
    fn rerank(&self, query: &str, candidates: &[RerankCandidate]) -> Vec<RerankResult> {
        let query_terms: std::collections::BTreeSet<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(String::from)
            .collect();

        let mut results: Vec<RerankResult> = candidates
            .iter()
            .map(|c| {
                let cand_terms: std::collections::BTreeSet<String> = c
                    .text
                    .to_lowercase()
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                let overlap = query_terms.intersection(&cand_terms).count() as f64;
                let overlap_ratio = if query_terms.is_empty() {
                    0.0
                } else {
                    overlap / query_terms.len() as f64
                };
                let fused = self.alpha * c.initial_score + (1.0 - self.alpha) * overlap_ratio;
                RerankResult {
                    id: c.id.clone(),
                    score: fused,
                    rank: 0,
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (i, r) in results.iter_mut().enumerate() {
            r.rank = i + 1;
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rerank_orders_by_fused_score() {
        let reranker = ScoreFusionReranker::new(0.5);
        let candidates = vec![
            RerankCandidate {
                id: "a".to_string(),
                text: "change management process".to_string(),
                initial_score: 0.5,
            },
            RerankCandidate {
                id: "b".to_string(),
                text: "quality control inspection".to_string(),
                initial_score: 0.9,
            },
        ];
        let results = reranker.rerank("change management", &candidates);
        assert_eq!(results.len(), 2);
        // "a" should rank higher due to term overlap despite lower initial score
        assert_eq!(results[0].id, "a");
    }

    #[test]
    fn rerank_assigns_ranks() {
        let reranker = ScoreFusionReranker::new(0.5);
        let candidates = vec![
            RerankCandidate {
                id: "a".to_string(),
                text: "hello".to_string(),
                initial_score: 0.5,
            },
            RerankCandidate {
                id: "b".to_string(),
                text: "world".to_string(),
                initial_score: 0.3,
            },
        ];
        let results = reranker.rerank("hello", &candidates);
        assert_eq!(results[0].rank, 1);
        assert_eq!(results[1].rank, 2);
    }

    #[test]
    fn alpha_zero_uses_only_overlap() {
        let reranker = ScoreFusionReranker::new(0.0);
        let candidates = vec![
            RerankCandidate {
                id: "a".to_string(),
                text: "change management".to_string(),
                initial_score: 0.0,
            },
            RerankCandidate {
                id: "b".to_string(),
                text: "quality control".to_string(),
                initial_score: 1.0,
            },
        ];
        let results = reranker.rerank("change management", &candidates);
        assert_eq!(results[0].id, "a");
    }

    #[test]
    fn alpha_one_uses_only_initial_score() {
        let reranker = ScoreFusionReranker::new(1.0);
        let candidates = vec![
            RerankCandidate {
                id: "a".to_string(),
                text: "change management".to_string(),
                initial_score: 0.3,
            },
            RerankCandidate {
                id: "b".to_string(),
                text: "quality control".to_string(),
                initial_score: 0.9,
            },
        ];
        let results = reranker.rerank("change management", &candidates);
        assert_eq!(results[0].id, "b");
    }

    #[test]
    fn empty_candidates_returns_empty() {
        let reranker = ScoreFusionReranker::new(0.5);
        let results = reranker.rerank("query", &[]);
        assert!(results.is_empty());
    }
}
