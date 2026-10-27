# Test and execution evidence

- **Status:** Active; local evidence only, with named blocks
- **Owner:** QA + Engineering
- **Last reviewed:** 2026-10-27
- **Repository revision for the re-run:** code baseline `6ae61348194e2daa63a6ac53a31db00771d2a5e1`; assurance-document commit `947cd27f19ba124aa0f5854a25f1e2a4a910ac97` (`main`)
- **Working tree during the re-run:** clean at each named revision; `947cd27` differs from `6ae6134` only in documentation and the required-document list, so the code-level checks remain attributed to `6ae6134`
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
| E-008 | HISTORICAL LOCAL | Last verified restore | `3ee2ee2` | 0 | PASS: fresh clone; restored `scripts/check.ps1` exit 0 |
| E-009 | SIMULATED | `age` encrypted backup drill | `56a944f` | 0 | PASS for encryption/decryption/byte comparison; destination class `local-simulated-off-host` |
| E-010 | HISTORICAL LOCAL | SBOM generation | `a826472` | 0 | platform 67 components; eval 11 components; hashes recorded |
| E-011 | HISTORICAL LOCAL | Full gate `gate.ps1 -IncludeSmoke` | `b2bae3e` | 0 | 13/13 PASS (historical; not current) |
| E-012 | HISTORICAL LOCAL | Release benchmark | `56a944f` / `b2bae3e` | 0 | eval p50/p95 17.432/23.319 ms; loopback p50/p95 19.010/24.069 ms; 5,369.21 req/s |
| E-013 | NOT RUN | CI, PR, branch protection | n/a | n/a | `BLOCKED`: no remote, no CI execution, no branch protection |
| E-014 | NOT RUN | Staging/production deployment | n/a | n/a | `BLOCKED`: no approved environment or deployment target |
| E-015 | NOT RUN | Independent review / pilot | n/a | n/a | `BLOCKED`: no independent reviewer, pilot owner or representative user |
| E-016 | LOCAL | Documentation check (assurance commit) | `947cd27` | 0 | PASS: 53 required documents, 652 links in 115 files |
| E-017 | LOCAL | Secret pattern scan (assurance commit) | `947cd27` | 0 | PASS: 229 files |

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

This is **not** a restore PASS. The last verified restore is E-008.

## E-008: last verified restore (`3ee2ee2`)

- Bundle: `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-27-125701.bundle`
- SHA-256: `7E21EEEE456F36DE79056508BE718705D6294525D98E39C5F1C80C585C73FE19`
- `git bundle verify`: okay; HEAD `3ee2ee2`
- Automatic `%TEMP%` drill: `BLOCKED` at `edge_cases` (`os error 4551`)
- Manual fresh clone: `%USERPROFILE%\EKPI-restore-drill-3ee2ee2`; restored `scripts/check.ps1` exit 0
- Restored check result: docs 503 links in 96 tracked files; secrets 220; eval 12; platform 261 (217 unit, 13 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health); self-test 49; schema 20; search 28 (20/20, 8/8, 4 role, 0 violations)

This is the last restore-verified local revision. It does not transfer to `28b68cc`.

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

`gate.ps1 -IncludeSmoke` at `b2bae3e` (2026-10-27) reported 13/13 PASS, including docs, secrets, both crates, schema, search and smoke/export. This is historical. It is not a current `6ae6134` pass and does not clear the current canonical block. The release profile at `23b8996` and `6ae6134` is the current behavioral evidence.

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
- It does not show that the final `28b68cc` revision can be restored and checked on this host.
