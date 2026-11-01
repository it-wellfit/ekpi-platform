# Final assurance report

- **Status:** Final for the 2026-10-27 local evidence baseline, with [2026-10-29](#0-2026-10-29-addendum-current-revision), [2026-10-30](#04-2026-10-30-multiline-bpmn-and-cookie-hardening-279c919) and [2026-11-02](#05-2026-11-02-legalruleml-export-and-bpmn-entity-hardening-eec11f2) addenda; **Phase A not approved**, **Phase B blocked**
- **Owner:** Engineering + QA; business, security, operations, data and independent-review roles `UNASSIGNED`
- **Evidence revision:** `eec11f2` (current hardening revision; 354 tests PASS, canonical **PASS** on the exact code commit, E-035); `279c919` (preceding hardening revision; 345 tests PASS, canonical **PASS** on two same-command runs, E-034); `21e23d6` (earlier hardening revision; 308 executed release-profile tests PASS, 27 tests blocked, canonical **BLOCKED** at the debug lib harness, E-033); `b2d8afc` (earlier hardening revision; 329 executed tests PASS, 2 edge tests blocked, canonical **BLOCKED**, E-032); `bbe49eb` (earlier hardening revision; release profile 325 tests, canonical **BLOCKED** at the debug CLI binary, E-031); `c827351` (earlier hardening revision; canonical **BLOCKED** at `edge_cases`); `604a2a5` (preceding working-tree canonical PASS); `32f2b36` (preceding canonical PASS); `718db57` (code change and earlier canonical PASS); `499e0d1` (current evidence bundle: automatic restore PASS); `6ed236d` (final evidence bundle clean-clone canonical PASS and automatic restore PASS); previous assurance bundle `41beaea`; code baseline `6ae6134`; release-profile and canonical checks re-run 2026-10-30 and the `eec11f2` checks on 2026-11-02
- **Environment:** Single managed Windows 11 host; Windows PowerShell 5.1; Rust 1.99.1; local loopback only; no remote, CI, staging or production
- **Matrix:** [phase-a-b-matrix.md](phase-a-b-matrix.md)
- **Test evidence:** [test-evidence.md](test-evidence.md)
- **Blockers:** [external-blockers.md](external-blockers.md)

## 0. 2026-10-29 addendum (current revision)

The 2026-10-27 sections below are retained unchanged as history. This addendum records the later hardening revision and must be read first for current status.

- **Current revision:** `eec11f2a9a02827e50317fbd0d146a3092f89ccf` — `fix: harden LegalRuleML export validation and BPMN entity decoding`. The six hardening revisions are recorded below in order (`c827351`, `bbe49eb`, `b2d8afc`, `21e23d6`, `279c919`, `eec11f2`); §0.5 is current.
- **Canonical result for the earlier `c827351` revision:** **BLOCKED**, not PASS. `scripts/check.ps1` exited 1 at the platform `cargo test --locked` step because the `edge_cases` harness could not start: `os error 4551`, Application Control policy `{0283ac0f-fff1-49ae-ada1-8a933130cad6}`, path `platform\target\debug\deps\edge_cases-111f9e750cab1c27.exe`. The harness was retried nine times and invoked directly with `--list`; every attempt was denied.
- **What still ran on `c827351`:** docs 53 required/667 links in 115 files PASS; secrets 231 files PASS; eval fmt, Clippy, 12 tests and 10 self-test checks PASS; platform fmt and Clippy PASS; unit **250**, CLI contract **28**, governance web **15**, integration **8**, search evaluation **7** and web health **4** PASS (**312** executed tests; the earlier `332` was an arithmetic error and is corrected here without changing any result); the 2 `edge_cases` tests are **NOT RUN**. Platform `--selftest` 49 assertions, schema 20 records, search gate 28 cases (20/20, 8/8, 4 role cases, 0 violations) and `--health` `health: OK` all exited 0. Details in [E-029](test-evidence.md#e-029-hardening-re-run-and-host-policy-block-c827351).
- **Engineering change:** `--health` now validates arguments first, accepts only `--data`, checks `rules.json`/`docs.json`/`processes.json`, and returns 0/1/2 for OK/DEGRADED/usage error (four new CLI contract tests). The absolute temp `target-dir` override and the `EKP_PLATFORM_BIN` test override were removed; tests use only `CARGO_BIN_EXE_ekp_platform` in the crate target directory. Both crates declare `[workspace]` so an unrelated parent manifest cannot capture them.
- **Phase A:** still **NOT APPROVED**. A1 `NOT MET`; A2/A3/A5/A6/A7/A9/A10/A11 `PARTIAL`; A4 is **`MET`** at current `eec11f2` because the canonical entry point exited 0 on the exact code commit and all 354 tests ran (E-035), and **`MET`** at preceding `279c919` because the canonical entry point exited 0 twice and all 345 release-profile tests ran (E-034); it was `BLOCKED` at `21e23d6` (E-033), `b2d8afc` (E-032) and `bbe49eb` (E-031). A8 `MET` only for the local `499e0d1` restore scope, and `c827351`/`bbe49eb`/`b2d8afc`/`21e23d6`/`279c919`/`eec11f2` are not yet bundled.
- **Phase B:** still **BLOCKED** (B1/B4/B5/B6/B8 blocked; B2/B3/B7 partial).
- **Boundary:** this addendum is local evidence on one managed host. It is not CI, staging, production, off-host, independent review, user acceptance or a security certification. No gate was lowered and no blocked test was counted as a pass.

### 0.1 2026-10-29 defect-hardening follow-up (`bbe49eb`)

The `c827351` findings led to a second code commit, `bbe49eb2ef3bd829ab51dcb340d4184250b2f050` (`fix: fail closed on replay, audit and header ambiguity`). It fixes ten defects found by review and adds eleven regression tests; every test was observed failing before the fix and passing afterwards.

- **Release-profile result:** platform **325** tests PASS (257 unit, 30 CLI contract, 2 edge, 17 governance web, 8 integration, 7 search evaluation, 4 web health); eval 12 tests + 10 self-test checks PASS; platform self-test 49 assertions, schema 20 records, search gate 28 cases (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) and `--health` `health: OK` all exit 0; `cargo fmt -- --check` and `cargo clippy --locked --all-targets -- -D warnings` pass in both crates. Details in [E-030](test-evidence.md#e-030-defect-hardening-re-run-bbe49eb).
- **Defects fixed:** search audit payload now uses a JSON serializer; existing audit events are structurally validated on read; replay validates event actors, record IDs, workflow versions and quality-case transitions; duplicate or non-UTF-8 `Origin`/`Sec-Fetch-Site` headers are refused; `--selftest` is mutually exclusive with every other flag; the BPMN scanner advances by the full ASCII prefix so multibyte text cannot cause a slice panic.
- **Regression caught during verification:** the new generic quality-case transition check initially displaced the CAPA-specific closure message, breaking the existing route test `governance_flow_persists_auditable_project_evidence_workflow_and_capa`. The message precedence was fixed in the implementation; the test expectation was not changed.
- **Canonical result:** **BLOCKED**, not PASS (E-031). At docs HEAD `17df06a`, docs, secrets, eval, platform fmt/Clippy and the 257 platform lib tests passed; `cargo test --locked` then stopped at `cli_contract` because the debug `ekp_platform.exe` was denied before spawn (`os error 4551`) for all 30 tests. The same path was retried with the identical result. No release-profile substitute, rename, copy, move or alternate target directory was used.
- **Phase A / Phase B:** unchanged. A4 remains `BLOCKED` (E-031); Phase A is **NOT APPROVED**; Phase B is **BLOCKED**. No external permission, approval, CI run, independent review or user acceptance was obtained.

### 0.2 2026-10-29 follow-up defect hardening (`b2d8afc`)

A follow-up review of the `bbe49eb` fail-closed changes found four more defects, fixed in `b2d8afc450a2b39364efc6a54ee27cd7bce28066` (`fix: close remaining replay actor, cookie and BPMN attribute gaps`) with six regression tests observed failing before the fix.

- **Release-profile result:** 329 executed platform tests PASS (262 unit, 30 CLI contract, 18 governance web, 8 integration, 7 search evaluation, 4 web health); the 2 `edge_cases` tests are **BLOCKED** at spawn (`os error 4551`, four same-path attempts) and are not counted. Eval 12 tests + 10 self-test checks, platform self-test 49, schema 20 and search 28 (20/20, 8/8, 4 role cases, 0 violations) pass; `--health` is `health: OK`; fmt and Clippy pass in both crates. Details in [E-032](test-evidence.md#e-032-second-defect-hardening-re-run-b2d8afc).
- **Defects fixed:** duplicate `Cookie` header lines are unauthenticated even when only one line carries a session; replay requires the payload actor/reviewer to match the event actor for evidence reviews, workflow publications, quality-case updates, quality inspections and gate decisions; BPMN attribute lookup is bounded to the current tag and does not fill a missing process/element ID from a later tag; BPMN tag and attribute names require a delimiter so longer names are not misparsed.
- **Canonical result:** **BLOCKED**, not PASS (E-032). Run 1 stopped at the debug platform lib harness; run 2 passed docs 53 required/677 links in 115 files, secrets 231, eval 12 + 10 self-test checks, platform fmt/Clippy and 262 lib tests, then stopped at the `cli_contract` harness (`os error 4551`) before its 30 tests. Both attempts used the same canonical command and paths; no release-profile substitute, rename, copy, move or alternate target directory was used.
- **Phase A / Phase B:** unchanged. A4 remains `BLOCKED`; Phase A is **NOT APPROVED**; Phase B is **BLOCKED**. No external permission, approval, CI run, independent review or user acceptance was obtained.

### 0.3 2026-10-29 audit whitespace-only fail-open fix (`21e23d6`)

A further review found one remaining audit fail-open, fixed in `21e23d6e5526caca5234cb651bbbad348d535996` (`fix: reject whitespace-only audit logs before append`) with three unit tests and one CLI-contract test.

- **Defect:** `read_jsonl` treated `contents.trim().is_empty()` as an empty log. A file containing only `"\n"` was accepted, the next append wrote an event after the blank line, and every later read rejected line 1 as empty, permanently poisoning the log.
- **Fix:** only a zero-byte file is an empty log; every non-empty file must be valid JSONL with a trailing newline and no blank lines.
- **Red/green:** with only the one-line guard reverted in the working tree, `read_jsonl_refuses_a_whitespace_only_file` failed (expected an error, received `AuditLog { events: [] }`). After restoring the `21e23d6` content, the full lib target passed **265/265** and the CLI contract passed **31/31**. A later single-test retry was denied by host policy after the temporary rebuild, so the green evidence is the earlier full-lib and CLI-contract runs.
- **Release-profile result:** 308 executed platform tests PASS (265 unit, 31 CLI contract, 8 integration, 4 web health); 27 tests are **BLOCKED** across three harnesses (`governance_web` 18 and `search_eval` 7 after two same-path attempts each, `edge_cases` 2) and are not counted. Eval 12 tests + 10 self-test checks, platform self-test 49, schema 20 and search 28 (20/20, 8/8, 4 role cases, 0 violations) pass; `--health` is `health: OK`; fmt and Clippy pass in both crates. Details in [E-033](test-evidence.md#e-033-audit-whitespace-only-fail-open-fix-21e23d6).
- **Canonical result:** **BLOCKED**, not PASS (E-033). Both same-command attempts passed docs 53 required/677 links in 115 files, secrets 231, eval 12 + 10 self-test checks, platform fmt/Clippy, then stopped at the debug platform lib harness (`os error 4551`) before any lib test could run. No release-profile substitute, rename, copy, move or alternate target directory was used.
- **Phase A / Phase B:** unchanged. A4 remains `BLOCKED`; Phase A is **NOT APPROVED**; Phase B is **BLOCKED**. No external permission, approval, CI run, independent review or user acceptance was obtained.

### 0.4 2026-10-30 multiline, BPMN and cookie hardening (`279c919`)

This revision fixes three usability/parser gaps found after the audit work: textarea newlines were rejected by the generic control-character guard, BPMN parsing only accepted the literal `bpmn:` prefix and double-quoted attributes, and logout did not mirror the login cookie's `Secure` attribute. It adds ten regression tests.

- **Fixes:** `validate_multiline_text` accepts LF and CRLF and rejects bare CR and other controls; `clean_multiline` stores CRLF as LF; workflow description, quality-case description and gate reason use it, while single-line fields still reject line breaks. BPMN matching uses local tag names (default namespace and arbitrary prefixes), a quote-aware attribute scanner accepts `'`/`"` and whitespace around `=`, and comments, CDATA and processing instructions are skipped. Login and logout share the `EKPI_COOKIE_SECURE` decision. The web pages render preserved newlines with `.multiline { white-space: pre-wrap; overflow-wrap: anywhere; }`.
- **Red/green:** no pre-fix red run was captured for these three behaviours. The only observed red was the new integration test using an incorrect hard-coded project ID (`PRJ-000002`); creating the project before the workflow fixed the fixture. No test was deleted, renamed to hide a failure, or weakened, and no expectation was changed to match the implementation.
- **Environment repair:** an initial canonical attempt failed at eval Clippy because stale debug `serde` build output contained `serde_flaps_core`; the registry build script and all source contain `serde_core`, not `serde_flaps_core`. Deleting only the `serde` build/fingerprint/dependency artifacts, then rerunning the same command, produced the results below. This is not a source fix or a lowered gate.
- **Release-profile result:** **345** platform tests PASS (273 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health); eval 12 tests + 10 self-test checks; platform self-test 49, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations) and `--health` `health: OK`; fmt and Clippy pass in both crates. Details in [E-034](test-evidence.md#e-034-multiline-textarea-bpmn-parser-and-secure-cookie-hardening-279c919).
- **Canonical result:** **PASS**, not blocked (E-034). Two same-command `scripts/check.ps1` runs exited 0 with docs 53 required/680 links in 115 files, secrets 231, eval 12 + 10 self-test checks, platform 345, self-test 49, schema 20, search 28 and `=== ALL CHECKS PASSED ===`. This is a local working-tree run, not CI, staging, production or independent execution.
- **Phase A / Phase B:** Phase A remains **NOT APPROVED** because A1 is `NOT MET`; A4 is now `MET` at this revision. Phase B remains **BLOCKED**. No external permission, approval, CI run, independent review or user acceptance was obtained.

### 0.5 2026-11-02 LegalRuleML export and BPMN entity hardening (`eec11f2`)

This revision fixes seven export/parser defects found in a second review pass. It was developed in an isolated git worktree on branch `audit/export-hardening` based on `c0c0e33`; the unrelated uncommitted main-worktree changes were not used, modified or committed.

- **Fixes:** comment escaping now repeats until no `--` remains; the validator rejects XML 1.0-invalid characters, non-comment markup declarations, and comments containing `--` or ending with `-`; effective dates must be real calendar days with the 4/100/400-year leap rule; forbidden construct names are matched against element local names rather than text and renamed lookalikes are refused; IDs and references are scanned only inside element tags with quote-aware parsing; BPMN attributes decode predefined and numeric entities, while unknown or XML-invalid references remain visible.
- **Red/green:** seven new tests were observed failing before the fixes (six export, one BPMN). Two further BPMN tests and a spoofed-ID negative case were added after the fixes, for nine regression tests. No test was deleted, renamed to hide a failure, or weakened.
- **Result:** **354** platform tests PASS (282 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health); eval 12 tests + 10 self-test checks; platform self-test 49, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations); fmt and Clippy pass in both crates. The export smoke wrote 51,026 bytes and an independent .NET `XmlDocument.Load` parsed root `lrml:LegalRuleML` with 718 elements. Details in [E-035](test-evidence.md#e-035-legalruleml-export-and-bpmn-entity-hardening-eec11f2).
- **Canonical result:** **PASS**, not blocked (E-035). `scripts/check.ps1` exited 0 on the exact code commit with docs 53 required/682 links in 105 files, secrets 231, eval 12 + 10 self-test checks, platform 354, self-test 49, schema 20, search 28 and `=== ALL CHECKS PASSED ===`. This is a local isolated-worktree run, not CI, staging, production, a clean clone or independent execution.
- **Phase A / Phase B:** Phase A remains **NOT APPROVED** because A1 is `NOT MET`; A4 is `MET` at this revision and at the preceding `279c919`. Phase B remains **BLOCKED**. No external permission, approval, CI run, independent review or user acceptance was obtained.

## 1. What was completed

### 1.1 Assurance record

This round created the missing assurance layer and connected it to the existing canonical documents:

| File | Content |
| --- | --- |
| [README.md](README.md) | Index, status vocabulary, evidence classes and non-claims |
| [phase-a-b-matrix.md](phase-a-b-matrix.md) | A1–A11 and B1–B8 requirement/status/defect/fix/test/threshold/owner/evidence records |
| [root-cause-analysis.md](root-cause-analysis.md) | RC-01–RC-12, including the canonical/restore host-policy root cause |
| [source-matrix.md](source-matrix.md) | Ten official sources plus four supporting tools, with version/date, applicability, evidence and limits |
| [test-evidence.md](test-evidence.md) | E-001–E-035: commands, environment, revision, exit codes, results and artifact paths |
| [external-blockers.md](external-blockers.md) | B-01–B-12: owner, missing resource, minimum access, unlock steps and acceptance criteria |
| [security-threat-model.md](security-threat-model.md) | Trust boundaries, TH-01–TH-15 assurance view, AST-01–AST-08, residual risk and non-claims |
| [operations-and-recovery.md](operations-and-recovery.md) | Local start/stop, monitoring, incident, backup/restore and DR procedure with explicit gaps |
| [final-report.md](final-report.md) | This consolidated result |

No existing canonical rule was replaced. The new pages link to the [phase gates](../quality/phase-gates.md), [production readiness](../operations/production-readiness.md), [threat model](../architecture/threat-model.md), [AI evaluation](../quality/ai-evaluation.md), [traceability register](../traceability/register.md) and [backup log](../operations/backup-log.md).

### 1.2 Engineering fixes already in the repository

These were completed in the preceding delivery commits and are re-verified here where the host permits:

| Fix | Commit | Root cause | Verification |
| --- | --- | --- | --- |
| CLI contract hardening: mock-only `--llm`, `--audit-log` requires `--search`, standalone `--import-wellfit`, fail-closed unreadable `docs.json`, usage errors for missing values/mixed modes | `23b8996` | Permissive argument parsing and implicit defaults | 22 CLI contract tests pass in release profile; canonical blocked by host policy |
| Local session auth for the workbench: PBKDF2-HMAC-SHA256 600k, CSPRNG salt/token, constant-time compare, lockout, Viewer/Editor/Approver, CSRF guard, CSP, 64 KiB body cap | `a826472` / `718db57` | No login and no route-level role boundary; mutation actor was form-supplied | 11 governance-web route tests (including session-derived actor) + 11 local-auth unit tests |
| Append-only JSONL search audit: verify chain before append, reject corrupt/legacy files, local writer lock, exit 1 on failure | `718db57` | Single-object overwrite was neither append-only nor tamper-evident | 11 audit unit tests + 24 CLI contract tests; canonical check exit 0 |
| Mock RAG labeling and versioned citation | `23b8996` | A mock could be mistaken for model capability | `mock-answer` label, citation requirement, contract tests |
| Backup script hardening: native stderr handling, .NET SHA-256, `age` drill, destination class | `56a944f` / `b2bae3e` | PowerShell native-command and `Get-FileHash` failures | Local restore PASS through `3ee2ee2`; `age` drill byte-identical (simulated) |
| Docs/traceability/evidence refresh | `3ee2ee2`, `c77fec4`, `bc7671a`, `28b68cc`, `6ae6134` | Evidence counts and limitations were stale or incomplete | Docs/secrets checks pass; release evidence updated |

### 1.3 What this round did not do

- It changed the served workbench and CLI audit behavior in `718db57`: web mutation actors come from the validated local session, and `--audit-log` is an append-only JSONL chain. It did not lower a gate, skip a failing test or rewrite history.
- The canonical check passed in a fresh clone at `41beaea`, at the final evidence revision `6ed236d` and at the current evidence bundle revision `499e0d1`; it also passed in the working tree at `4697aa1`, `718db57`, `32f2b36` and `604a2a5`. The earlier working-tree block at `41beaea` remains historical.
- It created the `41beaea` assurance bundle and the final `6ed236d` bundle, and verified both automatic restores (exit 0). The earlier `28b68cc` row remains `BLOCKED` and is retained as history.
- It did not create a remote, CI run, PR, branch protection, IdP, tenant model, AI provider, pilot or off-host backup.
- It did not obtain an independent review.

## 2. Test results

All results below are `LOCAL` unless marked `SIMULATED` or `HISTORICAL`. Full commands and output summaries are in [test-evidence.md](test-evidence.md).

| Check | Command (working directory) | Exit | Result |
| --- | --- | --- | --- |
| Canonical gate (isolated worktree, current) | `scripts/check.ps1` (root) | **0** | **PASS** at `eec11f2` on the exact code commit: docs 53 required/682 links in 105 files, secrets 231, eval 12 + 10 self-test checks, platform **354**, self-test 49, schema 20, search 28, `ALL CHECKS PASSED` (E-035) |
| Canonical gate (working tree, preceding) | `scripts/check.ps1` (root) | **0** | **PASS** at `279c919` on two same-command runs: docs 53 required/680 links in 115 files, secrets 231, eval 12 + 10 self-test checks, platform **345**, self-test 49, schema 20, search 28, `ALL CHECKS PASSED` (E-034) |
| Canonical gate (working tree, preceding) | `scripts/check.ps1` (root) | **1** | **BLOCKED** at `21e23d6`: both same-command attempts passed docs 53 required/677 links in 115 files, secrets 231, eval 12 + 10 self-test checks, platform fmt/Clippy, then stopped at the debug platform lib harness (`os error 4551`) before any lib test could run. Not a PASS (E-033) |
| Canonical gate (working tree, earlier) | `scripts/check.ps1` (root) | **1** | **BLOCKED** at `b2d8afc`: run 1 stopped at the debug platform lib harness (`os error 4551`); run 2 passed docs 53 required/677 links in 115 files, secrets 231, eval 12 + 10 self-test checks, platform fmt/Clippy and 262 lib tests, then stopped at the `cli_contract` harness (`os error 4551`) before its 30 tests. Same canonical command and paths on both attempts; not a PASS (E-032) |
| Canonical gate (working tree, earlier) | `scripts/check.ps1` (root) | **1** | **BLOCKED** at docs HEAD `17df06a` (code `bbe49eb`): docs 53/674/115 and secrets 231 PASS; eval 12 + 10 self-test checks PASS; platform fmt/Clippy and 257 lib tests PASS; `cli_contract` could not spawn the debug `ekp_platform.exe` (`os error 4551`) for all 30 tests. Same-path retry identical. Not a PASS (E-031) |
| Canonical gate (working tree, earlier) | `scripts/check.ps1` (root) | **1** | **BLOCKED**: platform `edge_cases` harness denied (`os error 4551`) at `c827351`; 250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation and 4 web health pass individually; 2 edge tests NOT RUN (E-029) |
| Canonical gate (working tree, 2026-10-27 historical) | `scripts/check.ps1` (root) | **0** | **PASS**: docs 53 required/660 links in 115 files, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28 (`604a2a5`) |
| Canonical gate (working tree, previous) | `scripts/check.ps1` (root) | **0** | **PASS**: docs 53 required/660 links in 115 files, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28 (`32f2b36`) |
| Canonical gate (working tree, earlier) | `scripts/check.ps1` (root) | **0** | **PASS**: docs 53 required/657 links in 115 files, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28 (`718db57`) |
| Canonical gate (working tree, historical) | `scripts/check.ps1` (root) | **1** | **BLOCKED** at platform test: 22 CLI contract tests could not start the existing debug binary (`os error 4551`) (`41beaea`) |
| Canonical gate (clean clone) | `scripts/check.ps1` (fresh clone of the `6ed236d` bundle) | **0** | **PASS**: docs 53 required/652 links in 105 tracked files, secrets 229, eval 12, platform 270, self-test 49, schema 20, search 28 |
| Docs | `scripts/check-docs.ps1` (root) | 0 | PASS: 53 required, 682 links in 105 files at the `eec11f2` canonical run; 680 links in 115 files at the `279c919` canonical runs; 677 at `21e23d6` and `b2d8afc`; 680 after the 2026-10-29 E-033 documentation reconciliation; 660 at `604a2a5`/`32f2b36`; 657 at `718db57`; 655 at `6ed236d`; 656 at `41beaea`; 652 at `947cd27`; 44/508/106 at the `6ae6134` code baseline |
| Secrets | `scripts/check-secrets.ps1` (root) | 0 | PASS: 231 files at the `eec11f2`, `279c919`, `21e23d6` and `b2d8afc` runs (229 at `6ed236d`; 220 at the `6ae6134` code baseline) |
| Format | `cargo fmt -- --check` (both crates) | 0 | PASS |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` (both crates) | 0 | PASS, no warnings |
| Release tests | `cargo test --release --locked` (platform) | 0 per executed target | PASS, **354** in the `eec11f2` canonical debug run (282 unit + 31 CLI contract + 2 edge + 20 governance web + 8 integration + 7 search evaluation + 4 web health); **345** at `279c919` (273 unit + 31 CLI contract + 2 edge + 20 governance web + 8 integration + 7 search evaluation + 4 web health); **308 executed** at `21e23d6` (265 + 31 + 8 + 4), with 27 tests **BLOCKED** (`governance_web` 18, `search_eval` 7, `edge_cases` 2); **329 executed** at `b2d8afc` (262 + 30 + 18 + 8 + 7 + 4), with 2 edge tests **BLOCKED**; **325** at `bbe49eb` (257 + 30 + 2 + 17 + 8 + 7 + 4); 270 at the `6ae6134` baseline (217 + 22 + 2 + 10 + 8 + 7 + 4); canonical debug profile at `718db57`/`32f2b36`/`604a2a5` passes **277** (221 + 24 + 2 + 11 + 8 + 7 + 4) |
| Release tests | `cargo test --release --locked` (eval) | 0 | PASS: 12 |
| Self-test | `cargo run --release --locked --bin ekp_platform -- --selftest` | 0 | PASS: 49 assertions |
| Schema | `cargo run --release --locked --bin ekp_platform -- --declared-rules-schema` | 0 | PASS: 20 records |
| Search gate | `cargo run --release --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json` | 0 | PASS: 28 cases, 20/20 top-1, 8/8 refusals, 4 role cases (2/2), 0 violations |
| Supply chain | `scripts/check-supply-chain.ps1` | 0 | PASS: 72/12 deps, 0 vulnerabilities; `cargo deny` OK |
| Current bundle | `git bundle verify` + automatic restore at `499e0d1` | verify 0; restore 0 | **PASS**: bundle `ekpi-platform-2026-10-27-170750.bundle`, SHA-256 `09A33F…09B8`; clone HEAD `499e0d1`; restored `scripts/check.ps1` exit 0 (docs 657/105, platform 277) |
| Final evidence bundle | `git bundle verify` + automatic restore at `6ed236d` | verify 0; restore 0 | **PASS**: bundle `ekpi-platform-2026-10-27-141734.bundle`, SHA-256 `E3E8C3EE…EEA13E`; clone HEAD `6ed236d`; restored `scripts/check.ps1` exit 0 |
| Earlier assurance bundle | `41beaea` bundle verify/restore | verify 0; restore 0 | **PASS (local)**: `ekpi-platform-2026-10-27-140645.bundle`, SHA-256 `7C5B337A…0623B3C`; clean-clone canonical check exit 0 |
| Earlier final bundle | `28b68cc` bundle verify/clone/restore | verify 0; restore 1 | Bundle verifies/clones; restore **BLOCKED** at `eval` Clippy (historical, not counted) |
| Last restore | `499e0d1` fresh clone + `scripts/check.ps1` | 0 | PASS (local); previous verified restores `6ed236d` and `41beaea` |
| Off-host drill | `scripts/backup-offhost.ps1 -Drill` | 0 | PASS for encryption/decryption; `SIMULATED`, not off-host |
| CI / PR / branch protection | n/a | n/a | **NOT RUN / BLOCKED**: no remote, no CI, no protection |
| Staging / production | n/a | n/a | **NOT RUN / BLOCKED**: no target |
| Real pilot / independent review | n/a | n/a | **NOT RUN / BLOCKED**: no named participants |

### Evidence classes

- `LOCAL`: this host, this repository/clone.
- `SIMULATED`: `age` drill to `%TEMP%`; no off-host service.
- `HISTORICAL`: prior revision evidence, not current.
- `CI` / `STAGING` / `PRODUCTION`: none.
- `INDEPENDENT REVIEW`: none.

## 3. Phase A status

| ID | Status | Reason |
| --- | --- | --- |
| A1 User value | **NOT MET** | No named pilot owner or representative user; no real workflow record. Prepared runbook and synthetic tests are not user acceptance. |
| A2 Functional correctness | **PARTIAL** | Release-profile tests and 67-case corpus pass; canonical passes in the working tree at `718db57`/`32f2b36`/`604a2a5` and the clean clone at `6ed236d`; target AT-01–AT-31 not run. |
| A3 Data correctness | **PARTIAL** | Version/state/replay/schema tests pass; no real-workflow dataset, independent review or DB transaction semantics. |
| A4 Engineering quality | **MET** (current and preceding); **BLOCKED** (historical) | At `eec11f2`, the canonical `scripts/check.ps1` exited 0 on the exact code commit (docs 53/682/105, secrets 231, eval 12 + 10 self-test checks, platform **354**, self-test 49, schema 20, search 28) and fmt/Clippy pass in both crates (E-035). At `279c919`, the canonical `scripts/check.ps1` exited 0 on two same-command runs (docs 53/680/115, platform **345**; E-034). At `21e23d6`, 308 executed release-profile tests passed but 27 tests were blocked across three harnesses and the canonical entry point was **BLOCKED** at the debug lib harness (E-033). At `b2d8afc`, 329 executed tests passed but 2 edge tests were blocked and the canonical entry point was **BLOCKED** (E-032). At `bbe49eb`, 325 release-profile tests passed but the canonical debug CLI binary was denied (E-031). At `c827351`, `edge_cases` was denied and 312 tests passed individually (E-029). No CI or named host-policy exception owner. |
| A5 Security | **PARTIAL** | Local session/role/CSRF hardening tested; no IdP, MFA, recovery, persistent identity, access review or tenant isolation. |
| A6 Maintainability | **PARTIAL** | Docs/runbooks exist; no named operator or handoff dry run. |
| A7 Traceability | **PARTIAL** | Register links requirements/code/tests/releases; no pilot run ID or validated row. |
| A8 Backup/restore | **MET** (local) | `499e0d1` bundle verifies, clones and the automatic restore drill exits 0 (docs 657/105, platform 277); `6ed236d` also remains restore-verified. No off-host/RPO/RTO. |
| A9 AI quality | **PARTIAL** | Fixed deterministic corpus/retrieval gate; no LLM/agent/tool path; tool-failure gap; mock is not AI quality. |
| A10 Docs/handoff | **PARTIAL** | Docs map and evidence current; no pilot handoff record. |
| A11 Review/improvement | **PARTIAL** | Local self-review/retro; no independent reviewer or second annotator. |

**Phase A decision: NOT APPROVED.** A1 is the decisive unmet requirement. A4 is now `MET` at the current `eec11f2` revision: the canonical entry point exited 0 on the exact code commit, all 354 platform tests ran, and fmt/Clippy pass (E-035); A4 is also `MET` at the preceding `279c919` revision (canonical exit 0 twice, 345 release-profile tests, E-034). The earlier `21e23d6` (E-033), `b2d8afc` (E-032) and `bbe49eb` (E-031) remain `BLOCKED` history. A8 is `MET` on the local restore scope at `499e0d1`, but neither `279c919` nor `eec11f2` is bundled; A2/A3/A5/A6/A7/A9/A10/A11 remain `PARTIAL`. No combination of release-profile tests, simulated drills or self-review can close A1 or the enterprise-identity, AI and independent-review gaps.

## 4. Phase B status

| ID | Status | Reason |
| --- | --- | --- |
| B1 Security/governance | **BLOCKED** | No security owner, IdP/MFA, tenant model, data policy or production target. |
| B2 Reliability/observability | **PARTIAL** | Liveness/metrics/logs/correlation/timeout exist; no SLO, traces, alerting, retry/backoff, circuit breaker or fault drill. |
| B3 Performance/scale | **PARTIAL** | One-host benchmark only; no approved workload, soak, bottleneck analysis or capacity model. |
| B4 Delivery/change governance | **BLOCKED** | Workflow file exists but no remote, CI run, branch protection, required checks or PR review. |
| B5 AI/agent quality | **BLOCKED** | No provider/data policy or model/tool path; mock only. |
| B6 Operations/support | **BLOCKED** | No named service/operations/support owner, on-call or escalation drill. |
| B7 Backup/DR | **PARTIAL** | Local bundle/restore and simulated `age` drill only; no off-host copy, key custody, approved RPO/RTO or DR exercise. |
| B8 Data governance | **BLOCKED** | No data owner, classification, retention/deletion, legal hold or residency decision. |

**Phase B decision: BLOCKED.** The weakest mandatory rows (B1/B4/B5/B6/B8) require external decisions, owners or environments. B2/B3/B7 cannot be accepted without an approved workload, SLO and off-host recovery target.

## 5. External blockers

| ID | Missing | Responsible role | Unlock |
| --- | --- | --- | --- |
| B-01 | Pilot owner, representative user, approved non-sensitive workflow | Business pilot owner + user | Run and sign one real workflow; link run ID/commit/evidence |
| B-02 | Remote, CI, branch protection, required checks | Repository admin + engineering | Push, run CI, configure protection and required checks |
| B-03 | IdP/MFA, account lifecycle, tenant model | IAM + security + data owner | Approve IdP/tenant, implement adapter, run cross-tenant denial tests |
| B-04 | Data owner, classification, retention/deletion, legal hold | Data owner + legal/privacy | Approve policy; test deletion/export; record access review |
| B-05 | AI provider/data policy, tool policy, budget | AI/data owner + security/legal | Approve provider; implement versioned eval; test tool failure |
| B-06 | Off-host target, key custody, RPO/RTO, restore owner | Operations + security + data | Approve target/RPO/RTO; retrieve/restore; test key recovery |
| B-07 | Named security/operations/support owners | Leadership + named owners | Accept roles; exercise incident/restore; sign handoff |
| B-08 | Independent reviewer, second annotator | Independent reviewer + QA | Rerun and sign the review checklist; run κ study |
| B-09 | Approved workload, SLO, cost/capacity budget | Product + operations | Approve workload/SLO; run load/soak; publish capacity model |
| B-10 | Production deployment target and change approval | Platform/operations + change approver | Approve staging first; deploy, rollback and verify |
| B-11 | Application Control exception or clean runner | Endpoint security owner | Scope exception/runner; rerun canonical and final restore |
| B-12 | Legal/privacy/compliance scope | Legal/privacy + compliance | Define obligations, map controls, sign assessment |

Detailed unlock steps, minimum access and acceptance criteria are in [external-blockers.md](external-blockers.md).

## 6. Final conclusion

### What can be proven

- The repository is a runnable local prototype with a deterministic adjudication core and a loopback governance workbench.
- At `eec11f2`, the canonical debug profile passed once on the exact code commit: docs 53/682/105, secrets 231, eval 12 + 10 self-test checks, platform 354, self-test 49 assertions, schema 20 records and search gate 28 cases with 20/20 top-1, 8/8 refusals, 4 role cases and 0 violations (E-035). At `279c919`, the canonical debug profile passed twice: docs 53/680/115, secrets 231, eval 12 + 10 self-test checks, platform 345, self-test 49 assertions, schema 20 records and search gate 28 cases with 20/20 top-1, 8/8 refusals, 4 role cases and 0 violations (E-034). At `718db57`/`32f2b36`/`604a2a5`, the canonical debug profile also passed (platform 277). At `c827351` the canonical check was blocked at the `edge_cases` harness (`os error 4551`); 312 other platform tests passed individually and 2 edge tests were NOT RUN. At `bbe49eb` the release profile passed 325 platform tests, but the canonical run at docs HEAD `17df06a` was `BLOCKED` at the debug CLI binary (E-031). At `b2d8afc` 329 executed tests passed, while 2 edge tests were blocked and the canonical entry point was `BLOCKED` (E-032). At `21e23d6` 308 executed tests passed, while 27 tests were blocked and the canonical entry point was `BLOCKED` (E-033).
- Format, Clippy, docs links, secret patterns and dependency audits pass locally on the named revision.
- Local Git bundles for `3ee2ee2`, `41beaea` and `6ed236d` were restored and checked successfully; the `56a944f` `age` drill encrypted and decrypted byte-identically to a local-simulated destination.
- A local session login, role checks, session-derived mutation actors, CSRF guard, CSP and request limits are implemented and tested.
- The final `6ed236d` bundle `ekpi-platform-2026-10-27-141734.bundle` verifies, clones to `6ed236d`, and its automatic restore drill ran the restored canonical `scripts/check.ps1` to exit 0 (platform 270, eval 12, self-test 49, schema 20, search 28, docs 652 links in 105 tracked files, secrets 229).
- The current `499e0d1` bundle `ekpi-platform-2026-10-27-170750.bundle` (SHA-256 `09A33F…09B8`) verifies, clones to `499e0d1`, and its automatic restore drill ran the restored canonical `scripts/check.ps1` to exit 0 (docs 657 links in 105 tracked files, secrets 229, platform 277, eval 12, self-test 49, schema 20, search 28).
- The previous `41beaea` assurance bundle also verifies, clones and ran the canonical check to exit 0.
- The working tree at `4697aa1`, `718db57`, `32f2b36` and `604a2a5` passed the canonical check; the earlier `41beaea` working-tree block (`os error 4551`) remains historical.
- The earlier `28b68cc` bundle verifies and clones, but its restored check remains `BLOCKED` and is not counted.
- No off-host backup, key custody, approved RPO/RTO or disaster-recovery exercise exists.

### What cannot be proven

- That the canonical `scripts/check.ps1` is reproducible on a different host or in CI. It passed once in an isolated worktree at `eec11f2` (E-035) and twice in the preceding working tree at `279c919` (E-034), at `604a2a5` and in a fresh clone at `6ed236d`, but the `c827351` run was blocked at the `edge_cases` harness (`os error 4551`), `bbe49eb` at docs HEAD `17df06a` was blocked at the debug CLI binary (E-031), `b2d8afc` had 2 edge tests blocked (E-032), `21e23d6` had 27 tests blocked (E-033), and there is no CI run or named host-policy exception owner.
- That the earlier `28b68cc` revision can be restored and checked on this host; it remains `BLOCKED` and is not counted.
- That the system is production-ready, secure, compliant, enterprise-authenticated, tenant-isolated, scalable or AI-validated.
- That CI, PR review, branch protection, a real pilot, independent review, off-host backup, approved RPO/RTO or a production deployment exists.
- That the lexical retrieval gate or mock LLM proves AI quality, hallucination resistance, tool success or cost/latency.

### Decision

**Phase A: not approved.** A1 is `NOT MET`; the pilot, enterprise-identity, AI and independent-review gaps remain. A4 is `MET` at the current `eec11f2` revision (canonical exit 0 on the exact code commit, 354 tests, E-035) and at the preceding `279c919` revision (canonical exit 0 twice, 345 release-profile tests, E-034); `21e23d6` (E-033), `b2d8afc` (E-032) and `bbe49eb` (E-031) remain blocked history. A8 is `MET` only on the local restore scope, and neither `279c919` nor `eec11f2` is bundled. **Phase B: blocked.**

The correct next actions are, in order: establish the pilot owner and real workflow for A1/A7; obtain independent review for A11; make the canonical check reproducible in the approved working environment or CI; then execute the Phase B identity, data, AI, SLO and off-host recovery work. Until those are evidenced and signed, no production or compliance claim is valid.
