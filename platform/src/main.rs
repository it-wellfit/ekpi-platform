#![forbid(unsafe_code)]

//! `ekp_platform` CLI。
//!
//! - `--selftest`：載入真實 `data/` + 種子案例，逐條斷言 9/9 判定類別與引用集合，
//!   並量測硬門檻 H-1..H-6、draft 規則拒收、snapshot 兩次重放一致。
//!   [PASS] 僅出自本次真機執行；任一失敗即回傳非零。
//! - `--out DIR`：`G_ekpi_engine` 接入 `ekp_eval` 同一 runner，與三個確定性 stub
//!   （oracle / allow_all / refuse_all）同場消融；產出 report.json / report.md、
//!   引擎快照（engine_snapshots.json）與硬門檻審計（engine_hard_gates.json）。
//! - `--params PATH`：參數治理檔（R3）。不變參數 fail-closed 鎖定（任一不符即
//!   拒執行）；未給旗標時自動載入預設 `config/engine-params.json`。
//!   `--threshold` 覆寫 governance 檔的 `retrieval_threshold`。
//! - `--declared-rules-schema`：以 `design/rule-schema-v0.1.1.schema.json`
//!   校驗 `data/rules.json` 全量紀錄；違規即整體拒執行（R3）。
//! - `--export-legalruleml DIR`：active 規則 → LegalRuleML 1.0 compact
//!   （`export` 模組；官方構件全集依 research/legalruleml-notes.md）。
//! - `--coverage-report`：seed 9 題 + casebook 46 題真機統計每條 active 規則的
//!   引用次數、verdict 分布、條款錨點與觀測引用，輸出 markdown（R3）。
//!   搭配 `--out` 時寫入 `out/coverage_report.md`。
//! - `--html`：搭配 `--out`，把同一 report.md 渲染為 `report.html`。
//! - `--search QUERY`：詞彙檢索並輸出逐字引用；低於門檻時誠實回報證據不足。
//! - `--eval --test-set PATH`：在固定 JSON 評測集上執行檢索回歸，任一案例失敗即
//!   回傳非零；`--threshold` 可覆寫評測集宣告的營運門檻。

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use ekp_eval::metrics::{Summary, summarize};
use ekp_eval::report::{self, Report};
use ekp_eval::systems::{AllowAll, Oracle, RefuseAll, System};
use ekp_eval::types::{Case, Decision, Ver};

use ekp_platform::coverage_report::{
    coverage_markdown, default_cases_path, load_cases_at, markdown_to_html,
};
use ekp_platform::docs::DocDatabase;
use ekp_platform::engine::{DEFAULT_RETRIEVAL_THRESHOLD, Platform};
use ekp_platform::export as lrml;
use ekp_platform::process::{ReplayEvent, events_sha256};
use ekp_platform::snapshot::DecisionSnapshot;
use ekp_platform::{params as params_module, validate};

const USAGE: &str = "usage: ekp_platform [--selftest | --health | --out DIR [--cases PATH] [--data DIR] [--threshold F]\n        [--params PATH] [--declared-rules-schema] [--export-legalruleml DIR]\n        [--coverage-report] [--html] | --search QUERY [--audit-log PATH] [--llm PROVIDER[:MODEL]]\n        | --eval --test-set PATH | --import-wellfit PATH]";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(run(&args));
}

fn default_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

const BOOLEAN_FLAGS: [&str; 6] = [
    "--selftest",
    "--declared-rules-schema",
    "--coverage-report",
    "--html",
    "--health",
    "--eval",
];

const VALUE_FLAGS: [&str; 11] = [
    "--out",
    "--cases",
    "--data",
    "--threshold",
    "--params",
    "--export-legalruleml",
    "--search",
    "--test-set",
    "--audit-log",
    "--import-wellfit",
    "--llm",
];

/// Flags that only make sense for report generation. `--search` and `--eval`
/// return before the report path, so accepting these together would silently
/// ignore the flag; treat that as a usage error instead.
const REPORT_ONLY_FLAGS: [&str; 5] = [
    "--out",
    "--cases",
    "--html",
    "--coverage-report",
    "--export-legalruleml",
];

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

/// Parses `--llm PROVIDER[:MODEL]`. Only the deterministic test double is
/// implemented; accepting another provider name would let an operator believe
/// an external model was called when it was not.
fn parse_llm_spec(spec: &str) -> Result<(&str, &str), String> {
    let mut parts = spec.splitn(2, ':');
    let provider = parts.next().unwrap_or_default();
    let model = parts.next().unwrap_or("default");
    if provider != "mock" {
        return Err(format!(
            "LLM provider `{provider}` is not implemented; the only supported provider is `mock`"
        ));
    }
    Ok((provider, model))
}

