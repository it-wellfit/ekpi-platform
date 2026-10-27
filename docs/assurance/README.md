# Assurance evidence index

- **Status:** Active; Phase A **not approved**, Phase B **blocked**
- **Owner:** QA + Engineering (accountable business, security and operations roles are `UNASSIGNED`)
- **Last reviewed:** 2026-10-27
- **Evidence baseline:** code `6ae6134`; current working-tree canonical revision `604a2a5` (canonical PASS, docs 660 links in 115 files, platform 277); final evidence revision `6ed236d` (clean-clone canonical PASS and automatic restore PASS, 2026-10-27); current evidence bundle `499e0d1` (automatic restore PASS, docs 657/105, platform 277); previous assurance bundle `41beaea`
- **Canonical gates:** [Phase A/B gates](../quality/phase-gates.md), [production readiness](../operations/production-readiness.md)

This directory is the cross-phase assurance record. It does not replace the canonical requirements, contracts or runbooks. It records what was actually executed, what failed or was blocked, and what cannot be claimed without an external owner, environment or approval.

## What is here

| Document | Purpose | Do not use it for |
| --- | --- | --- |
| [Phase A/B matrix](phase-a-b-matrix.md) | Requirement-by-requirement status, defect, fix, test, threshold and blocker for A1–A11 and B1–B8 | A substitute for the canonical [phase gates](../quality/phase-gates.md) or [governance acceptance matrix](../governance/04-acceptance-test-matrix.md) |
| [Root-cause analysis](root-cause-analysis.md) | Why the observed failures and gaps exist, what was fixed, and what remains environmental or external | A claim that a blocked check passed |
| [Source matrix](source-matrix.md) | Official source versions, applicability and the control/decision each one supports | A certification or compliance statement |
| [Test evidence](test-evidence.md) | Exact commands, environment, revisions, exit codes, results and evidence locations | A replacement for raw command output or CI logs |
| [External blockers](external-blockers.md) | Concrete owners, resources, approvals and unlock steps | A request for passwords, tokens or private keys |
| [Security threat model](security-threat-model.md) | Current trust boundaries, threats, residual risk and target controls | A production threat model or penetration-test report |
| [Operations and recovery](operations-and-recovery.md) | Operating boundary, monitoring, incident, backup, restore and disaster-recovery procedure | Evidence that an off-host backup or DR programme exists |
| [Final report](final-report.md) | Consolidated result, non-claims and next actions | Approval by an accountable human role |

## Status vocabulary

| Status | Meaning in this directory |
| --- | --- |
| `PASS` | The exact named command or drill ran on the named revision and exited 0; the evidence is linked |
| `FAIL` | The check ran and produced a non-zero result or a wrong outcome |
| `BLOCKED` | The check could not run because of a host policy, missing access, missing environment or missing external owner |
| `NOT RUN` | No execution has been attempted; no result is implied |
| `PARTIAL` | Some required evidence exists, but the full condition or scope is not met |
| `NOT MET` | The condition is known to fail or the required evidence is absent |
| `NOT APPLICABLE` | The condition does not apply to the approved scope; the reason is recorded |

`PASS` is revision-bound. A pass on an earlier revision, a release-profile substitute, a local clone, a simulated drill or a configuration file does not transfer to a later revision, a CI run, a staging environment or production. A clean-clone canonical pass is valid for that revision but is not CI evidence.

## Evidence classes

| Class | What it means here |
| --- | --- |
| `LOCAL` | Ran on the single managed Windows host against the local repository or a local clone |
| `SIMULATED` | A drill using a synthetic destination or fixture; it proves the procedure, not the external service |
| `CI` | Ran in a hosted CI system on a named commit; **none exists for this repository** |
| `STAGING` | Ran in an approved pre-production environment; **none exists** |
| `PRODUCTION` | Ran in the approved production environment; **none exists** |
| `INDEPENDENT REVIEW` | Performed and signed by a reviewer who did not implement the change; **none exists** |

## Non-claims

Nothing in this directory supports the following statements:

- production-ready, secure, compliant, enterprise-authenticated, tenant-isolated, scalable or AI-validated;
- a hosted CI run, GitHub pull request, branch protection rule, issue or review;
- a real pilot, representative user, customer acceptance or named business owner;
- an off-host backup, approved RPO/RTO, key custody or disaster-recovery exercise;
- a penetration test, ASVS certification, NIST certification or independent security review;
- LLM, agent or tool quality; the only LLM path is a labeled `mock-answer` fixture.

## Update rules

1. Update the affected rows and [test evidence](test-evidence.md) in the same change as the code, test or drill.
2. Add a new evidence row with the revision, date, environment, command, exit code and result; do not overwrite an earlier result.
3. Never convert `BLOCKED`, `FAIL` or `NOT RUN` into `PASS` without a new execution on the named revision.
4. Update the [traceability register](../traceability/register.md) and [release evidence](../operations/release-evidence-bundle.md) when a release claim changes.
5. Keep named people out of the record unless a real person has accepted the role. Use `UNASSIGNED` for missing accountability.
