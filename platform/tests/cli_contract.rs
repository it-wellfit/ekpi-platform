#![forbid(unsafe_code)]

//! Contract tests for `ekp_platform` argument parsing and exit codes.
//!
//! Each case runs the real binary. The tests assert that malformed arguments
//! are rejected before any report or data work, so a usage error can never
//! leave the previous report untouched while claiming success.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ekpi-cli-contract-{}-{}",
        std::process::id(),
        NEXT_DIR.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn platform_bin() -> PathBuf {
    // Application Control on some managed hosts blocks execution from the
    // Cargo target directory. The override lets the same test binary run
    // against a copied, executable-equivalent build.
    std::env::var_os("EKP_PLATFORM_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_ekp_platform")))
}

fn run_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(platform_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run ekp_platform")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn exit_code(output: &Output) -> i32 {
    output.status.code().expect("process exited with a code")
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn unknown_flag_is_a_usage_error() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--bogus"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("unknown flag `--bogus`"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn unknown_flag_prevents_report_generation() {
    let dir = temp_dir();
    let out = dir.join("report");
    let output = run_in(
        &dir,
        &["--out", out.to_str().expect("utf-8 temp path"), "--bogus"],
    );
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(!out.exists(), "a usage error must not create report output");
    assert!(!stdout(&output).contains("ablation report written"));
    cleanup(&dir);
}

#[test]
fn out_without_value_does_not_fall_back_to_the_default_directory() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--out"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("flag `--out` requires a value"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(!stdout(&output).contains("ablation report written"));
    assert!(
        !dir.join("report_out").exists(),
        "missing value must not write the default report directory"
    );
    cleanup(&dir);
}

#[test]
fn threshold_without_value_is_a_usage_error() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--threshold"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("flag `--threshold` requires a value"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn empty_value_is_a_usage_error() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--out", ""]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("flag `--out` requires a value"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn positional_argument_is_a_usage_error() {
    let dir = temp_dir();
    let output = run_in(&dir, &["data/eval/search_eval.json"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("unexpected argument `data/eval/search_eval.json`"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn a_flag_cannot_be_used_as_another_flags_value() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--out", "--html"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("requires a value, found `--html`"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn search_without_value_is_a_usage_error() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--search"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("flag `--search` requires a value"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn test_set_without_eval_is_a_usage_error() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--test-set", "data/eval/search_eval.json"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("`--test-set` requires `--eval`"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn search_and_eval_are_mutually_exclusive() {
    let dir = temp_dir();
    let output = run_in(
        &dir,
        &[
            "--search",
            "變更",
            "--eval",
            "--test-set",
            "data/eval/search_eval.json",
        ],
    );
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("`--search` and `--eval` are mutually exclusive"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn cases_without_out_is_a_usage_error() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--cases", "cases.jsonl"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("`--cases` requires `--out`"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn report_flags_cannot_be_silently_ignored_by_eval() {
    let dir = temp_dir();
    let out = dir.join("report");
    let output = run_in(
        &dir,
        &[
            "--eval",
            "--test-set",
            "data/eval/search_eval.json",
            "--out",
            out.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("`--out` cannot be combined with `--eval`"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(!out.exists());
    cleanup(&dir);
}

#[test]
fn selftest_runs_but_malformed_arguments_still_fail() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--selftest", "--bogus"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stdout(&output).contains("selftest PASSED"),
        "self-test should still run first: stdout: {}",
        stdout(&output)
    );
    assert!(
        stderr(&output).contains("unknown flag `--bogus`"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}