/// Rejects unknown flags and malformed values before any file or report work.
/// The previous implementation silently ignored unknown flags and treated a
/// trailing `--out` as "use the default directory", which contradicted the
/// published CLI contract.
fn validate_args(args: &[String]) -> Result<(), String> {
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if !arg.starts_with("--") {
            return Err(format!("unexpected argument `{arg}`"));
        }
        if BOOLEAN_FLAGS.contains(&arg.as_str()) {
            index += 1;
            continue;
        }
        if VALUE_FLAGS.contains(&arg.as_str()) {
            match args.get(index + 1) {
                Some(value) if !value.is_empty() && !value.starts_with("--") => index += 2,
                Some(value) => {
                    return Err(format!("flag `{arg}` requires a value, found `{value}`"));
                }
                None => return Err(format!("flag `{arg}` requires a value")),
            }
            continue;
        }
        return Err(format!("unknown flag `{arg}`"));
    }

    if has_flag(args, "--test-set") && !has_flag(args, "--eval") {
        return Err("`--test-set` requires `--eval`".to_string());
    }
    if has_flag(args, "--cases") && !has_flag(args, "--out") {
        return Err("`--cases` requires `--out`".to_string());
    }
    if has_flag(args, "--html") && !has_flag(args, "--out") {
        return Err("`--html` requires `--out`".to_string());
    }

    let search = has_flag(args, "--search");
    let eval = has_flag(args, "--eval");
    if search && eval {
        return Err("`--search` and `--eval` are mutually exclusive".to_string());
    }
    if search || eval {
        let mode = if search { "--search" } else { "--eval" };
        for flag in REPORT_ONLY_FLAGS {
            if has_flag(args, flag) {
                return Err(format!("`{flag}` cannot be combined with `{mode}`"));
            }
        }
    }
    if has_flag(args, "--audit-log") && !search {
        return Err("`--audit-log` requires `--search`".to_string());
    }
    if has_flag(args, "--llm") && !search {
        return Err("`--llm` requires `--search`".to_string());
    }
    if let Some(spec) = flag_value(args, "--llm") {
        parse_llm_spec(&spec)?;
    }
    if has_flag(args, "--import-wellfit") {
        if search || eval {
            return Err(
                "`--import-wellfit` cannot be combined with `--search` or `--eval`".to_string(),
            );
        }
        for flag in REPORT_ONLY_FLAGS {
            if has_flag(args, flag) {
                return Err(format!(
                    "`{flag}` cannot be combined with `--import-wellfit`"
                ));
            }
        }
    }
    if has_flag(args, "--health") {
        // Health is a local diagnostic mode with no data loading beyond the
        // existence checks below. Reject every other flag instead of silently
        // ignoring it, so `--health --selftest` cannot look like a health pass.
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--health" => index += 1,
                "--data" => index += 2,
                other => {
                    return Err(format!("`{other}` cannot be combined with `--health`"));
                }
            }
        }
    }
    Ok(())
}

fn health_report(data_dir: &Path) -> bool {
    // Liveness only: the three core seed files must be present as regular
    // files. This deliberately does not parse them, open a database or call a
    // dependency; a deeper readiness probe belongs to the web `/healthz` route.
    let required = ["rules.json", "docs.json", "processes.json"];
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|name| !data_dir.join(name).is_file())
        .collect();
    if missing.is_empty() {
        println!("health: OK");
        true
    } else {
        println!("health: DEGRADED (missing: {})", missing.join(", "));
        false
    }
}

