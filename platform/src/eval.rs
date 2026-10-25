#![forbid(unsafe_code)]

//! Knowledge search evaluation harness (Slice 4).
//!
//! Loads a fixed test set (JSON), runs retrieval for each question,
//! and computes precision@K, citation support, and permission isolation metrics.

use crate::docs::DocDatabase;
use crate::retrieval::{PermissionContext, Retriever, SearchResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single evaluation case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCase {
    /// Unique case ID.
    pub id: String,
    /// The question to search for.
    pub question: String,
    /// Expected document ID in the top results (for precision).
    pub expected_doc_id: String,
    /// Expected section in the top results.
    pub expected_section: String,
    /// Minimum acceptable score threshold.
    #[serde(default = "default_min_score")]
    pub min_score: f64,
    /// User roles for permission testing (empty = no permission filter).
    #[serde(default)]
    pub roles: Vec<String>,
}

fn default_min_score() -> f64 {
    0.1
}

/// Result of evaluating a single case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCaseResult {
    pub id: String,
    pub question: String,
    pub hit: bool,
    pub top_score: f64,
    pub citation_count: usize,
    pub has_evidence: bool,
    pub permission_isolated: bool,
    pub latency_ms: u64,
}

/// Aggregate evaluation metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalMetrics {
    pub total_cases: usize,
    pub hits: usize,
    pub precision_at_1: f64,
    pub citation_support_rate: f64,
    pub permission_isolation_rate: f64,
    pub avg_latency_ms: f64,
    pub cases: Vec<EvalCaseResult>,
}

impl EvalMetrics {
    pub fn compute(cases: &[EvalCaseResult]) -> Self {
        let total = cases.len();
        let hits = cases.iter().filter(|c| c.hit).count();
        let with_evidence = cases.iter().filter(|c| c.has_evidence).count();
        let isolated = cases.iter().filter(|c| c.permission_isolated).count();
        let total_latency: u64 = cases.iter().map(|c| c.latency_ms).sum();
        Self {
            total_cases: total,
            hits,
            precision_at_1: if total > 0 {
                hits as f64 / total as f64
            } else {
                0.0
            },
            citation_support_rate: if total > 0 {
                with_evidence as f64 / total as f64
            } else {
                0.0
            },
            permission_isolation_rate: if total > 0 {
                isolated as f64 / total as f64
            } else {
                0.0
            },
            avg_latency_ms: if total > 0 {
                total_latency as f64 / total as f64
            } else {
                0.0
            },
            cases: cases.to_vec(),
        }
    }
}

/// Run evaluation on a set of cases against a document database.
pub fn run_eval(
    db: &DocDatabase,
    doc_versions: &BTreeMap<String, String>,
    cases: &[EvalCase],
    threshold: f64,
) -> EvalMetrics {
    let mut results = Vec::new();
    for case in cases {
        let start = std::time::Instant::now();
        let perm = if case.roles.is_empty() {
            None
        } else {
            Some(PermissionContext::new("T1", "eval-user", case.roles.clone()))
        };
        let retriever = Retriever::build_with_permissions(db, doc_versions, threshold, perm.as_ref());
        let result = retriever.search_with_citations(&case.question, 5, db);
        let latency_ms = start.elapsed().as_millis() as u64;

        let (hit, top_score, citation_count, has_evidence) = match &result {
            SearchResult::Hits(citations) => {
                let top = citations.first().map(|c| c.score).unwrap_or(0.0);
                let hit = citations
                    .first()
                    .map(|c| {
                        c.doc_id == case.expected_doc_id && c.section == case.expected_section
                    })
                    .unwrap_or(false);
                (hit, top, citations.len(), true)
            }
            SearchResult::InsufficientEvidence { .. } => (false, 0.0, 0, false),
        };

        let permission_isolated = if case.roles.is_empty() {
            true
        } else {
            match &result {
                SearchResult::Hits(citations) => {
                    citations.iter().all(|c| {
                        db.doc(&c.doc_id)
                            .map(|d| {
                                d.permissions.is_empty()
                                    || d.permissions.iter().any(|p| case.roles.contains(p))
                            })
                            .unwrap_or(true)
                    })
                }
                _ => true,
            }
        };

        results.push(EvalCaseResult {
            id: case.id.clone(),
            question: case.question.clone(),
            hit,
            top_score,
            citation_count,
            has_evidence,
            permission_isolated,
            latency_ms,
        });
    }
    EvalMetrics::compute(&results)
}

