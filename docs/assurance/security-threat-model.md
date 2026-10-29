# Assurance threat model

- **Status:** Active for the local prototype; **not** a production threat model and not a security certification
- **Owner:** Engineering; security owner/approver `UNASSIGNED`
- **Last reviewed:** 2026-10-29
- **Canonical threat model:** [architecture/threat-model.md](../architecture/threat-model.md) (TH-01–TH-15)
- **Security requirements:** [governance/07](../governance/07-security-integration-spec-v1.md)
- **Readiness:** [production readiness](../operations/production-readiness.md)
- **Matrix:** [phase-a-b-matrix.md](phase-a-b-matrix.md)

This document is an assurance view of the canonical threat model. It does not create a second control set: the canonical model owns the threat IDs and controls; this page records the trust boundaries, residual risk, test coverage and the evidence that is still missing.

## Scope and assumptions

In scope:

- `ekp_platform` CLI (local process, operator-chosen paths).
- `governance_web` on `127.0.0.1:3000` with a local process-memory session.
- Local JSON/JSONL data and the governance ledger.
- Local Git bundles and the local-simulated `age` drill.
- Cargo dependencies, the pinned toolchain, GitHub Actions definitions and agent skills as supply-chain inputs.

Assumptions that invalidate this model if changed:

1. The host is a single-user managed Windows machine controlled by the operator.
2. Data is synthetic or authorized non-sensitive data; no real customer, personal or regulated data.
3. There is no network exposure beyond loopback, no enterprise IdP, no tenant boundary and no external connector.
4. The optional PM4Py sidecar and any LLM are not in the served path. `--llm` is mock-only.
5. A local administrator is trusted with the host filesystem and process memory. The model does not defend against a malicious administrator.

If any assumption changes, stop and revise this model before deployment. Exposing the web server beyond loopback is not supported.

## Assets

| Asset | Why it matters | Current protection |
| --- | --- | --- |
| Source references and versions | A wrong version can produce a wrong conclusion | Version binding, tests, schema, review |
| Active rules and schema | Draft/tampered rules can change decisions | Draft rejection, schema validation, fail-closed invariants |
| Process evidence and case data | Forbidden transitions/evidence gaps must not be released | Replay tests, hard gates |
| Retrieval corpus and eval set | Search must cite the right source and abstain when evidence is weak | Fixed set, citation checks, threshold, refusal cases |
| Decision snapshots and reports | Audit/reproduction record | Deterministic generation; not signed |
| Governance ledger | Local record of governed actions | Append-only JSONL hash chain with a single-host OS writer lock; not WORM |
| Local credentials/environment | Leakage compromises local access | Env-only password, PBKDF2 hash, no default account |
| Build/dependency integrity | Supply-chain compromise can alter behavior | Lockfiles, pinned toolchain, audit, SBOM |
| Backups | Recovery after loss | Local bundle + restore drill; no off-host copy |

## Trust boundaries

| Boundary | Trust assumption | Control | Residual risk |
| --- | --- | --- | --- |
| Operator ↔ local process | Operator is trusted to start the process and choose data paths | Host account permissions; local session for data pages | A local administrator can read/rewrite the ledger and process memory |
| Browser ↔ loopback HTTP | Browser is the UI; no public network | Loopback bind, single loopback `Host` header (duplicate/blank/non-loopback refused), session cookie, duplicate or malformed cookies unauthenticated, same-origin/fetch-metadata guard, CSP, `HttpOnly`/`SameSite=Strict`, body cap | No enterprise CSRF token, no TLS, no WAF; local malware can act as the user |
| Process ↔ filesystem | Operator-chosen paths; local ledger path | Path handling by the operator; append + `sync_data` | A filesystem administrator can rewrite/truncate the ledger; no external anchor |
| Repository ↔ supply chain | Dependencies/actions/skills are external | Lockfiles, pinned toolchain, `cargo audit`/`cargo deny`, secret scan, SBOM | Actions are tag-pinned, CI has not run, SBOM is not signed/attested |
| Data ↔ external services | None in the served path | No connector, no LLM, no PM4Py | A future provider integration would create a new boundary requiring policy |
| Search corpus ↔ caller | `--search` runs as the local operator; no identity | Library `PermissionContext` filter, synthetic eval cases | `--search` has no authenticated identity; no tenant enforcement |
| Credential boundary | One env-provided account | PBKDF2-HMAC-SHA256 600k, CSPRNG salt/token, constant-time compare, five-failure lockout; one generic failure message, lockout state not disclosed, invalid usernames pay a dummy hash and do not grow lockout state | No MFA, recovery, persistent identity, access review; env/host account is the effective store |
| Backup boundary | Local disk and local clone | Git bundle + fresh-clone drill; local `age` drill | No off-host copy, no key custody, no RPO/RTO, no immutable storage |