fn run(args: &[String]) -> i32 {
    // Setup panic recovery handler to prevent application crashes
    ekp_platform::errors::setup_panic_handler();

    if has_flag(args, "--health") {
        // Validate before doing any work. Unlike `--selftest`, health has no
        // value as a diagnostic if a typo in another flag is silently ignored.
        if let Err(message) = validate_args(args) {
            eprintln!("FAIL: {message}");
            eprintln!("{USAGE}");
            return 2;
        }
        let data_dir = flag_value(args, "--data")
            .map(PathBuf::from)
            .unwrap_or_else(default_data_dir);
        let healthy = health_report(&data_dir);
        println!("version: 0.1.0");
        println!("data_dir: {}", data_dir.display());
        return if healthy { 0 } else { 1 };
    }

    if has_flag(args, "--selftest") {
        // The self-test keeps precedence over other modes, but malformed
        // arguments are still rejected so that a typo is never silently
        // ignored.
        let code = selftest();
        if code != 0 {
            return code;
        }
        return match validate_args(args) {
            Ok(()) => match args.iter().find(|arg| arg.as_str() != "--selftest") {
                Some(other) => {
                    eprintln!("FAIL: `--selftest` cannot be combined with `{other}`");
                    eprintln!("{USAGE}");
                    2
                }
                None => 0,
            },
            Err(message) => {
                eprintln!("FAIL: {message}");
                2
            }
        };
    }

    if let Err(message) = validate_args(args) {
        eprintln!("FAIL: {message}");
        eprintln!("{USAGE}");
        return 2;
    }

    let data_dir = flag_value(args, "--data")
        .map(PathBuf::from)
        .unwrap_or_else(default_data_dir);

    // ── 參數治理檔（R3）：顯示 --params 或預設路徑自動載入；任一失敗即拒執行。
    let loaded_params = match flag_value(args, "--params") {
        Some(path) => match params_module::load(Path::new(&path)) {
            Ok(p) => {
                println!(
                    "params loaded: {path} (threshold={:.2})",
                    p.retrieval_threshold
                );
                Some(p)
            }
            Err(e) => {
                eprintln!("FAIL: params load `{path}`: {e}");
                return 1;
            }
        },
        None => {
            let default_path = params_module::default_params_path();
            if default_path.exists() {
                match params_module::load(&default_path) {
                    Ok(p) => {
                        println!(
                            "params auto-loaded: {} (threshold={:.2})",
                            default_path.display(),
                            p.retrieval_threshold
                        );
                        Some(p)
                    }
                    Err(e) => {
                        eprintln!("FAIL: params load (default): {e}");
                        return 1;
                    }
                }
            } else {
                None
            }
        }
    };

    // 閾值：CLI 覆寫 > 治理檔 > 預設。
    let threshold_explicit = flag_value(args, "--threshold").is_some();
    let mut threshold = loaded_params
        .as_ref()
        .map_or(DEFAULT_RETRIEVAL_THRESHOLD, |p| p.retrieval_threshold);
    if let Some(v) = flag_value(args, "--threshold") {
        match v.parse::<f64>() {
            Ok(t) if (0.0..=1.0).contains(&t) => threshold = t,
            _ => {
                eprintln!("FAIL: --threshold `{v}` is not a f64 within [0,1]");
                return 2;
            }
        }
    }

    // ── --declared-rules-schema：規則庫全量紀錄對 schema 校驗（fail-closed）。
    let schema_flag = args.iter().any(|a| a == "--declared-rules-schema");
    if schema_flag {
        match declared_rules_schema_check(&data_dir) {
            Ok(()) => println!("declared-rules-schema: OK"),
            Err(e) => {
                eprintln!("{e}");
                return 1;
            }
        }
    }

    let want_coverage = args.iter().any(|a| a == "--coverage-report");
    let want_html = args.iter().any(|a| a == "--html");
    let export_dir = flag_value(args, "--export-legalruleml").map(PathBuf::from);

    // ── --search QUERY：知識搜尋 with citations（Slice 3）。
    if let Some(query) = flag_value(args, "--search") {
        let data_dir = flag_value(args, "--data")
            .map(PathBuf::from)
            .unwrap_or_else(default_data_dir);
        let platform = match load_platform(&data_dir, threshold) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("FAIL: {e}");
                return 1;
            }
        };
        let doc_versions = platform.docs.latest_versions();
        let retriever =
            ekp_platform::retrieval::Retriever::build(&platform.docs, &doc_versions, threshold);
        let start = std::time::Instant::now();
        let result = retriever.search_with_citations(&query, 5, &platform.docs);
        let latency_ms = start.elapsed().as_millis() as u64;

        // ── Audit log: record search interaction (privacy-minimized).
        let audit_log_path = flag_value(args, "--audit-log");
        let citation_ids: Vec<String> = match &result {
            ekp_platform::retrieval::SearchResult::Hits(cits) => cits
                .iter()
                .map(|c| format!("{}@{}:{}", c.doc_id, c.version, c.section))
                .collect(),
            _ => Vec::new(),
        };
        let result_count = match &result {
            ekp_platform::retrieval::SearchResult::Hits(cits) => cits.len(),
            _ => 0,
        };
        let truncated_query = if query.chars().count() > 100 {
            query.chars().take(100).collect::<String>()
        } else {
            query.to_string()
        };
        // Serialize the nested payload instead of interpolating the query.
        // A query can contain quotes, backslashes, control characters or a
        // newline; hand-built JSON would produce a syntactically invalid
        // payload inside an otherwise valid audit event.
        let payload = serde_json::json!({
            "query": truncated_query,
            "result_count": result_count,
            "citations": citation_ids,
            "latency_ms": latency_ms,
        })
        .to_string();
        let event_id = format!(
            "KS-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        if let Some(path) = audit_log_path {
            match ekp_platform::audit::append_jsonl(
                Path::new(&path),
                &event_id,
                ekp_platform::audit::AuditEventType::KnowledgeSearch,
                "search-cli",
                "default",
                &payload,
            ) {
                Ok(events) => println!(
                    "audit: appended to {} ({} verified event(s) in chain)",
                    path, events
                ),
                Err(e) => {
                    eprintln!("FAIL: audit log append failed: {e}");
                    return 1;
                }
            }
        }

        // ── Optional mock RAG demonstration. No external model is called.
        if let Some(llm_spec) = flag_value(args, "--llm") {
            let (_provider, _model) = match parse_llm_spec(&llm_spec) {
                Ok(parsed) => parsed,
                Err(message) => {
                    eprintln!("FAIL: {message}");
                    return 2;
                }
            };
            eprintln!(
                "INFO: --llm uses the deterministic mock provider; no external model is called."
            );
            let search_citations = match &result {
                ekp_platform::retrieval::SearchResult::Hits(cits) => cits.clone(),
                _ => Vec::new(),
            };
            let llm = ekp_platform::rag::MockLlmProvider {
                response: format!(
                    "Based on the retrieved evidence, here is the answer regarding '{}'. [{}@{}]",
                    query,
                    search_citations
                        .first()
                        .map(|c| c.doc_id.clone())
                        .unwrap_or_default(),
                    search_citations
                        .first()
                        .map(|c| c.version.clone())
                        .unwrap_or_default()
                ),
            };
            let rag = ekp_platform::rag::RagOrchestrator::new(threshold);
            let rag_result = rag.answer(&query, &retriever, &platform.docs, &llm);
            match rag_result {
                ekp_platform::rag::RagResult::Answer {
                    text,
                    citations: cits,
                    confidence,
                    latency_ms,
                } => {
                    println!("\nmock-answer: {}", text);
                    println!("provider: mock (no external model)");
                    println!("confidence: {:.2}", confidence);
                    println!("citations: {}", cits.len());
                    println!("latency: {}ms", latency_ms);
                }
                ekp_platform::rag::RagResult::Refused { reason } => {
                    println!("\nrefused: {}", reason);
                }
            }
        }

        if result.has_evidence() {
            if let ekp_platform::retrieval::SearchResult::Hits(citations) = result {
                println!("search: \"{}\"", query);
                println!("results: {}", citations.len());
                for (i, c) in citations.iter().enumerate() {
                    println!("  [{}] {}", i + 1, c.format(),);
                    println!("      {}", c.verbatim_text);
                }
            }
        } else {
            println!("search: \"{}\"", query);
            println!("result: insufficient evidence");
            if let ekp_platform::retrieval::SearchResult::InsufficientEvidence { reason } = result {
                println!("reason: {}", reason);
            }
        }
        return 0;
    }

    // ── --import-wellfit PATH：匯入 Wellfit MTRIX 輸出為知識文件（Phase 2）。
    if let Some(import_path) = flag_value(args, "--import-wellfit") {
        let data_dir = flag_value(args, "--data")
            .map(PathBuf::from)
            .unwrap_or_else(default_data_dir);
        let docs_path = data_dir.join("docs.json");
        let mut db = if docs_path.exists() {
            match DocDatabase::load(&docs_path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("FAIL: load {}: {e}", docs_path.display());
                    return 1;
                }
            }
        } else {
            DocDatabase {
                documents: Vec::new(),
            }
        };

        let import_text = match std::fs::read_to_string(&import_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("FAIL: read {}: {e}", import_path);
                return 1;
            }
        };
        let imported: Vec<serde_json::Value> = match serde_json::from_str(&import_text) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("FAIL: parse {}: {e}", import_path);
                return 1;
            }
        };

        let mut count = 0;
        for item in &imported {
            let doc_id = item
                .get("doc_id")
                .and_then(|v| v.as_str())
                .unwrap_or("WELLFIT-UNKNOWN");
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
            let version = item
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("1.0");
            let effective_from = item
                .get("effective_from")
                .and_then(|v| v.as_str())
                .unwrap_or("2026-01-01");
            let verbatim = item
                .get("verbatim_text")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let section = item.get("section").and_then(|v| v.as_str()).unwrap_or("1");
            let permissions: Vec<String> = item
                .get("permissions")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_else(|| vec!["viewer".to_string(), "editor".to_string()]);

            if db.doc(doc_id).is_some() {
                continue;
            }

            db.documents.push(ekp_platform::docs::Doc {
                doc_id: doc_id.to_string(),
                title: title.to_string(),
                versions: vec![ekp_platform::docs::DocVersion {
                    version: version.to_string(),
                    effective_from: effective_from.to_string(),
                    effective_to: None,
                    injection_directive: None,
                    sections: vec![ekp_platform::docs::Section {
                        section: section.to_string(),
                        page: None,
                        verbatim_text: verbatim.to_string(),
                    }],
                }],
                permissions,
            });
            count += 1;
        }

        if let Err(e) = db.validate_shape() {
            eprintln!("FAIL: imported documents are invalid: {e}");
            return 1;
        }
        if let Err(e) = db.save(&docs_path) {
            eprintln!("FAIL: save docs: {e}");
            return 1;
        }
        println!(
            "import-wellfit: {} documents imported into {}",
            count,
            docs_path.display()
        );
        return 0;
    }

    // ── --eval --test-set PATH：固定評測集回歸；任一案例失敗即非零。
    let want_eval = args.iter().any(|a| a == "--eval");
    if want_eval || args.iter().any(|a| a == "--test-set") {
        let test_set_path = match flag_value(args, "--test-set") {
            Some(path) => path,
            None => {
                eprintln!("FAIL: --eval requires --test-set PATH");
                return 2;
            }
        };
        let data_dir = flag_value(args, "--data")
            .map(PathBuf::from)
            .unwrap_or_else(default_data_dir);
        let platform = match load_platform(&data_dir, threshold) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("FAIL: {e}");
                return 1;
            }
        };
        let set = match ekp_platform::eval::load_eval_set(Path::new(&test_set_path)) {
            Ok(set) => set,
            Err(e) => {
                eprintln!("FAIL: {e}");
                return 1;
            }
        };
        let eval_db = match set.database_with_fixtures(&platform.docs) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("FAIL: {e}");
                return 1;
            }
        };
        if !set.fixtures.is_empty() {
            println!(
                "eval fixtures: {} synthetic document(s) merged for this run only",
                set.fixtures.len()
            );
        }
        let doc_versions = eval_db.latest_versions();
        let eval_threshold = if threshold_explicit {
            threshold
        } else {
            set.threshold.unwrap_or(threshold)
        };
        let metrics =
            ekp_platform::eval::run_eval(&eval_db, &doc_versions, &set.cases, eval_threshold);
        println!(
            "eval: {} cases (set v{}, threshold {:.2})",
            metrics.total_cases, set.version, eval_threshold
        );
        println!(
            "  top-1 hits: {}/{} answer cases",
            metrics.hits, metrics.answer_cases
        );
        println!(
            "  refusals: {}/{} correct",
            metrics.correct_refusals, metrics.refusal_cases
        );
        println!("  precision@1: {:.2}", metrics.precision_at_1);
        println!("  citation support: {:.2}", metrics.citation_support_rate);
        println!(
            "  role-restricted cases: {} (allowed {}, denied {}), violations {}",
            metrics.permission_cases,
            metrics.permission_allowed_cases,
            metrics.permission_denied_cases,
            metrics.permission_violations
        );
        println!(
            "  permission isolation: {:.2}",
            metrics.permission_isolation_rate
        );
        println!("  refusal accuracy: {:.2}", metrics.refusal_accuracy);
        println!("  avg latency: {:.1}ms", metrics.avg_latency_ms);
        for case in metrics.cases.iter().filter(|c| !c.passed) {
            println!(
                "FAIL: [{}] {}: {}",
                case.id,
                case.category,
                case.failure.as_deref().unwrap_or("unknown failure")
            );
        }
        let gate_failures = metrics.gate_failures();
        if !gate_failures.is_empty() {
            for failure in &gate_failures {
                eprintln!("FAIL: eval gate: {failure}");
            }
            return 1;
        }
        println!("eval gate: PASS");
        return 0;
    }

    if !has_flag(args, "--out") {
        // 獨立模式：--export-legalruleml / --coverage-report / 純 schema 校驗。
        if export_dir.is_none() && !want_coverage {
            if schema_flag {
                // 純 schema 校驗：檢查結果即成敗（違規已在上方 return 1）。
                return 0;
            }

            eprintln!("{USAGE}");
            return 2;
        }
        let platform = match load_platform(&data_dir, threshold) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("FAIL: {e}");
                return 1;
            }
        };
        if let Some(dir) = &export_dir {
            let result =
                lrml::export_legalruleml(&platform).and_then(|xml| write_legalruleml(dir, &xml));
            match result {
                Ok(path) => println!("legalruleml export written: {}", path.display()),
                Err(e) => {
                    eprintln!("FAIL: legalruleml export: {e}");
                    return 1;
                }
            }
        }
        if want_coverage {
            match coverage_markdown(&platform) {
                Ok(md) => match write_text(Path::new("coverage_report.md"), &md) {
                    Ok(()) => println!("coverage report written: coverage_report.md"),
                    Err(e) => {
                        eprintln!("FAIL: coverage report: {e}");
                        return 1;
                    }
                },
                Err(e) => {
                    eprintln!("FAIL: coverage report: {e}");
                    return 1;
                }
            }
        }
        return 0;
    }

    let out_dir =
        PathBuf::from(flag_value(args, "--out").unwrap_or_else(|| "report_out".to_string()));
    let cases_path = flag_value(args, "--cases")
        .map(PathBuf::from)
        .unwrap_or_else(default_cases_path);

    match ablation(
        &out_dir,
        &data_dir,
        &cases_path,
        threshold,
        AblationOpts {
            coverage: want_coverage,
            html: want_html,
            export_lrml: export_dir,
        },
    ) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("FAIL: {e}");
            1
        }
    }
}

