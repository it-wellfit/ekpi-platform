# Operations and recovery

- **Status:** Local prototype procedure; production operation **blocked**
- **Owner:** Engineering; operations, security, data and backup owners `UNASSIGNED`
- **Last reviewed:** 2026-10-27
- **Canonical procedures:** [Operations](../operations/README.md), [local backup](../operations/local-backup.md), [production readiness](../operations/production-readiness.md)
- **Matrix:** [phase-a-b-matrix.md](phase-a-b-matrix.md)
- **Test evidence:** [test-evidence.md](test-evidence.md)

This page states how the current local service is started, observed, stopped and recovered, and what is missing before it can be operated as a production service. It does not authorize a deployment.

## Operating boundary

| Item | Current state |
| --- | --- |
| Served surfaces | `ekp_platform` CLI and `governance_web` on `127.0.0.1:3000` |
| Deployment | Local process from a Git checkout; no container, service manager, package registry or cloud target |
| Data | Synthetic or authorized non-sensitive fixtures only; no real customer/personal/regulated data |
| Identity | Local process-memory session, one startup-configured account; no IdP/MFA/recovery/tenant |
| Persistence | Local JSON/JSONL files; governance ledger outside Git; no database |
| Telemetry | Local JSON access log, aggregate `/metrics`, request correlation id, liveness `/healthz` |
| Backup | Local Git bundle + fresh-clone restore drill; local-simulated `age` drill; no off-host copy |
| Support | No on-call, escalation, SLO or named operations owner |

**Do not expose the web server beyond loopback.** It has no TLS, WAF, enterprise identity, tenant boundary or production hardening.

## Start and stop

### Prerequisites

- Windows 11 managed host with PowerShell 5.1.
- Rust `1.99.1` from `rust-toolchain.toml` (or the approved CI toolchain).
- Cargo registry access for the first build; dependencies are locked.
- Optional environment variables:
  - `EKP_GOVERNANCE_DATA` — ledger path.
  - `EKPI_LOCAL_AUTH_USER`, `EKPI_LOCAL_AUTH_PASSWORD`, `EKPI_LOCAL_AUTH_ROLE` — local account; no defaults; without the first two the data pages return `401`.
  - `EKPI_COOKIE_SECURE=1` — add `Secure` to the session cookie when TLS is present (not used on plain loopback).
- No real data, no production credentials and no secrets in the repository.

### Local commands

```powershell
# CLI self-test and schema
cd platform
cargo run --release --locked --bin ekp_platform -- --selftest
cargo run --release --locked --bin ekp_platform -- --declared-rules-schema

# Local workbench (set credentials in the process environment, never in a file)
$env:EKPI_LOCAL_AUTH_USER = "<local-user>"
$env:EKPI_LOCAL_AUTH_PASSWORD = "<local-password>"
$env:EKPI_LOCAL_AUTH_ROLE = "editor"
cargo run --release --locked --bin governance_web
```

Open `http://127.0.0.1:3000/login`. Do not bind to `0.0.0.0`. Stop with `Ctrl+C`; the server performs a graceful shutdown. Sessions disappear on restart.

### Health and readiness

- `/healthz` is a **liveness** endpoint; it returns a static healthy response and does not prove dependencies, data integrity or readiness.
- `/metrics` exposes aggregate counters; it is not a full telemetry pipeline.
- A request id is a correlation aid, not an authenticated identity.
- There is no readiness probe, SLO dashboard, alert rule, trace or on-call integration.

## Monitoring and logging

| Signal | Current | Limitation |
| --- | --- | --- |
| HTTP access | Local JSON log line per request | No central collection, rotation policy, retention or alerting |
| Metrics | Aggregate Prometheus counters at `/metrics` | No latency histogram, saturation, dependency or business SLI |
| Correlation | Request id echoed when safe | Not a user identity; no distributed trace |
| Audit | Local JSONL hash chain and `/audit` view | Not WORM; web actor is the local session username, not an enterprise identity; CLI search audit actor is fixed `search-cli`; no external anchor |
| Health | `/healthz` liveness | Not readiness; no synthetic probe or alert |
| Backup | Manual script + log | No scheduled job, missed-run alert or off-host verification |

