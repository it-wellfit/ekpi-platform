# Final assurance report

- **Status:** Final for the 2026-10-27 local evidence baseline; **Phase A not approved**, **Phase B blocked**
- **Owner:** Engineering + QA; business, security, operations, data and independent-review roles `UNASSIGNED`
- **Evidence revision:** `32f2b36` (current working-tree canonical PASS); `718db57` (code change and preceding canonical PASS); `6ed236d` (final evidence bundle clean-clone canonical PASS and automatic restore PASS); previous assurance bundle `41beaea`; code baseline `6ae6134`; release-profile checks re-run 2026-10-27
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
| [test-evidence.md](test-evidence.md) | E-001–E-023: commands, environment, revision, exit codes, results and artifact paths |
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
| Local session auth for the workbench: PBKDF2-HMAC-SHA256 600k, CSPRNG salt/token, constant-time compare, lockout, Viewer/Editor/Approver, CSRF guard, CSP, 64 KiB body cap | `a826472` / `718db57` | No login and no route-level role boundary; mutation actor was form-supplied | 11 governance-web route tests (including session-derived actor) + 11 local-auth unit tests |
| Append-only JSONL search audit: verify chain before append, reject corrupt/legacy files, local writer lock, exit 1 on failure | `718db57` | Single-object overwrite was neither append-only nor tamper-evident | 11 audit unit tests + 24 CLI contract tests; canonical check exit 0 |
| Mock RAG labeling and versioned citation | `23b8996` | A mock could be mistaken for model capability | `mock-answer` label, citation requirement, contract tests |
| Backup script hardening: native stderr handling, .NET SHA-256, `age` drill, destination class | `56a944f` / `b2bae3e` | PowerShell native-command and `Get-FileHash` failures | Local restore PASS through `3ee2ee2`; `age` drill byte-identical (simulated) |
| Docs/traceability/evidence refresh | `3ee2ee2`, `c77fec4`, `bc7671a`, `28b68cc`, `6ae6134` | Evidence counts and limitations were stale or incomplete | Docs/secrets checks pass; release evidence updated |

### 1.3 What this round did not do

- It changed the served workbench and CLI audit behavior in `718db57`: web mutation actors come from the validated local session, and `--audit-log` is an append-only JSONL chain. It did not lower a gate, skip a failing test or rewrite history.
- The canonical check passed in a fresh clone at `41beaea` and again at the final bundle revision `6ed236d`; it also passed in the working tree at `4697aa1`, `718db57` and `32f2b36`. The earlier working-tree block at `41beaea` remains historical.
- It created the `41beaea` assurance bundle and the final `6ed236d` bundle, and verified both automatic restores (exit 0). The earlier `28b68cc` row remains `BLOCKED` and is retained as history.
- It did not create a remote, CI run, PR, branch protection, IdP, tenant model, AI provider, pilot or off-host backup.
- It did not obtain an independent review.

## 2. Test results

All results below are `LOCAL` unless marked `SIMULATED` or `HISTORICAL`. Full commands and output summaries are in [test-evidence.md](test-evidence.md).