/// `--declared-rules-schema`：對 `design/rule-schema-v0.1.1.schema.json`
/// 校驗 `data/rules.json` 全量紀錄（含未入引擎者；形狀違規一律顯失敗清單）。
fn declared_rules_schema_check(data_dir: &Path) -> Result<(), String> {
    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("manifest root")
        .join("design")
        .join("rule-schema-v0.1.1.schema.json");
    let schema_text = std::fs::read_to_string(&schema_path)
        .map_err(|e| format!("schema `{}`: {e}", schema_path.display()))?;
    let schema: Value =
        serde_json::from_str(&schema_text).map_err(|e| format!("schema parse: {e}"))?;
    let rules_path = data_dir.join("rules.json");
    let text = std::fs::read_to_string(&rules_path)
        .map_err(|e| format!("rules `{}`: {e}", rules_path.display()))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| format!("rules parse: {e}"))?;
    let records = value
        .get("rules")
        .and_then(Value::as_array)
        .ok_or_else(|| "rules.json has no `rules` array".to_string())?;

    let mut total = 0usize;
    for (i, record) in records.iter().enumerate() {
        let errs = validate::validate(record, &schema, &schema);
        if !errs.is_empty() {
            let id = record.get("rule_id").and_then(Value::as_str).unwrap_or("?");
            let ver = record
                .get("rule_version")
                .and_then(Value::as_str)
                .unwrap_or("?");
            for e in &errs {
                println!("rule-schema violation {i} ({id}@{ver}): {e}");
            }
            total += errs.len();
        }
    }
    if total == 0 {
        println!(
            "declared-rules-schema: {} records validated clean",
            records.len()
        );
        Ok(())
    } else {
        Err(format!(
            "declared-rules-schema: {total} violation(s) — refuse to run (fail-closed)"
        ))
    }
}

