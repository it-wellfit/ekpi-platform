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
    PathBuf::from(env!("CARGO_BIN_EXE_ekp_platform"))
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
fn health_reports_ok_for_the_default_data_directory() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--health"]);
    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(out.contains("health: OK"), "stdout: {out}");
    assert!(out.contains("version: 0.1.0"), "stdout: {out}");
    cleanup(&dir);
}

#[test]
fn health_reports_degraded_when_core_files_are_missing() {
    let dir = temp_dir();
    let data = dir.join("data");
    std::fs::create_dir_all(&data).expect("create empty data dir");
    let output = run_in(
        &dir,
        &[
            "--health",
            "--data",
            data.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 1, "stderr: {}", stderr(&output));
    assert!(
        stdout(&output)
            .contains("health: DEGRADED (missing: rules.json, docs.json, processes.json)"),
        "stdout: {}",
        stdout(&output)
    );
    cleanup(&dir);
}

#[test]
fn health_does_not_ignore_an_unknown_flag() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--health", "--bogus"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("unknown flag `--bogus`"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(
        !stdout(&output).contains("health:"),
        "a malformed health request must not report a status: stdout: {}",
        stdout(&output)
    );
    cleanup(&dir);
}

#[test]
fn health_cannot_be_combined_with_a_mode_flag() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--health", "--selftest"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("`--selftest` cannot be combined with `--health`"),
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

