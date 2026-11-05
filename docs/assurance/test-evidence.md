# Test and execution evidence

- **Status:** Active; local evidence only, with named blocks
- **Owner:** QA + Engineering
- **Last reviewed:** 2026-11-03
- **Repository revision for the re-run:** current R2-7 eval revision `db7fc95fcdec3496c1898e43967d6faab4cf0b45`; preceding code revision `028037a630a854506edc7e057ccb2c83d9275f92`; documentation-only commit between them `db691ca8e0a56a5649f1afb76ca5bf2d4bb0b67d`; earlier hardening revision `6ae475d8cb07fdce8f5fb864df6930767c81b852`; earlier hardening revision `d502147d3db70aefb108f2245c8e3232871225fe`; earlier hardening revisions `eec11f2a9a02827e50317fbd0d146a3092f89ccf`, `279c919471940e5a0406814ba78c1600bfb3c175`, `21e23d6e5526caca5234cb651bbbad348d535996`, `b2d8afc450a2b39364efc6a54ee27cd7bce28066`, `bbe49eb2ef3bd829ab51dcb340d4184250b2f050` and `c827351bc0ecf1db589d58b1b57b068360d12195`; code baseline `6ae61348194e2daa63a6ac53a31db00771d2a5e1`; assurance-document commit `947cd27f19ba124aa0f5854a25f1e2a4a910ac97`; evidence-record commit `a2ba7de34aae351a9507a052e4f1c1280659e070`; first final bundle revision `41beaea3ab2157721f6ac60b08c097bafaedd0dd`; final bundle revision `6ed236dc4769b3d7581608df1a325b42f46f2004` (`main`)
- **Working tree during the re-run:** code and eval files committed at `db7fc95` in an isolated git worktree on branch `audit/export-hardening` based on `c0c0e33`; the worktree was clean when the canonical run started. The separate main worktree contains unrelated uncommitted 2026-10-30 changes and was not part of the revision. `db691ca` is documentation-only and `028037a` is the preceding code revision. `947cd27`, `a2ba7de`, `41beaea` and `6ed236d` are documentation/evidence commits relative to `6ae6134`, so the code-level checks remain attributed to `6ae6134` except where a clean-clone canonical run is explicitly recorded
- **Environment:** One managed Windows 11 host, Windows PowerShell `5.1.26100.9711`, Rust `1.99.1` (`58cf8f9c3 2026-10-13`), Cargo `1.99.1` (`5f94df478 2026-08-27`), Git `2.56.0.windows.3`; 16,556,589,056 bytes RAM; local loopback only
- **Evidence classes:** `LOCAL` unless explicitly marked `SIMULATED` or `HISTORICAL`

`PASS` in this file means the exact command ran on the named revision and exited 0. A blocked command is recorded as `BLOCKED`, never as a pass. Historical rows are retained for traceability and are not current evidence.

## Evidence index

| ID | Class | Check | Revision | Exit | Result |
| --- | --- | --- | --- | --- | --- |
| E-001 | LOCAL | Canonical `scripts/check.ps1` | `6ae6134` | 1 | `BLOCKED` at platform test: 22 CLI contract tests cannot start the debug binary (`os error 4551`) |
| E-002 | LOCAL | Release-profile tests, both crates | `6ae6134` | 0 | PASS: platform 270, eval 12 |
| E-003 | LOCAL | Release-profile self-test, schema and search gate | `6ae6134` | 0 | PASS: 49 assertions, 20 schema records, 28 search cases (20/20, 8/8, 4 role cases, 0 violations) |
| E-004 | LOCAL | Documentation check | `6ae6134` | 0 | PASS: 44 required documents, 508 links in 106 files |
| E-005 | LOCAL | Secret pattern scan | `6ae6134` | 0 | PASS: 220 files |
| E-006 | LOCAL | Supply-chain check | `6ae6134` | 0 | PASS: platform 72 deps, eval 12 deps, 0 vulnerabilities; `cargo deny` advisories/bans/licenses/sources OK |
| E-007 | LOCAL | Final bundle verify/clone/restore | `28b68cc` | verify 0; restore blocked | Bundle verifies, clone reaches `28b68cc`; restored check `BLOCKED` at `eval` Clippy (`os error 4551`) |
| E-008 | HISTORICAL LOCAL | Earlier verified restore | `3ee2ee2` | 0 | PASS: fresh clone; restored `scripts/check.ps1` exit 0 (superseded by E-020 for `41beaea`) |
| E-009 | SIMULATED | `age` encrypted backup drill | `56a944f` | 0 | PASS for encryption/decryption/byte comparison; destination class `local-simulated-off-host` |
| E-010 | HISTORICAL LOCAL | SBOM generation | `a826472` | 0 | platform 67 components; eval 11 components; hashes recorded |
| E-011 | HISTORICAL LOCAL | Full gate `gate.ps1 -IncludeSmoke` | `b2bae3e` | 0 | 13/13 PASS (historical; not current) |
| E-012 | HISTORICAL LOCAL | Release benchmark | `56a944f` / `b2bae3e` | 0 | eval p50/p95 17.432/23.319 ms; loopback p50/p95 19.010/24.069 ms; 5,369.21 req/s |
| E-013 | NOT RUN | CI, PR, branch protection | n/a | n/a | `BLOCKED`: no remote, no CI execution, no branch protection |
| E-014 | NOT RUN | Staging/production deployment | n/a | n/a | `BLOCKED`: no approved environment or deployment target |
| E-015 | NOT RUN | Independent review / pilot | n/a | n/a | `BLOCKED`: no independent reviewer, pilot owner or representative user |
| E-016 | LOCAL | Documentation check (assurance commit) | `947cd27` | 0 | PASS: 53 required documents, 652 links in 115 files |
| E-017 | LOCAL | Secret pattern scan (assurance commit) | `947cd27` | 0 | PASS: 229 files |
| E-018 | LOCAL | Documentation and secret checks (evidence-record commit) | `a2ba7de` | 0 | PASS: 53 required documents, 655 links in 115 files; 229 secret-scan files |
| E-019 | LOCAL | Canonical `scripts/check.ps1` (working tree) | `41beaea` | 1 | `BLOCKED` at platform test: 22 CLI contract tests cannot start the existing debug binary (`os error 4551`); docs 53/656/115, secrets 229 and eval 12 passed first |
| E-020 | LOCAL | Final bundle restore + clean-clone canonical check | `41beaea` | 0 | PASS: bundle verifies, clone HEAD `41beaea`; automatic restore drill and direct canonical check exit 0 (platform 270, eval 12, self-test 49, schema 20, search 28, docs 653/105, secrets 229) |
| E-021 | LOCAL | Final docs/secrets check (working tree) | `41beaea` | 0 | PASS: 53 required documents, 656 links in 115 files; 229 secret-scan files |
| E-022 | LOCAL | Documentation and secret checks (final bundle revision) | `6ed236d` | 0 | PASS: 53 required documents, 655 links in 115 files; 229 secret-scan files |
| E-023 | LOCAL | Final bundle restore + clean-clone canonical check | `6ed236d` | 0 | PASS: bundle verifies, clone HEAD `6ed236d`; automatic restore drill and restored canonical check exit 0 (platform 270, eval 12, self-test 49, schema 20, search 28, docs 652/105, secrets 229) |
| E-024 | HISTORICAL LOCAL | Canonical `scripts/check.ps1` (working tree) | `4697aa1` | 0 | PASS: docs 53 required/655 links in 115 files, secrets 229, eval 12, platform 270 (217 unit, 22 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health), self-test 49, schema 20, search 28; superseded by E-025 |
| E-025 | HISTORICAL LOCAL | Canonical `scripts/check.ps1` (working tree, `IMP-013`/`IMP-014`) | `718db57` | 0 | PASS: docs 53 required/657 links in 115 files, secrets 229, eval 12 + 10 self-test checks, platform 277 (221 unit, 24 CLI contract, 2 edge, 11 governance web, 8 integration, 7 search evaluation, 4 web health) + 49 self-test assertions, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations); superseded by E-026 |
| E-026 | HISTORICAL LOCAL | Canonical `scripts/check.ps1` (working tree, evidence-document commit) | `32f2b36` | 0 | PASS: docs 53 required/660 links in 115 files, secrets 229, eval 12 + 10 self-test checks, platform 277 (221 unit, 24 CLI contract, 2 edge, 11 governance web, 8 integration, 7 search evaluation, 4 web health) + 49 self-test assertions, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations); superseded by E-028 |
| E-027 | LOCAL | Current bundle restore + clean-clone canonical check | `499e0d1` | 0 | PASS: bundle verifies, clone HEAD `499e0d1`; automatic restore drill and restored canonical check exit 0 (docs 657/105, secrets 229, platform 277, eval 12, self-test 49, schema 20, search 28) |
| E-028 | LOCAL | Canonical `scripts/check.ps1` (working tree, final evidence commit) | `604a2a5` | 0 | PASS: docs 53 required/660 links in 115 files, secrets 229, eval 12 + 10 self-test checks, platform 277 (221 unit, 24 CLI contract, 2 edge, 11 governance web, 8 integration, 7 search evaluation, 4 web health) + 49 self-test assertions, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations) |
| E-029 | LOCAL | Hardening re-run: canonical check, individual targets and health contract | `c827351` | 1 | **BLOCKED** at platform `cargo test --locked`: `edge_cases` cannot start (`os error 4551`, Application Control). Individual targets: 250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation and 4 web health PASS (**312**, not the earlier 332 arithmetic error); 2 edge tests NOT RUN. Docs 53/667/115, secrets 231, eval 12 + 10 self-test checks, platform self-test 49, schema 20, search 28 PASS |
| E-030 | LOCAL | Defect-hardening re-run: release tests, replay/audit/CSRF regressions and runtime checks | `bbe49eb` | 0 | PASS for every executed target: platform **325** (257 unit, 30 CLI contract, 2 edge, 17 governance web, 8 integration, 7 search evaluation, 4 web health), eval 12 + 10 self-test checks, platform self-test 49, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations), `--health` OK. Canonical entry point is BLOCKED in E-031 (debug CLI binary) |
| E-031 | LOCAL | Canonical entry point on the final docs revision | `17df06a` (code `bbe49eb`) | 1 | **BLOCKED**, not PASS: docs 53 required/674 links in 115 files and secrets 231 PASS; eval fmt/Clippy/12 tests/10 self-test checks PASS; platform fmt/Clippy PASS; platform lib **257** PASS; then `cargo test --locked` stopped at `cli_contract` because the debug `ekp_platform.exe` was denied before spawn (`os error 4551`, Application Control) for all 30 tests. A same-path retry gave the identical result. The debug binary also failed to start directly; the release binary's `--health` returned OK. No rename, copy, move or alternate target directory was used |
| E-032 | LOCAL | Second defect-hardening re-run: regression red/green, release tests, runtime checks and canonical entry point | `b2d8afc` | 1 (canonical); 0 per executed release target; combined `cargo test --release --locked` 101 | PASS for every executed release-profile target: platform **329** (262 unit, 30 CLI contract, 18 governance web, 8 integration, 7 search evaluation, 4 web health); **2 edge tests BLOCKED** at spawn (`os error 4551`) after four same-path attempts. Eval 12 + 10 self-test checks, platform self-test 49, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations), `--health` OK. Six new regression tests were observed failing before the fix. **Canonical `scripts/check.ps1` is BLOCKED, exit 1:** run 1 stopped at the debug lib harness; run 2 passed docs 677/115, secrets 231, eval and platform fmt/Clippy plus 262 lib tests, then stopped at the `cli_contract` harness (`os error 4551`) |
| E-033 | LOCAL | Audit whitespace-only fail-open fix: red/green regression, release tests, runtime checks and canonical entry point | `21e23d6` | 1 (canonical); 0 per executed release target | PASS for every executed release-profile target: platform **308** (265 unit, 31 CLI contract, 8 integration, 4 web health); **27 tests BLOCKED** at spawn (`governance_web` 18 after two attempts, `search_eval` 7 after two attempts, `edge_cases` 2). Eval 12 + 10 self-test checks, platform self-test 49, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations), `--health` OK; fmt and Clippy pass in both crates. Three new unit tests and one CLI contract test cover the whitespace-only fix. **Canonical `scripts/check.ps1` is BLOCKED, exit 1 on two same-command attempts:** both stopped at the debug lib harness (`os error 4551`) after docs 677/115, secrets 231, eval and platform fmt/Clippy passed |
| E-034 | LOCAL | Multiline textarea, BPMN parser and Secure-cookie hardening: release tests, runtime checks and canonical entry point | `279c919` | 0 | PASS. Platform **345** (273 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health); eval 12 + 10 self-test checks; platform self-test 49; schema 20; search 28 (20/20, 8/8, 4 role cases, 0 violations); `--health` OK; fmt and Clippy pass in both crates. Ten new tests (2 governance unit, 5 BPMN unit, 1 governance-web unit, 2 governance-web integration) cover LF/CRLF acceptance and CR/control rejection, workflow/quality-case/gate replay, BPMN local names, quote styles, attribute boundaries, comments/CDATA and Secure-cookie mirroring. **Canonical `scripts/check.ps1` exited 0 on two same-command runs:** docs 53 required/680 links in 115 files, secrets 231, eval 12 + 10 self-test checks, platform 345, self-test 49, schema 20, search 28, `ALL CHECKS PASSED` |
| E-035 | LOCAL | LegalRuleML export and BPMN entity hardening: red/green regression, export smoke and canonical entry point | `eec11f2` | 0 | PASS. Platform **354** (282 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health); eval 12 + 10 self-test checks; platform self-test 49; schema 20; search 28 (20/20, 8/8, 4 role cases, 0 violations); fmt and Clippy pass in both crates. Nine new tests (6 export, 3 BPMN; one export test extended with a spoofed-reference negative case) cover comment escaping, XML 1.0 character validity, comment/declaration well-formedness, real calendar dates, element-only forbidden-name matching, tag-only ID/ref scanning, entity decoding and invalid numeric references. Seven of them were observed failing before the fixes. **Canonical `scripts/check.ps1` exited 0 on the exact code commit:** docs 53 required/682 links in 105 files, secrets 231, eval 12 + 10 self-test checks, platform 354, self-test 49, schema 20, search 28, `ALL CHECKS PASSED` |
| E-036 | LOCAL | RAG citation confidence and xs:dateTime year-zero hardening: red/green regression and canonical entry point | `d502147` | 0 | PASS. Platform **356** (284 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health); eval 12 + 10 self-test checks; platform self-test 49; schema 20; search 28 (20/20, 8/8, 4 role cases, 0 violations); fmt and Clippy pass in both crates. Two new RAG tests and one extended date test cover cited-confidence threshold enforcement, cite-aware confidence, rejection of xs:dateTime year `0000` and acceptance of `0001`. All three observations were red before the fixes. **Canonical `scripts/check.ps1` exited 0 on the exact code commit (two same-command runs; the second captured `EXITCODE=0`):** docs 53 required/687 links in 105 files, secrets 231, eval 12 + 10 self-test checks, platform 356, self-test 49, schema 20, search 28, `ALL CHECKS PASSED` |
| E-038 | HISTORICAL LOCAL | R2-7 eval corpus execution, κ study tooling and CI wiring | `028037a` | 0 | PASS for the executable scope. Eval **20** tests (was 12), 10 selftest checks and **16 corpus checks over 67 cases** (9 seed + 46 casebook + 12 edge); the earlier "17" count in this row was an arithmetic overcount corrected by E-039. Corpus manifest rejects duplicate ids and a sub-50 corpus; the 67-row blinded sheet is checked for label leakage; three stubs exercise the full corpus. Platform **366** (294 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health); platform self-test 49; schema 20; search 28 (20/20, 8/8, 4 role cases, 0 violations); fmt and Clippy pass in both crates. Red before the change: `--corpus-selftest`, `--kappa` and `--make-annotation-sheet` were unknown arguments (exit 2), and the default report ran only 9 cases. **Canonical `scripts/check.ps1` exited 0 on the exact code commit:** docs 53 required/695 links in 106 files, secrets 235, eval 20 + 10 + 16 checks, platform 366, self-test 49, schema 20, search 28, `ALL CHECKS PASSED`. **κ is NOT MEASURED (0/2 annotators); the CI workflow has NEVER EXECUTED.** Superseded by E-039; R2-7 remains partial |

