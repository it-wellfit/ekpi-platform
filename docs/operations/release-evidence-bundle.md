# EKPI MVP release evidence bundle

> Evidence pack for the Phase A (0 → 1 MVP) candidate. Every claim names the
> command, revision, date, environment and result. `PASS` means the exact
> command ran on the named revision and exited 0. It does **not** prove
> production readiness, enterprise authentication, tenant isolation, AI quality
> or compliance certification.

- **Status:** Local verification record; Phase A **not approved**, Phase B **blocked**
- **Owner:** QA + Engineering
- **Last reviewed:** 2026-11-04
- **Verification revision:** `db7fc95` (current R2-7 eval revision: 366 platform tests and 41 eval tests pass, the 67-case eval corpus executes and the κ tool binds a study to it, one canonical `scripts/check.ps1` run exits 0 on the exact code commit, E-039; not yet bundled; κ not measured, CI never executed); `028037a` (preceding code revision: 366 platform tests and 20 eval tests pass and the 67-case eval corpus executes, one canonical `scripts/check.ps1` run exits 0 on the exact code commit, E-038; not yet bundled); `6ae475d` (earlier hardening revision: 366 platform tests pass, one canonical `scripts/check.ps1` run exits 0 on the exact code commit, E-037; not yet bundled); `d502147` (earlier hardening revision: 356 platform tests pass, two canonical `scripts/check.ps1` runs exit 0 on the exact code commit, E-036; not yet bundled); `eec11f2` (earlier hardening revision: 354 platform tests pass, one canonical `scripts/check.ps1` run exits 0 on the exact code commit, E-035; not yet bundled); `279c919` (earlier hardening revision: 345 platform tests pass, two canonical `scripts/check.ps1` runs exit 0, E-034; not yet bundled); `b2bae3e` (last canonical `gate.ps1 -IncludeSmoke` PASS); `23b8996` (historical code delivery revision: release-profile suite, blocked canonical check); `3ee2ee2` (documentation re-check and restore drill); `c77fec4` (evidence bundle; clone and docs/secrets checks passed, restore check Blocked by host policy); `bc7671a` (evidence bundle; clone and docs/secrets checks passed, restore check Blocked by host policy); `28b68cc` (evidence bundle; clone and docs/secrets checks passed, restore check Blocked by host policy); `6ae6134` (assurance baseline: release-profile suite PASS, canonical check Blocked by host policy, docs/secrets PASS); `41beaea` (assurance bundle: clean-clone canonical PASS, automatic restore PASS); `6ed236d` (final evidence bundle: documentation reconciliation, clean-clone canonical PASS, automatic restore PASS); `718db57` (session-derived actors + JSONL audit; working-tree canonical PASS, platform 277); `32f2b36` (evidence-document reconciliation; working-tree canonical PASS, docs 660 links); `604a2a5` (last working-tree canonical PASS; docs 660 links, platform 277); `499e0d1` (current evidence bundle: automatic restore PASS, docs 657/105, platform 277); `c827351` (earlier hardening revision: canonical `Blocked` at `edge_cases`; 312 other platform tests pass, 2 edge tests NOT RUN; not yet bundled); `bbe49eb` (earlier hardening revision: release-profile 325 platform tests pass; canonical at docs HEAD `17df06a` Blocked at the debug CLI binary, all 30 CLI contract spawns; not yet bundled); `b2d8afc` (earlier hardening revision: release-profile 329 executed platform tests pass, 2 edge tests Blocked at spawn; canonical `Blocked` on two same-command attempts at the debug lib and `cli_contract` harnesses, E-032; not yet bundled); `21e23d6` (earlier hardening revision: release-profile 308 executed platform tests pass, 27 tests Blocked at spawn across three harnesses; canonical `Blocked` on two same-command attempts at the debug lib harness, E-033; not yet bundled)**
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
| Release profile (preceding) | `cargo test --release --locked` | Both crates | PASS: platform **345** (273 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health), eval 12; `--selftest` 49, schema 20, search 28; fmt/Clippy PASS | `279c919` |
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
| Canonical check (assurance baseline) | `scripts/check.ps1` | Repo root | **BLOCKED**, exit 1: docs 508/106 and secrets 220 PASS, eval 12 PASS, platform 217 unit PASS, then all 22 CLI contract tests could not start the debug binary (`os error 4551`); `FAIL: platform test` | `6ae6134` |
| Release profile (assurance baseline) | `cargo test --release --locked` | Both crates | PASS: platform 270, eval 12; `--selftest` 49, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations); fmt/Clippy PASS; supply chain 72/12, 0 vulnerabilities | `6ae6134` |
| Canonical check (clean clone) | `scripts/check.ps1` | Fresh clone of final bundle | **PASS**, exit 0: docs 53 required/652 links in 105 tracked files, secrets 229, eval 12, platform 270, self-test 49, schema 20, search 28 | `6ed236d` |
| Canonical check (working tree, earlier) | `scripts/check.ps1` | Repo root | **PASS**, exit 0: docs 53 required/657 links in 115 files, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28 | `718db57` |
| Canonical check (working tree, previous) | `scripts/check.ps1` | Repo root | **PASS**, exit 0: docs 53 required/660 links in 115 files, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28 | `32f2b36` |
| Canonical check (working tree, 2026-10-27 historical) | `scripts/check.ps1` | Repo root | **PASS**, exit 0: docs 53 required/660 links in 115 files, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28 | `604a2a5` |
| Canonical check (current) | `scripts/check.ps1` | Repo root | **PASS**, exit 0 on one same-command run on the exact code commit: docs 53 required/700 links in 106 files, secrets 236, eval **41 tests + 10 self-test checks + 18 corpus checks**, platform **366**, self-test 49, schema 20, search 28; `ALL CHECKS PASSED` (E-039; κ not measured, CI never executed) | `db7fc95` |
| Canonical check (preceding) | `scripts/check.ps1` | Repo root | **PASS**, exit 0 on one same-command run on the exact code commit: docs 53 required/695 links in 106 files, secrets 235, eval **20 tests + 10 self-test checks + 16 corpus checks**, platform **366**, self-test 49, schema 20, search 28; `ALL CHECKS PASSED` (E-038; the earlier 17-check count was an overcount) | `028037a` |
| Canonical check (earlier) | `scripts/check.ps1` | Repo root | **PASS**, exit 0 on one same-command run on the exact code commit: docs 53 required/691 links in 105 files, secrets 231, eval 12 + 10 self-test checks, platform **366**, self-test 49, schema 20, search 28; `ALL CHECKS PASSED` (E-037) | `6ae475d` |
| Canonical check (earlier) | `scripts/check.ps1` | Repo root | **PASS**, exit 0 on two same-command runs on the exact code commit (second run `EXITCODE=0`): docs 53 required/687 links in 105 files, secrets 231, eval 12 + 10 self-test checks, platform **356**, self-test 49, schema 20, search 28; `ALL CHECKS PASSED` (E-036) | `d502147` |
| Canonical check (earlier) | `scripts/check.ps1` | Repo root | **PASS**, exit 0 on the exact code commit: docs 53 required/682 links in 105 files, secrets 231, eval 12 + 10 self-test checks, platform **354**, self-test 49, schema 20, search 28; `ALL CHECKS PASSED` (E-035) | `eec11f2` |
| Canonical check (earlier) | `scripts/check.ps1` | Repo root | **PASS**, exit 0 on two same-command runs: docs 53 required/680 links in 115 files, secrets 231, eval 12 + 10 self-test checks, platform **345**, self-test 49, schema 20, search 28; `ALL CHECKS PASSED` (E-034) | `279c919` |
| Canonical check (earlier) | `scripts/check.ps1` | Repo root | **BLOCKED**, exit 1 on two same-command attempts. Both passed docs 53/677/115 and secrets 231, eval 12 + 10 self-test checks, platform fmt/Clippy, then the debug lib harness could not start (`os error 4551`) before any lib test ran. The release profile has 308 executed tests passing and 27 tests blocked across three harnesses; not a PASS (E-033) | `21e23d6` |
| Canonical check (earlier) | `scripts/check.ps1` | Repo root | **BLOCKED**, exit 1 on two same-command attempts. Run 1: docs 53/677/115 and secrets 231 PASS, eval 12 + 10 self-test checks PASS, platform fmt/Clippy PASS, then the debug lib harness could not start (`os error 4551`). Run 2: the same pre-test checks passed and 262 lib tests PASS, then the `cli_contract` harness could not start (`os error 4551`) before its 30 tests. The release profile has 329 executed tests passing and 2 `edge_cases` tests blocked at spawn; not a PASS (E-032) | `b2d8afc` |
| Canonical check (earlier) | `scripts/check.ps1` | Repo root | **BLOCKED**, exit 1: docs 53/674/115 and secrets 231 PASS; eval 12 + 10 self-test checks PASS; platform fmt/Clippy and 257 lib tests PASS; then all 30 CLI contract tests could not spawn the debug `ekp_platform.exe` (`os error 4551`, Application Control); same-path retry identical; no release-profile substitute | `17df06a` (code `bbe49eb`) |
| Canonical check (earlier) | `scripts/check.ps1` | Repo root | **BLOCKED**, exit 1: platform `edge_cases` harness denied (`os error 4551`, Application Control policy `{0283ac0f-fff1-49ae-ada1-8a933130cad6}`) after nine retries; docs 53/667/115, secrets 231, eval 12, platform fmt/Clippy, 250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation and 4 web health pass, 2 edge tests NOT RUN | `c827351` |
| Current bundle restore | `scripts/backup.ps1` + restored `scripts/check.ps1` | Repo root / fresh clone | **PASS**: automatic TEMP drill exit 0; bundle verifies, clone HEAD `499e0d1`; `RESTORE DRILL PASSED`; docs 657/105, platform 277 | `499e0d1` |
| Final bundle restore | `scripts/backup.ps1` + restored `scripts/check.ps1` | Repo root / fresh clone | **PASS**: automatic TEMP drill exit 0; bundle verifies, clone HEAD `6ed236d`; `RESTORE DRILL PASSED` | `6ed236d` |

