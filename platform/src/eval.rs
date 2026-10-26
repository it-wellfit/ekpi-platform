#![forbid(unsafe_code)]

//! Deterministic knowledge-search evaluation harness.
//!
//! Loads a fixed test set (JSON), runs retrieval for each question and records
//! whether the top citation matches the expected source, or whether the query
//! correctly returned insufficient evidence. This is a regression harness for
//! the lexical retrieval path; it is not an LLM/agent quality claim.

use crate::docs::{Doc, DocDatabase};
use crate::retrieval::{PermissionContext, Retriever, SearchResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The fixed set must exercise these dimensions. Tool failure is covered
/// separately by the fail-closed CLI test because retrieval calls no tool.
pub const REQUIRED_CATEGORIES: [&str; 6] = [
    "general",
    "ambiguous",
    "adversarial",
    "wrong_source",
    "insufficient_context",
    "permission",
];

/// A set smaller than this is not accepted as a regression baseline.
pub const MIN_EVAL_CASES: usize = 10;

/// A permission gate must exercise both an allowed and a denied retrieval.
/// Without both directions a 1.00 isolation score is vacuous.
pub const MIN_PERMISSION_CASES: usize = 2;

fn default_category() -> String {
    "general".to_string()
}

fn default_min_score() -> f64 {
    0.42
}

fn default_version() -> u32 {
    1
}

/// A single evaluation case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCase {
    /// Unique case ID.
    pub id: String,
    /// Dimension this case exercises (`general`, `ambiguous`, ...).
    #[serde(default = "default_category")]
    pub category: String,
    /// The question to search for.
    pub question: String,
    /// True when the correct behavior is an honest refusal, not a citation.
    #[serde(default)]
    pub expect_refusal: bool,
    /// Expected document ID in the top result (required unless refusing).
    #[serde(default)]
    pub expected_doc_id: String,
    /// Expected section in the top result (required unless refusing).
    #[serde(default)]
    pub expected_section: String,
    /// Minimum acceptable top-1 score for an expected hit.
    #[serde(default = "default_min_score")]
    pub min_score: f64,
    /// User roles for permission testing (empty = no permission filter).
    #[serde(default)]
    pub roles: Vec<String>,
}

/// A fixed evaluation set with an operating threshold.
///
/// `fixtures` are synthetic documents used only for this evaluation run. They
/// are merged into a clone of the production document database so a fixed set
/// can exercise role-restricted retrieval without exposing synthetic documents
/// through the ordinary `--search` surface.
#[derive(Debug, Clone)]
pub struct EvalSet {
    pub version: u32,
    pub threshold: Option<f64>,
    pub cases: Vec<EvalCase>,
    pub fixtures: Vec<Doc>,
}

#[derive(Debug, Deserialize)]
struct EvalSetFile {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default)]
    threshold: Option<f64>,
    #[serde(default)]
    fixtures: Vec<Doc>,
    cases: Vec<EvalCase>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EvalFile {
    Set(EvalSetFile),
    List(Vec<EvalCase>),
}

/// Result of evaluating a single case.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCaseResult {
    pub id: String,
    pub category: String,
    pub question: String,
    pub expect_refusal: bool,
    /// True when an expected answer matched the top citation.
    pub hit: bool,
    /// True when the full case contract was satisfied.
    pub passed: bool,
    /// Human-readable reason when `passed` is false.
    pub failure: Option<String>,
    pub top_doc_id: Option<String>,
    pub top_section: Option<String>,
    pub top_score: f64,
    pub citation_count: usize,
    pub has_evidence: bool,
    /// True when the case supplied at least one role and therefore exercised
    /// the permission filter. Public-only cases do not count toward the
    /// permission isolation metric.
    pub role_restricted: bool,
    pub permission_isolated: bool,
    pub latency_ms: u64,
}

/// Per-category pass counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategorySummary {
    pub total: usize,
    pub passed: usize,
}

