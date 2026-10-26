#requires -Version 5.1
# Supply-chain gate: advisory database, license/source policy and lockfile audit.
# This is a release/CI check, not part of the fast per-change scripts/check gate,
# because it fetches the RustSec advisory database.

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$failed = @()

function Require-Tool {
    param([string]$Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        Write-Host "FAIL: required tool '$Name' is not installed." -ForegroundColor Red
        Write-Host "Install it with: cargo install $Name --locked" -ForegroundColor Yellow
        exit 2
    }
}

Require-Tool "cargo-deny"
Require-Tool "cargo-audit"

foreach ($crate in @("platform", "eval")) {
    Write-Host "=== ${crate}: cargo deny ===" -ForegroundColor Cyan
    Push-Location (Join-Path $root $crate)
    try {
        cargo deny --config ../deny.toml check
        if ($LASTEXITCODE -ne 0) { $failed += "$crate cargo-deny" }
    }
    finally { Pop-Location }

    Write-Host "=== ${crate}: cargo audit ===" -ForegroundColor Cyan
    Push-Location (Join-Path $root $crate)
    try {
        cargo audit
        if ($LASTEXITCODE -ne 0) { $failed += "$crate cargo-audit" }
    }
    finally { Pop-Location }
}

if ($failed.Count -gt 0) {
    Write-Host ("SUPPLY CHAIN CHECK FAILED: " + ($failed -join ", ")) -ForegroundColor Red
    exit 1
}
Write-Host "SUPPLY CHAIN CHECK PASSED" -ForegroundColor Green
