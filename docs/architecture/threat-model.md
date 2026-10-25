# Threat model: local prototype

- **Status:** Active for the current local prototype; **not** a production threat model
- **Owner:** Engineering + Security
- **Last reviewed:** 2026-10-26
- **Target-product controls:** [governance/07](../governance/07-security-integration-spec-v1.md)
- **Readiness gate:** [production readiness](../operations/production-readiness.md)

## Scope and assumptions

This model covers the two served surfaces in this repository: the `ekp_platform` CLI and the local `governance_web` server on `127.0.0.1:3000`. It assumes:

- The host is a single-user local machine under the operator's control.
- Input data is synthetic or authorized non-sensitive data; no real customer, personal or regulated data is used.
- There is no network exposure, no enterprise identity provider, no tenant boundary and no external connector.
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

1. **Operator ↔ local process:** the operator is trusted to start the process and choose the data path. There is no login, so any local user who can run the binary can act as the operator.
2. **Browser ↔ loopback HTTP:** the browser is the user interface. The server trusts the loopback origin but enforces a same-origin guard on mutations and a restrictive CSP.
3. **Process ↔ filesystem:** the CLI reads and writes operator-chosen paths. The ledger writes to a local path. Path and permission handling depend on the host.
4. **Repository ↔ external supply chain:** Cargo dependencies, GitHub Actions and agent skills are external inputs. They are reviewed by version pinning and CI, not by a formal SBOM process yet.
5. **Data ↔ external services:** none in the served path. Source references are manual; the platform does not call Microsoft, Google, an LLM or PM4Py.
6. **Search corpus ↔ search caller:** the retrieval filter is a library API. `--search` runs as the local operator with no identity, so it cannot enforce document permissions; the fixed `--eval` set is the only place roles are exercised.

## Threats and controls

| ID | Threat | Current control | Residual risk | Target control |
| --- | --- | --- | --- | --- |
| TH-01 | Prompt injection in document text changes a verdict or triggers a tool | Engine treats source text as data; injection cases assert that embedded directives do not become system rules; deterministic engine decides | A future LLM or agent path could reintroduce the risk; not implemented today | Governance/07 AI controls; fixed adversarial eval set; human review; no autonomous publish |
| TH-02 | Local ledger is edited, truncated or reordered | JSONL append-only writes with `sync_data`; each event has sequence, previous hash and SHA-256; audit view verifies continuity | A user with filesystem access can rewrite the whole chain; no external anchor or WORM | Enterprise audit store, retention, external anchoring, access control (`DEC-14`) |
| TH-03 | Cross-site form submission mutates local data | `same_origin_guard` rejects mutations whose `Origin` does not match `Host`; CSP restricts sources; no JavaScript | A missing `Origin` header is not rejected by the current guard; local-only scope limits exposure | Require an approved origin/CSRF token; authenticated sessions; penetration test |
| TH-04 | Unauthorized local user reads or changes data | Loopback bind; host account permissions | No application authentication or role enforcement; `local_auth` is not wired | Enterprise IdP/MFA, RBAC, access reviews (`DEC-12`) |
| TH-05 | Tenant data leaks across records | None in the served app; `tenant` is library-only | No tenant isolation exists | Tenant-scoped storage and authorization tests; `DEC-12` |
| TH-06 | Secret or sensitive data is committed | `.env` and key patterns are ignored; CI secret scan; docs prohibit real data; `.env.example` contains placeholders only | Local ledger and reports can still contain operator-entered sensitive text; no DLP | Secret manager, data classification, redaction, retention (`DEC-13`) |
| TH-07 | Backup is missing or cannot be restored | `scripts/backup.ps1` / `backup.sh` create a git bundle and run a restore drill; one local drill passed at `283f83a` and is logged; the later `198d5f3` bundle reached a fresh clone but its full check was blocked by host Application Control | The `198d5f3` restore is not verified; the bundle excludes uncommitted files and the local ledger | Scheduled encrypted backup, off-host copy, RPO/RTO (`DEC-14`); re-run the blocked drill on an allowed host |
| TH-08 | Malformed or tampered rule/config data changes behavior | Schema validation, fail-closed parameter invariants, draft-rule rejection, tests | No signature or provenance verification of the data files | Signed rule artifacts, controlled publication, provenance |
| TH-09 | Supply-chain compromise in dependencies, CI or skills | `Cargo.lock`, Dependabot, CI audit/secret scan, pinned toolchain, skill provenance review; local `cargo audit` PASS on both lockfiles 2026-10-26 | No SBOM; actions are tag-pinned, not SHA-pinned; CI has not executed | Pin actions by SHA, mandatory audit, SBOM, dependency policy |
| TH-10 | Placeholder API is exposed and returns a false `comply` | `api.rs` is not imported by either binary; docs label it a placeholder | A future wiring mistake could expose it | Delete or implement with real engine-backed contract tests before any exposure |
| TH-11 | Decision artifacts are altered after the fact | Deterministic snapshots and report derivation rules | Files are not signed; no immutable storage | Signed snapshots, hash registry, retention (`DEC-14`) |
| TH-12 | Data loss on the single local machine | Git history, bundle procedure, restore drill | No automatic off-host backup or monitoring | Off-host encrypted backup, restore cadence, alerting |
| TH-13 | Search returns a restricted or wrong document, or fabricates an answer | Retrieval returns verbatim citations with source anchors; weak scores abstain; the fixed set covers wrong-source, adversarial and insufficient-context cases; the eval gate fails on any case | `--search` has no user identity and applies no permission filter; lexical matching can miss a correct source or surface a plausible wrong one; no semantic or LLM evaluation | Authenticated retrieval, enforced authorization, expanded and independently reviewed eval set, retrieval provenance (`DEC-12`, `DEC-17`) |
| TH-14 | Document text injects instructions through search output | Search returns text as quoted document content and never executes it; `injection_directive` is detected and not treated as a system rule; adversarial cases assert refusal or citation of the quoted section | A future LLM summarizer could follow the quoted instruction | Treat retrieved text as untrusted data, isolate tool/model context, adversarial evaluation before any LLM path |

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
- “Authenticated” — the web app has no login; `local_auth` is library-only.
- “Prompt-injection proof” — deterministic cases pass, but no LLM/agent path exists or has been evaluated.
- “Permission-aware search” as a user-facing authorization claim — only the library/eval path accepts a role list; `--search` has no identity and `governance_web` does not call retrieval.
- “Semantic search” — retrieval is lexical bigram cosine and abstains on paraphrases it cannot match.
- “Penetration tested”, “ASVS compliant” or “certified” — no such assessment has been performed.

## Review triggers

Revise this model before any of the following: network exposure, authentication/authorization, tenant support, external connectors, LLM/agent use, real customer data, new persistence, new deployment target, or a material change to the ledger or backup design. Record the review date, reviewer and resulting ADR or decision.