| E-039 | LOCAL | Eval CLI hardening and exact-corpus κ binding | `db7fc95` | 0 | PASS. Eval **41** tests (27 unit + 14 real-process CLI contract; was 20), 10 self-test checks and **18 corpus selftest checks** (corrected from the actual **16** on `028037a`). CLI is fail-closed: `--selftest`/`--corpus-selftest` accept no other argument, `--cases`/`--corpus` are mutually exclusive, empty or duplicate `--systems` selections and duplicate/flag-shaped values exit `2` before any report is written. `--kappa` now loads the fixed 67-case corpus and requires exact coverage; missing, extra and foreign case ids exit `2`. Annotation records reject unknown fields, so a leaked `expected_verdict` cannot enter a study. Landis–Koch bands are inclusive at the upper bound (`0.80` = `Substantial`, `0.81+` = `Almost perfect`) and non-finite κ is `Undefined`. Red evidence at `028037a`: `--selftest --bogus` exited 0, `--systems ","` exited 0 and wrote a report, and an invented 60-case study reported `cohens_kappa = 1.0`/`PASS`. **Canonical `scripts/check.ps1` exited 0 on exact `db7fc95`:** docs 53 required/700 links in 106 files, secrets 236, eval 41 tests + 10 + 18 checks, platform 366, self-test 49, schema 20, search 28, `ALL CHECKS PASSED`. **κ is NOT MEASURED (0/2 annotators); the CI workflow has NEVER EXECUTED.** R2-7 remains partial |

## E-001: canonical check (blocked)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on `6ae6134`, 2026-10-27:

```text
=== Documentation ===
PASS: required documents present (44)
PASS: relative Markdown links resolve (508 links in 106 files)
PASS: documentation
=== Secret Patterns ===
PASS: no high-signal secret patterns in 220 file(s)
PASS: secret patterns
=== Eval Crate ===
PASS: eval fmt
PASS: eval clippy
PASS: eval test (12)
PASS: eval selftest (10 checks)
=== Platform Crate ===
PASS: platform fmt
PASS: platform clippy
running 217 tests ... 217 passed
running 22 tests ... 0 passed; 22 failed
run ekp_platform: Os { code: 4551, kind: Uncategorized, message: "應用程式控制原則已封鎖此檔案。" }
FAIL: platform test
exit 1
```

The 22 failures are process-creation failures, not assertion failures. The affected test binary is `platform/target/debug/deps/cli_contract-*.exe`, which spawns `platform/target/debug/ekp_platform.exe`. The host's Application Control policy denies the file. `EKP_PLATFORM_BIN` pointing at the release binary makes the 270 tests pass, but the subsequent `cargo run --locked --bin ekp_platform -- --selftest` is still denied, so the canonical script cannot be converted into a pass by that environment variable.

**Classification:** `BLOCKED` (host policy). Do not cite as canonical PASS.

## E-002: release-profile tests

```powershell
cd platform
cargo test --release --locked

cd ..\eval
cargo test --release --locked
```

Observed on `6ae6134`:

| Suite | Count | Result |
| --- | --- | --- |
| platform unit tests (`src/lib.rs`) | 217 | PASS |
| platform CLI contract (`tests/cli_contract.rs`) | 22 | PASS |
| platform edge cases (`tests/edge_cases.rs`) | 2 | PASS |
| platform governance web (`tests/governance_web.rs`) | 10 | PASS |
| platform integration (`tests/integration.rs`) | 8 | PASS |
| platform search evaluation (`tests/search_eval.rs`) | 7 | PASS |
| platform web health (`tests/web_health.rs`) | 4 | PASS |
| **platform total** | **270** | **PASS** |
| eval unit tests | 12 | PASS |

Exit codes: platform `0`; eval `0`. This is behavioral coverage equivalent to the blocked debug suite, not a canonical `scripts/check.ps1` PASS.

## E-003: self-test, schema and search gate

```powershell
cd platform
cargo run --release --locked --bin ekp_platform -- --selftest
cargo run --release --locked --bin ekp_platform -- --declared-rules-schema
cargo run --release --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
```

Observed:

```text
selftest PASSED (49 assertions)
declared-rules-schema: 20 records validated clean
declared-rules-schema: OK
eval: 28 cases (set v1, threshold 0.42)
  top-1 hits: 20/20 answer cases
  refusals: 8/8 correct
  precision@1: 1.00
  citation support: 1.00
  role-restricted cases: 4 (allowed 2, denied 2), violations 0
  permission isolation: 1.00
  refusal accuracy: 1.00
eval gate: PASS
```

Exit codes: `0`, `0`, `0`. This is deterministic lexical retrieval regression evidence, not LLM/agent quality evidence. The four role cases use synthetic evaluation-only fixtures; they do not prove authenticated authorization or tenant isolation.

## E-004: documentation check

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check-docs.ps1
```

Observed:

```text
PASS: required documents present (44)
PASS: relative Markdown links resolve (508 links in 106 files)
Documentation check PASSED
exit 0
```

The count includes local untracked legacy pages; a fresh clone checks 96 tracked files (505 links at `28b68cc`). The check proves required files exist and relative links resolve; it does not prove content truth.

## E-005: secret scan

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check-secrets.ps1
```

Observed:

```text
PASS: no high-signal secret patterns in 220 file(s)
exit 0
```

The scan is a high-signal pattern scan, not a full entropy/DLP or credential-validity scan.

