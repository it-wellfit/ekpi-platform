# EKPI MVP release evidence bundle

> Evidence pack for the Phase A (0 → 1 MVP) candidate. Every claim names the
> command, revision, date, environment and result. `PASS` means the exact
> command ran on the named revision and exited 0. It does **not** prove
> production readiness, enterprise authentication, tenant isolation, AI quality
> or compliance certification.

- **Status:** Local verification record; Phase A **not approved**, Phase B **blocked**
- **Owner:** QA + Engineering
- **Last reviewed:** 2026-10-27
- **Verification revision:** `b2bae3e` (last canonical `gate.ps1 -IncludeSmoke` PASS); `23b8996` (latest code delivery revision: release-profile suite, blocked canonical check); `3ee2ee2` (documentation re-check and restore drill); `c77fec4` (evidence bundle; clone and docs/secrets checks passed, restore check Blocked by host policy); `bc7671a` (final evidence bundle; clone and docs/secrets checks passed, restore check Blocked by host policy)
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

Executed at `b2bae3e` on 2026-10-27 unless the row states another revision. Rows marked `23b8996` used the release profile because the debug binary was blocked; they are not a canonical `scripts/check.ps1` PASS.

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
| Canonical check (docs re-check) | `scripts/check.ps1` | Repo root | PASS, 506 links in 106 files; 220 secret-scan files; eval 12, platform 261, schema 20, search 28 cases | `3ee2ee2` |

`platform` test split at `b2bae3e`: 217 unit, 13 CLI contract, 2 edge-case, 10 governance web, 8 integration, 7 search evaluation, 4 web health. The `governance_web` binary has `test = false` in `platform/Cargo.toml` because the host's Application Control intermittently blocks freshly built test harnesses; route coverage is provided by `platform/tests/governance_web.rs` and `platform/tests/web_health.rs`.

### CLI contract hardening at `23b8996`

The canonical debug binary could not be started on this host, so the checks below used the release profile. They are equivalent behavioral coverage, not a canonical `scripts/check.ps1` PASS.

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt -- --check` (both crates) | PASS |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` (both crates) | PASS, zero warnings |
| Tests | `cargo test --release --locked` (platform) | PASS, **270** tests: 217 unit, 22 CLI contract, 2 edge-case, 10 governance web, 8 integration, 7 search evaluation, 4 web health |
| Tests | `cargo test --release --locked` (eval) | PASS, 12 tests |
| Self-test | `cargo run --release --locked --bin ekp_platform -- --selftest` | PASS, 49 assertions |
| Schema | `cargo run --release --locked --bin ekp_platform -- --declared-rules-schema` | PASS, 20 records |
| Search evaluation | `cargo run --release --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json` | PASS, 28 cases, 20/20 top-1, 8/8 refusals, 4 role-restricted (2 allowed, 2 denied), 0 violations |
| Documentation | `scripts/check-docs.ps1` | PASS, 44 required, 507 links in 106 files at the time of the code run (the evidence update reports 508; a fresh clone checks 96 tracked files) |
| Secret scan | `scripts/check-secrets.ps1` | PASS, 220 files |
| Canonical check | `scripts/check.ps1` | **BLOCKED**: exit 1 at `platform` test. All 22 CLI contract tests failed to start `target\debug\ekp_platform.exe` with `os error 4551` (Windows Application Control). `EKP_PLATFORM_BIN` pointing at the release binary made the 270 tests pass, but the following `cargo run --locked --bin ekp_platform -- --selftest` was still denied (`FAIL: platform selftest`) |