## Threat analysis

The canonical IDs are retained. The assurance column records the current evidence and the residual risk that matters for Phase A/B.

| Canonical ID | Threat | Current control | Assurance test/evidence | Residual risk |
| --- | --- | --- | --- | --- |
| TH-01 | Prompt injection changes a verdict or triggers a tool | Engine treats source text as data; deterministic engine decides; injection cases assert no rule change | Adversarial cases in the 67-case corpus; mock provider has no tools | A future LLM/agent path could reintroduce the risk; not implemented today |
| TH-02 | Ledger edited, truncated or reordered | Append-only JSONL, sequence, previous hash, SHA-256, `sync_data`, single-host OS writer lock, replay reference/transition validation; search audit v2 uses a domain separator + length-prefixed fields, UTC timestamp and unique event ID, with rollback on a failed append | `audit` 19 unit tests (tamper, legacy, truncated, duplicate ID, rollback, concurrent appenders); ledger route tests; `local_lock` tests | A filesystem administrator can rewrite the whole chain; no WORM or external anchor |
| TH-03 | Cross-site form submission mutates data | Same-origin guard; a single loopback `Host` is required; missing `Origin` only with `Sec-Fetch-Site: same-origin`/`none`; `SameSite=Strict`; CSP; no JS | 15 governance-web route tests + 5 parser unit tests; manual review | Local CSRF hardening, not an enterprise CSRF token or identity review |
| TH-04 | Unauthorized local user reads/changes data | Loopback, host permissions, local session, Viewer/Editor/Approver checks on data routes; mutation actor taken from the validated session; duplicate/blank/non-UTF-8 Host or cookie values fail closed | 15 route tests (including an actor-impersonation regression) + 5 parser unit tests + 13 local-auth unit tests | Local process-memory identity; no IdP/MFA/recovery/access review; local admin can read memory |
| TH-05 | Tenant data leaks across records | No tenant context is constructed by the CLI/web app; `tenant` is library-only | Library `PermissionContext` tests and synthetic role cases | No tenant isolation or tenant-scoped storage; do not claim tenant isolation |
| TH-06 | Secret or sensitive data committed | `.env` ignored, secret scan, docs prohibit real data, placeholders only, no password logging | E-005/E-017 secret scan (229 files at the latest docs commit); `.env.example` review | Ledger/reports can contain sensitive text typed by an operator; no DLP; process env holds the password |
| TH-07 | Backup missing or cannot be restored | Local bundle + restore drill; `age` drill | E-007 (earlier blocked), E-008 (earlier PASS), E-009 (simulated), E-020/E-023 (`41beaea`/`6ed236d` restore + clean-clone canonical PASS), E-027 (current `499e0d1` restore PASS) | No off-host copy, approved RPO/RTO, key custody or scheduled drill; the host policy has blocked working-tree runs before and has no named exception owner |
| TH-08 | Malformed/tampered rule/config changes behavior | Schema validation, fail-closed invariants, draft rejection, tests | Unit/integration tests; schema 20 records | No signature/provenance for data files |
| TH-09 | Supply-chain compromise | Lockfiles, pinned toolchain, audit/deny, secret scan, SBOM | E-006; E-010 | Actions tag-pinned, CI never run, SBOM not signed; no provenance service |
| TH-10 | Placeholder API exposed and returns false `comply` | `api.rs` not imported by either binary; docs label it placeholder | Architecture review; no route test can expose it because it is not served | A future wiring mistake could expose it; delete or implement with engine-backed contract tests |
| TH-11 | Decision artifacts altered after the fact | Deterministic snapshots/report rules | Replay/snapshot tests | Files are not signed; no immutable storage |
| TH-12 | Data loss on the single local machine | Git history, bundle procedure, restore drill | E-007/E-008 | No automatic off-host backup or monitoring |
| TH-13 | Search returns a restricted/wrong document or fabricates an answer | Verbatim citations, weak-score abstention, fixed cases, permission cases | E-003: 20/20, 8/8, 4 role cases, 0 violations | `--search` has no identity; lexical matching can miss/paraphrase; no semantic/LLM eval |
| TH-14 | Document text injects instructions through search output | Retrieved text is quoted, not executed; injection directive detected | Adversarial cases | A future summarizer could follow quoted instructions |
| TH-15 | Offline/online guessing of the local password | PBKDF2 600k, random salt, CSPRNG token, constant-time compare, dummy hash, five-failure/300 s lockout; failures and lockout return one generic message; invalid usernames do not create lockout state | 13 local-auth unit tests (generic failure, lockout non-disclosure, dummy-hash path, username shape, expiry) | Lockout is in-process and resets on restart; no MFA, source rate limit or recovery |

