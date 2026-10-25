# Local Backup Script (PowerShell)
# Creates a git bundle and runs a restore drill. Prints the result; the operator
# records the bundle path and outcome in docs/operations/backup-log.md.

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
$sha256 = [System.Security.Cryptography.SHA256]::Create()
$stream = [System.IO.File]::OpenRead($bundlePath)
try {
    $bundleHash = [System.BitConverter]::ToString($sha256.ComputeHash($stream)).Replace('-', '')
} finally {
    $stream.Dispose()
    $sha256.Dispose()
}
Write-Host "Bundle created." -ForegroundColor Green
Write-Host "SHA-256: $bundleHash"

# Restore drill
$drillDir = "$env:TEMP\ekpi-restore-drill"
if (Test-Path $drillDir) { Remove-Item -Recurse -Force $drillDir }
Write-Host "Restore drill: cloning bundle..." -ForegroundColor Cyan
git clone "$bundlePath" "$drillDir"
if ($LASTEXITCODE -ne 0) { Write-Host "FAIL: clone" -ForegroundColor Red; exit 1 }

Write-Host "Restore drill: running the restored repository check..." -ForegroundColor Cyan
$checkScript = Join-Path $drillDir "scripts\check.ps1"
if (-not (Test-Path -LiteralPath $checkScript)) {
    Write-Host "FAIL: restored bundle has no scripts/check.ps1" -ForegroundColor Red
    exit 1
}
& powershell -NoProfile -ExecutionPolicy Bypass -File $checkScript
$pass = ($LASTEXITCODE -eq 0)

Set-Location $root
if ($pass) {
    Write-Host "RESTORE DRILL PASSED" -ForegroundColor Green
    Write-Host "Bundle: $bundlePath"
    Write-Host "Record this in docs/operations/backup-log.md"
} else {
    Write-Host "RESTORE DRILL FAILED" -ForegroundColor Red
    exit 1
}