fn write_legalruleml(dir: &Path, xml: &str) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("create export dir: {e}"))?;
    let path = dir.join("legalruleml_v1_compact.xml");
    write_text(&path, xml)?;
    Ok(path)
}

fn write_text(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
    }
    std::fs::write(path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

fn load_platform(data_dir: &Path, threshold: f64) -> Result<Platform, String> {
    let mut platform = Platform::load(data_dir)?;
    platform.retrieval_threshold = threshold;
    for note in &platform.rejected_rule_notes {
        println!("poison-guard: {note}");
    }
    println!(
        "platform loaded: docs={} active_rules={} processes={} retrieval_threshold={threshold:.2}",
        platform.docs.documents.len(),
        platform.rules.active_rules().count(),
        platform.processes.processes.len(),
    );
    Ok(platform)
}

// `load_cases_at` / `default_cases_path` / `default_casebook_path` /
// `coverage_markdown` / `markdown_to_html` 皆自庫 `coverage_report` 模組匯入。

/// 確定性引擎 adapter：接進 `ekp_eval` 的 runner（消融報告同場可比）。
struct EngineAdapter<'a> {
    platform: &'a Platform,
}

impl System for EngineAdapter<'_> {
    fn name(&self) -> &'static str {
        "G_ekpi_engine"
    }

    fn decide(&self, case: &Case) -> Decision {
        let started = Instant::now();
        let outcome = self.platform.decide(case);
        Decision {
            case_id: case.case_id.clone(),
            system: self.name().to_string(),
            verdict: outcome.verdict,
            confidence: if outcome.verdict == Ver::InsufficientEvidence {
                0.0
            } else {
                1.0
            },
            cited_ids: outcome.citations.clone(),
            rule_ids_used: outcome.rule_ids_used.clone(),
            latency_ms: started.elapsed().as_millis() as u64,
            // 判定路徑無 LLM：成本記 0 是真實量測語義，不是常數宣稱。
            cost_tokens: 0,
        }
    }
}

