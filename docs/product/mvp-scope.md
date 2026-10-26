# Phase A MVP scope and non-goals

- **Status:** Active, evidence-limited
- **Owner:** Product + Engineering
- **Last reviewed:** 2026-10-26
- **Exit criteria:** [Phase gates](../quality/phase-gates.md)
- **Target-product requirements:** [governance/02](../governance/02-requirements-v0.1.md), [05](../governance/05-functional-spec-v1.md), [06](../governance/06-data-and-state-model-v1.md), [07](../governance/07-security-integration-spec-v1.md)

## Definition of the Phase A MVP

Phase A is complete when one real, representative workflow can be executed end to end on authorized non-sensitive data, with traceable inputs, a reproducible decision, a validated restore path and a named operator. The current repository is a **local prototype**; it has not met that definition yet.

The MVP is deliberately narrower than the target product. It proves the adjudication loop and the documentation/test discipline, not enterprise scale.

## Capability scope

| Capability | Current implementation | Phase A acceptance | Current evidence | Status |
| --- | --- | --- | --- | --- |
| Versioned documents and clause anchors | `platform/data/docs.json`; `docs` and `engine` modules; SHA-256 bindings | A decision resolves to one source version and a clause anchor; stale or ambiguous versions abstain | Unit/integration tests and self-test | Implemented for synthetic data |
| Active-rule layer and schema validation | `platform/data/rules.json`; `design/rule-schema-v0.1.1.schema.json`; `--declared-rules-schema` | 19 active + 1 draft record; draft is never consumable; invalid schema fails closed | Schema command (20 records), self-test, tests | Implemented for synthetic data |
| Process replay | `process` module; process models; forbidden pairs, deadlines, loops | Forbidden ordering produces `violate`; insufficient events produce `insufficient_evidence` or `escalate` | Seed/edge cases and self-test | Implemented for synthetic data |
| Evidence retrieval and abstention | `retrieval`/`evidence`; deterministic threshold; citations; `--search`; fixed 28-case retrieval set (including 4 role-restricted cases) | Missing, conflicting or unregistered evidence never produces an unsupported `comply`; search cites a source or abstains | Evaluation corpus, hard gates H-1–H-6, `--eval` search gate | Implemented for synthetic data; lexical search only; role filter is library/eval evidence, not identity or tenant enforcement |
| Deterministic decision snapshots | `snapshot` module; replay checks | Same input snapshot replays identically | Self-test and integration tests | Implemented for synthetic data |
| Evaluation harness and reports | `eval` crate; `--out`, `--coverage-report`, `--html` | Reproducible metrics, per-case failure detail, coverage and zero-script HTML | `eval` tests and self-test; smoke gate | Implemented |
| LegalRuleML export | `export` module; `--export-legalruleml` | Official root/namespace and required statement counts; no stub export | Smoke gate structural checks | Implemented for active corpus |
| Local governance workbench | `governance_web`; append-only hash-chained ledger | Create/link/review/publish/gate flows work on synthetic data; foreign-origin mutation is refused | Route/integration tests | Partial: local only, no enterprise auth or tenant isolation |
| Enterprise identity and authorization | `local_auth` library exists but is not wired into the served app | Named users, MFA/IdP, role separation, access review and denial evidence | None | Not in Phase A scope / blocked for Phase B |
| Tenant isolation | `tenant` library exists but is not wired | Tenant-scoped storage and authorization tests | None | Not in Phase A scope / blocked for Phase B |
| Microsoft/Google connectors | None; manual source references only | Authorized connector with source-version and permission evidence | None | Out of scope |
| AI-assisted drafting | None; no model, prompt or provider approved | Evaluated, human-reviewed draft that cannot self-publish | None | Out of scope |
| Production deployment and operations | None; no target, identity or owner approved | Approved environment, monitoring, rollback and on-call | None | Blocked |

## MVP acceptance scenarios

The MVP must demonstrate at least these scenarios on synthetic or authorized non-sensitive data:

1. A compliant change with a valid approval record returns `comply` with resolvable citations.
2. A change executed before approval returns `violate`; the forbidden transition is never released.
3. An expired or draft rule is not applied; the system abstains rather than guessing.
4. Missing cross-document evidence returns `insufficient_evidence` with no unsupported conclusion.
5. A stale or mismatched document version is detected and does not silently use the wrong source.
6. A prompt-injection string in source content does not change the verdict.
7. A deadline breach follows the configured `block`, `escalate` or `warn` semantics.
8. The same snapshot replays identically and the report numbers are derivable from persisted artifacts.
9. A local governance record is written to the append-only ledger and the audit view verifies the chain.
10. A fresh clone from the backup bundle passes the full local check and the restore is logged.
11. A knowledge question returns either a verbatim citation with its source anchor or `insufficient evidence`; the fixed retrieval set passes without claiming AI quality or authorization.

Items 1–9 have synthetic test coverage. Item 10 is a release/operations gate, not a unit test; it must be executed and logged.

## Non-goals

Phase A does **not** claim or attempt:

- Multi-tenant SaaS, enterprise SSO/MFA, delegated administration or access reviews.
- Live Microsoft 365, Google Workspace or other third-party connectors.
- Copying authoritative documents into the platform; evidence remains a source reference.
- AI approval, autonomous gate release, model training on customer data or an AI quality claim.
- Public network exposure, high availability, disaster recovery beyond the local restore drill, or 24×7 support.
- Regulatory certification, legal advice or a statement that the synthetic corpus represents a real jurisdiction.
- Microservices, Kubernetes, a relational database or a message queue. The current implementation is a local Rust process with JSON data.

## Phase A exit conditions

Before Phase A can be approved:

- The Phase A table in [quality/phase-gates.md](../quality/phase-gates.md) has no unresolved `Not met` row for the approved scope.
- A real, authorized workflow has been executed and the result recorded with user/owner validation, not only synthetic tests.
- The backup bundle and restore drill are recorded in [operations/backup-log.md](../operations/backup-log.md).
- Every accepted requirement in scope has a row in the [traceability register](../traceability/register.md) with code, test and release evidence.
- Known limitations and operating instructions are current, and a named operator can run the system from the documentation.
- A retrospective has been completed and accepted improvements are tracked in [reviews](../reviews/README.md).

## Change control

Adding a capability to Phase A requires an accepted spec or issue, a risk tier, an updated acceptance case, and a decision on whether it changes the Phase B boundary. Removing a capability requires an explicit scope decision; do not silently drop a control because the prototype does not implement it.

## Update triggers

Update this file when scope, acceptance scenarios, phase boundaries, or the status of an in-scope capability changes. Keep the detailed requirements in `docs/governance/` and the evidence in the traceability register.
