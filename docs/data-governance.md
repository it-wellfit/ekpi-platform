# Data governance

- **Status:** Draft for the local prototype; production data governance is **blocked**
- **Owner:** Data governance (role not yet assigned)
- **Last reviewed:** 2026-10-26
- **Data model:** [governance/06](governance/06-data-and-state-model-v1.md)
- **Security spec:** [governance/07](governance/07-security-integration-spec-v1.md)
- **Decision register:** [governance/08](governance/08-decision-register.md)

## Scope and current boundary

The repository is a local prototype. Use synthetic data or authorized non-sensitive data only. Do not load real customer documents, personal data, regulated data or organization-specific facts into the fixture, ledger, reports or tests.

There is **no tenant isolation** in the served app. The `tenant` module is compiled but not wired. There is a **local-only session login**, not enterprise authentication. The governance web app binds to loopback, requires a local session for data pages and roles for mutations, and has no IdP, MFA, recovery, persistent user directory or access review. Do not describe the data as tenant-isolated or enterprise access-controlled.

## Authorization

- Local-only web prototype; loopback bind; one startup-configured account; Viewer/Editor/Approver checks on data pages and mutations.
- Credentials come from `EKPI_LOCAL_AUTH_USER`, `EKPI_LOCAL_AUTH_PASSWORD` and `EKPI_LOCAL_AUTH_ROLE`; there is no default account and no persistent identity store. Passwords are held as salted PBKDF2-HMAC-SHA256 hashes in process memory and are never logged.
- The ledger records the validated local session username as the actor for web mutations. This is a local process-memory identity, not an enterprise-verified identity. A local administrator can read or rewrite the ledger.
- Target identity, RBAC, MFA, tenant isolation, access review and denial evidence are Phase B requirements (`DEC-02`, `DEC-12`).

## Data handling

- Source documents are versioned with SHA-256 bindings and clause anchors. The platform stores **source references and metadata**, not authoritative source contents.
- The governance ledger stores event metadata and references, not document contents.
- The ledger is append-only JSONL with a SHA-256 chain for tamper detection. It is **not WORM**, has no external anchor and can be rewritten by a filesystem administrator.
- No organization-specific facts are prefilled in compliance deliverables.
- Reports and exports are generated artifacts; they may contain operator-entered text. Treat them as sensitive if the input was sensitive.

## Classification and retention

No approved data classification, retention schedule, legal hold, residency rule, deletion procedure or data owner exists (`DEC-13`, `DEC-14`). Until those are approved:

- keep data synthetic or non-sensitive;
- minimize what is written to the ledger and reports;
- do not promise deletion, retention or residency guarantees;
- back up the ledger separately if it must be retained; a git bundle does not include it.

## Audit and integrity

- The local ledger records sequence, timestamp, actor, payload, previous hash and SHA-256.
- The audit page verifies chain continuity and recomputes hashes.
- Integrity evidence is local only. There is no enterprise audit store, retention control, legal hold or external anchoring (`DEC-14`).

## Incident and access

- Suspected tampering, data loss or secret exposure: follow the [incident log](operations/incident-log.md) procedure and preserve evidence.
- Secret handling: [secrets policy](operations/secrets-policy.md).
- Security boundary and unsupported claims: [threat model](architecture/threat-model.md) and [security hardening](deployment/security.md).

## Production blockers

Production data use requires an approved data owner, classification, lawful basis, retention/deletion, residency, access control, audit retention and AI data boundary. These are tracked in [production readiness](operations/production-readiness.md). They are not documentation gaps to be filled with optimistic text.

## Update triggers

Update this page when the data model, storage location, classification, retention, access control, audit design or applicable decision changes. Update the data-flow/threat model in the same change when the boundary changes.
