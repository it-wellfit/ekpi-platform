#!/usr/bin/env bash
# High-signal secret pattern scan for tracked and untracked, non-ignored files.
# This is not a full secret scanner and does not replace a reviewed secret-management process.
# It prints file and line locations only; it never prints the matching value.

set -u

root="$(cd "$(dirname "$0")/.." && pwd)"

patterns=(
    "AWS access key|(AKIA|ASIA)[0-9A-Z]{16}"
    "Private key block|-----BEGIN ([A-Z]+ )?PRIVATE KEY-----"
    "GitHub token|gh[pousr]_[A-Za-z0-9]{30,}"
    "Slack token|xox[baprs]-[A-Za-z0-9-]{10,}"
    "OpenAI-style key|sk-[A-Za-z0-9]{32,}"
    "Google API key|AIza[0-9A-Za-z_-]{35}"
    "Database URL with credentials|(postgres|postgresql|mysql|mongodb(\\+srv)?)://[^[:space:]]+:[^[:space:]]+@"
    "JWT-like token|eyJ[A-Za-z0-9_-]{20,}\\.[A-Za-z0-9_-]{20,}\\.[A-Za-z0-9_-]{10,}"
)

if ! git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
    echo "FAIL: not a Git repository: $root"
    exit 1
fi

files_file="$(mktemp "${TMPDIR:-/tmp}/ekpi-secret-files.XXXXXX")"
trap 'rm -f "$files_file"' EXIT
{
    git -C "$root" ls-files
    git -C "$root" ls-files --others --exclude-standard
} | sort -u > "$files_file"

findings=0
scanned=0

while IFS= read -r relative; do
    [ -z "$relative" ] && continue
    case "$relative" in
        */target/*|target/*|.opencode/*|node_modules/*|backups/*|EKPI-backup/*) continue ;;
        *.png|*.jpg|*.jpeg|*.gif|*.pdf|*.zip|*.bundle|*.rlib|*.exe|*.dll|*.so|*.dylib|*.ico|*.woff|*.woff2) continue ;;
    esac
    file="$root/$relative"
    [ -f "$file" ] || continue
    scanned=$((scanned + 1))

    for entry in "${patterns[@]}"; do
        name="${entry%%|*}"
        regex="${entry#*|}"
        matches="$(grep -nEI -- "$regex" "$file" 2>/dev/null || true)"
        if [ -n "$matches" ]; then
            while IFS= read -r match; do
                line="${match%%:*}"
                echo "  $relative:$line [$name]"
                findings=$((findings + 1))
            done <<< "$matches"
        fi
    done
done < "$files_file"

if [ "$findings" -gt 0 ]; then
    echo "FAIL: secret pattern scan ($findings finding(s)); values are not printed"
    exit 1
fi

echo "PASS: no high-signal secret patterns in $scanned file(s)"
exit 0
