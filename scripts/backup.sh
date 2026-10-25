#!/bin/bash
# Local Backup Script (POSIX shell)
set -e

root="$(cd "$(dirname "$0")/.." && pwd)"
bundle_dir="$HOME/EKPI-backup/bundles"
mkdir -p "$bundle_dir"

name="ekpi-platform-$(date +%Y-%m-%d-%H%M%S).bundle"
bundle_path="$bundle_dir/$name"

echo "Creating bundle: $bundle_path"
cd "$root"
git bundle create "$bundle_path" --all

drill_dir="/tmp/ekpi-restore-drill"
rm -rf "$drill_dir"
echo "Restore drill: cloning bundle..."
git clone "$bundle_path" "$drill_dir" >/dev/null 2>&1

echo "Restore drill: running the restored repository check..."
if [ ! -x "$drill_dir/scripts/check.sh" ]; then
    chmod +x "$drill_dir/scripts/check.sh"
fi
if "$drill_dir/scripts/check.sh"; then
    pass=true
else
    pass=false
fi

cd "$root"
if [ "$pass" = true ]; then
    echo "RESTORE DRILL PASSED"
    echo "Bundle: $bundle_path"
    echo "Record this in docs/operations/backup-log.md"
else
    echo "RESTORE DRILL FAILED"
    exit 1
fi