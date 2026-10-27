# External blockers and unlock actions

- **Status:** Active; all listed blockers are unresolved
- **Owner:** Engineering/QA for the record; each unlock owner is `UNASSIGNED`
- **Last reviewed:** 2026-10-27
- **Matrix:** [phase-a-b-matrix.md](phase-a-b-matrix.md)
- **Rule:** Do not ask for passwords, private keys, tokens or real customer data in this repository. Grant access through the approved organizational mechanism.

These blockers cannot be resolved by changing source code alone. Each entry names the missing resource, why code cannot fix it, the accountable role, the minimum access needed, the unlock steps, the acceptance criteria and the work that can continue now. A blocker is not a reason to stop unrelated engineering work.

## B-01 Pilot owner and representative user

| Field | Detail |
| --- | --- |
| Phase items | A1, A2, A3, A6, A7, A10 |
| Missing resource | A named business pilot owner and at least one representative user; an approved non-sensitive workflow and dataset; acceptance authority. |
| Why code cannot solve it | User value and fitness for use require a real person to execute and accept the workflow. Synthetic tests can prove conformance only. |
| Expected role | Business pilot owner + representative user + product owner. |
| What they must provide | Workflow scope, user role, authorized non-sensitive data, expected result, acceptance criteria, availability for a recorded session, and a signed acceptance/decline. |
| Minimum access | Read-only access to the local workbench/CLI and the pilot dataset; no production credentials; no real customer data. |
| Unlock steps | 1. Select one workflow and record it in the [pilot runbook](../operations/pilot-runbook.md). 2. Confirm data classification and authorization. 3. Run the workflow and record inputs/outputs/errors. 4. Ask the user to accept or reject. 5. Add the run ID, commit, evidence path and signature to the [traceability register](../traceability/register.md). |
| Acceptance criteria | At least one end-to-end workflow accepted by the representative user; no unresolved critical defect; evidence contains no secret or personal data. |
| Work that can continue now | Synthetic workflow automation, runbook/template refinement, defect fixes found by synthetic tests. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; [phase gates](../quality/phase-gates.md) A1 `Not met`. |

## B-02 Remote, CI and branch protection

| Field | Detail |
| --- | --- |
| Phase items | A4, A7, A11, B4 |
| Missing resource | An authorized Git remote, a CI runner that can execute `.github/workflows/ci.yml`, branch protection/rulesets, required status checks and a reviewer. |
| Why code cannot solve it | CI and branch protection are hosted-repository controls. A local workflow file is configuration, not an execution. |
| Expected role | Repository administrator + engineering owner + security owner. |
| What they must provide | Remote URL and push authorization, CI enablement, required-check names, branch protection for `main`, PR review requirement, action pinning policy. |
| Minimum access | Write access to a dedicated branch/PR; permission to read CI logs and branch-protection settings; no production deployment rights. |
| Unlock steps | 1. Create/authorize the remote. 2. Push the current commit and open a PR. 3. Run CI and retain the run URL/logs. 4. Configure branch protection: PR review, required checks, no force-push/deletion, no bypass. 5. Pin actions by commit SHA. 6. Record the run and settings in [release evidence](../operations/release-evidence-bundle.md). |
| Acceptance criteria | CI green on the named commit; a failing required check blocks merge; branch protection settings visible; no direct push to `main`; reviewer approval recorded. |
| Work that can continue now | Harden the workflow, pin actions, add local failure messages, keep `scripts/check` current. |
| Status / evidence | `BLOCKED`; no remote (`git remote -v` empty), `gh` not authenticated, CI never ran. |

## B-03 Enterprise identity, MFA and tenant model

| Field | Detail |
| --- | --- |
| Phase items | A5, B1, B5, B8 |
| Missing resource | An approved IdP (OIDC or SAML), MFA policy, account lifecycle/access review, tenant model and tenant-scoped data boundary. |
| Why code cannot solve it | The current login is a single startup-configured local account with process-memory sessions. Enterprise identity requires a trusted external authority, federation metadata, claim/role mapping and a data model decision. |
| Expected role | IAM owner + security approver + data/product owner. |
| What they must provide | IdP issuer/client registration, redirect URIs, MFA requirement, claim mapping, role catalog, tenant definition, test users/tenants, access-review process, session/revocation policy. |
| Minimum access | A non-production test tenant and test users; no production user data; no long-lived client secret in the repository. |
| Unlock steps | 1. Approve the identity/tenant decision (`DEC-12`). 2. Implement the OIDC/SAML adapter behind an interface and keep local auth explicitly non-production. 3. Map claims to Viewer/Editor/Approver and tenant. 4. Add positive/negative tests for missing/expired identity, role escalation, cross-tenant read/write/search/export and revocation. 5. Obtain security review. |
| Acceptance criteria | Unauthenticated and unauthorized requests denied by default; MFA enforced per policy; zero cross-tenant leakage in tests; session revocation and access review evidenced; no fallback to local auth when enterprise mode is configured. |
| Work that can continue now | Interface design, denial tests against the local session, threat-model updates, no-default-account checks. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; A5 `Partial`; [threat model](../architecture/threat-model.md). |

