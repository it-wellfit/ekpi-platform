# Threat model: local prototype

- **Status:** Active for the current local prototype; **not** a production threat model
- **Owner:** Engineering + Security
- **Last reviewed:** 2026-10-27
- **Target-product controls:** [governance/07](../governance/07-security-integration-spec-v1.md)
- **Readiness gate:** [production readiness](../operations/production-readiness.md)

## Scope and assumptions

This model covers the two served surfaces in this repository: the `ekp_platform` CLI and the local `governance_web` server on `127.0.0.1:3000`. It assumes:

- The host is a single-user local machine under the operator's control.
- Input data is synthetic or authorized non-sensitive data; no real customer, personal or regulated data is used.
- There is no network exposure, no enterprise identity provider, no tenant boundary and no external connector. The workbench has a local process-memory session login, which is not an enterprise identity control.
- The optional Python/PM4Py sidecar and any LLM are not in the served path.

If any assumption changes, this model is invalid and must be revised before deployment. In particular, exposing the web server beyond loopback is **not supported**.

## Assets

| Asset | Location | Why it matters |
| --- | --- | --- |
| Source references and versions | `platform/data/docs.json` and the ledger | A wrong version can produce a wrong conclusion |
| Active rules and schema | `platform/data/rules.json`, `design/rule-schema-v0.1.1.schema.json` | Draft or tampered rules can change decisions |
| Process evidence and case data | `platform/data/processes.json`, evaluation cases | Forbidden transitions and evidence gaps must not be released |
| Retrieval corpus and evaluation set | `platform/data/docs.json`, `platform/data/eval/search_eval.json` | Search must cite the right source, respect the filter it is given and abstain when evidence is weak |
| Decision snapshots and reports | `platform/out/`, `platform/report_out/` | They are the audit and reproduction record |
| Governance ledger | `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` or `EKP_GOVERNANCE_DATA` | It is the local record of governed actions |
| Local credentials and environment | `.env` (ignored), host environment | Leakage would compromise local or future remote access |
| Build and dependency integrity | `Cargo.lock`, CI workflows, skills | Supply-chain compromise can alter behavior |

## Trust boundaries

1. **Operator ↔ local process:** the operator is trusted to start the process and choose the data path. The workbench requires a local session for data pages, but any local user who can start or attach to the process controls the host and the ledger. The login is a usability and casual-access barrier, not an enterprise trust boundary.
2. **Browser ↔ loopback HTTP:** the browser is the user interface. The server accepts only loopback binds, requires a session cookie for data pages, enforces a same-origin/fetch-metadata guard on mutations, uses `HttpOnly`/`SameSite=Strict` cookies and a restrictive CSP.
3. **Process ↔ filesystem:** the CLI reads and writes operator-chosen paths. The ledger writes to a local path. Path and permission handling depend on the host.
4. **Repository ↔ external supply chain:** Cargo dependencies, GitHub Actions and agent skills are external inputs. They are reviewed by version pinning, lockfiles, local audit and CycloneDX SBOM artifacts, not by a hosted provenance service.
5. **Data ↔ external services:** none in the served path. Source references are manual; the platform does not call Microsoft, Google, an LLM or PM4Py.
6. **Search corpus ↔ search caller:** the retrieval filter is a library API. `--search` runs as the local operator with no identity, so it cannot enforce document permissions. The fixed `--eval` set now supplies a synthetic `PermissionContext` for four role-restricted cases and two evaluation-only fixtures with disjoint roles; it reports 2 allowed, 2 denied and 0 violations. The fixtures are merged only for `--eval` and are not in `docs.json`, so `--search` cannot see them. This proves the library filter's allow/deny behavior, not authentication or tenant authorization.
7. **Credential boundary:** the workbench reads one username/password pair from the process environment at startup and keeps only a salted PBKDF2 hash and in-memory sessions. There is no user database, recovery flow, MFA or access review; the environment and the host account are the effective credential store.

## Threats and controls

