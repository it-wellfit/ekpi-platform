# Automated backup scheduling for EKPI platform
# This script sets up Windows Task Scheduler for automated backups

param(
    [string]$BackupDir = "C:\EKPI\backups",
    [string]$DataDir = "C:\EKPI\data",
    [string]$TaskName = "EKPI-Automated-Backup",
    [int]$RetentionDays = 7,
    [string]$Schedule = "Daily",
    [string]$Time = "02:00"
)

# Create backup directory if it doesn't exist
if (-not (Test-Path $BackupDir)) {
    New-Item -ItemType Directory -Path $BackupDir -Force | Out-Null
    Write-Host "Created backup directory: $BackupDir"
}

# Create the backup script
$backupScript = @"
# EKPI Platform Backup Script
# Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

param(
    [string]`$BackupDir = "$BackupDir",
    [string]`$DataDir = "$DataDir",
    [int]`$RetentionDays = $RetentionDays
)

`$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
`$backupFile = Join-Path `$BackupDir "ekpi-backup-`$timestamp.zip"

# Create backup
try {
    if (Test-Path `$DataDir) {
        Compress-Archive -Path `$DataDir\* -DestinationPath `$backupFile -Force
        Write-Host "Backup created: `$backupFile"
        
        # Clean up old backups
        `$oldBackups = Get-ChildItem -Path `$BackupDir -Filter "ekpi-backup-*.zip" -ErrorAction SilentlyContinue |
            Where-Object { `$_.LastWriteTime -lt (Get-Date).AddDays(-`$RetentionDays) }
        if (`$oldBackups) {
            `$oldBackups | Remove-Item -Force
            Write-Host "Cleaned up `$(`$oldBackups.Count) backups older than $RetentionDays days"
        } else {
            Write-Host "No old backups to clean up"
        }
    } else {
        Write-Warning "Data directory not found: `$DataDir"
    }
} catch {
    Write-Error "Backup failed: $_"
    exit 1
}
"@

$backupScriptPath = Join-Path $BackupDir "backup.ps1"
$backupScript | Out-File -FilePath $backupScriptPath -Encoding UTF8

# Create scheduled task
$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-ExecutionPolicy Bypass -File `"$backupScriptPath`""
$trigger = New-ScheduledTaskTrigger -Daily -At $Time
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

# Register the task
try {
    Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Force | Out-Null
    Write-Host "Scheduled task '$TaskName' created successfully"
    Write-Host "Backup will run daily at $Time"
    Write-Host "Backups will be stored in: $BackupDir"
    Write-Host "Retention: $RetentionDays days"
} catch {
    Write-Error "Failed to create scheduled task: $_"
    exit 1
}

Write-Host "`nBackup automation setup complete!"
Write-Host "To test the backup manually, run:"
Write-Host "  powershell -ExecutionPolicy Bypass -File `"$backupScriptPath`""
