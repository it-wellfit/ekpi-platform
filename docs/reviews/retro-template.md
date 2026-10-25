# Retrospective template

- **ID:** RETRO-NNNN
- **Date:**
- **Facilitator:**
- **Participants / roles:**
- **Scope / period:**
- **Status:** Draft | Accepted | Closed
- **Evidence:** revision, reports, incidents, metrics, issues

> A retrospective is not a status report. Record what the evidence shows, what you believe caused it, and what will change. Do not claim user value, production readiness or team performance that was not measured.

## 1. Context

- What work was in scope?
- Which release, incident, gate or milestone triggered this retro?
- What was explicitly out of scope?

## 2. Observed facts

List only things that can be checked: commands run, results, dates, revisions, incidents, review findings, missed or failed checks. Link evidence.

- 

## 3. What went well

- 

## 4. What did not go well

For each item, state the observable effect and the affected requirement/gate.

| Item | Observable effect | Evidence | Affected gate/requirement |
| --- | --- | --- | --- |
| | | | |

## 5. Causes

Separate facts from hypotheses. Do not blame a person; identify the process, control or decision that allowed the outcome.

| Cause | Fact or hypothesis | Evidence / test that would confirm it |
| --- | --- | --- |
| | | |

## 6. Actions

Keep this list small and owned. Each action needs an observable completion condition and a tracking link.

| ID | Action | Owner | Due / trigger | Completion evidence | Status |
| --- | --- | --- | --- | --- | --- |
| IMP-NNN | | | | | Open |

Add each action to the [improvement register](README.md#improvement-register) and, when a remote exists, create a GitHub Issue.

## 7. Gate or control changes

- Does this retro require a new check, test, review rule or gate?
- Does it propose lowering a gate? If so, stop: lowering a gate requires an approved decision with a risk acceptor, not a retro action.
- Which document/contract/traceability row must be updated?

## 8. What remains blocked

List external dependencies (owner, access, decision, environment) with the responsible role. Do not disguise a blocker as an action.

| Blocker | Responsible role | Decision/access needed | Review trigger |
| --- | --- | --- | --- |
| | | | |

## 9. Follow-up

- Next review date or trigger:
- Actions closed since the last retro (with evidence):
- Actions carried forward:

## Rules

1. A retro action is `Done` only with linked evidence.
2. A retro cannot waive a legal, security or data-protection control.
3. Do not include secrets, real customer data or unredacted personal data.
4. If the retro finds that a released number or decision was wrong, correct the release note and the traceability register.
