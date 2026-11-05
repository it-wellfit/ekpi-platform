#![forbid(unsafe_code)]

//! Edge-case regression suite for the deterministic engine.
//!
//! Loads `eval/cases/edge_cases.jsonl` (independent of the 9-seed / 46-casebook
//! counts asserted elsewhere) and verifies each case's verdict, citation set,
//! and abstain semantics against `expected_verdict`.

use std::collections::BTreeSet;
use std::path::Path;

use ekp_eval::types;
use ekp_platform::engine::Platform;

fn edge_cases_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join("eval").join("cases").join("edge_cases.jsonl"))
        .expect("manifest root has a parent")
}

fn data_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn platform_fixture() -> Platform {
    let mut platform = Platform::load(&data_dir()).expect("platform data loads");
    platform.retrieval_threshold = 0.70;
    platform
}

#[test]
fn edge_cases_match_expected_verdict_and_citations() {
    let platform = platform_fixture();
    let (cases, warnings) = types::load_cases(&edge_cases_path()).expect("edge cases load");
    assert!(
        warnings.is_empty(),
        "edge cases must be shape-clean: {warnings:?}"
    );
    assert!(
        cases.len() >= 10,
        "edge case set must cover representative scenarios; got {}",
        cases.len()
    );

    for case in &cases {
        let outcome = platform.decide(case);
        let exp = case.expected().expect("expected_verdict known");
        assert_eq!(
            outcome.verdict,
            exp,
            "case {}: verdict expected {exp:?} got {:?}; notes {}",
            case.case_id,
            outcome.verdict,
            outcome.notes.join(" | ")
        );

        let got: BTreeSet<&str> = outcome.citations.iter().map(String::as_str).collect();
        let want: BTreeSet<&str> = case.citations.iter().map(String::as_str).collect();
        assert_eq!(
            got, want,
            "case {}: citation set mismatch (engine {:?} vs seed {:?})",
            case.case_id, outcome.citations, case.citations,
        );

        // insufficient_evidence always abstains without citations.
        if exp == ekp_eval::types::Ver::InsufficientEvidence {
            assert!(
                case.abstain_required && outcome.citations.is_empty(),
                "case {}: insufficient verdict must abstain without citations",
                case.case_id
            );
        } else {
            assert!(
                !case.abstain_required,
                "case {}: abstain flag on non-abstaining verdict",
                case.case_id
            );
        }
    }
}

#[test]
fn edge_case_count_meets_fifty_plus_target() {
    // M-12: eval corpus (seed + casebook + edge) must carry >= 50 scored cases.
    let seed = types::load_cases(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("eval")
            .join("cases")
            .join("seed_cases.jsonl"),
    )
    .expect("seed loads")
    .0
    .len();
    let casebook = types::load_cases(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("eval")
            .join("cases")
            .join("casebook_v3.jsonl"),
    )
    .expect("casebook loads")
    .0
    .len();
    let edge = types::load_cases(&edge_cases_path())
        .expect("edge loads")
        .0
        .len();
    let total = seed + casebook + edge;
    assert!(
        total >= 50,
        "eval corpus must reach 50+ cases; got {total} (seed={seed} casebook={casebook} edge={edge})"
    );
}
