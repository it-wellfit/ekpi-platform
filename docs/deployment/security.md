# Security hardening and current boundary

- **Status:** Current for the local prototype; **not** a production security control set
- **Owner:** Engineering + Security (security owner not yet assigned)
- **Last reviewed:** 2026-10-26
- **Threat model:** [Architecture threat model](../architecture/threat-model.md)
- **Target controls:** [Security integration spec](../governance/07-security-integration-spec-v1.md)
- **Readiness:** [Production readiness](../operations/production-readiness.md)

## Implemented controls (current)

- `#![forbid(unsafe_code)]` in Rust code.
- Clippy with `-D warnings` (zero-warning policy) and formatting checks.
- CSS-first, server-rendered web UI with **zero JavaScript**.
- `governance_web` binds to `127.0.0.1:3000` only; it is not designed for network exposure.
- Same-origin guard on mutating HTTP methods; foreign-origin POSTs return `403` and append no event.
- Response headers: restrictive CSP, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `X-Frame-Options: DENY`.
- Local governance ledger: append-only JSONL, one event per line, with a contiguous sequence, previous hash and SHA-256 chain; the audit page verifies continuity.
- Deterministic engine decisions and snapshot replay; draft rules are rejected; schema validation is fail-closed.
- Synthetic adversarial cases cover embedded prompt-injection directives in document text; the deterministic engine treats source text as data.
- Secret-handling rules and `.gitignore` patterns for local environment/key files; CI defines a secret-scan step.

## Controls that are **not** present

Do **not** claim any of the following for the current repository:

| Claim | Actual state |
| --- | --- |
| Enterprise authentication / SSO / MFA | Not implemented. `local_auth` is a library-only prototype and is not wired into the served app. |
| Authorization / role enforcement | Not implemented in the served app. Anyone who can reach loopback can use the forms. |
| Tenant isolation | Not implemented. `tenant` is library-only and not wired. |
| WORM / immutable audit | Not implemented. The local JSONL chain is tamper-evident against accidental edits, not against a filesystem administrator; no external anchor or retention control. |
| Prompt-injection proof for an LLM/agent | No LLM/agent path is served. Deterministic cases pass; that is not an AI security proof. |
| Encryption at rest / in transit | Not implemented for the local prototype. No TLS termination exists. |
| Penetration test / ASVS / certification | Not performed. |
| Production monitoring / incident response | Not implemented. See [monitoring](monitoring.md) and [incident log](../operations/incident-log.md). |
| CSRF token | Not implemented. The same-origin guard is the only mutation check; a request without an `Origin` header is currently allowed. |

## Secret handling

- No secrets, credentials or connection strings in the repository.
- `.env` and key files are local-only and must be ignored by Git. `.env.example` contains placeholders only.
- The only environment variable read by the current code is `EKP_GOVERNANCE_DATA` (local ledger path). Other placeholder names in `.env.example` are not read; do not assume they configure the app.
- Never place real customer, personal or regulated data in the local ledger, reports or test fixtures.
- If a secret is exposed, follow [secrets policy](../operations/secrets-policy.md).

## Supported deployment boundary

Local, single-user, loopback-only execution with synthetic or authorized non-sensitive data. There is no approved production deployment target. Exposing the web server to a LAN, shared host, container ingress or the internet is **unsupported** and requires the Phase B controls in [production readiness](../operations/production-readiness.md).

## Verification

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
powershell -ExecutionPolicy Bypass -File gate.ps1 -IncludeSmoke
```

The route tests verify the same-origin guard, CSP, zero-script rendering and ledger-chain behavior. They do not replace an independent security review or penetration test.

## Update triggers

Update this page when a control is implemented or removed, the bind/network boundary changes, authentication or tenancy is wired, or a security assessment is completed. Update the [threat model](../architecture/threat-model.md) in the same change.
