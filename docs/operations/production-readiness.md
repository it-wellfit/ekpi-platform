# Production readiness

- **Status:** Blocked
- **Owner:** Operations + Security (roles not yet assigned)
- **Last reviewed:** 2026-10-26
- **Phase gate:** [Phase B](../quality/phase-gates.md#phase-b-mvp--production-grade-gate)
- **Threat model:** [Local prototype threat model](../architecture/threat-model.md)
- **Decision register:** [governance/08](../governance/08-decision-register.md)

## Current answer

**The system is not production ready and must not be described as such.**

The repository contains a tested local prototype: a deterministic adjudication core and a loopback-only governance workbench. It has no approved deployment target, enterprise identity, tenant isolation, off-host backup, operational telemetry, incident owner or service level. Those are not missing documentation paragraphs; they are unimplemented controls and unassigned accountabilities.

This page is the readiness checklist and evidence index. It does not replace the [Phase B gate](../quality/phase-gates.md), which is the approval record.

## Readiness dimensions

| ID | Dimension | Required before any production claim | Current state | Blocker |
| --- | --- | --- | --- | --- |
| PR-01 | Approved purpose and scope | Named product owner, approved use, excluded uses, data classification | Not approved | `DEC-01`, `DEC-13`; no product owner named |
| PR-02 | Deployment target and environment | Named environment, network boundary, TLS, host hardening, capacity, cost owner | None | No target approved; local process only |
| PR-03 | Identity and authorization | Enterprise IdP/MFA, role model, least privilege, joiner/mover/leaver, access review | Not implemented in served app; `local_auth` is library-only; retrieval's `PermissionContext` is a library filter with no identity source | `DEC-12`; no IdP or security owner |
| PR-04 | Tenant and data isolation | Tenant-scoped storage and authorization tests, or an explicit single-tenant decision | None; `tenant` is library-only | `DEC-12`; no tenant model approved |
| PR-05 | Data governance | Inventory, lawful basis, retention/deletion, legal hold, residency, AI data boundary | Draft policy only | `DEC-13`; no data owner |
| PR-06 | Audit and retention | Tamper-evident audit store with retention, access control and external anchoring where required | Local JSONL hash chain; not WORM | `DEC-14`; no retention decision |
| PR-07 | Backup and disaster recovery | Off-host encrypted backup, approved RPO/RTO, scheduled restore tests, data-loss boundary | Local bundle restore drill PASS at `283f83a`; latest `198d5f3` bundle restore **BLOCKED** by host Application Control (`os error 4551`); no off-host encrypted copy, approved RPO/RTO or scheduled restore test | `DEC-14`; [backup log](backup-log.md) has local evidence only and one environment-blocked drill |
| PR-08 | Observability and SLO | SLI/SLO/error budget, structured logs, metrics, traces, correlation IDs, health/readiness, alerting | None; local stdout only | `DEC-16`; no monitoring target |
| PR-09 | Reliability controls | Timeouts, bounded retries with backoff, idempotency, rate limits, circuit breaking, degradation and recovery drills | Minimal; no cross-process lock or retry model | `DEC-16`; no target environment |
| PR-10 | Performance and capacity | Approved workload, reproducible load/stress/soak tests, capacity and cost model | None; no approved target | `DEC-19`; no workload owner |
| PR-11 | Security assurance | Threat model review, SAST/dependency/secret scanning, penetration test, vulnerability disposition, SBOM where required | Local threat model; CI scans defined but not executed against a remote; no pen test | `DEC-20`; no security owner |
| PR-12 | Supply chain | Pinned dependencies, audited lockfiles, pinned actions, provenance review for skills/tools | Lockfiles present; local `cargo audit` PASS on both lockfiles 2026-10-26 (0 vulnerabilities); actions tag-pinned; no SBOM | Security owner and policy required; CI security job not run |
| PR-13 | Change and release governance | Required CI checks, branch protection, review rules, migration/compatibility policy, progressive rollout, rollback and post-release verification | Local checks and templates exist; no remote, branch protection or deployment pipeline | Remote/auth unavailable; `DEC-20` |
| PR-14 | Incident response | Severity model, on-call/escalation, notification path, evidence preservation, drill | Local checklist only | `DEC-16`; no incident owner |
| PR-15 | Support and handoff | Named service owner, support hours, runbooks, training and handoff sign-off | None | No operations owner |
| PR-16 | AI/agent controls (if enabled) | Provider and data approval, fixed evaluation, human takeover, cost/latency budget, versioning | AI not implemented; no provider approved. A deterministic 24-case lexical retrieval gate exists, but it is not an LLM/agent evaluation | `DEC-15` |
| PR-17 | Legal and compliance | Applicable requirements identified, control mapping, non-waivable controls, certification scope if claimed | Not assessed | `DEC-09`, `DEC-13`; no compliance owner |

## Evidence standard

A dimension becomes `Ready` only when:

1. The accountable owner is named and accepts the residual risk.
2. The control is implemented, not merely documented.
3. The evidence names the environment, revision, date, command or drill, and result.
4. A failure or skipped check is recorded as `Not met` or `Blocked`, never converted to `PASS`.
5. Exceptions have a risk acceptor, compensating control, expiry and review date; non-waivable legal/security controls cannot be excepted by a normal approval.

## Minimum production gate

Before the first production release, all of the following must be true:

- PR-01 through PR-07 have named owners and linked evidence; PR-03 and PR-04 are implemented and tested, not planned.
- PR-08 and PR-09 have an approved SLO, dashboards, alerts and at least one executed failure/recovery drill.
- PR-10 has an approved workload and a capacity result within the approved budget.
- PR-11 has a completed security review with findings dispositioned; no unresolved critical or high finding remains without an accepted exception.
- PR-13 has required CI checks, branch protection, a rollback drill and a post-release verification record.
- PR-14 and PR-15 have named owners and an exercised escalation path.
- PR-16 is either explicitly out of scope or fully controlled; enabling AI without evaluation is prohibited.

## What would change this page

A production claim requires an approved deployment target, an assigned security owner and operations owner, the decisions listed above, and executed evidence. Until then, this page stays **Blocked**, and the Phase B gate stays **Blocked**.

Do not replace a blocker with a placeholder owner, a fabricated scan, a simulated restore or an aspirational paragraph. If a control cannot be evidenced, the correct status is `Blocked`.

## Related documents

- [Phase gates](../quality/phase-gates.md)
- [Threat model](../architecture/threat-model.md)
- [Security integration spec](../governance/07-security-integration-spec-v1.md)
- [Decision register](../governance/08-decision-register.md)
- [Local backup](local-backup.md), [backup log](backup-log.md)
- [Secrets policy](secrets-policy.md)
- [Incident log](incident-log.md)
- [Release and retro](../development/release-and-retro.md)

## Update triggers

Update this page when an owner is assigned, a control is implemented, a decision is made, a drill or scan is executed, or the deployment boundary changes. Link the evidence; do not change the status without it.
