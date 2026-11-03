# Reviews and improvements

- **Status:** Active; one bootstrap retrospective and two local self-audits completed; no independent review
- **Owner:** Engineering + QA
- **Last reviewed:** 2026-10-30
- **Quality gates:** [Quality gates](../development/quality-gates.md)
- **Phase gates:** [Phase gates](../quality/phase-gates.md)
- **Traceability:** [Traceability register](../traceability/register.md)

## Purpose

This directory holds review reports, retrospectives and the improvement register. A review finding is not complete when it is written down; it is complete when it has an owner, a tracked work item, a fix and rerun evidence.

Three records are kept separate:

| Record | Question | Output |
| --- | --- | --- |
| Code/spec review | Is this change correct, safe and traceable enough to merge? | Findings, disposition and rerun evidence in the PR/review report |
| Release review | Is this revision ready for the approved use? | Gate table, evidence links and explicit go/no-go |
| Retrospective | What should the team change in how it works? | Accepted actions in the improvement register, linked to issues |

## Review report index

| ID | Date | Scope | Reviewer | Result | Report |
| --- | --- | --- | --- | --- | --- |
| AUDIT-0001 | 2026-10-26 | End-to-end code, contract, test, documentation, backup and claims audit | Engineering + QA (self-audit) | 11 findings: 4 high, 5 medium, 2 low; locally fixable items fixed and verified at `7f6ab97`; canonical gate 13/13 and restore drill PASS locally; Phase A not approved; Phase B blocked | [2026-10-26-end-to-end-audit.md](2026-10-26-end-to-end-audit.md) |
| AUDIT-0002 | 2026-10-29 | Hardening review of `--health`, host-policy workarounds, workspace isolation, audit v2, local login/Host/cookie fail-closed behavior and current evidence | Engineering + QA (self-audit, not independent) | 250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation and 4 web health tests pass individually; `edge_cases` blocked by host policy (`os error 4551`), so canonical is `Blocked`; overrides removed; Phase A not approved; Phase B blocked | [test evidence E-029](../assurance/test-evidence.md#e-029-hardening-re-run-and-host-policy-block-c827351) |

Add a report when a review covers more than a single PR comment thread, or when a release, incident or gate decision needs a durable record. Use the PR template for ordinary code review; do not duplicate it here.

## Retrospective index

| ID | Date | Scope / period | Facilitator | Report |
| --- | --- | --- | --- | --- |
| RETRO-0001 | 2026-10-26 | Bootstrap round: documentation system, skills, quality gates | Engineering (self-review) | [bootstrap-retro.md](bootstrap-retro.md) |

## Improvement register

Every accepted improvement gets an ID, an owner, a due trigger and a link. “Improve documentation” is not a work item; name the document and the observable change.

| ID | Source | Improvement | Owner | Priority | Tracking | Status |
| --- | --- | --- | --- | --- | --- | --- |
| IMP-001 | RETRO-0001 | Add `scripts/check-docs` and wire it into `scripts/check` and `gate.ps1` | Engineering | High | This change | Done (`283f83a`) |
| IMP-002 | RETRO-0001 | Correct stale operational claims (database, WORM, tenant, monitoring, restore) | Engineering | High | This change | Done (`283f83a`) |
| IMP-003 | RETRO-0001 | Execute a real backup/restore drill and record it in `operations/backup-log.md` | Engineering | High | Release/ops gate | Done (`283f83a`, [backup log](../operations/backup-log.md)) |
| IMP-004 | RETRO-0001 | Complete S10–S17 traceability rows with contract IDs and verified/validated distinction | QA | High | `traceability/register.md` | Done (`283f83a`) |
| IMP-005 | RETRO-0001 | Provision five triage labels, create issues and run CI after a remote and `gh` auth exist | Engineering | High | GitHub Issues/CI | Blocked |
| IMP-006 | RETRO-0001 | Add tool-failure cases and a failure-injection harness when a tool/agent path is introduced | QA | Medium | `quality/ai-evaluation.md` `EVAL-GAP-01` | Open; retrieval missing-data fail-closed test does **not** close it |
| IMP-007 | RETRO-0001 | Delete or implement the placeholder `api.rs` before any HTTP API work | Backend | Medium | `architecture` `ARCH-GAP-01` | Open (blocked by no approved API scope) |
| IMP-008 | RETRO-0001 | Assign product, security, operations and data owners | Product / Security / Ops | High | `operations/production-readiness.md` | Blocked |
| IMP-009 | 2026-10-26 audit | Replace the vacuous permission metric: add a restricted fixture and positive/negative role cases to the fixed search set (or remove the metric from the gate) | QA | High | `quality/ai-evaluation.md` `EVAL-GAP-06`; `platform/data/eval/search_eval.json` | Done (`7f6ab97`: 2 eval-only fixtures, 4 role cases — 2 allowed, 2 denied, 0 violations; fixed-set coverage) |
| IMP-010 | 2026-10-26 audit | Enforce the published CLI argument contract and test it: unknown flags, missing/empty values, flag-as-value, positional arguments and incompatible modes exit `2` before any write | Engineering | High | `platform/src/main.rs` `validate_args`; `platform/tests/cli_contract.rs`; `contracts/cli.md` | Done (`7f6ab97`: 13/13 contract tests; canonical `cargo test --locked` and `gate.ps1 -IncludeSmoke` 13/13 PASS on this host with the repo-local build profile) |
| IMP-011 | 2026-10-26 audit | Execute one authorized pilot workflow with a named operator and retain the user/owner validation record | Product + Operations | High | `operations/pilot-runbook.md`; Phase A1 | Blocked (no named pilot owner, operator or data authorization) |
| IMP-012 | 2026-10-26 audit | Approve and implement off-host encrypted backup with RPO/RTO and scheduled restore tests | Operations + Security | High | `operations/backup-policy-draft.md`; Phase B7, `PR-07` | Blocked (proposal only; no target, key owner or RPO/RTO approval) |
| IMP-013 | 2026-10-27 audit | Bind every `governance_web` mutation actor to the validated local session username and ignore submitted `actor`/`reviewer` fields | Engineering | High | `platform/src/governance_web.rs`; `platform/tests/governance_web.rs`; [web routes](../contracts/web-routes.md) | Done (`718db57`; 11/11 route tests including the impersonation regression; canonical `scripts/check.ps1` exit 0) |
| IMP-014 | 2026-10-27 audit | Replace the single-object `--audit-log` write with an append-only JSONL hash chain: verify before append, refuse corrupt/legacy files, serialise local writers and fail closed with exit `1` | Engineering | High | `platform/src/audit.rs`; `platform/src/main.rs`; `platform/tests/cli_contract.rs`; [CLI contract](../contracts/cli.md) | Done (`718db57`; 11 audit unit tests + 24 CLI contract tests, including tamper, legacy and concurrent-append cases; canonical `scripts/check.ps1` exit 0) |
| IMP-015 | 2026-10-29 hardening | Make `--health` fail closed: validate arguments first, allow only `--data`, require the three core data files, and return 0/1/2 for OK/DEGRADED/usage error | Engineering | High | `platform/src/main.rs`; `platform/tests/cli_contract.rs`; [CLI contract](../contracts/cli.md) | Done (`c827351`; 4 new contract tests; 28/28 CLI contract tests pass) |
| IMP-016 | 2026-10-29 hardening | Remove the committed absolute temp `target-dir` and the `EKP_PLATFORM_BIN` test override; isolate both crates with `[workspace]` | Engineering | High | `platform/.cargo/config.toml` (deleted); `platform/tests/cli_contract.rs`; `platform/tests/search_eval.rs`; both `Cargo.toml` files | Done (`c827351`; tests use only `CARGO_BIN_EXE_ekp_platform`; no workaround path remains) |
| IMP-017 | 2026-10-29 hardening | Obtain a scoped host-policy exception or approved CI/clean runner so the canonical check can run on the current revision | Endpoint security + Engineering | High | [B-11](../assurance/external-blockers.md#b-11-host-application-control-policy) | In progress: the earlier `279c919` canonical check exited 0 twice (E-034), the earlier `eec11f2` exited 0 once on the exact code commit (E-035), the preceding `d502147` exited 0 twice on the exact code commit (E-036), and the current `6ae475d` exited 0 once on the exact code commit (E-037), but the policy has no named owner and remains intermittent (`c827351` `edge_cases` denied; `bbe49eb` debug CLI denied; `b2d8afc` 2 edge tests and debug harnesses denied; `21e23d6` 27 tests and debug lib harness denied; E-031/E-032/E-033). CI/cross-host evidence still requires an approved runner |
| IMP-018 | 2026-10-29 review | Close the fail-open paths found by review: serialize the audit payload, validate existing audit events, validate replay actors/record IDs/workflow versions/quality transitions, refuse duplicate or non-UTF-8 `Origin`/`Sec-Fetch-Site`, make `--selftest` exclusive and make BPMN scanning UTF-8 safe; then close the follow-up review gaps: duplicate `Cookie` header lines, payload actor/event actor consistency, BPMN attribute scoping, whitespace-only audit files, multiline textarea rejection, prefix-bound BPMN parsing and logout `Secure` mirroring | Engineering | High | `platform/src/{audit,governance,governance_web,main,bpmn}.rs`; tests; [CLI contract](../contracts/cli.md); [web routes](../contracts/web-routes.md) | Done (`bbe49eb`, `b2d8afc`, `21e23d6`, `279c919`, `eec11f2`, `d502147` and `6ae475d`; 34 defect areas, 42 regression tests plus one extended date test, and ten new unit tests plus four extended tests in the deadline/effective-window batch; at `6ae475d` 366 platform tests PASS and one canonical run exited 0 on the exact code commit, E-037; at `d502147` 356 platform tests PASS and two same-command canonical runs exited 0 on the exact code commit, E-036; at `eec11f2` 354 platform tests PASS and the canonical entry point exited 0 on the exact code commit, E-035; at `279c919` 345 tests PASS and the canonical entry point exited 0 twice, E-034) |

Status vocabulary: `Open`, `In progress`, `Done`, `Blocked`, `Deferred`. `Done` requires linked evidence, not a self-assertion. IMP-001–004 are closed by the 2026-10-26 gate and restore drills; see [backup log](../operations/backup-log.md) and [traceability register](../traceability/register.md). The search work at `9d0d793` adds a fixed retrieval gate, but it does not close IMP-006: a deterministic lexical retrieval function calls no tool, so tool timeout, partial result and retry-exhaustion behavior remain unevaluated. IMP-009 is done: the fixed set now has two evaluation-only restricted fixtures and four role-restricted cases (2 allowed, 2 denied, 0 violations), so the permission metric has a real denominator. IMP-010 is done and covered by `platform/tests/cli_contract.rs`. IMP-013/014 are done at `718db57`; `c827351` added 15 route tests, `bbe49eb` extended that to 17, `b2d8afc` has 18 route tests, 5 governance-web parser tests and 20 audit unit tests, and `21e23d6` has 23 audit unit tests. IMP-015/016 are done at `c827351`; IMP-018 is done at `bbe49eb`, `b2d8afc`, `21e23d6`, `279c919`, `eec11f2`, `d502147` and `6ae475d` with thirty-four defect areas closed, forty-two regression tests, one extended date test, and ten new unit tests plus four extended tests in the deadline/effective-window batch. IMP-017 is in progress: the host policy has no named owner, but the earlier `279c919` canonical check exited 0 twice (E-034), the earlier `eec11f2` exited 0 once on the exact code commit (E-035), the preceding `d502147` exited 0 twice on the exact code commit (E-036), and the current `6ae475d` exited 0 once on the exact code commit (E-037), while the historical `21e23d6` block (27 tests plus canonical, E-033), `b2d8afc` block (2 edge tests plus canonical, E-032) and `bbe49eb` block (debug CLI, E-031) are retained as history. The earlier repo-local `.cargo/config.toml` profile overrides were removed in `c827351` after the block returned. The current canonical run is `PASS`; cross-host/CI evidence is still absent. IMP-011 and IMP-012 are outside engineering's unilateral control: they need a named pilot owner/operator and an approved off-host backup target, respectively.

## Rules for review findings

1. Separate blocking findings from suggestions. A blocking finding is a correctness, safety, security, data-integrity or traceability defect.
2. Fix the defect, then rerun the affected checks. A green baseline does not erase an unaddressed high-risk finding.
3. If a finding is disputed, record the decision and the risk acceptor in the PR or an ADR. Do not silently drop it.
4. A skipped or blocked check is recorded as such with an owner; it is never marked `PASS`.
5. A review finding that reveals a missing control must add a check or a gate if recurrence is plausible, not only a one-off fix.
6. Do not include secrets, real customer data or unredacted personal data in review reports.

## Rules for retrospectives

Use [retro-template.md](retro-template.md). A retro must produce at most a small number of owned actions; a long wish list with no owner is not a retro. Distinguish:

- what the evidence shows happened;
- what the team believes caused it (label as hypothesis);
- what will change, who owns it and when it will be reviewed;
- what remains blocked by an external decision or access.

Accepted actions go into the improvement register and, when the remote exists, become GitHub Issues. `/retro` may be used to run the session; the register is the durable record.

## Update triggers

Update the review/retro index when a report is completed, and the improvement register whenever an action is opened, changed, closed or blocked. Recheck IMP-003 and IMP-005 at every release until they are done.
