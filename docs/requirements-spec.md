# Requirements Specification (EKPI Platform)

- **Status:** Historical summary. **Not the current source of truth.**
- **Last reviewed:** 2026-10-26
- **Canonical requirements:** [governance/02](governance/02-requirements-v0.1.md), [05 functional spec](governance/05-functional-spec-v1.md), [06 data and state model](governance/06-data-and-state-model-v1.md), [07 security and connectors](governance/07-security-integration-spec-v1.md)
- **Acceptance:** [governance/04](governance/04-acceptance-test-matrix.md); **metrics:** [metrics/targets.md](metrics/targets.md); **phase gates:** [quality/phase-gates.md](quality/phase-gates.md)

> This early FR/NFR list is retained for context. It mixes current deterministic behavior with target capabilities (for example, the “restricted agent loop” is not implemented). Do not use it as an acceptance basis or as evidence of delivery.

## Scope

EKPI platform provides deterministic adjudication for ISMS compliance cases:
document versioning, active-rule library, process token replay, evidence retrieval,
decision snapshots, and restricted agent workflow.

## Functional Requirements

- FR-1 Load versioned documents with clause anchors and SHA-256 binding.
- FR-2 Load active rules with JSON Schema validation; reject drafts.
- FR-3 Replay process events against Petri-net models; detect forbidden pairs.
- FR-4 Retrieve evidence with citations and abstain on insufficient evidence.
- FR-5 Produce deterministic decision snapshots (H-6).
- FR-6 Run restricted agent loop with stopping conditions (A10).

## Non-Functional Requirements

- NFR-1 No unsafe code; clippy zero warnings.
- NFR-2 Deterministic replay across runs.
- NFR-3 Local-only governance web prototype.