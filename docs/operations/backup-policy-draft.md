# Off-host backup and RPO/RTO proposal

- **Status:** Draft proposal; **not approved**, **not implemented**
- **Owner:** Operations + Security (proposed; acceptance required)
- **Last reviewed:** 2026-10-26
- **Phase gate:** [B7 backup and disaster recovery](../quality/phase-gates.md#phase-b-mvp--production-grade-gate)
- **Readiness row:** [PR-07](production-readiness.md)
- **Current local procedure:** [local-backup.md](local-backup.md), [backup log](backup-log.md)
- **Decision register:** [governance/08](../governance/08-decision-register.md) (`DEC-14`)

## Purpose and limit

This is a proposal for an off-host backup programme. It is not an approved policy, an implemented control or evidence that an off-host copy exists. The repository currently has only local Git bundles and a local restore drill. Until the decisions and owners below are recorded, B7 and PR-07 remain `Partial`/`Blocked`.

## Proposed scope

| Asset | Included | Notes |
| --- | --- | --- |
| Git repository history | Yes | `git bundle --all`; includes committed code, docs, tests and configuration |
| Governance ledger | Yes, separately | JSONL append-only chain outside Git; copy with integrity verification |
| Runtime configuration | Yes, redacted | Non-secret parameters and environment names; never secrets |
| Secrets and private keys | **No** | Keep in an approved secret manager; never in a bundle or backup artifact |
| Generated reports/exports | Optional | Reproducible from the CLI; do not treat as authoritative |
| Uncommitted work | **No** | Commit before backup; a bundle cannot recover it |
| Real customer/personal/regulated data | **No** in the prototype | Requires an approved data policy and environment first |

## Proposed RPO/RTO (requires written approval)

These values are proposals for a limited local pilot, not an approved service level.

| Scenario | Proposed RPO | Proposed RTO | Rationale / constraint |
| --- | --- | --- | --- |
| Committed source and docs | 24 hours | 8 hours | Daily bundle; restore is a clone plus the pinned toolchain |
| Governance ledger | 4 hours | 8 hours | Append-only file; copy frequency depends on pilot activity |
| Full pilot environment after host loss | 24 hours | 1 business day | Requires an approved off-host target and key access |

If the business needs tighter targets, the product owner and operations owner must approve different values and the cost/operability impact. Do not describe these proposals as agreed SLAs.

## Proposed storage and encryption

1. Store encrypted backup artifacts in an organization-approved off-host location, separate from the source host and its credentials.
2. Use envelope encryption with a managed key or an approved recovery key. The backup operator must not hold the only copy of the key.
3. Enable versioning or object lock/immutability where the target supports it; protect against deletion by a compromised operator account.
4. Record a SHA-256 manifest for each artifact and verify it after transfer.
5. Keep at least one copy in a separate failure domain from the primary host. A second folder on the same disk is not off-host.
6. Define retention and deletion, including legal hold and data-residency constraints, before storing any non-synthetic data.

Tooling, target, key management and retention require a security/data decision. They are not selected here.

## Proposed schedule

| Activity | Proposed cadence | Evidence |
| --- | --- | --- |
| Git bundle | Daily before/after a pilot session; before every release/tag | Bundle path, SHA-256, `git bundle verify` |
| Ledger copy | Every 4 hours during an active pilot; after each session | Ledger path, SHA-256, hash-chain check |
| Off-host transfer | After each bundle/ledger copy | Transfer log, manifest verification |
| Restore drill | Weekly during a pilot; monthly when idle; before release | Fresh clone, restored `scripts/check`, result row |
| Full disaster-recovery exercise | At least quarterly, or after a material change | Scenario, participants, RTO measured, gaps |

A missed run is an incident/exception with an owner; it is not silently skipped.

## Restore and verification

A backup is verified only when:

1. the artifact can be retrieved from the off-host location using the documented key path;
2. `git bundle verify` succeeds and the clone reaches the recorded revision;
3. the restored clone passes `scripts/check.ps1` (or `.sh`) and, for report/export changes, `gate.ps1 -IncludeSmoke`;
4. the ledger copy is present, its hash chain verifies, and the restored data matches the recorded manifest;
5. the result, duration, operator, limitations and any data loss are recorded in the [backup log](backup-log.md).

A local bundle that exists but has not been restored is not a verified backup. A successful restore of a source-only bundle does not prove ledger recovery or off-host availability.

## Required decisions and owners

| Item | Required decision / owner | Status |
| --- | --- | --- |
| Off-host target and failure domain | Operations + Security | Not assigned |
| Encryption/key management and recovery | Security | Not assigned |
| RPO/RTO approval | Product + Operations | Proposed only |
| Retention, legal hold and residency | Data owner + Legal | `DEC-13`, `DEC-14`; not assigned |
| Backup owner and restore tester | Operations | Not assigned |
| Incident path for lost/corrupted backup | Operations + Security | Not assigned |

## Acceptance criteria

- [ ] Named backup owner and key custodian accepted in writing.
- [ ] Approved off-host target and failure domain.
- [ ] Encryption at rest and key-recovery procedure tested.
- [ ] Approved RPO/RTO and retention.
- [ ] At least one off-host artifact retrieved and restored successfully.
- [ ] Ledger restore and hash-chain verification tested.
- [ ] RTO measured and recorded.
- [ ] Scheduled restore tests and missed-run escalation defined.
- [ ] Evidence linked from PR-07, B7 and the backup log.

Until every box is checked with linked evidence, the correct status is **not approved** and Phase B remains blocked.

## Sign-off (not yet completed)

| Decision | Name / role | Date | Residual risk |
| --- | --- | --- | --- |
| Backup policy approved | — | — | — |
| RPO/RTO approved | — | — | — |
| Off-host target approved | — | — | — |
| First off-host restore accepted | — | — | — |

## Update triggers

Update this proposal when the target, key management, RPO/RTO, retention, data classification, owner or restore procedure changes. Replace proposals with dated approvals and executed evidence; do not mark a proposal as implemented.
