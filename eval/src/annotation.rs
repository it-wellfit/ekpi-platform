#![forbid(unsafe_code)]

//! Inter-annotator agreement (Cohen's kappa) for eval ground truth.
//!
//! Provides Cohen's κ computation and a fail-closed study validator for two
//! annotators' independent verdict labels on the same case set. The validator
//! rejects duplicate ids, mixed annotator ids, mismatched case sets, unknown
//! labels and studies below the minimum size. It never invents a κ value: an
//! undefined or missing study is an error, not a pass.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::types::parse_verdict;

/// Minimum number of jointly annotated cases for a study to be reportable.
pub const MIN_ANNOTATED_CASES: usize = 50;

/// Proposed acceptance threshold (M-11). Product-owner sign-off is still
/// required before this becomes a release gate.
pub const KAPPA_ACCEPTANCE: f64 = 0.80;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Annotation {
    pub case_id: String,
    pub verdict: String,
    pub annotator: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug)]
pub struct Study {
    pub annotator_a: String,
    pub annotator_b: String,
    pub n: usize,
    pub observed_agreement: f64,
    pub expected_agreement: f64,
    pub kappa: f64,
    pub interpretation: &'static str,
    pub acceptance_threshold: f64,
    pub passed: bool,
    /// A caution flag for review, not a failure: two independently produced
    /// files can legitimately agree on every label.
    pub identical_label_vectors: bool,
}

/// Compute Cohen's kappa for two raters' labels.
///
/// Returns `None` when the computation is undefined (different-length inputs,
/// empty inputs, or perfect chance agreement where the denominator is zero).
/// Values range from -1.0 (perfect disagreement) to 1.0 (perfect agreement),
/// with 0.0 indicating agreement at chance level.
pub fn cohens_kappa(rater1: &[&str], rater2: &[&str]) -> Option<f64> {
    agreement_stats(rater1, rater2).map(|(_, _, kappa)| kappa)
}

/// Returns (observed agreement, expected chance agreement, kappa).
fn agreement_stats(rater1: &[&str], rater2: &[&str]) -> Option<(f64, f64, f64)> {
    if rater1.len() != rater2.len() || rater1.is_empty() {
        return None;
    }
    let n = rater1.len() as f64;

    // Observed agreement: proportion of cases where both raters agree.
    let agree = rater1
        .iter()
        .zip(rater2.iter())
        .filter(|(a, b)| a == b)
        .count() as f64;
    let p_o = agree / n;

    // Expected agreement by chance: sum over labels of
    // P(rater1=label) * P(rater2=label).
    let mut counts1: BTreeMap<&str, usize> = BTreeMap::new();
    let mut counts2: BTreeMap<&str, usize> = BTreeMap::new();
    for (a, b) in rater1.iter().zip(rater2.iter()) {
        *counts1.entry(a).or_default() += 1;
        *counts2.entry(b).or_default() += 1;
    }
    let mut p_e = 0.0;
    for (label, c1) in &counts1 {
        if let Some(c2) = counts2.get(label) {
            p_e += (*c1 as f64 / n) * (*c2 as f64 / n);
        }
    }

    let denominator = 1.0 - p_e;
    if denominator.abs() < f64::EPSILON {
        return None;
    }
    Some((p_o, p_e, (p_o - p_e) / denominator))
}

/// Cohen's κ qualitative interpretation bands (Landis & Koch, 1977).
///
/// The published bands are inclusive at their upper bound: 0.61–0.80 is
/// "Substantial" and 0.81–1.00 is "Almost perfect". Treating 0.80 as
/// "Almost perfect" would overstate the result at the proposed acceptance
/// threshold.
pub fn kappa_interpretation(kappa: f64) -> &'static str {
    match kappa {
        k if !k.is_finite() => "Undefined",
        k if k < 0.0 => "Poor (less than chance)",
        k if k <= 0.20 => "Slight",
        k if k <= 0.40 => "Fair",
        k if k <= 0.60 => "Moderate",
        k if k <= 0.80 => "Substantial",
        _ => "Almost perfect",
    }
}

/// Read one annotator's JSONL file. Blank lines and `#` comments are ignored.
pub fn load_annotations(path: &Path) -> Result<Vec<Annotation>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut records = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let record: Annotation = serde_json::from_str(line)
            .map_err(|e| format!("{}:{}: {e}", path.display(), lineno + 1))?;
        records.push(record);
    }
    validate_records(&records, "annotation file")?;
    Ok(records)
}

