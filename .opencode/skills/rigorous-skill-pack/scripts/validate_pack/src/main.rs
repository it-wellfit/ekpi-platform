#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_SKILLS: &[&str] = &[
    "00-orchestration",
    "01-product-discovery",
    "02-requirements-specification",
    "03-ux-and-accessibility",
    "04-architecture-and-adr",
    "05-task-planning",
    "06-tdd-and-implementation",
    "07-systematic-debugging",
    "08-code-review",
    "09-testing-and-quality",
    "10-security-and-privacy",
    "11-ai-evaluation-and-governance",
    "12-ci-cd-and-release",
    "13-observability-and-reliability",
    "14-retrospective-and-evolution",
];

const CORE_FILES: &[&str] = &[
    "README.md",
    "AGENTS.md",
    "MASTER-WORKFLOW.md",
    "CHANGELOG.md",
    "LICENSE",
    "PACK-MANIFEST.json",
];

const REQUIRED_DOCS: &[&str] = &["source-registry.md"];
const REQUIRED_TEMPLATES: &[&str] = &["prd.md", "adr.md"];
const REQUIRED_CHECKLISTS: &[&str] = &[
    "definition-of-done.md",
    "stage-quality-gates.md",
    "security-review.md",
    "incident-recovery.md",
];

const CASE_KEYS: &[&str] = &[
    "title",
    "domain",
    "scenario",
    "expectedBehavior",
    "stopCondition",
    "evidenceRequired",
];

struct Errors(Vec<String>);
struct Warnings(Vec<String>);

impl Errors {
    fn new() -> Self {
        Errors(Vec::new())
    }
    fn push(&mut self, msg: String) {
        self.0.push(msg);
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Warnings {
    fn new() -> Self {
        Warnings(Vec::new())
    }
    fn push(&mut self, msg: String) {
        self.0.push(msg);
    }
}

fn has_frontmatter_field(content: &str, field: &str) -> bool {
    content
        .lines()
        .any(|line| line.trim_start().starts_with(field))
}

fn check_core_files(pack_root: &Path, errors: &mut Errors) {
    for f in CORE_FILES {
        if !pack_root.join(f).exists() {
            errors.push(format!("Missing core file: {f}"));
        }
    }
}

fn check_skills(pack_root: &Path, errors: &mut Errors, warnings: &mut Warnings) {
    let skills_dir = pack_root.join("skills");
    if !skills_dir.exists() {
        errors.push("Missing skills directory".to_string());
        return;
    }
    let mut found = 0usize;
    for skill_id in EXPECTED_SKILLS {
        let skill_md = skills_dir.join(skill_id).join("SKILL.md");
        if !skill_md.exists() {
            errors.push(format!("Missing skill file: skills/{skill_id}/SKILL.md"));
            continue;
        }
        found += 1;
        match fs::read_to_string(&skill_md) {
            Ok(content) => {
                if !has_frontmatter_field(&content, "name:") {
                    errors.push(format!("Missing name frontmatter: {:?}", skill_md));
                }
                if !has_frontmatter_field(&content, "description:") {
                    errors.push(format!("Missing description frontmatter: {:?}", skill_md));
                }
            }
            Err(e) => {
                errors.push(format!("Cannot read {skill_id}/SKILL.md: {e}"));
            }
        }
    }
    if found != EXPECTED_SKILLS.len() {
        warnings.push(format!(
            "Expected {} skills, found {}",
            EXPECTED_SKILLS.len(),
            found
        ));
    }
}

fn check_evals(pack_root: &Path, errors: &mut Errors, warnings: &mut Warnings) {
    let cases_file = pack_root.join("evals").join("workflow-evals").join("cases.json");
    if !cases_file.exists() {
        errors.push(format!("Missing eval cases: {:?}", cases_file));
        return;
    }
    let content = match fs::read_to_string(&cases_file) {
        Ok(c) => c,
        Err(e) => {
            errors.push(format!("Cannot read cases.json: {e}"));
            return;
        }
    };
    let trimmed = content.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        errors.push("cases.json is not a JSON object".to_string());
        return;
    }
    if !content.contains("\"cases\"") {
        errors.push("cases.json missing 'cases' key".to_string());
        return;
    }
    let case_count = content.matches("\"id\":").count();
    if case_count != 12 {
        warnings.push(format!("Expected 12 eval cases, found {case_count}"));
    }
    for key in CASE_KEYS {
        if !content.contains(&format!("\"{key}\"")) {
            errors.push(format!("cases.json missing case key: {key}"));
        }
    }
}

fn check_docs(pack_root: &Path, errors: &mut Errors) {
    let docs_dir = pack_root.join("docs");
    for d in REQUIRED_DOCS {
        if !docs_dir.join(d).exists() {
            errors.push(format!("Missing doc: docs/{d}"));
        }
    }
}

fn check_templates(pack_root: &Path, errors: &mut Errors) {
    let templates_dir = pack_root.join("templates");
    for t in REQUIRED_TEMPLATES {
        if !templates_dir.join(t).exists() {
            errors.push(format!("Missing template: templates/{t}"));
        }
    }
}

fn check_checklists(pack_root: &Path, errors: &mut Errors) {
    let checklists_dir = pack_root.join("checklists");
    for c in REQUIRED_CHECKLISTS {
        if !checklists_dir.join(c).exists() {
            errors.push(format!("Missing checklist: checklists/{c}"));
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let pack_root = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir().unwrap_or_default()
    };

    let mut errors = Errors::new();
    let mut warnings = Warnings::new();

    check_core_files(&pack_root, &mut errors);
    check_skills(&pack_root, &mut errors, &mut warnings);
    check_evals(&pack_root, &mut errors, &mut warnings);
    check_docs(&pack_root, &mut errors);
    check_templates(&pack_root, &mut errors);
    check_checklists(&pack_root, &mut errors);

    if errors.is_empty() {
        println!("VALIDATION PASSED");
        for w in warnings.0 {
            println!("WARN:  {w}");
        }
        std::process::exit(0);
    } else {
        println!("VALIDATION FAILED");
        for e in errors.0 {
            println!("ERROR: {e}");
        }
        for w in warnings.0 {
            println!("WARN:  {w}");
        }
        std::process::exit(1);
    }
}
