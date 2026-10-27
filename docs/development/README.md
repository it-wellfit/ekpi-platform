# Engineering workflow

This repository uses the official Matt Pocock Skills plugin with GitHub Issues. See [skill setup](../agents/skill-setup.md) before first use. This document routes work to the smallest complete feedback loop; AGENTS.md is the always-loaded index, and [the documentation map](../README.md) names the canonical source for each topic. Phase status lives in [phase gates](../quality/phase-gates.md).

## Start every work item

1. Read AGENTS.md, inspect the affected code and existing tests, then read the relevant product specifications, data/state model, glossary entries, contracts and ADRs. Use [docs/README.md](../README.md) to find the owner document; do not rely on a summary.
2. Create or link a GitHub Issue with affected requirement IDs, user/business problem, acceptance criteria, constraints, risk tier, contract IDs and unresolved questions.
3. Choose a work route below. For facts outside the repo or unfamiliar technology, add /research; for unresolved domain terms or rules, add /domain-modeling.
4. Decide whether the change affects a Phase A/B gate, a public contract, the data model, security/privacy, AI evaluation or the backup/restore path. Record the decision in the issue.

## Templates

Use [the spec template](spec-template.md) for an accepted, traceable requirement set; create child issues from [the work-slice template](ticket-template.md) when multiple independently verifiable slices are needed. Use the [release and operations template](release-template.md) for version notes, smoke evidence, training and handoff. Use the [ADR template](../decisions/adr-template.md) for a lasting engineering decision and the [retro template](../reviews/retro-template.md) for a retrospective. GitHub also provides feature, bug, work-slice and improvement issue forms plus a PR template under `.github/`.

## Routes by work type

| Work type | Route |
| --- | --- |
| New feature | Inspect repo/tests → /grill-with-docs → /to-spec → /to-tickets only when multiple independently verifiable slices are needed → /implement for one small slice or /implement-spec for a multi-ticket graph → TDD → /code-review → fix findings and rerun affected checks → /pr. |
| Difficult bug | /diagnosing-bugs → reproduce with a failing regression test → locate the cause → minimal fix → regression and related tests → /code-review → fix findings and rerun affected checks → /pr. |
| UI or interaction | Use the feature or bug route, adding /prototype when comparing interaction options. Check semantic HTML, keyboard flow, screen-reader naming/status, contrast, narrow screens, and existing behavior. |
| Docs, maintenance, or isolated refactor | Inspect affected files and contracts → make the smallest change → run affected checks from quality-gates.md → review the diff → PR. Do not run the full discovery/spec sequence when behavior and acceptance are unchanged. |
| Large or uncertain work | Use /wayfinder when work spans sessions or has unresolved dependencies; /ask-matt when route selection is unclear. Use /research for unfamiliar technologies or external facts. |

/implement and /implement-spec provide TDD and code review. Review is not the endpoint: fix accepted findings, rerun affected checks, then open the PR. Split work into independently testable vertical slices; each slice must leave a runnable, reviewable state.

## Traceability and review

Every implementation slice links requirement or issue → spec/ticket → code change → contract (if any) → test evidence → release/PR decision. Record it in the [traceability register](../traceability/register.md):

- `Verified` means a named test proves conformance at the named revision.
- `Validated` means a representative user/owner confirmed fitness for the real use, with a named session.
- A synthetic test never makes a target acceptance case (`AT-*`) pass, and an `Implemented` row is not a delivered capability.

Preserve existing runtime behavior unless the approved spec changes it. Mark unresolved product decisions as unresolved; do not fill them with invented defaults.

A PR includes the problem and outcome, issue/spec links, risk tier, contract IDs, acceptance evidence, commands/results, UI accessibility evidence where relevant, data or compatibility impact, and rollback approach. Require CI success and human review before merge. A high-risk change needs an independent reviewer when the team can provide one.

## Checks and evidence

The fixed local entry point is `scripts/check.ps1` (Windows) or `scripts/check.sh` (POSIX). It runs documentation-link checks, a high-signal secret-pattern scan, format, Clippy, tests, both self-tests and rule-schema validation. The stronger pre-PR/release gate is `gate.ps1`; add `-IncludeSmoke` for report, export, coverage, HTML, data or CLI-output changes.

- Run the exact commands in [quality gates](quality-gates.md) from each crate directory; the crates are independent.
- A failed check returns to the defect flow: reproduce → regression test → minimal fix → rerun the affected check and the baseline. Do not convert a failure or a skipped check to `PASS`.
- Add E2E, contract, security, performance, accessibility or AI-evaluation checks when the risk tier or changed surface requires them. A higher tier never removes the baseline.
- Rerun the applicable AI/evaluation set after any model, prompt, retrieval, tool, rule, corpus or decision-logic change; see [AI evaluation](../quality/ai-evaluation.md).
- Keep the [traceability register](../traceability/register.md), contracts and [known gaps](../architecture/README.md) current in the same change.

## Definition of Done

A work unit is done only when all applicable conditions are evidenced:

1. The requirement, user problem and success measure are confirmed; open product/control decisions are recorded rather than invented.
2. The accepted spec and any lasting engineering decision (ADR) are in Git and linked from the issue.
3. Every in-scope requirement has a test or an explicitly accepted review-based verification; there is no untested behavior.
4. Primary cases and high-risk/refusal/failure cases pass at the named revision.
5. `scripts/check` passes; risk-appropriate additional checks pass; a failed/skipped check is recorded with an owner.
6. Major review findings are fixed and the affected checks are rerun; remaining findings have an accepted risk owner.
7. The version is committed and, for a release, tagged; a backup is created and a restore drill passes (or the release is explicitly blocked).
8. Delivery, operation, recovery, known-limit and handoff documents are current.
9. For an MVP claim, a real authorized workflow has been executed and validated by a representative user/owner, not only synthetic tests.
10. `/retro` has been run and accepted improvements are tracked in the [improvement register](../reviews/README.md#improvement-register).

A merge additionally requires CI success and human approval. A release additionally requires recorded smoke checks, release notes, restore evidence and the Phase A gate decision.

## Phase gates

Do not describe the prototype as an MVP or a production product on the basis of passing tests alone.

- [Phase A](../quality/phase-gates.md#phase-a-0--1-mvp-gate): current status **not approved**; real-user validation and complete pilot-scope traceability are outstanding. Local A4/A8 evidence passes at `b2bae3e` and in the `7fd0c0b`/`56a944f` restore drills; off-host backup and RPO/RTO remain Phase B.
- [Phase B](../quality/phase-gates.md#phase-b-mvp--production-grade-gate): current status **blocked**; no deployment target, identity, data policy or operations owner.
- [Production readiness](../operations/production-readiness.md): current answer **not production ready**.

A gate may be approved only by the named accountable role with linked evidence. A retrospective or a delivery pressure cannot lower a gate.

## Continuous improvement

After each merged work unit, release, incident or pilot, run /retro. Convert each accepted improvement into the cheapest durable control: a regression test or automated check for mechanical mistakes, or a concise agent instruction/reference for judgment and navigation. Link the improvement to the session, incident or PR that exposed it. Use the [retro template](../reviews/retro-template.md) and keep the [improvement register](../reviews/README.md#improvement-register) current.

## Update triggers

Update this workflow when the issue tracker, templates, risk tiers, required checks, Definition of Done, phase gates or review/retro process changes. Keep the canonical gate and traceability documents aligned in the same change.
