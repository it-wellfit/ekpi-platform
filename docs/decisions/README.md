# Decisions

- **Status:** Active
- **Owner:** Engineering (business decisions remain with the owners in `governance/08`)
- **Last reviewed:** 2026-10-26

This directory indexes engineering decisions and provides the ADR template. Decision **records** remain in [docs/adr/](../adr/); this page does not copy them.

## What is a decision record

Use an ADR when a choice is hard to reverse, constrains several components, changes a public contract, or would be expensive to rediscover. Examples: delivered vs library-only module boundary, data-store or identity approach, evaluation policy, backup/restore design, deployment target, or a compatibility break.

Do **not** use an ADR for:

- a business rule, control or acceptance criterion — those live in [governance/08](../governance/08-decision-register.md) and the functional/data specs;
- a routine implementation choice that is easy to reverse and contained in one module;
- a task status update or a meeting note.

## Index

| ADR | Title | Status | Date | Area |
| --- | --- | --- | --- | --- |
| [0001](../adr/0001-engineering-delivery-workflow.md) | GitHub Issues and reproducible Rust delivery checks | Accepted | 2026-10-25 | Delivery / CI |

New ADRs use the next four-digit number and the template below. Do not renumber existing records.

## Numbering and lifecycle

- File name: `docs/adr/NNNN-short-kebab-title.md`.
- Status: `Proposed`, `Accepted`, `Rejected`, `Superseded by ADR-NNNN`, or `Deprecated`.
- An accepted ADR is not edited to change its decision. Add a new ADR that supersedes it and update the old record's status and link.
- Correcting a typo or adding a link is allowed; changing context, decision or consequences is not.
- Link the ADR from the affected spec/contract, the traceability register and the PR.

## Relationship to other decision sources

| Source | Owns | Does not own |
| --- | --- | --- |
| [governance/08 decision register](../governance/08-decision-register.md) | Business/control decisions, owners, open business parameters, go/no-go | Engineering implementation choice |
| [docs/adr/](../adr/) | Engineering choices with lasting consequences | Business approval or legal interpretation |
| [tech-decision-records](../tech-decision-records/README.md) | Index of referenced technology decisions | A second ADR process |
| Issue/PR discussion | Working rationale and review | A durable decision of record |

If a business decision and an engineering decision interact, record both and link them. Do not bury a business approval inside an ADR.

## Template

Copy [adr-template.md](adr-template.md). Keep it short: context, decision, consequences, alternatives and evidence. A decision without consequences or rejected alternatives is usually a task description, not an ADR.

## Review triggers

Review this index when an ADR is added or superseded. Review the decision itself when its assumptions change, a linked decision changes, or a release/incident invalidates it.
