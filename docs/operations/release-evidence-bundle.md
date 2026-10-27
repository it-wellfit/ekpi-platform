# EKPI MVP release evidence bundle

> Evidence pack for the Phase A (0 → 1 MVP) candidate. Every claim names the
> command, revision, date, environment and result. `PASS` means the exact
> command ran on the named revision and exited 0. It does **not** prove
> production readiness, enterprise authentication, tenant isolation, AI quality
> or compliance certification.

- **Status:** Local verification record; Phase A **not approved**, Phase B **blocked**
- **Owner:** QA + Engineering
- **Last reviewed:** 2026-10-27
- **Verification revision:** `b2bae3e` (code, scripts and benchmark artifact)
- **Environment:** One managed Windows host, Windows PowerShell 5.1, Rust stable from `rust-toolchain.toml`; local loopback only
- **Repository state:** `main`, no remote, no CI execution
- **Phase gates:** [phase-gates.md](../quality/phase-gates.md)
- **Production readiness:** [production-readiness.md](production-readiness.md)

## Release identity

- Scope: both independent crates (`eval/`, `platform/`); no root workspace.
- Served surfaces: `ekp_platform` CLI and `governance_web` on `127.0.0.1:3000`.
- Not served: `platform/src/api.rs` (fixed `comply` placeholder) and every library-only module.
- Data: synthetic or authorized non-sensitive fixtures only; no real customer, personal or regulated data.
- Distribution: Git bundle and local clone. There is no package registry publication, container image or deployment target.

## Verification results

Executed at `b2bae3e` on 2026-10-27 unless the row states another revision.

| Check | Command | Working directory | Result | Evidence |
|---|---|---|---|---|
| Format | `cargo fmt -- --check` | `eval/` | PASS | `b2bae3e`; also in `gate.ps1` |
| Format | `cargo fmt -- --check` | `platform/` | PASS | `b2bae3e`; also in `gate.ps1` |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` | `eval/` | PASS, zero warnings | `b2bae3e` |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` | `platform/` | PASS, zero warnings | `b2bae3e` |
| Tests | `cargo test --locked` | `eval/` | PASS, 12 tests | `b2bae3e` |
| Tests | `cargo test --locked` | `platform/` | PASS, 261 tests | `b2bae3e` |
| Selftest | `cargo run --locked -- --selftest` | `eval/` | PASS, 10 checks | `b2bae3e` |
| Selftest | `cargo run --locked --bin ekp_platform -- --selftest` | `platform/` | PASS, 49 assertions | `b2bae3e` |
| Declared-rules schema | `cargo run --locked --bin ekp_platform -- --declared-rules-schema` | `platform/` | PASS, 20 records | `b2bae3e` |
| Search evaluation | `cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json` | `platform/` | PASS, 28 cases, 20/20 top-1, 8/8 refusals, 4 role-restricted (2 allowed, 2 denied), 0 violations, isolation 1.00 | `b2bae3e` |
| Full gate | `gate.ps1 -IncludeSmoke` | Repo root | **13/13 PASS** | `b2bae3e` |
| Documentation | `scripts/check-docs.ps1` | Repo root | PASS, 44 required, 494 links in 106 files (includes 10 local untracked legacy pages; a fresh clone checks 96 tracked files) | `b2bae3e` |
| Secret scan | `scripts/check-secrets.ps1` | Repo root | PASS, 220 files | `b2bae3e` |
| Supply chain | `scripts/check-supply-chain.ps1` | Repo root | PASS: `cargo deny` advisories/bans/licenses/sources OK; `cargo audit` platform 72 deps, eval 12 deps, 0 vulnerabilities | `b2bae3e` |

`platform` test split at `b2bae3e`: 217 unit, 13 CLI contract, 2 edge-case, 10 governance web, 8 integration, 7 search evaluation, 4 web health. The `governance_web` binary has `test = false` in `platform/Cargo.toml` because the host's Application Control intermittently blocks freshly built test harnesses; route coverage is provided by `platform/tests/governance_web.rs` and `platform/tests/web_health.rs`.

### Smoke and export details

- `G_ekpi_engine acc=1.000`, wrong-allow 0, violation-as-comply 0; hard gates H-1–H-6 all OK.
- LegalRuleML 51,026 bytes, 19 PrescriptiveStatement, 19 PenaltyStatement, 19 Reparation, 16 LegalSource.
- `report.html` contains zero script elements.
- Draft rule `RULE-CHG-001-DRAFT@3.1` is rejected at load and invisible to the engine.

### Performance observation

`scripts/benchmark.ps1 -Profile release` ran at `56a944f`; the artifact was committed at `b2bae3e` as `artifacts/benchmark/benchmark-2026-10-27-124954.json`.

- Evaluation p50/p95: **17.432 / 23.319 ms**.
- Loopback `/healthz` sequential p50/p95: **19.010 / 24.069 ms**.
- Concurrent loopback: **5,369.21 req/s**.

This is a one-host regression observation, not a capacity model, soak test or SLO. No approved workload exists (`DEC-19`).

### Supply chain artifacts

- CycloneDX SBOMs regenerated at `a826472`: `ekp_platform` 67 components, SHA-256 `5fdc80974503dcddd28ad8a73fb44877a5fc36144424f9acdbbf3699379f9478`; `ekp_eval` 11 components, SHA-256 `3a486a469b8b1976f18dd45d35218128881afe6134d057d4cb7f74015a405c04`.
- `artifacts/sbom/SHA256SUMS` records both hashes. No dependency changed between `a826472` and `b2bae3e`.
- The documents contain only the repository-relative `file://.` locator, not the local absolute path or user name.
- Actions remain tag-pinned, not SHA-pinned; the CI security job has never run because there is no remote.

