#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ekp_eval::annotation;
use ekp_eval::metrics::{self, summarize};
use ekp_eval::report;
use ekp_eval::systems::{self, System};
use ekp_eval::types::{self, Case, Decision};

const USAGE: &str = "usage: ekp_eval [--cases PATH | --corpus PATH] [--out DIR] [--systems a,b,c]\n       ekp_eval --selftest\n       ekp_eval --corpus-selftest\n       ekp_eval --make-annotation-sheet PATH [--corpus PATH]\n       ekp_eval --kappa ANNOTATOR_A.jsonl ANNOTATOR_B.jsonl [--corpus PATH]";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(run(&args));
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn default_cases_path() -> PathBuf {
    crate_root().join("cases").join("seed_cases.jsonl")
}

fn default_corpus_path() -> PathBuf {
    crate_root().join("cases").join("corpus.json")
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Runs one adapter over all cases. Verifies adapter bookkeeping: every
/// decision must carry the system tag of the adapter that produced it.
fn collect(sys: &dyn System, cases: &[Case]) -> Result<Vec<Decision>, String> {
    let mut decisions = Vec::new();
    for case in cases {
        decisions.push(sys.decide(case));
    }
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

/// Return the value for a flag, rejecting a missing, empty or flag-shaped value.
fn require_value<'a>(args: &'a [String], i: &mut usize, flag: &str) -> Result<&'a str, String> {
    let Some(value) = args.get(*i + 1) else {
        return Err(format!("{flag} requires a value"));
    };
    if value.is_empty() || value.starts_with("--") {
        return Err(format!("{flag} has an invalid value: {value:?}"));
    }
    *i += 1;
    Ok(value.as_str())
}

fn usage_error(message: &str) -> i32 {
    eprintln!("{message}");
    eprintln!("{USAGE}");
    2
}

fn run(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("--selftest") => {
            if args.len() != 1 {
                return usage_error("--selftest must be the only argument");
            }
            return selftest();
        }
        Some("--corpus-selftest") => {
            if args.len() != 1 {
                return usage_error("--corpus-selftest must be the only argument");
            }
            return corpus_selftest();
        }
        Some("--kappa") => return kappa_cmd(&args[1..]),
        Some("--make-annotation-sheet") => return make_annotation_sheet_cmd(&args[1..]),
        _ => {}
    }

    let mut cases_path: Option<PathBuf> = None;
    let mut corpus_path: Option<PathBuf> = None;
    let mut out_dir = PathBuf::from("report_out");
    let mut systems_wanted = vec![
        "stub:oracle".to_string(),
        "stub:allow_all".to_string(),
        "stub:refuse_all".to_string(),
    ];
    let mut seen_flags: BTreeSet<&str> = BTreeSet::new();
    let mut i = 0usize;
    while i < args.len() {
        let flag = args[i].as_str();
        if flag.starts_with("--") && !seen_flags.insert(flag) {
            return usage_error(&format!("duplicate argument: {flag}"));
        }
        match flag {
            "--cases" => match require_value(args, &mut i, "--cases") {
                Ok(value) => cases_path = Some(PathBuf::from(value)),
                Err(e) => return usage_error(&e),
            },
            "--corpus" => match require_value(args, &mut i, "--corpus") {
                Ok(value) => corpus_path = Some(PathBuf::from(value)),
                Err(e) => return usage_error(&e),
            },
            "--out" => match require_value(args, &mut i, "--out") {
                Ok(value) => out_dir = PathBuf::from(value),
                Err(e) => return usage_error(&e),
            },
            "--systems" => match require_value(args, &mut i, "--systems") {
                Ok(value) => {
                    let parsed: Vec<String> = value
                        .split(',')
                        .map(str::trim)
                        .filter(|part| !part.is_empty())
                        .map(ToOwned::to_owned)
                        .collect();
                    if parsed.is_empty() {
                        return usage_error("--systems must select at least one adapter");
                    }
                    systems_wanted = parsed;
                }
                Err(e) => return usage_error(&e),
            },
            other => {
                return usage_error(&format!("unknown argument: {other}"));
            }
        }
        i += 1;
    }

    if cases_path.is_some() && corpus_path.is_some() {
        return usage_error("--cases and --corpus are mutually exclusive");
    }
    if systems_wanted.iter().collect::<BTreeSet<_>>().len() != systems_wanted.len() {
        return usage_error("--systems contains a duplicate adapter");
    }

    let (cases, warnings, corpus_id, sources) = if let Some(path) = corpus_path {
        let (corpus, warnings) = match types::load_corpus(&path) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("load corpus: {e}");
                return 2;
            }
        };
        (corpus.cases, warnings, Some(corpus.id), corpus.sources)
    } else {
        let path = cases_path.unwrap_or_else(default_cases_path);
        let (cases, warnings) = match types::load_cases(&path) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("load cases: {e}");
                return 2;
            }
        };
        (cases, warnings, None, Vec::new())
    };

    if let Some(id) = &corpus_id {
        println!("corpus: {id}");
    }
    for (file, count) in &sources {
        println!("  {file}: {count} cases");
    }
    for warning in &warnings {
        println!("warning: {warning}");
    }
    println!("cases: {}", cases.len());

    let registry = systems::registry();
    let mut per_system: BTreeMap<String, metrics::Summary> = BTreeMap::new();
    for wanted in &systems_wanted {
        let Some(sys) = registry.iter().find(|s| s.name() == wanted.as_str()) else {
            eprintln!("unknown system adapter: {wanted}");
            return 2;
        };
        let decisions = match collect(sys.as_ref(), &cases) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        let summary = summarize(&cases, &decisions);
        per_system.insert(sys.name().to_string(), summary);
    }

    let generated = report::Report {
        generated_at_unix: now_unix(),
        per_system,
    };
    match report::write(&out_dir, &generated) {
        Ok(()) => {
            println!("report written to {}", out_dir.display());
            0
        }
        Err(e) => {
            eprintln!("write report: {e}");
            2
        }
    }
}

