# ADR-0002: Unified Integration Architecture — ekpi-platform as Governance/Approval Hub

- **Status:** Accepted
- **Date:** 2026-11-05
- **Owner:** Architecture
- **Deciders:** Architecture + Product + Engineering
- **Related:** ADR-0001 (tech stack baseline, Dev/), ADR-0004 (KB location), OQ-F1/OQ-F2, `docs/quality/phase-gates.md` (Phase A not approved, Phase B blocked)

## Context

The earlier proposal (Kafka + k3s + Service Mesh) conflicts with the binding "no Docker/container" decision (`windmill_deployment_no_docker` correction) and with ADR-0001 D-02 (root workspace stays edition 2021/SQLite). ekpi-platform already implements a 6-crate Clean Architecture with PostgreSQL 18.6 adapters, Argon2id, CSRF, zero-JS workbench, and an append-only audit chain. Re-implementing a parallel governance layer would violate `cargo_workspace_architecture` and duplicate working code.

## Decision

We will use **ekpi-platform as the governance/approval hub** (extend existing layers, do not create parallel modules). Integration with Wellfit (execution/data plane) and the root workspace FlapsErp (ERP execution plane) happens via **PostgreSQL event tables (outbox/inbox) + narrow API contracts**; **no cross-repo path dependencies**. Kafka is demoted to a pluggable adapter slot (reserved, not implemented). Deployment is native binary (sc.exe/systemd/launchd). Redis is optional (cache/distributed rate limiting only).

## Consequences

### Positive
- Reuses proven ekpi-platform architecture (PostgreSQL 18.6, audit chain, RBAC, zero-JS workbench).
- No container runtime required; native binary deployment.
- Clear ownership: ekpi-platform owns governance tables; platform-hub owns adapter/UDM/eventbus.

### Negative / accepted cost
- Three parallel tech baselines remain (ekpi-platform edition 2024/PostgreSQL, root workspace edition 2021/SQLite, Wellfit edition 2021/SQLite). Unification is a separate POC.
- Kafka deferred until throughput threshold (>5,000/s sustained or outbox backlog >100k with P99 <100ms unmet).

### Follow-up
- ADR-0003 (ecommerce integration boundary).
- Phase B: create `Dev/platform-hub/` workspace.
- Phase C: extend ekpi-platform domain/application/adapters.

## Alternatives considered

| Alternative | Why rejected |
| --- | --- |
| Kafka + k3s + Service Mesh | Conflicts with no-Docker decision; over-engineered for current scale |
| New parallel governance crate in root workspace | Duplicates ekpi-platform; violates Clean Architecture |
| SQLite-only shared database | Insufficient for multi-system event replay and concurrent writes |

## Evidence
- `ekpi-platform/Cargo.toml`: resolver 3, edition 2024, sqlx 0.9.0 (postgres)
- `ekpi-platform/migrations/20261102000000_init.sql`: PostgreSQL 18.6 schema with audit chain trigger
- `Dev/docs/adr/ADR-0001-tech-stack-baseline.md` D-02: root workspace stays edition 2021/SQLite

## Compliance and security impact
- No trust boundary change. Audit chain (`governance_audit_events`) extended with `source = "platform-hub"` events.
- Agent auto-approval remains **out of scope** per `phase-gates.md` §Out of scope until DEC-15 resolves.

## Supersession
- Supersedes: none (supersedes the Kafka+k3s proposal in the earlier plan `1792461065184-unified-integration-architecture.md`)
- Superseded by: none