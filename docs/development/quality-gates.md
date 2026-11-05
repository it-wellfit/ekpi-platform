# Risk tiers and quality gates

- **Status:** Active
- **Owner:** Engineering + QA
- **Last reviewed:** 2026-10-26
- **Phase gates:** [Phase A/B](../quality/phase-gates.md)
- **Traceability:** [Register](../traceability/register.md)
- **DoD:** [Engineering workflow](README.md#definition-of-done)

## Risk classification

Choose the highest tier triggered by the change and record it in the issue and PR. If the impact is unknown, keep it at the higher plausible tier until evidence narrows it.

| Tier | Typical change | Required validation |
| --- | --- | --- |
| Low | Documentation, wording, or an isolated internal refactor with no observable behavior change. | All baseline checks still run; review for accuracy, links, and unintended source changes. Documentation changes must pass `scripts/check-docs`. |
| Medium | User-visible UI, ordinary workflow behavior, data-format or API contract changes, or changes to existing integrations. | Baseline checks plus focused tests for acceptance behavior, affected contract/schema checks, and regression coverage. For UI, check keyboard use, semantic structure, status/error announcements, contrast, and narrow layouts. |
| High | Authentication/authorization, sensitive data, evidence or version integrity, audit history, approvals/gates, retention, data migration, external connectors, or AI-produced workflow drafts. | Baseline checks plus targeted negative/failure tests, authorization and fail-closed checks, idempotency/retry/version tests where applicable, migration and rollback evidence, security/dependency/secret scan, and independent human review where available. Add accessibility, performance, and platform compatibility tests when those risks are touched. |

A tier selects additional checks; it never removes baseline checks.

Use [the existing-functionality regression inventory](regression-checklist.md) to select focused cases; do not claim prototype tests satisfy the future target-product matrix.

## Required baseline

The crates are independent, so run commands in each package directory rather than assuming a root Cargo workspace. The fixed entry point is:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/check.ps1
```

```bash
bash scripts/check.sh
```

It runs, in order:

1. Documentation required-file and relative-link check (`scripts/check-docs`).
2. High-signal secret-pattern scan (`scripts/check-secrets`). This is a limited scan, not a full secret scanner.
3. `eval`: `cargo fmt -- --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo test --locked`, `cargo run --locked -- --selftest`, `cargo run --locked -- --corpus-selftest`.
4. `platform`: the same format/Clippy/test steps, plus `cargo run --locked --bin ekp_platform -- --selftest`, `--declared-rules-schema` and `--eval --test-set data/eval/search_eval.json`.

The equivalent per-crate commands are:

```powershell
Set-Location eval
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked -- --selftest
cargo run --locked -- --corpus-selftest
Set-Location ..

Set-Location platform
cargo fmt -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo run --locked --bin ekp_platform -- --selftest
cargo run --locked --bin ekp_platform -- --declared-rules-schema
cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
Set-Location ..
```

The integration suite exercises governance web routes and main lifecycle paths. Keep new behavior covered at the narrowest useful layer and add an end-to-end test when a user journey crosses module boundaries. The eval CLI contract tests spawn the real binary; the strict self-test modes, empty adapter selections, mutually exclusive case sources and exact-corpus κ binding are regression-protected there.

The repository-root `.cargo/config.toml` contains narrow codegen overrides for packages that Windows Application Control has blocked by hash on this managed host. Cargo applies them automatically, so the commands above remain canonical and were executed without extra flags at `7f6ab97` (2026-10-26). The overrides do not change program semantics or disable debug assertions. If a host still blocks a harness, follow [troubleshooting](../deployment/troubleshooting.md#windows-application-control-blocks-target-binaries) and record `Blocked` rather than weakening the gate.

CI calls the same `scripts/check` entry point on Ubuntu and Windows, and a separate security job runs the high-signal secret scan plus `cargo audit` on both lockfiles. CI has not executed yet because this checkout has no remote; do not claim a green CI run until it exists.

## Local pre-PR gate

From the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File gate.ps1
```

Add `-IncludeSmoke` to also run the platform report/schema/coverage/LegalRuleML/HTML path and verify its artifacts, including the zero-script check on `report.html` and structural checks on the LegalRuleML export (10 KB minimum, official root and namespace, at least 19 `PrescriptiveStatement`, 19 `PenaltyStatement`, 19 `Reparation`, and 16 `LegalSource` elements). The gate includes the documentation and secret-pattern steps, mirrors the baseline commands with `--locked` in both crates, stops at the first failure, and prints a pass/fail summary.

Use `-IncludeSmoke` for changes to reports, exports, coverage, HTML, data fixtures or CLI output. A pure documentation change still runs the full baseline per the Low tier, but does not need the smoke path unless it changes a reported number or contract.

## Evidence rules

1. A check is `PASS` only if the exact command ran on the named revision and exited 0.
2. A skipped, unavailable or environment-blocked check is `Not run` or `Blocked`, never `PASS`.
3. A test that asserts a stub or placeholder is not evidence that the real capability exists.
4. A public number must name its artifact and derivation; do not copy a README count as current evidence.
5. A target acceptance case remains `NOT RUN` until executed on the target product; a prototype unit test does not satisfy it.
6. An accepted review finding is fixed and the affected checks are rerun before the PR is complete.
7. Record the environment, revision, date, command and result for release, security, restore and manual accessibility evidence.

## High-risk test design

Add tests that prove both success and refusal paths. Choose applicable cases from:

- Permission denied, cross-origin mutation rejected, and forbidden action refused.
- Source link unavailable, stale or conflicting version, duplicate/retried evidence, and source-version mismatch.
- Review returned, gate blocked, non-waivable condition, and unauthorized approval.
- Workflow template draft/review/publish transitions and immutability of published versions.
- CAPA verification failure, effectiveness failure, and valid closure.
- Migration rollback and historical audit replay when persistence/schema changes.
- AI draft with missing or contradictory input marked for human resolution and unable to publish itself.
- Tool/LLM failure, timeout, partial result and retry exhaustion when such a path is introduced.

For UI changes, automated Rust route/render tests are the default; also record manual keyboard, screen-reader, contrast, and narrow-screen checks unless equivalent browser automation is added. Performance checks require an explicit changed performance requirement or target rather than an invented threshold. AI changes require the fixed corpus and rerun triggers in [AI evaluation](../quality/ai-evaluation.md).

## Phase gates

Passing this baseline does not approve an MVP or a production release. Use:

- [Phase A gate](../quality/phase-gates.md#phase-a-0--1-mvp-gate) — current status **not approved**.
- [Phase B gate](../quality/phase-gates.md#phase-b-mvp--production-grade-gate) — current status **blocked**.
- [Production readiness](../operations/production-readiness.md) — current answer **not production ready**.

## Merge gate

Require the `validate` matrix (Ubuntu and Windows) and `security` job to pass before merging. Require a human approval; for high-risk changes, use an independent QA/security/domain reviewer when available. Repository branch protection must be enabled after the GitHub repository URL and access are available.

CI success is necessary but not sufficient: a high-risk change also needs its targeted evidence, and an MVP/release claim needs the relevant phase gate. Do not describe a local gate pass as a CI pass or a phase approval.

## Update triggers

Update this document when a risk tier, required check, command, CI job, evidence rule, phase gate or merge rule changes. Keep the workflow, phase gates and traceability register aligned in the same change.