fn kappa_cmd(args: &[String]) -> i32 {
    let mut annotation_paths: Vec<&str> = Vec::new();
    let mut corpus_path = default_corpus_path();
    let mut seen_corpus = false;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus" => {
                if seen_corpus {
                    return usage_error("duplicate --corpus");
                }
                seen_corpus = true;
                match require_value(args, &mut i, "--corpus") {
                    Ok(value) => corpus_path = PathBuf::from(value),
                    Err(e) => return usage_error(&e),
                }
            }
            value if value.starts_with("--") => {
                return usage_error(&format!("unknown argument: {value}"));
            }
            value => {
                if annotation_paths.len() >= 2 {
                    return usage_error("--kappa takes exactly two annotation files");
                }
                annotation_paths.push(value);
            }
        }
        i += 1;
    }
    if annotation_paths.len() != 2 {
        return usage_error("--kappa takes exactly two annotation files");
    }

    let (corpus, _warnings) = match types::load_corpus(&corpus_path) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("load corpus: {e}");
            return 2;
        }
    };
    let corpus_case_ids: BTreeSet<String> = corpus
        .cases
        .iter()
        .map(|case| case.case_id.clone())
        .collect();

    let a = match annotation::load_annotations(Path::new(annotation_paths[0])) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("load annotator A: {e}");
            return 2;
        }
    };
    let b = match annotation::load_annotations(Path::new(annotation_paths[1])) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("load annotator B: {e}");
            return 2;
        }
    };
    let study = match annotation::evaluate_study_for_corpus(&a, &b, &corpus_case_ids) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("kappa study: {e}");
            return 2;
        }
    };

    let status = if study.passed {
        "PASS"
    } else {
        "BELOW_THRESHOLD"
    };
    let output = serde_json::json!({
        "status": status,
        "corpus_id": &corpus.id,
        "corpus_cases": corpus_case_ids.len(),
        "annotator_a": &study.annotator_a,
        "annotator_b": &study.annotator_b,
        "n": study.n,
        "observed_agreement": study.observed_agreement,
        "expected_agreement": study.expected_agreement,
        "cohens_kappa": study.kappa,
        "interpretation": study.interpretation,
        "acceptance_threshold": study.acceptance_threshold,
        "passed": study.passed,
        "identical_label_vectors": study.identical_label_vectors,
    });
    match serde_json::to_string_pretty(&output) {
        Ok(text) => println!("{text}"),
        Err(e) => {
            eprintln!("serialize kappa report: {e}");
            return 2;
        }
    }
    if study.passed {
        0
    } else {
        1
    }
}