/// Aggregate evaluation metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalMetrics {
    pub total_cases: usize,
    pub answer_cases: usize,
    pub refusal_cases: usize,
    pub hits: usize,
    pub correct_refusals: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub precision_at_1: f64,
    pub citation_support_rate: f64,
    /// Number of cases that supplied roles and therefore exercised filtering.
    pub permission_cases: usize,
    /// Role-restricted cases that expect a permitted answer.
    pub permission_allowed_cases: usize,
    /// Role-restricted cases that expect an honest refusal.
    pub permission_denied_cases: usize,
    /// Role-restricted cases that returned a citation the roles cannot see.
    pub permission_violations: usize,
    /// Violation-free role-restricted cases divided by `permission_cases`.
    /// Zero when no role-restricted case ran; the gate rejects that state.
    pub permission_isolation_rate: f64,
    pub refusal_accuracy: f64,
    pub avg_latency_ms: f64,
    pub categories: BTreeMap<String, CategorySummary>,
    pub cases: Vec<EvalCaseResult>,
}

impl EvalMetrics {
    pub fn compute(cases: &[EvalCaseResult]) -> Self {
        let total = cases.len();
        let answer_cases = cases.iter().filter(|c| !c.expect_refusal).count();
        let refusal_cases = total - answer_cases;
        let hits = cases.iter().filter(|c| c.hit).count();
        let passed_cases = cases.iter().filter(|c| c.passed).count();
        let with_evidence = cases
            .iter()
            .filter(|c| !c.expect_refusal && c.has_evidence)
            .count();
        let permission_cases = cases.iter().filter(|c| c.role_restricted).count();
        let permission_allowed_cases = cases
            .iter()
            .filter(|c| c.role_restricted && !c.expect_refusal)
            .count();
        let permission_denied_cases = cases
            .iter()
            .filter(|c| c.role_restricted && c.expect_refusal)
            .count();
        let permission_violations = cases
            .iter()
            .filter(|c| c.role_restricted && !c.permission_isolated)
            .count();
        let correct_refusals = cases
            .iter()
            .filter(|c| c.expect_refusal && c.passed)
            .count();
        let total_latency: u64 = cases.iter().map(|c| c.latency_ms).sum();
        let mut categories: BTreeMap<String, CategorySummary> = BTreeMap::new();
        for case in cases {
            let entry = categories.entry(case.category.clone()).or_default();
            entry.total += 1;
            if case.passed {
                entry.passed += 1;
            }
        }
        Self {
            total_cases: total,
            answer_cases,
            refusal_cases,
            hits,
            correct_refusals,
            passed_cases,
            failed_cases: total - passed_cases,
            precision_at_1: if answer_cases > 0 {
                hits as f64 / answer_cases as f64
            } else {
                0.0
            },
            citation_support_rate: if answer_cases > 0 {
                with_evidence as f64 / answer_cases as f64
            } else {
                0.0
            },
            permission_cases,
            permission_allowed_cases,
            permission_denied_cases,
            permission_violations,
            permission_isolation_rate: if permission_cases > 0 {
                (permission_cases - permission_violations) as f64 / permission_cases as f64
            } else {
                0.0
            },
            refusal_accuracy: if refusal_cases > 0 {
                correct_refusals as f64 / refusal_cases as f64
            } else {
                0.0
            },
            avg_latency_ms: if total > 0 {
                total_latency as f64 / total as f64
            } else {
                0.0
            },
            categories,
            cases: cases.to_vec(),
        }
    }

