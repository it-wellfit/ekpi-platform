#![forbid(unsafe_code)]

//! Vector search (Master Prompt Phase 3a).
//!
//! Trait-based vector index with in-memory brute-force cosine for MVP.
//! pgvector adapter is feature-gated for R4.

use serde::{Deserialize, Serialize};

/// A vector search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorHit {
    pub id: String,
    pub score: f64,
    pub metadata: std::collections::BTreeMap<String, String>,
}

/// Trait for vector index implementations.
pub trait VectorIndex {
    fn build(&mut self, vectors: &[(String, Vec<f64>)]);
    fn search(&self, query: &[f64], top_k: usize) -> Vec<VectorHit>;
    fn add(&mut self, id: &str, vector: Vec<f64>);
    fn remove(&mut self, id: &str);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// In-memory brute-force cosine similarity index (MVP).
#[derive(Debug, Clone, Default)]
pub struct InMemoryVectorIndex {
    vectors: Vec<(String, Vec<f64>)>,
}

impl InMemoryVectorIndex {
    pub fn new() -> Self {
        Self::default()
    }
}

impl VectorIndex for InMemoryVectorIndex {
    fn build(&mut self, vectors: &[(String, Vec<f64>)]) {
        self.vectors = vectors.to_vec();
    }

    fn search(&self, query: &[f64], top_k: usize) -> Vec<VectorHit> {
        let mut scores: Vec<(String, f64)> = self
            .vectors
            .iter()
            .map(|(id, v)| (id.clone(), cosine_similarity(query, v)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
            .into_iter()
            .take(top_k)
            .map(|(id, score)| VectorHit {
                id,
                score,
                metadata: std::collections::BTreeMap::new(),
            })
            .collect()
    }

    fn add(&mut self, id: &str, vector: Vec<f64>) {
        self.vectors.push((id.to_string(), vector));
    }

    fn remove(&mut self, id: &str) {
        self.vectors.retain(|(v_id, _)| v_id != id);
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_search() {
        let mut idx = InMemoryVectorIndex::new();
        idx.build(&[
            ("doc1".to_string(), vec![1.0, 0.0, 0.0]),
            ("doc2".to_string(), vec![0.0, 1.0, 0.0]),
            ("doc3".to_string(), vec![0.0, 0.0, 1.0]),
        ]);
        let hits = idx.search(&[1.0, 0.0, 0.0], 2);
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, "doc1");
        assert!((hits[0].score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn add_and_remove() {
        let mut idx = InMemoryVectorIndex::new();
        idx.add("d1", vec![1.0, 0.0]);
        idx.add("d2", vec![0.0, 1.0]);
        assert_eq!(idx.len(), 2);
        idx.remove("d1");
        assert_eq!(idx.len(), 1);
        assert_eq!(idx.search(&[1.0, 0.0], 1)[0].id, "d2");
    }

    #[test]
    fn empty_index_returns_empty() {
        let idx = InMemoryVectorIndex::new();
        assert!(idx.is_empty());
        assert!(idx.search(&[1.0, 0.0], 5).is_empty());
    }

    #[test]
    fn cosine_similarity_identical_is_one() {
        let a = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&a, &a) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn cosine_similarity_orthogonal_is_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b)).abs() < 1e-9);
    }
}
