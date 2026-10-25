#![forbid(unsafe_code)]

//! Inter-annotator agreement (Cohen's kappa) for eval ground truth (Master Prompt Phase 3).

use std::collections::BTreeMap;

/// Compute Cohen's kappa for two raters' labels.
/// Returns None if the computation is undefined (e.g., single class).
pub fn cohens_kappa(rater1: &[&str], rater2: &[&str]) -> Option<f64> {
    if rater1.len() != rater2.len() || rater1.is_empty() {
        return None;
    }
    let n = rater1.len() as f64;

    // Observed agreement.
    let agree = rater1
        .iter()
        .zip(rater2.iter())
        .filter(|(a, b)| a == b)
        .count() as f64;
    let p_o = agree / n;

    // Expected agreement by chance.
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

    if (1.0 - p_e).abs() < f64::EPSILON {
        return None;
    }
    Some((p_o - p_e) / (1.0 - p_e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_agreement() {
        let r1 = vec!["a", "b", "a", "b"];
        let r2 = vec!["a", "b", "a", "b"];
        let k = cohens_kappa(&r1, &r2).unwrap();
        assert!((k - 1.0).abs() < 1e-9);
    }

    #[test]
    fn chance_agreement() {
        let r1 = vec!["a", "a", "b", "b"];
        let r2 = vec!["a", "b", "a", "b"];
        let k = cohens_kappa(&r1, &r2).unwrap();
        // p_o = 0.5, p_e = 0.5 → kappa = 0
        assert!(k.abs() < 1e-9);
    }

    #[test]
    fn mismatched_lengths() {
        let r1 = vec!["a", "b"];
        let r2 = vec!["a"];
        assert!(cohens_kappa(&r1, &r2).is_none());
    }

    #[test]
    fn empty_input() {
        let r1: Vec<&str> = vec![];
        let r2: Vec<&str> = vec![];
        assert!(cohens_kappa(&r1, &r2).is_none());
    }
}
