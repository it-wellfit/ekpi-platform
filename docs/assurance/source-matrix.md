# Source and standards matrix

- **Status:** Active research record; not a certification or compliance mapping
- **Owner:** Engineering + QA; security/data/legal approvers `UNASSIGNED`
- **Last reviewed:** 2026-10-27
- **Scope:** The local Rust prototype (`eval/`, `platform/`), the CLI, the loopback governance web app, the local backup procedure and the planned Phase B controls

This matrix records the official sources consulted, the version/date observed, the part that applies to this stack, the engineering decision it supports, and the evidence or gap. It deliberately does not copy long passages from the sources. A source mapping is not an audit, a certification or a claim of conformity.

## Primary sources

| ID | Source and version | Published / observed | Applicability and decision | Implementation / evidence | Gap or limitation |
| --- | --- | --- | --- | --- | --- |
| SRC-01 | NIST SP 800-218, Secure Software Development Framework (SSDF) Version 1.1 | Final February 2022 (document history: 2022-02-03); DOI 10.6028/NIST.SP.800-218 | Organize secure development: prepare the organization, protect the software, produce well-secured software, respond to vulnerabilities. Supports pinned toolchain/lockfiles, code review, tests, secret scanning, SBOM, dependency audit, vulnerability response. | Toolchain `1.99.1` pinned in `rust-toolchain.toml`; `Cargo.lock`; `cargo fmt`/Clippy `-D warnings`; unit/integration tests; `scripts/check-secrets.ps1`; `cargo audit`/`cargo deny`; CycloneDX SBOM; release evidence. | No CI execution, no signed commits/artifacts, no provenance attestation, no independent review, no formal vulnerability-response SLA or security owner. |
| SRC-02 | OWASP Application Security Verification Standard (ASVS) 5.0.0 | Published 2025-05-30; stable release | Web application security requirements: authentication, session management, access control, input validation, encoding, cryptography, logging, configuration. Used as a checklist lens for the loopback workbench. | Local session with PBKDF2-HMAC-SHA256, CSPRNG tokens, `HttpOnly`/`SameSite=Strict`, Viewer/Editor/Approver checks, same-origin guard, CSP, 64 KiB body cap, HTML escaping, secret scan, negative auth tests. | No full ASVS assessment; no IdP/MFA/tenant isolation; no independent penetration test. Do not claim ASVS compliance. |
| SRC-03 | OWASP Top 10:2025 | 2025 edition; page accessed 2026-10-27 | Risk lens: A01 Broken Access Control, A02 Security Misconfiguration, A03 Software Supply Chain Failures, A04 Cryptographic Failures, A05 Injection, A06 Insecure Design, A07 Authentication Failures, A08 Software or Data Integrity Failures, A09 Security Logging and Alerting Failures, A10 Mishandling of Exceptional Conditions. | A01 local role checks and denial tests; A02 loopback-only, CSP, no JS; A03 lockfiles/audit/SBOM; A04 PBKDF2 and OS CSPRNG; A05 escaped HTML and typed parsing; A06 fail-closed contracts; A07 local login/lockout; A08 hash-chained ledger (tamper-evident, not WORM); A09 JSON access log and `/metrics`; A10 explicit exit codes and fail-closed input handling. | A01/A07 remain local-only; A08 has no signature/external anchor; A09 has no alerting; A03 CI has not run. |
| SRC-04 | NIST SP 800-63B-4, Digital Identity Guidelines: Authentication and Authenticator Management | Final July 2025 (2025-07-31); supersedes SP 800-63B (2020-03-02) | Authentication assurance, authenticator management, password guidance, rate limiting/lockout, session and recovery expectations. Used to scope the local login and to define what is missing for enterprise identity. | PBKDF2-HMAC-SHA256 600,000 iterations, random salt, constant-time compare, 32-byte CSPRNG token, five-failure/300-second lockout, expiry and logout; no default account. | No MFA, account recovery, identity proofing, federation, persistent identity or access review. The local login is explicitly not an enterprise authenticator. |
| SRC-05 | NIST SP 800-207, Zero Trust Architecture | Published 2020-08-10; DOI 10.6028/NIST.SP.800-207 | Zero-trust principles: no implicit trust from network location; authenticate and authorize before resource access; policy enforcement per request. Used to describe why loopback + local session is not a zero-trust enterprise architecture. | Current boundary is loopback-only and host-trust based; local session is a usability barrier. Target design calls for an IdP, per-request policy decision, tenant claim and deny-by-default authorization. | No policy engine, device identity, continuous verification, micro-segmentation or enterprise resource boundary. A5/B1 remain partial/blocked. |
| SRC-06 | NIST AI Risk Management Framework (AI RMF 1.0) and Generative AI Profile (NIST AI 600-1) | AI RMF released 2023-01-26; GenAI Profile published 2024-07-26; DOI 10.6028/NIST.AI.600-1 | Govern, map, measure and manage AI risk; generative-AI-specific risks including confabulation, data privacy, information integrity, security and human oversight. Used to define the target AI evaluation and to refuse an AI quality claim without a provider. | Fixed deterministic corpus and 28-case retrieval gate; eval policy names six dimensions and rerun triggers; mock output labeled; no external model or real data. | AI is not implemented. No provider/data policy, model card, human oversight, red-team, cost/latency evidence or production monitoring. |
| SRC-07 | OWASP GenAI LLM Top 10:2026 | Page date 2026-08-03; accessed 2026-10-27 | LLM application risks, including prompt injection, sensitive information disclosure, supply chain, data/model poisoning, improper output handling, excessive agency, system prompt leakage and unbounded consumption. Used to define future eval and tool-permission cases; not applied to a live LLM. | Deterministic prompt-injection/adversarial cases; retrieval returns quoted text and does not execute instructions; mock provider has no tools; citation and refusal tests. | No LLM/agent/tool path, no real prompt-injection exercise, no model supply-chain or cost controls. B5 `BLOCKED`. |
| SRC-08 | Google SRE Workbook | 2018 edition (O'Reilly; CC BY-NC-ND 4.0); online edition accessed 2026-10-27 | SLOs, error budgets, monitoring, alerting on SLOs, on-call, incident response, postmortems, load management, canarying and configuration. Used to define B2/B3 target evidence. | Liveness `/healthz`, aggregate `/metrics`, JSON access log, correlation id, request timeout and concurrency bound exist; one-host benchmark recorded. | No SLI/SLO, error budget, alerting, on-call, incident drill, canary, soak or capacity model. B2/B3 `PARTIAL`. |
| SRC-09 | OpenTelemetry documentation and specifications | Docs accessed 2026-10-27; OpenTelemetry Specification 1.62.0; OTLP 1.11.1; semantic conventions 1.44.0 | Vendor-neutral instrumentation for traces, metrics and logs; context propagation and OTLP export. Used as the target observability interface, not as a current implementation. | Current implementation is local JSON access logging and aggregate counters only; no OTel SDK, exporter, span or trace context. | No collector, exporter, trace backend, sampling or alerting. A future adapter must not log secrets or high-cardinality identifiers. |
| SRC-10 | GitHub Docs: About protected branches | Current docs; accessed 2026-10-27 | Branch protection: require pull-request reviews, required status checks, conversation resolution, signed commits, linear history, deployment success, no bypass, restrict pushes, block force-push/deletion. Used to define B4 evidence. | `.github/workflows/ci.yml` exists with validate/security/workflow-lint/SBOM jobs; local `gate.ps1` and `scripts/check` exist. | No remote, no CI run, no branch protection, no required checks, no PR review. Actions are tag-pinned, not SHA-pinned. B4 `BLOCKED`. |

## Supporting sources

| ID | Source and version | Published / observed | Use | Limitation |
| --- | --- | --- | --- | --- |
| SUP-01 | RustSec Advisory Database; `cargo-audit` 0.22.2 | Database loaded 1,378 advisories on 2026-10-27 | Dependency vulnerability scan of both `Cargo.lock` files | A clean scan means no known advisory matched at that time; it is not a proof of absence. |
| SUP-02 | `cargo-deny` 0.20.2 with repository `deny.toml` | Run 2026-10-27 | Advisory, ban, license and source policy | License allow-list warnings are not failures; the policy is local, not an organization approval. |
| SUP-03 | CycloneDX specification 1.7 | Released 2025-10-21; ECMA-424 published 2025-12-10 | SBOM format for both crates | SBOM is generated and hash-listed, not independently attested or signed. |
| SUP-04 | age v1.3.2 (age-encryption.org/v1) | Host `age --version` = v1.3.2; accessed 2026-10-27 | Local-simulated encrypted backup drill | The drill destination is `%TEMP%`; it is not an off-host copy and has no key-custody process. |

## Decisions derived from the sources

| Decision | Source basis | Implementation state |
| --- | --- | --- |
| Keep the canonical check as the fixed entry point, even when blocked | SSDF (consistent, repeatable verification); repository quality gate | `scripts/check.ps1` remains canonical; blocked result recorded, not replaced by release-only |
| Use PBKDF2-HMAC-SHA256 with a high iteration count and CSPRNG salt/token for the local login | NIST SP 800-63B-4; OWASP ASVS | Implemented and unit-tested; still local-only |
| Reject foreign-origin mutations and cap request bodies | OWASP ASVS/Top 10; SSDF | Implemented and route-tested |
| Keep the ledger append-only and hash-chained but do not call it WORM | OWASP Top 10 A08; SSDF integrity | Implemented; threat model states the boundary |
| Treat retrieved document text as untrusted and never execute it | OWASP GenAI LLM Top 10; NIST AI RMF | Deterministic adversarial tests; no LLM path |
| Require a fixed eval set and rerun on model/prompt/data/code change | NIST AI RMF GenAI Profile; OWASP GenAI LLM Top 10 | Eval policy and 28-case lexical gate; no model yet |
| Do not claim AI quality from a mock provider | NIST AI RMF measure/manage; source-matrix rule | `mock-answer` label and contract tests |
| Use structured logs, metrics and correlation IDs, then add SLOs and traces | Google SRE Workbook; OpenTelemetry | Local logs/metrics/correlation exist; SLO/traces absent |
| Require PR review, required checks and branch protection before production | GitHub protected branches; SSDF | Workflow file exists; remote/CI/protection absent |
| Use an off-host encrypted backup with separate key custody and tested restore | NIST SSDF (protect/respond), SRE recovery practice, age format | Local bundle + local-simulated age drill only |

## What these sources do not establish

- No NIST, OWASP, ISO, SOC 2, ASVS or other certification or attestation has been performed.
- No source mapping substitutes for a threat-model review, penetration test or legal/privacy assessment.
- A framework's existence does not prove this repository implements it. The evidence column is the only implementation claim.
- The sources are public references; their commands, code and downloads were not executed blindly. Only the tools already used by the repository were run, with the versions recorded above.

## Review triggers

Review this matrix when a source version changes, a new deployment/identity/AI/backup control is selected, a control is implemented, or an external assessment is completed. Record the reviewer and date; do not silently change a source version without re-evaluating the affected rows.
