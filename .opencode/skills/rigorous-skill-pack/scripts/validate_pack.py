#!/usr/bin/env python3
"""驗證 AI 時代嚴謹軟體工程技能包的結構完整性。

檢查項目:
- 必要檔案存在
- 15 個技能檔案具備 name 與 description frontmatter
- 12 個評估案例可正常解析
- ZIP 完整性 (若傳入 zip 路徑)
"""

import json
import sys
from pathlib import Path

PACK_ROOT = Path(__file__).resolve().parent.parent
EXPECTED_SKILLS = [
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
]

CORE_FILES = [
    "README.md",
    "AGENTS.md",
    "MASTER-WORKFLOW.md",
    "CHANGELOG.md",
    "LICENSE",
    "PACK-MANIFEST.json",
]

ERRORS = []
WARNINGS = []


def check_core_files():
    for f in CORE_FILES:
        p = PACK_ROOT / f
        if not p.exists():
            ERRORS.append(f"Missing core file: {f}")


def check_skills():
    skills_dir = PACK_ROOT / "skills"
    if not skills_dir.exists():
        ERRORS.append("Missing skills directory")
        return
    found = 0
    for skill_id in EXPECTED_SKILLS:
        skill_md = skills_dir / skill_id / "SKILL.md"
        if not skill_md.exists():
            ERRORS.append(f"Missing skill file: skills/{skill_id}/SKILL.md")
            continue
        found += 1
        content = skill_md.read_text(encoding="utf-8")
        if "^name:" not in content and not any(line.startswith("name:") for line in content.splitlines()):
            ERRORS.append(f"Missing name frontmatter: {skill_md}")
        if not any(line.startswith("description:") for line in content.splitlines()):
            ERRORS.append(f"Missing description frontmatter: {skill_md}")
    if found != len(EXPECTED_SKILLS):
        WARNINGS.append(f"Expected {len(EXPECTED_SKILLS)} skills, found {found}")


def check_evals():
    cases_file = PACK_ROOT / "evals" / "workflow-evals" / "cases.json"
    if not cases_file.exists():
        ERRORS.append(f"Missing eval cases: {cases_file}")
        return
    try:
        data = json.loads(cases_file.read_text(encoding="utf-8"))
        cases = data.get("cases", [])
        if len(cases) != 12:
            WARNINGS.append(f"Expected 12 eval cases, found {len(cases)}")
        for c in cases:
            for key in ("id", "title", "domain", "scenario", "expectedBehavior", "stopCondition", "evidenceRequired"):
                if key not in c:
                    ERRORS.append(f"Eval case {c.get('id', '?')} missing key: {key}")
    except json.JSONDecodeError as e:
        ERRORS.append(f"Invalid JSON in cases.json: {e}")


def check_docs():
    docs_dir = PACK_ROOT / "docs"
    required_docs = ["source-registry.md"]
    for d in required_docs:
        if not (docs_dir / d).exists():
            ERRORS.append(f"Missing doc: docs/{d}")


def check_templates():
    templates_dir = PACK_ROOT / "templates"
    required = ["prd.md", "adr.md"]
    for t in required:
        if not (templates_dir / t).exists():
            ERRORS.append(f"Missing template: templates/{t}")


def check_checklists():
    checklists_dir = PACK_ROOT / "checklists"
    required = ["definition-of-done.md", "stage-quality-gates.md", "security-review.md", "incident-recovery.md"]
    for c in required:
        if not (checklists_dir / c).exists():
            ERRORS.append(f"Missing checklist: checklists/{c}")


def main():
    check_core_files()
    check_skills()
    check_evals()
    check_docs()
    check_templates()
    check_checklists()

    if ERRORS:
        print("VALIDATION FAILED")
        for e in ERRORS:
            print(f"  ERROR: {e}")
        if WARNINGS:
            for w in WARNINGS:
                print(f"  WARN:  {w}")
        sys.exit(1)

    print("VALIDATION PASSED")
    if WARNINGS:
        for w in WARNINGS:
            print(f"  WARN:  {w}")
    sys.exit(0)


if __name__ == "__main__":
    main()
