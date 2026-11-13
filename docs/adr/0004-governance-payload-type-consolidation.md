# ADR-0004: Consolidate governance event payload types in the domain crate

- **Status:** Accepted
- **Date:** 2026-11-13
- **Owner:** Engineering
- **Deciders:** Engineering (refactor-criteria C6 slice; no business decision)
- **Related:** [Refactor criteria](../development/refactor-criteria.md) C6; traceability register `R27`; `platform/tests/governance_type_compat.rs`; `platform/tests/c6_parity.rs`

## Context

`platform::governance` and `ekp_domain::governance` each defined their own
`EventPayload` enum and payload structs. Measured on 2026-11-13:

- All 15 public type definitions in the duplicated family are identical after
  whitespace normalization (the enum, nine payload structs, `QualityCheckResult`,
  `LedgerEvent`, `GovernanceSnapshot`, and the two input structs).
- The nine payload structs carry identical derives and serde attributes.
- `platform/src/governance.rs` has no inherent `impl` on payload types; its only
  `impl` block is `impl Ledger`.
- `EventPayload` has 80 references, all through the platform module path.
- The JSONL ledger hashes the serialized event body, so the wire format is a
  hard compatibility constraint: a migration must not change one byte.

A previous review classified C6 as "next slice, not a simple re-export" because
the payload structs were believed to differ. The measurement above shows they do
not differ; the remaining risk is wire-format drift and future re-divergence.

## Decision

`ekp_domain::governance` is the single source for the governance event payload
type family. `platform::governance` re-exports, rather than redefines:

`EventPayload`, `Project`, `Evidence`, `EvidenceReview`, `Workflow`,
`WorkflowPublication`, `QualityCase`, `QualityCaseUpdate`, `QualityCheckResult`,
`QualityInspection`, `GateDecision`.

Ledger persistence (`Ledger`, `LedgerEvent`, hash/lock helpers, snapshot
assembly) stays in `platform`. Non-goals: deduplicating the validation
functions, `QUALITY_CHECKLIST`, the input structs or the persistence layer;
this is not a full module re-export, because the platform module keeps private
copies of validation helpers with the same names as the domain functions.

## Consequences

### Positive

- One type identity across domain, adapters and platform; no conversion layer.
- The duplicated payload family is removed (131 lines deleted from
  `platform/src/governance.rs`, 12 inserted).
- Call sites are unchanged, because the module path and names are preserved.

### Negative / accepted cost

- A future domain change to a payload struct now directly affects the ledger
  format. Mitigation: the fixtures in `platform/tests/governance_type_compat.rs`
  fail on any wire-format change, and `c6_parity` compares both module paths over
  all nine event kinds.
- Validation helpers, the checklist and the persistence types remain duplicated;
  they are separately tracked and are not claimed as consolidated here.

### Follow-up

- Keep both compatibility tests green in the canonical gate.
- A later candidate may deduplicate the private validation helpers; that needs
  its own slice because the platform copies are module-private and the domain
  ones are public.

## Alternatives considered

| Alternative | Why rejected |
| --- | --- |
| Do nothing | Two type identities and 80 references keep inviting divergence; the ledger would silently fork on the next edit. |
| Full module re-export (`pub use ekp_domain::governance::*`) | Name collisions with the platform module's private validation helpers; wider blast radius than the payload family. |
| Conversion functions between the two families | Preserves duplication, adds runtime conversion and still requires a wire-format lock. |

## Evidence

- 2026-11-13 structural comparison: 15/15 public type definitions identical;
  nine payload struct derives/attributes identical; no inherent impls on payload
  types in `platform`.
- `cargo test --locked -p ekp_platform --test governance_type_compat`: 1 passed
  before and after the migration; nine JSONL fixtures round-trip byte-for-byte
  through both module paths.
- `cargo test --locked -p ekp_platform --test c6_parity`: un-ignored and passing
  after the migration.
- `cargo test --locked --workspace`: 570 passed, 0 failed, 0 ignored on
  2026-11-13.
- Canonical gate result and revision: recorded in register `R27`; no CI run is
  claimed.

## Compliance and security impact

- No trust-boundary, identity, access-control, retention or data-classification
  change. The ledger's tamper-evident chain depends on serialization, which the
  compatibility fixtures now lock. No security review is triggered by a pure
  type-identity change, but any future payload edit must update the fixtures and
  the ledger contract together.

## Supersession

- Supersedes: none.
- Superseded by: none.
