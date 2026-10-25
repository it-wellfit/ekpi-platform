# Reviews and improvements

- **Status:** Active; one bootstrap retrospective completed
- **Owner:** Engineering + QA
- **Last reviewed:** 2026-10-26
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
| — | — | No completed review report yet | — | — | — |

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

Status vocabulary: `Open`, `In progress`, `Done`, `Blocked`, `Deferred`. `Done` requires linked evidence, not a self-assertion. IMP-001–004 are closed by the 2026-10-26 gate and restore drill; see [backup log](../operations/backup-log.md) and [traceability register](../traceability/register.md). The search work at `9d0d793` adds a fixed 24-case retrieval gate, but it does not close IMP-006: a deterministic lexical retrieval function calls no tool, so tool timeout, partial result and retry-exhaustion behavior remain unevaluated.

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
