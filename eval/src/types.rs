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

/// Declarative fixed-corpus manifest. Paths are relative to the manifest file.
#[derive(Debug, Deserialize)]
pub struct CorpusManifest {
    pub version: u32,
    pub id: String,
    pub minimum_cases: usize,
    pub files: Vec<String>,
}

#[derive(Debug)]
pub struct Corpus {
    pub id: String,
    pub cases: Vec<Case>,
    /// (manifest-relative file name, cases loaded from that file)
    pub sources: Vec<(String, usize)>,
}

/// Load and merge every JSONL file in a corpus manifest.
///
/// Fails closed on duplicate case ids, an empty manifest, a file that cannot be
/// parsed, or a corpus below the declared minimum. Shape problems are returned
/// as warnings so the caller can decide whether to treat them as fatal.
pub fn load_corpus(manifest_path: &Path) -> Result<(Corpus, Vec<String>), String> {
    let text = std::fs::read_to_string(manifest_path)
        .map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    let manifest: CorpusManifest =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    if manifest.version != 1 {
        return Err(format!(
            "{}: unsupported corpus manifest version {}",
            manifest_path.display(),
            manifest.version
        ));
    }
    if manifest.id.trim().is_empty() {
        return Err(format!("{}: corpus id is empty", manifest_path.display()));
    }
    if manifest.minimum_cases == 0 {
        return Err(format!(
            "{}: minimum_cases must be greater than zero",
            manifest_path.display()
        ));
    }
    if manifest.files.is_empty() {
        return Err(format!("{}: corpus has no files", manifest_path.display()));
    }

    let base = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let mut cases = Vec::new();
    let mut warnings = Vec::new();
    let mut sources = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();

    for file in &manifest.files {
        let path = base.join(file);
        let (mut loaded, mut file_warnings) = load_cases(&path)?;
        let count = loaded.len();
        for case in &loaded {
            if let Some(previous) = seen.insert(case.case_id.clone(), file.clone()) {
                return Err(format!(
                    "{}: duplicate case_id {:?} in {:?} and {:?}",
                    manifest_path.display(),
                    case.case_id,
                    previous,
                    file
                ));
            }
        }
        warnings.append(&mut file_warnings);
        cases.append(&mut loaded);
        sources.push((file.clone(), count));
    }

    if cases.len() < manifest.minimum_cases {
        return Err(format!(
            "{}: corpus {} has {} cases, minimum is {}",
            manifest_path.display(),
            manifest.id,
            cases.len(),
            manifest.minimum_cases
        ));
    }

    Ok((
        Corpus {
            id: manifest.id,
            cases,
            sources,
        },
        warnings,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_corpus_is_complete_and_unique() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("cases")
            .join("corpus.json");
        let (corpus, warnings) = load_corpus(&manifest).expect("shipped corpus must load");
        assert!(
            warnings.is_empty(),
            "unexpected shape warnings: {warnings:?}"
        );
        assert!(corpus.cases.len() >= 50, "corpus must meet the R2-7 target");
        assert_eq!(corpus.sources.len(), 3);
        let mut ids = std::collections::BTreeSet::new();
        for case in &corpus.cases {
            assert!(
                ids.insert(case.case_id.clone()),
                "duplicate case id {}",
                case.case_id
            );
        }
    }
}
