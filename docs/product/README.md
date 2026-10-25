# Product brief

- **Status:** Draft, evidence-limited
- **Owner:** Product
- **Last reviewed:** 2026-10-26
- **Canonical detail:** [requirements](../governance/02-requirements-v0.1.md), [functional spec](../governance/05-functional-spec-v1.md), [data and state model](../governance/06-data-and-state-model-v1.md), [acceptance matrix](../governance/04-acceptance-test-matrix.md), [decision register](../governance/08-decision-register.md), [target traceability](../governance/10-traceability-baseline.md), [metrics](../metrics/targets.md)

This brief frames the product; it does not replace the governance specifications. Where this page and a canonical specification differ, the specification wins.

## Problem

Organizations run quality, audit and compliance work across documents, tools and handoffs. A conclusion can be hard to reproduce because the document version, the active rule, the process evidence and the approval record are not bound into one traceable decision. A plausible answer is not enough: the system must show which source version and clause support the result, and it must refuse or escalate when evidence is insufficient.

The target product is an evidence-grounded knowledge and process intelligence platform. The current repository is a **local deterministic prototype** of the adjudication core plus a local governance workbench; it is not a deployed enterprise service.

## Users and roles

The role baseline is defined in [governance/02](../governance/02-requirements-v0.1.md). The main roles are:

- Sponsors and PMO, who own scope, risk acceptance and cross-project governance.
- Project managers and work owners, who execute a governed lifecycle and keep evidence.
- QA and independent reviewers, who test evidence sufficiency and recommend release.
- Document control, who own version, publication and archive integrity.
- IT, security and data governance, who own identity, access, retention, backup and integration.
- Compliance and legal, who interpret applicable requirements and non-waivable controls.
- The AI system owner, who owns model, prompt, evaluation and human-review policy.

One person may hold several roles. Conflicts must be recorded, and high-risk work requires independent review. The prototype does **not** enforce these role separations because enterprise identity and authorization are not wired.

## Evidence status

- The user need is currently sourced from a draft user brief (`DRAFT-USER-BRIEF`) and the governance design record, not from completed interviews or observed tenant workflows.
- No real organization, tenant inventory, document set, baseline measurement or pilot has been validated in this repository.
- The acceptance matrix describes target-product cases. Prototype tests do not mark those cases passed.
- Numeric targets marked `proposed` in [metrics/targets.md](../metrics/targets.md) still need product-owner sign-off.

Do not describe the product as validated, compliant, certified or production-ready on the basis of this repository alone.

## Value proposition

For a compliance or quality decision, the platform should:

1. Bind the decision to a specific document version and clause anchor.
2. Apply only active, reviewed rules and refuse drafts.
3. Replay process evidence and detect forbidden orderings, deadlines and loops.
4. Abstain or escalate when evidence is missing, stale, conflicting or unregistered.
5. Produce a deterministic snapshot that can be replayed and audited.
6. Keep AI assistance subordinate to named human review; AI never approves, waives or publishes by itself.

The current deterministic core implements parts of items 1–5 for a synthetic corpus. AI assistance is not implemented.

## MVP scope (Phase A)

The Phase A MVP is a locally runnable, testable and documented slice that proves the adjudication loop on synthetic data. It is **not** an enterprise deployment.

In scope:

- Versioned source documents with clause anchors and SHA-256 binding.
- Active-rule layer with schema validation and draft rejection.
- Process replay with forbidden-pair, deadline and loop checks.
- Evidence retrieval with citations and conservative abstention.
- A lexical knowledge-search CLI that returns verbatim cited clauses or an honest refusal, plus a fixed regression set.
- Deterministic decision snapshots and replay checks.
- A CLI that produces evaluation, coverage and LegalRuleML artifacts.
- A local, server-rendered governance workbench for projects, evidence, workflows, quality cases and gate decisions.
- Automated tests, a regression baseline, traceability and a local backup/restore procedure.

Out of scope for Phase A:

- Enterprise identity, MFA, role administration or tenant isolation.
- Microsoft 365, Google Workspace or other live connectors.
- AI-generated workflow drafts or autonomous approvals.
- Production deployment, public network exposure, SLAs or 24×7 operations.
- Regulatory certification or a claim that the synthetic corpus represents a real organization.

The detailed capability/evidence table and exit criteria are in [MVP scope](mvp-scope.md) and [Phase gates](../quality/phase-gates.md).

## Primary journeys

| Journey | Current surface | Canonical behavior | Evidence |
| --- | --- | --- | --- |
| J1: adjudicate a case against versioned sources and active rules | `ekp_platform` CLI / engine tests | FR-01–FR-06, [functional spec](../governance/05-functional-spec-v1.md) | `eval` and `platform` tests; `--selftest`; report artifacts |
| J2: link and review a source reference | local governance web `/evidence` | FR-02/FR-03, [functional spec](../governance/05-functional-spec-v1.md) | `platform/tests/governance_web.rs` |
| J3: record a gate or quality decision | local governance web `/lifecycle`, `/controls` | FR-05/FR-06/FR-11 | `platform/tests/governance_web.rs`; target matrix AT cases remain `NOT RUN` |

These journeys are demonstrated on synthetic data only. Real-user validation is a Phase A gate, not a completed fact.

## Success measures

The binding register is [metrics/targets.md](../metrics/targets.md). It distinguishes:

- `locked` hard gates that must never be relaxed (for example, no actual violation may be judged `comply`);
- `proposed` targets that need product-owner approval and measurement evidence;
- `deferred` measures that are intentionally not collected yet.

Do not copy numeric targets into this brief. Use the register so a target change has one home.

## Open product decisions

The open items are tracked in [governance/08](../governance/08-decision-register.md), including pilot scope, roles, risk classification, data policy, identity, retention, service levels and AI use. They are not silently defaulted in code or documentation. Items that block a production claim include `DEC-01`, `DEC-09`, `DEC-12`, `DEC-13`, `DEC-14`, `DEC-15`, `DEC-16`, `DEC-19` and `DEC-20`.

## Phase status

- **Phase A (0 → 1):** local prototype exists and is tested; the MVP gate is **not yet approved** because real-user validation, restore-drill evidence and complete traceability are outstanding.
- **Phase B (MVP → production-grade):** **blocked** by the absence of an approved deployment target, enterprise identity, data policy and operating owner. See [production readiness](../operations/production-readiness.md).

## Update triggers

Update this brief when the problem statement, target users, MVP scope, non-goals, phase status or canonical governance source changes. Update the linked specifications first; then update this summary.