## B-04 Data owner and data policy

| Field | Detail |
| --- | --- |
| Phase items | A3, A5, B1, B8 |
| Missing resource | A named data owner; data inventory/classification; lawful basis; retention/deletion; legal hold; residency; access and AI data boundaries. |
| Why code cannot solve it | These are legal, privacy and business decisions. A code change cannot create a lawful basis or approve retention. |
| Expected role | Data owner + Legal/Privacy + security approver. |
| What they must provide | Approved classification scheme, permitted data categories, retention/deletion schedule, residency constraints, legal-hold procedure, access-review cadence and AI data boundary. |
| Minimum access | Policy documents and a synthetic/approved non-sensitive test dataset; no real personal data. |
| Unlock steps | 1. Inventory data and flows. 2. Classify each dataset. 3. Approve purpose/lawful basis/retention/deletion. 4. Implement deletion/export and access review. 5. Test deletion and legal hold. 6. Record approval in [production readiness](../operations/production-readiness.md) PR-05. |
| Acceptance criteria | Every data class has an owner, purpose, location, retention and deletion method; deletion test passes; access review recorded; AI boundary approved before any external model call. |
| Work that can continue now | Data-flow inventory, synthetic-data policy, redaction checks, docs. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; [production readiness](../operations/production-readiness.md) PR-05; `DEC-13`. |

## B-05 AI provider and data policy

| Field | Detail |
| --- | --- |
| Phase items | A9, B5 |
| Missing resource | An approved model/provider, data-processing terms, permitted data classes, retention/training opt-out, tool integration policy, cost/latency budget and human-takeover process. |
| Why code cannot solve it | Selecting a provider and sending data to it are external contractual and data-governance decisions. The repository currently has only a mock provider. |
| Expected role | AI system owner + data owner + security/legal approver. |
| What they must provide | Approved provider/region, model/version, data boundary, retention/no-training terms, API credential via secret manager, budget, evaluation acceptance criteria, human escalation path. |
| Minimum access | A sandbox project with synthetic data and a scoped API key stored outside Git; no production data. |
| Unlock steps | 1. Approve provider and data policy (`DEC-15`). 2. Implement a versioned adapter behind an interface. 3. Add fixed eval cases for normal, ambiguous, adversarial, wrong-source, tool failure and insufficient context. 4. Record model/prompt/index/corpus versions and cost/latency. 5. Run human review and regression. 6. Keep mock mode as a test fixture, not a quality claim. |
| Acceptance criteria | Fixed eval passes approved thresholds; tool-failure and injection cases behave safely; no restricted data leaves the approved boundary; cost/latency within budget; human takeover tested. |
| Work that can continue now | Extend deterministic eval fixtures, tool-failure harness design, data-boundary tests, mock contract. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; [AI evaluation](../quality/ai-evaluation.md); `EVAL-GAP-01`/`02`/`03`. |

## B-06 Off-host backup, key custody and RPO/RTO

| Field | Detail |
| --- | --- |
| Phase items | A8, B7 |
| Missing resource | An approved off-host storage target and failure domain, encryption/key-custody process, retention/immutability, approved RPO/RTO and a restore owner. |
| Why code cannot solve it | A local script cannot create an off-host copy or approve a recovery objective. The `age` drill is explicitly `local-simulated-off-host`. |
| Expected role | Operations owner + security/key custodian + data owner + product owner for RPO/RTO. |
| What they must provide | Storage endpoint/bucket and region, access policy, key management/recovery procedure (operator must not hold the only key), retention/object-lock policy, approved RPO/RTO, restore test window and named tester. |
| Minimum access | Write/read to a dedicated non-production backup location; no production data; no broad cloud-admin role. |
| Unlock steps | 1. Approve target and RPO/RTO ([proposal](../operations/backup-policy-draft.md)). 2. Configure encryption and key custody. 3. Transfer a bundle and ledger copy; record SHA-256 manifest. 4. Retrieve from off-host and restore into an isolated environment. 5. Verify `git bundle verify`, clone, restored check, ledger hash chain and measured RTO. 6. Schedule recurring drills and missed-run escalation. |
| Acceptance criteria | At least one off-host artifact retrieved and restored; RPO/RTO met; key recovery tested without the operator holding the only key; result logged in [backup log](../operations/backup-log.md). |
| Work that can continue now | Local bundle/restore, ledger copy procedure, local `age` drill, manifest and failure-message hardening. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; B7 `Partial`; local A8 `Met` at `41beaea` (E-020); off-host evidence E-009 is `SIMULATED` only. |

