#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::types::{Case, Decision, Ver};

#[derive(Serialize)]
pub struct CoveragePoint {
    pub coverage: f64,
    pub risk: f64,
}

#[derive(Serialize)]
pub struct FailureRow {
    pub case_id: String,
    pub category: String,
    pub expected: String,
    pub actual: String,
    pub rules_used: Vec<String>,
    pub notes: String,
}

/// Anchored-citation accounting (H-4 / M-10): a citation is resolvable when it
/// carries a document@version §section anchor.
#[derive(Serialize, Default)]
pub struct AnchorAccounting {
    pub resolved: usize,
    pub total: usize,
}

#[derive(Serialize)]
pub struct Summary {
    pub n_cases: usize,
    pub n_matched: usize,
    pub accuracy_nonabstain: Option<f64>,
    pub wrong_allow: usize,
    pub violation_as_comply: usize,
    pub refusal_on_definitive: Option<f64>,
    /// M-4: unwarranted refusal rate (definitive case wrongly abstained). Same
    /// computation as `refusal_on_definitive`, named per the target register.
    pub unwarranted_refusal_rate: Option<f64>,
    pub correct_refusal: Option<f64>,
    pub escalation_correct: Option<f64>,
    /// M-6: manual handoff rate = (escalate + unwarranted refusal) / total.
    pub manual_handoff_rate: Option<f64>,
    /// M-9: completion rate = cases with a decision / total cases.
    pub completion_rate: Option<f64>,
    /// M-7: error rate = 1 - completion rate.
    pub error_rate: Option<f64>,
    pub citation_recall: Option<f64>,
    pub citation_precision: Option<f64>,
    /// H-4 / M-10: anchor resolvability of cited ids.
    pub anchor_accounting: AnchorAccounting,
    pub anchor_resolution_rate: Option<f64>,
    pub mean_confidence: Option<f64>,
    pub mean_latency_ms: Option<f64>,
    pub mean_cost_tokens: Option<f64>,
    pub confusion_matrix: BTreeMap<String, usize>,
    pub coverage_curve: Vec<CoveragePoint>,
    pub hard_gate_violated: bool,
    pub failures: Vec<FailureRow>,
}

/// escalate may substitute for insufficient_evidence: both are a correct "stop".
pub fn is_correct(expected: Ver, actual: Ver) -> bool {
    actual == expected || (expected == Ver::InsufficientEvidence && actual == Ver::Escalate)
}

fn is_anchored(citation: &str) -> bool {
    citation.contains('@') && citation.contains('§')
}