| ID | Threat | Current control | Residual risk | Target control |
| --- | --- | --- | --- | --- |
| TH-01 | Prompt injection in document text changes a verdict or triggers a tool | Engine treats source text as data; injection cases assert that embedded directives do not become system rules; deterministic engine decides | A future LLM or agent path could reintroduce the risk; not implemented today | Governance/07 AI controls; fixed adversarial eval set; human review; no autonomous publish |
| TH-02 | Local ledger is edited, truncated or reordered | JSONL append-only writes with `sync_data`; each event has sequence, previous hash and SHA-256; audit view verifies continuity | A user with filesystem access can rewrite the whole chain; no external anchor or WORM | Enterprise audit store, retention, external anchoring, access control (`DEC-14`) |
| TH-03 | Cross-site form submission mutates local data | `same_origin_guard` rejects mutations whose `Origin` does not match `Host`; when `Origin` is absent it accepts only `Sec-Fetch-Site: same-origin` or `none`; session cookie is `SameSite=Strict`; CSP restricts sources; no JavaScript | The guard and cookie are local CSRF hardening, not a substitute for an enterprise CSRF token or identity review | Require an approved origin/CSRF token; authenticated sessions; penetration test |
| TH-04 | Unauthorized local user reads or changes data | Loopback bind; host account permissions; local session login with Viewer/Editor/Approver checks on every data route and mutation | Local process-memory identity only: no IdP, MFA, recovery, persistent user store or access review. A local administrator can read the ledger and process memory | Enterprise IdP/MFA, RBAC, access reviews (`DEC-12`) |
| TH-05 | Tenant data leaks across records | No tenant context is constructed by the CLI or web app; `tenant` is referenced only by the library `PermissionContext` conversion | No tenant isolation or tenant-scoped storage exists | Tenant-scoped storage and authorization tests; `DEC-12` |
| TH-06 | Secret or sensitive data is committed | `.env` and key patterns are ignored; CI secret scan; docs prohibit real data; `.env.example` contains placeholders only; passwords are never logged or persisted by the workbench | Local ledger and reports can still contain operator-entered sensitive text; process environment contains the login password; no DLP | Secret manager, data classification, redaction, retention (`DEC-13`) |
| TH-07 | Backup is missing or cannot be restored | `scripts/backup.ps1` / `backup.sh` create a git bundle and run a restore drill; local drills passed at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f`, `3ee2ee2`, `41beaea`, `6ed236d` and **`499e0d1`** and are logged. At `499e0d1` the current evidence bundle verifies, clones to the recorded revision and the automatic restore drill exits 0 (docs 657/105, platform 277). The earlier `28b68cc`, `bc7671a`, `c77fec4` and `198d5f3` attempts remain blocked by host Application Control and are not counted. The `age` drill at `56a944f` decrypted byte-identically to a local-simulated destination | The bundle excludes uncommitted files and the local ledger; there is no off-host copy, approved RPO/RTO or scheduled restore test. The `age` off-host script is a proposal plus local drill, not an operating backup service | Scheduled encrypted backup, off-host copy, RPO/RTO (`DEC-14`) |
| TH-08 | Malformed or tampered rule/config data changes behavior | Schema validation, fail-closed parameter invariants, draft-rule rejection, tests | No signature or provenance verification of the data files | Signed rule artifacts, controlled publication, provenance |
| TH-09 | Supply-chain compromise in dependencies, CI or skills | `Cargo.lock`, Dependabot, CI audit/secret scan, pinned toolchain, skill provenance review; local `cargo audit` PASS on both lockfiles; CycloneDX SBOM artifacts are generated and hash-listed | Actions are tag-pinned, not SHA-pinned; CI has not executed; SBOM is not independently attested | Pin actions by SHA, mandatory audit, SBOM, dependency policy |
| TH-10 | Placeholder API is exposed and returns a false `comply` | `api.rs` is not imported by either binary; docs label it a placeholder | A future wiring mistake could expose it | Delete or implement with real engine-backed contract tests before any exposure |
| TH-11 | Decision artifacts are altered after the fact | Deterministic snapshots and report derivation rules | Files are not signed; no immutable storage | Signed snapshots, hash registry, retention (`DEC-14`) |
| TH-12 | Data loss on the single local machine | Git history, bundle procedure, restore drill | No automatic off-host backup or monitoring | Off-host encrypted backup, restore cadence, alerting |
| TH-13 | Search returns a restricted or wrong document, or fabricates an answer | Retrieval returns verbatim citations with source anchors; weak scores abstain; the fixed set covers wrong-source, adversarial, insufficient-context and role-restricted cases (2 allowed, 2 denied, 0 violations); the eval gate fails on any case or permission violation | `--search` has no user identity and applies no permission filter; the role cases come from a synthetic harness, not an identity provider; lexical matching can miss a correct source or surface a plausible wrong one; no semantic or LLM evaluation | Authenticated retrieval, enforced authorization with tenant isolation, an independently reviewed eval set, retrieval provenance (`DEC-12`, `DEC-17`) |
| TH-14 | Document text injects instructions through search output | Search returns text as quoted document content and never executes it; `injection_directive` is detected and not treated as a system rule; adversarial cases assert refusal or citation of the quoted section | A future LLM summarizer could follow the quoted instruction | Treat retrieved text as untrusted data, isolate tool/model context, adversarial evaluation before any LLM path |
| TH-15 | Offline or online guessing of the local workbench password | PBKDF2-HMAC-SHA256 with 600,000 iterations and a random salt; 32-byte OS-CSPRNG session tokens; constant-time hash comparison; a dummy hash for unknown users; 5 failures trigger a 300-second lockout; passwords are never logged | The lockout is in-process and resets on restart; there is no MFA, rate limit by source, password rotation or account recovery; a weak operator-chosen password is possible | Enterprise IdP/MFA, credential policy, monitored authentication (`DEC-12`) |

## Out of scope

- Multi-tenant isolation, enterprise SSO/MFA, privileged access management and access review.
- Public internet exposure, TLS termination, WAF, DDoS protection and reverse-proxy hardening.
- Live connectors, OAuth scopes, external permission verification and data-residency controls.
- LLM/model supply chain, prompt lifecycle, agent tool permissions and AI evaluation at scale.
- Regulatory certification, privacy impact assessment and legal retention obligations.

These are Phase B items. They remain blocked until the decisions and owners in [governance/08](../governance/08-decision-register.md) exist.

## Security claims that are not supported

Do not claim any of the following on the basis of this repository:

- “WORM” or “immutable audit” — the local JSONL chain is tamper-evident only against accidental edits, not against a filesystem administrator.
- “Tenant isolated” — the tenant module is not wired.
- “Authenticated” in an enterprise sense — the web app has only a local process-memory session login with one startup-configured account; it has no IdP, SSO, MFA, recovery, persistent identity or access review. Say “local session login” when describing the current control.
- “Prompt-injection proof” — deterministic cases pass, but no LLM/agent path exists or has been evaluated.
- “Permission-aware search” as a user-facing authorization claim — `PermissionContext` is a library API only; `--search` has no identity and `governance_web` does not call retrieval. The fixed eval set now exercises four synthetic role-restricted cases (2 allowed, 2 denied, 0 violations), which proves the library filter's allow/deny behavior, not authentication or tenant authorization (`EVAL-GAP-06` closed as fixed-set coverage; `S19`, `PR-03` remain open).
- “Semantic search” — retrieval is lexical bigram cosine and abstains on paraphrases it cannot match.
- “Penetration tested”, “ASVS compliant” or “certified” — no such assessment has been performed.

## Review triggers

Revise this model before any of the following: network exposure, authentication/authorization, tenant support, external connectors, LLM/agent use, real customer data, new persistence, new deployment target, or a material change to the ledger or backup design. Record the review date, reviewer and resulting ADR or decision.
