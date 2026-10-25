# Quality

- **Status:** Active
- **Owner:** QA + Engineering
- **Last reviewed:** 2026-10-26
- **Phase gates:** [Phase gates](phase-gates.md)
- **Risk tiers:** [Quality gates](../development/quality-gates.md)
- **Regression baseline:** [Regression checklist](../development/regression-checklist.md)

## Test strategy

| Layer | Scope | Command / evidence | When required |
| --- | --- | --- | --- |
| Format | Both Rust crates | `cargo fmt -- --check` | Every change |
| Static analysis | Both crates | `cargo clippy --locked --all-targets -- -D warnings` | Every change |
| Unit tests | Module behavior | `cargo test --locked` | Every change |
| Integration tests | Engine lifecycle and governance web routes | `cargo test --locked` (`platform/tests/`) | Every change |
| CLI self-test | Real data, seed cases, hard gates H-1–H-6 | `cargo run --locked --bin ekp_platform -- --selftest` | Every change |
| Schema/contract | Rule schema, 20 records | `--declared-rules-schema` | Every rule/data change |
| Smoke/export | Report, coverage, HTML, LegalRuleML | `gate.ps1 -IncludeSmoke` | CLI, report, export, data, or presentation changes |
| Security | Secret scan, dependency audit, boundary tests | CI security job; targeted refusal tests | High-risk changes and releases |
| Manual accessibility | Keyboard, screen reader, contrast, narrow layout | Recorded operator evidence | UI changes |
| AI evaluation | Fixed corpus, fixed lexical retrieval set, model/prompt/tool comparison | [AI evaluation](ai-evaluation.md); `--eval --test-set data/eval/search_eval.json` | AI/model/prompt/retrieval/tool change; release if AI is in scope |
| Documentation | Required docs and relative links | `scripts/check-docs.ps1` / `.sh` | Every change; part of `scripts/check` |

The fixed local entry point is `scripts/check.ps1` (Windows) or `scripts/check.sh` (POSIX). The stronger pre-PR/release gate is `gate.ps1`; add `-IncludeSmoke` when the change touches report, export, coverage, HTML, data or CLI output.

## Current evidence (dated)

The [regression checklist](../development/regression-checklist.md) records the 2026-10-25 baseline; the 2026-10-26 verified revisions `283f83a` and `9d0d793` re-ran the same suites plus the search gate:

- `eval`: 9 Cargo tests; self-test 10 checks.
- `platform`: 212 Cargo tests (193 unit, 2 edge, 3 governance web, 8 integration, 6 search evaluation); self-test 49 assertions; schema 20 records.
- Search: `--eval --test-set data/eval/search_eval.json` 24 cases, 18/18 top-1, 6/6 refusals, permission isolation 1.00, gate PASS. Lexical only.
- Smoke: `G_ekpi_engine acc=1.000`, hard gates H-1–H-6 all OK; LegalRuleML 51,026 bytes with 19/19/19/16 structure; `report.html` zero script.
- Dependencies: `cargo audit` on both lockfiles 2026-10-26, 0 vulnerabilities; no SBOM and no CI execution yet.
- 2026-10-26 only: documentation check 44 required files / 444 links in 92 files; high-signal secret scan 201 files; `gate.ps1 -IncludeSmoke` 13/13; restore drill PASS at `283f83a`; the later `198d5f3` bundle restore is **BLOCKED** by host Application Control (`os error 4551`), recorded as blocked in the [backup log](../operations/backup-log.md).

These are dated observations, not a permanent claim. Re-run the gate for the revision being delivered and attach the output to the PR/release. Do not copy a test count from a README as current evidence without a run.

## Test data policy

- Use synthetic cases or authorized non-sensitive data only.
- Never commit real personal data, customer documents, credentials or organization-specific facts.
- Keep case IDs stable. A failing case is not deleted to make a run pass; fix the behavior or record an approved change to the expected result.
- The governance web smoke test must use a unique disposable ledger path, never an operator's existing ledger.

## Risk-based additional checks

| Risk | Additional evidence |
| --- | --- |
| Low (docs/internal refactor) | Baseline checks; link/accuracy review |
| Medium (UI, ordinary workflow, contract/data format) | Focused acceptance tests; contract/schema checks; UI accessibility checks |
| High (auth, sensitive data, evidence/version integrity, audit, approvals, migration, connectors, AI) | Negative/failure tests; fail-closed and authorization tests; retry/idempotency/version tests; migration/rollback; security/dependency scan; independent review |

A higher tier adds checks; it never removes the baseline. See [quality gates](../development/quality-gates.md) for the authoritative tiers.

## Performance and capacity

There is no approved performance target or capacity model. `DEC-19` requires an owner-approved workload, latency/throughput budget and growth model before a performance claim. Local `mean_latency_ms` in a report is not an SLO.

When a performance requirement is approved, add a reproducible load script, a capacity report and a regression threshold. Do not invent a number to fill the gap.

## Security and accessibility

- Security evidence: [threat model](../architecture/threat-model.md), [security spec](../governance/07-security-integration-spec-v1.md), [production readiness](../operations/production-readiness.md).
- Accessibility target: WCAG 2.2 AA, with local legal review pending (`DEC-17`).
- Automated route tests cover semantics/CSP/zero-script, not full accessibility. Manual keyboard, screen-reader, contrast and narrow-screen checks are required for UI changes.

## Reports and review

- Per-change evidence goes in the PR and the [traceability register](../traceability/register.md).
- Per-release evidence goes in the [release template](../development/release-template.md) and a review report under [reviews](../reviews/README.md).
- Retrospectives and improvement actions are tracked in [reviews](../reviews/README.md).
- The 2026-10-25 [final validation report](../operations/final-validation-report.md) is historical; do not cite its stale counts.

## Evidence rules

1. A check is `PASS` only if the exact command ran on the named revision and exited 0.
2. A skipped, unavailable or environment-blocked check is `Not run` or `Blocked`, never `PASS`.
3. A test that asserts a stub or placeholder is not evidence that the real capability exists.
4. A public number must name its artifact and derivation.
5. An accepted review finding is fixed and the affected checks are rerun before the PR is complete.

## Update triggers

Update this document when the test layers, commands, risk-based checks, current evidence, test-data policy or performance/accessibility policy changes. Update the phase gate and traceability register in the same change when a gate or evidence requirement changes.
