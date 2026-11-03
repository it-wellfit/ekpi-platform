#!/bin/bash
# Local Check Script (POSIX shell)
# Runs the full quality-gates baseline for both independent crates.
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

echo "=== Eval Crate ==="
cd "$root/eval"

echo "cargo fmt -- --check"
cargo fmt -- --check
check "eval fmt"

echo "cargo clippy --locked --all-targets -- -D warnings"
cargo clippy --locked --all-targets -- -D warnings
check "eval clippy"

echo "cargo test --locked"
cargo test --locked
check "eval test"

echo "cargo run --locked -- --selftest"
cargo run --locked -- --selftest
check "eval selftest"

echo "cargo run --locked -- --corpus-selftest"
cargo run --locked -- --corpus-selftest
check "eval corpus selftest"

echo "=== Platform Crate ==="
cd "$root/platform"

echo "cargo fmt -- --check"
cargo fmt -- --check
check "platform fmt"

echo "cargo clippy --locked --all-targets -- -D warnings"
cargo clippy --locked --all-targets -- -D warnings
check "platform clippy"

echo "cargo test --locked"
cargo test --locked
check "platform test"

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
