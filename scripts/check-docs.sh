#!/usr/bin/env bash
# Documentation check: required documents exist and relative Markdown links resolve.
# It does not prove that a document is true; command/review evidence does that.

set -u

root="$(cd "$(dirname "$0")/.." && pwd)"
failed=0

required=(
    "README.md"
    "AGENTS.md"
    "GLOSSARY.md"
    "docs/README.md"
    "docs/product/README.md"
    "docs/product/mvp-scope.md"
    "docs/architecture/README.md"
    "docs/architecture/threat-model.md"
    "docs/backend/README.md"
    "docs/frontend/README.md"
    "docs/contracts/README.md"
    "docs/contracts/cli.md"
    "docs/contracts/web-routes.md"
    "docs/contracts/data-formats.md"
    "docs/quality/README.md"
    "docs/quality/phase-gates.md"
    "docs/quality/ai-evaluation.md"
    "docs/operations/README.md"
    "docs/operations/production-readiness.md"
    "docs/operations/incident-log.md"
    "docs/operations/local-backup.md"
    "docs/operations/backup-log.md"
    "docs/decisions/README.md"
    "docs/decisions/adr-template.md"
    "docs/reviews/README.md"
    "docs/reviews/retro-template.md"
    "docs/reviews/bootstrap-retro.md"
    "docs/traceability/README.md"
    "docs/traceability/register.md"
    "docs/agents/community-skills-assessment.md"
    "docs/development/README.md"
    "docs/development/quality-gates.md"
    "docs/governance/README.md"
    "docs/governance/08-decision-register.md"
    "docs/metrics/targets.md"
    ".github/ISSUE_TEMPLATE/feature.yml"
    ".github/ISSUE_TEMPLATE/ticket.yml"
    ".github/ISSUE_TEMPLATE/improvement.yml"
    "scripts/check.ps1"
    "scripts/check.sh"
    "scripts/check-docs.ps1"
    "scripts/check-docs.sh"
    "scripts/check-secrets.ps1"
    "scripts/check-secrets.sh"
)

for relative in "${required[@]}"; do
    if [ ! -f "$root/$relative" ]; then
        echo "MISSING: $relative"
        failed=1
    fi
done

if [ "$failed" -eq 0 ]; then
    echo "PASS: required documents present (${#required[@]})"
fi

broken_file="$(mktemp "${TMPDIR:-/tmp}/ekpi-docs-broken.XXXXXX")"
links_file="$(mktemp "${TMPDIR:-/tmp}/ekpi-docs-links.XXXXXX")"
trap 'rm -f "$broken_file" "$links_file"' EXIT
checked_files=0
checked_links=0

while IFS= read -r -d '' file; do
    relative="${file#"$root"/}"
    case "$relative" in
        .git/*|.opencode/*|node_modules/*|*/target/*|target/*|backups/*|EKPI-backup/*) continue ;;
    esac
    checked_files=$((checked_files + 1))
    dir="$(dirname "$file")"

    # Strip fenced code blocks and inline code, then emit every ](...) target.
    # One awk process per file keeps this script linear and portable.
    if ! awk '
        /^[[:space:]]*(```|~~~)/ { fence = !fence; next }
        fence { next }
        {
            line = $0
            gsub(/`[^`]*`/, "", line)
            while ((start = index(line, "](")) > 0) {
                rest = substr(line, start + 2)
                endpos = index(rest, ")")
                if (endpos == 0) break
                print substr(rest, 1, endpos - 1)
                line = substr(rest, endpos + 1)
            }
        }
    ' "$file" > "$links_file"; then
        echo "FAIL: documentation check could not parse $relative"
        exit 1
    fi

    while IFS= read -r target; do
        target="${target#"${target%%[![:space:]]*}"}"
        target="${target%"${target##*[![:space:]]}"}"
        [ -z "$target" ] && continue
        case "$target" in
            \#*) continue ;;
        esac
        if [[ "$target" =~ ^[a-zA-Z][a-zA-Z0-9+.-]*: ]]; then
            continue
        fi
        target="${target#<}"
        target="${target%>}"

        checked_links=$((checked_links + 1))
        path="${target%%#*}"
        path="${path%%\?*}"
        [ -z "$path" ] && continue
        path="${path//%20/ }"

        if [ "${path#/}" != "$path" ]; then
            resolved="$root/${path#/}"
        else
            resolved="$dir/$path"
        fi

        if [ ! -e "$resolved" ]; then
            echo "$relative -> $target" >> "$broken_file"
        fi
    done < "$links_file"
done < <(find "$root" -type f -name '*.md' -print0)

if [ -s "$broken_file" ]; then
    echo "Broken relative links:"
    sort -u "$broken_file" | while IFS= read -r entry; do
        echo "  $entry"
    done
    echo "FAIL: documentation check ($(sort -u "$broken_file" | wc -l | tr -d ' ') broken link entries)"
    exit 1
fi

echo "PASS: relative Markdown links resolve ($checked_links links in $checked_files files)"
echo "Documentation check PASSED"
exit 0
