# Pilot runbook and ownership proposal

- **Status:** Draft proposal; **not executed**, **not approved**, no owner assigned
- **Owner:** Product + Operations (proposed; acceptance required)
- **Last reviewed:** 2026-10-26
- **Phase gate:** [A1 user value](../quality/phase-gates.md#phase-a-0--1-mvp-gate), [A6 maintainability](../quality/phase-gates.md#phase-a-0--1-mvp-gate), [A10 handoff](../quality/phase-gates.md#phase-a-0--1-mvp-gate)
- **Readiness:** [Production readiness](production-readiness.md)
- **Related:** [MVP scope](../product/mvp-scope.md), [decision register](../governance/08-decision-register.md), [local backup](local-backup.md), [secrets policy](secrets-policy.md)

## Purpose and limit

This document is a runbook for one pilot workflow and a proposed ownership model. It does not claim that a pilot occurred. A pilot is valid only when a representative user executes an authorized workflow on non-sensitive data, the evidence is retained, and the user or owner accepts the result in writing.

The current repository has **no named pilot owner, no approved pilot workflow and no real-user validation**. A1 remains `Not met` until those exist. Do not fill the sign-off table with placeholder names.

## Non-negotiable preconditions

All of the following must be true before the first pilot session:

1. A named product owner and pilot operator accept the scope in writing.
2. The pilot workflow is selected and recorded against `DEC-01`; it is one end-to-end path, not a broad demo.
3. The data is synthetic or authorized non-sensitive data. Real customer, personal, regulated or confidential data is prohibited.
4. A security/data reviewer confirms the data handling, retention and deletion method.
5. The exact revision, build command and data manifest are recorded.
6. A local backup exists and a restore drill has passed for that revision; the governance ledger path is recorded separately.
7. A stop condition and rollback path are agreed before the session.
8. The participant understands that this is a prototype with no enterprise authentication, tenant isolation or production monitoring.

If any precondition is missing, the session is `Blocked`, not a partial pilot.

## Proposed ownership (requires written acceptance)

| Role | Proposed responsibility | Named person | Accepted on |
| --- | --- | --- | --- |
| Product owner | Owns pilot scope, success criteria and go/no-go | Not assigned | — |
| Pilot operator | Runs the workflow, captures evidence, stops on an incident | Not assigned | — |
| QA reviewer | Checks evidence sufficiency and expected outcomes | Not assigned | — |
| Security/data owner | Approves data handling, access and retention | Not assigned | — |
| Operations owner | Owns backup, runbook, escalation and handoff | Not assigned | — |
| Engineering owner | Fixes defects and preserves the tested revision | Not assigned | — |

One person may hold more than one role in a local prototype, but the conflict must be recorded. High-risk decisions require independent review.

## Pilot workflow

Use the [MVP scope](../product/mvp-scope.md) to select **one** workflow. Record the selection and rationale before running it.

1. **Freeze the revision.** Record `git rev-parse HEAD`, the bundle SHA-256, the build command and the lockfile hashes. Do not test uncommitted changes.
2. **Prepare the data.** Create a synthetic or authorized non-sensitive dataset; record file names, sizes and SHA-256. Keep it outside the repository if it is not part of the approved fixture set.
3. **Set the ledger path.** Use a disposable or dedicated path, for example:
   ```powershell
   $env:EKP_GOVERNANCE_DATA = Join-Path $env:TEMP ("ekpi-pilot-" + [guid]::NewGuid().ToString("N") + ".jsonl")
   ```
4. **Run the pre-flight checks.** In the frozen checkout run `scripts/check.ps1` (or `gate.ps1 -IncludeSmoke` for report/export changes). Record the exit code.
5. **Execute the workflow.** Use the CLI and, only on loopback, the governance workbench. Capture the exact commands, timestamps, outputs and the ledger path.
6. **Check the outcome.** A QA reviewer verifies the verdict, citations, evidence sufficiency and any refusal against the expected result. Record discrepancies; do not edit the expected result after the fact.
7. **Ask the user.** Conduct a short structured interview: what was useful, what was missing, what was confusing, whether the outcome is fit for the intended use. Record the questions and raw answers.
8. **Decide.** The product owner records `accepted`, `accepted with conditions` or `rejected`, with the residual risks and the next action.
9. **Retain evidence.** Link the run record in the [traceability register](../traceability/register.md) and, if a defect or improvement exists, in the [improvement register](../reviews/README.md).
10. **Back up and restore-test.** Create a bundle after the run, record its SHA-256, and run the restore drill on a fresh clone. Do not treat the bundle's existence as verification.

## Evidence record

| Field | Required content |
| --- | --- |
| Revision | `git rev-parse HEAD` and branch |
| Bundle | Path, SHA-256 and `git bundle verify` result |
| Data | Source, classification, SHA-256, authorization reference |
| Environment | Host, OS, Rust toolchain, working directory |
| Commands | Exact commands, including environment variables |
| Outputs | Console output, reports, ledger path and hash-chain check |
| Expected result | Named acceptance case or workflow criterion |
| Observed result | Pass/fail/refusal and any discrepancy |
| Participant | Role, not unnecessary personal data |
| Decision | Accepted / accepted with conditions / rejected, with rationale |
| Follow-up | Defect/improvement ID, owner and review date |

## Stop conditions

Stop immediately and record an incident if any of these occurs:

- real personal, customer or regulated data appears;
- a secret or credential is exposed;
- the ledger chain fails verification or a file is unexpectedly rewritten;
- a verdict or citation contradicts the source and the cause is unknown;
- the process crashes, loses data or cannot be restarted;
- an unauthorized user or host can reach the workbench;
- the operator cannot explain how to recover the previous revision.

Do not continue a pilot to make a metric look complete.

## Acceptance checklist

- [ ] Scope, owner, operator and data authorization are recorded.
- [ ] Revision, bundle, data and environment are identified.
- [ ] Pre-flight check passed on the frozen revision.
- [ ] Workflow executed end to end by a representative user.
- [ ] Evidence and ledger path retained; no real data.
- [ ] QA review completed; discrepancies recorded.
- [ ] User interview completed and retained.
- [ ] Product owner decision recorded with residual risks.
- [ ] Defects/improvements linked to owners.
- [ ] Backup and restore drill passed for the tested revision.

An unchecked box is not a pass. A1 stays `Not met` until the completed record exists and is linked from the phase gate and traceability register.

## Sign-off (not yet completed)

| Decision | Name / role | Date | Scope and residual risk |
| --- | --- | --- | --- |
| Pilot authorized | — | — | — |
| Data handling approved | — | — | — |
| Pilot result accepted | — | — | — |

## Update triggers

Update this runbook when the pilot scope, owner, data policy, environment, backup procedure or evidence requirements change. Record the executed session as a dated review or traceability row; do not rewrite this proposal as if it had been executed.