    /// Gate failures that make this run unsuitable as regression evidence.
    pub fn gate_failures(&self) -> Vec<String> {
        let mut failures = Vec::new();
        if self.total_cases < MIN_EVAL_CASES {
            failures.push(format!(
                "only {} case(s); at least {MIN_EVAL_CASES} are required",
                self.total_cases
            ));
        }
        if self.failed_cases > 0 {
            failures.push(format!("{} case(s) failed", self.failed_cases));
        }
        if self.permission_cases < MIN_PERMISSION_CASES {
            failures.push(format!(
                "only {} role-restricted permission case(s); at least {MIN_PERMISSION_CASES} are required",
                self.permission_cases
            ));
        }
        if self.permission_allowed_cases == 0 {
            failures
                .push("permission gate has no case that expects a permitted answer".to_string());
        }
        if self.permission_denied_cases == 0 {
            failures.push("permission gate has no case that expects an honest refusal".to_string());
        }
        if self.permission_violations > 0 {
            failures.push(format!(
                "permission isolation rate is {:.2}, expected 1.00 ({} role-restricted violation(s))",
                self.permission_isolation_rate, self.permission_violations
            ));
        }
        for category in REQUIRED_CATEGORIES {
            if self.categories.get(category).map_or(0, |c| c.total) == 0 {
                failures.push(format!("required category `{category}` has no cases"));
            }
        }
        failures
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
        let role_restricted = !case.roles.is_empty();
        let perm = if role_restricted {
            Some(PermissionContext::new(
                "eval-tenant",
                "eval-user",
                case.roles.clone(),
            ))
        } else {
            None
        };
        let retriever =
            Retriever::build_with_permissions(db, doc_versions, threshold, perm.as_ref());
        let result = retriever.search_with_citations(&case.question, 5, db);
        let latency_ms = start.elapsed().as_millis() as u64;

        let (citations, has_evidence) = match &result {
            SearchResult::Hits(citations) => (citations.as_slice(), !citations.is_empty()),
            SearchResult::InsufficientEvidence { .. } => (&[][..], false),
        };
        let top = citations.first();
        let top_doc_id = top.map(|c| c.doc_id.clone());
        let top_section = top.map(|c| c.section.clone());
        let top_score = top.map(|c| c.score).unwrap_or(0.0);

        let expected_match = top
            .map(|c| c.doc_id == case.expected_doc_id && c.section == case.expected_section)
            .unwrap_or(false);
        let hit = !case.expect_refusal && expected_match && top_score >= case.min_score;
        let permission_isolated = citations.iter().all(|c| {
            db.doc(&c.doc_id)
                .map(|d| {
                    case.roles.is_empty()
                        || d.permissions.is_empty()
                        || d.permissions.iter().any(|p| case.roles.contains(p))
                })
                .unwrap_or(false)
        });

        let failure = if permission_isolated {
            match (case.expect_refusal, has_evidence, expected_match, top_score) {
                (true, false, _, _) => None,
                (true, true, _, _) => Some(format!(
                    "expected refusal but returned {} citation(s), top {}@{}",
                    citations.len(),
                    top_doc_id.as_deref().unwrap_or("?"),
                    top_section.as_deref().unwrap_or("?")
                )),
                (false, false, _, _) => Some(format!(
                    "expected {}@{} but returned insufficient evidence",
                    case.expected_doc_id, case.expected_section
                )),
                (false, true, false, _) => Some(format!(
                    "expected {}@{} but top was {}@{}",
                    case.expected_doc_id,
                    case.expected_section,
                    top_doc_id.as_deref().unwrap_or("?"),
                    top_section.as_deref().unwrap_or("?")
                )),
                (false, true, true, score) if score < case.min_score => Some(format!(
                    "top {:.3} is below min_score {:.3}",
                    score, case.min_score
                )),
                _ => None,
            }
        } else {
            Some("returned a citation the user is not permitted to see".to_string())
        };

        results.push(EvalCaseResult {
            id: case.id.clone(),
            category: case.category.clone(),
            question: case.question.clone(),
            expect_refusal: case.expect_refusal,
            hit,
            passed: failure.is_none(),
            failure,
            top_doc_id,
            top_section,
            top_score,
            citation_count: citations.len(),
            has_evidence,
            role_restricted,
            permission_isolated,
            latency_ms,
        });
    }
    EvalMetrics::compute(&results)
}

/// Load an evaluation set from a JSON array or an object containing `cases`.
pub fn load_eval_set(path: &Path) -> Result<EvalSet, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let parsed: EvalFile =
        serde_json::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let set = match parsed {
        EvalFile::Set(set) => set,
        EvalFile::List(cases) => EvalSetFile {
            version: 1,
            threshold: None,
            fixtures: Vec::new(),
            cases,
        },
    };
    validate_eval_set(path, &set)?;
    Ok(EvalSet {
        version: set.version,
        threshold: set.threshold,
        cases: set.cases,
        fixtures: set.fixtures,
    })
}

