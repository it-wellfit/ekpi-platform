# RETRO-0001: Bootstrap round

- **Date:** 2026-10-26
- **Facilitator:** Engineering (self-review; no independent facilitator)
- **Scope:** Workflow bootstrap, community skills, documentation architecture, quality gates
- **Status:** Draft; actions tracked in the [improvement register](README.md#improvement-register)
- **Evidence:** repository state at `a2520ed` plus the uncommitted documentation change under review

## Ground rules

This is a self-review of the bootstrap round. It is **not** evidence of team performance, user value or MVP acceptance. No named product, security or operations owner participated. Treat the causes below as hypotheses unless they are directly observable.

## What happened (observed)

- 27 Matt Pocock skills were installed and their frontmatter validated. The gate script was tightened and the real run passed 10/10 steps.
- Defects EC-006, BC-143 and BC-194 were fixed; the LegalRuleML exporter was restored (51,026 bytes, 1,168 lines).
- `Platform::subject_activities` and its tests were added; the platform test count became 183.
- A first commit (`a2520ed`, 199 files) captured the working baseline; the worktree was clean at that point.
- A canonical documentation index, product/architecture/backend/frontend/contracts/quality/operations pages, a Phase A/B gate page and an AI evaluation policy were drafted.
- Live metadata for three community skill repositories was checked; no additional bundle was installed.
- The repository still has no remote, no `gh` authentication, no pilot user, no named security/operations owner and no executed restore drill.

## What went well

- The evidence rule held: stale claims such as “WORM”, “tenant isolated” and “PostgreSQL required” were found by inspecting code and wiring, not by trusting prose.
- The module audit distinguished served surfaces from library-only experiments, which prevents capability inflation.
- The first commit created a reproducible baseline before the documentation expansion.
- The decision to keep GitHub Issues as the tracker and avoid duplicating governance rules in new files preserved one source of truth.

## What did not go well

- Early operational documents overstated controls (WORM, tenant isolation, monitoring) and the final validation report implied a restore drill that had not been run. This is a trust defect, not a wording defect.
- Documentation and check wiring were allowed to diverge: `scripts/check` did not verify documentation links, so stale pages could pass the gate.
- The traceability register retained planned or unverified rows as `implemented` and lacked contract IDs and a verified/validated distinction.
- A local-only repository accumulated a large uncommitted documentation change before the backup/restore path was exercised.
- The placeholder `api.rs` was left in the tree without a hard guard against exposure.

## Hypotheses (not proven)

- The overstatement came from writing target-state documents in the same voice as current-state documents.
- The gap persisted because no check failed when a document contradicted the code.
- The large uncommitted change grew because the documentation scope was expanded faster than the commit/backup cadence.

## Actions

| ID | Action | Owner | Due / trigger | Status |
| --- | --- | --- | --- | --- |
| IMP-001 | Add `scripts/check-docs` and make it part of `scripts/check` and `gate.ps1` | Engineering | This change | In progress |
| IMP-002 | Correct stale operational claims and label historical documents | Engineering | This change | In progress |
| IMP-003 | Run a real bundle + restore drill and log it | Engineering | Before release/tag | Open |
| IMP-004 | Complete S10–S17 traceability rows with contract IDs and verified/validated columns | QA | This change | In progress |
| IMP-005 | Provision triage labels, create issues and run CI once remote/auth exist | Engineering | Remote/auth available | Blocked |
| IMP-006 | Add tool-failure cases when a tool/agent path is introduced | QA | AI/tool scope approved | Open |
| IMP-007 | Remove or implement `api.rs` before HTTP API work | Backend | API scope approved | Open |
| IMP-008 | Assign product, security, operations and data owners | Product/Security/Ops | Before pilot/production claim | Blocked |

## What we will not do

- We will not lower a gate to make the current state look approved.
- We will not install an additional community skill bundle merely to increase coverage; the evaluated alternatives did not justify the maintenance and trust cost.
- We will not copy governance rules into new summaries; links and local consequences only.

## Review

Recheck this retro at the next release or after the first real pilot session. A retro action is closed only with linked evidence in the improvement register.

## Next retro trigger

The next retro is required after the first real pilot workflow, after any S1/S2 incident, or when a Phase gate changes status.
