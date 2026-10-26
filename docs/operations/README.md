# Operations

- **Status:** Active; local-only operating mode
- **Owner:** Operations (role not yet assigned)
- **Last reviewed:** 2026-10-26
- **Readiness gate:** [Production readiness](production-readiness.md) — current answer: **not production ready**

## Operating boundary

There is no approved deployment target. The supported mode is a single local operator running the CLI and, optionally, the loopback-only governance workbench. Do not expose `governance_web` to a shared host, LAN, container ingress or the public internet. Do not store real customer, personal or regulated data in the prototype.

## Document index

| Document | Purpose | Status |
| --- | --- | --- |
| [Installation](../deployment/install.md) | Prerequisites, build and run commands | Current; local only |
| [Configuration](../deployment/config.md) | Engine parameters, data files and environment variables | Current; local only |
| [Monitoring](../deployment/monitoring.md) | Current observability and target requirements | Target; no production monitoring |
| [Backup and recovery](../deployment/backup.md) | Short operator pointer | Current; canonical procedure is `local-backup.md` |
| [Local backup](local-backup.md) | Git bundle cadence, restore drill and scope | Current |
| [Backup log](backup-log.md) | Executed backup/restore evidence | Evidence log; local PASS drills at `283f83a` and `7f6ab97`; one earlier host-blocked attempt recorded 2026-10-26 |
| [Off-host backup and RPO/RTO proposal](backup-policy-draft.md) | Draft policy for encrypted off-host copies, RPO/RTO and scheduled restore tests | Proposal; not approved or implemented |
| [Pilot runbook and ownership proposal](pilot-runbook.md) | One-workflow pilot procedure, evidence record, stop conditions and proposed owners | Draft; not executed; no owner assigned |
| [Security hardening](../deployment/security.md) | Current controls vs target controls | Current boundary documented |
| [Troubleshooting](../deployment/troubleshooting.md) | Common local build/test issues | Current |
| [Local-first workflow](local-first-workflow.md) | Git-first, offline development and remote-upgrade criteria | Current |
| [Secrets policy](secrets-policy.md) | No-secrets rule, `.env` handling and incident response | Current |
| [Incident log](incident-log.md) | Operational/security incident record | Empty until an incident is recorded |
| [Production readiness](production-readiness.md) | Phase B readiness checklist and blockers | Blocked |
| [Final validation report](final-validation-report.md) | Dated 2026-10-25 validation snapshot | Historical; not current evidence |
| [Remote collaboration criteria](../remote-collaboration-criteria.md) | When to add a remote and CI | Current |

## Routine local operation

1. Work from a clean Git checkout; run `scripts/check.ps1` or `scripts/check.sh`.
2. For report/export/coverage/HTML changes, run `gate.ps1 -IncludeSmoke`.
3. Before a release or tag, run the backup and restore drill:
   ```powershell
   powershell -ExecutionPolicy Bypass -File scripts/backup.ps1
   ```
   Record the bundle path and result in [backup-log.md](backup-log.md).
4. For the web workbench, use a disposable ledger path for smoke tests:
   ```powershell
   $env:EKP_GOVERNANCE_DATA = Join-Path $env:TEMP ("ekpi-governance-smoke-" + [guid]::NewGuid().ToString("N") + ".jsonl")
   cd platform
   cargo run --locked --bin governance_web
   ```
   Open `http://127.0.0.1:3000`, use synthetic data only, then stop the server.
5. Record evidence, known limits and any failed/skipped check in the release or review record. Never report an unrun check as passed.

## What is not available

- No production environment, TLS, reverse proxy, container or service manager.
- No enterprise identity, MFA, role administration or tenant isolation.
- No centralized logs, metrics, traces, alerting, on-call or SLAs.
- No automated deployment, rollback or blue/green release.
- No live Microsoft/Google connector, LLM or PM4Py call.
- No approved data classification, retention or incident-response owner.

These gaps are tracked in [production readiness](production-readiness.md). They are not documentation omissions to be filled with optimistic text.

## Incident handling (current local scope)

For a local security or data incident:

1. Stop the process and preserve the ledger/report files.
2. Do not delete evidence; record the time, operator, affected path and observed behavior in [incident-log.md](incident-log.md).
3. If a secret was exposed, rotate it and follow [secrets policy](secrets-policy.md).
4. If the ledger or a report is suspected to be tampered with, preserve a copy and verify the hash chain; do not overwrite it.
5. Record the follow-up action and link it in the [improvement register](../reviews/README.md).

This is a local checklist, not a production incident-response plan. Phase B requires an owner, severity levels, notification path and drill evidence.

## Update triggers

Update this index when an operational document is added, superseded or changes status. Update the readiness page when a control, owner, environment or evidence changes. Keep the canonical procedure in `operations/`; deployment pages should point here rather than duplicate it.
