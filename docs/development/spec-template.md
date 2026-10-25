# Product or engineering spec template

> Copy this into a GitHub Issue or a linked document. Delete guidance after replacing it. Do not turn unknowns into assumed behavior.

## Record

- Spec owner:
- Status: Draft / In review / Accepted / Superseded
- Issue:
- Reviewers and decision owner:
- Last updated:

## Problem and evidence

- Affected users and roles:
- Current situation and real-world example:
- Impact, frequency, and available workaround:
- Evidence source and date (interview, observation, support record, metric, or authoritative document):
- Baseline measurement and measurement method, if applicable:

## Intended outcome

Describe the observable user or business outcome. State how success will be measured, who owns the target, and when it will be reviewed. Do not invent a target where the owner has not approved one.

## Scope

### In scope

- 

### Out of scope

- 

### Assumptions and unresolved decisions

| Question or assumption | Owner | Needed by | Resolution / evidence |
| --- | --- | --- | --- |
|  |  |  |  |

## Domain model and rules

- Roles and responsibilities:
- Terms (link to GLOSSARY.md or propose a glossary update):
- Entities, identifiers, and relationships:
- Lifecycle states and allowed transitions:
- Business rules, precedence, and invariants:
- Exceptions, refusal conditions, and escalation:
- Authoritative source documents and applicable requirement IDs:

## User journeys and behavior

For each journey, state actor, preconditions, action, visible result, persisted result, and failure/empty/refusal behavior. Include cross-role handoffs and audit-visible decisions.

1. 

## Functional requirements

Give each requirement a stable ID. Keep requirements atomic and testable. Link every requirement to its evidence/source and acceptance case.

| ID | Requirement (shall) | Rationale/source | Priority | Acceptance case | Verification |
| --- | --- | --- | --- | --- | --- |
| SPEC-001 |  |  | Must / Should / Could | AC-001 |  |

## Quality and operational requirements

Specify only requirements that apply; include measurable thresholds and their owner/source.

- Authorization and separation of duties:
- Sensitive data, privacy, and retention:
- Auditability, evidence integrity, and traceability:
- Availability, performance, and recovery:
- Accessibility, localization, and supported screen sizes:
- Compatibility, API/data contracts, and integrations:
- Observability, support, and operational ownership:

## Data, interfaces, and migration

- Data and event changes, including versioning:
- Source-of-truth and external system boundaries:
- Idempotency, retries, conflict handling, and link/version failure:
- Migration, backfill, rollback, and historical audit behavior:
- Permissions and credentials required by each connector:

## UX and content

- Navigation and entry points:
- Screen states: loading, empty, success, partial, error, denied:
- Keyboard and screen-reader behavior:
- Contrast, responsive behavior, and content terminology:
- Existing functionality regression cases:

## Risk and safeguards

- Tier: Low / Medium / High / Unknown (use highest plausible tier)
- Risk rationale:
- Failure and abuse cases:
- Human review/approval points:
- Fail-closed behavior and recovery:

## Verification and traceability

| Requirement / risk | Test level and scenario | Expected evidence | Owner | Result / run link |
| --- | --- | --- | --- | --- |
|  | Unit / integration / end-to-end / manual |  |  | Pending |

Every Must requirement needs at least one acceptance case. For high-risk behavior, include success and refusal/failure cases. Record review findings, fixes, and rerun results. Mark evidence pending until it has actually been produced.

## Delivery and rollout

- Work slices / child issue links:
- Feature flag or staged rollout:
- Release notes and operator instructions:
- Pre/post-release smoke checks:
- Rollback or recovery:
- Known limitations:

## Decision log

| Date | Decision | Owner | Rationale/evidence |
| --- | --- | --- | --- |
|  |  |  |  |