**Target (B2):** approved SLIs/SLOs, error budget, OpenTelemetry traces/metrics/logs, dashboards, alert routing, on-call, readiness probe, and fault-injection/recovery drills. None exist today.

## Incident handling

### Current local procedure

1. **Detect and record.** Capture the time, command, request id, affected data path, revision and exact error. Do not delete logs or rewrite the ledger.
2. **Contain.** Stop the process. If a secret may have leaked, rotate it through the approved mechanism; do not commit a replacement secret.
3. **Preserve evidence.** Copy the ledger and relevant logs to a separate path; record SHA-256. Do not modify the original chain.
4. **Assess.** Determine whether the issue is code, data, host policy, credential, backup or external dependency.
5. **Recover.** Use the last verified bundle/ledger copy; follow the restore procedure below. If the final revision cannot be checked, record `BLOCKED`.
6. **Notify.** There is no approved notification path or on-call owner. Record the gap; do not invent an escalation contact.
7. **Review.** Add a postmortem/retro entry, a regression test and a traceability row. Do not lower a gate to close the incident.

### Severity and escalation (proposed, not approved)

| Severity | Example | Proposed response | Owner |
| --- | --- | --- | --- |
| S1 | Data loss/corruption, secret exposure, unauthorized access | Stop service; preserve evidence; security/operations escalation | `UNASSIGNED` |
| S2 | Cannot restore, repeated auth failure, major workflow blocked | Same-day investigation and workaround | `UNASSIGNED` |
| S3 | Single feature failure, degraded performance | Normal backlog with regression test | `UNASSIGNED` |
| S4 | Documentation/UX defect | Next scheduled change | `UNASSIGNED` |

The severity model is a proposal. Without named owners and a notification path, it is not an incident-response capability.

## Backup and restore

### What is backed up

| Asset | Included in Git bundle? | Procedure |
| --- | --- | --- |
| Committed source/docs/tests/config | Yes | `scripts/backup.ps1` creates a `git bundle --all` |
| Governance ledger | **No** | Copy separately from `EKP_GOVERNANCE_DATA` or `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl`; verify the hash chain |
| Uncommitted/untracked work | **No** | Commit before backup; a bundle cannot recover it |
| Secrets/private keys | **No** | Keep in an approved secret manager; never in a bundle |
| Generated reports/exports | Optional | Reproducible; not authoritative |
| Real customer/personal/regulated data | **No** in the prototype | Requires an approved policy and environment first |

### Create and verify a local bundle

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backup.ps1
```

The script creates `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-<timestamp>.bundle`, prints its SHA-256, clones it to a temporary directory and runs the restored `scripts/check.ps1`. A backup is verified only if the restored check exits 0. A bundle that exists but was not restored is not a verified backup.

If host Application Control blocks the automatic `%TEMP%` drill (`os error 4551`):

1. Record the automatic run as `BLOCKED`.
2. Clone the same bundle to a user-profile path.
3. Run the restored `scripts/check.ps1` there.
4. Record the manual result separately. Do not merge `BLOCKED` and `PASS` into one unqualified pass.
5. If the manual run is also blocked, the restore stays `BLOCKED`.

### Encrypted local-simulated drill

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backup-offhost.ps1 `
  -Destination "$env:TEMP\ekpi-offhost-drill" `
  -Recipient <age-recipient> `
  -IdentityFile "$env:TEMP\ekpi-age-drill-key.txt" `
  -Drill
```

This produces `destination_class: local-simulated-off-host`. It is **not** an off-host backup and does not prove key custody, retention or RPO/RTO.

### Restore procedure

1. Retrieve the bundle from the recorded location and verify its SHA-256 against the [backup log](../operations/backup-log.md).
2. `git bundle verify <bundle>`.
3. `git clone <bundle> <clean-path>`.
4. `git rev-parse HEAD` must equal the recorded revision.
5. Run `scripts/check.ps1` (or `.sh`) in the clone. For report/export/data changes, also run `gate.ps1 -IncludeSmoke`.
6. Restore the ledger separately; verify its hash chain on `/audit`.
7. Record date, bundle path, revision, SHA-256, environment, operator, result and limitations.
8. If the check fails for a code/data reason, stop the release and fix it. Do not lower the drill.

