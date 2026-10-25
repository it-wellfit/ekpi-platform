# Requirements Specification (EKPI Platform)

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