fn make_annotation_sheet_cmd(args: &[String]) -> i32 {
    let mut out_path: Option<PathBuf> = None;
    let mut corpus_path = default_corpus_path();
    let mut seen_corpus = false;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--corpus" => {
                if seen_corpus {
                    return usage_error("duplicate --corpus");
                }
                seen_corpus = true;
                match require_value(args, &mut i, "--corpus") {
                    Ok(value) => corpus_path = PathBuf::from(value),
                    Err(e) => return usage_error(&e),
                }
            }
            value if value.starts_with("--") => {
                return usage_error(&format!("unknown argument: {value}"));
            }
            value => {
                if out_path.is_some() {
                    return usage_error("--make-annotation-sheet takes exactly one output path");
                }
                out_path = Some(PathBuf::from(value));
            }
        }
        i += 1;
    }
    let Some(out) = out_path else {
        return usage_error("--make-annotation-sheet requires an output path");
    };

    let (corpus, warnings) = match types::load_corpus(&corpus_path) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("load corpus: {e}");
            return 2;
        }
    };
    for warning in &warnings {
        println!("warning: {warning}");
    }

    let mut body = String::new();
    for case in &corpus.cases {
        let events: Vec<serde_json::Value> = case
            .events
            .iter()
            .map(|event| {
                serde_json::json!({
                    "activity": &event.activity,
                    "actor": &event.actor,
                    "ts": &event.ts,
                })
            })
            .collect();
        let row = serde_json::json!({
            "case_id": &case.case_id,
            "task": &case.task,
            "doc_versions": &case.doc_versions,
            "events": events,
            "rules": &case.rules,
            "loop": &case.loop_cfg,
            "verdict": null,
            "annotator": null,
        });
        body.push_str(&serde_json::to_string(&row).expect("serialize annotation sheet row"));
        body.push('\n');
    }

    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("create annotation directory: {e}");
                return 2;
            }
        }
    }
    if let Err(e) = std::fs::write(&out, body) {
        eprintln!("write annotation sheet: {e}");
        return 2;
    }
    println!(
        "annotation sheet written to {} ({} blinded cases)",
        out.display(),
        corpus.cases.len()
    );
    0
}

struct Checker {
    failures: usize,
}

impl Checker {
    fn check(&mut self, name: &str, condition: bool, detail: &str) {
        let status = if condition { "PASS" } else { "FAIL" };
        println!("[{status}] {name}: {detail}");
        if !condition {
            self.failures += 1;
        }
    }
}

