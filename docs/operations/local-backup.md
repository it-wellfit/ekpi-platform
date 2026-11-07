# Local backup and restore procedure

- **Status:** Procedure defined; local restore drills PASS at `283f83a`, `7f6ab97`, `5328787`, `7fd0c0b`, `56a944f`, `3ee2ee2`, `41beaea`, `6ed236d` and `499e0d1` (the last three via the automatic TEMP drill with restored `scripts/check.ps1` exit 0); earlier `198d5f3`, `c77fec4`, `bc7671a` and `28b68cc` attempts were BLOCKED by host policy and are not counted as verification
- **Owner:** Engineering / Operations (operations owner not yet assigned)
- **Last reviewed:** 2026-10-27
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

1. creates `ekpi-platform-<timestamp>.bundle` with `git bundle create --all` in `%USERPROFILE%\EKPI-backup\bundles\` or `~/EKPI-backup/bundles/`, and prints its SHA-256;
2. clones the bundle to a temporary directory;
3. runs the restored repository's check script (`scripts/check.ps1` or `check.sh`) — documentation links, high-signal secret scan, format, Clippy, tests, self-tests and schema validation;
4. reports `RESTORE DRILL PASSED` or fails non-zero.

A backup is verified only if the restored clone passes the drill. Do not mark it verified from the bundle's existence or size.

On a managed host the automatic `%TEMP%` drill can fail because Windows Application Control denies a freshly built binary (`os error 4551`). Do not edit the check or bypass the policy. Record the automatic run as **Blocked**, then perform the manual equivalent: clone the same bundle to a path under the user profile, run the restored `scripts/check.ps1` there and record both results. If the manual clone passes, the bundle is restore-verified; the blocked automatic run is still not a PASS.

## Encrypted off-host drill (Windows, `age`)

`scripts/backup-offhost.ps1` wraps the local drill, encrypts the new bundle with
[`age`](https://age-encryption.org/) to an operator-supplied destination and can
decrypt it again for verification. It refuses a dirty tree, a destination inside
the repository, the local bundle directory, or a run without `-Drill` or
`-AcknowledgeOffHost`.

```powershell
# Generate a throwaway key pair for a drill.
age-keygen -o "$env:TEMP\ekpi-age-drill-key.txt"
$recipient = (Select-String -Path "$env:TEMP\ekpi-age-drill-key.txt" `
    -Pattern '^# public key: (age1[0-9a-z]+)$').Matches.Groups[1].Value

# Drill into a temporary directory; verify the decrypted bundle byte-for-byte.
powershell -ExecutionPolicy Bypass -File scripts/backup-offhost.ps1 `
    -Destination "$env:TEMP\ekpi-offhost-drill" `
    -Recipient $recipient `
    -IdentityFile "$env:TEMP\ekpi-age-drill-key.txt" `
    -Drill
```

The receipt records the revision, bundle and ciphertext hashes, the restore
result and `destination_class: local-simulated-off-host`. A `-Drill` run is
**not** evidence of an off-host copy. A real run requires
`-AcknowledgeOffHost`, an approved destination outside the host and a managed
key-custody process; none of those exist in this repository.

Both PowerShell scripts use a .NET SHA-256 helper instead of `Get-FileHash`,
which is unavailable on this managed host. Native `git` and `age` output on
stderr is handled with `ErrorActionPreference=Continue` and an explicit exit-code
check so a successful command is not mistaken for a failure (and a failure is
never mistaken for success).

## Ledger and uncommitted work

- The local governance ledger (default `%LOCALAPPDATA%\EKPI\governance-ledger.jsonl` or `EKP_GOVERNANCE_DATA`) is outside Git. Copy it separately if it contains data you need to retain, and verify its hash chain on `/audit`.
- The approval workflow store (default `%LOCALAPPDATA%\EKPI\approvals.jsonl` or `EKP_APPROVAL_DATA`, colocated with the ledger) is likewise outside Git: copy both files together so the two chains describe the same point in time, and verify each chain (`/audit` for governance, the integrity badge on `/approvals` for approvals). No runtime-data restore drill has been executed yet (`IMP-021`); the log rows above drill the repository only.
- Commit important work before backing up. A bundle cannot recover uncommitted changes.
- `.env`, keys and local secrets must not be placed in the bundle or any backup artifact.
- Generated reports/exports are reproducible from the CLI; do not treat them as authoritative backups.

## Restore drill steps (manual equivalent)

1. `git clone <bundle> <tmp-path>`
2. In the clone, run `scripts/check.ps1` (Windows) or `scripts/check.sh` (POSIX).
3. If the change touches report/export/HTML/data, also run `gate.ps1 -IncludeSmoke` in the clone.
4. Restore the ledger separately if required; verify the audit chain.
5. Record date, bundle path, result, operator and any limitation in [backup-log.md](backup-log.md). If the automatic drill was blocked by host policy, record that as `BLOCKED` and the manual clone separately as `PASS`; never merge the two into one unqualified PASS.
6. If the drill fails for a code or data reason, block the release, preserve the failing output and fix the cause. Do not lower the drill.

## Record format

| Date | Bundle path | Restore result | Operator | Notes |
|---|---|---|---|---|
| | | | | |

Add one row per backup. `Restore result` is `PASS`, `FAIL` or `NOT RUN`; never leave an unexecuted drill implied as passed.

## Production gap

The local drill does not provide off-host encrypted backup, approved RPO/RTO, scheduled restore testing, integrity monitoring or a named backup owner. Those are Phase B requirements (`DEC-14`). See [production readiness](production-readiness.md) and the unapproved [off-host backup and RPO/RTO proposal](backup-policy-draft.md).

## Update triggers

Update this procedure when the backup scope, script, storage location, ledger path or drill changes. Keep the log current.
