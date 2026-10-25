# Backup & Disaster Recovery

## Local Backup

```powershell
# Backup data directory
Compress-Archive -Path platform/data -DestinationPath backups/data-$(Get-Date -Format 'yyyyMMdd').zip
# Backup config
Compress-Archive -Path platform/config -DestinationPath backups/config-$(Get-Date -Format 'yyyyMMdd').zip
```

## Verification

```powershell
# Verify backup integrity
Test-Path backups/data-*.zip
# Restore
Expand-Archive -Path backups/data-*.zip -DestinationPath platform/
```

## Disaster Recovery

1. Restore from latest backup
2. Re-run `cargo build --locked`
3. Re-run `cargo test --locked`
4. Re-run `--selftest` to verify
