# Quality

- **Status:** Active
- **Owner:** QA + Engineering
- **Last reviewed:** 2026-10-27
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

The [regression checklist](../development/regression-checklist.md) records the 2026-10-25 baseline. The last canonical local gate PASS was `b2bae3e` (2026-10-27). The latest delivery revision `23b8996` re-ran the suite in the release profile because the canonical debug binary was blocked by host Application Control:

- `eval`: 12 Cargo tests; self-test 10 checks.
- `platform`: 270 Cargo tests in the release profile (217 unit, 22 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health); self-test 49 assertions; schema 20 records. At `23b8996` the canonical `scripts/check.ps1` is **Blocked** (`os error 4551`); the 270/12 result came from `cargo test --release --locked`. The docs check passed with 507 links in 106 files before this evidence update and 508 links after it.
- CLI contract: `--llm` requires `--search` and accepts only `mock`; the answer is labeled `mock-answer`, states `provider: mock (no external model)` and requires an exact `[doc_id@version]` citation. `--audit-log` requires `--search` and writes one local `KnowledgeSearch` event by replacing the target file; it is not an authenticated or append-only audit service. Legacy `--import-wellfit` is standalone and fails closed on an unreadable or invalid database.
- Authentication boundary: PBKDF2-HMAC-SHA256 (600,000 iterations, random salt), OS-CSPRNG session tokens, constant-time comparison, 5-failure/300-second lockout, Viewer/Editor/Approver route checks, 64 KiB form limit and a stricter same-origin/fetch-metadata guard. Failed logins are only tracked for existing accounts, so arbitrary usernames cannot grow lockout state; expired sessions are pruned on login. These are local session controls, not enterprise identity.
- Search: `--eval --test-set data/eval/search_eval.json` 28 cases, 20/20 top-1, 8/8 refusals, 4 role-restricted cases (2 allowed, 2 denied), 0 violations, isolation 1.00, gate PASS. Lexical only. This is library-filter evidence, not authentication or tenant isolation (`S19`).
- Smoke: `G_ekpi_engine acc=1.000`, hard gates H-1–H-6 all OK; LegalRuleML 51,026 bytes with 19/19/19/16 structure; `report.html` zero script.
- `gate.ps1 -IncludeSmoke`: 13/13 steps PASS at `b2bae3e` (not rerun at `23b8996`; the canonical check there was blocked by host policy). Documentation and high-signal secret scan pass; exact link/file counts are recorded in the [release evidence bundle](../operations/release-evidence-bundle.md).
- Restore drill: PASS at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f` and `3ee2ee2` (manual fresh clone; the automatic TEMP drill for `3ee2ee2` was blocked by host Application Control and is not counted as PASS). Latest bundles: `ekpi-platform-2026-10-27-124512.bundle` (revision `7fd0c0b`, SHA-256 `EBFD373C45EC7A556C6B7C59483F16CE42FA77C4E9C35A85BD50C5B8FF34A2FF`), `ekpi-platform-2026-10-27-124757.bundle` (revision `56a944f`, SHA-256 `23EA44011711566CC7147767E6FF4258F75030CFFA0507EC98E00323AACE6421`) and `ekpi-platform-2026-10-27-125701.bundle` (revision `3ee2ee2`, SHA-256 `7E21EEEE456F36DE79056508BE718705D6294525D98E39C5F1C80C585C73FE19`), all `git bundle verify` okay and a fresh clone whose restored `scripts/check.ps1` exited 0. The `56a944f` bundle also passed an `age` encrypt/decrypt drill, but the destination was local-simulated, not off-host. The final `bc7671a` bundle (`ekpi-platform-2026-10-27-132616.bundle`, SHA-256 `24E3BF7E…8DBB06`; preceded by `c77fec4`, SHA-256 `780464D6…4D9FC9`) verifies and its clone reached `bc7671a`, but the automatic and manual restore checks were **Blocked** at `eval` Clippy by host Application Control (`os error 4551`), so it is not a restore PASS. See the [backup log](../operations/backup-log.md).
- Dependencies: local `cargo audit` against both lockfiles reports 0 vulnerabilities; CycloneDX SBOM artifacts for `ekp_platform` (67 components) and `ekp_eval` (11 components) are generated with a SHA-256 list. No CI execution or independent attestation yet.
- Performance: one local release benchmark at `56a944f` recorded eval p50/p95 `17.432/23.319 ms` and loopback `/healthz` p50/p95 `19.010/24.069 ms` at `5,369.21 req/s` (artifact `artifacts/benchmark/benchmark-2026-10-27-124954.json`). This is a one-host local observation, not a capacity model or SLO.

Historical note: the earlier audit revision `8bf2d53` could not run canonical `cargo test --locked`/`gate.ps1` because Windows Application Control blocked execution from `target` (`os error 4551`). The narrow profile overrides in `.cargo/config.toml` removed that host block for a period (`7f6ab97`–`b2bae3e`), but the block returned at `23b8996` on the freshly rebuilt debug binary. The release-profile suite passed there; the canonical result stays **Blocked**. `EKP_PLATFORM_BIN` is a diagnostic override for the test harness and cannot convert a blocked canonical run into a PASS. The earlier `198d5f3` restore attempt also remains a recorded **BLOCKED** row. There is still no CI run, remote, off-host backup or approved RPO/RTO. See [troubleshooting](../deployment/troubleshooting.md#windows-application-control-blocks-target-binaries).

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

There is no approved performance target or capacity model. `DEC-19` requires an owner-approved workload, latency/throughput budget and growth model before a performance claim. The recorded local benchmark is useful as a regression baseline only; local `mean_latency_ms` and loopback throughput are not an SLO or a capacity model.

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
