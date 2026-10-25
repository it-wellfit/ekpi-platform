# EKPI Local Backup Procedure

## Cadence

- Full repository bundle: weekly (or before any release).
- Store bundles in a local backup directory outside the workspace, e.g.
  `%USERPROFILE%\EKPI-backup\bundles\` (Windows) or `~/EKPI-backup/bundles/` (POSIX).

## Backup command (PowerShell)

```powershell
$bundleDir = "$env:USERPROFILE\EKPI-backup\bundles"
New-Item -ItemType Directory -Force -Path $bundleDir | Out-Null
$name = "ekpi-platform-$(Get-Date -Format 'yyyy-MM-dd').bundle"
git bundle create "$bundleDir\$name" --all
```

## Restore drill (verify every backup)

1. Clone the bundle to a temp path: `git clone <bundle> <tmp-path>`
2. Run `cargo test --locked` in both `eval/` and `platform/` → exit 0
3. Run `cargo clippy --locked --all-targets -- -D warnings` in both → exit 0
4. Run `cargo fmt -- --check` in both → exit 0
5. If all pass, the backup is verified; otherwise block release and retry.

## Backup log

Record each backup + restore result in `docs/operations/backup-log.md`:
date, bundle path, restore result (pass/fail), operator.

Format per entry:

| Date | Bundle path | Restore result | Operator | Notes |
|---|---|---|---|---|

A backup is considered verified only when the restore drill passes.