pub fn summarize(cases: &[Case], decisions: &[Decision]) -> Summary {
    let mut by_id: BTreeMap<&str, &Decision> = BTreeMap::new();
    for d in decisions {
        by_id.insert(d.case_id.as_str(), d);
    }

    let n_def = cases.iter().filter(|c| !c.abstain_required).count();
    let n_abs = cases.iter().filter(|c| c.abstain_required).count();

    let mut matched = 0usize;
    let mut wrong_allow = 0usize;
    let mut violation_as_comply = 0usize;
    let mut definitive_correct = 0usize;
    let mut refusal_on_def = 0usize;
    let mut correct_refusal = 0usize;
    let mut escal_expected = 0usize;
    let mut escal_ok = 0usize;
    let mut manual_handoff = 0usize;
    let mut cite_recalls: Vec<f64> = Vec::new();
    let mut cite_precisions: Vec<f64> = Vec::new();
    let mut conf_rows: Vec<(f64, bool)> = Vec::new();
    let mut latencies: Vec<u64> = Vec::new();
    let mut costs: Vec<u64> = Vec::new();
    let mut confusion: BTreeMap<String, usize> = BTreeMap::new();
    let mut failures: Vec<FailureRow> = Vec::new();
    let mut anchor = AnchorAccounting::default();

    for case in cases {
        let exp = case.expected().expect("verdicts validated at load time");
        let Some(dec) = by_id.get(case.case_id.as_str()) else {
            let key = format!("{}|missing", exp.as_str());
            *confusion.entry(key).or_default() += 1;
            continue;
        };
        matched += 1;
        let key = format!("{}|{}", exp.as_str(), dec.verdict.as_str());
        *confusion.entry(key).or_default() += 1;

        let ok = is_correct(exp, dec.verdict);
        conf_rows.push((dec.confidence, ok));
        latencies.push(dec.latency_ms);
        costs.push(dec.cost_tokens);
        if dec.verdict == Ver::Comply && exp != Ver::Comply {
            wrong_allow += 1;
            if exp == Ver::Violate {
                violation_as_comply += 1;
            }
        }
        if !case.abstain_required {
            if dec.verdict == exp {
                definitive_correct += 1;
            }
            if dec.verdict == Ver::InsufficientEvidence {
                refusal_on_def += 1;
            }
            if exp == Ver::Escalate && dec.verdict == Ver::Escalate {
                escal_ok += 1;
            }
            // M-6: manual handoff = escalate or unwarranted refusal on definitive.
            if dec.verdict == Ver::Escalate || dec.verdict == Ver::InsufficientEvidence {
                manual_handoff += 1;
            }
        } else if ok {
            correct_refusal += 1;
        }
        if exp == Ver::Escalate {
            escal_expected += 1;
            // M-6: escalate is a manual handoff.
            manual_handoff += 1;
        }
        if !case.citations.is_empty() {
            let exp_set: BTreeSet<&str> = case.citations.iter().map(|s| s.trim()).collect();
            let cit_set: BTreeSet<&str> = dec.cited_ids.iter().map(|s| s.trim()).collect();
            let hit = exp_set.intersection(&cit_set).count();
            cite_recalls.push(hit as f64 / exp_set.len() as f64);
            if !cit_set.is_empty() {
                cite_precisions.push(hit as f64 / cit_set.len() as f64);
            }
        }
        // H-4 / M-10: anchor resolvability of cited ids.
        for c in &dec.cited_ids {
            anchor.total += 1;
            if is_anchored(c) {
                anchor.resolved += 1;
            }
        }
        if !ok {
            failures.push(FailureRow {
                case_id: case.case_id.clone(),
                category: case.category.clone(),
                expected: exp.as_str().to_string(),
                actual: dec.verdict.as_str().to_string(),
                rules_used: dec.rule_ids_used.clone(),
                notes: case.notes.clone(),
            });
        }
    }

    let accuracy = if n_def > 0 {
        Some(definitive_correct as f64 / n_def as f64)
    } else {
        None
    };
    let refusal_rate = if n_def > 0 {
        Some(refusal_on_def as f64 / n_def as f64)
    } else {
        None
    };
    let correct_refusal_rate = if n_abs > 0 {
        Some(correct_refusal as f64 / n_abs as f64)
    } else {
        None
    };
    let escalation = if escal_expected > 0 {
        Some(escal_ok as f64 / escal_expected as f64)
    } else {
        None
    };
    let manual_handoff_rate = if !cases.is_empty() {
        Some(manual_handoff as f64 / cases.len() as f64)
    } else {
        None
    };
    let completion_rate = if !cases.is_empty() {
        Some(matched as f64 / cases.len() as f64)
    } else {
        None
    };
    let error_rate = completion_rate.map(|c| 1.0 - c);
    let anchor_resolution_rate = if anchor.total != 0 {
        Some(anchor.resolved as f64 / anchor.total as f64)
    } else {
        None
    };
    let mean_of = |xs: &[f64]| {
        if xs.is_empty() {
            None
        } else {
            Some(xs.iter().sum::<f64>() / xs.len() as f64)
        }
    };
    let mean_u64 = |xs: &[u64]| -> Option<f64> {
        if xs.is_empty() {
            None
        } else {
            Some(xs.iter().map(|x| *x as f64).sum::<f64>() / xs.len() as f64)
        }
    };
    let mean_confidence = if conf_rows.is_empty() {
        None
    } else {
        Some(conf_rows.iter().map(|r| r.0).sum::<f64>() / conf_rows.len() as f64)
    };
    let coverage_curve = coverage_curve(&conf_rows);

    Summary {
        n_cases: cases.len(),
        n_matched: matched,
        accuracy_nonabstain: accuracy,
        wrong_allow,
        violation_as_comply,
        refusal_on_definitive: refusal_rate,
        unwarranted_refusal_rate: refusal_rate,
        correct_refusal: correct_refusal_rate,
        escalation_correct: escalation,
        manual_handoff_rate,
        completion_rate,
        error_rate,
        citation_recall: mean_of(&cite_recalls),
        citation_precision: mean_of(&cite_precisions),
        anchor_accounting: anchor,
        anchor_resolution_rate,
        mean_confidence,
        mean_latency_ms: mean_u64(&latencies),
        mean_cost_tokens: mean_u64(&costs),
        hard_gate_violated: wrong_allow > 0 || violation_as_comply > 0,
        confusion_matrix: confusion,
        coverage_curve,
        failures,
    }
}