## E-006: supply chain

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check-supply-chain.ps1
```

Observed on `6ae6134`:

```text
cargo audit: platform 72 crate dependencies, 0 vulnerabilities
cargo audit: eval 12 crate dependencies, 0 vulnerabilities
cargo deny: advisories ok, bans ok, licenses ok, sources ok
SUPPLY CHAIN CHECK PASSED
exit 0
```

Tool versions: `cargo-audit 0.22.2`, `cargo-deny 0.20.2`; the advisory database loaded 1,378 advisories. A clean result means no known advisory matched at that time; it is not proof of absence. The `cargo deny` output includes `license-not-encountered` warnings for allowances that were not used; those warnings did not fail the check.

## E-007: final bundle and blocked restore

- Bundle: `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-27-132905.bundle`
- Revision: `28b68cc` (full SHA recorded in Git; the bundle is a complete-history bundle)
- SHA-256: `B9E398CAF38DCC0311DFE371E735846021D573D58312C8F9A8D0613392031C58`
- Size: 751,761 bytes
- `git bundle verify`: okay; complete history
- Manual clone: `%USERPROFILE%\EKPI-restore-drill-28b68cc`; `git rev-parse HEAD` = `28b68cc`
- Restored docs check: 505 links in 96 tracked files, PASS
- Restored secret scan: 220 files, PASS
- Restored `scripts/check.ps1`: `BLOCKED` at `eval` Clippy; `proc-macro2` build script denied (`os error 4551`)
- Automatic `%TEMP%` drill: `BLOCKED` at `eval` Clippy; `proc-macro2` and `serde_core` build scripts denied

This is **not** a restore PASS. At the time of this row, the last verified restore was E-008; E-020 now supersedes it for `41beaea`.

## E-008: earlier verified restore (`3ee2ee2`)

- Bundle: `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-27-125701.bundle`
- SHA-256: `7E21EEEE456F36DE79056508BE718705D6294525D98E39C5F1C80C585C73FE19`
- `git bundle verify`: okay; HEAD `3ee2ee2`
- Automatic `%TEMP%` drill: `BLOCKED` at `edge_cases` (`os error 4551`)
- Manual fresh clone: `%USERPROFILE%\EKPI-restore-drill-3ee2ee2`; restored `scripts/check.ps1` exit 0
- Restored check result: docs 503 links in 96 tracked files; secrets 220; eval 12; platform 261 (217 unit, 13 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health); self-test 49; schema 20; search 28 (20/20, 8/8, 4 role, 0 violations)

This was the last restore-verified local revision before `41beaea`. It does not transfer to `28b68cc`. The current local restore evidence is E-020 (`41beaea`).

## E-009: local-simulated `age` drill (`56a944f`)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backup-offhost.ps1 `
  -Destination "$env:TEMP\ekpi-offhost-drill" `
  -Recipient <throwaway age recipient> `
  -IdentityFile "$env:TEMP\ekpi-age-drill-key.txt" `
  -Drill
```

Observed at `56a944f`:

- Bundle SHA-256: `23EA44011711566CC7147767E6FF4258F75030CFFA0507EC98E00323AACE6421`
- Ciphertext SHA-256: `0DBA33BCC90B94749DAF7547AF524364C5F4567358953E01A3D61260284AC181`
- Decryption verified byte-identically
- `destination_class: local-simulated-off-host`
- `age` version on host: `v1.3.2`

This proves the script, encryption and decryption path. It is **not** an off-host copy, does not prove key custody, and does not satisfy B7.

## E-010: SBOM (historical, `a826472`)

- `ekp_platform`: 67 components; SHA-256 `5fdc80974503dcddd28ad8a73fb44877a5fc36144424f9acdbbf3699379f9478`
- `ekp_eval`: 11 components; SHA-256 `3a486a469b8b1976f18dd45d35218128881afe6134d057d4cb7f74015a405c04`
- `artifacts/sbom/SHA256SUMS` records both hashes
- No dependency changed between `a826472` and `b2bae3e`; the current release-profile test run did not regenerate the SBOMs

The documents use repository-relative locators, not the local absolute path or user name. The SBOM is not signed or independently attested.

## E-011: historical full gate (`b2bae3e`)

`gate.ps1 -IncludeSmoke` at `b2bae3e` (2026-10-27) reported 13/13 PASS, including docs, secrets, both crates, schema, search and smoke/export. This is historical. It is not a current `6ae6134` pass and does not clear the current canonical block. At `c827351`, the behavioral evidence is the 312 individually executed platform tests plus 49 self-test assertions, schema 20 and search 28; the canonical check remains Blocked at `edge_cases` (E-029). At `bbe49eb`, every platform target (325) and both crates' release checks pass; the canonical entry point is **BLOCKED** in E-031 at the debug CLI binary (`os error 4551`). At `b2d8afc`, 329 executed platform tests and both crates' release checks pass, while the 2 `edge_cases` tests are BLOCKED at spawn and the canonical entry point is BLOCKED at the debug lib and `cli_contract` harnesses (E-032). At `21e23d6`, 308 executed platform tests and both crates' release checks pass, while 27 tests across three harnesses are BLOCKED at spawn and the canonical entry point is BLOCKED at the debug lib harness (E-033). The release profile at `23b8996` and `6ae6134` is historical behavioral evidence, not a current canonical PASS.

## E-012: benchmark (historical, `56a944f` / `b2bae3e`)

Artifact: `artifacts/benchmark/benchmark-2026-10-27-124954.json`.

- Evaluation p50/p95: `17.432 / 23.319 ms`
- Loopback `/healthz` sequential p50/p95: `19.010 / 24.069 ms`
- Concurrent loopback: `5,369.21 req/s`

This is one host, one short observation. It is not a capacity model, soak test or SLO.

## E-013 to E-015: not run

| Check | Status | Reason | Required to unlock |
| --- | --- | --- | --- |
| GitHub Actions / required checks | `NOT RUN` / `BLOCKED` | No Git remote; `gh` not authenticated; workflow never executed | Authorized remote, CI runner, repository admin |
| PR review / branch protection | `NOT RUN` / `BLOCKED` | No remote or hosted review | Remote, branch protection, reviewer |
| Staging deployment | `NOT RUN` / `BLOCKED` | No approved target, credentials, network or data policy | Deployment owner and approved environment |
| Production deployment | `NOT RUN` / `BLOCKED` | Explicitly out of scope without authorization; no target | Production owner, change approval, rollback plan |
| Real pilot | `NOT RUN` / `BLOCKED` | No named pilot owner or representative user | A1 owner and approved non-sensitive workflow |
| Independent review | `NOT RUN` / `BLOCKED` | No reviewer independent of the implementer | A11 reviewer |
| Off-host restore | `NOT RUN` / `BLOCKED` | No approved off-host target or key custody | B7 owner, target, keys, RPO/RTO |

## E-016/E-017: assurance-document commit re-check

Executed on clean `947cd27` on 2026-10-27:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check-docs.ps1
powershell -ExecutionPolicy Bypass -File scripts/check-secrets.ps1
```

```text
PASS: required documents present (53)
PASS: relative Markdown links resolve (652 links in 115 files)
Documentation check PASSED
exit 0

PASS: no high-signal secret patterns in 229 file(s)
exit 0
```

The increase from 44/508/106 and 220 files is the nine new assurance documents plus the updated required-document list. This proves existence and link resolution, not the truth of the contents. The canonical `scripts/check.ps1` was not re-run after this documentation-only commit; its last run at `6ae6134` remains `BLOCKED` (E-001), and no code changed.

## E-018: evidence-record commit re-check

After adding the S27 traceability row and the evidence references, the clean evidence-record commit `a2ba7de` was checked on 2026-10-27:

```text
PASS: required documents present (53)
PASS: relative Markdown links resolve (655 links in 115 files)
Documentation check PASSED
exit 0

PASS: no high-signal secret patterns in 229 file(s)
exit 0
```

The three additional links are the assurance references in the S27 row. The file count is unchanged. This remains existence/link evidence; it does not establish content truth or a phase approval. The bundle created after this commit is recorded in the [backup log](../operations/backup-log.md).

## E-019: canonical check in the working tree at `41beaea` (blocked)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-27 at `41beaea`:

```text
PASS: required documents present (53)
PASS: relative Markdown links resolve (656 links in 115 files)
PASS: no high-signal secret patterns in 229 file(s)
PASS: eval fmt / clippy / test (12) / selftest
PASS: platform fmt / clippy
test result: ok. 217 passed; 0 failed
test result: FAILED. 0 passed; 22 failed
FAIL: platform test
exit 1
```

The 22 failures are again process-creation failures for the existing `platform\target\debug\ekp_platform.exe` (`os error 4551`). This is a working-tree block, not an assertion failure.

## E-020: final bundle restore and clean-clone canonical check (`41beaea`)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backup.ps1
# then, in the restored clone:
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-27:

- Bundle: `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-27-140645.bundle`
- SHA-256: `7C5B337A44EB9EA8A791522A107F5FA136299E1BB5CE32755F14448150623B3C`
- Size: 808,250 bytes
- `git bundle verify`: okay; refs `refs/heads/main` and `HEAD` both at `41beaea`; complete history
- Automatic restore drill: clone to `%TEMP%\ekpi-restore-drill`, HEAD `41beaea`; restored `scripts/check.ps1` exit 0; `RESTORE DRILL PASSED`
- Direct canonical run in the clone: exit 0; `ALL CHECKS PASSED`
- Clone check counts: docs 53 required/653 links in 105 tracked files; secrets 229; eval 12 + 10 self-test checks; platform 270 (217 unit, 22 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health); self-test 49 assertions; schema 20; search 28 (20/20, 8/8, 4 role cases, 0 violations)

This is a **local restore PASS** and a **revision-bound canonical PASS** in a fresh clone. It is not CI, staging, production or off-host evidence.

## E-021: final docs/secrets check

At clean `41beaea` in the working tree:

```text
PASS: required documents present (53)
PASS: relative Markdown links resolve (656 links in 115 files)
Documentation check PASSED
PASS: no high-signal secret patterns in 229 file(s)
exit 0
```

The fresh clone checks 105 tracked files (653 links); the working tree also contains ignored local pages, so it reports 115 files (656 links). Both runs cover all 53 required documents.

## E-022: final bundle-revision docs/secrets check (`6ed236d`)

```text
PASS: required documents present (53)
PASS: relative Markdown links resolve (655 links in 115 files)
Documentation check PASSED
PASS: no high-signal secret patterns in 229 file(s)
exit 0
```

This run followed the documentation reconciliation that removed the stale A4/A8 and final-restore statements. The tracked clone checks 105 files/652 links; the working tree includes ignored local pages.

## E-023: final bundle restore and clean-clone canonical check (`6ed236d`)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backup.ps1
```

Observed on 2026-10-27:

- Bundle: `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-27-141734.bundle`
- SHA-256: `E3E8C3EE2DF8980A46008A14A7AA1A820D192DA47A99B8ABF0079ADEB3EEA13E`
- Size: 827,919 bytes
- `git bundle verify`: okay; `refs/heads/main` and `HEAD` both at `6ed236d`; complete history
- Automatic restore drill: clone to `%TEMP%\ekpi-restore-drill`, HEAD `6ed236d`; restored `scripts/check.ps1` exit 0; `RESTORE DRILL PASSED`
- Clone check counts: docs 53 required/652 links in 105 tracked files; secrets 229; eval 12 + 10 self-test checks; platform 270 (217 unit, 22 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health); self-test 49 assertions; schema 20; search 28 (20/20, 8/8, 4 role cases, 0 violations)

This supersedes E-020 as the current final-bundle evidence. It is still local clean-clone evidence, not CI, staging, production or off-host evidence. The code is unchanged from `41beaea`/`6ae6134`; only documentation changed. The working-tree canonical run was not repeated for this docs-only revision and remains blocked at the 22 CLI contract spawns (`os error 4551`).

## E-024: working-tree canonical check (`4697aa1`, historical)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-27 at clean `4697aa1`: exit 0, `ALL CHECKS PASSED`.

- Docs 53 required/655 links in 115 files; secrets 229.
- Eval 12 tests + 10 self-test checks.
- Platform 270 tests (217 unit, 22 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health) + 49 self-test assertions.
- Schema 20 records; search gate 28 cases (20/20, 8/8, 4 role cases, 0 violations).

This was the first recorded working-tree canonical PASS after the earlier `41beaea` block. It is retained as historical evidence: the subsequent `IMP-013`/`IMP-014` change is verified separately in E-025.

## E-025: canonical check for the session-actor and JSONL-audit change (`718db57`)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-27 at clean `718db57` (working tree clean): exit 0, `=== ALL CHECKS PASSED ===`.

