#![forbid(unsafe_code)]

//! Inter-annotator agreement (Cohen's kappa) for eval ground truth.
//!
//! Provides Cohen's κ computation for two annotators' independent verdict labels
//! on the same case set. Used to establish inter-annotator agreement before
//! treating expected verdicts as ground truth in eval corpora.

use std::collections::BTreeMap;

/// Compute Cohen's kappa for two raters' labels.
///
/// Returns `None` when the computation is undefined (different-length inputs,
/// empty inputs, or perfect chance agreement where the denominator is zero).
/// Values range from -1.0 (perfect disagreement) to 1.0 (perfect agreement),
/// with 0.0 indicating agreement at chance level.
pub fn cohens_kappa(rater1: &[&str], rater2: &[&str]) -> Option<f64> {
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
    Some((p_o - p_e) / denominator)
}

/// Cohen's κ qualitative interpretation bands (Landis & Koch, 1977).
pub fn kappa_interpretation(kappa: f64) -> &'static str {
    match kappa {
        k if k < 0.0 => "Poor (less than chance)",
        k if k < 0.20 => "Slight",
        k if k < 0.40 => "Fair",
        k if k < 0.60 => "Moderate",
        k if k < 0.80 => "Substantial",
        _ => "Almost perfect",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