fn selftest() -> i32 {
    let (cases, warnings) = match types::load_cases(&default_cases_path()) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("load cases: {e}");
            return 2;
        }
    };
    for warning in &warnings {
        println!("warning: {warning}");
    }

    let registry = systems::registry();
    let mut results: BTreeMap<String, metrics::Summary> = BTreeMap::new();
    for sys in &registry {
        let decisions = match collect(sys.as_ref(), &cases) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        let summary = summarize(&cases, &decisions);
        results.insert(sys.name().to_string(), summary);
    }

    let Some(oracle) = results.get("stub:oracle") else {
        eprintln!("stub:oracle missing");
        return 2;
    };
    let Some(allow) = results.get("stub:allow_all") else {
        eprintln!("stub:allow_all missing");
        return 2;
    };
    let Some(refuse) = results.get("stub:refuse_all") else {
        eprintln!("stub:refuse_all missing");
        return 2;
    };

    let mut checker = Checker { failures: 0 };
    checker.check(
        "oracle verdict accuracy == 1.0",
        oracle.accuracy_nonabstain == Some(1.0),
        &format!("got {:?}", oracle.accuracy_nonabstain),
    );
    checker.check(
        "oracle wrong_allow == 0 (H-1/H-2 plumbing)",
        oracle.wrong_allow == 0,
        &format!("got {}", oracle.wrong_allow),
    );
    checker.check(
        "oracle refusal_on_definitive == 0.0",
        oracle.refusal_on_definitive == Some(0.0),
        &format!("got {:?}", oracle.refusal_on_definitive),
    );
    checker.check(
        "oracle correct_refusal == 1.0",
        oracle.correct_refusal == Some(1.0),
        &format!("got {:?}", oracle.correct_refusal),
    );
    checker.check(
        "oracle escalation_correct == 1.0",
        oracle.escalation_correct == Some(1.0),
        &format!("got {:?}", oracle.escalation_correct),
    );
    checker.check(
        "oracle citation_recall == 1.0",
        oracle.citation_recall == Some(1.0),
        &format!("got {:?}", oracle.citation_recall),
    );
    checker.check(
        "allow_all trips hard gate (violation_as_comply > 0)",
        allow.violation_as_comply > 0 && allow.hard_gate_violated,
        &format!(
            "violation_as_comply={}, gate={}",
            allow.violation_as_comply, allow.hard_gate_violated
        ),
    );
    checker.check(
        "allow_all wrong_allow counts every stop-expected case",
        allow.wrong_allow > 0,
        &format!("got {}", allow.wrong_allow),
    );
    checker.check(
        "refuse_all refuses ALL definitive cases",
        refuse.refusal_on_definitive == Some(1.0),
        &format!("got {:?}", refuse.refusal_on_definitive),
    );
    checker.check(
        "refuse_all correct_refusal == 1.0 (obeys `stop` cases)",
        refuse.correct_refusal == Some(1.0),
        &format!("got {:?}", refuse.correct_refusal),
    );

    if checker.failures == 0 {
        println!("\nselftest PASSED");
        0
    } else {
        println!("\nselftest FAILED ({} checks)", checker.failures);
        2
    }
}

struct SheetInspection {
    rows: usize,
    ids: BTreeSet<String>,
    ids_match: bool,
    forbidden_fields: Vec<String>,
    prefilled_labels: Vec<String>,
}

fn inspect_annotation_sheet(
    path: &Path,
    corpus: &types::Corpus,
) -> Result<SheetInspection, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut rows = 0usize;
    let mut ids = BTreeSet::new();
    let mut forbidden = BTreeSet::new();
    let mut prefilled = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("{}:{}: {e}", path.display(), lineno + 1))?;
        let object = value
            .as_object()
            .ok_or_else(|| format!("{}:{}: row is not an object", path.display(), lineno + 1))?;
        let id = object
            .get("case_id")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                format!(
                    "{}:{}: row has no string case_id",
                    path.display(),
                    lineno + 1
                )
            })?;
        rows += 1;
        ids.insert(id.to_string());
        for key in [
            "expected_verdict",
            "notes",
            "citations",
            "category",
            "abstain_required",
        ] {
            if object.contains_key(key) {
                forbidden.insert(key.to_string());
            }
        }
        for key in ["verdict", "annotator"] {
            if object.get(key).is_some_and(|value| !value.is_null()) {
                prefilled.push(format!("{id}:{key}"));
            }
        }
    }
    let corpus_ids: BTreeSet<String> = corpus
        .cases
        .iter()
        .map(|case| case.case_id.clone())
        .collect();
    Ok(SheetInspection {
        rows,
        ids_match: ids == corpus_ids,
        ids,
        forbidden_fields: forbidden.into_iter().collect(),
        prefilled_labels: prefilled,
    })
}