The new contract behavior: `--llm` requires `--search` and accepts only `mock` (other providers exit `2` before data load or audit write); the output is labeled `mock-answer` and must carry an exact `[doc_id@version]` citation. `--audit-log` requires `--search`, writes one local `KnowledgeSearch` event (`actor: search-cli`, `tenant_id: default`) and replaces the target file; it is not an authenticated or append-only service. `--import-wellfit` is standalone, fails closed on an unreadable `docs.json` and validates imported documents before saving.

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
| `3ee2ee2` | `ekpi-platform-2026-10-27-125701.bundle` | `7E21EEEE456F36DE79056508BE718705D6294525D98E39C5F1C80C585C73FE19` | `git bundle verify` okay; automatic TEMP drill **BLOCKED** at `edge_cases` by Application Control (`os error 4551`); manual fresh clone at `%USERPROFILE%\EKPI-restore-drill-3ee2ee2` ran the restored `scripts/check.ps1` and exited 0 (eval 12, platform 261, docs 503 links in 96 tracked files, secrets 220) |
| `c77fec4` | `ekpi-platform-2026-10-27-132212.bundle` | `780464D606AC5A68D7ED1FF4883F58E3A9FEC9475D5948EA95F77B83D74D9FC9` | `git bundle verify` okay and records a complete history; automatic TEMP drill **BLOCKED** at `eval` Clippy (`serde`, `serde_core`, `proc-macro2` build scripts, `os error 4551`); two manual fresh clones (`EKPI-restore-drill-c77fec4` and `-c77fec4b`) reached `c77fec4`; the first clone's docs check (505 links in 96 tracked files) and secrets (220) passed, but both restored `scripts/check.ps1` runs failed at `eval` Clippy (`zmij`, `quote`, `serde_core`, `proc-macro2` build scripts); release-profile tests in the clone were also blocked. **Not** a restore PASS; superseded by `bc7671a` |
| `bc7671a` | `ekpi-platform-2026-10-27-132616.bundle` | `24E3BF7E64D7C7A92A40FC82F547E9F0D27594F436E7C715F7CDA563BE8DBB06` | `git bundle verify` okay and records a complete history; automatic TEMP drill **BLOCKED** at `eval` Clippy (`serde`, `serde_core`, `proc-macro2` build scripts, `os error 4551`); manual fresh clone at `%USERPROFILE%\EKPI-restore-drill-bc7671a` reached `bc7671a`; docs check (505 links in 96 tracked files) and secrets (220) passed, but the restored `scripts/check.ps1` failed at `eval` Clippy (`proc-macro2`, `serde_core` build scripts). **Not** a restore PASS |

The `56a944f` off-host drill receipt records ciphertext SHA-256 `0DBA33BCC90B94749DAF7547AF524364C5F4567358953E01A3D61260284AC181`, `decryption_verified: true` and `destination_class: local-simulated-off-host`. A drill under `%TEMP%` is **not** an off-host copy.

The `3ee2ee2` and earlier drills prove that committed history can be restored and checked locally. The final `bc7671a` bundle (and the preceding `c77fec4` bundle) is created and verifies, but its restore check is **Blocked** by host policy and is not a PASS; A8 therefore remains `Met` only for the earlier local drills, not for the final revision. None of these drills proves an off-host backup, an approved RPO/RTO, a scheduled restore programme, encryption-key custody or disaster recovery. Uncommitted work, the external governance ledger and ignored secrets are outside the bundle.

## Traceability

- Register: [traceability/register.md](../traceability/register.md). Rows name the specification, code, contract, test evidence and release revision.
- S1–S26, A-05 and A-06 are recorded; A-06 is `Implemented` with local session evidence, not enterprise identity. S23/S26 evidence is release-profile only at `23b8996`, where the canonical check was blocked.
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
8. **Host policy risk** — Windows Application Control intermittently blocks freshly built executables (`os error 4551`). At `23b8996` the canonical `scripts/check.ps1` was blocked at the platform test because the debug binary could not start; the release profile passed the same 270 tests. The automatic TEMP drill for `3ee2ee2` was blocked at `edge_cases`; a manual fresh clone of the identical bundle then passed. For the `c77fec4` and final `bc7671a` bundles, both the automatic TEMP drill and manual fresh clones were blocked at `eval` Clippy build scripts, and release-profile tests in the clone were blocked too; those restores are `Blocked`. A blocked run is recorded as `Blocked`, never `PASS`.

## Sign-off

This pack records what was verified on the named revisions. It is not production approval, compliance certification, enterprise authorization or evidence of real-user value. Phase A stays `Not approved`; Phase B stays `Blocked`.
