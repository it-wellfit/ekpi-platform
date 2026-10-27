# Secrets and sensitive data policy

- **Status:** Active for the local prototype
- **Owner:** Engineering + Security (security owner not yet assigned)
- **Last reviewed:** 2026-10-26
- **Readiness:** [Production readiness](production-readiness.md)

## Core rules

1. **No secrets in the repository.** Do not commit passwords, API keys, tokens, private keys, connection strings or real customer data. The repository is not currently scanned by an executed remote CI job; review staged changes manually and treat the CI secret-scan definition as unproven until it runs.
2. **Local configuration stays local.** `.env`, key files and local ledgers are ignored by Git. `.env.example` contains placeholders only. The application does **not** automatically load `.env`; only variables actually read by the code have an effect.
3. **Only the documented variables are functional.** The current code reads `EKP_GOVERNANCE_DATA` (local ledger path), `EKPI_LOCAL_AUTH_USER`, `EKPI_LOCAL_AUTH_PASSWORD`, `EKPI_LOCAL_AUTH_ROLE` (local workbench account) and `EKPI_COOKIE_SECURE` (cookie flag). No other placeholder is read. Do not assume a secret or setting is active because it appears in `.env.example`.
4. **Local-only governance prototype.** The workbench has a local session login, not enterprise authentication; no external-source permission enforcement and no AI-produced workflow drafts that self-publish. The prototype stores source references and metadata, not authoritative source contents.
5. **No prefilled organization facts.** Compliance deliverables must not prefill organization-specific facts.
6. **Sensitive data handling.** Do not put personal data, customer content or credentials in the ledger, reports, test fixtures, issue text or review records. The local ledger is not encrypted and is not WORM. The local login password is held in the process environment and hashed in memory; never commit or log it.

## `.env` and `.env.example`

- `.env.example` documents placeholders; it must never contain real values.
- `.env` is local-only and must remain untracked. If a local secret file is needed, keep it outside the repository or in an ignored path.
- Adding a new environment variable requires a code reference, a config-document update, a `.env.example` entry and a test. An unused placeholder is misleading and should be removed.

## Audit boundary

The local governance ledger records *that* a governed action happened, by the validated local session username for web mutations (or the fixed `search-cli` label for the CLI search audit log), with a hash chain. It does not record credentials or document contents. The local session is **not** an enterprise identity. The ledger is tamper-evident against accidental edits, not WORM.

## Violation response

If a secret is accidentally committed:

1. Treat it as exposed and rotate/revoke it immediately.
2. Preserve evidence and record the incident in [incident-log.md](incident-log.md) without repeating the secret value.
3. Remove it from history only with coordination; a force-push is a destructive action and may not be possible in a local-only clone.
4. Add or improve a check if the same exposure could recur.
5. Do not claim the incident is resolved until the secret is rotated and the evidence is recorded.

## Unsupported claims

Do not claim “secret scanning enforced”, “encrypted at rest”, “DLP”, “tenant-isolated data” or “WORM audit” on the basis of this repository. Those controls are not implemented. See [security hardening](../deployment/security.md) and the [threat model](../architecture/threat-model.md).

## Update triggers

Update this policy when a secret-bearing configuration, scan, identity control, ledger design or data-classification rule changes. Keep it aligned with `.gitignore`, `.env.example` and the code that reads environment variables.
