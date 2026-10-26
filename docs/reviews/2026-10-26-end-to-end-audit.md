# End-to-end audit — 2026-10-26

- **Status:** Complete for the audited working tree; release gate remains **Blocked**
- **Owner:** Engineering + QA
- **Scope:** repository content, served surfaces, contracts, tests, documentation, backup/restore evidence, phase-gate claims
- **Revision at audit start:** `dd0dd74` (all findings below were checked against the working tree derived from it)
- **Related:** [Traceability register](../traceability/register.md), [Phase gates](../quality/phase-gates.md), [Improvement register](README.md#improvement-register), [Backup log](../operations/backup-log.md)

## Method

The audit did not stop at re-reading documents. For each claim it tried to produce a counter-example or a command result:

1. **Code/wiring audit:** traced every module to a binary or request path (`main.rs`, `governance_web.rs`) and marked unreferenced or unread modules as library-only.
2. **Contract audit:** exercised the real CLI exit-code matrix, including unknown flags, missing/empty values, flag-as-value, positional arguments and incompatible modes.
3. **Test audit:** executed the unit, edge, web, integration, search-eval and CLI-contract suites; compared counts with the documents.
4. **Data audit:** counted the 67-case verdict corpus, the 24-case search set, 20 rule records and 7 documents/10 versions; checked role and permission fields.
5. **Documentation audit:** checked required files, relative links, fragments, stale counts, target/historical wording and environment-variable claims.
6. **Release audit:** re-verified backup hashes and `git bundle verify`, and attempted the restore drill.
7. **Claims audit:** searched for production-ready, secure, tenant-isolated, compliant, WORM, permission-aware, 212/183-test and “PASS” claims that exceeded the evidence.

## Findings

| ID | Severity | Finding | Disposition |
| --- | --- | --- | --- |
| F-01 | High | `permission isolation 1.00` on the fixed search set was presented as permission evidence, but no case declares `roles` and every fixture document has `permissions: []`. The metric is 1.00 by construction. | Fixed the claim in architecture, backend, contracts, quality, threat model and register. Added `EVAL-GAP-06` and `IMP-009`; the missing role-restricted fixture remains open. |
| F-02 | High | The CLI silently ignored unknown flags and treated a trailing `--out`/`--threshold` as “use the default”, contradicting the published exit-code contract. `--out ... --bogus` exited 0. | Fixed in `platform/src/main.rs` (`validate_args`); added `platform/tests/cli_contract.rs` (13 tests). All rejection paths now exit `2` before writing. |
| F-03 | High | The canonical `cargo test --locked` and `gate.ps1` could not run on this host. Windows Application Control blocked specific binaries under `target` with `os error 4551`, including test harnesses and the `maud_macros`/`tokio_macros` DLLs. | Recorded as **Blocked**, not PASS. A4 was downgraded from `Met` to `Partial`. The suites were run from copied harnesses and crate-local profile overrides; those results are diagnostic, not a canonical gate. |
| F-04 | High | The latest backup restore drill (`198d5f3`) is incomplete because the restored build was blocked by the same policy. | Already recorded as `BLOCKED`; confirmed by re-checking the bundle hash and log. A8/PR-07/S3/S5 remain `Partial`. No new PASS claim. |
| F-05 | Medium | The architecture and backend docs described `evidence` and `causal` as served capabilities. In fact `evidence` is unreferenced and `causal_results` is stored but never consumed. `tenant` is reached only by a library conversion. | Corrected the module table and added precise boundaries. |
| F-06 | Medium | The troubleshooting page advised “run `cargo test`” for `os error 4551` and suggested disabling the policy, which does not describe the observed behaviour and can violate host policy. | Rewrote the section with the observed hash/path behaviour, safe rebuild diagnostics, `EKP_PLATFORM_BIN` test override and the rule that a workaround is not a gate PASS. |
| F-07 | Medium | `docs/research-framework-v1.md` contains target-state language (WORM, manifest hash chains) that could be mistaken for current capability. | Marked the document historical/target-state and added it to the historical list in `docs/README.md`. |
| F-08 | Medium | Traceability rows S15/S18/S19/S20 and the quality page used stale revisions, test counts and the unqualified permission claim. | Updated after the audit commit; S19 is now `Partial` because the permission evidence is library-only and the fixed set does not exercise roles. |
| F-09 | Medium | No remote, authenticated `gh`, GitHub Issues, branch protection or CI execution exist. Issue templates and workflows are files only. | Remains **Blocked** (`IMP-005`, `S10`); not represented as done. |
| F-10 | Low | Historical release rows contain older counts (183/212 tests). They are accurate for those revisions but can be misread as current. | Kept as historical rows; current rows and the quality page now state the audit result and the blocked canonical gate. |
| F-11 | Low | Document link counts (445) and secret-scan counts (200) were stale after the audit edits. | Re-ran the checks; current working tree is 44 required files, 452 links in 92 files and 201 scanned files. |

## Verification evidence

| Check | Result | Limitation |
| --- | --- | --- |
| `cargo fmt -- --check` (platform, eval) | Exit 0 | Canonical command |
| `cargo clippy --locked --all-targets -- -D warnings` (eval) | Exit 0 | Canonical command |
| `cargo clippy --locked --all-targets` (platform) | Exit 0 | Required crate-local profile overrides for two blocked proc-macros; not the canonical command line |
| `cargo test --locked` (eval) | 9 passed, 0 failed | Canonical command |
| Platform unit tests | 193 passed | Copied harness; canonical run blocked |
| Platform edge cases | 2 passed | Copied harness |
| Governance web tests | 3 passed | Copied harness |
| Engine integration tests | 8 passed | Copied harness |
| Search evaluation tests | 6 passed | Copied harness with `EKP_PLATFORM_BIN` |
| CLI contract tests | 13 passed | Copied harness with `EKP_PLATFORM_BIN` |
| `--selftest` (platform) | 49 assertions, exit 0 | Copied CLI binary |
| `--declared-rules-schema` | 20 records clean, exit 0 | Copied CLI binary |
| `--eval --test-set data/eval/search_eval.json` | 24 cases; 18/18 top-1; 6/6 refusals; gate PASS | Lexical only; permission metric vacuous |
| `scripts/check-docs.ps1` / `.sh` | 44 required; 457 links in 93 files; exit 0 | Working tree |
| `scripts/check-secrets.ps1` / `.sh` | 202 files; exit 0 | High-signal patterns only; not a DLP scan |
| Bundle `198d5f3` | `git bundle verify` okay; SHA-256 `E6EAB351…D903847`; fresh clone reached HEAD | Restore check **BLOCKED** |
| Canonical `scripts/check.ps1` / `gate.ps1` | **Blocked** | Application Control denies execution from `target` |

## Decision

- The repository is **not approved for Phase A**. A1 (real user value), A4 (canonical engineering gate for this revision), A7 (complete traceability) and A8 (latest restore) are not met.
- Phase B remains **blocked** by the missing remote/CI, named owners, enterprise identity and an approved deployment target.
- The code and documentation defects that could be fixed locally were fixed. The remaining items are tracked as `IMP-005`, `IMP-006`, `IMP-008`, `IMP-009` and the blocked restore/CI items.
- Do not cite the copied-harness results as a release PASS. Re-run the canonical gate and the restore drill on an approved host or CI, then update this report and the register with the new revision.

## Update triggers

Update this report when the canonical gate or restore drill is rerun, when `EVAL-GAP-06` is closed, when CI/remote access is provisioned, or when a new audit changes a phase-gate status.
