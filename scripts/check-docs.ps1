#requires -Version 5.1
# Documentation check: required documents exist and relative Markdown links resolve.
# It does not prove that a document is true; command/review evidence does that.

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot\..").Path
$failed = 0

$required = @(
    'README.md',
    'AGENTS.md',
    'GLOSSARY.md',
    'docs/README.md',
    'docs/product/README.md',
    'docs/product/mvp-scope.md',
    'docs/architecture/README.md',
    'docs/architecture/threat-model.md',
    'docs/backend/README.md',
    'docs/frontend/README.md',
    'docs/contracts/README.md',
    'docs/contracts/cli.md',
    'docs/contracts/web-routes.md',
    'docs/contracts/data-formats.md',
    'docs/quality/README.md',
    'docs/quality/phase-gates.md',
    'docs/quality/ai-evaluation.md',
    'docs/operations/README.md',
    'docs/operations/production-readiness.md',
    'docs/operations/incident-log.md',
    'docs/operations/local-backup.md',
    'docs/operations/backup-log.md',
    'docs/decisions/README.md',
    'docs/decisions/adr-template.md',
    'docs/reviews/README.md',
    'docs/reviews/retro-template.md',
    'docs/reviews/bootstrap-retro.md',
    'docs/traceability/README.md',
    'docs/traceability/register.md',
    'docs/assurance/README.md',
    'docs/assurance/phase-a-b-matrix.md',
    'docs/assurance/root-cause-analysis.md',
    'docs/assurance/source-matrix.md',
    'docs/assurance/test-evidence.md',
    'docs/assurance/external-blockers.md',
    'docs/assurance/security-threat-model.md',
    'docs/assurance/operations-and-recovery.md',
    'docs/assurance/final-report.md',
    'docs/agents/community-skills-assessment.md',
    'docs/development/README.md',
    'docs/development/quality-gates.md',
    'docs/governance/README.md',
    'docs/governance/08-decision-register.md',
    'docs/metrics/targets.md',
    '.github/ISSUE_TEMPLATE/feature.yml',
    '.github/ISSUE_TEMPLATE/ticket.yml',
    '.github/ISSUE_TEMPLATE/improvement.yml',
    'scripts/check.ps1',
    'scripts/check.sh',
    'scripts/check-docs.ps1',
    'scripts/check-docs.sh',
    'scripts/check-secrets.ps1',
    'scripts/check-secrets.sh'
)

foreach ($relative in $required) {
    $path = Join-Path $root $relative
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        Write-Host "MISSING: $relative" -ForegroundColor Red
        $failed++
    }
}
if ($failed -eq 0) {
    Write-Host "PASS: required documents present ($($required.Count))" -ForegroundColor Green
}

$excluded = '(^|\\)(\.git|\.opencode|node_modules|target|backups|EKPI-backup)(\\|$)'
$files = Get-ChildItem -LiteralPath $root -Recurse -File -Filter *.md |
    Where-Object {
        $relative = $_.FullName.Substring($root.Length).TrimStart('\', '/')
        $relative -notmatch $excluded
    }

$checkedFiles = 0
$checkedLinks = 0
$broken = New-Object System.Collections.Generic.List[string]

foreach ($file in $files) {
    $checkedFiles++
    $inFence = $false
    $lines = Get-Content -LiteralPath $file.FullName -Encoding UTF8
    foreach ($line in $lines) {
        if ($line -match '^\s*(```|~~~)') {
            $inFence = -not $inFence
            continue
        }
        if ($inFence) { continue }

        # Remove inline code so examples are not treated as links.
        $probe = $line -replace '`[^`]*`', ''
        foreach ($match in [regex]::Matches($probe, '!?\[[^\]]*\]\(([^)]+)\)')) {
            $target = $match.Groups[1].Value.Trim()
            $target = $target.Trim('<', '>')
            if ([string]::IsNullOrWhiteSpace($target)) { continue }
            if ($target.StartsWith('#')) { continue }
            if ($target -match '^[a-zA-Z][a-zA-Z0-9+.-]*:') { continue }

            $checkedLinks++
            $pathPart = $target.Split('#')[0].Split('?')[0]
            if ([string]::IsNullOrWhiteSpace($pathPart)) { continue }
            $pathPart = [System.Uri]::UnescapeDataString($pathPart)

            if ($pathPart.StartsWith('/')) {
                $resolved = Join-Path $root $pathPart.TrimStart('/')
            } else {
                $resolved = Join-Path $file.DirectoryName $pathPart
            }

            if (-not (Test-Path -LiteralPath $resolved)) {
                $relative = $file.FullName.Substring($root.Length).TrimStart('\', '/')
                $broken.Add("$relative -> $target")
            }
        }
    }
}

if ($broken.Count -gt 0) {
    Write-Host "Broken relative links:" -ForegroundColor Red
    $broken | Sort-Object -Unique | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    Write-Host "FAIL: documentation check ($($broken.Count) broken link entries)" -ForegroundColor Red
    exit 1
}

if ($failed -gt 0) {
    Write-Host "FAIL: required documents missing ($failed)" -ForegroundColor Red
    exit 1
}

Write-Host "PASS: relative Markdown links resolve ($checkedLinks links in $checkedFiles files)" -ForegroundColor Green
Write-Host "Documentation check PASSED" -ForegroundColor Green
exit 0
