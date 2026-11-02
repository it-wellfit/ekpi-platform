# Phase A and Phase B assurance matrix

- **Status:** Active; Phase A **not approved**, Phase B **blocked**
- **Owner:** Product + QA + Engineering; business, security, operations, data and independent-review roles are `UNASSIGNED`
- **Last reviewed:** 2026-11-02
- **Evidence baseline:** `d502147` (2026-11-02) current hardening revision with two same-command canonical PASS runs; `eec11f2` (2026-11-02) preceding hardening revision with canonical PASS; `279c919` (2026-10-30) earlier hardening revision with canonical PASS; `21e23d6` (2026-10-29) earlier hardening revision; `b2d8afc`, `bbe49eb` and `c827351` (2026-10-29) earlier hardening revisions; `604a2a5` (2026-10-27) preceding working-tree canonical PASS; `6ed236d` (2026-10-27) last clean-clone canonical PASS; `499e0d1` (2026-10-27) last restore-verified bundle
- **Canonical gate:** [Phase A/B gates](../quality/phase-gates.md)
- **Test evidence:** [test-evidence.md](test-evidence.md)

This matrix expands the canonical gate into the fields required for an auditable acceptance decision. It records the requirement, observed state, defect and root cause, impact, fix, executable acceptance test, threshold, external dependency, evidence, owner and remaining blocker. It does not replace the canonical gate or the [governance acceptance matrix](../governance/04-acceptance-test-matrix.md).

## Status rules

- `PASS` requires the named command or drill to run on the named revision and exit 0. At `41beaea` A4 is `MET` on the clean-clone canonical run; at `6ed236d` A4 is `MET` again and A8 is `MET` on the local restore drill; at `499e0d1` A8 is `MET` again on the current bundle restore. At `c827351` the canonical check is `BLOCKED` at the `edge_cases` harness (`os error 4551`), so A4 is not `MET` for that revision. At `bbe49eb` every release-profile target passed, but A4 is `BLOCKED` because the canonical entry point at docs HEAD `17df06a` stopped at the debug CLI binary (`os error 4551`, E-031). At `b2d8afc`, 329 executed release-profile tests pass, the 2 `edge_cases` tests are host-blocked, and A4 remains `BLOCKED` because the canonical entry point is blocked at the debug lib and `cli_contract` harnesses (`os error 4551`, E-032). At `21e23d6`, 308 executed release-profile tests pass, 27 tests are host-blocked across three harnesses, and A4 remains `BLOCKED` because the canonical entry point is blocked at the debug lib harness (`os error 4551`, E-033). At `279c919`, the canonical entry point exited 0 on two same-command runs and all 345 release-profile tests ran, so A4 is `MET` for that revision (E-034); at `eec11f2`, the canonical entry point exited 0 on the exact code commit and all 354 platform tests ran (E-035); at `d502147`, two same-command canonical runs exited 0 on the exact code commit and all 356 platform tests ran, so A4 is `MET` for the current revision (E-036). All are local worktree evidence, not CI.
- A release-profile result is not a canonical `scripts/check.ps1` pass. A clean-clone canonical pass is revision-bound and is not CI, staging or production evidence.
- A local clone is not CI. A simulated `age` drill is not an off-host backup. A self-review is not independent review.
- A row is `PARTIAL` when some evidence exists but the full condition is unmet; it is `BLOCKED` when an external owner, environment or approval is required; it is `NOT MET` when the required condition is absent.
- The overall phase result follows the weakest mandatory row, not the majority.

## Phase A summary

| ID | Dimension | Status at `604a2a5` / restore `499e0d1` (code baseline `6ae6134`) | Strongest current evidence | Decisive remaining gap |
| --- | --- | --- | --- | --- |
| A1 | User value | `NOT MET` | Prepared [pilot runbook](../operations/pilot-runbook.md) and synthetic workflow tests | No named pilot owner or representative user; no real workflow record |
| A2 | Functional correctness | `PARTIAL` | 67-case corpus, unit/integration tests, self-test, search gate | Target acceptance cases AT-01–AT-31 are `NOT RUN`; pilot workflow not exercised |
| A3 | Data correctness | `PARTIAL` | Version binding, rule activity, state, replay and schema tests | No real-workflow dataset, no independent review, no database transaction semantics |
| A4 | Engineering quality | **`MET` at current `d502147` (E-036)**; `MET` at preceding `eec11f2` (E-035); `MET` at earlier `279c919` (E-034); `BLOCKED` at `21e23d6` (E-033), `b2d8afc` (E-032) and `bbe49eb` (E-031); historical `MET` at `604a2a5`/`6ed236d` | `d502147`: canonical exit 0 twice; docs 53/687/105, secrets 231, eval 12 + 10 self-test checks, platform **356** (284 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health), self-test 49, schema 20, search 28; fmt/Clippy clean. `eec11f2`: canonical exit 0; docs 53/682/105, secrets 231, eval 12 + 10 self-test checks, platform **354** (282 unit, 31 CLI contract, 2 edge, 20 governance web, 8 integration, 7 search evaluation, 4 web health), self-test 49, schema 20, search 28; fmt/Clippy clean. `279c919`: canonical exit 0 twice; platform 345. `21e23d6`: 308 executed, 27 **BLOCKED**. `b2d8afc`: 329 executed, 2 edge **BLOCKED**. `bbe49eb`: 325 release-profile tests passed, canonical debug CLI denied. `c827351`: 312 executed, 2 edge NOT RUN. Fresh clone `6ed236d`: canonical exit 0, platform 270. Working tree `718db57`/`32f2b36`/`604a2a5`: canonical exit 0, platform 277 | No CI and no named host-policy exception owner; the `d502147` PASS is local isolated-worktree evidence only |
| A5 | Security | `PARTIAL` | Local PBKDF2 session, Viewer/Editor/Approver, CSRF guard with single-value header checks, CSP, body cap, 17 route tests + 5 parser unit tests + 13 local-auth unit tests | No enterprise IdP, MFA, recovery, persistent identity, access review or tenant isolation |
| A6 | Maintainability | `PARTIAL` | Install, configuration, troubleshooting, runbook and evidence pages | No named operator, no independent handoff dry run |
| A7 | Traceability | `PARTIAL` | Requirement → spec → code → test → release rows in the register | No pilot run ID, no validated row, no real-user acceptance |
| A8 | Backup and restore | `MET` (local) | `499e0d1` bundle verifies, clones and the automatic restore drill exits 0; logged | No off-host copy, approved RPO/RTO, key custody or scheduled drill (B7) |
| A9 | AI quality | `PARTIAL` | Fixed 67-case verdict corpus and 28-case lexical retrieval gate | No LLM/agent/tool path; tool-failure dimension absent; mock output is not AI quality |
| A10 | Documentation and handoff | `PARTIAL` | Canonical docs map, commands, limits and evidence pages | No pilot handoff record or new-operator acceptance |
| A11 | Review and improvement | `PARTIAL` | Local review, retro template, improvement register | No independent reviewer, no second annotator, no signed review |