A documentation-only re-check after the E-034 reconciliation passed with 53 required documents and 680 relative links in 115 files; the secret scan passed with 231 files. The E-035 canonical run recorded 53 required documents and 682 relative links in 105 files, with the same 231-file secret scan. The E-036 canonical runs recorded 53 required documents and 687 relative links in 105 files, again with 231 secret-scan files. The documentation-only check after the E-036 evidence edits passed with 53 required documents and 691 relative links in 105 files, with the same 231 secret-scan files. The E-037 canonical run recorded 53 required documents and 691 relative links in 105 files, with the same 231-file secret scan. The documentation-only check after the E-037 evidence edits passed with 53 required documents and 695 relative links in 105 files, with the same 231 secret-scan files. The E-038 canonical run recorded 53 required documents and 695 relative links in 106 files, with 235 secret-scan files. The documentation-only check after the E-038 evidence edits passed with 53 required documents and 700 relative links in 106 files, with the same 235 secret-scan files. The E-039 canonical run recorded 53 required documents and 700 relative links in 106 files, with 236 secret-scan files; its eval crate ran 41 tests, 10 self-test checks and 18 corpus checks. These documentation checks do not change any test or canonical result.

`platform` test split at `b2bae3e`: 217 unit, 13 CLI contract, 2 edge-case, 10 governance web, 8 integration, 7 search evaluation, 4 web health. Historical split at `718db57`/`32f2b36`/`604a2a5`: 221 unit, 24 CLI contract, 2 edge-case, 11 governance web, 8 integration, 7 search evaluation, 4 web health = **277**. Split at `c827351`: 250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation, 4 web health = **312 executed**, plus 2 edge-case tests that are **NOT RUN** because the harness is host-blocked. Split at `bbe49eb`: 257 unit, 30 CLI contract, 2 edge-case, 17 governance web, 8 integration, 7 search evaluation, 4 web health = **325**. Split at `b2d8afc`: 262 unit, 30 CLI contract, 18 governance web, 8 integration, 7 search evaluation, 4 web health = **329 executed**, plus 2 edge-case tests **BLOCKED** at spawn and not counted. Current split at `db7fc95` and `028037a`: 294 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health = **366**. The eval split is 41 tests at `db7fc95` (27 unit + 14 real-process CLI contract) and 20 at `028037a`. Split at `21e23d6`: 265 unit, 31 CLI contract, 8 integration, 4 web health = **308 executed**, plus 27 tests **BLOCKED** at spawn (`governance_web` 18, `search_eval` 7, `edge_cases` 2) and not counted. The `governance_web` binary has `test = false` in `platform/Cargo.toml` because the host's Application Control intermittently blocks freshly built test harnesses; route coverage is provided by `platform/tests/governance_web.rs` and `platform/tests/web_health.rs`.

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

