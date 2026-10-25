# Traceability

- **Status:** Active; register incomplete
- **Owner:** QA
- **Last reviewed:** 2026-10-26
- **Register:** [register.md](register.md)
- **Target-product baseline:** [governance/10](../governance/10-traceability-baseline.md)
- **Acceptance matrix:** [governance/04](../governance/04-acceptance-test-matrix.md)

## Purpose

Traceability answers: for a given requirement, what implements it, what test proves it, what release contains it, and has a real user validated it? It is not a list of documents. It is the chain that lets a reviewer reproduce a claim.

There are two traceability layers:

| Layer | Scope | Owner | Source |
| --- | --- | --- | --- |
| Target-product requirements | Business needs → FR/NFR → module → state → acceptance case → evidence → open decision | Product + QA | [governance/10](../governance/10-traceability-baseline.md) |
| Repository changes | Work item (`S` ID / issue) → spec → code → test → contract → release evidence → review | Engineering + QA | [register.md](register.md) |

Do not merge them into one table. The governance baseline describes the target product; the register records what this repository actually changed and proved. A prototype test does not make a target acceptance case pass.

## ID rules

| Prefix | Meaning | Where defined |
| --- | --- | --- |
| `FR-`, `NFR-` | Product functional/non-functional requirement | [governance/05](../governance/05-functional-spec-v1.md) |
| `AT-` | Target acceptance case | [governance/04](../governance/04-acceptance-test-matrix.md) |
| `DEC-` | Open business/control decision | [governance/08](../governance/08-decision-register.md) |
| `ADR-` | Engineering decision record | [docs/adr/](../adr/) |
| `S` | Repository work slice / change item | [register.md](register.md) |
| `CTR-` | Contract ID | [contracts](../contracts/README.md) |
| `M-`, `H-` | Metric or hard gate | [metrics/targets.md](../metrics/targets.md) |
| `IMP-` | Accepted improvement | [reviews](../reviews/README.md#improvement-register) |
| `ARCH-GAP-`, `BE-GAP-`, `FE-GAP-`, `EVAL-GAP-` | Known gap | Architecture/backend/frontend/quality pages |

IDs are stable. Do not reuse an ID for a different item; do not renumber an existing `S` or `AT` item.

## Register columns

| Column | Meaning |
| --- | --- |
| Requirement / work item | The requirement, `S` item or issue ID |
| Spec / ticket | Accepted spec, ticket or decision that defines the work |
| Code change(s) | Files/modules changed; use paths, not vague areas |
| Contract(s) | `CTR-` ID when a public interface is affected |
| Test evidence | Exact command/test and the revision/date it was run |
| Verified | Does a test prove the implementation matches the spec? `Yes` / `No` / `Partial` |
| Validated | Has a representative user/owner confirmed it solves the real problem? `Yes` / `No` / `Not attempted` |
| Release / PR | Commit, tag, PR or release record containing it |
| Status | `Implemented`, `Verified`, `Validated`, `Planned`, `Partial`, `Blocked`, `Historical` |

`Verified` and `Validated` are different. A passing test verifies conformance; only a user/owner can validate fitness for the real use. Never mark `Validated` from a synthetic test.

## Rules

1. Add or update the row in the same change as the code/test/spec.
2. Every `Implemented` row must name a file and a test. An unimplemented plan is `Planned`, not `Implemented`.
3. A skipped or blocked check is named as such; it is never `PASS`.
4. A public number must be derivable from the named artifact or persisted count.
5. If a target acceptance case is still `NOT RUN`, do not imply it passed by linking a prototype unit test.
6. A release row links the commit/tag, gate output, backup/restore evidence and the review record.
7. Historical rows stay for the record; mark them `Historical` and point to the current evidence.

## Coverage check

Before a release or pilot claim:

- Every in-scope `FR`/`NFR` has at least one `S`/issue row or an explicit `Planned`/`Blocked` note.
- Every served surface and contract in [contracts](../contracts/README.md) has a contract ID and a test.
- Every high-risk change has negative/failure evidence, not only a happy path.
- Every `Implemented` row has a test path; every `Validated` row names the user/owner and the session.
- The Phase A/B gate rows point to the evidence that supports their status.

## Current gaps

- S10–S17 and the contract IDs were added on 2026-10-26 but most rows lack release/validation evidence.
- No row is `Validated`; no real user session has occurred.
- The acceptance matrix remains `NOT RUN` for target cases.
- The register is not yet enforced by a script; review it manually until an ID/link check exists.

## Update triggers

Update the register whenever a requirement, code path, test, contract, release, gate decision or review changes. Update this page when the ID scheme, columns, evidence standard or review process changes.