```text
PASS: required documents present (53)
PASS: relative Markdown links resolve (657 links in 115 files)
PASS: no high-signal secret patterns in 229 file(s)
PASS: eval fmt / clippy / test (12) / selftest (10 checks)
PASS: platform fmt / clippy
PASS: platform test
PASS: platform selftest (49 assertions)
PASS: platform declared-rules-schema (20 records)
PASS: platform search evaluation (28 cases)
=== ALL CHECKS PASSED ===
```

- Platform split: 221 unit, 24 CLI contract, 2 edge, 11 governance web, 8 integration, 7 search evaluation, 4 web health = **277**.
- The 11 audit unit tests include JSONL round-trip, tamper refusal without modification, legacy-format refusal and an 8-thread concurrent append that forms a single verified chain.
- The 24 CLI contract tests include two successive `--search --audit-log` runs forming a 2-event verified chain, corrupt-chain refusal and legacy single-object refusal.
- The 11 governance-web route tests include `ledger_actor_is_the_authenticated_session_not_the_form_value`: a POST with `actor=Jordan` records `tester` (the validated session username).
- This is local working-tree evidence for `718db57`. It is not CI, staging, production, off-host or independent review. The host Application Control policy still has no named exception owner, so a future blocked run must be recorded as `BLOCKED`.

## E-026: canonical check after the evidence-document reconciliation (`32f2b36`)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-27 at clean `32f2b36` (working tree clean): exit 0, `=== ALL CHECKS PASSED ===`.

- Docs 53 required/660 links in 115 files; secrets 229.
- Eval 12 tests + 10 self-test checks; platform 277 tests (221 unit, 24 CLI contract, 2 edge, 11 governance web, 8 integration, 7 search evaluation, 4 web health) + 49 self-test assertions; schema 20 records; search gate 28 cases (20/20, 8/8, 4 role cases, 0 violations).
- `32f2b36` is a documentation/evidence commit; the code is unchanged from `718db57`. The link count rose from 657 to 660 because the evidence index, register and release bundle gained cross-references.
- This is local working-tree evidence. It is not CI, staging, production, off-host or independent review.

## E-027: current bundle restore and clean-clone canonical check (`499e0d1`)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backup.ps1
```

Observed on 2026-10-27:

- Bundle: `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-27-170750.bundle`
- SHA-256: `09A33FFD9AAFC70C5D2BF701019B33CE8B67BED360B4A529B593FDA9E6B809B8`
- Size: 848,599 bytes
- `git bundle verify`: okay; `refs/heads/main` and `HEAD` both at `499e0d1`; complete history
- Automatic restore drill: clone to `%TEMP%\ekpi-restore-drill`, HEAD `499e0d1`; restored `scripts/check.ps1` exit 0; `RESTORE DRILL PASSED`
- Clone check counts: docs 53 required/657 links in 105 tracked files; secrets 229; eval 12 + 10 self-test checks; platform 277 (221 unit, 24 CLI contract, 2 edge, 11 governance web, 8 integration, 7 search evaluation, 4 web health) + 49 self-test assertions; schema 20; search 28 (20/20, 8/8, 4 role cases, 0 violations)

`499e0d1` records the `718db57` and `32f2b36` evidence; its code is unchanged from `718db57`. This is a **local restore PASS** and a **revision-bound canonical PASS** in a fresh clone. It is not CI, staging, production, off-host or independent review. The bundle is created before its own log row is committed, so the row is evidence in the later revision.

## E-028: final working-tree canonical check (`604a2a5`)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-27 at clean `604a2a5` (working tree clean): exit 0, `=== ALL CHECKS PASSED ===`.

- Docs 53 required/660 links in 115 files; secrets 229.
- Eval 12 tests + 10 self-test checks; platform 277 tests (221 unit, 24 CLI contract, 2 edge, 11 governance web, 8 integration, 7 search evaluation, 4 web health) + 49 self-test assertions; schema 20 records; search gate 28 cases (20/20, 8/8, 4 role cases, 0 violations).
- `604a2a5` records the current bundle restore evidence; the code is unchanged from `718db57`.
- This is local working-tree evidence. It is not CI, staging, production, off-host or independent review.

## E-029: hardening re-run and host-policy block (`c827351`)

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-29 at `c827351bc0ecf1db589d58b1b57b068360d12195` (code files clean at that revision; the 2026-10-29 documentation edits later recorded in `de419d1` were present but uncommitted during the run; unrelated `.opencode` skill changes were not part of the revision):

- Canonical exit **1**, stopped at the platform `cargo test --locked` step. The `edge_cases` harness could not start: `平台應用程式控制原則已封鎖此檔案 (os error 4551)` / Application Control policy `{0283ac0f-fff1-49ae-ada1-8a933130cad6}`. The path was `platform\target\debug\deps\edge_cases-111f9e750cab1c27.exe`. The final re-run after the 2026-10-29 documentation updates is saved at `%TEMP%\ekpi-canonical-20261029-094042.log`.
- The canonical run completed the checks before that point: docs 53 required/667 links in 115 files PASS; secrets 231 files PASS; eval fmt, clippy, 12 tests and 10 self-test checks PASS; platform fmt and clippy PASS; platform unit tests 250 PASS; CLI contract tests 28 PASS.
- The blocked `edge_cases` target was retried nine times over two intervals and also invoked directly with `--list`; every attempt was denied. Its two tests are **NOT RUN** and are not counted as passes.
- The remaining platform targets were run individually and exited 0: unit **250**, CLI contract **28**, governance web **15**, integration **8**, search evaluation **7**, web health **4**. Total executed and passing: **312** (250 + 28 + 15 + 8 + 7 + 4), excluding the two blocked edge tests. The earlier `332` figure in this row was an arithmetic error and is corrected here; no test result changed. This is not a substitute for a canonical PASS.
- Platform `--selftest` exited 0 with 49 assertions; `--declared-rules-schema` exited 0 with 20 records; `--eval --test-set data/eval/search_eval.json` exited 0 with 28 cases, 20/20 top-1, 8/8 refusals, 4 role-restricted cases (2 allowed, 2 denied), 0 violations; `--health` exited 0 with `health: OK`.
- The change adds four CLI contract tests for `--health`: OK on the default data, DEGRADED with missing core files, rejection of an unknown flag before any status output, and rejection when combined with `--selftest`.
- The committed absolute `target-dir` override and the `EKP_PLATFORM_BIN` test override are removed. Tests now use only `CARGO_BIN_EXE_ekp_platform` in the crate's canonical target directory. The two crates declare `[workspace]` so an unrelated parent manifest cannot capture them.
- Because the exact canonical command did not exit 0, A4 for `c827351` is **Blocked**, not `Met`. The earlier `604a2a5` working-tree PASS (E-028) and `6ed236d` clean-clone PASS (E-023) remain historical evidence for those revisions only.
- This is local evidence. It is not CI, staging, production, off-host or independent review. No test, gate or policy was lowered to obtain it.

## E-030: defect-hardening re-run (`bbe49eb`)

```powershell
cd platform
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --release --lib
cargo test --locked --release --test cli_contract
cargo test --locked --release --test edge_cases
cargo test --locked --release --test governance_web
cargo test --locked --release --test integration
cargo test --locked --release --test search_eval
cargo test --locked --release --test web_health
cargo run --locked --release --bin ekp_platform -- --selftest
cargo run --locked --release --bin ekp_platform -- --declared-rules-schema
cargo run --locked --release --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
cargo run --locked --release --bin ekp_platform -- --health

