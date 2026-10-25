#![forbid(unsafe_code)]

//! Integration evidence for the deterministic search/eval CLI surface.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use ekp_platform::docs::DocDatabase;
use ekp_platform::eval::{load_eval_set, run_eval, REQUIRED_CATEGORIES};

const THRESHOLD: f64 = 0.42;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn latest_versions(db: &DocDatabase) -> BTreeMap<String, String> {
    let mut versions = BTreeMap::new();
    for doc in &db.documents {
        if let Some(version) = doc.versions.last() {
            versions.insert(doc.doc_id.clone(), version.version.clone());
        }
    }
    versions
}

#[test]
fn fixed_search_eval_set_passes_gate() {
    let root = manifest_dir();
    let db = DocDatabase::load(&root.join("data/docs.json")).expect("load docs");
    let set = load_eval_set(&root.join("data/eval/search_eval.json")).expect("load eval set");
    let versions = latest_versions(&db);
    let metrics = run_eval(
        &db,
        &versions,
        &set.cases,
        set.threshold.unwrap_or(THRESHOLD),
    );
    let failures = metrics.gate_failures();
    assert!(
        failures.is_empty(),
        "eval gate failures: {failures:#?}\ncase results: {:#?}",
        metrics.cases
    );
    assert_eq!(metrics.failed_cases, 0);
    for category in REQUIRED_CATEGORIES {
        assert!(
            metrics
                .categories
                .get(category)
                .is_some_and(|summary| summary.total > 0),
            "missing required category: {category}"
        );
    }
}

#[test]
fn eval_cli_passes_fixed_set() {
    let output = Command::new(env!("CARGO_BIN_EXE_ekp_platform"))
        .args(["--eval", "--test-set", "data/eval/search_eval.json"])
        .current_dir(manifest_dir())
        .output()
        .expect("run ekp_platform");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn eval_cli_fails_closed_on_wrong_expectation() {
    let path = std::env::temp_dir().join(format!("ekpi-bad-eval-{}.json", std::process::id()));
    let body = r#"[
        {
            "id": "BAD-01",
            "category": "general",
            "question": "變更執行前，須完成風險評估並取得核准紀錄",
            "expected_doc_id": "SOP-CHANGE",
            "expected_section": "9.9",
            "min_score": 0.42
        }
    ]"#;
    std::fs::write(&path, body).expect("write bad eval set");
    let output = Command::new(env!("CARGO_BIN_EXE_ekp_platform"))
        .args([
            "--eval",
            "--test-set",
            path.to_str().expect("utf-8 temp path"),
        ])
        .current_dir(manifest_dir())
        .output()
        .expect("run ekp_platform");
    let _ = std::fs::remove_file(&path);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("FAIL: [BAD-01]"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("eval gate"));
}

#[test]
fn search_cli_fails_closed_when_data_source_is_missing() {
    let missing = std::env::temp_dir().join(format!(
        "ekpi-missing-data-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_ekp_platform"))
        .args([
            "--search",
            "任何事情",
            "--data",
            missing.to_str().expect("utf-8 temp path"),
        ])
        .current_dir(manifest_dir())
        .output()
        .expect("run ekp_platform");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("FAIL"));
}

#[test]
fn eval_cli_requires_a_test_set() {
    let output = Command::new(env!("CARGO_BIN_EXE_ekp_platform"))
        .args(["--eval"])
        .current_dir(manifest_dir())
        .output()
        .expect("run ekp_platform");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("--test-set"));
}

#[test]
fn fixed_set_file_is_not_empty() {
    let set = load_eval_set(&manifest_dir().join("data/eval/search_eval.json"))
        .expect("load fixed eval set");
    assert!(set.cases.len() >= 10);
    assert_eq!(set.version, 1);
}
