# Risk model (historical summary)

- **Status:** Historical; retained for early context. **Not current evidence.**
- **Last reviewed:** 2026-10-26
- **Canonical current model:** [Architecture threat model](architecture/threat-model.md)
- **Target controls:** [Security integration spec](governance/07-security-integration-spec-v1.md)
- **Readiness:** [Production readiness](operations/production-readiness.md)

> This early summary described target-product threats and included control claims that were not implemented. It is kept only to preserve the record. Use the canonical threat model above for the current local-prototype risks and unsupported claims.

## Threats

- T-1 Prompt injection embedded in documents (H-5): the deterministic engine treats source text as data; injected directives do not become system rules. There is no served LLM/agent path.
- T-2 Forbidden process transitions (H-1): hard gate; a forbidden pair is never released.
- T-3 Unanchored conclusions (H-4): every definitive verdict must cite a resolvable anchor.
- T-4 Draft rule poisoning (A1/A8): draft rules are rejected and never enter engine space.
- T-5 Unregistered rule ids (D-02): conservative abstention.
- T-6 Local ledger tampering: append-only JSONL with a SHA-256 chain gives tamper evidence against accidental edits. It is **not** WORM and does not resist a filesystem administrator.
- T-7 Unauthorized local access: no application authentication, authorization or tenant isolation is served. Loopback binding and host permissions are the only boundary.

## Mitigations (current)

- Version-pinned document bindings.
- Active-only rule layer with schema validation.
- Conservative abstention on missing evidence.
- Deterministic snapshots and replay checks.
- Append-only, hash-chained local ledger (tamper-evident, not WORM).
- Loopback-only web bind, same-origin mutation guard and restrictive headers.

## Not implemented

Enterprise identity/MFA, RBAC, tenant isolation, WORM/retention, encryption at rest/in transit, centralized logging/monitoring, incident response and penetration testing. See [production readiness](operations/production-readiness.md) and the [threat model](architecture/threat-model.md).