cd ..\eval
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked -- --selftest
```

Observed on 2026-10-29 at `bbe49eb2ef3bd829ab51dcb340d4184250b2f050` (code files committed; the E-030 documentation edits were uncommitted during the run):

| Target | Count | Exit |
| --- | --- | --- |
| Platform unit (`--lib`) | 257 | 0 |
| Platform CLI contract | 30 | 0 |
| Platform edge cases | 2 | 0 |
| Platform governance web | 17 | 0 |
| Platform integration | 8 | 0 |
| Platform search evaluation | 7 | 0 |
| Platform web health | 4 | 0 |
| **Platform total** | **325** | **0** |
| Eval unit/integration tests | 12 | 0 |
| Eval self-test checks | 10 | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |
| `--health` | `health: OK` | 0 |

The host Application Control policy remained intermittent. Several freshly rebuilt harnesses (`--lib`, `integration`, `web_health`, `governance_web`, `search_eval`) were denied with `os error 4551` on first attempt; retries of the same canonical paths later started and passed. No binary was renamed, copied, moved or pointed at an alternate target directory. A denial is still recorded as `BLOCKED` whenever it persists; it was not converted into a pass.

### Defects and regression tests

| ID | Defect | Failing regression test (observed before the fix) | Fix |
| --- | --- | --- | --- |
| D-01 | The `--search` audit payload was assembled by string interpolation. A query containing a quote, backslash, newline or control character produced syntactically invalid JSON inside an otherwise valid event. | `audit_payload_remains_valid_json_for_hostile_query_characters` (CLI contract): the pre-fix run failed with `invalid escape, line 1, column 30`. | The payload is serialized with `serde_json::json!`; no manual escaping. |
| D-02 | `audit::read_jsonl` verified the timestamp, duplicate IDs and hash chain but accepted an existing event with an empty ID, blank actor, control character in `tenant_id`, or empty/oversized payload. | `read_jsonl_refuses_structurally_invalid_existing_events` (unit): the pre-fix run returned an `AuditLog` containing the crafted event. | `validate_event_field` and `validate_payload` now run on every existing event before it is admitted. |
| D-03 | Ledger replay did not validate the event actor, so a crafted event with actor `"   "` entered the snapshot. | `replay_refuses_an_event_with_an_invalid_actor` (unit): the pre-fix run opened the ledger successfully. | Replay calls `validate_actor` on every event before `apply_event`. |
| D-04 | `--selftest` ran first and then ignored any other syntactically valid mode flag, so `--selftest --search ...` exited 0 without searching. | `selftest_refuses_a_valid_mode_flag_it_would_otherwise_ignore` (CLI contract): the pre-fix run exited 0 and printed no search result. | After the self-test, any argument other than `--selftest` is a usage error (exit 2); malformed arguments still fail after the self-test, preserving the earlier precedence rule. |
| D-05 | A repeated `Origin` header was resolved by taking the first value; an intermediary could use a different copy. | `duplicate_origin_headers_are_rejected` (governance web): the pre-fix run returned `303` and appended the project. | The same-origin guard requires exactly one UTF-8 header value for `Host`, `Origin` and `Sec-Fetch-Site` on mutations; duplicates and non-UTF-8 values receive `403`. |
| D-06 | A repeated `Sec-Fetch-Site` header had the same first-value ambiguity. | `duplicate_sec_fetch_site_headers_are_rejected` (governance web): the pre-fix run returned `303` and appended the project. | Same single-value helper as D-05; ambiguous fetch metadata is refused. |
| D-07 | Quality-case updates accepted `open → closed`, `open → verification`, `verification → open`, and reopening a closed case; replay did not check the transition graph. | `quality_case_rejects_skipped_transitions_and_reopening` and `replay_refuses_a_quality_case_that_skips_verification` (unit). | `quality_case_transition_allowed` permits only `open → in_progress`, `in_progress → verification`, `verification → in_progress`, `verification → closed`, plus same-state notes. Reopening a closed case is unsupported until a new due date and approval can be captured. The CAPA-specific closure message keeps precedence over the generic transition message; the existing `governance_flow_persists_auditable_project_evidence_workflow_and_capa` route test caught that message-precedence regression. |
| D-08 | Replay computed `workflow.version - 1` without checking version 0 (underflow/panic), and `save_workflow` used `prior.version + 1` (overflow). | `replay_refuses_workflow_version_zero` (unit): the pre-fix replay path did not reject the crafted event. | Version 0 is refused during replay; `save_workflow` uses `checked_add`. |
| D-09 | The BPMN scanner advanced by `tag.len() + 1` rather than the full `<tag` prefix length, so a multibyte character after a tag name caused an out-of-boundary slice panic. | `parse_does_not_panic_on_multibyte_text_after_a_tag_name` (unit): the pre-fix parser panicked on `<bpmn:task中`. | Scan positions advance by the full ASCII prefix length for elements and sequence flows. |
| D-10 | Replay accepted empty, oversized or non-ASCII record IDs for projects, evidence, workflows, publications, quality cases, inspections and gate decisions. | `replay_refuses_an_empty_record_id` (unit): the pre-fix run accepted a project with `id: ""`. | `validate_record_id` enforces a non-empty ASCII `<PREFIX>-<digits>`-style identifier of 1–80 characters for every creating payload. |
| D-11 | `session_token` iterated every `Cookie` header line, so a request with two lines was authenticated when only the second carried `ekpi_session`. | `duplicate_cookie_header_lines_are_rejected_even_when_only_one_has_a_session` (governance web): the pre-fix run returned `200` instead of `401`. | Any request with more than one `Cookie` header line is unauthenticated; the server never scans one line after seeing another. |
| D-12 | Replay validated the payload reviewer/actor only for shape, not for agreement with the event actor. A crafted event could show actor A in the envelope while attributing a review, publication, quality update, inspection or gate decision to B. | `replay_refuses_payload_actors_that_differ_from_the_event_actor` (unit): the pre-fix run applied all five crafted payloads successfully. | `require_payload_actor` requires `EvidenceReviewed.reviewer`, `WorkflowPublished.reviewer`, `QualityCaseUpdated.actor`, `QualityInspectionRecorded.reviewer` and `GateDecisionRecorded.actor` to equal `event.actor`. |
| D-13 | `extract_attr` searched from the tag start to the end of the document, so a tag without `id` borrowed the next tag's `id`; process `id`/`name` extraction had the same defect. | `attributes_do_not_leak_from_a_later_tag` and `process_metadata_does_not_leak_from_a_later_tag` (unit): the pre-fix parser produced a phantom task and a process ID of `task-id`. | Attribute search is bounded to the current tag, and the closing `>` is found outside quoted values. |
| D-14 | The BPMN scanner treated `<bpmn:taskSequence` as `<bpmn:task`, and `data-id="…"` as `id="…"`. | `tag_prefix_requires_a_delimiter` and `attribute_names_require_a_delimiter` (unit): the pre-fix parser produced an extra task for both inputs. | A tag or attribute name must end at whitespace, `/` or `>`. |

Every named regression test was executed before and after its fix. The four BPMN tests, the replay-actor test and the duplicate-Cookie test were all observed failing on the release profile before the fix. The final write-up was produced after the fixes.

This is local release-profile evidence on one managed host. It is not CI, staging, production, off-host execution, independent review or user acceptance. The canonical `scripts/check.ps1` result for this revision is recorded separately in E-031; a green release suite does not replace it.

## E-031: canonical entry point on the final docs revision

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-29 at `17df06a` (the documentation commit on top of code `bbe49eb2ef3bd829ab51dcb340d4184250b2f050`):

| Step | Result |
| --- | --- |
| Documentation | PASS: 53 required documents, 674 links in 115 files |
| Secret patterns | PASS: no high-signal patterns in 231 files |
| Eval fmt / Clippy / tests / self-test | PASS: 12 tests, 10 self-test checks |
| Platform fmt / Clippy | PASS |
| Platform unit (`--lib`) | PASS: 257 tests |
| Platform CLI contract | **BLOCKED**: all 30 tests failed before spawn; the debug `ekp_platform.exe` could not start (`os error 4551`, Application Control) |
| Remaining canonical steps | **NOT RUN**: the script stops at the first failure, so edge, governance-web, integration, search-eval, web-health and the runtime self-tests were not reached in this run |
| **Command exit** | **1 — BLOCKED, not PASS** |

The block is on the debug binary itself, not on the test logic: invoking `platform\target\debug\ekp_platform.exe --health` directly also failed to start, while the release-profile `platform\target\release\ekp_platform.exe --health` returned `health: OK` and exit 0. The same canonical command was retried on the same path and produced the identical 30/30 denial. No binary was renamed, copied or moved, and no alternate target directory or release-profile substitute was used; a release-profile pass (E-030) is not a canonical PASS.

Status: **BLOCKED**. The code-level release suite is green (E-030), but this revision has no canonical PASS. The host policy has no named exception owner or CI runner (B-11). A future canonical run that exits 0 on a new revision can replace this row; this row must not be rewritten as PASS.

## E-032: second defect-hardening re-run (`b2d8afc`)

```powershell
cd platform
cargo fmt -- --check
cargo clippy --release --locked --all-targets -- -D warnings
cargo test --release --locked --lib
cargo test --release --locked --test cli_contract
cargo test --release --locked --test edge_cases
cargo test --release --locked --test governance_web
cargo test --release --locked --test integration
cargo test --release --locked --test search_eval
cargo test --release --locked --test web_health
cargo run --release --locked --bin ekp_platform -- --selftest
cargo run --release --locked --bin ekp_platform -- --declared-rules-schema
cargo run --release --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
cargo run --release --locked --bin ekp_platform -- --health

cd ..\eval
cargo fmt -- --check
cargo clippy --release --locked --all-targets -- -D warnings
cargo test --release --locked
cargo run --release --locked -- --selftest
```

Observed on 2026-10-29 at `b2d8afc450a2b39364efc6a54ee27cd7bce28066` (code files committed; the E-032 documentation edits were uncommitted during the run):

| Target | Count | Exit |
| --- | --- | --- |
| Platform unit (`--lib`) | 262 | 0 |
| Platform CLI contract | 30 | 0 |
| Platform edge cases | **BLOCKED**, 0 executed of 2 | 1 |
| Platform governance web | 18 | 0 |
| Platform integration | 8 | 0 |
| Platform search evaluation | 7 | 0 |
| Platform web health | 4 | 0 |
| **Platform executed and passing** | **329** | 0 per completed target |
| Eval unit/integration tests | 12 | 0 |
| Eval self-test checks | 10 | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |
| `--health` | `health: OK` | 0 |

The combined `cargo test --release --locked` command exited 101 because the `edge_cases` harness could not start (`os error 4551`, Application Control). The same harness path was attempted four times (`cargo test --release --locked` and three direct `--test edge_cases` retries) and was denied every time; its two tests are **NOT RUN** and are not counted. A fresh CLI-contract spawn and the `web_health` harness were also denied once each and passed on a same-path retry. No binary was renamed, copied, moved or pointed at an alternate target directory.

Before the fix, the targeted release-profile runs failed as follows, which is the red side of the regression evidence:

- `attributes_do_not_leak_from_a_later_tag`, `process_metadata_does_not_leak_from_a_later_tag`, `tag_prefix_requires_a_delimiter` and `attribute_names_require_a_delimiter` each failed with the phantom-element or borrowed-attribute behaviour described in D-13 and D-14.
- `replay_refuses_payload_actors_that_differ_from_the_event_actor` failed because `apply_event` returned `Ok` for all five mismatched payloads (D-12).
- `duplicate_cookie_header_lines_are_rejected_even_when_only_one_has_a_session` returned `200` instead of `401` (D-11).

After the fixes the same six tests pass in the release profile, along with the rest of the executed suite. No failing test was deleted, renamed or weakened, and no expectation was changed to match the new behaviour.

### Canonical entry-point result

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-29 at `b2d8afc450a2b39364efc6a54ee27cd7bce28066` (code committed; the E-032 documentation edits were uncommitted during the run), **exit 1 — BLOCKED, not PASS**:

- **First canonical run:** documentation PASS (53 required documents, 677 links in 115 files); secrets PASS (231 files); eval fmt, Clippy, 12 tests and 10 self-test checks PASS; platform fmt and Clippy PASS; then the debug `ekp_platform` lib harness could not start (`os error 4551`, Application Control) and `scripts/check.ps1` printed `FAIL: platform test`.
- **Second canonical run (same command, same paths, no workaround):** documentation and secrets PASS with the same counts; eval PASS as above; platform fmt and Clippy PASS; **262 lib tests PASS**; then the `cli_contract` harness could not start (`os error 4551`) before any of its 30 tests ran, and the script printed `FAIL: platform test` again.

The canonical debug profile therefore has **no PASS** at `b2d8afc`. The release-profile evidence above (329 executed tests) is behavioral coverage only and is not substituted for the canonical result. No binary was renamed, copied, moved or pointed at an alternate target directory, and the repository-level `target-dir` override or `EKP_PLATFORM_BIN` test override was not reintroduced.

## E-033: audit whitespace-only fail-open fix (`21e23d6`)

```powershell
cd platform
cargo fmt -- --check
cargo clippy --release --locked --all-targets -- -D warnings
cargo test --release --locked --lib
cargo test --release --locked --test cli_contract
cargo test --release --locked --test edge_cases
cargo test --release --locked --test governance_web
cargo test --release --locked --test integration
cargo test --release --locked --test search_eval
cargo test --release --locked --test web_health
cargo run --release --locked --bin ekp_platform -- --selftest
cargo run --release --locked --bin ekp_platform -- --declared-rules-schema
cargo run --release --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
cargo run --release --locked --bin ekp_platform -- --health