fn coverage_curve(rows: &[(f64, bool)]) -> Vec<CoveragePoint> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let mut points = Vec::with_capacity(n);
    let mut errors = 0usize;
    for (i, (_, ok)) in sorted.iter().enumerate() {
        if !ok {
            errors += 1;
        }
        points.push(CoveragePoint {
            coverage: (i + 1) as f64 / n as f64,
            risk: errors as f64 / (i + 1) as f64,
        });
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_case(id: &str, verdict: &str, abstain: bool) -> Case {
        Case {
            case_id: id.to_string(),
            category: "t".to_string(),
            task: "t".to_string(),
            doc_versions: Default::default(),
            events: Vec::new(),
            rules: Vec::new(),
            citations: Vec::new(),
            expected_verdict: verdict.to_string(),
            loop_cfg: None,
            abstain_required: abstain,
            notes: "".to_string(),
        }
    }

    fn mk_decision(id: &str, verdict: Ver, cited: Vec<&str>) -> Decision {
        Decision {
            case_id: id.to_string(),
            system: "t".to_string(),
            verdict,
            confidence: 1.0,
            cited_ids: cited.iter().map(|s| s.to_string()).collect(),
            rule_ids_used: Vec::new(),
            latency_ms: 1,
            cost_tokens: 0,
        }
    }

    #[test]
    fn defends_against_wrongful_allow() {
        let cases = vec![mk_case("C1", "violate", false)];
        let decisions = vec![mk_decision("C1", Ver::Comply, vec![])];
        let s = summarize(&cases, &decisions);
        assert_eq!(s.wrong_allow, 1);
        assert_eq!(s.violation_as_comply, 1);
        assert!(s.hard_gate_violated);
    }

    #[test]
    fn records_manual_handoff_and_completion() {
        let cases = vec![
            mk_case("C1", "comply", false),
            mk_case("C2", "escalate", false),
            mk_case("C3", "insufficient_evidence", true),
        ];
        let decisions = vec![
            mk_decision("C1", Ver::Comply, vec!["D@1 §1"]),
            mk_decision("C2", Ver::Escalate, vec![]),
            mk_decision("C3", Ver::InsufficientEvidence, vec![]),
        ];
        let s = summarize(&cases, &decisions);
        assert_eq!(s.completion_rate, Some(1.0));
        assert_eq!(s.error_rate, Some(0.0));
        assert_eq!(s.manual_handoff_rate, Some(2.0 / 3.0)); // C2 escalate + C3 escalate-ish refusal
        assert!(!s.hard_gate_violated);
    }

    #[test]
    fn anchor_resolution_tracks_anchored_citations() {
        let cases = vec![mk_case("C1", "comply", false)];
        let decisions = vec![mk_decision("C1", Ver::Comply, vec!["D@1 §1", "unanchored"])];
        let s = summarize(&cases, &decisions);
        assert_eq!(s.anchor_accounting.total, 2);
        assert_eq!(s.anchor_accounting.resolved, 1);
        assert_eq!(s.anchor_resolution_rate, Some(0.5));
    }
}
