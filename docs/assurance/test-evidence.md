# Test and execution evidence

- **Status:** Active; local evidence only, with named blocks
- **Owner:** QA + Engineering
- **Last reviewed:** 2026-10-29
- **Repository revision for the re-run:** current hardening revision `c827351bc0ecf1db589d58b1b57b068360d12195`; code baseline `6ae61348194e2daa63a6ac53a31db00771d2a5e1`; assurance-document commit `947cd27f19ba124aa0f5854a25f1e2a4a910ac97`; evidence-record commit `a2ba7de34aae351a9507a052e4f1c1280659e070`; first final bundle revision `41beaea3ab2157721f6ac60b08c097bafaedd0dd`; final bundle revision `6ed236dc4769b3d7581608df1a325b42f46f2004` (`main`)
- **Working tree during the re-run:** clean at each named revision; `947cd27`, `a2ba7de`, `41beaea` and `6ed236d` are documentation/evidence commits relative to `6ae6134`, so the code-level checks remain attributed to `6ae6134` except where a clean-clone canonical run is explicitly recorded
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
| E-029 | LOCAL | Hardening re-run: canonical check, individual targets and health contract | `c827351` | 1 | **BLOCKED** at platform `cargo test --locked`: `edge_cases` cannot start (`os error 4551`, Application Control). Individual targets: 250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation and 4 web health PASS; 2 edge tests NOT RUN. Docs 53/667/115, secrets 231, eval 12 + 10 self-test checks, platform self-test 49, schema 20, search 28 PASS |

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

`gate.ps1 -IncludeSmoke` at `b2bae3e` (2026-10-27) reported 13/13 PASS, including docs, secrets, both crates, schema, search and smoke/export. This is historical. It is not a current `6ae6134` pass and does not clear the current canonical block. At the current `c827351` revision, the behavioral evidence is the 332 individually executed platform tests plus 49 self-test assertions, schema 20 and search 28; the canonical check remains Blocked at `edge_cases` (E-029). The release profile at `23b8996` and `6ae6134` is historical behavioral evidence, not a current canonical PASS.

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

Observed on 2026-10-29 at `c827351bc0ecf1db589d58b1b57b068360d12195` (working tree clean for tracked files; unrelated untracked `.opencode` skill packs were not part of the revision):

- Canonical exit **1**, stopped at the platform `cargo test --locked` step. The `edge_cases` harness could not start: `平台應用程式控制原則已封鎖此檔案 (os error 4551)` / Application Control policy `{0283ac0f-fff1-49ae-ada1-8a933130cad6}`. The path was `platform\target\debug\deps\edge_cases-111f9e750cab1c27.exe`. The final re-run after the 2026-10-29 documentation updates is saved at `%TEMP%\ekpi-canonical-20261029-094042.log`.
- The canonical run completed the checks before that point: docs 53 required/667 links in 115 files PASS; secrets 231 files PASS; eval fmt, clippy, 12 tests and 10 self-test checks PASS; platform fmt and clippy PASS; platform unit tests 250 PASS; CLI contract tests 28 PASS.
- The blocked `edge_cases` target was retried nine times over two intervals and also invoked directly with `--list`; every attempt was denied. Its two tests are **NOT RUN** and are not counted as passes.
- The remaining platform targets were run individually and exited 0: unit **250**, CLI contract **28**, governance web **15**, integration **8**, search evaluation **7**, web health **4**. Total executed and passing: **332** (250 + 28 + 15 + 8 + 7 + 4), excluding the two blocked edge tests. This is not a substitute for a canonical PASS.
- Platform `--selftest` exited 0 with 49 assertions; `--declared-rules-schema` exited 0 with 20 records; `--eval --test-set data/eval/search_eval.json` exited 0 with 28 cases, 20/20 top-1, 8/8 refusals, 4 role-restricted cases (2 allowed, 2 denied), 0 violations; `--health` exited 0 with `health: OK`.
- The change adds four CLI contract tests for `--health`: OK on the default data, DEGRADED with missing core files, rejection of an unknown flag before any status output, and rejection when combined with `--selftest`.
- The committed absolute `target-dir` override and the `EKP_PLATFORM_BIN` test override are removed. Tests now use only `CARGO_BIN_EXE_ekp_platform` in the crate's canonical target directory. The two crates declare `[workspace]` so an unrelated parent manifest cannot capture them.
- Because the exact canonical command did not exit 0, A4 for `c827351` is **Blocked**, not `Met`. The earlier `604a2a5` working-tree PASS (E-028) and `6ed236d` clean-clone PASS (E-023) remain historical evidence for those revisions only.
- This is local evidence. It is not CI, staging, production, off-host or independent review. No test, gate or policy was lowered to obtain it.

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
- It does not show that the canonical check is reproducible on a different host or in CI; the earlier `41beaea` working-tree run was blocked, later working-tree runs at `4697aa1`, `718db57`, `32f2b36` and `604a2a5` passed, and the current `c827351` run is blocked at the `edge_cases` harness (`os error 4551`) even though the other six platform targets pass (E-029). There is still no CI run.
- It does not show that the earlier `28b68cc` revision can be restored and checked; that row remains `BLOCKED`.
