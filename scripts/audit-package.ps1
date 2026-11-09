# S14: audit package builder (CTR-EXP-005).
# Produces an auditable, self-describing package:
#   ekpi-platform.bundle          git bundle (full history)
#   governance-ledger.jsonl       runtime governance chain copy
#   approvals.jsonl               runtime approval chain copy
#   evidence_pack.json            chain digests (CTR-EXP-003)
#   data_map.json                 lineage (CTR-DATA-008)
#   manifest.json                 ekpi-audit-manifest-v1 (file list + SHA-256,
#                                 head commit; content deterministic, no timestamps)
#   restore-drill.ps1             the operational drill, packaged with the evidence
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts/audit-package.ps1 [-OutputDir path]
# Exit:  0 = package written and manifest hashes verified, non-zero = fail.

param(
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
Set-Location $repo
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
if (-not $OutputDir) {
    $OutputDir = Join-Path $repo (Join-Path "artifacts" (Join-Path "audit-package" $stamp))
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

function Fail([string]$step, [string]$reason) {
    Write-Output "PACKAGE FAIL: step '$step' - $reason"
    exit 6
}

# ── 1. git bundle (full history) ────────────────────────────────────────
# PS 5.1 turns git's stderr notices ("bundle is okay") into error records;
# under Stop that aborts the script. Downgrade locally and rely on the
# explicit $LASTEXITCODE checks (same pattern as backup.ps1, S24).
$bundle = Join-Path $OutputDir "ekpi-platform.bundle"
$ErrorActionPreference = "Continue"
git bundle create $bundle --all 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Fail "bundle" "git bundle create failed" }
git bundle verify $bundle 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Fail "bundle-verify" "git bundle verify failed" }
$ErrorActionPreference = "Stop"
$headCommit = (git rev-parse HEAD).Trim()
Write-Output "PACKAGE: bundle verified (head $headCommit)"

# ── 2. runtime chain copies (env or documented defaults) ────────────────
$gov = if ($env:EKP_GOVERNANCE_DATA) { $env:EKP_GOVERNANCE_DATA } else { Join-Path $env:LOCALAPPDATA (Join-Path "EKPI" "governance-ledger.jsonl") }
$appr = if ($env:EKP_APPROVAL_DATA) { $env:EKP_APPROVAL_DATA } else { Join-Path $env:LOCALAPPDATA (Join-Path "EKPI" "approvals.jsonl") }
foreach ($pair in @(@("governance-ledger.jsonl", $gov), @("approvals.jsonl", $appr))) {
    $name = $pair[0]
    $source = $pair[1]
    if (Test-Path $source) {
        Copy-Item $source (Join-Path $OutputDir $name) -Force
        Write-Output "PACKAGE: copied $name from $source"
    } else {
        Write-Output "PACKAGE: $name not present yet (empty chain) - skipped copy"
    }
}

# ── 3. evidence pack (chain digests via the CLI, explicit paths) ────────
$cli = Join-Path $repo (Join-Path "target" (Join-Path "debug" "ekp_platform.exe"))
if (-not (Test-Path $cli)) { Fail "cli" "ekp_platform.exe not built" }
& $cli --export-evidence-pack $OutputDir --governance-data $gov --approval-data $appr | Out-Null
if ($LASTEXITCODE -ne 0) { Fail "evidence-pack" "evidence pack export failed (chain refused or tampered - see stderr)" }
Write-Output "PACKAGE: evidence pack exported"

# ── 4. data map (lineage, deterministic) ────────────────────────────────
& $cli --data-map $OutputDir | Out-Null
if ($LASTEXITCODE -ne 0) { Fail "data-map" "data map export failed" }

# ── 5. manifest (ekpi-audit-manifest-v1; deterministic given content) ───
$manifestFiles = @()
foreach ($file in (Get-ChildItem $OutputDir -File | Sort-Object Name)) {
    if ($file.Name -eq "manifest.json") { continue }
    $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $file.FullName).Hash
    $manifestFiles += @{ name = $file.Name; sha256 = $hash; bytes = $file.Length }
}
$manifest = [ordered]@{
    schema = "ekpi-audit-manifest-v1"
    head_commit = $headCommit
    files = $manifestFiles
}
$manifestPath = Join-Path $OutputDir "manifest.json"
[System.IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 4), [System.Text.UTF8Encoding]::new($false))

# ── 6. self-verify: every manifest entry's hash must match on disk ──────
foreach ($entry in $manifest.files) {
    $target = Join-Path $OutputDir $entry.name
    if (-not (Test-Path $target)) { Fail "self-verify" "manifest entry missing: $($entry.name)" }
    $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $target).Hash
    if ($actual -ne $entry.sha256) { Fail "self-verify" "hash mismatch: $($entry.name)" }
}
Write-Output "PACKAGE: manifest self-verified ($($manifest.files.Count) files)"

# ── 7. package the operational drill with the evidence ──────────────────
Copy-Item (Join-Path $PSScriptRoot "drill-runtime-data.ps1") (Join-Path $OutputDir "restore-drill.ps1") -Force

Write-Output "PACKAGE PASS: audit package at $OutputDir"
Write-Output "PACKAGE: manifest $manifestPath"
exit 0