**Historical behavior at `23b8996`:** `--audit-log` requires `--search`, writes one local `KnowledgeSearch` event (`actor: search-cli`, `tenant_id: default`) and replaces the target file; it is not an authenticated or append-only service. The current contract is the JSONL append-only chain described below.

### Session-derived actors and JSONL audit at `718db57`

- Every `governance_web` mutation (`/projects`, `/evidence`, `/evidence/{id}/review`, `/inspections`, `/workflows`, `/workflows/{id}/{version}/publish`, `/controls`, `/controls/{id}/status`, `/lifecycle`) derives the ledger actor from the validated local session username. Submitted `actor`/`reviewer` fields are ignored; `/inspections` takes the reviewer from the session. This is local process-memory identity, not enterprise authentication.
- `--audit-log` appends one JSONL event to a hash chain. The CLI reads and verifies the existing chain before appending; a malformed line, broken chain or legacy single-object file exits `1` without modifying the file. Concurrent local writers serialise on a sibling `<PATH>.lock` directory. It is tamper-evident, not WORM or an authenticated audit service.
- Verification at `718db57`: canonical `scripts/check.ps1` exit 0; platform 277 tests (221 unit, 24 CLI contract, 2 edge, 11 governance web, 8 integration, 7 search evaluation, 4 web health); the 11 audit unit tests include tamper and 8-thread concurrent-append cases; the 24 CLI contract tests include two-run chain, corrupt refusal and legacy refusal; the 11 route tests include the `actor=Jordan` impersonation regression.
- `--llm` remains mock-only and `--import-wellfit` remains standalone; see the [CLI contract](../contracts/cli.md).

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

