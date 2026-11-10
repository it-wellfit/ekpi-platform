#!/bin/bash
# Local Check Script (POSIX shell)
# Runs the full quality-gates baseline for the root Cargo workspace.
# Executed from the repo root.

set -e

root="$(cd "$(dirname "$0")/.." && pwd)"

check() {
    local label="$1"
    echo "PASS: $label"
}

echo "=== Documentation ==="
"$root/scripts/check-docs.sh"
check "documentation"

echo "=== Secret Patterns ==="
"$root/scripts/check-secrets.sh"
check "secret patterns"

echo "=== Workspace ==="
cd "$root"

echo "cargo fmt --all -- --check"
cargo fmt --all -- --check
check "workspace fmt"

echo "cargo clippy --locked --workspace --all-targets -- -D warnings"
cargo clippy --locked --workspace --all-targets -- -D warnings
check "workspace clippy"

echo "cargo test --locked --workspace"
cargo test --locked --workspace
check "workspace test"

echo "=== Eval Crate ==="
cd "$root/eval"

echo "cargo run --locked -- --selftest"
cargo run --locked -- --selftest
check "eval selftest"

echo "=== Platform Crate ==="
cd "$root/platform"

echo "cargo run --locked --bin ekp_platform -- --selftest"
cargo run --locked --bin ekp_platform -- --selftest
check "platform selftest"

echo "cargo run --locked --bin ekp_platform -- --declared-rules-schema"
cargo run --locked --bin ekp_platform -- --declared-rules-schema
check "platform declared-rules-schema"

echo "cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json"
cargo run --locked --bin ekp_platform -- --eval --test-set data/eval/search_eval.json
check "platform search evaluation"

echo "=== ALL CHECKS PASSED ==="