cd ..\eval
cargo fmt -- --check
cargo clippy --release --locked --all-targets -- -D warnings
cargo test --release --locked
cargo run --release --locked -- --selftest
```

Observed on 2026-10-29 at `21e23d6e5526caca5234cb651bbbad348d535996` (code committed; the E-033 documentation edits were uncommitted during the run):

| Target | Count | Exit |
| --- | --- | --- |
| Platform unit (`--lib`) | 265 | 0 |
| Platform CLI contract | 31 | 0 |
| Platform edge cases | **BLOCKED**, 0 executed of 2 | 1 |
| Platform governance web | **BLOCKED**, 0 executed of 18 | 1 (two same-path attempts) |
| Platform integration | 8 | 0 |
| Platform search evaluation | **BLOCKED**, 0 executed of 7 | 1 (two same-path attempts) |
| Platform web health | 4 | 0 |
| **Platform executed and passing** | **308** | 0 per completed target |
| Eval unit/integration tests | 12 | 0 |
| Eval self-test checks | 10 | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |
| `--health` | `health: OK` | 0 |

The defect was an audit fail-open: `read_jsonl` treated `contents.trim().is_empty()` as an empty log. A file containing only `"\n"` was therefore accepted, the next append wrote an event after the blank line, and every subsequent read rejected line 1 as empty, permanently poisoning the log. The fix treats only a zero-byte file as empty; every non-empty file must be valid JSONL with a trailing newline and no blank lines.

Red/green evidence:

- With only the one-line guard reverted in the working tree, `read_jsonl_refuses_a_whitespace_only_file` failed: it expected an error but received `AuditLog { events: [] }`.
- After restoring the `21e23d6` content, the full lib target passed **265/265** and the CLI contract passed **31/31**. A later single-test retry was denied by host Application Control (`os error 4551`) after the temporary rebuild, so the green evidence is the earlier full-lib and CLI-contract runs, not that retry.
- The fix adds three unit tests (whitespace-only file, zero-byte file, append refusal without modification) and one CLI-contract test (whitespace-only file refused without modification). No test was deleted, renamed to hide a failure, or weakened.

### Canonical entry-point result

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-29 at `21e23d6`, **exit 1 — BLOCKED, not PASS**:

- **First canonical run:** documentation PASS (53 required documents, 677 links in 115 files); secrets PASS (231 files); eval fmt, Clippy, 12 tests and 10 self-test checks PASS; platform fmt and Clippy PASS; then the debug `ekp_platform` lib harness could not start (`os error 4551`, Application Control) and `scripts/check.ps1` printed `FAIL: platform test`.
- **Second canonical run (same command, same paths, no workaround):** the same pre-test checks passed with the same counts, then the same debug lib harness was denied (`os error 4551`) and the script printed `FAIL: platform test` again.

The canonical debug profile therefore has **no PASS** at `21e23d6`. The release-profile evidence above (308 executed tests) is behavioral coverage only. No binary was renamed, copied, moved or pointed at an alternate target directory, and the repository-level `target-dir` override or `EKP_PLATFORM_BIN` test override was not reintroduced.

After the E-033 documentation reconciliation, `scripts/check-docs.ps1` passes with 53 required documents and 680 relative links in 115 files, and `scripts/check-secrets.ps1` passes with 231 files. That is a documentation-only check, not a canonical run.

## E-034: multiline textarea, BPMN parser and Secure-cookie hardening (`279c919`)

```powershell
cd platform
cargo fmt -- --check
cargo clippy --release --locked --all-targets -- -D warnings
cargo test --release --locked
cargo run --release --locked --bin ekp_platform -- --selftest
cargo run --release --locked --bin ekp_platform -- --declared-rules-schema
cargo run --release --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
cargo run --release --locked --bin ekp_platform -- --health

cd ..\eval
cargo fmt -- --check
cargo clippy --release --locked --all-targets -- -D warnings
cargo test --release --locked
cargo run --release --locked -- --selftest
```

Observed on 2026-10-30 at `279c919471940e5a0406814ba78c1600bfb3c175` (code committed; these E-034 documentation edits were uncommitted during the run):

| Target | Count | Exit |
| --- | --- | --- |
| Platform unit (`--lib`) | 273 | 0 |
| Platform CLI contract | 31 | 0 |
| Platform edge cases | 2 | 0 |
| Platform governance web | 20 | 0 |
| Platform integration | 8 | 0 |
| Platform search evaluation | 7 | 0 |
| Platform web health | 4 | 0 |
| **Platform total** | **345** | 0 |
| Eval unit/integration tests | 12 | 0 |
| Eval self-test checks | 10 | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |
| `--health` | `health: OK` | 0 |

The three behaviours fixed in this revision were:

1. **Textarea input was rejected.** `validate_text` rejected every control character, including `\n`; a browser submits textarea newlines as CRLF. Workflow description, quality-case description and gate reason now use `validate_multiline_text`, which accepts LF and CRLF, rejects bare CR and every other control character, and stores CRLF normalized to LF via `clean_multiline`. Single-line fields still reject all line breaks.
2. **BPMN parsing was prefix-bound.** The parser only matched literal `<bpmn:...` tags and only double-quoted attributes. It now matches the local tag name (so a default namespace and arbitrary prefixes work), uses a quote-aware attribute scanner that accepts single and double quotes and whitespace around `=`, and skips comments, CDATA and processing instructions.
3. **Logout did not mirror `Secure`.** Login and logout now share `configured_secure_cookie_suffix`, so `EKPI_COOKIE_SECURE=1` emits `Secure` on both the session cookie and the clearing cookie.

Red/green evidence is deliberately limited: no pre-fix red run was captured for these three behaviours. The only observed red in this cycle was the new governance-web integration test failing because it hard-coded project `PRJ-000002`; creating the project before the workflow corrected the fixture, not the product expectation. The final release suite and both canonical runs are green. No test was deleted, renamed to hide a failure, or weakened, and no expectation was changed to match the implementation.

An initial canonical attempt immediately after the code commit failed at **eval Clippy**, not at project code: the stale debug build artifact `eval/target/debug/build/serde-*/out/private.rs` contained `use serde_flaps_core::...`, while the installed registry `serde` build script writes `serde_core` and no repository or registry source contains `serde_flaps_core`. Deleting only the `serde` build, fingerprint and dependency artifacts (source, tests and gates untouched) and rerunning the same command produced the canonical results below. This is recorded as an environment repair, not as a source PASS or a lowered gate. The same corruption had earlier been cleared in the release profile.

### Canonical entry-point result

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-10-30 at `279c919`, **exit 0 on two same-command runs**:

- **First canonical run:** documentation PASS (53 required documents, 680 links in 115 files); secrets PASS (231 files); eval fmt, Clippy, 12 tests and 10 self-test checks PASS; platform fmt, Clippy, **345 tests** PASS; platform self-test 49 assertions, schema 20 records, search 28 cases with gate PASS and `health: OK`; the script printed `=== ALL CHECKS PASSED ===`.
- **Second canonical run (same command, same paths, no workaround):** exited 0 with the same check set and counts; no step was skipped or reported as `BLOCKED`.

This is the first canonical PASS after the `c827351`, `bbe49eb`, `b2d8afc` and `21e23d6` host-policy blocks. It is a local working-tree run, not CI, staging, production or an independent execution. The host policy remains intermittent and has no named exception owner; a future denial must still be recorded as `BLOCKED`.

After the E-034 documentation reconciliation, `scripts/check-docs.ps1` passes with 53 required documents and 685 relative links in 115 files, and `scripts/check-secrets.ps1` passes with 231 files. That is a documentation-only check of the uncommitted evidence edits, not a third canonical run; the canonical PASS remains bound to the code revision `279c919`.

## E-035: LegalRuleML export and BPMN entity hardening (`eec11f2`)

```powershell
cd platform
cargo fmt --all
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked --bin ekp_platform -- --export-legalruleml <temp-dir>
```

The hardening was developed in an isolated git worktree on branch `audit/export-hardening`, based on `c0c0e33`. Only `platform/src/export.rs` and `platform/src/bpmn.rs` are in the code commit. The separate main worktree contains unrelated uncommitted 2026-10-30 changes and was neither used nor modified.

Observed on 2026-11-02 at `eec11f2a9a02827e50317fbd0d146a3092f89ccf` (code committed; these E-035 documentation edits were uncommitted during the canonical run):

| Target | Count | Exit |
| --- | --- | --- |
| Platform unit (`--lib`) | 282 | 0 |
| Platform CLI contract | 31 | 0 |
| Platform edge cases | 2 | 0 |
| Platform governance web | 20 | 0 |
| Platform integration | 8 | 0 |
| Platform search evaluation | 7 | 0 |
| Platform web health | 4 | 0 |
| **Platform total** | **354** | 0 |
| Eval unit/integration tests | 12 | 0 |
| Eval self-test checks | 10 | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |

The defects fixed in this revision were:

1. **Comment escaping could still produce invalid XML.** `esc_comment` replaced `--` with `- -` in a single pass. A run of three hyphens (`a---b`) became `a- --b`, which still contains `--`. The function now repeats until the comment body is stable, and the validator rejects any comment body containing `--` or ending with `-`.
2. **XML 1.0-invalid characters were accepted.** The validator did not check the XML `Char` production, so NUL, other C0 controls, `U+FFFE`/`U+FFFF` and similar code points could reach the export. `check_xml_chars` now rejects them before structural validation.
3. **Non-comment markup declarations were accepted.** The lightweight well-formedness scanner treated any `<!...>` as a comment-shaped region. It now accepts only `<!-- ... -->`; DOCTYPE and other declarations are refused.
4. **Impossible dates were accepted.** `date_in_force_time` checked only the `YYYY-MM-DD` shape, so `2026-02-30`, `2026-13-01` and non-leap `2023-02-29` passed. It now validates month and day, including the 4/100/400-year leap rule.
5. **Forbidden constructs were matched in text and missed when renamed.** `check_names` searched the raw body, so ordinary text containing `LegalSchema` or `IsActive` failed, while an element named `NormativeSources` escaped the exact-substring check. It now extracts element local names only, is prefix-insensitive, and refuses both the exact forbidden name and a longer local name beginning with it.
6. **IDs and references were scanned in text.** `check_ids_and_refs` searched byte patterns across the whole document, so text such as `key="ghost"` could register a key or satisfy a `keyref`. It now parses attributes only inside element tags, using quote-aware tag boundaries, and rejects malformed, unquoted, empty or non-`#` references.
7. **BPMN attributes did not decode XML entities.** `extract_attr` returned raw source text, so `id="T&amp;1"` stayed `T&amp;1` and did not match `sourceRef="T&amp;1"`. The parser now decodes the five predefined entities and decimal/hex numeric references. Unknown or malformed references remain visible rather than being dropped, and numeric references to XML 1.0-invalid characters are left verbatim.

Red/green evidence: seven new tests were observed failing before the fixes: `comment_escaping_never_leaves_a_double_hyphen`, `xml_10_invalid_characters_are_rejected`, `validator_rejects_double_hyphen_inside_comments`, `dates_must_be_real_calendar_days`, `forbidden_name_check_targets_elements_not_text`, `id_scanning_ignores_attribute_like_text` and `attributes_decode_xml_entities`. After the fixes, two further BPMN tests were added (`unknown_entities_are_left_visible`, `numeric_references_to_invalid_xml_characters_are_left_visible`) and the ID test was extended with a spoofed-reference negative case, for nine new test functions in total. No test was deleted, renamed to hide a failure, or weakened.

Export smoke: `cargo run --locked --bin ekp_platform -- --export-legalruleml <temp-dir>` wrote a 51,026-byte `legalruleml_v1_compact.xml`. An independent .NET `System.Xml.XmlDocument.Load` parsed it with root `lrml:LegalRuleML` and 718 elements. This is a local parser smoke check, not an additional repository gate or a standards conformance claim.

### Canonical entry-point result

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-11-02 at `eec11f2`, **exit 0**:

- Documentation PASS: 53 required documents, 682 relative links in 105 files.
- Secret patterns PASS: 231 files.
- Eval fmt, Clippy, 12 tests and 10 self-test checks PASS.
- Platform fmt, Clippy and **354 tests** PASS.
- Platform self-test 49 assertions, schema 20 records, search 28 cases with gate PASS, `=== ALL CHECKS PASSED ===`.

This is a local run in an isolated git worktree on the same managed host, not CI, staging, production, a clean clone or an independent execution. The branch has no remote and no pull request. A future denial must still be recorded as `BLOCKED`.

After the E-035 documentation reconciliation, `scripts/check-docs.ps1` passes with 53 required documents and 687 relative links in 105 files, and `scripts/check-secrets.ps1` passes with 231 files. That is a documentation-only check of the uncommitted evidence edits, not an additional canonical run; the canonical PASS remains bound to the code revision `eec11f2`.

## E-036: RAG citation confidence and date-year hardening (`d502147`)

