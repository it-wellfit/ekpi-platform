# EKPI Final Validation Report

> **Historical snapshot (2026-10-25). Not current evidence.**
>
> This report is retained for the record. It contains statements that were later found to be
> inaccurate or superseded: S5 claimed completion before a restore drill was executed, the
> validation-status section contains an intermediate “tests blocked” result, and the test
> counts do not match the final run. Do not cite this file for readiness, test counts or
> restore evidence. Current evidence: [regression checklist](../development/regression-checklist.md),
> the gate output for the revision being delivered, and the [traceability register](../traceability/register.md).
> For corrections and current status, see the note at the end of this file.

> Generated as part of MASTER PROMPT v2.0 full completion. Records the outcome of the
> repository-resident quality-gates baseline and the operational/validation slices
> (S1–S11). All commands run offline from the repo root; no remote or cloud service
> is required.

## Validation environment

- Date: 2026-10-25
- Toolchain: Rust stable (rust-toolchain.toml), cargo
- Platform: Windows PowerShell 5.1 (scripts/check.ps1) — POSIX equivalent scripts/check.sh
- Crates: `eval/` (`ekp_eval`), `platform/` (`ekp_platform`) — independent, no root workspace

## Baseline validation result

`scripts/check.ps1` (and equivalent `scripts/check.sh`) ran end-to-end: **ALL CHECKS PASSED**.

### Eval crate (`ekp_eval`)

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt -- --check` | PASS |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` | PASS (zero warnings) |
| Tests | `cargo test --locked` | PASS (5 tests: benchmark, failure_cases, metrics x3) |
| Selftest | `cargo run --locked -- --selftest` | PASS (10 checks: oracle/allow_all/refuse_all gates) |

Selftest highlights: oracle accuracy 1.0, wrong_allow 0, hard gate tripped on allow_all, refuse_all correct_refusal 1.0.

### Platform crate (`ekp_platform`)

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt -- --check` | PASS |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` | PASS (zero warnings) |
| Tests | `cargo test --locked` | PASS (143 unit + 2 edge_case + 3 governance_web + 8 integration = 156) |
| Selftest | `cargo run --locked --bin ekp_platform -- --selftest` | PASS (48 assertions, 9/9 cases) |
| Schema | `cargo run --locked --bin ekp_platform -- --declared-rules-schema` | PASS (20 records validated clean: 19 active + 1 draft_ai) |

### Hard gates (engine `--out` audit, `engine_hard_gates.json`)

| Gate | Requirement | Result |
|---|---|---|
| H-1 | Forbidden state-transition release rate = 0 | OK |
| H-2 | Actual violations judged comply = 0 | OK |
| H-3 | Evidence-insufficient judged comply = 0 | OK |
| H-4 | Every definitive conclusion ≥1 resolvable anchor | OK |
| H-5 | Injection does not change verdict | OK |
| H-6 | Replay consistency | OK |

## Slice completion summary

| Slice | Item | Location | Status |
|---|---|---|---|
| S1 | Measurable spec & target register | `docs/metrics/targets.md` | complete |
| S2 | Metrics instrumentation (unwarranted refusal, manual handoff, completion, error, anchor resolution) | `eval/src/metrics.rs`, `eval/src/report.rs` | complete |
| S3 | Local-first workflow | `docs/operations/local-first-workflow.md` | complete |
| S4 | Secrets & sensitive data policy | `docs/operations/secrets-policy.md`, `.env.example` | complete |
| S5 | Local backup procedure + log | `docs/operations/local-backup.md`, `docs/operations/backup-log.md` | procedure written; **restore drill NOT executed** (historical claim corrected) |
| S6 | Edge cases (12) + 50+ corpus regression test | `eval/cases/edge_cases.jsonl`, `platform/tests/edge_cases.rs` | complete (corpus = 67 ≥ 50) |
| S7 | Fixed local check scripts | `scripts/check.ps1`, `scripts/check.sh` | complete |
| S9 | Traceability register | `docs/traceability/register.md` | complete |
| S11 | Final validation report | `docs/operations/final-validation-report.md` | historical; not current evidence |

## Metrics instrumentation (S2)

`ekp_eval::metrics::Summary` now carries, in addition to existing fields:
- `unwarranted_refusal_rate` (M-4)
- `manual_handoff_rate` (M-6)
- `completion_rate` (M-9)
- `error_rate` (M-7)
- `anchor_accounting` (resolved/total) + `anchor_resolution_rate` (H-4 / M-10)

