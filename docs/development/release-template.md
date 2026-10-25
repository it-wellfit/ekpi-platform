# Release notes and operations handoff template

> Complete this for a release or pilot handoff. Replace unknowns with `Pending` plus an owner and due date; never imply that a deployment, approval, training, or control exists without evidence.

## Release record

- Product/component and release version:
- Release date/time and time zone:
- Source revision / commit:
- Toolchain and build environment:
- Release owner and approver:
- Related issues, specs, and requirements:
- Phase gate being claimed (Phase A / Phase B / none) and status:
- Traceability register rows updated (IDs):
- Contract IDs affected:
- Data classification and security impact:

## Version notes

### Added

- 

### Changed

- 

### Fixed

- 

### Removed or deprecated

- 

### Data, configuration, compatibility, and security impact

- Schema/configuration changes:
- Required migration or backfill:
- API, file, or integration compatibility:
- Dependency/security findings and disposition:

### Known limits and operator-visible behavior

- 

## Verification evidence

| Gate / smoke case | Environment and revision | Command or steps | Result | Evidence link | Defect / follow-up |
| --- | --- | --- | --- | --- | --- |
| Documentation links and required files |  | `scripts/check-docs` | Pending |  |  |
| High-signal secret pattern scan |  | `scripts/check-secrets` | Pending |  |  |
| Formatting / Clippy / tests |  |  | Pending |  |  |
| eval self-test and report |  |  | Pending |  |  |
| platform self-test and schema |  |  | Pending |  |  |
| Report / export / coverage / HTML (if affected) |  |  | Not run |  |  |
| Governance web local smoke (if affected) |  |  | Not run |  |  |
| Security / dependency scans |  |  | Pending |  |  |
| Backup bundle + restore drill |  | `scripts/backup` | Pending |  |  |
| Phase A/B gate evidence |  | Link this template | Pending |  |  |

Do not mark an unrun or blocked check as passed. Link the PR checks and attach manual evidence with date, operator, and test-data identifier.

## Release and recovery

- Artifact names, checksums, and storage location:
- Release channel / deployment target, or `Not configured` with reason:
- Pre-release backup bundle path and creation time:
- Restore drill environment, command, result, and backup-log row:
- Rollback / recovery procedure and owner:
- Recovery verification and data-loss boundary:
- Post-release smoke result and time:
- Monitoring/alerts, owner, and response path, or `Not configured` with reason:

## Operations handoff

- Service and environment owner:
- Support contact and escalation path:
- Start / stop / health-check instructions:
- Data store locations, classification, retention, and backup owner:
- Identity, role, and access-review owner (never include secret values):
- Logs, metrics, audit review, and incident procedure:
- Scheduled maintenance and dependency-update responsibility:
- Unresolved operational risks, issue links, owners, and due dates:

## Training and adoption

- Audience and required roles:
- Materials / runbook links:
- Training owner, date, and delivery method:
- Attendance or completion evidence:
- User feedback, open questions, and follow-up owner:

## Approval

| Responsibility | Name / role | Decision | Date | Evidence |
| --- | --- | --- | --- | --- |
| Product / process owner |  | Pending |  |  |
| IT / security / data owner (as applicable) |  | Pending |  |  |
| Operations owner |  | Pending |  |  |
| Phase A/B gate owner (if a gate is claimed) |  | Pending |  |  |