```powershell
cd platform
cargo fmt --all
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The fixes were developed in the same isolated git worktree on branch `audit/export-hardening`, based on `c0c0e33`. Only `platform/src/rag.rs` and `platform/src/export.rs` are in the code commit. The separate main worktree contains unrelated uncommitted 2026-10-30 changes and was neither used nor modified.

Observed on 2026-11-02 at `d502147d3db70aefb108f2245c8e3232871225fe` (code committed; these E-036 documentation edits were uncommitted during the canonical run):

| Target | Count | Exit |
| --- | --- | --- |
| Platform unit (`--lib`) | 284 | 0 |
| Platform CLI contract | 31 | 0 |
| Platform edge cases | 2 | 0 |
| Platform governance web | 20 | 0 |
| Platform integration | 8 | 0 |
| Platform search evaluation | 7 | 0 |
| Platform web health | 4 | 0 |
| **Platform total** | **356** | 0 |
| Eval unit/integration tests | 12 | 0 |
| Eval self-test checks | 10 | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |

The defects fixed in this revision were:

1. **A citation disabled the RAG confidence gate.** The orchestrator set `has_citation` when any retrieved citation appeared in the model response and then skipped the `min_confidence` refusal whenever that flag was true. A response citing a retrieved item at `0.347` was therefore accepted even though `RagOrchestrator::new` uses `min_confidence = 0.5`; the citation acted as a fail-open switch. The refusal now applies to the evidence that was actually cited, regardless of the presence of a citation.
2. **The reported confidence came from the uncited top hit.** Even when the model cited a lower-ranked document, `confidence` was taken from `citations.first()`. The authoritative value is now the maximum score among the citations that appear in the response; an answer with no matching citation is refused. In the regression fixture the cited hit scored `0.676471` while the top hit scored `0.733359`; the answer now reports the cited value.
3. **`xs:dateTime` year `0000` was accepted.** `date_in_force_time` parsed the four-digit year but did not reject zero. XML Schema 1.0 has no year `0000` in the `xs:dateTime` lexical space. The exporter now refuses `0000-01-01`, and the boundary test confirms `0001-01-01` remains accepted as the minimum year.

Red/green evidence: `rag_refuses_cited_answer_below_min_confidence` was observed accepting the answer at `0.347` before the fix; `rag_confidence_uses_cited_evidence_not_the_top_hit` was observed reporting `0.733359` from the top hit instead of `0.676471` from the cited hit; and `dates_must_be_real_calendar_days` was observed accepting `0000-01-01` before the fix. All three pass afterwards, including the new `0001-01-01` boundary assertion. No test was deleted, renamed to hide a failure, or weakened.

The RAG provider in this repository is still `MockLlmProvider`, a deterministic test double. This evidence covers the orchestration contract (citation matching, threshold enforcement and reported confidence), not model quality, a real LLM integration or AI acceptance.

### Canonical entry-point result

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-11-02 at `d502147`, **exit 0** on two same-command runs; the second run captured `EXITCODE=0`:

- Documentation PASS: 53 required documents, 687 relative links in 105 files.
- Secret patterns PASS: 231 files.
- Eval fmt, Clippy, 12 tests and 10 self-test checks PASS.
- Platform fmt, Clippy and **356 tests** PASS.
- Platform self-test 49 assertions, schema 20 records, search 28 cases with gate PASS, `=== ALL CHECKS PASSED ===`.

This is a local run in an isolated git worktree on the same managed host, not CI, staging, production, a clean clone or an independent execution. The branch has no remote and no pull request. A future denial must still be recorded as `BLOCKED`.

After the E-036 documentation reconciliation, `scripts/check-docs.ps1` passes with 53 required documents and 691 relative links in 105 files, and `scripts/check-secrets.ps1` passes with 231 files. That is a documentation-only check of the uncommitted evidence edits, not an additional canonical run; the canonical PASS remains bound to the code revision `d502147`.

## E-037: second-precision deadlines and inclusive effective windows (`6ae475d`)

```powershell
cd platform
cargo fmt --all
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The fixes were developed in the same isolated git worktree on branch `audit/export-hardening`, based on `c0c0e33`. Only `platform/src/engine.rs`, `platform/src/docs.rs` and `platform/src/governance.rs` are in the code commit. The separate main worktree contains unrelated uncommitted 2026-10-30 changes and was neither used nor modified.

Observed on 2026-11-02 at `6ae475d8cb07fdce8f5fb864df6930767c81b852` (code committed; these E-037 documentation edits were uncommitted during the canonical run):

| Target | Count | Exit |
| --- | --- | --- |
| Platform unit (`--lib`) | 294 | 0 |
| Platform CLI contract | 31 | 0 |
| Platform edge cases | 2 | 0 |
| Platform governance web | 20 | 0 |
| Platform integration | 8 | 0 |
| Platform search evaluation | 7 | 0 |
| Platform web health | 4 | 0 |
| **Platform total** | **366** | 0 |
| Eval unit/integration tests | 12 | 0 |
| Eval self-test checks | 10 | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |

The defects fixed in this revision were:

1. **Sub-day deadlines were silently skipped.** `parse_iso_duration_days("PT12H")` returned `Some(0)`, and the engine filtered `days > 0`, so a 12-hour deadline was never evaluated and a one-second breach returned `Comply`. Durations are now parsed and compared in seconds; `PT12H` plus one second returns `Violate`, while an exact 12-hour timestamp remains `Comply` because the comparison is strict `>`.
2. **Malformed event timestamps were coerced to day zero.** `date_diff_days` split date strings, used `unwrap_or(0)` for every component and sliced raw bytes. A malformed value such as `2026-03-0~` therefore compared as a valid date and could produce `Comply` with no note. The engine now validates every event timestamp with `parse_utc_timestamp_seconds`; any unparseable value sets `invalid_event_timestamp`, skips process replay and yields `InsufficientEvidence`.
3. **The old date helper could panic on a multibyte boundary.** `&a[..a.len().min(10)]` can split a UTF-8 character. The strict parser rejects non-ASCII input before any byte slice, and `timestamp_parser_rejects_impossible_or_multibyte_values` covers `2026-03-0２`.
4. **Unsupported, missing or zero-length deadlines were fail-open.** `P1M`, `P1Y`, an empty string and `not-a-duration` previously produced only a note and could end in `Comply`; `P0D` was filtered out entirely. An unsupported or missing duration now makes the verdict `InsufficientEvidence`, while `P0D` is an immediate deadline: one second late violates, an identical timestamp complies. Empty anchor `ref` values are also rejected as `deadline_anchor_invalid` instead of being skipped.
5. **The `effective_to` boundary was wrong for timestamps with a time.** `"2026-02-28T23:59:59" <= "2026-02-28"` is false in string order, so an event on the closing day was treated as expired. Both `DocDatabase::in_force` and the engine's rule-temporal comparison now compare the validated calendar-date prefix, making `effective_to` inclusive through the end of that day while leaving the following day expired.
6. **Governance dates accepted year `0000`.** `validate_date` checked month/day ranges but not the year. XML Schema 1.0 has no year `0000`; `0000-01-01` is now rejected and `0001-01-01` remains accepted as the minimum.

Red/green evidence: five observations were red before the fixes. `deadline_subday_duration_is_enforced` observed `Comply` with notes `[]` for `PT12H` exceeded by one second; `malformed_event_timestamp_is_insufficient_evidence` observed `Comply` with notes `[]` for `2026-03-0~`; `date_validation_rejects_impossible_dates_and_accepts_leap_day` failed on `0000-01-01`; `effective_window_checks` observed `Ok(false)` for `2026-02-28T23:59:59`; and `rule_effective_to_is_inclusive_through_the_closing_day` observed `InsufficientEvidence` with `rule_expired_or_not_yet_effective`. All pass afterwards. Ten new unit tests and four extended tests cover the behavior, including the fail-closed duration/anchor guards that were added with the fix; no test was deleted, renamed to hide a failure, or weakened.

### Canonical entry-point result

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-11-02 at `6ae475d`, **exit 0** on one same-command run:

- Documentation PASS: 53 required documents, 691 relative links in 105 files.
- Secret patterns PASS: 231 files.
- Eval fmt, Clippy, 12 tests and 10 self-test checks PASS.
- Platform fmt, Clippy and **366 tests** PASS.
- Platform self-test 49 assertions, schema 20 records, search 28 cases with gate PASS, `=== ALL CHECKS PASSED ===`.

This is a local run in an isolated git worktree on the same managed host, not CI, staging, production, a clean clone or an independent execution. The branch has no remote and no pull request. A future denial must still be recorded as `BLOCKED`.

After the E-037 documentation reconciliation, `scripts/check-docs.ps1` passes with 53 required documents and 695 relative links in 105 files, and `scripts/check-secrets.ps1` passes with 231 files. That is a documentation-only check of the uncommitted evidence edits, not an additional canonical run; the canonical PASS remains bound to the code revision `6ae475d`.

## E-038: R2-7 eval corpus execution and κ study tooling (`028037a`)

```powershell
cd eval
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked -- --selftest
cargo run --locked -- --corpus-selftest
cargo run --locked -- --make-annotation-sheet annotation/blinded-cases.jsonl
cargo run --locked -- --corpus cases/corpus.json --out target/corpus-report
```

The change was developed in the isolated worktree on branch `audit/export-hardening`, based on `c0c0e33`. It touches `eval/src/types.rs`, `eval/src/annotation.rs`, `eval/src/main.rs`, `eval/cases/corpus.json`, `eval/annotation/`, `eval/README.md`, `scripts/check.ps1`, `scripts/check.sh`, `.github/workflows/ci.yml`, `docs/quality/ai-evaluation.md` and `docs/tech-selection-round1.md`. The separate main worktree contains unrelated uncommitted 2026-10-30 changes and was neither used nor modified.

Observed on 2026-11-03 at `028037a630a854506edc7e057ccb2c83d9275f92` (code committed; the worktree was clean when the canonical run started; the E-038 evidence edits were made after the run):

| Target | Count | Exit |
| --- | --- | --- |
| Eval unit tests | 20 (was 12) | 0 |
| Eval self-test checks | 10 | 0 |
| Eval corpus selftest checks | 16 | 0 |
| Fixed corpus cases | 67 (9 seed + 46 casebook + 12 edge) | 0 |
| Platform unit (`--lib`) | 294 | 0 |
| Platform CLI contract | 31 | 0 |
| Platform edge cases | 2 | 0 |
| Platform governance web | 20 | 0 |
| Platform integration | 8 | 0 |
| Platform search evaluation | 7 | 0 |
| Platform web health | 4 | 0 |
| **Platform total** | **366** | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |

What changed in this revision:

1. **The 67-case corpus is now executable, not just present.** `eval/cases/corpus.json` declares the three JSONL files and a 50-case minimum. `types::load_corpus` merges them, rejects duplicate case ids across files, rejects an unsupported manifest version, an empty file list and a corpus below the declared minimum. `--corpus-selftest` checks the 50-case minimum, three sources, all four verdict classes (`comply` 28, `violate` 11, `escalate` 10, `insufficient_evidence` 18), the 67-row blinded sheet, label-leakage fields and the three stub behaviors. It runs in the canonical `scripts/check.ps1` and `scripts/check.sh`.
2. **The annotation study is fail-closed.** `eval/src/annotation.rs` now loads JSONL annotation records and validates a study before computing κ: at least 50 cases, two different annotator ids, one id per file, identical case sets, unique case ids, known verdicts and a defined κ. `--kappa A.jsonl B.jsonl` exits `2` on invalid or missing input, `1` below the proposed 0.80 threshold and `0` only on a valid study at or above the threshold. A degenerate one-label study returns an error instead of a fabricated κ of 1.0.
3. **The blinded sheet is reviewable and checked.** `eval/annotation/blinded-cases.jsonl` has 67 rows with `case_id`, `task`, `doc_versions`, `events`, `rules` and `loop`. It contains no `expected_verdict`, `notes`, `citations`, `category` or `abstain_required`. `eval/annotation/README.md` defines independence, inputs, rubric, adjudication and the exact commands; `eval/annotation/annotation.schema.json` defines a completed record.
4. **CI is wired to the fixed entry point.** `scripts/check.ps1` and `check.sh` run `--corpus-selftest` and the workflow calls those scripts. The workflow also has `workflow_dispatch`. However, the repository has no remote or runner, so the workflow has never executed; `actionlint` exit 0 is static linting, not a CI run.
5. **Tests.** Eight new eval tests cover the shipped corpus, the 50-case minimum, same-annotator rejection, mismatched case sets, duplicate ids, unknown verdicts, a short study and an undefined κ. Eval tests increased from 12 to 20; no test was deleted, renamed to hide a failure, weakened or skipped.