**Phase A decision:** `NOT APPROVED`. A1 is `NOT MET`; A2/A3/A5/A6/A7/A9/A10/A11 are `PARTIAL`; A4 is `MET` at the current `d502147` revision because two same-command canonical runs exited 0 on the exact code commit and all 356 platform tests ran (E-036), `MET` at the preceding `eec11f2` revision because the canonical entry point exited 0 on the exact code commit and all 354 platform tests ran (E-035), and `MET` at the earlier `279c919` revision (E-034), while `21e23d6` (E-033), `b2d8afc` (E-032) and `bbe49eb` (E-031) remain blocked history; A8 is `MET` on the local restore scope at `499e0d1`, and none of `279c919`, `eec11f2` or `d502147` is bundled. A passing canonical run and restore drill cannot waive A1 or the unmet pilot, security, AI and independent-review scope.

## Phase B summary

| ID | Dimension | Status at `499e0d1` | Strongest current evidence | Decisive remaining gap |
| --- | --- | --- | --- | --- |
| B1 | Security and governance | `BLOCKED` | Local threat model, boundary tests, secret scan, SBOM, dependency audit | No security owner, IdP/MFA, tenant model, approved data policy or production target |
| B2 | Reliability and observability | `PARTIAL` | Request timeout, concurrency bound, JSON access log, correlation id, `/metrics`, liveness `/healthz` | No SLI/SLO, error budget, traces, alerting, retry/backoff, circuit breaker or fault drill |
| B3 | Performance and scale | `PARTIAL` | One-host release benchmark (eval p50/p95 17.432/23.319 ms; loopback p50/p95 19.010/24.069 ms; 5,369.21 req/s) | No approved workload, soak, bottleneck analysis, capacity model or cost model |
| B4 | Delivery and change governance | `BLOCKED` | CI workflow file and local gates exist | No remote, no CI execution, no branch protection, no required checks, no deployment target |
| B5 | AI and agent quality | `BLOCKED` | Mock-only LLM contract and deterministic eval policy | No approved AI provider/data policy; no versioned model evaluation or tool-failure test |
| B6 | Operations and support | `BLOCKED` | Draft runbooks and ownership proposal | No named service, operations or support owner; no on-call, escalation drill or handoff sign-off |
| B7 | Backup and disaster recovery | `PARTIAL` | Local bundle + restore drills; local-simulated `age` encryption drill | No off-host artifact, approved RPO/RTO, key custody, scheduled restore or DR exercise |
| B8 | Data governance | `BLOCKED` | Data-handling limits in docs and threat model | No data owner, inventory, classification, retention/deletion, legal hold or residency decision |

**Phase B decision:** `BLOCKED`. B1/B4/B5/B6/B8 require decisions, owners or environments that do not exist. B2/B3/B7 are `PARTIAL`; their local evidence is exploratory, not an approved production acceptance.

---

## Phase A records

### A1 — Real pilot and named accountability

