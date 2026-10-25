#requires -Version 5.1
[CmdletBinding()]
param(
    [switch]$IncludeSmoke
)

$ErrorActionPreference = 'Stop'

$repoRoot = $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($repoRoot)) {
    $repoRoot = (Get-Location).Path
}

$script:Passed = 0
$script:Failed = 0

function Invoke-CargoStep {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$WorkingDirectory,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    $index = $script:Passed + $script:Failed + 1
    Write-Host ''
    Write-Host ("[{0}] {1}" -f $index, $Name)

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    Push-Location -LiteralPath $WorkingDirectory
    try {
        & cargo @Arguments
        $exitCode = $LASTEXITCODE
    }
    finally {
        Pop-Location
    }
    $timer.Stop()

    if ($exitCode -ne 0) {
        $script:Failed++
        Write-Host ("FAILED: {0} (exit {1}, {2:N1}s)" -f $Name, $exitCode, $timer.Elapsed.TotalSeconds) -ForegroundColor Red
        exit $exitCode
    }

    $script:Passed++
    Write-Host ("PASS: {0} ({1:N1}s)" -f $Name, $timer.Elapsed.TotalSeconds) -ForegroundColor Green
}

function Assert-Artifact {
    param(
        [Parameter(Mandatory = $true)][string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Missing artifact: $Path"
    }
    if ((Get-Item -LiteralPath $Path).Length -le 0) {
        throw "Empty artifact: $Path"
    }
    Write-Host ("  artifact ok: {0}" -f (Split-Path -Path $Path -Leaf))
}

function Assert-LegalRuleML {
    param(
        [Parameter(Mandatory = $true)][string]$Path
    )

    $item = Get-Item -LiteralPath $Path
    if ($item.Length -lt 10000) {
        throw "LegalRuleML artifact is only $($item.Length) bytes; a stub or truncated export cannot pass."
    }

    try {
        [xml]$document = Get-Content -LiteralPath $Path -Raw -Encoding UTF8
    }
    catch {
        throw "LegalRuleML artifact is not well-formed XML: $Path ($($_.Exception.Message))"
    }

    $root = $document.DocumentElement
    if ($null -eq $root -or $root.LocalName -ne 'LegalRuleML') {
        throw "LegalRuleML artifact root must be lrml:LegalRuleML, found '$($root.LocalName)'."
    }
    if ($root.NamespaceURI -ne 'http://docs.oasis-open.org/legalruleml/ns/v1.0/') {
        throw "LegalRuleML artifact uses unexpected namespace '$($root.NamespaceURI)'."
    }

    $prescriptive = $document.SelectNodes("//*[local-name()='PrescriptiveStatement']").Count
    $penalties = $document.SelectNodes("//*[local-name()='PenaltyStatement']").Count
    $reparations = $document.SelectNodes("//*[local-name()='Reparation']").Count
    $sources = $document.SelectNodes("//*[local-name()='LegalSource']").Count

    if ($prescriptive -lt 19) {
        throw "LegalRuleML artifact has $prescriptive PrescriptiveStatement elements; expected at least 19 active rules."
    }
    if ($penalties -lt 19 -or $reparations -lt 19) {
        throw "LegalRuleML artifact is missing Penalty/Reparation statements (penalty=$penalties, reparation=$reparations)."
    }
    if ($sources -lt 16) {
        throw "LegalRuleML artifact has only $sources LegalSource elements; expected the active rule corpus."
    }

    Write-Host ("  LegalRuleML ok: {0:N0} bytes, root {{{1}}}LegalRuleML, {2} PrescriptiveStatement, {3} PenaltyStatement, {4} Reparation, {5} LegalSource" -f $item.Length, $root.NamespaceURI, $prescriptive, $penalties, $reparations, $sources)
}

function Invoke-DocsStep {
    $index = $script:Passed + $script:Failed + 1
    Write-Host ''
    Write-Host ("[{0}] documentation" -f $index)

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $repoRoot 'scripts\check-docs.ps1')
    $exitCode = $LASTEXITCODE
    $timer.Stop()

    if ($exitCode -ne 0) {
        $script:Failed++
        Write-Host ("FAILED: documentation (exit {0}, {1:N1}s)" -f $exitCode, $timer.Elapsed.TotalSeconds) -ForegroundColor Red
        exit $exitCode
    }

    $script:Passed++
    Write-Host ("PASS: documentation ({0:N1}s)" -f $timer.Elapsed.TotalSeconds) -ForegroundColor Green
}

function Invoke-SecretsStep {
    $index = $script:Passed + $script:Failed + 1
    Write-Host ''
    Write-Host ("[{0}] secret patterns" -f $index)

    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $repoRoot 'scripts\check-secrets.ps1')
    $exitCode = $LASTEXITCODE
    $timer.Stop()

    if ($exitCode -ne 0) {
        $script:Failed++
        Write-Host ("FAILED: secret patterns (exit {0}, {1:N1}s)" -f $exitCode, $timer.Elapsed.TotalSeconds) -ForegroundColor Red
        exit $exitCode
    }

    $script:Passed++
    Write-Host ("PASS: secret patterns ({0:N1}s)" -f $timer.Elapsed.TotalSeconds) -ForegroundColor Green
}