**Current status:** last verified restore `6ed236d`; the final evidence bundle `ekpi-platform-2026-10-27-141734.bundle` verifies, clones to `6ed236d` and passes the automatic restore drill with the restored `scripts/check.ps1` exit 0. The earlier `41beaea` bundle also passed; the `28b68cc` restore remains `BLOCKED` and is not counted. The clean-clone canonical PASS does not make the working-tree run pass; that path is still blocked by host policy.

## Disaster recovery

### Current capability

| Layer | Evidence | Status |
| --- | --- | --- |
| Local Git bundle | Bundle + `git bundle verify` | `6ed236d` bundle verifies and clones |
| Local fresh-clone check | `6ed236d` automatic clone + restored `scripts/check.ps1` exit 0 | `PASS` (local) |
| Encrypted artifact | `56a944f` `age` drill, byte-identical decrypt | `SIMULATED` |
| Off-host copy | None | `BLOCKED` |
| Approved RPO/RTO | Proposal only | `BLOCKED` |
| Key custody/recovery | Proposal only | `BLOCKED` |
| Scheduled restore programme | Manual/ad hoc | `BLOCKED` |
| Ledger restore test | Not executed | `NOT RUN` |
| Full DR exercise | Not executed | `NOT RUN` |

### Proposed recovery order

1. **Safety and authority.** Confirm the incident, authorize recovery, and name the recovery lead. `UNASSIGNED`.
2. **Acquire the artifact.** Retrieve the latest verified bundle and ledger copy from the approved location; verify SHA-256 and `git bundle verify`.
3. **Restore source.** Clone to a clean host/path; check the revision.
4. **Restore ledger.** Copy the ledger, verify the hash chain and compare the recorded manifest.
5. **Build and check.** Run the canonical check; if the host policy blocks it, use an approved clean runner and record the environment.
6. **Validate.** Run the pilot workflow or smoke checks; compare key outputs and citations.
7. **Resume.** Start the service only after the check and data validation pass. Record the RTO and any data loss.
8. **Review.** Postmortem, regression test, backup-log update and traceability row.

### Proposed RPO/RTO (not approved)

| Scenario | Proposed RPO | Proposed RTO | Status |
| --- | --- | --- | --- |
| Committed source/docs | 24 hours | 8 hours | Proposal only |
| Governance ledger | 4 hours | 8 hours | Proposal only |
| Full pilot environment after host loss | 24 hours | 1 business day | Proposal only; requires off-host target |

Do not present these as agreed SLAs. The product and operations owners must approve different values if the business need differs.

## Ownership and support

| Role | Responsibility | Current |
| --- | --- | --- |
| Service owner | Scope, availability, change approval | `UNASSIGNED` |
| Operations owner | Start/stop, monitoring, incident command, handoff | `UNASSIGNED` |
| Security owner | Threat model, access review, vulnerability disposition | `UNASSIGNED` |
| Data owner | Classification, retention/deletion, residency, AI boundary | `UNASSIGNED` |
| Backup owner | Schedule, off-host copy, restore test | `UNASSIGNED` |
| Key custodian | Encryption key custody/recovery | `UNASSIGNED` |
| Support/escalation | User support, severity routing | `UNASSIGNED` |
| Independent reviewer | Review and sign evidence | `UNASSIGNED` |

No named person is recorded because no one has accepted these roles. A placeholder name would be a fabricated approval.

## Production readiness decision

**Not ready.** The following are missing and cannot be replaced by documentation:

- approved deployment target, network/TLS boundary and change approval;
- enterprise IdP/MFA, tenant isolation and access review;
- off-host encrypted backup, key custody, approved RPO/RTO and a tested restore;
- SLO/alerting/traces, on-call/escalation and an incident drill;
- named service/operations/security/data/backup owners;
- independent security review and a data policy.

Until these exist and are evidenced, the correct status is **Phase B blocked**. See [production readiness](../operations/production-readiness.md) and [external blockers](external-blockers.md).

## Update triggers

Update this page when the deployment boundary, start/stop command, data path, auth mode, telemetry, backup target, key custody, RPO/RTO, incident process or owner changes. Record the revision, date and evidence; do not change a status without an execution.