/// Load evaluation cases from a JSON file.
pub fn load_eval_cases(path: &std::path::Path) -> Result<Vec<EvalCase>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let cases: Vec<EvalCase> =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    if cases.is_empty() {
        return Err("eval case file is empty".to_string());
    }
    Ok(cases)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_db() -> (DocDatabase, BTreeMap<String, String>) {
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
                            {"section": "4.3", "verbatim_text": "quality management system requires document control"},
                            {"section": "5.1", "verbatim_text": "procurement process for purchasing goods"}
                        ]
                    }]
                },
                {
                    "doc_id": "SECRET-DOC",
                    "title": "Secret",
                    "permissions": ["admin"],
                    "versions": [{
                        "version": "1.0",
                        "effective_from": "2026-01-01",
                        "sections": [{"section": "s1", "verbatim_text": "confidential admin data"}]
                    }]
                }
            ]
        }"#;
        let db: DocDatabase = serde_json::from_str(json).unwrap();
        let mut dv = BTreeMap::new();
        dv.insert("SOP-CHANGE".to_string(), "3.2".to_string());
        dv.insert("SECRET-DOC".to_string(), "1.0".to_string());
        (db, dv)
    }

    #[test]
    fn eval_case_hits_expected() {
        let (db, dv) = make_db();
        let cases = vec![EvalCase {
            id: "TC-001".to_string(),
            question: "quality management".to_string(),
            expected_doc_id: "SOP-CHANGE".to_string(),
            expected_section: "4.3".to_string(),
            min_score: 0.1,
            roles: vec![],
        }];
        let metrics = run_eval(&db, &dv, &cases, 0.1);
        assert_eq!(metrics.total_cases, 1);
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.precision_at_1, 1.0);
        assert!(metrics.citation_support_rate > 0.0);
    }

    #[test]
    fn eval_case_misses_wrong_expected() {
        let (db, dv) = make_db();
        let cases = vec![EvalCase {
            id: "TC-002".to_string(),
            question: "quality management".to_string(),
            expected_doc_id: "NONEXISTENT".to_string(),
            expected_section: "9.9".to_string(),
            min_score: 0.1,
            roles: vec![],
        }];
        let metrics = run_eval(&db, &dv, &cases, 0.1);
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.precision_at_1, 0.0);
    }

    #[test]
    fn eval_permission_isolation() {
        let (db, dv) = make_db();
        let cases = vec![EvalCase {
            id: "TC-003".to_string(),
            question: "confidential admin".to_string(),
            expected_doc_id: "SECRET-DOC".to_string(),
            expected_section: "s1".to_string(),
            min_score: 0.1,
            roles: vec!["viewer".to_string()],
        }];
        let metrics = run_eval(&db, &dv, &cases, 0.1);
        assert_eq!(metrics.permission_isolation_rate, 1.0);
    }

    #[test]
    fn load_eval_cases_from_json() {
        let json = r#"[{"id":"TC-1","question":"test","expected_doc_id":"D1","expected_section":"s1"}]"#;
        let path = std::env::temp_dir().join("test_eval_cases.json");
        std::fs::write(&path, json).unwrap();
        let cases = load_eval_cases(&path).unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].id, "TC-1");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_eval_cases_empty_file_fails() {
        let path = std::env::temp_dir().join("test_eval_empty.json");
        std::fs::write(&path, "[]").unwrap();
        assert!(load_eval_cases(&path).is_err());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn eval_metrics_compute() {
        let cases = vec![
            EvalCaseResult {
                id: "1".to_string(),
                question: "q1".to_string(),
                hit: true,
                top_score: 0.9,
                citation_count: 2,
                has_evidence: true,
                permission_isolated: true,
                latency_ms: 10,
            },
            EvalCaseResult {
                id: "2".to_string(),
                question: "q2".to_string(),
                hit: false,
                top_score: 0.0,
                citation_count: 0,
                has_evidence: false,
                permission_isolated: true,
                latency_ms: 5,
            },
        ];
        let m = EvalMetrics::compute(&cases);
        assert_eq!(m.total_cases, 2);
        assert_eq!(m.hits, 1);
        assert!((m.precision_at_1 - 0.5).abs() < 1e-9);
        assert!((m.citation_support_rate - 0.5).abs() < 1e-9);
        assert!((m.permission_isolation_rate - 1.0).abs() < 1e-9);
        assert!((m.avg_latency_ms - 7.5).abs() < 1e-9);
    }
}