| Check | Command (working directory) | Exit | Result |
| --- | --- | --- | --- |
| Canonical gate (working tree, current) | `scripts/check.ps1` (root) | **0** | **PASS**: docs 53 required/660 links in 115 files, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28 (`32f2b36`) |
| Canonical gate (working tree, previous) | `scripts/check.ps1` (root) | **0** | **PASS**: docs 53 required/657 links in 115 files, secrets 229, eval 12, platform 277, self-test 49, schema 20, search 28 (`718db57`) |
| Canonical gate (working tree, historical) | `scripts/check.ps1` (root) | **1** | **BLOCKED** at platform test: 22 CLI contract tests could not start the existing debug binary (`os error 4551`) (`41beaea`) |
| Canonical gate (clean clone) | `scripts/check.ps1` (fresh clone of the `6ed236d` bundle) | **0** | **PASS**: docs 53 required/652 links in 105 tracked files, secrets 229, eval 12, platform 270, self-test 49, schema 20, search 28 |
| Docs | `scripts/check-docs.ps1` (root) | 0 | PASS: 53 required, 660 links in 115 files at `32f2b36` (657 at `718db57`; 655 at `6ed236d`; 656 at `41beaea`; 652 at `947cd27`; 44/508/106 at the `6ae6134` code baseline) |
| Secrets | `scripts/check-secrets.ps1` (root) | 0 | PASS: 229 files at `6ed236d` (220 at the `6ae6134` code baseline) |
| Format | `cargo fmt -- --check` (both crates) | 0 | PASS |
| Clippy | `cargo clippy --locked --all-targets -- -D warnings` (both crates) | 0 | PASS, no warnings |
| Release tests | `cargo test --release --locked` (platform) | 0 | PASS, **270** at the `6ae6134` baseline (217 + 22 + 2 + 10 + 8 + 7 + 4); canonical debug profile at `718db57`/`32f2b36` passes **277** (221 + 24 + 2 + 11 + 8 + 7 + 4) |
| Release tests | `cargo test --release --locked` (eval) | 0 | PASS: 12 |
| Self-test | `cargo run --release --locked --bin ekp_platform -- --selftest` | 0 | PASS: 49 assertions |
| Schema | `cargo run --release --locked --bin ekp_platform -- --declared-rules-schema` | 0 | PASS: 20 records |
| Search gate | `cargo run --release --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json` | 0 | PASS: 28 cases, 20/20 top-1, 8/8 refusals, 4 role cases (2/2), 0 violations |
| Supply chain | `scripts/check-supply-chain.ps1` | 0 | PASS: 72/12 deps, 0 vulnerabilities; `cargo deny` OK |
| Final bundle | `git bundle verify` + automatic restore at `6ed236d` | verify 0; restore 0 | **PASS**: bundle `ekpi-platform-2026-10-27-141734.bundle`, SHA-256 `E3E8C3EE…EEA13E`; clone HEAD `6ed236d`; restored `scripts/check.ps1` exit 0 |
| Earlier assurance bundle | `41beaea` bundle verify/restore | verify 0; restore 0 | **PASS (local)**: `ekpi-platform-2026-10-27-140645.bundle`, SHA-256 `7C5B337A…0623B3C`; clean-clone canonical check exit 0 |
| Earlier final bundle | `28b68cc` bundle verify/clone/restore | verify 0; restore 1 | Bundle verifies/clones; restore **BLOCKED** at `eval` Clippy (historical, not counted) |
| Last restore | `6ed236d` fresh clone + `scripts/check.ps1` | 0 | PASS (local); previous verified restore `41beaea` |
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
| A2 Functional correctness | **PARTIAL** | Release-profile tests and 67-case corpus pass; canonical passes in the working tree at `718db57`/`32f2b36` and the clean clone at `6ed236d`; target AT-01–AT-31 not run. |
| A3 Data correctness | **PARTIAL** | Version/state/replay/schema tests pass; no real-workflow dataset, independent review or DB transaction semantics. |
| A4 Engineering quality | **MET** (local) | Working tree at `32f2b36`: canonical `scripts/check.ps1` exit 0, platform 277, docs 53/660/115 (`718db57`: 657). Fresh clone at `6ed236d`: exit 0, platform 270. No CI or named host-policy exception owner. |
| A5 Security | **PARTIAL** | Local session/role/CSRF hardening tested; no IdP, MFA, recovery, persistent identity, access review or tenant isolation. |
| A6 Maintainability | **PARTIAL** | Docs/runbooks exist; no named operator or handoff dry run. |
| A7 Traceability | **PARTIAL** | Register links requirements/code/tests/releases; no pilot run ID or validated row. |
| A8 Backup/restore | **MET** (local) | `6ed236d` bundle verifies, clones and the automatic restore drill exits 0; logged. No off-host/RPO/RTO. |
| A9 AI quality | **PARTIAL** | Fixed deterministic corpus/retrieval gate; no LLM/agent/tool path; tool-failure gap; mock is not AI quality. |
| A10 Docs/handoff | **PARTIAL** | Docs map and evidence current; no pilot handoff record. |
| A11 Review/improvement | **PARTIAL** | Local self-review/retro; no independent reviewer or second annotator. |

**Phase A decision: NOT APPROVED.** A1 is the decisive unmet requirement. A4 is `MET` on the local working-tree and clean-clone scope; A8 is `MET` on the local restore scope; A2/A3/A5/A6/A7/A9/A10/A11 remain `PARTIAL`. No combination of release-profile tests, simulated drills or self-review can close A1 or the enterprise-identity, AI and independent-review gaps.

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
- On `718db57`/`32f2b36`, the canonical debug profile passes: platform 277, eval 12; self-test 49 assertions; schema 20 records; search gate 28 cases with 20/20 top-1, 8/8 refusals, 4 role cases and 0 violations.
- Format, Clippy, docs links, secret patterns and dependency audits pass locally on the named revision.
- Local Git bundles for `3ee2ee2`, `41beaea` and `6ed236d` were restored and checked successfully; the `56a944f` `age` drill encrypted and decrypted byte-identically to a local-simulated destination.
- A local session login, role checks, session-derived mutation actors, CSRF guard, CSP and request limits are implemented and tested.
- The final `6ed236d` bundle `ekpi-platform-2026-10-27-141734.bundle` verifies, clones to `6ed236d`, and its automatic restore drill ran the restored canonical `scripts/check.ps1` to exit 0 (platform 270, eval 12, self-test 49, schema 20, search 28, docs 652 links in 105 tracked files, secrets 229).
- The previous `41beaea` assurance bundle also verifies, clones and ran the canonical check to exit 0.
- The working tree at `4697aa1`, `718db57` and `32f2b36` passed the canonical check; the earlier `41beaea` working-tree block (`os error 4551`) remains historical.
- The earlier `28b68cc` bundle verifies and clones, but its restored check remains `BLOCKED` and is not counted.
- No off-host backup, key custody, approved RPO/RTO or disaster-recovery exercise exists.

### What cannot be proven

- That the canonical `scripts/check.ps1` is reproducible on a different host or in CI. It passes in the working tree at `32f2b36` and a fresh clone at `6ed236d`, but there is no CI run and no named host-policy exception owner.
- That the earlier `28b68cc` revision can be restored and checked on this host; it remains `BLOCKED` and is not counted.
- That the system is production-ready, secure, compliant, enterprise-authenticated, tenant-isolated, scalable or AI-validated.
- That CI, PR review, branch protection, a real pilot, independent review, off-host backup, approved RPO/RTO or a production deployment exists.
- That the lexical retrieval gate or mock LLM proves AI quality, hallucination resistance, tool success or cost/latency.

### Decision

**Phase A: not approved.** A1 is `NOT MET`; the pilot, enterprise-identity, AI and independent-review gaps remain. A4 and A8 are `MET` only on the local clean-clone/restore scope. **Phase B: blocked.**

The correct next actions are, in order: establish the pilot owner and real workflow for A1/A7; obtain independent review for A11; make the canonical check reproducible in the approved working environment or CI; then execute the Phase B identity, data, AI, SLO and off-host recovery work. Until those are evidenced and signed, no production or compliance claim is valid.
