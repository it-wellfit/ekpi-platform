# Backup and disaster recovery

- **Status:** Procedure defined; local restore drills PASS at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f`, `3ee2ee2`, `41beaea`, `6ed236d` and `499e0d1` (the last three via the automatic TEMP drill with restored `scripts/check.ps1` exit 0); earlier `198d5f3`, `c77fec4`, `bc7671a` and `28b68cc` attempts were BLOCKED by host policy and are not counted as verification
- **Owner:** Engineering / Operations (operations owner not yet assigned)
- **Last reviewed:** 2026-10-27
- **Canonical procedure:** [Local backup](../operations/local-backup.md)
- **Evidence log:** [Backup log](../operations/backup-log.md)

## What exists

- `scripts/backup.ps1` and `scripts/backup.sh` create a Git bundle in a directory outside the workspace and run a restore drill.
- The drill clones the bundle to a temporary path and runs the repository check script in the clone.
- `docs/operations/backup-log.md` is the evidence record. It contains local PASS drills at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b` and `56a944f`, including an `age` drill with a local-simulated destination, and one BLOCKED attempt (`198d5f3`, host Application Control denied freshly built build scripts). A blocked row is not verification.

## What the bundle contains and excludes

A `git bundle --all` contains committed Git history and refs. It does **not** contain:

- uncommitted or untracked working-tree files;
- ignored files such as `.env` and local keys;
- the local governance ledger outside the repository (default `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` or `EKP_GOVERNANCE_DATA`);
- generated artifacts that were not committed.

A bundle is **not** a full system backup. It does not replace an encrypted off-host backup, database backup (there is no database in the current prototype), or a documented RPO/RTO. For the local prototype, back up the ledger separately if it contains data you need to retain.

## Run the drill

```powershell
powershell -ExecutionPolicy Bypass -File scripts/backup.ps1
```

```bash
./scripts/backup.sh
```

The drill is verified only when the restored clone passes the check. Record the date, bundle path, result and operator in [backup-log.md](../operations/backup-log.md). An empty log means **unverified**, not “no incidents”.

## Disaster recovery (local)

1. Clone the latest verified bundle to a new directory.
2. Run `scripts/check.ps1` (Windows) or `scripts/check.sh` (POSIX) in the clone.
3. Restore the ledger separately if it is needed, and verify the hash chain on `/audit`.
4. Record the recovery drill and any data-loss gap in the backup log and, if material, the [incident log](../operations/incident-log.md).
5. Do not overwrite the only copy of a suspect ledger; preserve it before repair.

## Production gap

Phase B requires off-host encrypted backup, approved RPO/RTO, scheduled restore tests, integrity checks and a named backup owner (`DEC-14`). Those do not exist. See [production readiness](../operations/production-readiness.md).

## Update triggers

Update this page when the backup scope, script, storage location, ledger path or recovery procedure changes. Keep the backup log current; do not mark a backup verified without a passing restore drill.
