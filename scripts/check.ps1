# Local Check Script (PowerShell)
# Runs the full quality-gates baseline for both independent crates.
# Executed from the repo root. Uses `;` between commands (no `&&`/`||`).

$root = Resolve-Path "$PSScriptRoot\.."

function Check-Result($label) {
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAIL: $label" -ForegroundColor Red
        exit 1
    }
    Write-Host "PASS: $label" -ForegroundColor Green
}

Write-Host "=== Documentation ===" -ForegroundColor Cyan

Write-Host "scripts/check-docs.ps1" -ForegroundColor Yellow
& powershell -NoProfile -ExecutionPolicy Bypass -File "$root\scripts\check-docs.ps1"
Check-Result "documentation"

Write-Host "=== Secret Patterns ===" -ForegroundColor Cyan

Write-Host "scripts/check-secrets.ps1" -ForegroundColor Yellow
& powershell -NoProfile -ExecutionPolicy Bypass -File "$root\scripts\check-secrets.ps1"
Check-Result "secret patterns"

Write-Host "=== Eval Crate ===" -ForegroundColor Cyan
Set-Location "$root\eval"

Write-Host "cargo fmt -- --check" -ForegroundColor Yellow
cargo fmt -- --check
Check-Result "eval fmt"

Write-Host "cargo clippy --locked --all-targets -- -D warnings" -ForegroundColor Yellow
cargo clippy --locked --all-targets -- -D warnings
Check-Result "eval clippy"

Write-Host "cargo test --locked" -ForegroundColor Yellow
cargo test --locked
Check-Result "eval test"

Write-Host "cargo run --locked -- --selftest" -ForegroundColor Yellow
cargo run --locked -- --selftest
Check-Result "eval selftest"

Write-Host "=== Platform Crate ===" -ForegroundColor Cyan
Set-Location "$root\platform"

Write-Host "cargo fmt -- --check" -ForegroundColor Yellow
cargo fmt -- --check
Check-Result "platform fmt"

Write-Host "cargo clippy --locked --all-targets -- -D warnings" -ForegroundColor Yellow
cargo clippy --locked --all-targets -- -D warnings
Check-Result "platform clippy"

Write-Host "cargo test --locked" -ForegroundColor Yellow
cargo test --locked
Check-Result "platform test"

Write-Host "cargo run --locked --bin ekp_platform -- --selftest" -ForegroundColor Yellow
cargo run --locked --bin ekp_platform -- --selftest
Check-Result "platform selftest"

Write-Host "cargo run --locked --bin ekp_platform -- --declared-rules-schema" -ForegroundColor Yellow
cargo run --locked --bin ekp_platform -- --declared-rules-schema
Check-Result "platform declared-rules-schema"

Write-Host "=== ALL CHECKS PASSED ===" -ForegroundColor Green