#[test]
fn selftest_refuses_a_valid_mode_flag_it_would_otherwise_ignore() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--selftest", "--search", "變更核准"]);
    assert_eq!(
        exit_code(&output),
        2,
        "stdout: {} stderr: {}",
        stdout(&output),
        stderr(&output)
    );
    assert!(
        stderr(&output).contains("`--selftest` cannot be combined with `--search`"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(
        !stdout(&output).contains("results:"),
        "search mode must not run or be silently skipped: stdout: {}",
        stdout(&output)
    );
    cleanup(&dir);
}

#[test]
fn audit_log_without_search_is_a_usage_error() {
    let dir = temp_dir();
    let audit = dir.join("audit.json");
    let output = run_in(
        &dir,
        &["--audit-log", audit.to_str().expect("utf-8 temp path")],
    );
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("`--audit-log` requires `--search`"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(!audit.exists(), "a usage error must not write an audit log");
    cleanup(&dir);
}

#[test]
fn llm_without_search_is_a_usage_error() {
    let dir = temp_dir();
    let output = run_in(&dir, &["--llm", "mock:test"]);
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("`--llm` requires `--search`"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}

#[test]
fn unsupported_llm_provider_is_rejected_before_audit_write() {
    let dir = temp_dir();
    let audit = dir.join("audit.json");
    let output = run_in(
        &dir,
        &[
            "--search",
            "變更執行前，須完成風險評估並取得核准紀錄",
            "--llm",
            "openai:gpt",
            "--audit-log",
            audit.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains(
            "LLM provider `openai` is not implemented; the only supported provider is `mock`"
        ),
        "stderr: {}",
        stderr(&output)
    );
    assert!(
        !audit.exists(),
        "provider validation must happen before the audit log write"
    );
    cleanup(&dir);
}

#[test]
fn llm_mock_is_labeled_and_requires_a_versioned_citation() {
    let dir = temp_dir();
    let output = run_in(
        &dir,
        &[
            "--search",
            "變更執行前，須完成風險評估並取得核准紀錄",
            "--llm",
            "mock:test",
        ],
    );
    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    let out = stdout(&output);
    assert!(
        stderr(&output).contains("INFO: --llm uses the deterministic mock provider"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(
        out.contains("provider: mock (no external model)"),
        "stdout: {out}"
    );
    let answer = out
        .lines()
        .find(|line| line.starts_with("mock-answer:"))
        .unwrap_or_else(|| panic!("missing labeled mock answer: {out}"));
    assert!(
        answer.contains('[') && answer.contains('@') && answer.contains(']'),
        "the mock answer must carry an exact [doc_id@version] citation: {answer}"
    );
    assert!(
        !out.contains("provider: openai") && !out.contains("provider: anthropic"),
        "the mock path must not claim an external provider: {out}"
    );
    cleanup(&dir);
}

#[test]
fn audit_log_appends_a_verified_chain_across_runs() {
    let dir = temp_dir();
    let audit = dir.join("audit.jsonl");
    for run in 0..2 {
        let output = run_in(
            &dir,
            &[
                "--search",
                "變更執行前，須完成風險評估並取得核准紀錄",
                "--audit-log",
                audit.to_str().expect("utf-8 temp path"),
            ],
        );
        assert_eq!(
            exit_code(&output),
            0,
            "run {run} stderr: {}",
            stderr(&output)
        );
    }

    let log = ekp_platform::audit::read_jsonl(&audit).expect("valid JSONL audit chain");
    assert_eq!(log.len(), 2);
    assert!(log.verify_chain());
    assert_eq!(log.events[0].prev_hash, "GENESIS");
    assert_eq!(log.events[1].prev_hash, log.events[0].hash);
    for event in &log.events {
        assert_eq!(
            event.event_type,
            ekp_platform::audit::AuditEventType::KnowledgeSearch
        );
        assert_eq!(event.actor, "search-cli");
        assert_eq!(event.tenant_id, "default");
    }

    let contents = std::fs::read_to_string(&audit).expect("read audit log");
    assert_eq!(contents.lines().count(), 2);
    assert!(
        !contents.contains("session") && !contents.contains("password"),
        "the local search audit must not invent an identity: {contents}"
    );
    cleanup(&dir);
}

#[test]
fn audit_payload_remains_valid_json_for_hostile_query_characters() {
    let dir = temp_dir();
    let audit = dir.join("audit.jsonl");
    let query = "change \"quoted\" \\ path\nsecond line";
    let output = run_in(
        &dir,
        &[
            "--search",
            query,
            "--audit-log",
            audit.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));

    let log = ekp_platform::audit::read_jsonl(&audit).expect("valid JSONL audit chain");
    assert_eq!(log.len(), 1);
    let payload: serde_json::Value = serde_json::from_str(&log.events[0].payload)
        .expect("the audit payload must itself be valid JSON");
    assert_eq!(payload["query"], query);
    assert_eq!(payload["result_count"], 0);
    assert!(payload["citations"].is_array());
    cleanup(&dir);
}

#[test]
fn audit_log_refuses_a_corrupt_chain_without_modifying_it() {
    let dir = temp_dir();
    let audit = dir.join("audit.jsonl");
    std::fs::write(&audit, "{\"not\":\"an event\"}\n").expect("write corrupt audit log");
    let before = std::fs::read_to_string(&audit).expect("read corrupt log");

    let output = run_in(
        &dir,
        &[
            "--search",
            "變更執行前，須完成風險評估並取得核准紀錄",
            "--audit-log",
            audit.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 1, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("FAIL: audit log append failed"),
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(
        std::fs::read_to_string(&audit).expect("read log after refusal"),
        before,
        "a failed append must leave the corrupt file unchanged"
    );
    cleanup(&dir);
}

#[test]
fn audit_log_rejects_a_legacy_single_object_file() {
    let dir = temp_dir();
    let audit = dir.join("audit.json");
    std::fs::write(&audit, "{\n  \"events\": []\n}\n").expect("write legacy audit log");
    let before = std::fs::read_to_string(&audit).expect("read legacy log");

    let output = run_in(
        &dir,
        &[
            "--search",
            "變更執行前，須完成風險評估並取得核准紀錄",
            "--audit-log",
            audit.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 1, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("legacy single-object format"),
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(
        std::fs::read_to_string(&audit).expect("read log after refusal"),
        before,
        "a legacy file must not be rewritten in place"
    );
    cleanup(&dir);
}

#[test]
fn audit_log_refuses_a_whitespace_only_file_without_modifying_it() {
    let dir = temp_dir();
    let audit = dir.join("audit.jsonl");
    std::fs::write(&audit, "\n").expect("write whitespace-only audit log");
    let before = std::fs::read_to_string(&audit).expect("read whitespace-only log");

    let output = run_in(
        &dir,
        &[
            "--search",
            "變更執行前，須完成風險評估並取得核准紀錄",
            "--audit-log",
            audit.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 1, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("line 1 is empty"),
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(
        std::fs::read_to_string(&audit).expect("read log after refusal"),
        before,
        "a whitespace-only file must not be rewritten in place"
    );
    cleanup(&dir);
}

#[test]
fn import_wellfit_writes_documents_to_the_selected_data_directory() {
    let dir = temp_dir();
    let data = dir.join("data");
    std::fs::create_dir_all(&data).expect("create data dir");
    let input = dir.join("wellfit.json");
    std::fs::write(
        &input,
        r#"[{"doc_id":"WF-1","title":"Imported","version":"1.0","verbatim_text":"local fixture","permissions":["viewer"]}]"#,
    )
    .expect("write import fixture");
    let output = run_in(
        &dir,
        &[
            "--import-wellfit",
            input.to_str().expect("utf-8 temp path"),
            "--data",
            data.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 0, "stderr: {}", stderr(&output));
    assert!(
        stdout(&output).contains("import-wellfit: 1 documents imported"),
        "stdout: {}",
        stdout(&output)
    );
    let saved =
        std::fs::read_to_string(data.join("docs.json")).expect("read imported docs database");
    assert!(
        saved.contains("\"doc_id\": \"WF-1\""),
        "saved docs: {saved}"
    );
    cleanup(&dir);
}

#[test]
fn import_wellfit_refuses_to_overwrite_an_unreadable_docs_database() {
    let dir = temp_dir();
    let data = dir.join("data");
    std::fs::create_dir_all(&data).expect("create data dir");
    let docs = data.join("docs.json");
    std::fs::write(&docs, "{ not json").expect("write corrupt docs database");
    let input = dir.join("wellfit.json");
    std::fs::write(
        &input,
        r#"[{"doc_id":"WF-1","title":"Imported","version":"1.0","verbatim_text":"local fixture"}]"#,
    )
    .expect("write import fixture");
    let output = run_in(
        &dir,
        &[
            "--import-wellfit",
            input.to_str().expect("utf-8 temp path"),
            "--data",
            data.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 1, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("load") && stderr(&output).contains("docs.json"),
        "stderr: {}",
        stderr(&output)
    );
    assert_eq!(
        std::fs::read_to_string(&docs).expect("read preserved docs database"),
        "{ not json",
        "a failed import must not overwrite the existing database"
    );
    cleanup(&dir);
}

#[test]
fn import_wellfit_refuses_invalid_imported_documents() {
    let dir = temp_dir();
    let data = dir.join("data");
    std::fs::create_dir_all(&data).expect("create data dir");
    let input = dir.join("wellfit.json");
    std::fs::write(
        &input,
        r#"[{"doc_id":"","title":"Invalid","version":"1.0","verbatim_text":"local fixture"}]"#,
    )
    .expect("write invalid import fixture");
    let output = run_in(
        &dir,
        &[
            "--import-wellfit",
            input.to_str().expect("utf-8 temp path"),
            "--data",
            data.to_str().expect("utf-8 temp path"),
        ],
    );
    assert_eq!(exit_code(&output), 1, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output).contains("imported documents are invalid")
            && stderr(&output).contains("doc_id empty"),
        "stderr: {}",
        stderr(&output)
    );
    assert!(
        !data.join("docs.json").exists(),
        "invalid imported documents must not be saved"
    );
    cleanup(&dir);
}

#[test]
fn import_wellfit_cannot_be_silently_combined_with_eval() {
    let dir = temp_dir();
    let input = dir.join("wellfit.json");
    std::fs::write(&input, "[]").expect("write import fixture");
    let output = run_in(
        &dir,
        &[
            "--import-wellfit",
            input.to_str().expect("utf-8 temp path"),
            "--eval",
            "--test-set",
            "data/eval/search_eval.json",
        ],
    );
    assert_eq!(exit_code(&output), 2, "stderr: {}", stderr(&output));
    assert!(
        stderr(&output)
            .contains("`--import-wellfit` cannot be combined with `--search` or `--eval`"),
        "stderr: {}",
        stderr(&output)
    );
    cleanup(&dir);
}
