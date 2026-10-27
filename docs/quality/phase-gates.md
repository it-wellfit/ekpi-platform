# Phase A and Phase B gates

- **Status:** Active; Phase A **not approved**, Phase B **blocked**
- **Owner:** Product + QA + Engineering
- **Last reviewed:** 2026-10-27
- **Definition of Done:** [Engineering workflow](../development/README.md#definition-of-done)
- **Production readiness:** [Operations readiness](../operations/production-readiness.md)

These gates separate “the prototype works” from “the product is ready for the intended use.” They are not a waterfall: every new feature re-enters the relevant rows. A gate is approved only by the named accountable role, with linked evidence and a review date.

## Status vocabulary

| Status | Meaning |
| --- | --- |
| `Met` | The required evidence exists, was executed on the named revision, and is linked |
| `Partial` | Some evidence exists, but the row’s full condition or scope is not met |
| `Not met` | The condition is known to fail or the required evidence is absent |
| `Not started` | No work or evidence exists yet |
| `Blocked` | Cannot proceed without an external decision, access, owner or environment |
| `N/A` | The condition does not apply to the approved scope; the reason is recorded |

## Phase A: 0 → 1 MVP gate

Phase A asks: can a real, authorized workflow be completed, reproduced and handed to another operator using this repository, without pretending the prototype is an enterprise service?

| ID | Dimension | Required condition | Evidence | Status (2026-10-26) | Blocker / next action |
| --- | --- | --- | --- | --- | --- |
| A1 | User value | At least one complete real workflow is executed by a representative user and the outcome is recorded | User/owner validation record, not only synthetic tests | `Not met` | Select a pilot workflow and owner (`DEC-01`); run it on authorized non-sensitive data |
| A2 | Functional correctness | Primary use cases and high-risk boundary cases pass for the approved scope | 67-case corpus, unit/integration tests, self-test, hard gates | `Partial` | Prototype scope passes; target acceptance matrix AT-01–AT-31 remains `NOT RUN` |
| A3 | Data correctness | Version binding, rule activity, state transitions, evidence sufficiency and error handling match the spec | Integration tests, schema check, snapshot replay | `Partial` | Add a real-workflow dataset and independent review; no DB transaction semantics exist |
| A4 | Engineering quality | Format, Clippy (warnings denied), unit/integration tests and self-tests pass | `b2bae3e` (2026-10-27, historical): canonical `gate.ps1 -IncludeSmoke` 13/13 PASS; eval 12, platform 261 (217 unit, 13 CLI contract, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health); docs 494 links in 106 files; secrets 220; supply chain PASS; SBOM at `a826472`. `23b8996` (latest delivery): release-profile `cargo fmt`/Clippy PASS in both crates; `cargo test --release --locked` eval 12, platform **270** (217 unit, **22 CLI contract**, 2 edge, 10 governance web, 8 integration, 7 search evaluation, 4 web health); self-test 49 assertions, schema 20 and search 28 cases PASS; docs 507 links in 106 files at that time; secrets 220. **Canonical `scripts/check.ps1` BLOCKED** (`os error 4551` on the debug binary; `EKP_PLATFORM_BIN` let tests pass but `cargo run -- --selftest` was still denied). | `Partial` | At `23b8996` the canonical check did not run to completion on this host; release-profile results are equivalent coverage, not a canonical PASS. CI has never run because there is no remote. |
| A5 | Security | Authentication, authorization, input validation, secret handling and data isolation are appropriate to the approved scope | Threat model, boundary tests, secret scan, manual review | `Partial` | `governance_web` now requires a local session for every ledger-backed page, enforces Viewer/Editor/Approver roles, stores PBKDF2-HMAC-SHA256 hashes with CSPRNG salts, uses `HttpOnly`/`SameSite=Strict` cookies, caps form bodies at 64 KiB and refuses unlocked cross-site mutations. There is still no enterprise IdP, SSO, MFA, account recovery, persistent identity database, access review or tenant isolation; this remains local-only and does not satisfy `DEC-12`. |
| A6 | Maintainability | Install, configure, diagnose and hand off from the documentation | Install/config/troubleshooting docs; operator walkthrough | `Partial` | No named operator yet; run a handoff dry run |
| A7 | Traceability | Each in-scope requirement links to spec, code, test, release evidence and validation | [Traceability register](../traceability/register.md) and [governance/10](../governance/10-traceability-baseline.md) | `Partial` | Complete rows for the pilot scope; distinguish verified vs validated |
| A8 | Backup and restore | A backup exists from the first important version; a fresh clone passes the restore drill; the result is logged | [Backup log](../operations/backup-log.md): `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f` and `3ee2ee2` PASS (the last by manual fresh clone after the automatic TEMP drill was blocked; each verified clone's restored `scripts/check.ps1` exited 0). The final `c77fec4` bundle verifies and its clone reached `c77fec4`, but the automatic TEMP drill and the manual clone both failed at `eval` Clippy because host Application Control denied build-script execution (`os error 4551`); release-profile tests in the clone were blocked too. That row is recorded as **BLOCKED** and is not a restore PASS. Earlier `198d5f3` is also BLOCKED. | `Partial` | The final delivery revision is not restore-verified. Local-only evidence; off-host encrypted copy, approved RPO/RTO, scheduled restore tests and a named backup owner remain Phase B (`B7`, `PR-07`, `DEC-14`). The `age` drill under `%TEMP%` is not an off-host copy |
| A9 | AI quality (if AI is in scope) | Fixed evaluation set covers normal, ambiguous, adversarial, wrong-source, tool-failure and insufficient-context cases; changes rerun the applicable set | [AI evaluation](ai-evaluation.md); 67-case verdict corpus; 28-case retrieval set with 4 role-restricted permission cases | `Partial` | Verdict corpus covers 5 of 6 mandatory dimensions; the retrieval set adds the five deterministic dimensions plus permission coverage, but is lexical-only. Tool-failure is absent and no LLM/agent exists (`EVAL-GAP-01`); `EVAL-GAP-06` is closed as fixed-set permission coverage |
| A10 | Documentation and handoff | Scope, known limits, commands, recovery and evidence are current and readable by a new operator | Docs map, release template, runbooks | `Partial` | Add the pilot handoff record and close stale operational claims |
| A11 | Review and improvement | Major review findings are fixed; a retro is completed; accepted improvements are tracked | PR/review report; [reviews](../reviews/README.md) | `Partial` | No GitHub review/issue execution yet; local review and improvement register exist |

### Phase A decision

**Not approved.** The prototype is testable and locally runnable. The canonical engineering gate passed locally at `b2bae3e` (A4 `Met` there), but at the latest delivery revision `23b8996` A4 is `Partial` because host Application Control blocked the canonical check; the release-profile suite passed instead. Local bundle restore drills passed at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f` and `3ee2ee2` (A8 `Met` only for those revisions, local-only); the final `c77fec4` restore is Blocked by host policy, so A8 is `Partial` at the latest revision. A1 (real user value) is still `Not met`, and A7 (complete traceability for the pilot scope) plus the pilot-scope parts of A2/A3/A5/A6/A10 remain `Partial`. Passing A4/A8 alone is not an MVP acceptance, and neither row closes the Phase B off-host/RPO/RTO gaps.

Phase A may be approved only when every row in the approved scope is `Met` or has an explicitly accepted, time-bound exception with the accountable owner. A1 cannot be waived by a test result.

## Phase B: MVP → production-grade gate

Phase B asks: can this be operated safely for multiple users/organizations, at the approved scale, with evidence for failure, recovery, security and change?

| ID | Dimension | Required condition | Evidence required | Status (2026-10-26) | Blocking decision |
| --- | --- | --- | --- | --- | --- |
| B1 | Security and governance | Threat model owned; identity/MFA; least privilege; tenant isolation; secret management; audit/retention; vulnerability and supply-chain process | Threat model review, authZ/denial tests, tenant isolation tests, scan/SBOM results, access review | `Blocked` | No deployment target, IdP, data policy or security owner (`DEC-12`, `DEC-13`, `DEC-14`, `DEC-20`) |
| B2 | Reliability and observability | SLI/SLO/error budget; structured logs, metrics, traces, correlation IDs; timeouts, bounded retries, backoff, circuit breaker, rate limits; health checks; failure isolation, degradation and recovery drills | Dashboards, alert drills, fault injection, RTO/RPO and data-loss test | `Partial` | Local request timeout, concurrency bound, JSON access log, correlation id, aggregate `/metrics` and liveness `/healthz` exist and are tested. No SLO, alerting, traces, retry/backoff, circuit breaker, fault-injection drill or target environment (`DEC-16`) |
| B3 | Performance and scale | Reproducible benchmark/load/stress/soak tests; bottleneck analysis; capacity model; caching/batching/backpressure; scale decision based on cost/latency/throughput | Load scripts, capacity report, trend and cost model | `Partial` | One local release benchmark exists (eval p50/p95 and loopback `/healthz` throughput); no approved workload, sustained load, soak, bottleneck analysis or capacity model (`DEC-19`) |
| B4 | Delivery and change governance | CI required checks; dependency/security scans; migration and compatibility policy; versioned API/events; feature flags, progressive rollout, rollback, post-release verification; PR review and branch protection | CI runs, migration/rollback drill, deployment record, release checklist | `Blocked` | No remote, `gh` auth, branch protection or deployment target; CI has not run |
| B5 | AI and agent quality | Model/prompt/retrieval/tool/data versioning; fixed eval set; hallucination/citation/tool-success metrics; cost/latency budget; retry and human takeover; online/offline regression | Version comparison, error taxonomy, regression report, cost/latency evidence | `Blocked` | AI is not implemented and no provider/data policy is approved (`DEC-15`) |
| B6 | Operations and support | Named service/environment owner; support and escalation; start/stop/health; data locations/classification/retention; backup owner; incident procedure; maintenance and dependency ownership | Runbook, on-call/escalation record, incident drill, handoff sign-off | `Blocked` | No operating owner or service level (`DEC-16`) |
| B7 | Backup and disaster recovery | Off-host encrypted backup; defined RPO/RTO; scheduled restore tests; integrity check; data-loss boundary; incident path | Backup/restore log, RPO/RTO approval, off-host evidence | `Partial` | Local git bundle/restore drill only; no off-host copy or approved RPO/RTO (`DEC-14`) |
| B8 | Data governance | Classification, lawful basis, retention/deletion, legal hold, residency, access and AI data boundaries | Data inventory, retention schedule, deletion test, access review | `Blocked` | `DEC-13`; no data owner |

### Phase B decision

**Blocked.** B1/B4/B5/B6/B8 cannot start without decisions and owners that are not in this repository. B2/B3 require an approved target and workload. Do not describe the system as production-ready, secure, reliable, scalable or compliant until these rows have linked evidence.

## Gate evidence standard

For each row that claims `Met`:

1. Name the requirement/decision ID and the accountable owner.
2. Link the command, test, artifact, review or drill record, with revision/date/environment.
3. State the result and any residual limitation.
4. For a failed or skipped check, record `Not met`/`Blocked`, the defect/issue and the next action. Never mark it `PASS`.
5. For an exception, record the reason, impact, risk acceptor, compensating control, expiry and review date. Non-waivable legal/security controls cannot be waived by a normal exception.

## Sign-off

| Gate | Accountable roles | Evidence location |
| --- | --- | --- |
| Phase A | Product owner + QA + Engineering | This page, traceability register, review report |
| Phase B | Product owner + Security + Operations + Engineering | This page, production readiness, release record |

Named people are not recorded because no pilot owner, security owner or operations owner has been assigned. That absence is itself a blocker, not a documentation gap to fill with a placeholder name.

## Review cadence

- Re-evaluate the affected rows at every release and after any material incident.
- Re-run the full gate before a pilot expansion, a new tenant, a new connector, a model/provider change or a deployment change.
- Review this page at least quarterly, or when a decision in `governance/08` changes.
- A retro may add a control or a check, but it cannot lower a gate without an explicit, approved decision.

## Related documents

- [Engineering workflow and Definition of Done](../development/README.md#definition-of-done)
- [Quality gates and risk tiers](../development/quality-gates.md)
- [AI evaluation](ai-evaluation.md)
- [Production readiness](../operations/production-readiness.md)
- [Release and retro](../development/release-and-retro.md)
- [Decision register](../governance/08-decision-register.md)
