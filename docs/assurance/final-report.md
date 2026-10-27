# Final assurance report

- **Status:** Final for the 2026-10-27 local evidence baseline; **Phase A not approved**, **Phase B blocked**
- **Owner:** Engineering + QA; business, security, operations, data and independent-review roles `UNASSIGNED`
- **Evidence revision:** `6ae6134` (code/docs baseline); release-profile checks re-run 2026-10-27
- **Environment:** Single managed Windows 11 host; Windows PowerShell 5.1; Rust 1.99.1; local loopback only; no remote, CI, staging or production
- **Matrix:** [phase-a-b-matrix.md](phase-a-b-matrix.md)
- **Test evidence:** [test-evidence.md](test-evidence.md)
- **Blockers:** [external-blockers.md](external-blockers.md)

## 1. What was completed

### 1.1 Assurance record

This round created the missing assurance layer and connected it to the existing canonical documents:

| File | Content |
| --- | --- |
| [README.md](README.md) | Index, status vocabulary, evidence classes and non-claims |
| [phase-a-b-matrix.md](phase-a-b-matrix.md) | A1–A11 and B1–B8 requirement/status/defect/fix/test/threshold/owner/evidence records |
| [root-cause-analysis.md](root-cause-analysis.md) | RC-01–RC-12, including the canonical/restore host-policy root cause |
| [source-matrix.md](source-matrix.md) | Ten official sources plus four supporting tools, with version/date, applicability, evidence and limits |
| [test-evidence.md](test-evidence.md) | E-001–E-015: commands, environment, revision, exit codes, results and artifact paths |
| [external-blockers.md](external-blockers.md) | B-01–B-12: owner, missing resource, minimum access, unlock steps and acceptance criteria |
| [security-threat-model.md](security-threat-model.md) | Trust boundaries, TH-01–TH-15 assurance view, AST-01–AST-08, residual risk and non-claims |
| [operations-and-recovery.md](operations-and-recovery.md) | Local start/stop, monitoring, incident, backup/restore and DR procedure with explicit gaps |
| [final-report.md](final-report.md) | This consolidated result |

No existing canonical rule was replaced. The new pages link to the [phase gates](../quality/phase-gates.md), [production readiness](../operations/production-readiness.md), [threat model](../architecture/threat-model.md), [AI evaluation](../quality/ai-evaluation.md), [traceability register](../traceability/register.md) and [backup log](../operations/backup-log.md).

### 1.2 Engineering fixes already in the repository

These were completed in the preceding delivery commits and are re-verified here where the host permits:

| Fix | Commit | Root cause | Verification |
| --- | --- | --- | --- |
| CLI contract hardening: mock-only `--llm`, `--audit-log` requires `--search`, standalone `--import-wellfit`, fail-closed unreadable `docs.json`, usage errors for missing values/mixed modes | `23b8996` | Permissive argument parsing and implicit defaults | 22 CLI contract tests pass in release profile; canonical blocked by host policy |
| Local session auth for the workbench: PBKDF2-HMAC-SHA256 600k, CSPRNG salt/token, constant-time compare, lockout, Viewer/Editor/Approver, CSRF guard, CSP, 64 KiB body cap | `a826472` | No login and no route-level role boundary | 10 governance-web route tests + 11 local-auth unit tests |
| Mock RAG labeling and versioned citation | `23b8996` | A mock could be mistaken for model capability | `mock-answer` label, citation requirement, contract tests |
| Backup script hardening: native stderr handling, .NET SHA-256, `age` drill, destination class | `56a944f` / `b2bae3e` | PowerShell native-command and `Get-FileHash` failures | Local restore PASS through `3ee2ee2`; `age` drill byte-identical (simulated) |
| Docs/traceability/evidence refresh | `3ee2ee2`, `c77fec4`, `bc7671a`, `28b68cc`, `6ae6134` | Evidence counts and limitations were stale or incomplete | Docs/secrets checks pass; release evidence updated |

### 1.3 What this round did not do

- It did not change product behavior. It did not lower a gate, skip a failing test or rewrite history.
- It did not make the canonical check pass. The host Application Control block remains.
- It did not verify the final `28b68cc` restore. It remains `BLOCKED`.
- It did not create a remote, CI run, PR, branch protection, IdP, tenant model, AI provider, pilot or off-host backup.
- It did not obtain an independent review.

## 2. Test results

All results below are `LOCAL` unless marked `SIMULATED` or `HISTORICAL`. Full commands and output summaries are in [test-evidence.md](test-evidence.md).