/// runner 慣例：每個 decision 必須帶產生它的 adapter 標記。
fn collect(sys: &dyn System, cases: &[Case]) -> Result<Vec<Decision>, String> {
    let decisions: Vec<Decision> = cases.iter().map(|c| sys.decide(c)).collect();
    if let Some(mismatch) = decisions.iter().find(|d| d.system != sys.name()) {
        return Err(format!(
            "adapter bookkeeping: decision for case {} tagged {:?}, expected {:?}",
            mismatch.case_id,
            mismatch.system,
            sys.name(),
        ));
    }
    Ok(decisions)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ─── selftest ────────────────────────────────────────────────────────────

/// 一條斷言的記錄器：PASS 只在 `ok == true` 時輸出。
struct Assertions {
    passed: usize,
    failed: Vec<String>,
}

impl Assertions {
    fn check(&mut self, label: String, ok: bool, detail: String) {
        if ok {
            println!("[PASS] {label}");
            self.passed += 1;
        } else {
            println!("[FAIL] {label}: {detail}");
            self.failed.push(format!("{label}: {detail}"));
        }
    }
}

fn selftest() -> i32 {
    println!("ekp_platform selftest — every [PASS] comes from this real run");

    let data_dir = default_data_dir();
    let cases_path = default_cases_path();
    let platform = match load_platform(&data_dir, DEFAULT_RETRIEVAL_THRESHOLD) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[FAIL] platform load: {e}");
            return 1;
        }
    };
    let cases = match load_cases_at(&cases_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[FAIL] cases load: {e}");
            return 1;
        }
    };

    let mut a = Assertions {
        passed: 0,
        failed: Vec::new(),
    };
    a.check(
        "selftest case count == 9".to_string(),
        cases.len() == 9,
        format!("loaded {}", cases.len()),
    );

    for case in &cases {
        let outcome = platform.decide(case);
        let verdict_exact = case.expected().is_ok_and(|exp| exp == outcome.verdict);
        a.check(
            format!("{} verdict == expected", case.case_id),
            verdict_exact,
            format!(
                "expected {} got {} | notes: {}",
                case.expected_verdict,
                outcome.verdict.as_str(),
                outcome.notes.join(" | ")
            ),
        );

        // 引用集合去重後逐條相等（insufficient 案件兩側皆空）。
        let got: BTreeSet<&str> = outcome.citations.iter().map(String::as_str).collect();
        let want: BTreeSet<&str> = case.citations.iter().map(String::as_str).collect();
        a.check(
            format!("{} citation set equality", case.case_id),
            got == want,
            format!("want {want:?} got {got:?}"),
        );

        // H-3／拒答語義：證據不足案不得攜引用。
        let is_refusal = outcome.verdict == Ver::InsufficientEvidence;
        a.check(
            format!("{} abstain clears citations", case.case_id),
            is_refusal == outcome.citations.is_empty(),
            format!(
                "verdict={} citations={:?}",
                outcome.verdict.as_str(),
                outcome.citations
            ),
        );

        // H-1：觀察到禁止性狀態轉移 → 判定必為 violate（絕不放行）。
        if outcome.notes.iter().any(|n| n.contains("forbidden pair")) {
            a.check(
                format!("{} H-1 forbidden pair released=0", case.case_id),
                outcome.verdict == Ver::Violate,
                outcome.verdict.as_str().to_string(),
            );
        }

        // H-4：非拒答結論必攜 ≥1 個可解析錨點（doc@ver §sec）。
        let anchor_ok = outcome
            .citations
            .iter()
            .all(|c| c.contains('@') && c.contains("§"));
        a.check(
            format!("{} H-4 anchor resolvable", case.case_id),
            is_refusal || (!outcome.citations.is_empty() && anchor_ok),
            format!("citations {got:?}"),
        );

        // H-5：文件內嵌指令不充當系統規則（僅登記、絕不改判定）。
        if case.case_id == "SEC-001" {
            a.check(
                format!("{} H-5 injection ignored", case.case_id),
                outcome
                    .notes
                    .iter()
                    .any(|n| n.contains("H5_injection_ignored"))
                    && outcome.verdict != Ver::Comply,
                format!("verdict={}", outcome.verdict.as_str()),
            );
        }

        // H-6：同一快照兩次重放 manifest 雜湊一致。
        let replay_result = platform.replay_check(case);
        a.check(
            format!("{} H-6 replay consistency", case.case_id),
            replay_result.is_ok(),
            replay_result.err().unwrap_or_default(),
        );
    }

    // 投毒防線：ai_extracted draft 規則必被拒收（審計痕跡在拒收清單）。
    let draft_rejected = platform
        .rejected_rule_notes
        .iter()
        .any(|n| n.contains("RULE-CHG-001-DRAFT") && n.contains("draft_ai"));
    a.check(
        "draft rule rejected at load (audit present)".to_string(),
        draft_rejected,
        format!("rejected={:?}", platform.rejected_rule_notes),
    );

    if a.failed.is_empty() {
        println!("selftest PASSED ({} assertions)", a.passed);
        0
    } else {
        eprintln!(
            "selftest FAILED: {} assertion(s), {} passed",
            a.failed.len(),
            a.passed
        );
        1
    }
}