fn validate_records(records: &[Annotation], role: &str) -> Result<(), String> {
    if records.is_empty() {
        return Err(format!("{role}: no annotation records"));
    }
    let mut seen = BTreeSet::new();
    for record in records {
        if record.case_id.trim().is_empty() {
            return Err(format!("{role}: empty case_id"));
        }
        if record.annotator.trim().is_empty() {
            return Err(format!("{role}: {} has an empty annotator", record.case_id));
        }
        if parse_verdict(&record.verdict).is_none() {
            return Err(format!(
                "{role}: {} has unknown verdict {:?}",
                record.case_id, record.verdict
            ));
        }
        if !seen.insert(record.case_id.as_str()) {
            return Err(format!("{role}: duplicate case_id {:?}", record.case_id));
        }
    }
    Ok(())
}

/// Validate two independent annotation files and compute the study statistics.
///
/// This fails closed: fewer than [`MIN_ANNOTATED_CASES`] cases, the same
/// annotator id on both files, a case present on only one side, a duplicate id,
/// an unknown verdict, or an undefined κ all return an error instead of a
/// report. The caller must record the error as `NOT MEASURED`, not as a pass.
pub fn evaluate_study(a: &[Annotation], b: &[Annotation]) -> Result<Study, String> {
    validate_records(a, "annotator A")?;
    validate_records(b, "annotator B")?;

    let annotator_a = a[0].annotator.clone();
    let annotator_b = b[0].annotator.clone();
    if a.iter().any(|record| record.annotator != annotator_a) {
        return Err("annotator A file mixes multiple annotator ids".to_string());
    }
    if b.iter().any(|record| record.annotator != annotator_b) {
        return Err("annotator B file mixes multiple annotator ids".to_string());
    }
    if annotator_a == annotator_b {
        return Err("the two files must come from different annotator ids".to_string());
    }
    if a.len() < MIN_ANNOTATED_CASES {
        return Err(format!(
            "study has {} cases; at least {MIN_ANNOTATED_CASES} are required",
            a.len()
        ));
    }
    if a.len() != b.len() {
        return Err(format!("case counts differ: {} vs {}", a.len(), b.len()));
    }

    let mut by_id: BTreeMap<&str, &Annotation> = BTreeMap::new();
    for record in b {
        by_id.insert(record.case_id.as_str(), record);
    }
    let mut pairs: Vec<(&str, &str)> = Vec::with_capacity(a.len());
    for record in a {
        let Some(other) = by_id.get(record.case_id.as_str()) else {
            return Err(format!(
                "case {:?} is missing from annotator B",
                record.case_id
            ));
        };
        pairs.push((record.verdict.as_str(), other.verdict.as_str()));
    }

    // Equal lengths plus every A id found in B means the case sets match:
    // duplicate ids were already rejected on both sides.
    let labels_a: Vec<&str> = pairs.iter().map(|(a, _)| *a).collect();
    let labels_b: Vec<&str> = pairs.iter().map(|(_, b)| *b).collect();
    let (observed_agreement, expected_agreement, kappa) = agreement_stats(&labels_a, &labels_b)
        .ok_or_else(|| "kappa is undefined (empty input or zero chance denominator)".to_string())?;
    let passed = kappa >= KAPPA_ACCEPTANCE;
    let identical_label_vectors = labels_a == labels_b;

    Ok(Study {
        annotator_a,
        annotator_b,
        n: labels_a.len(),
        observed_agreement,
        expected_agreement,
        kappa,
        interpretation: kappa_interpretation(kappa),
        acceptance_threshold: KAPPA_ACCEPTANCE,
        passed,
        identical_label_vectors,
    })
}

