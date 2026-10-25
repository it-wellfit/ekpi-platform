# Engineering workflow

This repository uses the official Matt Pocock Skills plugin with GitHub Issues. See [skill setup](../agents/skill-setup.md) before first use. This document routes work to the smallest complete feedback loop; AGENTS.md is the always-loaded index.

## Start every work item

1. Read AGENTS.md, inspect the affected code and existing tests, then read relevant product specifications, data/state model, glossary entries, and ADRs.
2. Create or link a GitHub Issue with affected requirement IDs, user/business problem, acceptance criteria, constraints, risk tier, and unresolved questions.
3. Choose a work route below. For facts outside the repo or unfamiliar technology, add /research; for unresolved domain terms or rules, add /domain-modeling.

## Templates

Use [the spec template](spec-template.md) for an accepted, traceable requirement set; create child issues from [the work-slice template](ticket-template.md) when multiple independently verifiable slices are needed. Use the [release and operations template](release-template.md) for version notes, smoke evidence, training, and handoff. GitHub also provides feature, bug, and work-slice issue forms plus a PR template under .github/.

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

Every implementation slice links requirement or issue → spec/ticket → code change → test evidence → PR decision. Preserve existing runtime behavior unless the approved spec changes it. Mark unresolved product decisions as unresolved; do not fill them with invented defaults.

A PR includes the problem and outcome, issue/spec links, risk tier, acceptance evidence, commands/results, UI accessibility evidence where relevant, data or compatibility impact, and rollback approach. Require CI success and human review before merge. A high-risk change needs an independent reviewer when the team can provide one.

## Definition of Done

A work unit is done only when its acceptance criteria pass; each Must requirement has traceable test or review evidence; affected existing behavior is covered; risk-appropriate checks pass; accepted review findings are fixed and affected checks rerun; documentation and known limits are current; and the PR links the source issue/spec. A merge additionally requires CI success and human approval. A release also needs recorded smoke checks, operator/recovery guidance, and release notes.

## Continuous improvement

After each merged work unit or release, run /retro. Convert each accepted improvement into the cheapest durable control: a regression test or automated CI check for mechanical mistakes, or a concise agent instruction/reference for judgment and navigation. Link the improvement to the session or PR that exposed it.