impl EvalSet {
    /// Build the database used by this fixed set: the production documents plus
    /// the set's synthetic role-restricted fixtures. Fixture ids must not
    /// collide with production ids, and the merged shape is re-validated.
    pub fn database_with_fixtures(&self, base: &DocDatabase) -> Result<DocDatabase, String> {
        let mut merged = base.clone();
        for fixture in &self.fixtures {
            if merged.doc(&fixture.doc_id).is_some() {
                return Err(format!(
                    "eval fixture `{}` collides with a production document id",
                    fixture.doc_id
                ));
            }
            merged.documents.push(fixture.clone());
        }
        merged.validate_shape()?;
        Ok(merged)
    }
}

fn validate_eval_set(path: &Path, set: &EvalSetFile) -> Result<(), String> {
    if set.cases.is_empty() {
        return Err(format!("{}: eval case set is empty", path.display()));
    }
    if let Some(threshold) = set.threshold {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(format!(
                "{}: threshold {threshold} is outside [0,1]",
                path.display()
            ));
        }
    }
    if !set.fixtures.is_empty() {
        DocDatabase {
            documents: set.fixtures.clone(),
        }
        .validate_shape()
        .map_err(|e| format!("{}: invalid fixture document: {e}", path.display()))?;
        for fixture in &set.fixtures {
            if !set
                .cases
                .iter()
                .any(|case| case.expected_doc_id == fixture.doc_id)
            {
                return Err(format!(
                    "{}: fixture `{}` is not referenced by any case expected_doc_id",
                    path.display(),
                    fixture.doc_id
                ));
            }
        }
    }
    let mut ids = BTreeSet::new();
    for case in &set.cases {
        if case.id.trim().is_empty() {
            return Err(format!("{}: case with empty id", path.display()));
        }
        if !ids.insert(case.id.clone()) {
            return Err(format!(
                "{}: duplicate case id `{}`",
                path.display(),
                case.id
            ));
        }
        if case.question.trim().is_empty() {
            return Err(format!(
                "{}: case `{}` has an empty question",
                path.display(),
                case.id
            ));
        }
        if case.category.trim().is_empty() {
            return Err(format!(
                "{}: case `{}` has an empty category",
                path.display(),
                case.id
            ));
        }
        if !(0.0..=1.0).contains(&case.min_score) {
            return Err(format!(
                "{}: case `{}` min_score {} is outside [0,1]",
                path.display(),
                case.id,
                case.min_score
            ));
        }
        if !case.expect_refusal
            && (case.expected_doc_id.trim().is_empty() || case.expected_section.trim().is_empty())
        {
            return Err(format!(
                "{}: case `{}` must name expected_doc_id and expected_section unless it expects refusal",
                path.display(),
                case.id
            ));
        }
    }
    Ok(())
}