Red/green evidence: before the change, `cargo run --locked -- --corpus-selftest`, `--kappa` and `--make-annotation-sheet` were unknown arguments and exited 2. The default `cargo run --locked -- --out target/red-baseline` report contained `"n_cases": 9` even though the repository held 67 cases. After the change, `--corpus-selftest` exits 0 with 67 cases and **16** checks, `--corpus` writes a 67-case report, and the annotation sheet has 67 rows. (An earlier version of this section said 17; a line-by-line recount of the `028037a` output found 16 `[PASS]` lines. E-039 records the corrected count and the 18-check output of the hardened revision.) A synthetic CLI smoke test with 60 records (5 deliberate disagreements) produced `cohens_kappa = 0.8666666666666667`, `passed = true`, exit 0; that is a tool smoke test, not an annotation study and not evidence for M-11. The fail-closed unit tests cover the rejection paths.

**Non-claims.** Inter-annotator agreement remains **NOT MEASURED**: 0 of 2 annotators are assigned, no `annotator-a.jsonl` or `annotator-b.jsonl` exists and no κ value is reported. The CI workflow remains **NEVER EXECUTED**: there is no remote, runner, run ID, branch protection or PR. The corpus is synthetic and the adapters are deterministic stubs, so this is not AI quality, user validation or pilot acceptance. R2-7 is therefore **partial**: the 50+ corpus is complete and executable, while the human κ study and the CI execution are still external blockers.

### Canonical entry-point result

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-11-03 at `028037a`, **exit 0** on one same-command run:

- Documentation PASS: 53 required documents, 695 relative links in 106 files.
- Secret patterns PASS: 235 files.
- Eval fmt, Clippy, 20 tests, 10 self-test checks and 16 corpus selftest checks PASS.
- Platform fmt, Clippy and **366 tests** PASS.
- Platform self-test 49 assertions, schema 20 records, search 28 cases with gate PASS, `=== ALL CHECKS PASSED ===`.

This is a local run in an isolated git worktree on the same managed host, not CI, staging, production, a clean clone or an independent execution. The branch has no remote and no pull request. A future denial must still be recorded as `BLOCKED`.

After the E-038 documentation edits, `scripts/check-docs.ps1` passed with 53 required documents and 700 relative links in 106 files, and `scripts/check-secrets.ps1` passed with 235 files. That is a documentation-only check of the uncommitted evidence edits, not an additional canonical run; the canonical PASS remains bound to the code revision `028037a`.

## E-039: eval CLI hardening and exact-corpus κ binding (`db7fc95`)

```powershell
cd eval
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked -- --selftest
cargo run --locked -- --corpus-selftest
cargo run --locked -- --kappa target/annotator-a.jsonl target/annotator-b.jsonl --corpus cases/corpus.json
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

`db7fc95` is the child of documentation-only `db691ca`; its code parent is `028037a`. The commit changed 10 files (647 insertions, 97 deletions): `eval/src/annotation.rs`, `eval/src/main.rs`, the new `eval/tests/cli_contract.rs`, and the eval/contract/quality/development documents that describe them.

Observed on 2026-11-04 at `db7fc95fcdec3496c1898e43967d6faab4cf0b45` (worktree clean when the canonical run started; evidence edits made after the run):

| Target | Count | Exit |
| --- | --- | --- |
| Eval unit tests | 27 | 0 |
| Eval CLI contract tests (real process) | 14 | 0 |
| **Eval Cargo tests total** | **41** (was 20) | 0 |
| Eval self-test checks | 10 | 0 |
| Eval corpus selftest checks | 18 (was 16) | 0 |
| Fixed corpus cases | 67 | 0 |
| Platform unit (`--lib`) | 294 | 0 |
| Platform CLI contract | 31 | 0 |
| Platform edge cases | 2 | 0 |
| Platform governance web | 20 | 0 |
| Platform integration | 8 | 0 |
| Platform search evaluation | 7 | 0 |
| Platform web health | 4 | 0 |
| **Platform total** | **366** | 0 |
| Platform self-test assertions | 49 | 0 |
| Declared-rules schema records | 20 | 0 |
| Search gate cases | 28 (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations) | 0 |

What changed in this revision:

1. **Fail-closed CLI parsing.** `--selftest` and `--corpus-selftest` are exclusive single-argument modes. `--cases` and `--corpus` are mutually exclusive. `--systems` must select at least one adapter and must not repeat one. Duplicate flags and flag-shaped values (for example `--out --selftest`) exit `2` before any report is written. Each accepted flag consumes exactly one value.
2. **κ studies are bound to the fixed corpus.** `--kappa A.jsonl B.jsonl [--corpus PATH]` loads `cases/corpus.json` by default and calls `evaluate_study_for_corpus`. The two annotation files must cover every fixed-corpus case id exactly once. A subset, an extra case or an invented id exits `2`; the result cannot silently shrink to whatever the annotators happened to label. The JSON output now records `corpus_id` and `corpus_cases`, so a result names the frozen corpus it belongs to.
3. **Strict annotation records.** `Annotation` uses `serde(deny_unknown_fields)`. A file containing `expected_verdict`, `notes` or any other extra field is rejected instead of being accepted while the label leaks into the study.
4. **Corrected interpretation bands.** Landis–Koch bands are inclusive at the upper bound: `0.80` is `Substantial` and `0.81+` is `Almost perfect`; non-finite κ is `Undefined`. This prevents the proposed `0.80` acceptance threshold from being reported with the stronger label.
5. **Blinded-sheet integrity.** `--corpus-selftest` now also requires unique sheet rows and rejects prefilled `verdict` or `annotator` values, in addition to the existing row-count, case-id and forbidden-field checks. The sheet itself remains 67 rows with null labels.
6. **Real-process tests.** The new `eval/tests/cli_contract.rs` spawns `CARGO_BIN_EXE_ekp_eval` for 14 tests: self-test exclusivity, unknown flags with no report write, `--cases`/`--corpus` exclusion, empty and duplicate adapter selections, κ argument errors, exact-corpus success, a missing case, a foreign id, the same annotator id, a leaked `expected_verdict`, blinded-sheet generation and sheet-mode refusal.

Red/green evidence. The red observations were taken from a detached checkout of the preceding code revision `028037a`, then the same commands were re-run on `db7fc95`:

| Observation | `028037a` | `db7fc95` |
| --- | --- | --- |
| `--selftest --bogus` | exit 0 | exit 2 |
| `--corpus-selftest --bogus` | not rejected as a mode conflict | exit 2 |
| `--systems ","` | exit 0 and wrote a report | exit 2, no report |
| `--kappa` with 60 invented case ids | exit 0, `cohens_kappa = 1.0`, `PASS` | exit 2 |
| `--corpus-selftest` PASS lines | 16 | 18 |

The invented-case κ observation is the important one: the old command computed a perfect score for a study that did not cover the corpus at all. The new command rejects it before reporting a κ value. A degenerate one-label study is still an error (`Undefined`), never a fabricated 1.0.

### Canonical entry-point result

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/check.ps1
```

Observed on 2026-11-04 at `db7fc95`, **exit 0** on one same-command run (log: `canonical-db7fc95.log`):

- Documentation PASS: 53 required documents, 700 relative links in 106 files.
- Secret patterns PASS: 236 files.
- Eval fmt, Clippy, 41 tests (27 unit + 14 CLI contract), 10 self-test checks and 18 corpus selftest checks PASS.
- Platform fmt, Clippy and **366 tests** PASS.
- Platform self-test 49 assertions, schema 20 records, search 28 cases with gate PASS, `=== ALL CHECKS PASSED ===`.

This is a local run in an isolated git worktree on the same managed host, not CI, staging, production, a clean clone or an independent execution. The branch has no remote and no pull request. A future denial must still be recorded as `BLOCKED`.

**Non-claims.** The hardening proves that the tooling rejects the specific fail-open paths above; it does not produce inter-annotator agreement. κ remains **NOT MEASURED**: 0 of 2 annotators are assigned, no completed annotation files exist and no κ value is reported. The CI workflow remains **NEVER EXECUTED**: there is no remote, runner, run ID or branch protection. The corpus is synthetic and the adapters are deterministic stubs, so this is not AI quality, user validation or pilot acceptance. R2-7 remains **partial**: the 50+ corpus is complete and executable, while the human κ study and CI execution are external blockers.

## Reproduction notes

1. Run commands from the repository root unless the command shows `cd`.
2. Use `powershell -ExecutionPolicy Bypass -File scripts/...` on Windows; the scripts are not signed.
3. The host's Application Control policy can block newly built executables. If `os error 4551` appears, record `BLOCKED`; do not retry with a lowered gate.
4. Release-profile results do not replace the canonical entry point. Keep both rows.
5. Do not commit raw logs containing local absolute paths, usernames, secrets or personal data.
6. Re-run the affected evidence after any code, dependency, data, host-policy or deployment change.

## What this evidence does not show

- It does not show a CI, staging or production run.
- It does not show a real user, pilot acceptance or independent review.
- It does not show enterprise authentication, tenant isolation, off-host backup, RPO/RTO or disaster recovery.
- It does not show AI/LLM quality; the only model path is a labeled mock.
- It does not show that the canonical check is reproducible on a different host or in CI. The current `db7fc95` worktree passed once on the same host (E-039); the preceding `028037a` worktree passed once (E-038; its corpus-check count is 16, corrected from the earlier 17); the `6ae475d` worktree passed once (E-037); the `d502147` worktree passed twice (E-036); the earlier `eec11f2` worktree passed once (E-035); the earlier `279c919` working tree passed twice (E-034). The earlier `41beaea` working-tree run was blocked, later working-tree runs at `4697aa1`, `718db57`, `32f2b36` and `604a2a5` passed, the `c827351` run was blocked at the `edge_cases` harness (`os error 4551`), `bbe49eb` at docs HEAD `17df06a` was blocked at the debug CLI binary (E-031), `b2d8afc` had 2 edge tests blocked at spawn and its canonical entry point blocked at the debug lib and `cli_contract` harnesses (E-032), and `21e23d6` had 27 tests blocked across three harnesses and its canonical entry point blocked on two same-command attempts (E-033). There is still no CI run; the defined workflow has never executed.
- It does not show LegalRuleML 1.0 conformance beyond the repository validator and one local .NET parser smoke check; no external schema validator, conformance suite or independent review was run.
- It does not show that the earlier `28b68cc` revision can be restored and checked; that row remains `BLOCKED`.
