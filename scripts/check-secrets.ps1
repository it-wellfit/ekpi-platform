#requires -Version 5.1
# High-signal secret pattern scan for tracked and untracked, non-ignored files.
# This is not a full secret scanner and does not replace a reviewed secret-management process.
# It prints file and line locations only; it never prints the matching value.

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path "$PSScriptRoot\..").Path

$patterns = @(
    @{ Name = 'AWS access key'; Regex = '\b(AKIA|ASIA)[0-9A-Z]{16}\b' },
    @{ Name = 'Private key block'; Regex = '-----BEGIN ([A-Z]+ )?PRIVATE KEY-----' },
    @{ Name = 'GitHub token'; Regex = '\bgh[pousr]_[A-Za-z0-9]{30,}\b' },
    @{ Name = 'Slack token'; Regex = '\bxox[baprs]-[A-Za-z0-9-]{10,}\b' },
    @{ Name = 'OpenAI-style key'; Regex = '\bsk-[A-Za-z0-9]{32,}\b' },
    @{ Name = 'Google API key'; Regex = '\bAIza[0-9A-Za-z_-]{35}\b' },
    @{ Name = 'Database URL with credentials'; Regex = '(postgres|postgresql|mysql|mongodb(\+srv)?)://[^\s]+:[^\s]+@' },
    @{ Name = 'JWT-like token'; Regex = '\beyJ[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{10,}\b' }
)

$tracked = @(git -C $root ls-files)
if ($LASTEXITCODE -ne 0) {
    Write-Host "FAIL: cannot list tracked files (is this a Git repository?)" -ForegroundColor Red
    exit 1
}
$untracked = @(git -C $root ls-files --others --exclude-standard)
if ($LASTEXITCODE -ne 0) {
    Write-Host "FAIL: cannot list untracked files" -ForegroundColor Red
    exit 1
}

$files = @($tracked + $untracked | Sort-Object -Unique)
$findings = New-Object System.Collections.Generic.List[string]
$scanned = 0

foreach ($relative in $files) {
    $relative = $relative.Trim()
    if ([string]::IsNullOrWhiteSpace($relative)) { continue }
    if ($relative -match '(^|/)(target|\.opencode|node_modules|backups|EKPI-backup)/') { continue }
    if ($relative -match '\.(png|jpg|jpeg|gif|pdf|zip|bundle|rlib|exe|dll|so|dylib|ico|woff2?)$') { continue }

    $path = Join-Path $root $relative
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
    $scanned++
    $lineNumber = 0
    foreach ($line in Get-Content -LiteralPath $path -Encoding UTF8) {
        $lineNumber++
        foreach ($pattern in $patterns) {
            if ($line -match $pattern.Regex) {
                $findings.Add("$relative`:$lineNumber [$($pattern.Name)]")
            }
        }
    }
}

if ($findings.Count -gt 0) {
    Write-Host "Potential secrets found (values not printed):" -ForegroundColor Red
    $findings | Sort-Object -Unique | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
    Write-Host "FAIL: secret pattern scan ($($findings.Count) finding(s))" -ForegroundColor Red
    exit 1
}

Write-Host "PASS: no high-signal secret patterns in $scanned file(s)" -ForegroundColor Green
exit 0
