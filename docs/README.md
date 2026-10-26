# Documentation map

- **Status:** Active
- **Owner:** Engineering (content owners are named per document)
- **Last reviewed:** 2026-10-26

This is the entry point for repository documentation. It answers three questions: where a rule lives, who owns it, and when it must be updated. Do not create a second copy of a rule in a new document; link to the canonical document and record only the local consequence.

## Canonical sources

| Question | Canonical document | Do not duplicate in |
| --- | --- | --- |
| Why the product exists, users, roles, MVP scope and non-goals | [Product brief](product/README.md), [MVP scope](product/mvp-scope.md), and the approved governance baselines | README marketing text, release notes |
| Business requirements, functional behavior, data/state, security, decisions | [Governance requirements](governance/02-requirements-v0.1.md), [functional spec](governance/05-functional-spec-v1.md), [data and state model](governance/06-data-and-state-model-v1.md), [security and connector spec](governance/07-security-integration-spec-v1.md), [decision register](governance/08-decision-register.md) | Architecture and backend summaries |
| Acceptance cases and target-product gaps | [Acceptance test matrix](governance/04-acceptance-test-matrix.md) and [traceability baseline](governance/10-traceability-baseline.md) | Test plans, release notes |
| Numeric targets and hard gates | [Metrics target register](metrics/targets.md) | README, success-metrics summaries |
| Current system structure, data flow and deployment boundary | [Architecture](architecture/README.md) and [threat model](architecture/threat-model.md) | Deployment guides |
| Backend responsibilities, persistence and authorization boundary | [Backend](backend/README.md) | Architecture overview |
| Frontend information architecture and interaction states | [Frontend](frontend/README.md) | Product requirements |
| CLI, web-route, data-format and export contracts | [Contracts](contracts/README.md) | Implementation docs |
| Risk tiers, required checks and merge gates | [Quality gates](development/quality-gates.md) | Individual specs |
| Phase A/B exit criteria and production readiness | [Phase gates](quality/phase-gates.md) and [production readiness](operations/production-readiness.md) | Release notes |
| Test strategy, AI evaluation and current evidence | [Quality](quality/README.md), [AI evaluation](quality/ai-evaluation.md), and [regression checklist](development/regression-checklist.md) | README counts |
| Installation, configuration, backup, monitoring, incidents and pilot handoff | [Operations](operations/README.md), including the [pilot runbook](operations/pilot-runbook.md) and [off-host backup proposal](operations/backup-policy-draft.md) | Deployment summaries |
| Engineering and business decisions | [ADR index](decisions/README.md), [ADR-0001](adr/0001-engineering-delivery-workflow.md), and [decision register](governance/08-decision-register.md) | Specs and contracts |
| Requirement-to-evidence chain | [Traceability rules](traceability/README.md) and [register](traceability/register.md) | PR descriptions only |
| Reviews, retrospectives and improvement tracking | [Reviews](reviews/README.md) | Release notes |
| Local-first Git, backup and remote-upgrade rules | [Local-first workflow](operations/local-first-workflow.md), [local backup](operations/local-backup.md), [remote collaboration criteria](remote-collaboration-criteria.md) | New process documents |
| Agent skills and issue-tracker setup | [Skill setup](agents/skill-setup.md), [issue tracker](agents/issue-tracker.md), [triage labels](agents/triage-labels.md), and [community skill assessment](agents/community-skills-assessment.md) | AGENTS.md detail |

## Directory index

| Directory | Purpose | Owner | Update trigger |
| --- | --- | --- | --- |
| `product/` | Product problem, users, MVP scope, non-goals and phase framing | Product | User evidence, scope, metric or MVP decision changes |
| `governance/` | Authoritative cross-project requirements, specs, acceptance matrix, decisions and target traceability | Product + QA | Requirement, control, acceptance or business decision changes |
| `architecture/` | Context, containers, module boundaries, data flow, deployment boundary, threat model and capacity assumptions | Engineering | Module, dependency, data-flow, deployment or threat changes |
| `backend/` | Domain responsibilities, persistence, served surfaces, authorization boundary, errors and known gaps | Backend | Behavior, persistence, API or authorization changes |
| `frontend/` | Information architecture, pages, forms, interaction states and accessibility | Frontend/UX | Route, form, state, rendering or accessibility changes |
| `contracts/` | CLI, web-route, data-format and LegalRuleML contracts plus compatibility policy | Engineering | Any consumer-visible interface or schema change |
| `quality/` | Test strategy, phase gates, AI evaluation, performance assumptions and quality evidence | QA | Gate, test, eval, metric or risk changes |
| `operations/` | Installation, configuration, backup, monitoring, incidents and production readiness | Operations | Deployment, recovery, monitoring, data policy or ownership changes |
| `decisions/` | Decision index and ADR template; ADR storage remains `docs/adr/` | Engineering | New or superseded decision |
| `reviews/` | Review reports, retrospectives and the improvement register | Engineering + QA | Each review, release or retrospective |
| `traceability/` | ID rules and the repository-change traceability ledger | QA | Each accepted requirement, test, release or retro |
| `development/` | Workflow, risk tiers, templates, regression baseline and release/retro procedure | Engineering | Process, template or gate changes |
| `agents/` | Agent skill setup, issue-tracker conventions, triage labels and external skill assessment | Engineering | Tooling or skill changes |
| `metrics/` | Numeric target register and hard gates | Product | Release review or approved target change |
| `deployment/` | Concise operator pages; canonical procedures live in `operations/` | Operations | Deployment or environment changes |
| `adr/` | Engineering decision records | Engineering | New or superseded ADR |
| `research/` | Research evidence, source registries and selection records | Research | New evidence or selection decision |
| `prompt/` | Master-prompt addenda | Product Engineering | Prompt or scope correction |
| `tech-decision-records/` | Index of referenced technology decisions | Engineering | Decision added or superseded |

