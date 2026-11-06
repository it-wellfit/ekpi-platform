# ADR-0003: Ecommerce Platform Integration Boundary

- **Status:** Accepted
- **Date:** 2026-11-05
- **Owner:** Architecture
- **Deciders:** Architecture + Integration
- **Related:** ADR-0002, `Knowledge/sources/research-protocol.md` §1.1, OQ-F2-5

## Context

momo / PChome / Cyberbiz API specifications are not yet obtained. Contract facts (endpoints, fields, signatures, error codes, rate limits) must not be inferred from the internet (research-protocol §1.1). The existing Wellfit codebase demonstrates a proven pattern: `ColumnContract` (name-based resolution, hard failure on missing required columns) and `ChannelAdapter`-style trait boundaries.

## Decision

We will build a **generic skeleton first** (`ChannelAdapter` trait + UDM + rule engine), with **momo as the first complete implementation** (only after obtaining its official API spec). PChome/Cyberbiz follow the same skeleton once their specs are obtained. Each unverified fact is marked `Unverified` and logged in `Knowledge/issues/`.

## Consequences

### Positive
- No fabricated contract values; all external facts traceable to source-register.json with SHA-256.
- Reusable skeleton reduces per-platform effort.
- Clear blocking: Phase E cannot start without momo spec.

### Negative / accepted cost
- Phase E blocked until external spec obtained (OQ-F2-5).
- Adapter stubs (`pending_contract`) add indirection until specs arrive.

### Follow-up
- Phase B: create `adapters-momo` crate (skeleton only).
- Phase E: implement real endpoints after spec obtained.
- Register momo/PChome/Cyberbiz spec requests in `Knowledge/issues/mtx-issue-register.md`.

## Alternatives considered

| Alternative | Why rejected |
| --- | --- |
| Implement momo from reverse-engineered/online data | Violates research-protocol §1.1; produces unverifiable contracts |
| Wait for all three platforms before starting | Delays skeleton work; momo spec may arrive first |

## Evidence
- `Knowledge/sources/research-protocol.md` §1.1: contract facts must not be web-inferred
- `Wellfit/crates/wellfit_core/src/contract.rs`: ColumnContract name-based resolution pattern
- `Dev/docs/research/proposals/OQ-F1_F2_輔翼ERP正式需求提案_v1.0.md:70`: OQ-F2-5 (platform API spec request)

## Compliance and security impact
- No trust boundary change. Webhook signature verification required for each adapter (E4).

## Supersession
- Supersedes: none
- Superseded by: none