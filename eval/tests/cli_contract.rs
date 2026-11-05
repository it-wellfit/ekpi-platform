#![forbid(unsafe_code)]

//! Real-process CLI contract for `ekp_eval`.
//!
//! These tests spawn the built binary so argument validation, exit codes and
//! no-write behavior are exercised end to end, not only through unit helpers.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use ekp_eval::types;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ekp_eval"))
        .args(args)
        .output()
        .expect("spawn ekp_eval")
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "ekp-eval-cli-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn corpus_manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cases")
        .join("corpus.json")
}

fn seed_cases() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("cases")
        .join("seed_cases.jsonl")
}

fn corpus_case_ids() -> Vec<String> {
    let (corpus, warnings) = types::load_corpus(&corpus_manifest()).expect("load corpus");
    assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    corpus.cases.into_iter().map(|case| case.case_id).collect()
}

fn write_labels(path: &Path, annotator: &str, ids: &[String]) {
    let verdicts = ["comply", "violate", "escalate", "insufficient_evidence"];
    let mut body = String::new();
    for (index, id) in ids.iter().enumerate() {
        body.push_str(&format!(
            "{{\"case_id\":{},\"verdict\":\"{}\",\"annotator\":{}}}\n",
            serde_json::to_string(id).expect("serialize case id"),
            verdicts[index % verdicts.len()],
            serde_json::to_string(annotator).expect("serialize annotator")
        ));
    }
    fs::write(path, body).expect("write annotation file");
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn selftest_passes_alone() {
    let output = run(&["--selftest"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn selftest_refuses_extra_arguments() {
    for args in [
        &["--selftest", "--bogus"][..],
        &["--selftest", "--corpus-selftest"][..],
    ] {
        let output = run(args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "args={args:?} stderr={}",
            stderr(&output)
        );
    }
}

#[test]
fn corpus_selftest_refuses_extra_arguments() {
    let output = run(&["--corpus-selftest", "--bogus"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
}

#[test]
fn unknown_flag_does_not_write_a_report() {
    let dir = temp_dir("unknown-flag");
    let out = dir.to_str().expect("utf-8 temp path");
    let output = run(&["--out", out, "--bogus"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(!dir.join("report.json").exists());
    assert!(!dir.join("report.md").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cases_and_corpus_are_mutually_exclusive() {
    let dir = temp_dir("case-and-corpus");
    let out = dir.to_str().expect("utf-8 temp path");
    let seed = seed_cases();
    let manifest = corpus_manifest();
    let output = run(&[
        "--cases",
        seed.to_str().expect("utf-8 seed path"),
        "--corpus",
        manifest.to_str().expect("utf-8 manifest path"),
        "--out",
        out,
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(!dir.join("report.json").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn empty_or_duplicate_systems_are_rejected() {
    let dir = temp_dir("systems");
    let out = dir.to_str().expect("utf-8 temp path");
    for selection in ["", ",", "stub:oracle,stub:oracle"] {
        let output = run(&["--systems", selection, "--out", out]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "selection={selection:?} stderr={}",
            stderr(&output)
        );
    }
    assert!(!dir.join("report.json").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn kappa_requires_two_files() {
    for args in [&["--kappa"][..], &["--kappa", "only-one.jsonl"][..]] {
        let output = run(args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "args={args:?} stderr={}",
            stderr(&output)
        );
    }
}

#[test]
fn kappa_accepts_exact_corpus_coverage_and_reports_it() {
    let ids = corpus_case_ids();
    assert_eq!(ids.len(), 67);
    let dir = temp_dir("kappa-pass");
    let a = dir.join("annotator-a.jsonl");
    let b = dir.join("annotator-b.jsonl");
    write_labels(&a, "annotator-a", &ids);
    write_labels(&b, "annotator-b", &ids);
    let output = run(&[
        "--kappa",
        a.to_str().expect("utf-8 path"),
        b.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("kappa output must be JSON");
    assert_eq!(value["status"], "PASS");
    assert_eq!(value["corpus_id"], "ekpi-fixed-verdict-corpus-v3");
    assert_eq!(value["corpus_cases"], 67);
    assert_eq!(value["n"], 67);
    assert_eq!(value["cohens_kappa"], 1.0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn kappa_rejects_a_missing_corpus_case() {
    let mut ids = corpus_case_ids();
    ids.pop();
    let dir = temp_dir("kappa-missing");
    let a = dir.join("annotator-a.jsonl");
    let b = dir.join("annotator-b.jsonl");
    write_labels(&a, "annotator-a", &ids);
    write_labels(&b, "annotator-b", &ids);
    let output = run(&[
        "--kappa",
        a.to_str().expect("utf-8 path"),
        b.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("fixed corpus"),
        "{}",
        stderr(&output)
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn kappa_rejects_a_foreign_case_id() {
    let mut ids = corpus_case_ids();
    ids[0] = "FOREIGN-999".to_string();
    let dir = temp_dir("kappa-foreign");
    let a = dir.join("annotator-a.jsonl");
    let b = dir.join("annotator-b.jsonl");
    write_labels(&a, "annotator-a", &ids);
    write_labels(&b, "annotator-b", &ids);
    let output = run(&[
        "--kappa",
        a.to_str().expect("utf-8 path"),
        b.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("not in the fixed corpus"),
        "{}",
        stderr(&output)
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn kappa_rejects_the_same_annotator_id() {
    let ids = corpus_case_ids();
    let dir = temp_dir("kappa-same");
    let a = dir.join("annotator-a.jsonl");
    let b = dir.join("annotator-b.jsonl");
    write_labels(&a, "same-person", &ids);
    write_labels(&b, "same-person", &ids);
    let output = run(&[
        "--kappa",
        a.to_str().expect("utf-8 path"),
        b.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("different annotator ids"),
        "{}",
        stderr(&output)
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn kappa_rejects_a_leaked_expected_verdict() {
    let ids = corpus_case_ids();
    let dir = temp_dir("kappa-leak");
    let a = dir.join("annotator-a.jsonl");
    let b = dir.join("annotator-b.jsonl");
    write_labels(&a, "annotator-a", &ids);
    let mut leaked = String::new();
    for id in &ids {
        leaked.push_str(&format!(
            "{{\"case_id\":{},\"verdict\":\"comply\",\"annotator\":\"annotator-b\",\"expected_verdict\":\"comply\"}}\n",
            serde_json::to_string(id).expect("serialize case id"),
        ));
    }
    fs::write(&b, leaked).expect("write leaked file");
    let output = run(&[
        "--kappa",
        a.to_str().expect("utf-8 path"),
        b.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("unknown field"),
        "{}",
        stderr(&output)
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn make_annotation_sheet_writes_blinded_rows() {
    let dir = temp_dir("sheet");
    let sheet = dir.join("sheet.jsonl");
    let output = run(&[
        "--make-annotation-sheet",
        sheet.to_str().expect("utf-8 path"),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = fs::read_to_string(&sheet).expect("read sheet");
    let rows: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(rows.len(), 67);
    for row in rows {
        let value: serde_json::Value = serde_json::from_str(row).expect("sheet row JSON");
        assert!(value["verdict"].is_null());
        assert!(value["annotator"].is_null());
        assert!(value.get("expected_verdict").is_none());
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn make_annotation_sheet_rejects_unknown_flags() {
    let dir = temp_dir("sheet-flag");
    let sheet = dir.join("sheet.jsonl");
    let output = run(&[
        "--make-annotation-sheet",
        sheet.to_str().expect("utf-8 path"),
        "--bogus",
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(!sheet.exists());
    let _ = fs::remove_dir_all(&dir);
}
