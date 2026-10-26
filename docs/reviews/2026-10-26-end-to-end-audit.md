# End-to-end audit — 2026-10-26

- **Status:** Audit complete; follow-up fixes verified at `7f6ab97`; Phase A **not approved**, Phase B **blocked**
- **Owner:** Engineering + QA
- **Scope:** repository content, served surfaces, contracts, tests, documentation, backup/restore evidence, phase-gate claims
- **Revision at audit start:** `dd0dd74` (findings checked against the working tree derived from it); follow-up verified at `7f6ab97`
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
| F-01 | High | `permission isolation 1.00` on the fixed search set was presented as permission evidence, but no case declares `roles` and every fixture document has `permissions: []`. The metric is 1.00 by construction. | Fixed the claim in architecture, backend, contracts, quality, threat model and register. Added `EVAL-GAP-06` and `IMP-009`. **Follow-up at `7f6ab97`:** two evaluation-only restricted fixtures and four role cases (2 allowed, 2 denied, 0 violations) give the metric a real denominator; `EVAL-GAP-06` closed as fixed-set coverage. |
| F-02 | High | The CLI silently ignored unknown flags and treated a trailing `--out`/`--threshold` as “use the default”, contradicting the published exit-code contract. `--out ... --bogus` exited 0. | Fixed in `platform/src/main.rs` (`validate_args`); added `platform/tests/cli_contract.rs` (13 tests). All rejection paths now exit `2` before writing. |
| F-03 | High | The canonical `cargo test --locked` and `gate.ps1` could not run on this host. Windows Application Control blocked specific binaries under `target` with `os error 4551`, including test harnesses and the `maud_macros`/`tokio_macros` DLLs. | Recorded as **Blocked**, not PASS; A4 was `Partial` at the audit revision. **Follow-up at `7f6ab97`:** the committed `.cargo/config.toml` profile overrides produce executable artifacts; canonical `cargo test --locked`, Clippy, fmt and `gate.ps1 -IncludeSmoke` (13/13) passed. A4 is now `Met`; CI still has not run. |
| F-04 | High | The latest backup restore drill (`198d5f3`) is incomplete because the restored build was blocked by the same policy. | Recorded as `BLOCKED`; that row remains blocked and is not a pass. **Follow-up at `7f6ab97`:** new bundle SHA-256 `4D35413B…3192B276`, `git bundle verify` okay, fresh clone reached `7f6ab97`, restored `scripts/check.ps1` exit 0. A8 is now `Met` (local-only); PR-07/S3/S5 are verified for this revision, while off-host/RPO/RTO remain Phase B. |
| F-05 | Medium | The architecture and backend docs described `evidence` and `causal` as served capabilities. In fact `evidence` is unreferenced and `causal_results` is stored but never consumed. `tenant` is reached only by a library conversion. | Corrected the module table and added precise boundaries. |
| F-06 | Medium | The troubleshooting page advised “run `cargo test`” for `os error 4551` and suggested disabling the policy, which does not describe the observed behaviour and can violate host policy. | Rewrote the section with the observed hash/path behaviour, safe rebuild diagnostics, `EKP_PLATFORM_BIN` test override and the rule that a workaround is not a gate PASS. |
| F-07 | Medium | `docs/research-framework-v1.md` contains target-state language (WORM, manifest hash chains) that could be mistaken for current capability. | Marked the document historical/target-state and added it to the historical list in `docs/README.md`. |
| F-08 | Medium | Traceability rows S15/S18/S19/S20 and the quality page used stale revisions, test counts and the unqualified permission claim. | Updated after the audit commit. **Follow-up at `7f6ab97`:** S15/S18/S20 record 233 tests, 28 search cases and the canonical gate; S19 remains `Partial` because the role filter is library/eval evidence, not authentication or tenant isolation. |
| F-09 | Medium | No remote, authenticated `gh`, GitHub Issues, branch protection or CI execution exist. Issue templates and workflows are files only. | Remains **Blocked** (`IMP-005`, `S10`); not represented as done. |
| F-10 | Low | Historical release rows contain older counts (183/212 tests). They are accurate for those revisions but can be misread as current. | Kept as historical rows; current rows and the quality page now state the audit result and the blocked canonical gate. |
| F-11 | Low | Document link counts (445) and secret-scan counts (200) were stale after the audit edits. | Re-ran the checks. **Final `7f6ab97`:** 44 required files, 488 links in 95 files and 205 scanned files. |

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
| Canonical `scripts/check.ps1` | **Blocked** at `31fa5a3`: docs, secret scan, eval fmt/clippy/tests/selftest and platform fmt passed; platform Clippy stopped with `can't find crate for axum/maud` after Application Control denied the proc-macro DLLs | Same host restriction; not a code failure and not a PASS |

## Follow-up evidence (`7f6ab97`, 2026-10-26)

| Check | Result | Limitation |
| --- | --- | --- |
| Canonical platform `cargo test --locked` | 233 passed (200 unit, 2 edge, 3 web, 8 integration, 7 search evaluation, 13 CLI contract) | Local host only |
| Canonical eval `cargo test --locked` | 9 passed | Local host only |
| Canonical fmt/Clippy (both crates) | Exit 0 with warnings denied | Local host only |
| `gate.ps1 -IncludeSmoke` | 13/13 PASS; docs 44 required/488 links in 95 files; secrets 205 files | No CI execution |
| Search gate | 28 cases; 20/20 top-1; 8/8 refusals; 4 role-restricted (2 allowed, 2 denied); 0 violations; isolation 1.00 | Lexical library filter only; not authentication/tenant isolation |
| Smoke | `G_ekpi_engine acc=1.000`; H-1–H-6 OK; LegalRuleML 51,026 bytes 19/19/19/16; zero-script HTML | Local synthetic data |
| Restore drill | PASS; bundle `ekpi-platform-2026-10-26-105233.bundle`, SHA-256 `4D35413B…3192B276`; `git bundle verify` okay; fresh clone `7f6ab97` ran restored `scripts/check.ps1` exit 0 | Local bundle only; no off-host copy or approved RPO/RTO |
| CI/remote | Not run | No remote and `gh` is not authenticated |

## Decision

- The repository is **not approved for Phase A**. At `7f6ab97`, A4 (canonical engineering gate) and A8 (local restore drill) are `Met`; A1 (real user value) is `Not met`, and A7 plus the pilot-scope parts of A2/A3/A5/A6/A10 remain `Partial`.
- Phase B remains **blocked** by the missing remote/CI, named owners, enterprise identity, off-host backup/RPO/RTO and an approved deployment target.
- The code and documentation defects that could be fixed locally were fixed and verified at `7f6ab97`. Remaining items are tracked as `IMP-005`, `IMP-006`, `IMP-008`, `IMP-011` and `IMP-012`; the audit-time `IMP-009` and `IMP-010` are done.
- The audit-time copied-harness results remain diagnostic only. The `7f6ab97` canonical gate and restore drill are separate, clean evidence; CI must still run when a remote exists.

## Update triggers

Update this report when the canonical gate or restore drill is rerun, when `EVAL-GAP-06` is closed (closed 2026-10-26), when CI/remote access is provisioned, or when a new audit changes a phase-gate status.