$evalDir = Join-Path $repoRoot 'eval'
$platformDir = Join-Path $repoRoot 'platform'

foreach ($crateDir in @($evalDir, $platformDir)) {
    if (-not (Test-Path -LiteralPath (Join-Path $crateDir 'Cargo.toml'))) {
        throw "Not a Cargo crate: $crateDir"
    }
}

Invoke-DocsStep
Invoke-SecretsStep

Invoke-CargoStep -Name 'eval fmt' -WorkingDirectory $evalDir -Arguments @(
    'fmt', '--', '--check'
)
Invoke-CargoStep -Name 'eval clippy' -WorkingDirectory $evalDir -Arguments @(
    'clippy', '--locked', '--all-targets', '--', '-D', 'warnings'
)
Invoke-CargoStep -Name 'eval tests' -WorkingDirectory $evalDir -Arguments @(
    'test', '--locked'
)
Invoke-CargoStep -Name 'eval selftest' -WorkingDirectory $evalDir -Arguments @(
    'run', '--locked', '--', '--selftest'
)

Invoke-CargoStep -Name 'platform fmt' -WorkingDirectory $platformDir -Arguments @(
    'fmt', '--', '--check'
)
Invoke-CargoStep -Name 'platform clippy' -WorkingDirectory $platformDir -Arguments @(
    'clippy', '--locked', '--all-targets', '--', '-D', 'warnings'
)
Invoke-CargoStep -Name 'platform tests' -WorkingDirectory $platformDir -Arguments @(
    'test', '--locked'
)
Invoke-CargoStep -Name 'platform selftest' -WorkingDirectory $platformDir -Arguments @(
    'run', '--locked', '--bin', 'ekp_platform', '--', '--selftest'
)
Invoke-CargoStep -Name 'platform declared-rules schema' -WorkingDirectory $platformDir -Arguments @(
    'run', '--locked', '--bin', 'ekp_platform', '--', '--declared-rules-schema'
)
Invoke-CargoStep -Name 'platform search evaluation' -WorkingDirectory $platformDir -Arguments @(
    'run', '--locked', '--bin', 'ekp_platform', '--', '--eval', '--test-set', 'data/eval/search_eval.json'
)

if ($IncludeSmoke) {
    $reportDir = Join-Path $platformDir 'target\regression-report'
    $lrmlDir = Join-Path $platformDir 'target\regression-lrml'

    foreach ($dir in @($reportDir, $lrmlDir)) {
        if (Test-Path -LiteralPath $dir) {
            Remove-Item -LiteralPath $dir -Recurse -Force
        }
    }

    Invoke-CargoStep -Name 'platform smoke (report, schema, coverage, LegalRuleML, HTML)' -WorkingDirectory $platformDir -Arguments @(
        'run', '--locked', '--bin', 'ekp_platform', '--',
        '--params', 'config/engine-params.json',
        '--declared-rules-schema',
        '--out', 'target/regression-report',
        '--coverage-report',
        '--export-legalruleml', 'target/regression-lrml',
        '--html'
    )

    $expectedArtifacts = @(
        (Join-Path $reportDir 'report.json'),
        (Join-Path $reportDir 'report.md'),
        (Join-Path $reportDir 'report.html'),
        (Join-Path $reportDir 'coverage_report.md'),
        (Join-Path $reportDir 'engine_snapshots.json'),
        (Join-Path $reportDir 'engine_hard_gates.json')
    )
    foreach ($path in $expectedArtifacts) {
        Assert-Artifact -Path $path
    }

    $lrmlFiles = @(Get-ChildItem -LiteralPath $lrmlDir -Recurse -File -Filter '*.xml')
    if ($lrmlFiles.Count -eq 0) {
        throw "Missing LegalRuleML XML under $lrmlDir"
    }
    foreach ($file in $lrmlFiles) {
        Assert-LegalRuleML -Path $file.FullName
    }

    $htmlPath = Join-Path $reportDir 'report.html'
    if (Select-String -LiteralPath $htmlPath -Pattern '<script' -SimpleMatch -Quiet) {
        throw 'report.html contains a script element; the presentation layer must stay zero-JavaScript.'
    }
    Write-Host '  zero-script check ok: report.html'
}

Write-Host ''
Write-Host ("GATE PASSED: {0} step(s)" -f $script:Passed) -ForegroundColor Green
if (-not $IncludeSmoke) {
    Write-Host 'Smoke checks skipped. Use -IncludeSmoke for CLI report, export, coverage, or HTML changes.'
}
exit 0