| Check | Command (working directory) | Exit | Result |
| --- | --- | --- | --- |
| Canonical gate | `scripts/check.ps1` (root) | **1** | **BLOCKED** at platform test: 22 CLI contract tests cannot start the debug binary (`os error 4551`) |
| Docs | `scripts/check-docs.ps1` (root) | 0 | PASS: 44 required, 508 links in 106 files |
| Secrets | `scripts/check-secrets.ps1` (root) | 0 | PASS: 220 files |
| Format | `cargo fmt -- --check` (both crates) | 0 | PASS |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` (both crates) | 0 | PASS, no warnings |
| Release tests | `cargo test --release --locked` (platform) | 0 | PASS: 270 (217 + 22 + 2 + 10 + 8 + 7 + 4) |
| Release tests | `cargo test --release --locked` (eval) | 0 | PASS: 12 |
| Self-test | `cargo run --release --locked --bin ekp_platform -- --selftest` | 0 | PASS: 49 assertions |
| Schema | `cargo run --release --locked --bin ekp_platform -- --declared-rules-schema` | 0 | PASS: 20 records |
| Search gate | `cargo run --release --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json` | 0 | PASS: 28 cases, 20/20 top-1, 8/8 refusals, 4 role cases (2/2), 0 violations |
| Supply chain | `scripts/check-supply-chain.ps1` | 0 | PASS: 72/12 deps, 0 vulnerabilities; `cargo deny` OK |
| Final bundle | `git bundle verify` + clone + restore check | verify 0; restore 1 | Bundle verifies/clones; restore **BLOCKED** at `eval` Clippy |
| Last restore | `3ee2ee2` fresh clone + `scripts/check.ps1` | 0 | PASS (historical/local) |
| Off-host drill | `scripts/backup-offhost.ps1 -Drill` | 0 | PASS for encryption/decryption; `SIMULATED`, not off-host |
| CI / PR / branch protection | n/a | n/a | **NOT RUN / BLOCKED**: no remote, no CI, no protection |
| Staging / production | n/a | n/a | **NOT RUN / BLOCKED**: no target |
| Real pilot / independent review | n/a | n/a | **NOT RUN / BLOCKED**: no named participants |

### Evidence classes

- `LOCAL`: this host, this repository/clone.
- `SIMULATED`: `age` drill to `%TEMP%`; no off-host service.
- `HISTORICAL`: prior revision evidence, not current.
- `CI` / `STAGING` / `PRODUCTION`: none.
- `INDEPENDENT REVIEW`: none.

## 3. Phase A status

| ID | Status | Reason |
| --- | --- | --- |
| A1 User value | **NOT MET** | No named pilot owner or representative user; no real workflow record. Prepared runbook and synthetic tests are not user acceptance. |
| A2 Functional correctness | **PARTIAL** | Release-profile tests and 67-case corpus pass; canonical run blocked; target AT-01–AT-31 not run. |
| A3 Data correctness | **PARTIAL** | Version/state/replay/schema tests pass; no real-workflow dataset, independent review or DB transaction semantics. |
| A4 Engineering quality | **PARTIAL** | Release 270/12, self-test/schema/search PASS; canonical `scripts/check.ps1` exit 1 due to host policy. Release profile is not a canonical PASS. |
| A5 Security | **PARTIAL** | Local session/role/CSRF hardening tested; no IdP, MFA, recovery, persistent identity, access review or tenant isolation. |
| A6 Maintainability | **PARTIAL** | Docs/runbooks exist; no named operator or handoff dry run. |
| A7 Traceability | **PARTIAL** | Register links requirements/code/tests/releases; no pilot run ID or validated row. |
| A8 Backup/restore | **PARTIAL** | `3ee2ee2` restore PASS (local); final `28b68cc` restore BLOCKED at build scripts; no off-host/RPO/RTO. |
| A9 AI quality | **PARTIAL** | Fixed deterministic corpus/retrieval gate; no LLM/agent/tool path; tool-failure gap; mock is not AI quality. |
| A10 Docs/handoff | **PARTIAL** | Docs map and evidence current; no pilot handoff record. |
| A11 Review/improvement | **PARTIAL** | Local self-review/retro; no independent reviewer or second annotator. |

**Phase A decision: NOT APPROVED.** A1 is the decisive unmet requirement. A4 and A8 are additionally blocked at the final revision. No combination of release-profile tests, simulated drills or self-review can close these rows.

## 4. Phase B status

| ID | Status | Reason |
| --- | --- | --- |
| B1 Security/governance | **BLOCKED** | No security owner, IdP/MFA, tenant model, data policy or production target. |
| B2 Reliability/observability | **PARTIAL** | Liveness/metrics/logs/correlation/timeout exist; no SLO, traces, alerting, retry/backoff, circuit breaker or fault drill. |
| B3 Performance/scale | **PARTIAL** | One-host benchmark only; no approved workload, soak, bottleneck analysis or capacity model. |
| B4 Delivery/change governance | **BLOCKED** | Workflow file exists but no remote, CI run, branch protection, required checks or PR review. |
| B5 AI/agent quality | **BLOCKED** | No provider/data policy or model/tool path; mock only. |
| B6 Operations/support | **BLOCKED** | No named service/operations/support owner, on-call or escalation drill. |
| B7 Backup/DR | **PARTIAL** | Local bundle/restore and simulated `age` drill only; no off-host copy, key custody, approved RPO/RTO or DR exercise. |
| B8 Data governance | **BLOCKED** | No data owner, classification, retention/deletion, legal hold or residency decision. |

**Phase B decision: BLOCKED.** The weakest mandatory rows (B1/B4/B5/B6/B8) require external decisions, owners or environments. B2/B3/B7 cannot be accepted without an approved workload, SLO and off-host recovery target.

## 5. External blockers

| ID | Missing | Responsible role | Unlock |
| --- | --- | --- | --- |
| B-01 | Pilot owner, representative user, approved non-sensitive workflow | Business pilot owner + user | Run and sign one real workflow; link run ID/commit/evidence |
| B-02 | Remote, CI, branch protection, required checks | Repository admin + engineering | Push, run CI, configure protection and required checks |
| B-03 | IdP/MFA, account lifecycle, tenant model | IAM + security + data owner | Approve IdP/tenant, implement adapter, run cross-tenant denial tests |
| B-04 | Data owner, classification, retention/deletion, legal hold | Data owner + legal/privacy | Approve policy; test deletion/export; record access review |
| B-05 | AI provider/data policy, tool policy, budget | AI/data owner + security/legal | Approve provider; implement versioned eval; test tool failure |
| B-06 | Off-host target, key custody, RPO/RTO, restore owner | Operations + security + data | Approve target/RPO/RTO; retrieve/restore; test key recovery |
| B-07 | Named security/operations/support owners | Leadership + named owners | Accept roles; exercise incident/restore; sign handoff |
| B-08 | Independent reviewer, second annotator | Independent reviewer + QA | Rerun and sign the review checklist; run κ study |
| B-09 | Approved workload, SLO, cost/capacity budget | Product + operations | Approve workload/SLO; run load/soak; publish capacity model |
| B-10 | Production deployment target and change approval | Platform/operations + change approver | Approve staging first; deploy, rollback and verify |
| B-11 | Application Control exception or clean runner | Endpoint security owner | Scope exception/runner; rerun canonical and final restore |
| B-12 | Legal/privacy/compliance scope | Legal/privacy + compliance | Define obligations, map controls, sign assessment |

Detailed unlock steps, minimum access and acceptance criteria are in [external-blockers.md](external-blockers.md).

## 6. Final conclusion

### What can be proven

- The repository is a runnable local prototype with a deterministic adjudication core and a loopback governance workbench.
- On `6ae6134`, the release-profile suite passes: platform 270, eval 12; self-test 49 assertions; schema 20 records; search gate 28 cases with 20/20 top-1, 8/8 refusals, 4 role cases and 0 violations.
- Format, Clippy, docs links, secret patterns and dependency audits pass locally on the named revision.
- A local Git bundle for `3ee2ee2` was restored and checked successfully; the `56a944f` `age` drill encrypted and decrypted byte-identically to a local-simulated destination.
- A local session login, role checks, CSRF guard, CSP and request limits are implemented and tested.
- The final `28b68cc` bundle verifies and clones, but its restored check is blocked by host policy.

### What cannot be proven

- That the canonical `scripts/check.ps1` passes at the final revision.
- That the final `28b68cc` revision can be restored and checked on this host.
- That the system is production-ready, secure, compliant, enterprise-authenticated, tenant-isolated, scalable or AI-validated.
- That CI, PR review, branch protection, a real pilot, independent review, off-host backup, approved RPO/RTO or a production deployment exists.
- That the lexical retrieval gate or mock LLM proves AI quality, hallucination resistance, tool success or cost/latency.

### Decision

**Phase A: not approved.** **Phase B: blocked.**

The correct next actions are, in order: unlock the host-policy/clean-runner path for A4/A8; establish the pilot owner and real workflow for A1/A7; obtain independent review for A11; then execute the Phase B identity, data, AI, SLO and off-host recovery work. Until those are evidenced and signed, no production or compliance claim is valid.
