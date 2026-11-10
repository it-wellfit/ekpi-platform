# Local Check Script (PowerShell)
# Runs the full quality-gates baseline for the root Cargo workspace.
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

Write-Host "=== Workspace ===" -ForegroundColor Cyan
Set-Location $root

Write-Host "cargo fmt --all -- --check" -ForegroundColor Yellow
cargo fmt --all -- --check
Check-Result "workspace fmt"

Write-Host "cargo clippy --locked --workspace --all-targets -- -D warnings" -ForegroundColor Yellow
cargo clippy --locked --workspace --all-targets -- -D warnings
Check-Result "workspace clippy"

Write-Host "cargo test --locked --workspace" -ForegroundColor Yellow
cargo test --locked --workspace
Check-Result "workspace test"

Write-Host "=== Eval Crate ===" -ForegroundColor Cyan
Set-Location "$root\eval"

Write-Host "cargo run --locked -- --selftest" -ForegroundColor Yellow
cargo run --locked -- --selftest
Check-Result "eval selftest"

Write-Host "=== Platform Crate ===" -ForegroundColor Cyan
Set-Location "$root\platform"

Write-Host "cargo run --locked --bin ekp_platform -- --selftest" -ForegroundColor Yellow
cargo run --locked --bin ekp_platform -- --selftest
Check-Result "platform selftest"

Write-Host "cargo run --locked --bin ekp_platform -- --declared-rules-schema" -ForegroundColor Yellow
cargo run --locked --bin ekp_platform -- --declared-rules-schema
Check-Result "platform declared-rules-schema"

Write-Host "cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json" -ForegroundColor Yellow
cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
Check-Result "platform search evaluation"

Write-Host "=== ALL CHECKS PASSED ===" -ForegroundColor Green
