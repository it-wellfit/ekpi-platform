#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::metrics::Summary;

#[derive(Serialize)]
pub struct Report {
    pub generated_at_unix: u64,
    pub per_system: BTreeMap<String, Summary>,
}

fn f3(value: Option<f64>) -> String {
    match value {
        Some(x) => format!("{x:.3}"),
        None => "-".to_string(),
    }
}

fn escape_pipe(text: &str) -> String {
    text.replace('|', "\\|")
}

pub fn write(out_dir: &Path, report: &Report) -> Result<(), String> {
    std::fs::create_dir_all(out_dir).map_err(|e| format!("create out dir: {e}"))?;
    let json =
        serde_json::to_string_pretty(report).map_err(|e| format!("serialize report: {e}"))?;

    let mut md = String::new();
    md.push_str("# EKPI eval report\n\n");
    md.push_str(
        "| system | n | acc | wrong_allow | viol_as_comply | refusal_on_def | unwarr_refusal | correct_refusal | manual_handoff | completion | error | cite_recall | anchor_res | hard_gate |\n",
    );
    md.push_str("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|\n");
    for (name, s) in &report.per_system {
        let anchor_text = format!(
            "{}/{}",
            s.anchor_accounting.resolved, s.anchor_accounting.total
        );
        md.push_str(&format!(
            "| {name} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            s.n_cases,
            f3(s.accuracy_nonabstain),
            s.wrong_allow,
            s.violation_as_comply,
            f3(s.refusal_on_definitive),
            f3(s.unwarranted_refusal_rate),
            f3(s.correct_refusal),
            f3(s.manual_handoff_rate),
            f3(s.completion_rate),
            f3(s.error_rate),
            f3(s.citation_recall),
            anchor_text,
            s.hard_gate_violated,
        ));
    }
    md.push_str("\n## Failure details\n\n");
    for (name, s) in &report.per_system {
        md.push_str(&format!("### {name}\n\n"));
        if s.failures.is_empty() {
            md.push_str("none\n\n");
            continue;
        }
        md.push_str("| case | category | expected | actual | rules_used | notes |\n|---|---|---|---|---|---|\n");
        for f in &s.failures {
            let rules_used = if f.rules_used.is_empty() {
                "-".to_string()
            } else {
                f.rules_used.join(",")
            };
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                f.case_id,
                f.category,
                f.expected,
                f.actual,
                rules_used,
                escape_pipe(&f.notes),
            ));
        }
        md.push('\n');
    }

    std::fs::write(out_dir.join("report.json"), json)
        .map_err(|e| format!("write report.json: {e}"))?;
    // UTF-8 BOM：Windows 記事本/PowerShell 直接開啟不亂碼（JSON 不加，RFC 8259）。
    let md_with_bom = format!("\u{FEFF}{md}");
    std::fs::write(out_dir.join("report.md"), md_with_bom)
        .map_err(|e| format!("write report.md: {e}"))?;
    Ok(())
}