fn corpus_selftest() -> i32 {
    let (corpus, warnings) = match types::load_corpus(&default_corpus_path()) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("load corpus: {e}");
            return 2;
        }
    };
    for warning in &warnings {
        println!("warning: {warning}");
    }

    let mut checker = Checker { failures: 0 };
    checker.check(
        "corpus meets the 50-case minimum",
        corpus.cases.len() >= 50,
        &format!("{} cases", corpus.cases.len()),
    );
    checker.check(
        "corpus manifest has 3 source files",
        corpus.sources.len() == 3,
        &format!("{} sources", corpus.sources.len()),
    );
    for (file, count) in &corpus.sources {
        println!("  source {file}: {count}");
    }

    let mut verdict_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for case in &corpus.cases {
        *verdict_counts
            .entry(case.expected_verdict.as_str())
            .or_default() += 1;
    }
    for verdict in ["comply", "violate", "escalate", "insufficient_evidence"] {
        let count = verdict_counts.get(verdict).copied().unwrap_or(0);
        checker.check(
            &format!("corpus covers {verdict}"),
            count > 0,
            &format!("{count} cases"),
        );
    }

    let sheet_path = crate_root().join("annotation").join("blinded-cases.jsonl");
    match inspect_annotation_sheet(&sheet_path, &corpus) {
        Ok(sheet) => {
            checker.check(
                "blinded annotation sheet row count",
                sheet.rows == corpus.cases.len(),
                &format!("{} rows for {} cases", sheet.rows, corpus.cases.len()),
            );
            checker.check(
                "blinded annotation sheet case ids",
                sheet.ids_match,
                &format!("{} unique ids", sheet.ids.len()),
            );
            checker.check(
                "blinded annotation sheet hides labels",
                sheet.forbidden_fields.is_empty(),
                &format!("forbidden fields: {:?}", sheet.forbidden_fields),
            );
            checker.check(
                "blinded annotation sheet rows are unique",
                sheet.ids.len() == sheet.rows,
                &format!("{} unique ids for {} rows", sheet.ids.len(), sheet.rows),
            );
            checker.check(
                "blinded annotation sheet has no prefilled labels",
                sheet.prefilled_labels.is_empty(),
                &format!("prefilled labels: {:?}", sheet.prefilled_labels),
            );
        }
        Err(e) => checker.check("blinded annotation sheet", false, &e),
    }

    let registry = systems::registry();
    let mut results: BTreeMap<String, metrics::Summary> = BTreeMap::new();
    for sys in &registry {
        let decisions = match collect(sys.as_ref(), &corpus.cases) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return 2;
            }
        };
        let summary = summarize(&corpus.cases, &decisions);
        results.insert(sys.name().to_string(), summary);
    }

    let Some(oracle) = results.get("stub:oracle") else {
        eprintln!("stub:oracle missing");
        return 2;
    };
    let Some(allow) = results.get("stub:allow_all") else {
        eprintln!("stub:allow_all missing");
        return 2;
    };
    let Some(refuse) = results.get("stub:refuse_all") else {
        eprintln!("stub:refuse_all missing");
        return 2;
    };

    checker.check(
        "oracle accuracy == 1.0 on the full corpus",
        oracle.accuracy_nonabstain == Some(1.0),
        &format!("got {:?}", oracle.accuracy_nonabstain),
    );
    checker.check(
        "oracle wrong_allow == 0 on the full corpus",
        oracle.wrong_allow == 0,
        &format!("got {}", oracle.wrong_allow),
    );
    checker.check(
        "oracle correct_refusal == 1.0 on the full corpus",
        oracle.correct_refusal == Some(1.0),
        &format!("got {:?}", oracle.correct_refusal),
    );
    checker.check(
        "oracle hard gate clean on the full corpus",
        !oracle.hard_gate_violated,
        &format!("gate={}", oracle.hard_gate_violated),
    );
    checker.check(
        "allow_all trips the hard gate on the full corpus",
        allow.violation_as_comply > 0 && allow.hard_gate_violated,
        &format!(
            "violation_as_comply={}, gate={}",
            allow.violation_as_comply, allow.hard_gate_violated
        ),
    );
    checker.check(
        "refuse_all still refuses every definitive case",
        refuse.refusal_on_definitive == Some(1.0),
        &format!("got {:?}", refuse.refusal_on_definitive),
    );
    checker.check(
        "refuse_all keeps correct_refusal == 1.0",
        refuse.correct_refusal == Some(1.0),
        &format!("got {:?}", refuse.correct_refusal),
    );

    if checker.failures == 0 {
        println!("\ncorpus selftest PASSED ({} cases)", corpus.cases.len());
        0
    } else {
        println!("\ncorpus selftest FAILED ({} checks)", checker.failures);
        2
    }
}
