# EKPI Backup Log

- **Status:** Active; local restore drills PASS at `283f83a`, `7f6ab97` and `5328787`; one earlier attempt (`198d5f3`) BLOCKED by host policy and not counted as verification
- **Owner:** Engineering / Operations (operations owner not yet assigned)
- **Last reviewed:** 2026-10-26
- **Procedure:** [local-backup.md](local-backup.md)
- **Phase gate:** [A8](../quality/phase-gates.md#phase-a-0--1-mvp-gate)
- **Production gap:** [production-readiness.md](production-readiness.md)

| Date | Bundle path | Restore result | Operator | Notes |
|---|---|---|---|---|
| 2026-10-26 | `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-26-012202.bundle` | PASS | OpenCode (DeepSeek V4.1) | Revision `283f83a`; SHA-256 `21D2CD808A2FECF469CAD334DB2CC6CF1CA2DBF52DAA3CB0B6192AFDE47ED443`; fresh clone ran `scripts/check.ps1`: docs 44 required/427 links in 92 files, secret scan 197 files, eval 9 tests + 10 self-test checks, platform 183 tests + 49 self-test assertions, schema 20 records; local-only bundle; uncommitted work excluded; no off-host copy or approved RPO/RTO |
| 2026-10-26 | `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-26-020948.bundle` | BLOCKED | OpenCode (DeepSeek V4.1) | Revision `198d5f3`; SHA-256 `E6EAB351E04BC7E9943704DC8E0438307B2302E602B986086900CF3A9D903847`; `git bundle verify` okay; fresh clone reached `198d5f3` and passed docs (44 required/444 links in 92 files) and secret scan (201 files, including the local drill log); the full check stopped at eval clippy because Windows Application Control refused to execute freshly built build scripts (`os error 4551`); code-identical revision `f34c0d6` (docs-only diff) had passed a clean-clone `gate.ps1 -IncludeSmoke` 13/13 before the policy block; restore is **not verified**; local-only bundle; no off-host copy or approved RPO/RTO |
| 2026-10-26 | `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-26-105233.bundle` | PASS | OpenCode (DeepSeek V4.1) | Revision `7f6ab97`; SHA-256 `4D35413B0963BBB2C45F0169F2EC1039CEDC66833093556D7336266E3192B276`; `git bundle verify` okay and records a complete history; fresh clone reached `7f6ab97`; restored `scripts/check.ps1` exited 0: docs 44 required/488 links in 95 files, secret scan 205 files, eval 9 tests + 10 self-test checks, platform 233 tests (200 unit, 2 edge, 3 web, 8 integration, 7 search evaluation, 13 CLI contract) + 49 self-test assertions, schema 20 records, search gate 28 cases 20/20 top-1 / 8/8 refusals / 4 role-restricted (2 allowed, 2 denied) / 0 violations; local-only bundle; uncommitted work excluded; no off-host copy or approved RPO/RTO |
| 2026-10-26 | `%USERPROFILE%\EKPI-backup\bundles\ekpi-platform-2026-10-26-110117.bundle` | PASS | OpenCode (DeepSeek V4.1) | Revision `5328787` (docs-only diff from `7f6ab97`; program revision unchanged); SHA-256 `789626183B04AA0429214F902E79FE4924D469431FAC14758CF7DAF339D0E93F`; `git bundle verify` okay and records a complete history; fresh clone reached `5328787`; restored `scripts/check.ps1` exited 0: docs 44 required/489 links in 95 files, secret scan 205 files, eval 9 tests + 10 self-test checks, platform 233 tests (200 unit, 2 edge, 3 web, 8 integration, 7 search evaluation, 13 CLI contract) + 49 self-test assertions, schema 20 records, search gate 28 cases 20/20 top-1 / 8/8 refusals / 4 role-restricted (2 allowed, 2 denied) / 0 violations; local-only bundle; uncommitted work excluded; no off-host copy or approved RPO/RTO |

> The `198d5f3` row remains **BLOCKED**, not PASS. The host's Windows Application
> Control denied execution of freshly compiled build scripts (`os error 4551`).
> The later `7f6ab97` drill passed on the same host after the narrow, committed
> `.cargo/config.toml` profile overrides produced executable artifacts. The
> `198d5f3` failure is historical and is not retroactively converted into a pass.
>
> Add one row per backup. A backup is verified only when the restore drill
> (the restored `scripts/check` exits 0 on a fresh clone of the bundle) passes.
> Record the tested revision and bundle SHA-256. A `PASS` here proves the
> committed revision can be restored and checked; it does not prove an
> off-host backup, an approved RPO/RTO or a scheduled restore programme.
>
> The bundle is created before this row is committed, so a restore of that
> bundle does not contain its own log entry. The row is evidence in the later
> revision; the next backup includes it.
