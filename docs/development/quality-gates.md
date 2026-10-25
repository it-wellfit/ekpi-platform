# Risk tiers and quality gates

## Risk classification

Choose the highest tier triggered by the change and record it in the issue and PR. If the impact is unknown, keep it at the higher plausible tier until evidence narrows it.

| Tier | Typical change | Required validation |
| --- | --- | --- |
| Low | Documentation, wording, or an isolated internal refactor with no observable behavior change. | All baseline CI checks still run; review for accuracy, links, and unintended source changes. |
| Medium | User-visible UI, ordinary workflow behavior, data-format or API contract changes, or changes to existing integrations. | Baseline checks plus focused tests for acceptance behavior, affected contract/schema checks, and regression coverage. For UI, check keyboard use, semantic structure, status/error announcements, contrast, and narrow layouts. |
| High | Authentication/authorization, sensitive data, evidence or version integrity, audit history, approvals/gates, retention, data migration, external connectors, or AI-produced workflow drafts. | Baseline checks plus targeted negative/failure tests, authorization and fail-closed checks, idempotency/retry/version tests where applicable, migration and rollback evidence, security/dependency/secret scan, and independent human review where available. Add accessibility, performance, and platform compatibility tests when those risks are touched. |

A tier selects additional checks; it never removes baseline checks.

Use [the existing-functionality regression inventory](regression-checklist.md) to select focused cases; do not claim prototype tests satisfy the future target-product matrix.

## Required baseline

The crates are independent, so run commands in each package directory rather than assuming a root Cargo workspace. CI runs the full baseline on Linux and Windows.

For eval/:

    Set-Location eval
    cargo fmt -- --check
    cargo clippy --locked --all-targets -- -D warnings
    cargo test --locked
    cargo run --locked -- --selftest
    Set-Location ..

For platform/:

    Set-Location platform
    cargo fmt -- --check
    cargo clippy --locked --all-targets -- -D warnings
    cargo test --locked
    cargo run --locked --bin ekp_platform -- --selftest
    cargo run --locked --bin ekp_platform -- --declared-rules-schema
    Set-Location ..

The integration suite exercises governance web routes and main lifecycle paths. Keep new behavior covered at the narrowest useful layer and add an end-to-end test when a user journey crosses module boundaries. The CI security job audits both Cargo lockfiles and scans the PR/push commit range for secrets.

## Local pre-PR gate

From the repository root, run:

    powershell -ExecutionPolicy Bypass -File gate.ps1

Add `-IncludeSmoke` to also run the platform report/schema/coverage/LegalRuleML/HTML path from the regression checklist and verify its artifacts, including the zero-script check on `report.html` and structural checks on the LegalRuleML export (10 KB minimum, official root and namespace, at least 19 `PrescriptiveStatement`, 19 `PenaltyStatement`, 19 `Reparation`, and 16 `LegalSource` elements). The script mirrors the baseline commands above with `--locked` in both crates, stops at the first failure, and prints a pass/fail summary. CI remains authoritative; the gate exists so a contributor can reproduce the same checks locally before opening a PR.

## High-risk test design

Add tests that prove both success and refusal paths. Choose applicable cases from:

- Permission denied, cross-origin mutation rejected, and forbidden action refused.
- Source link unavailable, stale or conflicting version, duplicate/retried evidence, and source-version mismatch.
- Review returned, gate blocked, non-waivable condition, and unauthorized approval.
- Workflow template draft/review/publish transitions and immutability of published versions.
- CAPA verification failure, effectiveness failure, and valid closure.
- Migration rollback and historical audit replay when persistence/schema changes.
- AI draft with missing or contradictory input marked for human resolution and unable to publish itself.

For UI changes, automated Rust route/render tests are the default; also record manual keyboard, screen-reader, contrast, and narrow-screen checks unless equivalent browser automation is added. Performance checks require an explicit changed performance requirement or target rather than an invented threshold.

## Merge gate

Require validate (ubuntu-latest), validate (windows-latest), and security to pass before merging. Require a human approval; for high-risk changes, use an independent QA/security/domain reviewer when available. Repository branch protection must be enabled after the existing GitHub repo URL and access are available.