/// Validate a study against the exact case ids of a frozen corpus.
///
/// [`evaluate_study`] only proves that both annotators labelled the same set.
/// This wrapper additionally requires that set to be the fixed corpus itself,
/// so a study cannot silently omit cases, invent case ids or drift from the
/// versioned corpus. The corpus case ids must be unique; `load_corpus`
/// guarantees that for the shipped corpus.
pub fn evaluate_study_for_corpus(
    a: &[Annotation],
    b: &[Annotation],
    corpus_case_ids: &BTreeSet<String>,
) -> Result<Study, String> {
    if corpus_case_ids.is_empty() {
        return Err("fixed corpus has no case ids".to_string());
    }
    let study = evaluate_study(a, b)?;
    if study.n != corpus_case_ids.len() {
        return Err(format!(
            "study covers {} cases but the fixed corpus has {}",
            study.n,
            corpus_case_ids.len()
        ));
    }
    if let Some(extra) = a
        .iter()
        .map(|record| record.case_id.as_str())
        .find(|case_id| !corpus_case_ids.contains(*case_id))
    {
        return Err(format!("case {extra:?} is not in the fixed corpus"));
    }
    // The study validator already proved that A and B have identical, unique
    // case sets. Equal counts plus every A id being in the corpus therefore
    // means the corpus is covered exactly once.
    Ok(study)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn annotation(case_id: &str, verdict: &str, annotator: &str) -> Annotation {
        Annotation {
            case_id: case_id.to_string(),
            verdict: verdict.to_string(),
            annotator: annotator.to_string(),
            notes: String::new(),
        }
    }

    fn records(count: usize, annotator: &str) -> Vec<Annotation> {
        let verdicts = ["comply", "violate", "insufficient_evidence", "escalate"];
        (0..count)
            .map(|i| {
                annotation(
                    &format!("C-{i:03}"),
                    verdicts[i % verdicts.len()],
                    annotator,
                )
            })
            .collect()
    }

    #[test]
    fn perfect_agreement_is_one() {
        let r1 = vec!["comply", "violate", "insufficient_evidence", "escalate"];
        let r2 = vec!["comply", "violate", "insufficient_evidence", "escalate"];
        let k = cohens_kappa(&r1, &r2).unwrap();
        assert!((k - 1.0).abs() < 1e-9);
        assert_eq!(kappa_interpretation(k), "Almost perfect");
    }

    #[test]
    fn chance_agreement_is_zero() {
        let r1 = vec!["comply", "comply", "violate", "violate"];
        let r2 = vec!["comply", "violate", "comply", "violate"];
        let k = cohens_kappa(&r1, &r2).unwrap();
        assert!(k.abs() < 1e-9);
        assert_eq!(kappa_interpretation(k), "Slight");
    }

    #[test]
    fn disagreement_is_negative() {
        // Perfectly opposed, balanced raters: p_o = 0, p_e = 0.5, kappa = -1.
        let r1 = vec!["comply", "comply", "violate", "violate"];
        let r2 = vec!["violate", "violate", "comply", "comply"];
        let k = cohens_kappa(&r1, &r2).unwrap();
        assert!((k + 1.0).abs() < 1e-9);
        assert_eq!(kappa_interpretation(k), "Poor (less than chance)");
    }

    #[test]
    fn mismatched_lengths_returns_none() {
        let r1 = vec!["comply", "violate"];
        let r2 = vec!["comply"];
        assert!(cohens_kappa(&r1, &r2).is_none());
    }

    #[test]
    fn empty_input_returns_none() {
        let r1: Vec<&str> = vec![];
        let r2: Vec<&str> = vec![];
        assert!(cohens_kappa(&r1, &r2).is_none());
    }

    #[test]
    fn single_label_all_agree_is_undefined() {
        // With one observed class, p_e = 1 and kappa has a zero denominator.
        // Returning None is deliberate; calling this 1.0 would overstate
        // agreement and is not a valid kappa result.
        let r1 = vec!["comply", "comply", "comply"];
        let r2 = vec!["comply", "comply", "comply"];
        assert!(cohens_kappa(&r1, &r2).is_none());
    }

    #[test]
    fn moderate_agreement_example() {
        // r1: A A B B C C, r2: A B A B C C -> 4/6 agree, p_o = 0.667
        // p_e = (2/6*2/6) + (2/6*2/6) + (2/6*2/6) = 3*(1/9) = 0.333
        // kappa = (0.667 - 0.333) / (1 - 0.333) = 0.333 / 0.667 = 0.5
        let r1 = vec!["A", "A", "B", "B", "C", "C"];
        let r2 = vec!["A", "B", "A", "B", "C", "C"];
        let k = cohens_kappa(&r1, &r2).unwrap();
        assert!((k - 0.5).abs() < 1e-9);
        assert_eq!(kappa_interpretation(k), "Moderate");
    }

    #[test]
    fn study_accepts_fifty_or_more_cases() {
        let study = evaluate_study(&records(60, "annotator-a"), &records(60, "annotator-b"))
            .expect("60-case study must be valid");
        assert_eq!(study.n, 60);
        assert!((study.kappa - 1.0).abs() < 1e-9);
        assert!(study.passed);
        assert!(study.identical_label_vectors);
    }

    #[test]
    fn study_rejects_same_annotator_id() {
        let error = evaluate_study(&records(60, "same"), &records(60, "same"))
            .expect_err("same annotator must be rejected");
        assert!(error.contains("different annotator ids"));
    }

    #[test]
    fn study_rejects_mismatched_case_sets() {
        let mut b = records(60, "annotator-b");
        b[59].case_id = "C-999".to_string();
        let error =
            evaluate_study(&records(60, "annotator-a"), &b).expect_err("missing case must fail");
        assert!(error.contains("missing from annotator B"));
    }

    #[test]
    fn study_rejects_short_study() {
        let error = evaluate_study(&records(49, "annotator-a"), &records(49, "annotator-b"))
            .expect_err("short study must be rejected");
        assert!(error.contains("at least 50"));
    }

    #[test]
    fn study_rejects_unknown_verdict() {
        let mut a = records(60, "annotator-a");
        a[0].verdict = "maybe".to_string();
        let error =
            evaluate_study(&a, &records(60, "annotator-b")).expect_err("unknown label must fail");
        assert!(error.contains("unknown verdict"));
    }

    #[test]
    fn study_rejects_duplicate_case_id() {
        let mut a = records(60, "annotator-a");
        a[1].case_id = a[0].case_id.clone();
        let error = evaluate_study(&a, &records(60, "annotator-b"))
            .expect_err("duplicate case id must fail");
        assert!(error.contains("duplicate case_id"));
    }

    #[test]
    fn study_rejects_degenerate_kappa() {
        let all_same: Vec<Annotation> = (0..60)
            .map(|i| annotation(&format!("C-{i:03}"), "comply", "annotator-a"))
            .collect();
        let all_same_b: Vec<Annotation> = (0..60)
            .map(|i| annotation(&format!("C-{i:03}"), "comply", "annotator-b"))
            .collect();
        let error = evaluate_study(&all_same, &all_same_b)
            .expect_err("undefined kappa must not be reported as a pass");
        assert!(error.contains("undefined"));
    }

    #[test]
    fn interpretation_bands_are_inclusive_at_the_upper_bound() {
        assert_eq!(kappa_interpretation(-0.01), "Poor (less than chance)");
        assert_eq!(kappa_interpretation(0.0), "Slight");
        assert_eq!(kappa_interpretation(0.20), "Slight");
        assert_eq!(kappa_interpretation(0.40), "Fair");
        assert_eq!(kappa_interpretation(0.60), "Moderate");
        assert_eq!(kappa_interpretation(0.80), "Substantial");
        assert_eq!(kappa_interpretation(0.81), "Almost perfect");
        assert_eq!(kappa_interpretation(1.0), "Almost perfect");
    }

    #[test]
    fn interpretation_handles_non_finite_values() {
        assert_eq!(kappa_interpretation(f64::NAN), "Undefined");
        assert_eq!(kappa_interpretation(f64::INFINITY), "Undefined");
    }

    #[test]
    fn annotation_record_rejects_unknown_fields() {
        let json =
            r#"{"case_id":"C-001","verdict":"comply","annotator":"a","expected_verdict":"comply"}"#;
        let error = serde_json::from_str::<Annotation>(json)
            .expect_err("a leaked expected_verdict must be rejected");
        assert!(error.to_string().contains("unknown field"), "{error}");
    }

    #[test]
    fn study_bound_to_corpus_accepts_exact_case_set() {
        let a = records(60, "annotator-a");
        let b = records(60, "annotator-b");
        let expected: BTreeSet<String> = a.iter().map(|r| r.case_id.clone()).collect();
        let study = evaluate_study_for_corpus(&a, &b, &expected).expect("exact coverage must pass");
        assert_eq!(study.n, 60);
        assert!(study.passed);
    }

    #[test]
    fn study_bound_to_corpus_rejects_foreign_case_id() {
        let a = records(60, "annotator-a");
        let b = records(60, "annotator-b");
        let mut expected: BTreeSet<String> = a.iter().map(|r| r.case_id.clone()).collect();
        expected.remove("C-000");
        expected.insert("FOREIGN-999".to_string());
        let error = evaluate_study_for_corpus(&a, &b, &expected)
            .expect_err("a case outside the corpus must fail");
        assert!(error.contains("not in the fixed corpus"), "{error}");
    }

    #[test]
    fn study_bound_to_corpus_rejects_missing_case() {
        let a = records(60, "annotator-a");
        let b = records(60, "annotator-b");
        let mut expected: BTreeSet<String> = a.iter().map(|r| r.case_id.clone()).collect();
        expected.insert("EXTRA-999".to_string());
        let error = evaluate_study_for_corpus(&a, &b, &expected)
            .expect_err("an omitted corpus case must fail");
        assert!(error.contains("fixed corpus has"), "{error}");
    }

    #[test]
    fn study_bound_to_corpus_rejects_empty_expected_set() {
        let a = records(60, "annotator-a");
        let b = records(60, "annotator-b");
        let error = evaluate_study_for_corpus(&a, &b, &BTreeSet::new())
            .expect_err("an empty corpus must fail");
        assert!(error.contains("no case ids"), "{error}");
    }
}