## B-07 Security and operations owners

| Field | Detail |
| --- | --- |
| Phase items | A5, A6, A11, B1, B2, B6 |
| Missing resource | Named security owner/approver, operations owner, incident commander, service owner and support/escalation path. |
| Why code cannot solve it | Accountability and on-call authority are organizational roles. A document can propose them but cannot assign them. |
| Expected role | Executive/engineering leadership to assign; the named owners to accept. |
| What they must provide | Written role acceptance, decision rights, escalation contacts, support hours, incident severity model, maintenance ownership and residual-risk acceptance. |
| Minimum access | Access to the repository, runbooks and incident channel; no production credentials required to accept the role. |
| Unlock steps | 1. Assign roles in [production readiness](../operations/production-readiness.md). 2. Review the [threat model](security-threat-model.md). 3. Exercise an incident and restore drill. 4. Sign the handoff. 5. Record the date and residual risk. |
| Acceptance criteria | Every critical service/control has a named owner; escalation path exercised; handoff signed; residual risk accepted. |
| Work that can continue now | Runbooks, ownership matrix, incident templates, drill design. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; [production readiness](../operations/production-readiness.md) PR-14/PR-15. |

## B-08 Independent reviewer and second annotator

| Field | Detail |
| --- | --- |
| Phase items | A11, B1, B5 |
| Missing resource | A reviewer who did not implement the change; a second independent annotator for the κ study. |
| Why code cannot solve it | Independence is a role-separation property. An AI self-review or the same implementer re-running a check is not independent. |
| Expected role | Independent reviewer (engineering/security) + second annotator (domain/QA). |
| What they must provide | Time to rerun commands, inspect the diff/threat model/evidence, record findings and sign; for κ, independent labels on the fixed corpus. |
| Minimum access | Read-only repository access and a clean environment; no production data; no authoring access during review. |
| Unlock steps | 1. Freeze the review revision. 2. Provide the [reviewer checklist](phase-a-b-matrix.md#a11-reviewer-checklist-prepared-not-signed). 3. Reviewer reruns canonical/release/docs/secrets checks and records the host block if present. 4. Reviewer inspects the matrix and non-claims. 5. Record findings and disposition. 6. For κ, run the two-annotator study and report the denominator. |
| Acceptance criteria | Reviewer signs; no unresolved critical/high finding; every exception has an owner/expiry; κ computed with a non-zero denominator or reported as undefined. |
| Work that can continue now | Evidence packaging, reproducibility, checklist refinement, annotation tooling. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; A11 `Partial`; `EVAL-GAP-03`. |

## B-09 Approved workload, SLO and capacity model

| Field | Detail |
| --- | --- |
| Phase items | B2, B3 |
| Missing resource | User/concurrency profile, request mix and sizes, growth assumptions, latency/availability/error SLOs, cost budget and peak factors. |
| Why code cannot solve it | These are product/operations decisions. A benchmark without an approved workload cannot prove fitness. |
| Expected role | Product owner + operations owner + engineering. |
| What they must provide | Approved workload sheet, SLO/SLI definitions, error-budget policy, peak/scale assumptions, budget and acceptance thresholds. |
| Minimum access | Non-production load-test environment and synthetic data; no production traffic. |
| Unlock steps | 1. Approve workload/SLO (`DEC-16`, `DEC-19`). 2. Add load/stress/soak scripts. 3. Run against the approved environment. 4. Analyze bottlenecks and capacity headroom. 5. Compare with thresholds and record the result. |
| Acceptance criteria | Reproducible load/soak results; SLO met for the approved window; capacity/cost model reviewed; no unbounded degradation. |
| Work that can continue now | Benchmark harness, local exploratory measurements, dashboard design. |
| Status / evidence | `BLOCKED` for formal acceptance; B2/B3 `Partial`; E-012. |

## B-10 Production deployment target and change approval

| Field | Detail |
| --- | --- |
| Phase items | B1, B4, B6, B7 |
| Missing resource | An approved deployment platform/environment, network/TLS boundary, storage/database, secrets injection, change approval, rollback and post-release verification. |
| Why code cannot solve it | Deploying to production requires organizational authorization, infrastructure and a change process. It must not be improvised. |
| Expected role | Platform/operations owner + security + change approver. |
| What they must provide | Environment definition, credentials via secret manager, network policy, TLS, capacity, backup target, rollback plan, maintenance window and approval. |
| Minimum access | A non-production/staging environment first; production access only after approval. |
| Unlock steps | 1. Approve the target and data policy. 2. Build deployment config and health checks. 3. Deploy to staging and run the gate. 4. Exercise rollback and restore. 5. Obtain change approval. 6. Deploy progressively with post-release verification. |
| Acceptance criteria | Staging gate passes; rollback and restore drills pass; production change approved; post-release checks pass. |
| Work that can continue now | Deployment manifests, health checks, rollback/runbook drafts, staging checklist. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; no target or credentials exist. |

## B-11 Host Application Control policy

| Field | Detail |
| --- | --- |
| Phase items | A4, A8 |
| Missing resource | An approved endpoint-security exception or a clean runner where newly built unsigned executables/build scripts can run; alternatively, a signed build path. |
| Why code cannot solve it | The host denies process creation with `os error 4551` before the test or build script runs. Changing source cannot grant execution. Lowering the gate would hide the failure. |
| Expected role | Endpoint/security administrator (policy owner) + engineering. |
| What they must provide | A scoped allow rule for the repository's build output and Rust build scripts, or an approved CI/clean runner; the policy change record and scope. |
| Minimum access | Permission to request/approve the policy exception; no broad disablement of Application Control. |
| Unlock steps | 1. Capture the exact blocked path and policy ID. 2. Request a scoped exception or runner. 3. Re-run `scripts/check.ps1` on `6ae6134` or the target revision. 4. Restore the final bundle in a fresh clone and run the restored check. 5. Record the result with the policy/runner identifier. |
| Acceptance criteria | Canonical check exit 0 on the target revision; final bundle restore check exit 0; no gate lowered or test skipped. |
| Work that can continue now | Release-profile coverage, docs/secrets checks, bundle creation/verification, manual clone inspection. |
| Status / evidence | `PARTIAL` — the `41beaea` clean clone passed the canonical check and the automatic restore drill; the working-tree canonical run is still `BLOCKED` at the 22 CLI contract tests. E-001/E-007/E-019/E-020; policy observed as `{0283ac0f-fff1-49ae-ada1-8a933130cad6}` on this host. |

## B-12 Legal, privacy and compliance scope

| Field | Detail |
| --- | --- |
| Phase items | B1, B8 |
| Missing resource | Applicable legal/regulatory scope, privacy impact assessment, control mapping, non-waivable controls and certification scope (if any). |
| Why code cannot solve it | Legal applicability and certification are external determinations. A source matrix is not a legal opinion. |
| Expected role | Legal/privacy + compliance owner + security. |
| What they must provide | Applicable jurisdictions/regulations, data-protection impact assessment, retention/legal-hold requirements, certification scope and auditor. |
| Minimum access | Policy/legal guidance; no real data required for the initial scoping. |
| Unlock steps | 1. Identify applicable obligations. 2. Map controls and gaps. 3. Complete privacy/security assessment. 4. Approve residual risk. 5. If certification is sought, engage an accredited auditor. |
| Acceptance criteria | Written scope and control mapping; non-waivable controls identified; assessment signed; no unsupported compliance claim. |
| Work that can continue now | Source matrix, data-flow inventory, control evidence index, non-claim review. |
| Status / evidence | `BLOCKED` / `UNASSIGNED`; [production readiness](../operations/production-readiness.md) PR-17. |

## Priority and sequencing

1. **B-11** still limits working-tree reproducibility; the clean-clone canonical and restore path now passes at `41beaea`, so A4/A8 are `Met` on that scope. Obtain an exception or CI runner for the working-tree path.
2. **B-02** unlocks CI/branch-protection evidence and gives a clean canonical runner, reducing dependence on the managed host.
3. **B-01** unlocks A1/A7 and the pilot-scope rows; it is the decisive Phase A gap.
4. **B-08** unlocks independent review and the κ study.
5. **B-03/B-04/B-05** unlock A5/A9 and the Phase B identity/data/AI controls.
6. **B-06/B-09/B-10** unlock B7/B2/B3 and the production-readiness path.
7. **B-07/B-12** provide the accountable ownership and legal scope without which no production claim is valid.

No blocker should be marked resolved without a dated record, the responsible role, the evidence path and an acceptance decision.