## Additional assurance threats

| ID | Threat | Why it matters here | Current mitigation | Residual risk / target |
| --- | --- | --- | --- | --- |
| AST-01 | Host Application Control blocks build/test execution | Interfered with canonical acceptance and earlier restores; could tempt a team to lower the gate | Recorded as `BLOCKED`; release-profile coverage retained; clean clones at `41beaea`/`6ed236d` passed the canonical check; no gate change; the absolute target-dir override and `EKP_PLATFORM_BIN` test override were removed | Current `c827351` working-tree canonical is blocked at `edge_cases` (`os error 4551`); earlier restores remain blocked until a scoped exception or clean runner exists |
| AST-02 | A blocked check is misreported as PASS | Would invalidate the entire evidence chain | Status vocabulary; matrix separates `PASS`/`BLOCKED`/`NOT RUN`; no row claims a working-tree or CI canonical PASS | Requires reviewer discipline and automated link/claim checks |
| AST-03 | Local admin rewrites ledger and backup | The ledger is not WORM; a bundle can be replaced | Hash chain detects accidental edits; backup log records hashes | No external anchor, signature, object lock or separation of duties |
| AST-04 | Real data enters a synthetic-data workflow | Privacy/legal exposure and invalid evaluation | Docs prohibit real data; env/data path is operator-controlled | No DLP or classification enforcement; B8 blocked |
| AST-05 | AI provider leaks or retains restricted data | Future LLM path could send sensitive content externally | No provider is configured; `--llm` is mock-only | Requires provider/data policy, redaction, retention terms and tests before enabling |
| AST-06 | Backup encryption key is lost or held by one person | Encrypted backup becomes unrecoverable or a single point of compromise | `age` drill uses a throwaway key; proposal calls for separate custody | No approved key custody or recovery test; B7 blocked |
| AST-07 | CI workflow is assumed to run because the file exists | Configuration is not execution | Release evidence states CI never ran; B4 blocked | Need remote, CI run, branch protection and required checks |
| AST-08 | Self-review is presented as independent review | Shared assumptions remain unchallenged | A11 checklist prepared; status `PARTIAL` | Need a reviewer who did not implement the change |

## Risk rating under the current boundary

These ratings are meaningful only for the local, synthetic-data, single-user prototype:

| Risk | Likelihood | Impact | Rating | Basis |
| --- | --- | --- | --- | --- |
| Local admin tampering with ledger/backup | Medium | High | **High** | No WORM, no external anchor, local disk |
| Final backup unrecoverable | Medium | High | **High** | `41beaea` and `6ed236d` restore locally; the earlier `28b68cc` restore is `BLOCKED`; neither `c827351` nor `bbe49eb` is bundled; no off-host copy |
| Secret/sensitive text in ledger or report | Medium | Medium | **Medium** | Operator-entered content; no DLP |
| Password guessing on local login | Low | Medium | **Medium** | 600k PBKDF2 + lockout; no MFA |
| Supply-chain compromise | Low | High | **Medium** | Lockfiles/audit/SBOM, but CI/SHA pinning absent |
| Prompt injection changing a verdict | Low today | High | **Medium** | Deterministic engine; no LLM path |
| Cross-tenant leak | Not applicable today | High | **High if multi-tenant** | No tenant model; must be designed before any multi-tenant claim |
| Placeholder API exposure | Low | High | **Medium** | Not wired; future wiring risk |