This is **not** an IdP, SSO, MFA, account recovery, persistent identity store, access review, tenant boundary or delegated administration. Web mutations record the validated local session username as the ledger actor and ignore submitted actor/reviewer fields; the CLI search audit uses the fixed `search-cli` label. Because the session is process-memory only, this is not an enterprise-verified identity. A local administrator can read or rewrite the ledger and process memory. `A5` stays `Partial`; `DEC-12` remains open.

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
| `bc7671a` | `ekpi-platform-2026-10-27-132616.bundle` | `24E3BF7E64D7C7A92A40FC82F547E9F0D27594F436E7C715F7CDA563BE8DBB06` | `git bundle verify` okay and records a complete history; automatic TEMP drill **BLOCKED** at `eval` Clippy (`serde`, `serde_core`, `proc-macro2` build scripts, `os error 4551`); manual fresh clone at `%USERPROFILE%\EKPI-restore-drill-bc7671a` reached `bc7671a`; docs check (505 links in 96 tracked files) and secrets (220) passed, but the restored `scripts/check.ps1` failed at `eval` Clippy (`proc-macro2`, `serde_core` build scripts). **Not** a restore PASS; superseded by `28b68cc` |
| `28b68cc` | `ekpi-platform-2026-10-27-132905.bundle` | `B9E398CAF38DCC0311DFE371E735846021D573D58312C8F9A8D0613392031C58` | `git bundle verify` okay and records a complete history; automatic TEMP drill **BLOCKED** at `eval` Clippy (`proc-macro2`, `serde_core` build scripts, `os error 4551`); manual fresh clone at `%USERPROFILE%\EKPI-restore-drill-28b68cc` reached `28b68cc`; docs check (505 links in 96 tracked files) and secrets (220) passed, but the restored `scripts/check.ps1` failed at `eval` Clippy (`proc-macro2` build script). **Not** a restore PASS |
| `41beaea` | `ekpi-platform-2026-10-27-140645.bundle` | `7C5B337A44EB9EA8A791522A107F5FA136299E1BB5CE32755F14448150623B3C` | **PASS**: `git bundle verify` okay, complete history, HEAD `41beaea`; automatic TEMP drill cloned and the restored `scripts/check.ps1` exited 0; direct clean-clone rerun also exited 0 (docs 653/105, secrets 229, eval 12, platform 270, self-test 49, schema 20, search 28) |
| `6ed236d` | `ekpi-platform-2026-10-27-141734.bundle` | `E3E8C3EE2DF8980A46008A14A7AA1A820D192DA47A99B8ABF0079ADEB3EEA13E` | **PASS**: `git bundle verify` okay, complete history, HEAD `6ed236d`; automatic TEMP drill cloned and the restored `scripts/check.ps1` exited 0 (docs 652/105, secrets 229, eval 12, platform 270, self-test 49, schema 20, search 28) |
| `499e0d1` | `ekpi-platform-2026-10-27-170750.bundle` | `09A33FFD9AAFC70C5D2BF701019B33CE8B67BED360B4A529B593FDA9E6B809B8` | **PASS**: `git bundle verify` okay, complete history, HEAD `499e0d1`; automatic TEMP drill cloned and the restored `scripts/check.ps1` exited 0 (docs 657/105, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28) |

