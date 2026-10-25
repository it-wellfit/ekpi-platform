#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ekp_eval::metrics::{self, summarize};
use ekp_eval::report;
use ekp_eval::systems::{self, System};
use ekp_eval::types::{self, Case, Decision};

const USAGE: &str = "usage: ekp_eval [--selftest] [--cases PATH] [--out DIR] [--systems a,b,c]";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(run(&args));
}

fn default_cases_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("cases")
        .join("seed_cases.jsonl")
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

fn run(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--selftest") {
        return selftest();
    }

    let mut cases_path = default_cases_path();
    let mut out_dir = PathBuf::from("report_out");
    let mut systems_wanted = vec![
        "stub:oracle".to_string(),
        "stub:allow_all".to_string(),
        "stub:refuse_all".to_string(),
    ];
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--cases" => match args.get(i + 1) {
                Some(v) => {
                    cases_path = PathBuf::from(v);
                    i += 1;
                }
                None => {
                    eprintln!("{USAGE}");
                    return 2;
                }
            },
            "--out" => match args.get(i + 1) {
                Some(v) => {
                    out_dir = PathBuf::from(v);
                    i += 1;
                }
                None => {
                    eprintln!("{USAGE}");
                    return 2;
                }
            },
            "--systems" => match args.get(i + 1) {
                Some(v) => {
                    systems_wanted = v
                        .split(',')
                        .filter_map(|s| {
                            let s = s.trim();
                            if s.is_empty() {
                                None
                            } else {
                                Some(s.to_string())
                            }
                        })
                        .collect();
                    i += 1;
                }
                None => {
                    eprintln!("{USAGE}");
                    return 2;
                }
            },
            other => {
                eprintln!("unknown argument: {other}");
                eprintln!("{USAGE}");
                return 2;
            }
        }
        i += 1;
    }

    let (cases, warnings) = match types::load_cases(&cases_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load cases: {e}");
            return 2;
        }
    };
    for w in &warnings {
        println!("warning: {w}");
    }

    let registry = systems::registry();
    let mut per_system: BTreeMap<String, metrics::Summary> = BTreeMap::new();
    for wanted in &systems_wanted {
        let Some(sys) = registry.iter().find(|s| s.name() == wanted.as_str()) else {
            eprintln!("unknown system adapter: {wanted}");
            return 2;
        };
        let decisions = match collect(sys.as_ref(), &cases) {
            Ok(v) => v,
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

fn selftest() -> i32 {
    let (cases, warnings) = match types::load_cases(&default_cases_path()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("load cases: {e}");
            return 2;
        }
    };
    for w in &warnings {
        println!("warning: {w}");
    }

    let registry = systems::registry();
    let mut results: BTreeMap<String, metrics::Summary> = BTreeMap::new();
    for sys in &registry {
        let decisions = match collect(sys.as_ref(), &cases) {
            Ok(v) => v,
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

    let mut failure_count = 0usize;
    fn check(name: &str, cond: bool, detail: &str, failure_count: &mut usize) {
        let status = if cond { "PASS" } else { "FAIL" };
        println!("[{status}] {name}: {detail}");
        if !cond {
            *failure_count += 1;
        }
    }

    check(
        "oracle verdict accuracy == 1.0",
        oracle.accuracy_nonabstain == Some(1.0),
        &format!("got {:?}", oracle.accuracy_nonabstain),
        &mut failure_count,
    );
    check(
        "oracle wrong_allow == 0 (H-1/H-2 plumbing)",
        oracle.wrong_allow == 0,
        &format!("got {}", oracle.wrong_allow),
        &mut failure_count,
    );
    check(
        "oracle refusal_on_definitive == 0.0",
        oracle.refusal_on_definitive == Some(0.0),
        &format!("got {:?}", oracle.refusal_on_definitive),
        &mut failure_count,
    );
    check(
        "oracle correct_refusal == 1.0",
        oracle.correct_refusal == Some(1.0),
        &format!("got {:?}", oracle.correct_refusal),
        &mut failure_count,
    );
    check(
        "oracle escalation_correct == 1.0",
        oracle.escalation_correct == Some(1.0),
        &format!("got {:?}", oracle.escalation_correct),
        &mut failure_count,
    );
    check(
        "oracle citation_recall == 1.0",
        oracle.citation_recall == Some(1.0),
        &format!("got {:?}", oracle.citation_recall),
        &mut failure_count,
    );
    check(
        "allow_all trips hard gate (violation_as_comply > 0)",
        allow.violation_as_comply > 0 && allow.hard_gate_violated,
        &format!(
            "violation_as_comply={}, gate={}",
            allow.violation_as_comply, allow.hard_gate_violated
        ),
        &mut failure_count,
    );
    check(
        "allow_all wrong_allow counts every stop-expected case",
        allow.wrong_allow > 0,
        &format!("got {}", allow.wrong_allow),
        &mut failure_count,
    );
    check(
        "refuse_all refuses ALL definitive cases",
        refuse.refusal_on_definitive == Some(1.0),
        &format!("got {:?}", refuse.refusal_on_definitive),
        &mut failure_count,
    );
    check(
        "refuse_all correct_refusal == 1.0 (obeys `stop` cases)",
        refuse.correct_refusal == Some(1.0),
        &format!("got {:?}", refuse.correct_refusal),
        &mut failure_count,
    );

    if failure_count == 0 {
        println!("\nselftest PASSED");
        0
    } else {
        println!("\nselftest FAILED ({failure_count} checks)");
        2
    }
}