If the service is exposed beyond loopback, all ratings become invalid and the model must be revised before exposure. A network-exposed deployment without IdP/TLS/tenant controls would be **High** or **Critical** across access-control and data-confidentiality risks.

## A5-specific conclusions

- The local session is a real control for casual local access, but it is not enterprise authentication.
- The role checks are enforced on the served routes and covered by tests, but the identity source is a single startup account, not an IdP.
- Login failures return one generic message, lockout state is not disclosed, invalid usernames pay a dummy hash and do not grow lockout state, and duplicate/malformed session cookies plus duplicate/non-loopback `Host` values fail closed. These reduce local disclosure and request-routing risk; they do not provide enterprise identity.
- The ledger is protected by a single-host OS writer lock and replay validates references and transitions. Search audit v2 uses a domain-separated, length-prefixed hash with unique IDs and rolls back a failed append. None of this makes the ledger WORM or externally anchored.
- There is no tenant boundary. The `tenant` module is library-only. Cross-tenant tests do not exist because there is no tenant-scoped storage or identity claim.
- The ledger actor is the validated local session username for web mutations, but the session is process-memory only, so the ledger is not proof of an enterprise-verified identity. Submitted form `actor`/`reviewer` fields are ignored. The CLI search audit uses the fixed `search-cli` label.
- A local administrator can read or rewrite the ledger and process memory. The hash chain is tamper-evident against accidental edits, not WORM.

## A9-specific conclusions

- The deterministic engine and retrieval gate are testable and have adversarial cases; this is not an LLM evaluation.
- `--llm` accepts only `mock`, labels output `mock-answer`, requires a versioned citation and makes no network call.
- There is no tool path, so tool-failure, timeout, partial result and retry-exhaustion behavior are untested (`EVAL-GAP-01`).
- No provider/data policy exists, so sending real data to a model is prohibited.
- A future LLM must treat retrieved text as untrusted, isolate tool context, enforce least privilege, log safely and pass a fixed six-dimension evaluation before any quality claim.

## Target controls (not implemented)

| Target | Phase item | Acceptance evidence |
| --- | --- | --- |
| Enterprise IdP + MFA + session revocation + access review | A5, B1 | OIDC/SAML integration tests, denial matrix, access-review record |
| Tenant-scoped storage and authorization | A5, B1, B8 | Cross-tenant read/write/search/export tests, zero leakage |
| Off-host encrypted backup with separate key custody | A8, B7 | Off-host retrieve, restore, RTO measurement, key-recovery test |
| SLO/alerting/traces and fault injection | B2 | Dashboard, alert drill, trace export, failure recovery record |
| Approved AI provider + fixed eval + tool policy | A9, B5 | Versioned eval report, tool-failure cases, human review |
| CI required checks + branch protection + signed/pinned supply chain | B4 | CI run, protection settings, SHA-pinned actions, SBOM attestation |
| Independent security review / penetration test | A11, B1 | Signed report, findings disposition, retest |

## Non-claims

Do not claim any of the following from this repository:

- “secure”, “hardened”, “production-ready” or “enterprise-grade”;
- “WORM”, “immutable audit” or “tamper-proof”;
- “tenant-isolated” or “permission-aware search” as a user-facing authorization guarantee;
- “enterprise-authenticated”, “SSO”, “MFA” or “access-reviewed”;
- “prompt-injection proof”, “AI-validated” or “hallucination-free”;
- “penetration tested”, “ASVS compliant”, “NIST certified” or “SOC 2”;
- “off-host backup”, “disaster-recovery tested” or “RPO/RTO met”.

## Review triggers

Revise this model before any of the following: network exposure, identity/authorization change, tenant support, external connector, LLM/agent/tool use, real customer data, new persistence, new deployment target, ledger/backup design change, or a material incident. Record the reviewer, date and resulting decision.