/// Load evaluation cases from a JSON file.
pub fn load_eval_cases(path: &Path) -> Result<Vec<EvalCase>, String> {
    Ok(load_eval_set(path)?.cases)
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

    fn case(id: &str, category: &str, question: &str, doc: &str, section: &str) -> EvalCase {
        EvalCase {
            id: id.to_string(),
            category: category.to_string(),
            question: question.to_string(),
            expect_refusal: false,
            expected_doc_id: doc.to_string(),
            expected_section: section.to_string(),
            min_score: 0.1,
            roles: vec![],
        }
    }

    fn refusal_case(id: &str, category: &str, question: &str) -> EvalCase {
        EvalCase {
            id: id.to_string(),
            category: category.to_string(),
            question: question.to_string(),
            expect_refusal: true,
            expected_doc_id: String::new(),
            expected_section: String::new(),
            min_score: 0.1,
            roles: vec![],
        }
    }

    #[test]
    fn eval_case_hits_expected() {
        let (db, dv) = make_db();
        let cases = vec![case(
            "TC-001",
            "general",
            "quality management",
            "SOP-CHANGE",
            "4.3",
        )];
        let metrics = run_eval(&db, &dv, &cases, 0.1);
        assert_eq!(metrics.total_cases, 1);
        assert_eq!(metrics.hits, 1);
        assert_eq!(metrics.passed_cases, 1);
        assert_eq!(metrics.precision_at_1, 1.0);
        assert!(metrics.citation_support_rate > 0.0);
    }

    #[test]
    fn eval_case_misses_wrong_expected() {
        let (db, dv) = make_db();
        let cases = vec![case(
            "TC-002",
            "general",
            "quality management",
            "NONEXISTENT",
            "9.9",
        )];
        let metrics = run_eval(&db, &dv, &cases, 0.1);
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.passed_cases, 0);
        assert_eq!(metrics.precision_at_1, 0.0);
        assert!(metrics.cases[0]
            .failure
            .as_deref()
            .unwrap_or("")
            .contains("expected NONEXISTENT@9.9"));
    }

    #[test]
    fn eval_permission_isolation() {
        let (db, dv) = make_db();
        let cases = vec![EvalCase {
            roles: vec!["viewer".to_string()],
            expect_refusal: true,
            expected_doc_id: String::new(),
            expected_section: String::new(),
            ..case(
                "TC-003",
                "general",
                "confidential admin",
                "SECRET-DOC",
                "s1",
            )
        }];
        let metrics = run_eval(&db, &dv, &cases, 0.3);
        assert_eq!(metrics.permission_isolation_rate, 1.0);
        assert_eq!(
            metrics.passed_cases, 1,
            "viewer must get an honest refusal: {:?}",
            metrics.cases
        );
    }

    #[test]
    fn refusal_case_passes_when_no_evidence() {
        let (db, dv) = make_db();
        let cases = vec![refusal_case(
            "TC-004",
            "insufficient_context",
            "completely unrelated topic",
        )];
        let metrics = run_eval(&db, &dv, &cases, 0.99);
        assert_eq!(metrics.refusal_accuracy, 1.0);
        assert_eq!(metrics.passed_cases, 1);
    }

    #[test]
    fn refusal_case_fails_when_evidence_returned() {
        let (db, dv) = make_db();
        let cases = vec![refusal_case("TC-005", "adversarial", "quality management")];
        let metrics = run_eval(&db, &dv, &cases, 0.1);
        assert_eq!(metrics.refusal_accuracy, 0.0);
        assert_eq!(metrics.passed_cases, 0);
        assert!(metrics.cases[0]
            .failure
            .as_deref()
            .unwrap_or("")
            .contains("expected refusal"));
    }

    #[test]
    fn min_score_is_enforced() {
        let (db, dv) = make_db();
        let mut candidate = case(
            "TC-006",
            "general",
            "quality management",
            "SOP-CHANGE",
            "4.3",
        );
        candidate.min_score = 1.0;
        let metrics = run_eval(&db, &dv, &[candidate], 0.1);
        assert_eq!(metrics.hits, 0);
        assert_eq!(metrics.passed_cases, 0);
        assert!(metrics.cases[0]
            .failure
            .as_deref()
            .unwrap_or("")
            .contains("below min_score"));
    }

    #[test]
    fn gate_detects_missing_required_category() {
        let cases = vec![EvalCaseResult {
            id: "1".to_string(),
            category: "general".to_string(),
            question: "q".to_string(),
            expect_refusal: false,
            hit: true,
            passed: true,
            failure: None,
            top_doc_id: Some("D".to_string()),
            top_section: Some("s".to_string()),
            top_score: 0.9,
            citation_count: 1,
            has_evidence: true,
            role_restricted: false,
            permission_isolated: true,
            latency_ms: 1,
        }];
        let metrics = EvalMetrics::compute(&cases);
        let failures = metrics.gate_failures();
        assert!(failures.iter().any(|f| f.contains("ambiguous")));
        assert!(failures.iter().any(|f| f.contains("at least")));
    }

    #[test]
    fn load_eval_set_from_object() {
        let json = r#"{
            "version": 2,
            "threshold": 0.33,
            "cases": [{"id":"TC-1","category":"general","question":"test","expected_doc_id":"D1","expected_section":"s1"}]
        }"#;
        let path = std::env::temp_dir().join("test_eval_set_object.json");
        std::fs::write(&path, json).unwrap();
        let set = load_eval_set(&path).unwrap();
        assert_eq!(set.version, 2);
        assert_eq!(set.threshold, Some(0.33));
        assert_eq!(set.cases.len(), 1);
        assert_eq!(set.cases[0].id, "TC-1");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_eval_cases_from_json_array() {
        let json =
            r#"[{"id":"TC-1","question":"test","expected_doc_id":"D1","expected_section":"s1"}]"#;
        let path = std::env::temp_dir().join("test_eval_cases.json");
        std::fs::write(&path, json).unwrap();
        let cases = load_eval_cases(&path).unwrap();
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].id, "TC-1");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn duplicate_ids_rejected() {
        let json = r#"[
            {"id":"TC-1","question":"test","expected_doc_id":"D1","expected_section":"s1"},
            {"id":"TC-1","question":"test","expected_doc_id":"D1","expected_section":"s1"}
        ]"#;
        let path = std::env::temp_dir().join("test_eval_duplicate.json");
        std::fs::write(&path, json).unwrap();
        let error = load_eval_cases(&path).unwrap_err();
        assert!(error.contains("duplicate case id"));
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
                category: "general".to_string(),
                question: "q1".to_string(),
                expect_refusal: false,
                hit: true,
                passed: true,
                failure: None,
                top_doc_id: Some("D1".to_string()),
                top_section: Some("s1".to_string()),
                top_score: 0.9,
                citation_count: 2,
                has_evidence: true,
                role_restricted: true,
                permission_isolated: true,
                latency_ms: 10,
            },
            EvalCaseResult {
                id: "2".to_string(),
                category: "general".to_string(),
                question: "q2".to_string(),
                expect_refusal: false,
                hit: false,
                passed: false,
                failure: Some("miss".to_string()),
                top_doc_id: None,
                top_section: None,
                top_score: 0.0,
                citation_count: 0,
                has_evidence: false,
                role_restricted: true,
                permission_isolated: true,
                latency_ms: 5,
            },
        ];
        let m = EvalMetrics::compute(&cases);
        assert_eq!(m.total_cases, 2);
        assert_eq!(m.hits, 1);
        assert_eq!(m.passed_cases, 1);
        assert!((m.precision_at_1 - 0.5).abs() < 1e-9);
        assert!((m.citation_support_rate - 0.5).abs() < 1e-9);
        assert_eq!(m.permission_cases, 2);
        assert_eq!(m.permission_allowed_cases, 2);
        assert_eq!(m.permission_denied_cases, 0);
        assert_eq!(m.permission_violations, 0);
        assert!((m.permission_isolation_rate - 1.0).abs() < 1e-9);
        assert!((m.avg_latency_ms - 7.5).abs() < 1e-9);
    }

    #[test]
    fn role_cases_cover_allow_and_deny() {
        let (db, dv) = make_db();
        let cases = vec![
            EvalCase {
                roles: vec!["admin".to_string()],
                ..case(
                    "TC-ROLE-ALLOW",
                    "permission",
                    "confidential admin",
                    "SECRET-DOC",
                    "s1",
                )
            },
            EvalCase {
                roles: vec!["viewer".to_string()],
                expect_refusal: true,
                ..case(
                    "TC-ROLE-DENY",
                    "permission",
                    "confidential admin",
                    "SECRET-DOC",
                    "s1",
                )
            },
        ];
        let metrics = run_eval(&db, &dv, &cases, 0.3);
        assert_eq!(metrics.passed_cases, 2);
        assert_eq!(metrics.permission_cases, 2);
        assert_eq!(metrics.permission_allowed_cases, 1);
        assert_eq!(metrics.permission_denied_cases, 1);
        assert_eq!(metrics.permission_violations, 0);
        assert_eq!(metrics.permission_isolation_rate, 1.0);
    }

    #[test]
    fn gate_rejects_vacuous_permission_metric() {
        let mut cases = Vec::new();
        for category in REQUIRED_CATEGORIES {
            for index in 0..2 {
                cases.push(EvalCaseResult {
                    id: format!("{category}-{index}"),
                    category: category.to_string(),
                    question: "q".to_string(),
                    expect_refusal: false,
                    hit: true,
                    passed: true,
                    failure: None,
                    top_doc_id: Some("D".to_string()),
                    top_section: Some("s".to_string()),
                    top_score: 0.9,
                    citation_count: 1,
                    has_evidence: true,
                    role_restricted: false,
                    permission_isolated: true,
                    latency_ms: 1,
                });
            }
        }
        let metrics = EvalMetrics::compute(&cases);
        let failures = metrics.gate_failures();
        assert!(failures
            .iter()
            .any(|failure| failure.contains("role-restricted permission")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("permitted answer")));
        assert!(failures
            .iter()
            .any(|failure| failure.contains("honest refusal")));
    }

    #[test]
    fn gate_rejects_permission_leak() {
        let cases = vec![EvalCaseResult {
            id: "LEAK-01".to_string(),
            category: "permission".to_string(),
            question: "q".to_string(),
            expect_refusal: true,
            hit: false,
            passed: false,
            failure: Some("returned a citation the user is not permitted to see".to_string()),
            top_doc_id: Some("SECRET".to_string()),
            top_section: Some("s1".to_string()),
            top_score: 0.9,
            citation_count: 1,
            has_evidence: true,
            role_restricted: true,
            permission_isolated: false,
            latency_ms: 1,
        }];
        let metrics = EvalMetrics::compute(&cases);
        let failures = metrics.gate_failures();
        assert_eq!(metrics.permission_violations, 1);
        assert_eq!(metrics.permission_isolation_rate, 0.0);
        assert!(failures
            .iter()
            .any(|failure| failure.contains("permission isolation rate is 0.00")));
    }

    #[test]
    fn eval_fixtures_merge_without_mutating_base() {
        let (db, _) = make_db();
        let json = r#"{
            "version": 1,
            "threshold": 0.42,
            "fixtures": [{
                "doc_id": "FIX-1",
                "title": "fixture",
                "permissions": ["auditor"],
                "versions": [{
                    "version": "1.0",
                    "effective_from": "2026-01-01",
                    "sections": [{"section": "s1", "verbatim_text": "fixture text"}]
                }]
            }],
            "cases": [{
                "id": "F-1",
                "category": "permission",
                "question": "fixture text",
                "expected_doc_id": "FIX-1",
                "expected_section": "s1",
                "min_score": 0.42,
                "roles": ["auditor"]
            }]
        }"#;
        let path = std::env::temp_dir().join("test_eval_fixture.json");
        std::fs::write(&path, json).unwrap();
        let set = load_eval_set(&path).unwrap();
        let merged = set.database_with_fixtures(&db).unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(db.documents.len(), 2);
        assert_eq!(merged.documents.len(), 3);
        assert!(merged.doc("FIX-1").is_some());
        assert_eq!(
            merged.latest_versions().get("FIX-1"),
            Some(&"1.0".to_string())
        );
    }

    #[test]
    fn eval_fixture_id_collision_is_rejected() {
        let (db, _) = make_db();
        let json = r#"{
            "version": 1,
            "fixtures": [{
                "doc_id": "SOP-CHANGE",
                "permissions": ["admin"],
                "versions": [{
                    "version": "9.9",
                    "effective_from": "2026-01-01",
                    "sections": [{"section": "s1", "verbatim_text": "collision"}]
                }]
            }],
            "cases": [{
                "id": "F-1",
                "category": "permission",
                "question": "collision",
                "expected_doc_id": "SOP-CHANGE",
                "expected_section": "s1",
                "roles": ["admin"]
            }]
        }"#;
        let path = std::env::temp_dir().join("test_eval_fixture_collision.json");
        std::fs::write(&path, json).unwrap();
        let set = load_eval_set(&path).unwrap();
        let error = set.database_with_fixtures(&db).unwrap_err();
        let _ = std::fs::remove_file(&path);
        assert!(error.contains("collides with a production document id"));
    }
}