| Field | Record |
| --- | --- |
| Requirement | At least one complete real workflow is executed by a representative user, with a named owner and a recorded outcome. |
| Actual state | `NOT MET`. The [pilot runbook](../operations/pilot-runbook.md) and synthetic workflow tests are prepared, but no pilot owner, representative user, session record or acceptance signature exists. |
| Defect and root cause | Process/ownership gap, not a code defect. No business pilot owner has selected an authorized non-sensitive workflow or accepted the residual risk. |
| Impact and risk | Synthetic tests prove conformance, not fitness for real use. User value, usability, data suitability and operational handoff remain unproven. |
| Fix / next action | Name a pilot owner and representative user; select one workflow from the approved scope; run it on authorized non-sensitive data; record inputs, outputs, errors, acceptance and consent; log the run in the traceability register. |
| Acceptance test | A completed pilot record containing the workflow ID, owner, user role, date, environment, input dataset class, expected/actual result, defects, acceptance decision and signature. |
| Threshold | At least one end-to-end workflow accepted by the named representative user; no unresolved critical defect; data is non-sensitive or explicitly authorized. |
| External role / environment | Business pilot owner + representative user + approved non-sensitive dataset/environment. All are `UNASSIGNED`. |
| Evidence | [Pilot runbook](../operations/pilot-runbook.md); [traceability register](../traceability/register.md); [phase gates](../quality/phase-gates.md). No pilot execution log exists. |
| Owner | `UNASSIGNED` (business/pilot owner) |
| Final status / blocker | `NOT MET` — cannot be closed by code. See [external blocker B-01](external-blockers.md#b-01-pilot-owner-and-representative-user). |

### A2 — Functional correctness

| Field | Record |
| --- | --- |
| Requirement | Primary use cases and high-risk boundary cases pass for the approved scope. |
| Actual state | `PARTIAL`. Release-profile tests pass at `6ae6134`; the clean-clone canonical check also passes at `6ed236d`; the working-tree canonical run is blocked. The target acceptance matrix AT-01–AT-31 is not executed. |
| Defect and root cause | Scope gap and host policy. The current synthetic 67-case corpus covers the prototype; the target acceptance cases require approved workflows and a runnable canonical environment. |
| Impact and risk | A regression outside the synthetic corpus or target cases could ship unnoticed. A blocked canonical run prevents a single-command acceptance claim. |
| Fix / next action | Execute the target acceptance cases on the approved scope; add regression tests for every accepted case; restore a host/CI path that can run the canonical check. |
| Acceptance test | `cargo test --release --locked` in both crates plus `--selftest`, schema and search gate; target AT cases executed and recorded. |
| Threshold | 0 failed tests in the executed suite; all in-scope AT cases pass or have an accepted, time-bound exception. |
| External role / environment | QA owner; pilot workflow; canonical host or CI runner. |
| Evidence | E-002, E-003, E-004 in [test-evidence.md](test-evidence.md); [regression checklist](../development/regression-checklist.md). |
| Owner | QA (role) + Engineering; named QA owner `UNASSIGNED` |
| Final status / blocker | `PARTIAL` — AT-01–AT-31 not run; the working-tree canonical run is blocked (the clean clone passes). |

### A3 — Data correctness

| Field | Record |
| --- | --- |
| Requirement | Version binding, rule activity, state transitions, evidence sufficiency and error handling match the specification. |
| Actual state | `PARTIAL`. Unit/integration tests cover binding, draft rejection, replay, schema and fail-closed behavior. No real-workflow dataset or independent data review exists. |
| Defect and root cause | The prototype uses file/JSONL persistence, not a transactional database. Real-workflow data and an independent reviewer are absent. |
| Impact and risk | Concurrency, partial-write and recovery behavior under a real workload are not proven. An incorrect version binding could change a conclusion. |
| Fix / next action | Run a real-workflow dataset; verify version binding and replay; add transactional or explicit single-writer guarantees if the deployment target requires them; obtain independent review. |
| Acceptance test | Replay a real workflow set, compare expected/actual verdicts and citations, verify schema and state transitions, and record the dataset hash. |
| Threshold | 100% of in-scope cases match approved expected results; no unresolved data-integrity defect. |
| External role / environment | Product/data owner; representative dataset; independent reviewer. |
| Evidence | 67-case corpus and tests; [data and state model](../governance/06-data-and-state-model-v1.md); [AI evaluation](../quality/ai-evaluation.md). |
| Owner | Engineering + QA; data owner `UNASSIGNED` |
| Final status / blocker | `PARTIAL` — no real dataset or independent review. |

### A4 — Engineering quality and canonical acceptance

| Field | Record |
| --- | --- |
| Requirement | Format, Clippy with warnings denied, unit/integration tests and self-tests pass through the canonical entry point. |
| Actual state | `MET` at current `d502147`: two same-command canonical runs exited 0 on the exact code commit (docs 53/687/105, secrets 231, eval 12 + 10 self-test checks, platform **356** = 284 unit + 31 CLI contract + 2 edge + 20 governance web + 8 integration + 7 search evaluation + 4 web health, self-test 49, schema 20, search 28), with fmt/Clippy clean in both crates (E-036). `MET` at preceding `eec11f2`: canonical exit 0 on the exact code commit (docs 53/682/105, secrets 231, eval 12 + 10 self-test checks, platform **354** = 282 unit + 31 CLI contract + 2 edge + 20 governance web + 8 integration + 7 search evaluation + 4 web health, self-test 49, schema 20, search 28), with fmt/Clippy clean in both crates (E-035). `MET` at earlier `279c919`: canonical exit 0 on two same-command runs (docs 53/680/115, secrets 231, eval 12 + 10 self-test checks, platform **345** = 273 unit + 31 CLI contract + 2 edge + 20 governance web + 8 integration + 7 search evaluation + 4 web health, self-test 49, schema 20, search 28), with fmt/Clippy clean in both crates (E-034). `BLOCKED` at earlier `21e23d6` (308 executed release-profile tests pass, 27 tests host-blocked across three harnesses, canonical blocked at the debug lib harness, E-033); `BLOCKED` at `b2d8afc` (329 executed release-profile tests pass, 2 edge tests host-blocked, canonical blocked at the debug lib and `cli_contract` harnesses, E-032); `BLOCKED` at `bbe49eb` (canonical at docs HEAD `17df06a` stopped at the debug CLI binary, E-031); `BLOCKED` at earlier `c827351`; historical `MET` at `604a2a5`/`6ed236d`. At `b2d8afc`, 329 platform release-profile tests pass (262 unit, 30 CLI contract, 18 governance web, 8 integration, 7 search evaluation, 4 web health), the 2 edge tests are BLOCKED at spawn, eval passes 12 tests + 10 self-test checks, self-test 49, schema 20, search 28 and fmt/Clippy pass (E-032). At `bbe49eb`, every platform release-profile target passed: 325 tests (257 unit, 30 CLI contract, 2 edge, 17 governance web, 8 integration, 7 search evaluation, 4 web health), plus self-test 49 assertions, schema 20 records and search 28 cases (20/20 top-1, 8/8 refusals, 4 role cases, 0 violations); eval passes 12 tests + 10 self-test checks; fmt and Clippy pass in both crates (E-030). The canonical run at docs HEAD `17df06a` passed docs (53 required/674 links in 115 files), secrets (231), eval (12 + 10 self-test checks), platform fmt/Clippy and 257 lib tests, then failed at `cli_contract` because the debug `ekp_platform.exe` was denied before spawn (`os error 4551`) for all 30 tests; the same path was retried with the identical result (E-031). At `6ed236d`, the restored final bundle's fresh clone ran the canonical `scripts/check.ps1` and exited 0: docs 53 required/652 links in 105 tracked files, secrets 229, eval 12, platform 270 (217 unit, 22 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health), self-test 49 assertions, schema 20, search 28 (20/20, 8/8, 4 role cases, 0 violations). At `718db57`, the working tree itself ran the canonical `scripts/check.ps1` and exited 0 (platform 277, links 657); it re-ran at `32f2b36` and `604a2a5` with 660 links. Earlier working-tree runs at `41beaea` were denied (`os error 4551`) and remain historical. At `c827351` the canonical check exited 1 because the `edge_cases` harness was denied by the same policy; the six other platform targets passed (250 unit, 28 CLI contract, 15 governance web, 8 integration, 7 search evaluation, 4 web health = 312) and the 2 edge tests were NOT RUN (E-029). |
| Defect and root cause | The earlier working-tree blocks are host-policy issues, not source-code failures: the same tests passed in the release profile and clean clones, and the current `d502147` canonical runs pass on the same host in an isolated worktree (E-036); the preceding `eec11f2` canonical run also passed on the same host (E-035); the earlier `279c919` canonical run also passed on the same host after only stale `serde` build artifacts were cleared (E-034). At `21e23d6`, the debug lib harness was denied (`os error 4551`) on two same-command attempts while 308 release-profile tests passed; 27 tests in three other harnesses were also denied. At `b2d8afc`, the `edge_cases` harness was denied (`os error 4551`) on four same-path attempts while every other target passed. At `bbe49eb`, the debug `ekp_platform.exe` itself could not start (`os error 4551`) while the release binary ran `--health` successfully; the policy blocks that binary before any test logic executes. At `c827351`, the `edge_cases` harness was denied while the other six platform targets passed. A blocked canonical run cannot be converted into a PASS by copying, renaming or relocating the binary. |
| Impact and risk | The current `d502147` revision is now proven through the canonical entry point with 356 tests, so A4 is `MET` locally; the preceding `eec11f2` revision is also `MET` with 354 tests (E-035), and the earlier `279c919` revision is also `MET` with 345 tests (E-034). Historical `21e23d6` (27 tests blocked), `b2d8afc` (2 edge tests blocked) and `bbe49eb` (debug CLI denied) still have no canonical PASS and are not retroactively upgraded. CI has never run because there is no remote, and the host policy has no named exception owner, so a future recurrence is not prevented. |
| Fix / next action | Keep the `d502147` canonical PASS (E-036), the `eec11f2` canonical PASS (E-035), the `279c919` canonical PASS (E-034) and the historical blocks (E-031/E-032/E-033); obtain a scoped Application Control exception or an approved CI/POSIX runner so recurrence is prevented, not only observed. Do not redefine canonical as release-only and do not copy or relocate a blocked harness. |
| Acceptance test | `powershell -ExecutionPolicy Bypass -File scripts/check.ps1` at the target revision, with the full output and exit code retained. |
| Threshold | Exit 0; docs, secrets, fmt, Clippy, all tests, both self-tests, schema and search gate pass in the same run. |
| External role / environment | Host/endpoint security owner for the Application Control policy; or an approved CI runner. The policy exception is `UNASSIGNED`. |
| Evidence | E-002/E-003 (release profile), E-004/E-005 (docs/secrets), E-019 (working-tree block at `41beaea`), E-020/E-023 (clean-clone canonical PASS at `41beaea`/`6ed236d`), E-024 (working-tree PASS at `4697aa1`), E-025 (working-tree PASS with `platform` 277 at `718db57`), E-026 (working-tree PASS with docs 660 links at `32f2b36`), E-028 (working-tree PASS at `604a2a5`), E-029 (earlier `c827351` block and individual target results), E-030 (`bbe49eb` release suite and 10 defect regressions), E-031 (canonical result for `bbe49eb`), E-032 (`b2d8afc` release suite, six additional regression tests and canonical result), E-033 (`21e23d6` audit fail-open fix, 308 executed release tests and canonical result), E-034 (`279c919` ten regression tests, 345 release tests and two canonical PASS runs), E-035 (`eec11f2` nine export/BPMN regression tests, red/green, 354 tests and one canonical PASS run on the exact code commit), E-036 (`d502147` three red observations, two RAG tests plus one extended date test, 356 tests and two canonical PASS runs on the exact code commit) in [test-evidence.md](test-evidence.md). |
| Owner | Engineering + QA |
| Final status / blocker | `MET` at current `d502147`: two same-command `scripts/check.ps1` runs exited 0 on the exact code commit and all 356 platform tests, eval 12 + 10 self-test checks, self-test 49, schema 20 and search 28 ran (E-036). `MET` at preceding `eec11f2`: canonical exit 0 on the exact code commit and 354 platform tests (E-035). `MET` at earlier `279c919`: canonical exit 0 on two same-command runs and 345 platform tests (E-034). Historical `21e23d6`, `b2d8afc` and `bbe49eb` remain `BLOCKED` and are not retroactively upgraded. CI is absent and the host policy still has no named exception owner, so the local PASS is not a CI or cross-host claim. See [B-11](external-blockers.md#b-11-host-application-control-policy). |

### A5 — Security, identity and isolation

| Field | Record |
| --- | --- |
| Requirement | Authentication, authorization, input validation, secret handling and data isolation are appropriate to the approved scope. |
| Actual state | `PARTIAL`. `governance_web` has a local process-memory session, PBKDF2-HMAC-SHA256 (600,000 iterations), CSPRNG salt/token, constant-time comparison, five-failure lockout, Viewer/Editor/Approver checks, same-origin guard, CSP and a 64 KiB body cap. Login failures return one generic message, lockout is checked after password-hash work and not disclosed, unknown usernames pay a dummy hash cost, usernames with whitespace/control characters are rejected, duplicate/malformed session cookies are unauthenticated, and duplicate/non-loopback `Host` headers are refused. On mutations, `Origin` and `Sec-Fetch-Site` are each accepted only when exactly one UTF-8 value is present; a repeated or non-UTF-8 value is refused. The ledger holds a single-host OS writer lock and replay validates actors, record IDs, references, workflow versions and quality-case transitions. |
| Defect and root cause | The local session is a usability barrier, not enterprise identity. There is no IdP/OIDC/SAML, MFA, account recovery, persistent identity store, access review or tenant boundary. The ledger actor is the validated local session username for web mutations (the CLI search audit uses the fixed label `search-cli`), but the session is process-memory only, so this is not an enterprise-verified identity. Duplicate or non-UTF-8 header ambiguity and duplicate Cookie header lines were accepted before the hardening and are now fail-closed; replay also requires payload actors to match the event actor. |
| Impact and risk | A local administrator can read or rewrite the ledger and process memory. Cross-tenant isolation is not implemented; the `tenant` module is library-only. The system must not be described as enterprise-authenticated or tenant-isolated. |
| Fix / next action | Target design (not implemented): OIDC authorization-code + PKCE with an approved IdP, MFA policy, server-side session store, claims-to-role mapping, mandatory tenant claim, deny-by-default resource checks, access-review and joiner/mover/leaver process. Until approved, keep the local mode explicitly non-production and loopback-only. |
| Acceptance test | Positive and negative tests for missing/invalid identity, expired sessions, role escalation, cross-tenant read/write/search/export/background work, and authorization changes; secret scan; session revocation test. |
| Threshold | All denial tests pass; zero cross-tenant leakage; no default account; no secret in repository or logs; security owner signs the residual-risk decision. |
| External role / environment | Enterprise IdP/MFA, tenant model, security owner/approver, approved test tenant. All are `UNASSIGNED`. |
| Evidence | [Threat model](../architecture/threat-model.md); [web routes](../contracts/web-routes.md); [security threat model](security-threat-model.md); 18 governance-web route tests + 5 governance-web parser unit tests + 13 local-auth unit tests at `b2d8afc`; E-032 |
| Owner | Security + Engineering; named security owner `UNASSIGNED` |
| Final status / blocker | `PARTIAL` — local-only session; no IdP/MFA/tenant isolation. See [B-03](external-blockers.md#b-03-enterprise-identity-mfa-and-tenant-model). |

### A6 — Maintainability and handoff

| Field | Record |
| --- | --- |
| Requirement | Install, configure, diagnose and hand off using the documentation. |
| Actual state | `PARTIAL`. Install/config/troubleshooting/runbook pages exist; no named operator and no independent handoff dry run. |
| Defect and root cause | Ownership gap. The documentation is written by the implementer and has not been exercised by a different operator. |
| Impact and risk | A new operator may miss environment prerequisites, host-policy constraints or recovery steps. |
| Fix / next action | Have a different operator follow the documented install, start, diagnose and restore path on a clean host; record every ambiguity and fix it. |
| Acceptance test | Timed handoff dry run with a checklist, commands, observed results and operator sign-off. |
| Threshold | Operator completes the documented workflow without undocumented help; all defects fixed or tracked. |
| External role / environment | Named operations owner/operator. `UNASSIGNED`. |
| Evidence | [Operations index](../operations/README.md); [deployment troubleshooting](../deployment/troubleshooting.md); [local backup](../operations/local-backup.md). No handoff record. |
| Owner | Operations; named owner `UNASSIGNED` |
| Final status / blocker | `PARTIAL` — no operator dry run. |

### A7 — Pilot traceability

| Field | Record |
| --- | --- |
| Requirement | Each in-scope requirement links to specification, implementation, test, run ID, commit SHA, result and validation evidence. |
| Actual state | `PARTIAL`. The [traceability register](../traceability/register.md) records S1–S27, A-05/A-06, release evidence and verified/validated status. There is no pilot run ID or validated row. |
| Defect and root cause | No real pilot execution exists, so the final `Validation` link is absent. Synthetic tests can establish `Verified` only. |
| Impact and risk | An auditor can trace code and tests, but cannot trace a real user outcome or acceptance decision. |
| Fix / next action | Add a pilot run record with requirement IDs, workflow ID, dataset hash, commit, environment, operator role, expected/actual result and evidence path; mark `Verified` vs `Validated` explicitly. |
| Acceptance test | Sample audit: pick each in-scope requirement and follow the chain to a run record and a signed acceptance. |
| Threshold | 100% of in-scope requirements have a complete chain; no secret or personal data in the evidence. |
| External role / environment | Pilot owner, operator, QA; approved data. `UNASSIGNED`. |
| Evidence | [Traceability rules](../traceability/README.md); [register](../traceability/register.md); [governance traceability baseline](../governance/10-traceability-baseline.md). |
| Owner | QA + Product; named owners `UNASSIGNED` |
| Final status / blocker | `PARTIAL` — no pilot run or validated row. |

### A8 — Backup and restore

| Field | Record |
| --- | --- |
| Requirement | A backup exists from the first important version; a fresh clone passes the restore drill; the result is logged. |
| Actual state | `MET` (local). Local fresh-clone restore PASS at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f`, `3ee2ee2`, `41beaea`, `6ed236d` and `499e0d1`. The current `499e0d1` bundle `ekpi-platform-2026-10-27-170750.bundle` (SHA-256 `09A33F…09B8`) passed `git bundle verify`, cloned to HEAD `499e0d1`, and the automatic restore drill ran the restored `scripts/check.ps1` to exit 0 (docs 657/105, secrets 229, platform 277, eval 12, self-test 49, schema 20, search 28). The final `6ed236d` bundle remains restore-verified (platform 270). Earlier `28b68cc`, `bc7671a`, `c77fec4` and `198d5f3` attempts remain `BLOCKED` and are not counted. |
| Defect and root cause | The earlier blocks were caused by host Application Control denying freshly built build scripts. That policy did not block the `41beaea` and `6ed236d` clean builds; the automatic drills completed. The underlying policy remains intermittent and host-specific. |
| Impact and risk | The final evidence revision is restore-verified on this host. Off-host recovery, RPO/RTO, key custody and scheduled drills remain absent, so this does not satisfy B7. |
| Fix / next action | Keep the `499e0d1` bundle and log row; retain the `6ed236d` and earlier rows. For Phase B, approve an off-host target and key custody, define RPO/RTO, and schedule restore drills. |
| Acceptance test | `git bundle verify`; fresh clone; `git rev-parse HEAD` equals the recorded revision; restored `scripts/check.ps1` exits 0. |
| Threshold | All four steps pass on the final bundle; log row records revision, bundle SHA-256, environment, operator and result. |
| External role / environment | Host/endpoint security owner for reproducibility; off-host storage and key custodian for B7. `UNASSIGNED`. |
| Evidence | E-007/E-008/E-009/E-020/E-023/E-027 in [test-evidence.md](test-evidence.md); [backup log](../operations/backup-log.md); [release evidence](../operations/release-evidence-bundle.md). |
| Owner | Engineering/Operations; named backup owner `UNASSIGNED` |
| Final status / blocker | `MET` (local) — `499e0d1` bundle and fresh-clone restore PASS are logged; `6ed236d` remains restore-verified. Off-host copy, approved RPO/RTO and key custody remain `BLOCKED` under [B-06](external-blockers.md#b-06-off-host-backup-key-custody-and-rporo). |

### A9 — AI and agent quality

| Field | Record |
| --- | --- |
| Requirement | A fixed evaluation set covers normal, ambiguous, adversarial, wrong-source, tool-failure and insufficient-context cases; changes rerun the applicable set. |
| Actual state | `PARTIAL`. The deterministic verdict corpus has 67 cases and covers five dimensions; the 28-case lexical retrieval gate covers normal/ambiguous/adversarial/wrong-source/insufficient-context plus four synthetic role cases. No LLM/agent/tool path exists; `--llm` accepts only `mock` and labels output `mock-answer`. |
| Defect and root cause | No approved AI provider or data policy, no model integration and no tool path. Tool-failure cases therefore cannot be exercised; the mock provider is a contract fixture, not a model. |
| Impact and risk | No claim about hallucination, reasoning, tool success, cost, latency or model regression is possible. A future provider change has no baseline. |
| Fix / next action | After provider/data approval: implement a versioned adapter, prompt/tool registry, fixed evaluation set including timeout/partial-failure/retry exhaustion, citation and refusal metrics, cost/latency budgets, human takeover and a rerun-on-change policy. |
| Acceptance test | Reproducible evaluation command with recorded provider/model/prompt/index/corpus versions; six dimensions present; regression comparison against an approved baseline. |
| Threshold | 0 violations on hard safety gates; required categories present; approved quality/cost/latency thresholds met; human review for subjective quality. |
| External role / environment | AI system owner, data owner, approved provider and data policy. `UNASSIGNED`. |
| Evidence | [AI evaluation](../quality/ai-evaluation.md); [CLI contract](../contracts/cli.md); 28-case gate result in E-003; `EVAL-GAP-01`/`02`/`03`. |
| Owner | AI system owner + QA; `UNASSIGNED` |
| Final status / blocker | `PARTIAL` — mock only; no AI/agent evaluation or tool-failure coverage. See [B-05](external-blockers.md#b-05-ai-provider-and-data-policy). |

### A10 — Documentation and handoff

| Field | Record |
| --- | --- |
| Requirement | Scope, known limits, commands, recovery and evidence are current and readable by a new operator. |
| Actual state | `PARTIAL`. The docs map, release evidence, phase gates, backup log and this assurance index are current at `604a2a5` (working tree, docs 660 links) and `499e0d1` (restore evidence); no pilot handoff record exists. |
| Defect and root cause | No real operator has consumed the documentation; the pilot record is absent. |
| Impact and risk | Documentation may be internally consistent but insufficient for an external handoff. |
| Fix / next action | Run the A6 handoff dry run; record questions, missing commands and fixes; link the signed handoff record. |
| Acceptance test | New operator completes install/start/diagnose/restore from the docs and signs the checklist. |
| Threshold | No undocumented step; all discovered defects fixed or tracked with owners. |
| External role / environment | Operations owner/operator. `UNASSIGNED`. |
| Evidence | [Documentation map](../README.md); [operations](../operations/README.md); [release evidence](../operations/release-evidence-bundle.md). |
| Owner | Engineering + Operations |
| Final status / blocker | `PARTIAL` — no pilot handoff. |

### A11 — Review and improvement

| Field | Record |
| --- | --- |
| Requirement | Major review findings are fixed; a retro is completed; accepted improvements are tracked; an independent reviewer can verify the result. |
| Actual state | `PARTIAL`. Local self-review, retro template and improvement register exist. No independent reviewer, no second annotator and no signed review. |
| Defect and root cause | The implementer performed the review. Independence is a role separation that cannot be manufactured by an AI self-check. |
| Impact and risk | Self-review can miss shared assumptions, unsafe defaults and evidence gaps. The κ metric has no second annotator. |
| Fix / next action | Assign a reviewer who did not implement the change; provide the diff, threat model, test evidence and matrix; require the reviewer to rerun commands and sign findings; track remediation. |
| Acceptance test | Completed reviewer checklist with command reruns, diff inspection, finding disposition and signature; κ study with two independent annotators. |
| Threshold | No unresolved critical/high finding; reviewer signs; any exception has an owner and expiry. |
| External role / environment | Independent reviewer and second annotator. `UNASSIGNED`. |
| Evidence | [Reviews](../reviews/README.md); [retro template](../reviews/retro-template.md); [improvement register](../reviews/README.md); [annotation tests](../quality/ai-evaluation.md#annotation-and-agreement). |
| Owner | Independent reviewer + QA; `UNASSIGNED` |
| Final status / blocker | `PARTIAL` — self-review only. See [B-08](external-blockers.md#b-08-independent-reviewer-and-second-annotator). |

#### A11 reviewer checklist (prepared, not signed)

The reviewer must be able to reproduce the evidence without trusting the implementer:

- [ ] Confirm the commit SHA and clean worktree.
- [ ] Inspect the full diff and the affected requirements/ADRs.
- [ ] Re-run `scripts/check-docs.ps1`, `scripts/check-secrets.ps1`, `scripts/check.ps1` (or record the host block), `gate.ps1`, and the release-profile suite.
- [ ] Verify each `PASS` row names a command, revision, date and exit code.
- [ ] Verify each `BLOCKED`/`NOT RUN` row is not described as a pass.
- [ ] Review the threat model, auth boundary, ledger semantics and backup scope.
- [ ] Verify no secret, real personal data or fabricated owner/reviewer exists.
- [ ] Recompute the Phase A/B decision from the weakest mandatory row.
- [ ] Record findings, severity, owner, disposition and signature/date.

---

## Phase B records

### B1 — Security and governance

| Field | Record |
| --- | --- |
| Requirement | Owned threat model; identity/MFA; least privilege; tenant isolation; secret management; audit/retention; vulnerability and supply-chain process. |
| Actual state | `BLOCKED`. Local threat model, boundary tests, secret scan, `cargo audit`/`cargo deny` and SBOM exist. No security owner, IdP/MFA, tenant model, approved data policy or production target. |
| Defect and root cause | External governance and environment decisions are absent; the local prototype intentionally stops at a local session. |
| Impact and risk | The system cannot be operated for multiple users or organizations; security claims beyond the local boundary are unsupported. |
| Fix / next action | Assign security owner; approve IdP/MFA/tenant/data policy; implement and test authorization/tenant boundaries; define audit retention and access review; run independent security review. |
| Acceptance test | AuthZ/denial matrix, cross-tenant tests, secret scan, dependency/SBOM review, access review and signed threat-model review. |
| Threshold | Zero cross-tenant leakage; no unresolved critical/high finding; owner accepts residual risk. |
| External role / environment | Security owner/approver, IAM owner, data owner, approved environment. `UNASSIGNED`. |
| Evidence | [Threat model](../architecture/threat-model.md); [security threat model](security-threat-model.md); [production readiness](../operations/production-readiness.md); E-006. |
| Owner | Security + Engineering; `UNASSIGNED` |
| Final status / blocker | `BLOCKED` — see [B-03](external-blockers.md#b-03-enterprise-identity-mfa-and-tenant-model), [B-07](external-blockers.md#b-07-security-and-operations-owners). |

### B2 — Reliability and observability

| Field | Record |
| --- | --- |
| Requirement | SLI/SLO/error budget; structured logs, metrics, traces, correlation IDs; timeouts, bounded retries, backoff, circuit breaker, rate limits; health checks; failure isolation, degradation and recovery drills. |
| Actual state | `PARTIAL`. Local JSON access log, request correlation id, aggregate Prometheus counters, liveness `/healthz`, 15 s request timeout, 64-way concurrency bound with `429`, 64 KiB body cap and graceful shutdown exist and are tested. No SLO, traces, alerting, retry/backoff, circuit breaker or fault drill. |
| Defect and root cause | No approved service level or target environment; the prototype has no dependency graph or external services to exercise. |
| Impact and risk | Reliability under failure, saturation or dependency loss is unmeasured. `/healthz` is liveness only, not readiness or an SLO. |
| Fix / next action | Approve SLIs/SLOs and error budget; add OpenTelemetry traces/metrics/log correlation and an exporter; define retry/backoff/timeout budgets; add alert rules and a fault-injection drill in an approved environment. |
| Acceptance test | Dashboard/alert drill, injected dependency failure, timeout/retry exhaustion, saturation test and recovery measurement against the approved SLO. |
| Threshold | SLO met for the approved window; alerts fire and recover; no unbounded retry or thundering herd; RTO/RPO met. |
| External role / environment | Operations owner, monitoring target, approved environment. `UNASSIGNED`. |
| Evidence | [Production readiness](../operations/production-readiness.md) PR-08/PR-09; 4 web-health tests; [architecture](../architecture/README.md). |
| Owner | Operations + Engineering |
| Final status / blocker | `PARTIAL` — no SLO, traces, alerting or fault drill. |

### B3 — Performance and scale

| Field | Record |
| --- | --- |
| Requirement | Reproducible benchmark/load/stress/soak tests; bottleneck analysis; capacity model; caching/batching/backpressure; scale decision based on cost/latency/throughput. |
| Actual state | `PARTIAL`. One local release benchmark exists: eval p50/p95 17.432/23.319 ms; loopback `/healthz` p50/p95 19.010/24.069 ms; concurrent 5,369.21 req/s. No approved workload, soak, bottleneck analysis or capacity model. |
| Defect and root cause | No workload owner, user/concurrency profile, SLO or cost budget. The benchmark is a one-host observation. |
| Impact and risk | Scalability and cost cannot be predicted; a single short run does not prove behavior under sustained load. |
| Fix / next action | Approve workload and growth assumptions; run load/stress/soak with latency percentiles, error rate and resource use; identify bottlenecks; publish a capacity/cost model with limits. |
| Acceptance test | Reproducible load/soak scripts with recorded environment, dataset, duration and results; capacity model reviewed against approved budget. |
| Threshold | Approved latency/throughput/error targets met with headroom; no unbounded degradation; cost within budget. |
| External role / environment | Product/operations workload owner; approved target. `UNASSIGNED`. |
| Evidence | [Benchmark artifact](../../artifacts/benchmark/benchmark-2026-10-27-124954.json); [release evidence](../operations/release-evidence-bundle.md#performance-observation). |
| Owner | Engineering + Operations |
| Final status / blocker | `PARTIAL` — exploratory only; formal B3 `BLOCKED` without approved workload/SLO. |

### B4 — Delivery and change governance

| Field | Record |
| --- | --- |
| Requirement | CI required checks; dependency/security scans; migration and compatibility policy; versioned API/events; feature flags, progressive rollout, rollback, post-release verification; PR review and branch protection. |
| Actual state | `BLOCKED`. `.github/workflows/ci.yml` defines validate/security/workflow-lint/SBOM jobs, but there is no Git remote and no CI execution. No branch protection or required checks exist. |
| Defect and root cause | No remote repository, no `gh` authentication, no deployment target and no human reviewer/release owner. |
| Impact and risk | Changes are not independently gated; a local pass cannot prove the CI workflow runs. Actions are tag-pinned, not SHA-pinned. |
| Fix / next action | Create/authorize a remote; push; run CI on the commit; configure branch protection/required checks; pin actions by SHA; add PR review and rollback/post-release verification. |
| Acceptance test | CI run URL/result on the target commit; branch protection settings; a PR blocked by a failing required check; rollback drill. |
| Threshold | Required checks green; no direct push to protected branch; rollback and post-release verification recorded. |
| External role / environment | Repository admin, CI runner, reviewer. `UNASSIGNED`. |
| Evidence | `.github/workflows/ci.yml`; [phase gates](../quality/phase-gates.md); no CI URL exists. |
| Owner | Engineering + repository admin; `UNASSIGNED` |
| Final status / blocker | `BLOCKED` — see [B-02](external-blockers.md#b-02-remote-ci-and-branch-protection). |

### B5 — AI and agent quality

| Field | Record |
| --- | --- |
| Requirement | Model/prompt/retrieval/tool/data versioning; fixed eval set; hallucination/citation/tool-success metrics; cost/latency budget; retry and human takeover; online/offline regression. |
| Actual state | `BLOCKED`. No model provider or agent path. `--llm` accepts only `mock`, labels output `mock-answer`, and requires a versioned citation. The deterministic retrieval gate is lexical-only. |
| Defect and root cause | No approved provider, data policy or tool integration; AI is intentionally not implemented. |
| Impact and risk | No AI quality, safety, cost or latency claim is possible. Enabling a provider without policy/evaluation would be unsafe. |
| Fix / next action | Approve provider and data boundary; implement versioned adapter and tool policy; add fixed eval including tool failure, prompt injection, sensitive-data leakage and cross-tenant attempts; define human takeover and cost/latency budgets. |
| Acceptance test | Versioned comparison report across model/prompt/index changes; fixed six-dimension eval; tool-failure and permission tests; human review record. |
| Threshold | Approved safety/quality/cost thresholds met; no unresolved high-risk finding; data policy enforced. |
| External role / environment | AI/data owner, security, approved provider. `UNASSIGNED`. |
| Evidence | [AI evaluation](../quality/ai-evaluation.md); [CLI contract](../contracts/cli.md); [source matrix](source-matrix.md). |
| Owner | AI system owner + Security; `UNASSIGNED` |
| Final status / blocker | `BLOCKED` — see [B-05](external-blockers.md#b-05-ai-provider-and-data-policy). |

### B6 — Operations and support

| Field | Record |
| --- | --- |
| Requirement | Named service/environment owner; support and escalation; start/stop/health; data locations/classification/retention; backup owner; incident procedure; maintenance and dependency ownership. |
| Actual state | `BLOCKED`. Draft runbooks and an ownership proposal exist; no named service, operations or support owner, no on-call/escalation drill and no handoff sign-off. |
| Defect and root cause | Ownership and service-level decisions are absent. |
| Impact and risk | No accountable responder for an incident, data loss or failed restore. |
| Fix / next action | Assign service/operations/data/backup owners; approve support hours and escalation; exercise an incident and restore drill; record handoff sign-off. |
| Acceptance test | On-call roster, escalation contact tree, incident drill record, restore owner, maintenance calendar and signed handoff. |
| Threshold | Every critical service has a named owner and an exercised escalation path. |
| External role / environment | Operations owner, support owner, data owner. `UNASSIGNED`. |
| Evidence | [Production readiness](../operations/production-readiness.md) PR-14/PR-15; [incident log](../operations/incident-log.md); [pilot runbook](../operations/pilot-runbook.md). |
| Owner | Operations; `UNASSIGNED` |
| Final status / blocker | `BLOCKED` — see [B-07](external-blockers.md#b-07-security-and-operations-owners). |

### B7 — Backup and disaster recovery

| Field | Record |
| --- | --- |
| Requirement | Off-host encrypted backup; defined RPO/RTO; scheduled restore tests; integrity check; data-loss boundary; incident path. |
| Actual state | `PARTIAL`. Local Git bundle + restore drills pass through `499e0d1`; the current evidence bundle verifies, clones to `499e0d1` and the automatic restore drill exits 0. `6ed236d` also remains restore-verified. The earlier `28b68cc` restore is `BLOCKED` by host policy. The `age` drill at `56a944f` encrypted/decrypted byte-identically to `%TEMP%` and is labeled `local-simulated-off-host`. |
| Defect and root cause | No approved off-host target, key custodian, RPO/RTO, retention or scheduled restore programme. |
| Impact and risk | A host loss can lose uncommitted work, the external ledger and the local bundle. The `age` drill does not prove off-host availability or key recovery. |
| Fix / next action | Approve target/failure domain and key custody; implement transfer and immutability/versioning; schedule restores; test ledger restore and hash chain; measure RTO; record off-host receipt. |
| Acceptance test | Retrieve an artifact from off-host using the documented key path; `git bundle verify`; clone; restored check exit 0; ledger hash chain verifies; RTO measured. |
| Threshold | Approved RPO/RTO met; at least one off-host restore passes; key recovery tested without the operator holding the only key. |
| External role / environment | Operations + Security + Data owner; approved storage and key custody. `UNASSIGNED`. |
| Evidence | E-007/E-008/E-009; [backup log](../operations/backup-log.md); [off-host proposal](../operations/backup-policy-draft.md). |
| Owner | Operations + Security; `UNASSIGNED` |
| Final status / blocker | `PARTIAL` — local-only; no off-host/RPO/RTO. See [B-06](external-blockers.md#b-06-off-host-backup-key-custody-and-rporo). |

### B8 — Data governance

| Field | Record |
| --- | --- |
| Requirement | Classification, lawful basis, retention/deletion, legal hold, residency, access and AI data boundaries. |
| Actual state | `BLOCKED`. Docs prohibit real customer/personal/regulated data and define a draft data boundary; no data owner, inventory, classification, retention/deletion test or legal hold. |
| Defect and root cause | No data owner or approved policy; the prototype is synthetic-data only. |
| Impact and risk | Real data cannot be lawfully or safely processed; retention, deletion and residency obligations are unknown. |
| Fix / next action | Assign data owner; inventory data; classify; approve lawful basis, retention, deletion, legal hold and residency; implement deletion/export and access review; publish AI data boundary. |
| Acceptance test | Data inventory and classification; retention/deletion test; access review; legal-hold procedure; signed policy. |
| Threshold | Every data class has an owner, purpose, location, retention and deletion method; policy approved. |
| External role / environment | Data owner + Legal/Privacy. `UNASSIGNED`. |
| Evidence | [Production readiness](../operations/production-readiness.md) PR-05; [threat model](../architecture/threat-model.md); [backup proposal](../operations/backup-policy-draft.md). |
| Owner | Data owner + Legal; `UNASSIGNED` |
| Final status / blocker | `BLOCKED` — see [B-04](external-blockers.md#b-04-data-owner-and-data-policy). |

## Decision record

| Decision | Result | Basis |
| --- | --- | --- |
| Phase A approved? | **No** | A1 `NOT MET`; A2/A3/A5/A6/A7/A9/A10/A11 `PARTIAL`; A4 `MET` at current `d502147` (canonical exit 0 twice; 356 tests; E-036), at preceding `eec11f2` (canonical exit 0; 354 tests; E-035) and at earlier `279c919` (canonical exit 0 twice; 345 tests; E-034), and `BLOCKED` only in the historical `21e23d6`/`b2d8afc`/`bbe49eb` revisions; A8 `MET` only on the local restore scope, and none of `279c919`, `eec11f2` or `d502147` is bundled; no pilot, enterprise identity or independent review |
| Phase B approved? | **No** | B1/B4/B5/B6/B8 `BLOCKED`; B2/B3/B7 `PARTIAL` with no approved workload, SLO, off-host target or RPO/RTO |
| Can the system be called production-ready? | **No** | No deployment target, enterprise identity, tenant isolation, off-host recovery, SLO or operations owner |
| Can the system be called AI-validated? | **No** | Only a mock LLM and deterministic lexical evaluation exist |
| Can the system be called compliant? | **No** | No certification, independent assessment or legal mapping |

The decision is intentionally conservative. A future revision must replace a row only with new, revision-bound evidence and a named accountable owner.