## Read by task

| Task | Read first | Then |
| --- | --- | --- |
| New feature or behavior change | [Product brief](product/README.md) | [Governance functional spec](governance/05-functional-spec-v1.md), [data model](governance/06-data-and-state-model-v1.md), [architecture](architecture/README.md), [contracts](contracts/README.md), [quality gates](development/quality-gates.md), [engineering workflow](development/README.md) |
| Create or pick up a work item | [Issue tracker](agents/issue-tracker.md) | Issue form under `.github/ISSUE_TEMPLATE/` (`feature`, `ticket`, `improvement`, `bug`), [quality gates](development/quality-gates.md), [traceability register](traceability/register.md) |
| Defect or regression | [Regression checklist](development/regression-checklist.md) | [Backend](backend/README.md) or [frontend](frontend/README.md), [contracts](contracts/README.md), [quality gates](development/quality-gates.md) |
| Security or data change | [Threat model](architecture/threat-model.md) | [Security spec](governance/07-security-integration-spec-v1.md), [production readiness](operations/production-readiness.md), [secrets policy](operations/secrets-policy.md) |
| Release or pilot handoff | [Phase gates](quality/phase-gates.md) | [Release and retro](development/release-and-retro.md), [release template](development/release-template.md), [operations](operations/README.md) |
| Retrospective | [Reviews](reviews/README.md) | [Retro template](reviews/retro-template.md), [traceability register](traceability/register.md) |
| Documentation-only change | This page | Relevant canonical document; run `scripts/check-docs.ps1` or `scripts/check-docs.sh` |

## Status legend

| Status | Meaning |
| --- | --- |
| `Implemented` | Present in the repository and covered by an executed check |
| `Partial` | Present for a limited scope; the document states the boundary and gap |
| `Target` | Required for the target product or Phase B, but not implemented |
| `Blocked` | Cannot proceed without an external decision, access or owner |
| `Deferred` | Intentionally postponed; reason and review trigger recorded |
| `Historical` | Retained for the record; not current evidence and not authoritative |
| `Superseded` | Replaced by a named document; kept only for traceability |

## Document rules

1. Every new or materially changed document states its status, owner and last-reviewed date. This index is the backstop when a legacy document omits them.
2. A document owns a decision, not a copy of someone else's decision. Link to the canonical source and state only the local consequence.
3. A fact that can be checked by a command must name the command and the revision or date observed. Never present an unrun check as passed.
4. Product requirements use the IDs in `docs/governance/`; engineering change items use the `S` register in `docs/traceability/register.md`; decisions use `DEC-` or `ADR-` IDs. Do not invent a parallel ID scheme.
5. When a requirement, contract, test or release changes, update the traceability register in the same change.
6. Do not put real customer data, credentials or organization-specific facts in documentation or test fixtures. Use synthetic data.
7. Keep legacy summaries only as historical pointers; do not update them independently.

## Historical and superseded documents

These files are retained but are not the current source of truth:

- `docs/requirements-spec.md` — an early FR/NFR summary. Canonical: `docs/governance/02-requirements-v0.1.md`, `05`, `06`, `07`.
- `docs/success-metrics.md` — an early metric summary with targets that differ from the locked register. Canonical: `docs/metrics/targets.md`; where they differ, the register wins.
- `docs/risk-model.md` — an early threat summary. Canonical: `docs/architecture/threat-model.md` for the current system and `docs/governance/07-security-integration-spec-v1.md` for target controls.
- `docs/operations/final-validation-report.md` — a dated 2026-10-25 snapshot. Current evidence: `docs/development/regression-checklist.md`, this repository's gate output, and review reports under `docs/reviews/`.
- `docs/tech-selection-round1.md` — a research and selection record. Current implementation facts: `docs/architecture/README.md`. Target technologies mentioned there are not implementation claims.
- `docs/research-framework-v1.md` — a historical research/target-state framework dated 2026-10-25. Mentions of WORM, tenant isolation, enterprise auth or manifest hash chains are targets, not current capabilities. Canonical current facts: `docs/architecture/README.md`, `docs/architecture/threat-model.md`.
- `docs/deployment/` — concise operator pages; canonical procedures live in `docs/operations/`.

## Verification

From the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check-docs.ps1
```

```bash
./scripts/check-docs.sh
```

The check verifies that required documents exist and that relative Markdown links resolve. It does not prove that a document is true; that requires the command or review evidence named in the document.

## Where to start

- First visit: [README](../README.md) for the system map, then [product brief](product/README.md) and [architecture](architecture/README.md).
- Before coding: [engineering workflow](development/README.md), [quality gates](development/quality-gates.md), and the affected canonical specification.
- Before release: [phase gates](quality/phase-gates.md) and [release and retro](development/release-and-retro.md).
- Before claiming production readiness: [production readiness](operations/production-readiness.md). The current answer is **not production ready**.
