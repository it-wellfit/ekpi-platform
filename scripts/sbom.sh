#!/usr/bin/env sh
# Generate CycloneDX 1.5 SBOMs for both crates at the current revision.
# The generated documents are sanitized to remove the absolute local path and
# are written with a deterministic timestamp derived from the Git commit.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
out_dir=${1:-"$root/artifacts/sbom"}

if ! command -v cargo-cyclonedx >/dev/null 2>&1; then
    echo "FAIL: cargo-cyclonedx is not installed." >&2
    echo "Install it with: cargo install cargo-cyclonedx --locked" >&2
    exit 2
fi

mkdir -p "$out_dir"
epoch=$(git -C "$root" log -1 --format=%ct)
SOURCE_DATE_EPOCH=$epoch
export SOURCE_DATE_EPOCH

root_uri=$(cd "$root" && pwd -P)
sums=""

for crate in platform eval; do
    manifest="$root/$crate/Cargo.toml"
    package=$(sed -n 's/^name[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' "$manifest" | head -n 1)
    version=$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' "$manifest" | head -n 1)
    base_name="$package-$version"
    generated="$root/$crate/$base_name.cdx.json"
    target="$out_dir/$base_name.cdx.json"

    echo "=== $crate: CycloneDX $base_name ==="
    (cd "$root/$crate" && cargo cyclonedx --format json --spec-version 1.5 --override-filename "$base_name.cdx")
    if [ ! -f "$generated" ]; then
        echo "FAIL: expected SBOM was not created: $generated" >&2
        exit 1
    fi

    # The BOM embeds a file:// URI for the workspace; replace it with a stable
    # repository URI so the committed artifact contains no local user path.
    # The generic expression also covers percent-encoded non-ASCII segments.
    sed -e "s|$root_uri|repo://ekpi-platform|g" \
        -e 's|file://[^"]*ekpi-platform|repo://ekpi-platform|g' \
        "$generated" > "$target"
    rm -f "$generated"

    grep -q '"bomFormat"[[:space:]]*:[[:space:]]*"CycloneDX"' "$target" || {
        echo "FAIL: $target is not a CycloneDX document" >&2
        exit 1
    }
    grep -q '"specVersion"[[:space:]]*:[[:space:]]*"1.5"' "$target" || {
        echo "FAIL: $target is not CycloneDX 1.5" >&2
        exit 1
    }
    components=$(grep -c '"type"[[:space:]]*:[[:space:]]*"library"' "$target" || true)
    if [ "$components" -lt 1 ]; then
        echo "FAIL: $target has no components" >&2
        exit 1
    fi

    hash=$(sha256sum "$target" | awk '{print $1}')
    sums="$sums$hash  $base_name.cdx.json\n"
    echo "PASS: $target ($components components, $hash)"
done

printf "%b" "$sums" > "$out_dir/SHA256SUMS"
echo "SBOM PASSED: $out_dir"
