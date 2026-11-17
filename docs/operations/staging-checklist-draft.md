# Staging checklist for the internal single-host direction (draft)

- **Status:** Draft proposal; **not approved**, **not implemented**
- **Owner:** Operations + Security (proposed; acceptance required)
- **Last reviewed:** 2026-11-10
- **Phase gate:** [Phase B](../quality/phase-gates.md#phase-b-mvp--production-grade-gate) (B1/B4/B7)
- **Readiness row:** [PR-02](production-readiness.md) — deployment target and environment
- **External blocker:** [B-10](../assurance/external-blockers.md) — production deployment target and change approval
- **Direction of record:** 2026-11-10 owner decision — single internal Windows host, native binary; recorded in [PR-02](production-readiness.md) and [B-10](../assurance/external-blockers.md) (session `20261110-dev-blockers`)
- **Related:** [install](../deployment/install.md), [security](../deployment/security.md), [monitoring](../deployment/monitoring.md), [local backup](local-backup.md), [release evidence bundle](release-evidence-bundle.md)

## Purpose and limit

This checklist operationalises the first step of the chosen direction: a **staging** deployment on a single internal Windows host, before any production or LAN-wide exposure. It is not an approval, not a deployment record and not evidence that staging exists.

Until the PR-02 fields and the Stage 2 gate below are approved and implemented, the existing operating boundary stands: no shared-host, LAN, container or public exposure; synthetic or authorized non-sensitive data only.

## PR-02 fields to fill (owner blanks)

| Field | Value | Owner |
| --- | --- | --- |
| Host name / OS build | `<blank>` | Operations |
| Service account (least privilege) | `<blank>` | Operations + Security |
| Network boundary (who can reach the port) | Stage 1: local logon only; Stage 2 requires an approved LAN decision | Security |
| TLS (certificate, custody, renewal) | `<blank>` | Security |
| Host hardening baseline | per [deployment/security.md](../deployment/security.md) | Security |
| Capacity and cost owner | `<blank>` | Operations |
| Change approver | `<blank>` | Change approver |

## Stage 1 — single-host staging (this checklist)

Stage 1 is a **manual-start** deployment on one internal Windows host. It does
not register a Windows service: `governance_web` is a console binary without an
SCM `ServiceMain`, so `sc.exe create` would fail with error 1053. Service
hosting is a Stage 2 design item.

### Storage decision

Stage 1 uses the JSONL governance ledger and the JSONL approval store only.
`EKPI_STORAGE=postgres` is not wired into `governance_web` and must not be set
in Stage 1. PostgreSQL wiring is a Stage 2 item requiring a `GovernanceRepository`
port plus an approval port abstraction, real PostgreSQL 18 integration tests,
and a migration account.

### Preconditions

- [ ] Clean Git revision; commit hash recorded
- [ ] Fresh local bundle + restore drill PASS recorded in [backup-log.md](backup-log.md) (the 2026-11-17 bundle is currently **FAIL** at the restored docs check; resolve the out-of-repo link policy before Stage 1)
- [ ] Synthetic or authorized non-sensitive dataset only; no real customer data
- [ ] Host fields above filled and approved for **staging scope only**
- [ ] A dedicated non-administrator operator account is used for the console session

### Build and stage artifacts

- [ ] `cargo build --release --locked` exit 0 on the host; binary SHA-256 recorded
- [ ] Artifacts staged: `governance_web.exe` + `platform/data/{docs.json,rules.json,processes.json}`
- [ ] SHA-256 of the binary and data manifest compared on both sides; values recorded
- [ ] Data directory is a **local fixed disk** (not SMB/network drive; file locks are unreliable)
- [ ] Dedicated directories for `EKP_GOVERNANCE_DATA`, `EKP_APPROVAL_DATA` and logs

### Environment

Set in the console session before start (never commit real values):

| Variable | Stage 1 value |
| --- | --- |
| `EKPI_LOCAL_AUTH_USER` | ASCII operator name; no default account |
| `EKPI_LOCAL_AUTH_PASSWORD` | 8–128 chars; enter with `Read-Host -AsSecureString`, not as a literal command |
| `EKPI_LOCAL_AUTH_ROLE` | `approver` for the end-to-end smoke |
| `EKP_GOVERNANCE_DATA` | Dedicated ledger path |
| `EKP_APPROVAL_DATA` | Dedicated approval path (back up together with the ledger) |
| `EKPI_DATA_DIR` | Staged `platform/data` directory |
| `EKPI_BIND_IP` | `127.0.0.1` (loopback only; non-loopback refuses startup) |
| `EKPI_WEB_PORT` | `3000` |
| `EKPI_COOKIE_SECURE` | **unset** (plain HTTP; `Secure` cookies may break login) |

### Start and stop

- [ ] Start in the foreground: `.\governance_web.exe *> D:\EKPI\stage1\logs\stage1.log`
- [ ] Do **not** register a service, enable auto-start, or open firewall ports
- [ ] Only one instance runs; a second instance is refused by the ledger/approval lock (expected)
- [ ] Stop with **Ctrl+C** in the console; do not close the window (may act as a hard kill)
- [ ] After an abnormal stop, verify the chain on `/audit/chains`; if the ledger refuses to load (torn final line), restore the last paired backup and record an incident

### Smoke and health (evidence per check)

- [ ] `Invoke-WebRequest -UseBasicParsing -Uri "http://127.0.0.1:3000/healthz"` → 200 (`curl.exe` returned 000 on the dev host; `curl` is a PowerShell alias)
- [ ] `/ready` → 200 with `"status":"ready"` (readiness checks ledger + auth, not the data directory)
- [ ] `/metrics` → 200
- [ ] One local login with synthetic credentials → dashboard
- [ ] In the logged-in browser, open `/search?q=變更執行前，須完成風險評估並取得核准紀錄` → results, no "文件庫載入失敗"
- [ ] `/audit/chains` shows both chains verified
- [ ] One end-to-end workflow: create project → link evidence → review → submit approval → decide → `/events` count increases
- [ ] CLI selftest: `cargo run --release --locked --bin ekp_platform -- --selftest` → 49 assertions PASS
- [ ] Host time recorded (`Get-Date`); approval `due_at` uses Unix time, so large skew must be recorded
- [ ] Logs land in the agreed path; no secrets in logs

### Rollback

- [ ] Stop the process; restore the previous binary + data + ledger/approvals; re-run smoke
- [ ] Recorded decision: keep staging running / roll back

### Stop conditions (do not proceed; roll back and record)

- Any real customer data on disk; missing evidence; failed health/smoke; `/search` load failure; ledger lock refusal; unexplained log errors.

### Evidence

Record every checkbox result (command, revision, output, hash) in
[stage1-staging-record.md](stage1-staging-record.md) and link it from
[PR-02](production-readiness.md). Record failures or data anomalies in
[incident-log.md](incident-log.md). A checked box without evidence is not done.

## Stage 2 gate (out of scope here)

LAN exposure, TLS termination and any non-synthetic data require PR-03 (identity), PR-04 (isolation) and PR-11 (security assurance) progress plus an approved change. Do not enable auto-start or open firewall ports before that approval.

## Evidence

Record every checkbox result (command, revision, output, hash) in the release evidence bundle and link it from [PR-02](production-readiness.md). A checked box without evidence is not done.
