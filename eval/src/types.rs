#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// Verdict classes (addendum A5): comply | violate | insufficient_evidence | escalate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ver {
    Comply,
    Violate,
    InsufficientEvidence,
    Escalate,
}

impl Ver {
    pub const fn as_str(self) -> &'static str {
        match self {
            Ver::Comply => "comply",
            Ver::Violate => "violate",
            Ver::InsufficientEvidence => "insufficient_evidence",
            Ver::Escalate => "escalate",
        }
    }
}

pub fn parse_verdict(text: &str) -> Option<Ver> {
    match text {
        "comply" => Some(Ver::Comply),
        "violate" => Some(Ver::Violate),
        "insufficient_evidence" => Some(Ver::InsufficientEvidence),
        "escalate" => Some(Ver::Escalate),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
pub struct EventRaw {
    #[serde(default)]
    pub activity: String,
    #[serde(default)]
    pub actor: String,
    #[serde(default)]
    pub ts: String,
}

#[derive(Debug, Deserialize)]
pub struct Case {
    pub case_id: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub doc_versions: BTreeMap<String, String>,
    #[serde(default)]
    pub events: Vec<EventRaw>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default)]
    pub citations: Vec<String>,
    #[serde(rename = "loop", default)]
    pub loop_cfg: Option<serde_json::Value>,
    pub expected_verdict: String,
    #[serde(default)]
    pub abstain_required: bool,
    #[serde(default)]
    pub notes: String,
}

impl Case {
    pub fn expected(&self) -> Result<Ver, String> {
        parse_verdict(&self.expected_verdict).ok_or_else(|| {
            format!(
                "case {}: unknown expected_verdict {:?}",
                self.case_id, self.expected_verdict
            )
        })
    }

    /// Shape validation: reads every semantic field so malformed cases are
    /// surfaced as loader warnings instead of silently ignored.
    pub fn check_shape(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if self.case_id.is_empty() {
            issues.push("case_id is empty".to_string());
        }
        if self.category.is_empty() {
            issues.push("category is empty".to_string());
        }
        if self.task.is_empty() {
            issues.push("task is empty".to_string());
        }
        if self.doc_versions.is_empty() {
            issues.push("doc_versions is empty".to_string());
        }
        if self.events.is_empty() {
            issues.push("events is empty".to_string());
        }
        for (i, ev) in self.events.iter().enumerate() {
            if ev.activity.is_empty() {
                issues.push(format!("events[{i}].activity empty"));
            }
            if ev.actor.is_empty() {
                issues.push(format!("events[{i}].actor empty"));
            }
            if ev.ts.is_empty() {
                issues.push(format!("events[{i}].ts empty"));
            }
        }
        if self.rules.is_empty() {
            issues.push("rules is empty".to_string());
        }
        let needs_loop = self.task == "loop_verdict";
        let has_loop = self.loop_cfg.is_some();
        if needs_loop && !has_loop {
            issues.push("loop_verdict case lacks loop state".to_string());
        }
        if has_loop && !needs_loop {
            issues.push("loop state present on non-loop_verdict task".to_string());
        }
        if self.abstain_required && self.expected_verdict != "insufficient_evidence" {
            issues.push(
                "abstain_required case expects something other than insufficient_evidence"
                    .to_string(),
            );
        }
        issues
    }
}

#[derive(Debug)]
pub struct Decision {
    pub case_id: String,
    pub system: String,
    pub verdict: Ver,
    pub confidence: f64,
    pub cited_ids: Vec<String>,
    pub rule_ids_used: Vec<String>,
    pub latency_ms: u64,
    pub cost_tokens: u64,
}

/// Load JSONL cases (tolerant of blank lines and `#` comments).
pub fn load_cases(path: &Path) -> Result<(Vec<Case>, Vec<String>), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut cases = Vec::new();
    let mut warnings = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let case: Case = serde_json::from_str(line)
            .map_err(|e| format!("{}:{}: {e}", path.display(), lineno + 1))?;
        case.expected()
            .map_err(|e| format!("{}:{}: {e}", path.display(), lineno + 1))?;
        for issue in case.check_shape() {
            warnings.push(format!("{}: line {}: {issue}", case.case_id, lineno + 1));
        }
        cases.push(case);
    }
    if cases.is_empty() {
        return Err(format!("{}: no cases loaded", path.display()));
    }
    Ok((cases, warnings))
}
