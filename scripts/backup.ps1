# Local Backup Script (PowerShell)
# Creates a git bundle and runs a restore drill. Records result in backup-log.md.

$ErrorActionPreference = "Stop"
$root = Resolve-Path "$PSScriptRoot\.."
$bundleDir = "$env:USERPROFILE\EKPI-backup\bundles"
New-Item -ItemType Directory -Force -Path $bundleDir | Out-Null

$name = "ekpi-platform-$(Get-Date -Format 'yyyy-MM-dd-HHmmss').bundle"
$bundlePath = "$bundleDir\$name"

Write-Host "Creating bundle: $bundlePath" -ForegroundColor Cyan
Set-Location $root
git bundle create "$bundlePath" --all
if ($LASTEXITCODE -ne 0) { Write-Host "FAIL: bundle create" -ForegroundColor Red; exit 1 }
Write-Host "Bundle created." -ForegroundColor Green

# Restore drill
$drillDir = "$env:TEMP\ekpi-restore-drill"
if (Test-Path $drillDir) { Remove-Item -Recurse -Force $drillDir }
Write-Host "Restore drill: cloning bundle..." -ForegroundColor Cyan
git clone "$bundlePath" "$drillDir" 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "FAIL: clone" -ForegroundColor Red; exit 1 }

$pass = $true
foreach ($crate in @("eval", "platform")) {
    Set-Location "$drillDir\$crate"
    Write-Host "  [$crate] cargo fmt -- --check" -ForegroundColor Yellow
    cargo fmt -- --check 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { $pass = $false; Write-Host "  FAIL fmt" -ForegroundColor Red }

    Write-Host "  [$crate] cargo clippy --locked --all-targets -- -D warnings" -ForegroundColor Yellow
    cargo clippy --locked --all-targets -- -D warnings 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { $pass = $false; Write-Host "  FAIL clippy" -ForegroundColor Red }

    Write-Host "  [$crate] cargo test --locked" -ForegroundColor Yellow
    cargo test --locked 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { $pass = $false; Write-Host "  FAIL test" -ForegroundColor Red }
}

Set-Location $root
if ($pass) {
    Write-Host "RESTORE DRILL PASSED" -ForegroundColor Green
    Write-Host "Bundle: $bundlePath"
    Write-Host "Record this in docs/operations/backup-log.md"
} else {
    Write-Host "RESTORE DRILL FAILED" -ForegroundColor Red
    exit 1
}