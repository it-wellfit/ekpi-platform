# Incident log

- **Status:** Active; no incidents recorded
- **Owner:** Operations (role not yet assigned)
- **Last reviewed:** 2026-10-26
- **Procedure:** [Operations index](README.md#incident-handling-current-local-scope)
- **Production readiness:** [Production readiness](production-readiness.md)

## Scope

This log records local prototype incidents: data loss or corruption, suspected ledger/report tampering, secret exposure, unauthorized local access, a false decision released by a served surface, and any event that required stopping a process or restoring a backup.

It is not a production incident system. There is no severity policy, on-call rotation, notification path or regulatory reporting owner. Those are Phase B controls (`DEC-16`, `DEC-20`).

## Recording rules

1. Record the incident even if the root cause is unknown. Do not wait for a complete narrative.
2. Never paste secrets, credentials, real personal data or customer content into this file.
3. Preserve evidence before remediation: copy the ledger/report, record the hash or file size, and note the time and operator.
4. Distinguish observed facts from hypotheses.
5. Link the corrective action to the [improvement register](../reviews/README.md#improvement-register). Do not close an incident because the symptom disappeared.
6. An incident that affects a released artifact or a public number requires a release note correction and, if the evidence was wrong, a traceability update.

## Incident table

| ID | Date/time | Detected by | Severity (local) | Summary | Affected revision / path | Evidence preserved | Immediate action | Root cause | Corrective action / owner | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| — | — | — | — | No incidents recorded | — | — | — | — | — | — |

## Severity vocabulary (local)

| Level | Meaning | Expected response |
| --- | --- | --- |
| S1 | Wrong decision released by a served surface; secret committed; ledger integrity lost | Stop the affected process, preserve evidence, rotate exposed secrets, notify the owner |
| S2 | Data loss/corruption; report or export materially wrong; backup restore fails | Stop writes, preserve evidence, restore from a verified bundle, record the drill |
| S3 | Local-only defect or near miss with no released incorrect result | Record, fix with a regression test, review the gate that should have caught it |
| S4 | Documentation or process defect with no direct data impact | Record, fix, and add a check if recurrence is plausible |

These levels are for the local prototype only. A production incident system requires an approved severity model and notification obligations.

## Post-incident review

After an S1 or S2 incident, add a review report under [reviews](../reviews/README.md) and answer:

- What happened, in observable terms, and when?
- Which control should have prevented or detected it, and why did it not?
- Was any released number, decision or artifact wrong? Which traceability row is affected?
- What evidence proves the fix works, and which check would catch a recurrence?
- What remains blocked by an external owner or decision?

## Update triggers

Add a row for every incident or near miss. Update the severity vocabulary only with an approved operations decision; do not silently lower a severity to avoid a review.