The `56a944f` off-host drill receipt records ciphertext SHA-256 `0DBA33BCC90B94749DAF7547AF524364C5F4567358953E01A3D61260284AC181`, `decryption_verified: true` and `destination_class: local-simulated-off-host`. A drill under `%TEMP%` is **not** an off-host copy.

The `3ee2ee2` and earlier drills prove that committed history can be restored and checked locally. The `41beaea` bundle passed a clean-clone canonical check and automatic restore; the `6ed236d` bundle also verifies, clones and passes the automatic restore drill with the restored canonical check exit 0. The current `499e0d1` bundle (`ekpi-platform-2026-10-27-170750.bundle`, SHA-256 `09A33F…09B8`) verifies, clones to `499e0d1` and the automatic drill's restored check exits 0 (docs 657/105, secrets 229, platform 277). The earlier `28b68cc`, `bc7671a` and `c77fec4` bundles remain **Blocked** and are not counted. Local A8 is therefore `Met` at `499e0d1`; later working-tree canonical runs are recorded separately under A4 and do not change A8. None of these drills proves an off-host backup, an approved RPO/RTO, a scheduled restore programme, encryption-key custody or disaster recovery. Uncommitted work, the external governance ledger and ignored secrets are outside the bundle.

## Traceability

- Register: [traceability/register.md](../traceability/register.md). Rows name the specification, code, contract, test evidence and release revision.
- S1–S36, A-05 and A-06 are recorded; A-05 is `Partial` with κ tooling, a 67-row blinded sheet and fail-closed tests but no measured κ; A-06 is `Implemented` with local session evidence, not enterprise identity. S26 is historical (superseded by S28); S28 (append-only JSONL search audit), S29 (session-derived mutation actor), S31 (fail-closed replay/audit/header hardening), S32 (multiline/BPMN/cookie hardening), S33 (LegalRuleML export/BPMN entity hardening), S34 (RAG citation confidence/date-year hardening), S35 (second-precision deadlines/inclusive effective windows) and S36 (R2-7 eval corpus execution and κ study tooling) are `Verified`/`Partial` locally, with S31 at `bbe49eb`/`b2d8afc`/`21e23d6`/`279c919` and S28/S29 extended there. S23/S26 evidence is release-profile only at `23b8996`, where the canonical check was blocked. The canonical result for `bbe49eb` at docs HEAD `17df06a` is `BLOCKED` in E-031 at the debug CLI binary; `b2d8afc` has 2 edge tests blocked at spawn plus a canonical block (E-032); `21e23d6` has 27 tests blocked across three harnesses plus a canonical block (E-033). The earlier `279c919` revision passes 345 release-profile tests and the canonical entry point twice (E-034); the earlier `eec11f2` revision passes 354 platform tests and the canonical entry point once on the exact code commit (E-035); the preceding `d502147` revision passes 356 platform tests and the canonical entry point twice on the exact code commit (E-036); the preceding `6ae475d` revision passes 366 platform tests and the canonical entry point once on the exact code commit (E-037); the current `028037a` revision passes 366 platform tests, executes the 67-case eval corpus and passes the canonical entry point once on the exact code commit (E-038; κ not measured, CI never executed). Release-profile PASS for an earlier revision is not a current canonical PASS.
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
8. **Host policy risk** — Windows Application Control intermittently blocks freshly built executables (`os error 4551`). At `23b8996` the canonical `scripts/check.ps1` was blocked at the platform test because the debug binary could not start; the release profile passed the same 270 tests. The automatic TEMP drill for `3ee2ee2` was blocked at `edge_cases`; a manual fresh clone of the identical bundle then passed. For the `c77fec4`, `bc7671a` and `28b68cc` bundles, both the automatic TEMP drill and manual fresh clones were blocked at `eval` Clippy build scripts. At `41beaea` the automatic drill and a direct clean-clone canonical rerun both passed, while the working-tree canonical run still failed at the 22 CLI contract spawns. The working tree later passed at `4697aa1`, `718db57`, `32f2b36` and `604a2a5`. At `c827351` the canonical check was blocked at `edge_cases`; at `bbe49eb` the release suite passed all 325 platform tests while several freshly rebuilt harnesses were denied on first attempt and then passed on the same canonical paths; at `b2d8afc` 329 release-profile tests passed while the `edge_cases` harness was denied on four same-path attempts and the canonical entry point was denied on two same-command attempts (debug lib harness, then `cli_contract` harness; E-032); at `21e23d6` 308 release-profile tests passed while 27 tests across three harnesses were denied at spawn and the canonical entry point was denied on two same-command attempts at the debug lib harness (E-033); at `279c919` all 345 release-profile tests and two canonical runs pass after only stale `serde` build artifacts were cleared (E-034); at `eec11f2` all 354 platform tests and one canonical run pass in an isolated worktree (E-035); at `d502147` all 356 platform tests and two same-command canonical runs pass in the same isolated worktree (E-036); at `6ae475d` all 366 platform tests and one canonical run pass in the same isolated worktree (E-037). A blocked run is recorded as `Blocked`, never `PASS`; a pass is local and revision-bound, and the host policy still has no named exception owner.

## Sign-off

This pack records what was verified on the named revisions. It is not production approval, compliance certification, enterprise authorization or evidence of real-user value. Phase A stays `Not approved`; Phase B stays `Blocked`.