## Local authentication boundary (A-06)

`local_auth` is wired into `governance_web` as a **local session** mechanism:

- One account is registered at startup from `EKPI_LOCAL_AUTH_USER` and `EKPI_LOCAL_AUTH_PASSWORD`; `EKPI_LOCAL_AUTH_ROLE` selects `viewer`, `editor`, `approver` or `admin` (default `editor`). There is no default account.
- Passwords use PBKDF2-HMAC-SHA256 with a random 16-byte salt and 600,000 iterations. Session tokens are 32 OS-CSPRNG bytes. Comparison is constant-time.
- Five failed logins lock the account for 300 seconds. Failure state is tracked only for accounts that exist, so arbitrary usernames cannot grow the map without bound.
- Sessions live in process memory, expire after 3,600 seconds and are pruned on login; they disappear on restart.
- `ekpi_session` is `HttpOnly`, `SameSite=Strict` and `Path=/`; `EKPI_COOKIE_SECURE=1` adds `Secure`.
- Every ledger-backed GET requires a Viewer session. Creation requires Editor. Review, publication, quality-status and lifecycle decisions require Approver.
- Public paths: `/healthz`, `/metrics`, `/login`, `/assets/*`. `/logout` requires a valid session.
- Foreign-origin mutations are refused; a missing `Origin` is accepted only with `Sec-Fetch-Site: same-origin` or `none`. Form bodies are limited to 64 KiB.

This is **not** an IdP, SSO, MFA, account recovery, persistent identity store, access review, tenant boundary or delegated administration. The ledger actor remains the operator-entered string, not the session identity. A local administrator can read or rewrite the ledger and process memory. `A5` stays `Partial`; `DEC-12` remains open.

## Annotation κ (A-05)

`eval/src/annotation.rs` computes Cohen's κ for two annotators' independent verdict labels:

- `cohens_kappa(rater1, rater2) -> Option<f64>` returns `None` for different-length inputs, empty input, or a zero denominator.
- `kappa_interpretation(kappa) -> &'static str` uses the Landis & Koch bands.
- 7 unit tests: perfect agreement, chance agreement, negative disagreement, mismatched lengths, empty input, single-label all-agree (undefined, `None`) and moderate agreement.

Single-class agreement is **not** reported as κ = 1.0 because the chance-agreement denominator is zero. No second human annotator has produced labels, so `A11` remains `Partial`.

## Backup and restore (A-08, local only)

| Revision | Bundle | SHA-256 | Result |
|---|---|---|---|
| `7fd0c0b` | `ekpi-platform-2026-10-27-124512.bundle` | `EBFD373C45EC7A556C6B7C59483F16CE42FA77C4E9C35A85BD50C5B8FF34A2FF` | Fresh clone reached `7fd0c0b`; restored `scripts/check.ps1` PASS (eval 12, platform 261) |
| `56a944f` | `ekpi-platform-2026-10-27-124757.bundle` | `23EA44011711566CC7147767E6FF4258F75030CFFA0507EC98E00323AACE6421` | Fresh clone reached `56a944f`; restored `scripts/check.ps1` PASS; `age` drill encrypted and decrypted byte-identically; `git bundle verify` okay |

The `56a944f` off-host drill receipt records ciphertext SHA-256 `0DBA33BCC90B94749DAF7547AF524364C5F4567358953E01A3D61260284AC181`, `decryption_verified: true` and `destination_class: local-simulated-off-host`. A drill under `%TEMP%` is **not** an off-host copy.

These drills prove that committed history can be restored and checked locally. They do not prove an off-host backup, an approved RPO/RTO, a scheduled restore programme, encryption-key custody or disaster recovery. Uncommitted work, the external governance ledger and ignored secrets are outside the bundle.

## Traceability

- Register: [traceability/register.md](../traceability/register.md). Rows name the specification, code, contract, test evidence and release revision.
- S1–S21, A-05 and A-06 are recorded; A-06 is `Implemented` with local session evidence, not enterprise identity.
- `Verified` means a test proves conformance. No row is `Validated`: no representative user or named pilot owner has confirmed fitness for real use.
- Phase A remains `Not approved` because A1 (real user value) is `Not met` and the pilot-scope rows are `Partial`.
- Phase B remains `Blocked` on an approved deployment target, enterprise identity, data policy, tenant model, off-host backup and operational ownership.

## Known limitations

1. **No real pilot** — no named product owner, operator, reviewer or user session. Synthetic tests do not validate user value (A1 `Not met`).
2. **No remote or CI** — `gate.ps1` ran locally; branch protection, pull requests, issue labels and the CI security job were never exercised.
3. **No enterprise identity** — local process-memory login only; no IdP, SSO, MFA, recovery, persistent identity, access review or tenant isolation (`DEC-12`).
4. **No off-host backup** — local bundle and encrypted drill only; no off-host destination, approved RPO/RTO or key-custody owner (`DEC-14`).
5. **No production observability** — local JSON access log, correlation id, aggregate `/metrics` and liveness `/healthz`; no SLO, traces, alerting, on-call or readiness probe (`DEC-16`).
6. **No AI/LLM capability** — retrieval is deterministic lexical; `--llm` accepts only `mock` and labels output `mock-answer`; no model quality claim (`EVAL-GAP-01`).
7. **No independent review** — the threat model, audit and this pack are self-assessments; no penetration test or certification.
8. **Host policy risk** — Windows Application Control intermittently blocks freshly built executables (`os error 4551`). A blocked run is recorded as `Blocked`, never `PASS`.

## Sign-off

This pack records what was verified on the named revisions. It is not production approval, compliance certification, enterprise authorization or evidence of real-user value. Phase A stays `Not approved`; Phase B stays `Blocked`.
