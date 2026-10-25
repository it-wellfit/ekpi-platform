# Local backup and restore procedure

- **Status:** Procedure defined; **no executed restore evidence yet**
- **Owner:** Engineering / Operations (operations owner not yet assigned)
- **Last reviewed:** 2026-10-26
- **Evidence log:** [backup-log.md](backup-log.md)
- **Production gap:** [production-readiness.md](production-readiness.md)

## Principle

Git history is the source of truth for committed work. A Git bundle is a portable, verifiable copy of that history. It is **not** a full system backup: it excludes uncommitted/untracked files, ignored secrets, generated artifacts that were never committed and the local governance ledger outside the repository.

Create the first backup at the first important version, then weekly or before any release/tag. Store bundles outside the workspace.

## Create and verify

Windows:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backup.ps1
```

POSIX:

```bash
./scripts/backup.sh
```

The script:

1. creates `ekpi-platform-<timestamp>.bundle` with `git bundle create --all` in `%USERPROFILE%\EKPI-backup\bundles\` or `~/EKPI-backup/bundles/`;
2. clones the bundle to a temporary directory;
3. runs the restored repository's check script (`scripts/check.ps1` or `check.sh`) — format, Clippy, tests, self-tests and schema validation;
4. reports `RESTORE DRILL PASSED` or fails non-zero.

A backup is verified only if the restored clone passes the drill. Do not mark it verified from the bundle's existence or size.

## Ledger and uncommitted work

- The local governance ledger (default `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` or `EKP_GOVERNANCE_DATA`) is outside Git. Copy it separately if it contains data you need to retain, and verify its hash chain on `/audit`.
- Commit important work before backing up. A bundle cannot recover uncommitted changes.
- `.env`, keys and local secrets must not be placed in the bundle or any backup artifact.
- Generated reports/exports are reproducible from the CLI; do not treat them as authoritative backups.

## Restore drill steps (manual equivalent)

1. `git clone <bundle> <tmp-path>`
2. In the clone, run `scripts/check.ps1` (Windows) or `scripts/check.sh` (POSIX).
3. If the change touches report/export/HTML/data, also run `gate.ps1 -IncludeSmoke` in the clone.
4. Restore the ledger separately if required; verify the audit chain.
5. Record date, bundle path, result, operator and any limitation in [backup-log.md](backup-log.md).
6. If the drill fails, block the release, preserve the failing output and fix the cause. Do not lower the drill.

## Record format

| Date | Bundle path | Restore result | Operator | Notes |
|---|---|---|---|---|
| | | | | |

Add one row per backup. `Restore result` is `PASS`, `FAIL` or `NOT RUN`; never leave an unexecuted drill implied as passed.

## Production gap

The local drill does not provide off-host encrypted backup, approved RPO/RTO, scheduled restore testing, integrity monitoring or a named backup owner. Those are Phase B requirements (`DEC-14`). See [production readiness](production-readiness.md).

## Update triggers

Update this procedure when the backup scope, script, storage location, ledger path or drill changes. Keep the log current.
