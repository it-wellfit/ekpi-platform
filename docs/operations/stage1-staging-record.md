# Stage 1 Staging Record

- **Status:** Template; Stage 1 has **not been executed** on an approved host. This record is not an approval, a production-readiness claim or a Phase A gate result.
- **Owner:** Operations + Engineering (operations owner not yet assigned)
- **Scope:** [Stage 1 of the staging checklist](staging-checklist-draft.md): one internal Windows host, native binary, manual foreground start, loopback only, synthetic or authorized non-sensitive data.
- **Not in scope:** LAN exposure, TLS, Windows service hosting, PostgreSQL storage, off-host backup, real customer data.

## Release identity

| Field | Value | Evidence |
| --- | --- | --- |
| Revision | `<commit>` | `git rev-parse HEAD` |
| Build command | `cargo build --release --locked` | output / exit code |
| Binary | `governance_web.exe` | SHA-256, size |
| Data directory | `platform/data` | manifest of `docs.json`, `rules.json`, `processes.json` with SHA-256 |
| Toolchain | `rust-toolchain.toml` (`1.99.1`) | `rustc --version` |
| Artifact transfer | source host → staging host | SHA-256 compared on both sides |

## Environment (no secrets)

| Variable | Value |
| --- | --- |
| `EKPI_LOCAL_AUTH_USER` | `<operator>` |
| `EKPI_LOCAL_AUTH_ROLE` | `approver` |
| `EKP_GOVERNANCE_DATA` | `<dedicated ledger path>` |
| `EKP_APPROVAL_DATA` | `<dedicated approval path>` |
| `EKPI_DATA_DIR` | `<staged platform/data>` |
| `EKPI_BIND_IP` | `127.0.0.1` |
| `EKPI_WEB_PORT` | `3000` |
| `EKPI_COOKIE_SECURE` | unset |

Do not record the password here. Record only that it was set in the console session and cleared afterwards.

## Smoke evidence

| Check | Command | Expected | Observed | Result |
| --- | --- | --- | --- | --- |
| Liveness | `Invoke-WebRequest -UseBasicParsing -Uri "http://127.0.0.1:3000/healthz"` | 200 | | `Not run` |
| Readiness | `Invoke-WebRequest -UseBasicParsing -Uri "http://127.0.0.1:3000/ready"` | 200 + `"status":"ready"` | | `Not run` |
| Metrics | `Invoke-WebRequest -UseBasicParsing -Uri "http://127.0.0.1:3000/metrics"` | 200 | | `Not run` |
| Login | Browser with synthetic credentials | Dashboard | | `Not run` |
| Search | Logged-in browser: `/search?q=變更執行前，須完成風險評估並取得核准紀錄` | Results; no `文件庫載入失敗` | | `Not run` |
| Chains | `/audit/chains` | Both chains verified | | `Not run` |
| End-to-end | Create project → link evidence → review → submit → decide | `/events` count increases | | `Not run` |
| CLI selftest | `cargo run --release --locked --bin ekp_platform -- --selftest` | 49 assertions PASS | | `Not run` |

## Drill and backup evidence

| Check | Command | Result | Evidence |
| --- | --- | --- | --- |
| Runtime-data drill | `powershell -ExecutionPolicy Bypass -File scripts/drill-runtime-data.ps1` | `Not run` | |
| Repository bundle | `powershell -ExecutionPolicy Bypass -File scripts/backup.ps1` | `Not run` | bundle path + SHA-256 in [backup-log.md](backup-log.md) |
| Paired JSONL restore | copy `EKP_GOVERNANCE_DATA` + `EKP_APPROVAL_DATA` to temp; start on another port; verify `/audit/chains` and `/approvals` | `Not run` | |

## Decision

- [ ] Keep staging running
- [ ] Roll back
- [ ] Stop and escalate

Record the decision, operator, timestamp and reason. A checked box without evidence is not done.

## Sign-off

| Role | Name | Date | Statement |
| --- | --- | --- | --- |
| Operator | `<blank>` | `<blank>` | I ran the commands above and recorded the observed output. |
| Engineering reviewer | `<blank>` | `<blank>` | The evidence matches the named revision and the Stage 1 scope. |

This record does not approve LAN exposure, production use, enterprise identity, tenant isolation, WORM retention or an off-host backup. Those remain Phase B items (DEC-12/13/14/16/20; PR-02).