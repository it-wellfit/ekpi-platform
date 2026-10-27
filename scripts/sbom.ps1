#requires -Version 5.1
# Generate CycloneDX 1.5 SBOMs for both crates at the current revision.
# The generated documents are sanitized to remove the absolute local path and
# are written with a deterministic timestamp derived from the Git commit.

param(
    [string]$OutDir = ""
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutDir)) {
    $OutDir = Join-Path $root "artifacts\sbom"
}

if (-not (Get-Command "cargo-cyclonedx" -ErrorAction SilentlyContinue)) {
    Write-Host "FAIL: cargo-cyclonedx is not installed." -ForegroundColor Red
    Write-Host "Install it with: cargo install cargo-cyclonedx --locked" -ForegroundColor Yellow
    exit 2
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

function Get-Sha256Hex {
    param([string]$Path)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $stream = [System.IO.File]::OpenRead($Path)
        try {
            return ([System.BitConverter]::ToString($sha.ComputeHash($stream))).Replace('-', '')
        }
        finally { $stream.Dispose() }
    }
    finally { $sha.Dispose() }
}

$epoch = (git -C $root log -1 --format=%ct 2>$null)
if ([string]::IsNullOrWhiteSpace($epoch)) {
    Write-Host "FAIL: cannot read the Git commit timestamp." -ForegroundColor Red
    exit 1
}
$env:SOURCE_DATE_EPOCH = $epoch.Trim()

$rootUri = ([System.Uri]($root + [IO.Path]::DirectorySeparatorChar)).AbsoluteUri.TrimEnd("/")
$rootForward = $root.Replace("\", "/")
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
$sums = @()

foreach ($crate in @("platform", "eval")) {
    $manifest = Join-Path $root "$crate\Cargo.toml"
    $package = (Select-String -Path $manifest -Pattern '^name\s*=\s*"(.+)"' | Select-Object -First 1).Matches.Groups[1].Value
    $version = (Select-String -Path $manifest -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1).Matches.Groups[1].Value
    $baseName = "$package-$version"
    $generated = Join-Path $root "$crate\$baseName.cdx.json"
    $target = Join-Path $OutDir "$baseName.cdx.json"

    Write-Host "=== ${crate}: CycloneDX $baseName ===" -ForegroundColor Cyan
    Push-Location (Join-Path $root $crate)
    try {
        cargo cyclonedx --format json --spec-version 1.5 --override-filename "$baseName.cdx"
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAIL: cargo cyclonedx failed for $crate" -ForegroundColor Red
            exit 1
        }
    }
    finally { Pop-Location }

    if (-not (Test-Path -LiteralPath $generated)) {
        Write-Host "FAIL: expected SBOM was not created: $generated" -ForegroundColor Red
        exit 1
    }

    $text = [System.IO.File]::ReadAllText($generated)
    $text = $text.Replace($rootUri, "repo://ekpi-platform")
    $text = $text.Replace($rootForward, "repo://ekpi-platform")
    $text = $text.Replace($root.Replace("\", "\\"), "repo://ekpi-platform")
    $text = $text.Replace($root, "repo://ekpi-platform")
    # cargo-cyclonedx percent-encodes non-ASCII path segments, so the literal
    # replacements above cannot match every workspace URL. Remove any remaining
    # file:// URL that points at this repository.
    $text = [regex]::Replace($text, 'file://[^"]*?ekpi-platform', 'repo://ekpi-platform')
    $text = [regex]::Replace($text, 'C:\\\\Users\\\\[^"]*?ekpi-platform', 'repo://ekpi-platform')
    [System.IO.File]::WriteAllText($target, $text, $utf8NoBom)
    Remove-Item -LiteralPath $generated -Force

    $bom = [System.IO.File]::ReadAllText($target) | ConvertFrom-Json
    if ($bom.bomFormat -ne "CycloneDX" -or $bom.specVersion -ne "1.5") {
        Write-Host "FAIL: $target is not a CycloneDX 1.5 document" -ForegroundColor Red
        exit 1
    }
    if ($bom.components.Count -lt 1) {
        Write-Host "FAIL: $target has no components" -ForegroundColor Red
        exit 1
    }

    $hash = (Get-Sha256Hex -Path $target).ToLowerInvariant()
    $sums += "$hash  $baseName.cdx.json"
    Write-Host "PASS: $target ($($bom.components.Count) components, $hash)" -ForegroundColor Green
}

$sumFile = Join-Path $OutDir "SHA256SUMS"
[System.IO.File]::WriteAllText($sumFile, (($sums -join "`n") + "`n"), $utf8NoBom)
Write-Host "SBOM PASSED: $OutDir" -ForegroundColor Green