// ─── --out ablation ──────────────────────────────────────────────────────

#[derive(Serialize)]
struct EngineSnapshotOut {
    case_id: String,
    verdict: String,
    citations: Vec<String>,
    rule_ids_used: Vec<String>,
    manifest_sha: String,
    replay_consistent: bool,
    event_sequence_sha: String,
    doc_binding_shas: Vec<String>,
    rule_shas: Vec<String>,
    notes: Vec<String>,
}

#[derive(Serialize)]
struct GateRow {
    gate: String,
    requirement: String,
    measured: String,
    ok: bool,
}

#[derive(Serialize)]
struct GateAudit {
    generated_at_unix: u64,
    retrieval_threshold: f64,
    gates: Vec<GateRow>,
}

/// R3：--out 流程的附加值開關。
struct AblationOpts {
    coverage: bool,
    html: bool,
    export_lrml: Option<PathBuf>,
}

fn ablation(
    out_dir: &Path,
    data_dir: &Path,
    cases_path: &Path,
    threshold: f64,
    opts: AblationOpts,
) -> Result<(), String> {
    let platform = load_platform(data_dir, threshold)?;
    let cases = load_cases_at(cases_path)?;

    let engine = EngineAdapter {
        platform: &platform,
    };
    let oracle = Oracle;
    let allow_all = AllowAll;
    let refuse_all = RefuseAll;
    let adapters: Vec<&dyn System> = vec![&engine, &oracle, &allow_all, &refuse_all];

    let mut per_system: BTreeMap<String, Summary> = BTreeMap::new();
    for sys in &adapters {
        let decisions = collect(*sys, &cases)?;
        per_system.insert(sys.name().to_string(), summarize(&cases, &decisions));
    }

    // 主控台摘要（與 report.md 同表）。
    println!(
        "| system | n | acc | wrong_allow | viol_as_comply | refusal_on_def | correct_refusal | cite_recall | gate |"
    );
    for (name, s) in &per_system {
        println!(
            "| {} | {} | {:.3} | {} | {} | {:.3} | {:.3} | {:.3} | {} |",
            name,
            s.n_cases,
            s.accuracy_nonabstain.unwrap_or(f64::NAN),
            s.wrong_allow,
            s.violation_as_comply,
            s.refusal_on_definitive.unwrap_or(f64::NAN),
            s.correct_refusal.unwrap_or(f64::NAN),
            s.citation_recall.unwrap_or(f64::NAN),
            s.hard_gate_violated,
        );
    }

    let wrong_allow = per_system.get("G_ekpi_engine").map_or(0, |s| s.wrong_allow);
    let violation_as_comply = per_system
        .get("G_ekpi_engine")
        .map_or(0, |s| s.violation_as_comply);

    let report = Report {
        generated_at_unix: now_unix(),
        per_system,
    };
    report::write(out_dir, &report)?;

    // 引擎側逐案快照（H-6 審計證據）。
    let mut snapshots: Vec<EngineSnapshotOut> = Vec::new();
    let mut forbidden_observed = 0usize;
    let mut forbidden_released = 0usize;
    let mut insufficient_released = 0usize;
    let mut unanchored_conclusions = 0usize;
    let mut injection_cases = 0usize;
    let mut injection_logged = 0usize;
    let mut injection_leaked = 0usize;
    let mut replay_failures = 0usize;

    for case in &cases {
        let outcome = platform.decide(case);
        let replay_events: Vec<ReplayEvent> = case
            .events
            .iter()
            .map(|e| ReplayEvent {
                activity: e.activity.clone(),
                ts: e.ts.clone(),
                actor: e.actor.clone(),
            })
            .collect();
        let snapshot = DecisionSnapshot::build(
            &case.case_id,
            &outcome,
            platform.doc_binding_shas(case),
            platform.rule_binding_shas(case),
            events_sha256(&replay_events),
        );
        let consistent = snapshot.replay_consistent() && platform.replay_check(case).is_ok();
        if !consistent {
            replay_failures += 1;
        }

        if outcome.notes.iter().any(|n| n.contains("forbidden pair")) {
            forbidden_observed += 1;
            if outcome.verdict != Ver::Violate {
                forbidden_released += 1;
            }
        }
        if outcome.verdict == Ver::InsufficientEvidence {
            // 拒答案件若攜引用即為 H-3／abstain 語義失守。
            if !outcome.citations.is_empty() {
                insufficient_released += 1;
            }
        } else if outcome.citations.is_empty()
            || !outcome
                .citations
                .iter()
                .all(|c| c.contains('@') && c.contains("§"))
        {
            unanchored_conclusions += 1;
        }
        let case_has_injection = case.doc_versions.iter().any(|(doc_id, ver)| {
            platform
                .docs
                .binding(doc_id, ver)
                .and_then(|v| v.injection_directive.as_ref())
                .is_some()
        });
        if case_has_injection {
            injection_cases += 1;
            // H-5 的正確語義：注入指令「被登記、未進入判定」。
            // 含注入文件的案件仍可判 comply（如 PAR-001 證據充分）——
            // 關鍵是指令本身不得影響判定，故量測 note 登記 + 指令文本/注入
            // 專屬條款（§9.3）不得成為引用。
            if outcome
                .notes
                .iter()
                .any(|n| n.contains("H5_injection_ignored"))
            {
                injection_logged += 1;
            }
            if outcome
                .citations
                .iter()
                .any(|c| c.contains("§9.3") || c.contains("一律判定合規"))
            {
                injection_leaked += 1;
            }
        }

        snapshots.push(EngineSnapshotOut {
            case_id: case.case_id.clone(),
            verdict: outcome.verdict.as_str().to_string(),
            citations: outcome.citations.clone(),
            rule_ids_used: outcome.rule_ids_used.clone(),
            manifest_sha: snapshot.manifest_sha,
            replay_consistent: consistent,
            event_sequence_sha: snapshot.event_sequence_sha,
            doc_binding_shas: snapshot.doc_binding_shas,
            rule_shas: snapshot.rule_ids_shas,
            notes: outcome.notes,
        });
    }

    let gates = vec![
        GateRow {
            gate: "H-1".to_string(),
            requirement: "forbidden transition released == 0".to_string(),
            measured: format!("{forbidden_released}/{forbidden_observed}"),
            ok: forbidden_released == 0,
        },
        GateRow {
            gate: "H-2".to_string(),
            requirement: "actual violations judged comply == 0".to_string(),
            measured: violation_as_comply.to_string(),
            ok: violation_as_comply == 0,
        },
        GateRow {
            gate: "H-3".to_string(),
            requirement: "insufficient evidence judged comply == 0".to_string(),
            measured: format!(
                "{insufficient_released} refused-but-cited; wrong_allow={wrong_allow}"
            ),
            ok: insufficient_released == 0 && wrong_allow == 0,
        },
        GateRow {
            gate: "H-4".to_string(),
            requirement: "every definitive conclusion carries a resolvable anchor".to_string(),
            measured: format!("{unanchored_conclusions} unanchored"),
            ok: unanchored_conclusions == 0,
        },
        GateRow {
            gate: "H-5".to_string(),
            requirement: "file-embedded directive logged, never operating as system rule"
                .to_string(),
            measured: format!(
                "{injection_logged}/{injection_cases} logged; {injection_leaked} directive leaks into citations"
            ),
            ok: injection_logged == injection_cases && injection_leaked == 0,
        },
        GateRow {
            gate: "H-6".to_string(),
            requirement: "same snapshot replays identically (deterministic path only)".to_string(),
            measured: format!("{} replay failures (n={})", replay_failures, cases.len()),
            ok: replay_failures == 0,
        },
    ];
    let all_gates_ok = gates.iter().all(|g| g.ok);
    let audit = GateAudit {
        generated_at_unix: now_unix(),
        retrieval_threshold: threshold,
        gates,
    };
    std::fs::create_dir_all(out_dir).map_err(|e| format!("create out dir: {e}"))?;
    std::fs::write(
        out_dir.join("engine_snapshots.json"),
        serde_json::to_string_pretty(&snapshots)
            .map_err(|e| format!("serialize snapshots: {e}"))?,
    )
    .map_err(|e| format!("write engine_snapshots.json: {e}"))?;
    std::fs::write(
        out_dir.join("engine_hard_gates.json"),
        serde_json::to_string_pretty(&audit).map_err(|e| format!("serialize gates: {e}"))?,
    )
    .map_err(|e| format!("write engine_hard_gates.json: {e}"))?;

    println!(
        "ablation report written to {} (hard gates: {})",
        out_dir.display(),
        if all_gates_ok { "ALL OK" } else { "VIOLATED" }
    );
    if !all_gates_ok {
        return Err("hard gate violated — deliverable is non-shippable".to_string());
    }

    // ── R3 附加值：LegalRuleML 匯出 / 覆蓋率報告 / HTML 渲染 ─────────
    if let Some(dir) = &opts.export_lrml {
        let xml = lrml::export_legalruleml(&platform)?;
        let path = write_legalruleml(dir, &xml)?;
        println!("legalruleml export written: {}", path.display());
    }
    if opts.coverage {
        let md = coverage_markdown(&platform)?;
        write_text(&out_dir.join("coverage_report.md"), &md)?;
        println!(
            "coverage report written: {}",
            out_dir.join("coverage_report.md").display()
        );
    }
    if opts.html {
        let md_path = out_dir.join("report.md");
        let md = std::fs::read_to_string(&md_path)
            .map_err(|e| format!("read {}: {e}", md_path.display()))?;
        let md = md.trim_start_matches('\u{FEFF}');
        write_text(&out_dir.join("report.html"), &markdown_to_html(md))?;
        println!(
            "html render written: {}",
            out_dir.join("report.html").display()
        );
    }
    Ok(())
}

// ─── --coverage-report / --html ──────────────────────────────────────────
// R3 的 coverage 統計與 markdown→HTML 渲染核心在庫 `coverage_report` 模組
// （含其單元測試）；以下僅為 CLI 的寫檔出口。
