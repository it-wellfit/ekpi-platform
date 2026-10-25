# EKPI Backup Log

- **Status:** Active; one local restore drill executed
- **Owner:** Engineering / Operations (operations owner not yet assigned)
- **Last reviewed:** 2026-10-26
- **Procedure:** [local-backup.md](local-backup.md)
- **Phase gate:** [A8](../quality/phase-gates.md#phase-a-0--1-mvp-gate)
- **Production gap:** [production-readiness.md](production-readiness.md)

| Date | Bundle path | Restore result | Operator | Notes |
|---|---|---|---|---|
| 2026-10-26 | `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-26-012202.bundle` | PASS | OpenCode (DeepSeek V4.1) | Revision `283f83a`; SHA-256 `21D2CD808A2FECF469CAD334DB2CC6CF1CA2DBF52DAA3CB0B6192AFDE47ED443`; fresh clone ran `scripts/check.ps1`: docs 44 required/427 links in 92 files, secret scan 197 files, eval 9 tests + 10 self-test checks, platform 183 tests + 49 self-test assertions, schema 20 records; local-only bundle; uncommitted work excluded; no off-host copy or approved RPO/RTO |

> Add one row per backup. A backup is verified only when the restore drill
> (the restored `scripts/check` exits 0 on a fresh clone of the bundle) passes.
> Record the tested revision and bundle SHA-256. A `PASS` here proves the
> committed revision can be restored and checked; it does not prove an
> off-host backup, an approved RPO/RTO or a scheduled restore programme.
>
> The bundle is created before this row is committed, so a restore of that
> bundle does not contain its own log entry. The row is evidence in the later
> revision; the next backup includes it.