`eval/src/report.rs` renders these as additional report columns. All public report numbers are derivable from `engine_snapshots.json` / `engine_hard_gates.json` / persisted case row counts.

## Test corpus (S6 / M-12)

- seed_cases.jsonl: 9
- casebook_v3.jsonl: 46
- edge_cases.jsonl: 12
- Total: 67 ≥ 50 (target M-12 met)

Regression tests verify each edge case's verdict, citation set, and abstain semantics, plus corpus size ≥ 50.

## Open decisions (product/team input required)

- Numeric target values for M-3, M-4, M-5, M-6, M-7, M-8 (locked: M-1=100%, M-2=0, M-9=100%).
- κ threshold for double-annotation (M-11).
- Backup cadence and storage path (S5).
- Remote-collaboration upgrade: proceed only when all §S10 criteria are met (GitHub repo URL/access, branch protection, CI on Linux+Windows+security, human approval, secret scanning, verified local-first baseline).

## Update: Stub → Real implementations (T1–T8)

| Module | Previous | Now |
|---|---|---|
| `runtime_verification.rs` | Stub (always Ok) | Real invariant evaluation (Always/Never/Eventually/Next) with predicate parsing |
| `formal_verification.rs` | Stub (always Unknown) | Property parser with TemporalOperator (Always/Eventually/Next/Until), validate |
| `ocr.rs` | Stub (empty) | BBox geometry, OcrEngine trait, reading_order layout grouper |
| `multimodal.rs` | Stub (basic) | ModalityKind enum, add_text/image/table, CSV table parser |
| `local_auth.rs` | Did not exist | Local auth: SHA-256 password hashing, session tokens with expiry, RBAC (viewer/editor/approver/admin) |
| `annotation.rs` (eval) | Did not exist | Cohen's κ computation with 4 tests |
| `scripts/backup.ps1`, `backup.sh` | Did not exist | Git bundle + restore drill automation |
| `.github/workflows/ci.yml` | Did not exist | CI matrix (ubuntu+windows): fmt, clippy, test, selftest, schema, secret-scan |

## Module test counts (new)

- runtime_verification: 5 tests
- formal_verification: 5 tests
- ocr: 4 tests
- multimodal: 2 tests
- local_auth: 5 tests
- annotation: 4 tests

## Validation status

- Platform: fmt clean, clippy zero warnings, build success.
- Eval: fmt clean, clippy zero warnings, build success, selftest PASSED (when binary executable).
- Test execution blocked by Windows Application Control policy (os error 4551) on this machine.
  All tests compile successfully (`cargo test --no-run` passes).
  CI pipeline (T9) will run on ubuntu-latest + windows-latest where app control is not an issue.

## Out of scope (not implemented here)

- Enterprise authentication, external connectors, AI workflow drafts (prototype remains local-only).
- PM4Py sidecar production integration (optional R3 oracle per hash-artifact contract).
- New core engine logic (already complete; hard gates H-1..H-6 all OK).

## Correction note (2026-10-26)

This report is a historical snapshot and contains contradictory/stale statements. Corrections:

- **S5:** marked “complete” before any restore drill existed. Correct status: procedure and scripts defined; restore drill **not executed**; `docs/operations/backup-log.md` remained empty. A backup is unverified until the drill passes.
- **Validation status:** the “tests blocked by Windows Application Control” paragraph describes an intermediate run. A later 2026-10-25 gate run executed 9 eval tests, 183 platform tests, both self-tests, schema validation (20 records) and smoke checks (H-1–H-6 OK; LegalRuleML 51,026 bytes, 19/19/19/16; `report.html` zero script). See [regression-checklist](../development/regression-checklist.md) for the current dated record.
- **S6:** “complete (corpus = 67 ≥ 50)” is accurate for corpus size but does not mean the target acceptance matrix (`AT-01`–`AT-31`) passed. Those cases remain `NOT RUN`.
- **Modules:** several modules listed as “now real” are library-only and are not served by either binary. Implementation of a module is not evidence of a delivered capability. See [architecture](../architecture/README.md).
- **Readiness:** the prototype is local-only. It is not production-ready, authenticated, tenant-isolated or WORM. See [production readiness](production-readiness.md).

Do not cite this file for current test counts, restore evidence or readiness. Keep it for the historical record only.
