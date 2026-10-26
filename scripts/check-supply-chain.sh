#!/usr/bin/env sh
# Supply-chain gate: advisory database, license/source policy and lockfile audit.
# This is a release/CI check, not part of the fast per-change scripts/check gate,
# because it fetches the RustSec advisory database.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

for tool in cargo-deny cargo-audit; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "FAIL: required tool '$tool' is not installed." >&2
        echo "Install it with: cargo install $tool --locked" >&2
        exit 2
    fi
done

failed=""
for crate in platform eval; do
    echo "=== $crate: cargo deny ==="
    if ! (cd "$root/$crate" && cargo deny --config ../deny.toml check); then
        failed="$failed $crate:cargo-deny"
    fi

    echo "=== $crate: cargo audit ==="
    if ! (cd "$root/$crate" && cargo audit); then
        failed="$failed $crate:cargo-audit"
    fi
done

if [ -n "$failed" ]; then
    echo "SUPPLY CHAIN CHECK FAILED:$failed" >&2
    exit 1
fi
echo "SUPPLY CHAIN CHECK PASSED"
