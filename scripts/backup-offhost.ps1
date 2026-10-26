#requires -Version 5.1
# Encrypted off-host backup envelope.
#
# The script creates a normal Git bundle + restore drill first, then encrypts the
# bundle with age to an operator-supplied destination. It refuses to run on a
# dirty tree, inside the repository, or without an explicit off-host attestation.
# A -Drill run uses a temporary destination, requires the identity file, and
# verifies the decrypted bundle byte-for-byte. A drill is not off-host evidence.

param(
    [Parameter(Mandatory = $true)][string]$Destination,
    [Parameter(Mandatory = $true)][string]$Recipient,
    [string]$IdentityFile = "",
    [string]$BundleDir = "$env:USERPROFILE\EKPI-backup\bundles",
    [switch]$Drill,
    [switch]$AcknowledgeOffHost,
    [switch]$Force
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

function Fail {
    param([string]$Message)
    Write-Host "FAIL: $Message" -ForegroundColor Red
    exit 1
}

function Resolve-Age {
    $command = Get-Command "age" -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    $winget = Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages"
    if (Test-Path -LiteralPath $winget) {
        $found = Get-ChildItem -LiteralPath $winget -Recurse -Filter "age.exe" -ErrorAction SilentlyContinue |
            Select-Object -First 1
        if ($found) { return $found.FullName }
    }
    return $null
}

$age = Resolve-Age
if (-not $age) {
    Fail "age is not installed. Install it with: winget install --id FiloSottile.age -e"
}
if ($Recipient -notmatch '^age1[0-9a-z]+$') {
    Fail "Recipient must be an age public key (age1...)."
}
if (-not $Drill -and -not $AcknowledgeOffHost) {
    Fail "Pass -AcknowledgeOffHost to attest that the destination is off-host, or -Drill for a local test."
}
if ($Drill -and [string]::IsNullOrWhiteSpace($IdentityFile)) {
    Fail "A drill requires -IdentityFile so the ciphertext can be decrypted and verified."
}

$destinationPath = [System.IO.Path]::GetFullPath($Destination)
$rootPath = [System.IO.Path]::GetFullPath($root)
$bundlePath = [System.IO.Path]::GetFullPath($BundleDir)
if ($destinationPath.StartsWith($rootPath, [System.StringComparison]::OrdinalIgnoreCase)) {
    Fail "Destination must not be inside the repository."
}
if ($destinationPath.TrimEnd("\") -eq $bundlePath.TrimEnd("\")) {
    Fail "Destination must differ from the local bundle directory."
}
if ($Drill) {
    $tempRoot = [System.IO.Path]::GetFullPath($env:TEMP)
    if (-not $destinationPath.StartsWith($tempRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        Fail "A drill destination must be under TEMP."
    }
}
if ($IdentityFile -and -not (Test-Path -LiteralPath $IdentityFile)) {
    Fail "Identity file not found: $IdentityFile"
}

$dirty = git -C $root status --porcelain
if ($LASTEXITCODE -ne 0) { Fail "git status failed." }
if (-not [string]::IsNullOrWhiteSpace(($dirty -join ""))) {
    Fail "Working tree is dirty; commit the work before creating a backup."
}

New-Item -ItemType Directory -Force -Path $destinationPath | Out-Null
New-Item -ItemType Directory -Force -Path $bundlePath | Out-Null

$started = Get-Date
$stamp = Get-Date -Format "yyyy-MM-dd-HHmmss"
$log = Join-Path $env:TEMP "ekpi-offhost-$stamp.log"

Write-Host "=== Plaintext bundle and restore drill ===" -ForegroundColor Cyan
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "backup.ps1") *> $log
if ($LASTEXITCODE -ne 0) {
    Write-Host "Last 40 lines of $log" -ForegroundColor Yellow
    Get-Content -LiteralPath $log -Tail 40
    Fail "scripts/backup.ps1 failed; no encrypted copy was made."
}

$bundle = Get-ChildItem -LiteralPath $bundlePath -Filter "*.bundle" |
    Where-Object { $_.LastWriteTime -ge $started } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if (-not $bundle) {
    Fail "No new bundle was found in $bundlePath."
}

$cipher = Join-Path $destinationPath ($bundle.BaseName + ".bundle.age")
if ((Test-Path -LiteralPath $cipher) -and -not $Force) {
    Fail "Ciphertext already exists: $cipher (use -Force to overwrite)."
}

Write-Host "=== age encryption ===" -ForegroundColor Cyan
& $age --encrypt --recipient $Recipient --output $cipher $bundle.FullName
if ($LASTEXITCODE -ne 0) { Fail "age encryption failed." }

$bundleHash = (Get-FileHash -LiteralPath $bundle.FullName -Algorithm SHA256).Hash
$cipherHash = (Get-FileHash -LiteralPath $cipher -Algorithm SHA256).Hash
[System.IO.File]::WriteAllText("$cipher.sha256", "$cipherHash  $($bundle.BaseName).bundle.age`n", $utf8NoBom)

$decryptionVerified = $false
if ($IdentityFile) {
    Write-Host "=== Decryption verification ===" -ForegroundColor Cyan
    $plain = Join-Path $env:TEMP ("ekpi-decrypted-" + [guid]::NewGuid().ToString("N") + ".bundle")
    try {
        & $age --decrypt --identity $IdentityFile --output $plain $cipher
        if ($LASTEXITCODE -ne 0) { Fail "age decryption failed." }
        $plainHash = (Get-FileHash -LiteralPath $plain -Algorithm SHA256).Hash
        if ($plainHash -ne $bundleHash) {
            Fail "Decrypted bundle hash does not match the source bundle."
        }
        git -C $root bundle verify $plain *> $null
        if ($LASTEXITCODE -ne 0) { Fail "Decrypted bundle failed git bundle verify." }
        $decryptionVerified = $true
        Write-Host "PASS: decrypted bundle matches the source and verifies." -ForegroundColor Green
    }
    finally {
        if (Test-Path -LiteralPath $plain) { Remove-Item -LiteralPath $plain -Force }
    }
}

$revision = (git -C $root rev-parse HEAD).Trim()
$receipt = [ordered]@{
    schema              = "ekpi.backup.offhost.v1"
    mode                = $(if ($Drill) { "drill" } else { "off-host" })
    created_at_utc      = (Get-Date).ToUniversalTime().ToString("o")
    revision            = $revision
    bundle              = $bundle.Name
    bundle_sha256       = $bundleHash
    ciphertext          = (Split-Path -Leaf $cipher)
    ciphertext_sha256   = $cipherHash
    recipient           = $Recipient
    restore_drill       = "PASS (scripts/backup.ps1)"
    decryption_verified = $decryptionVerified
    destination_class   = $(if ($Drill) { "local-simulated-off-host" } else { "operator-attested-off-host" })
}
$receiptPath = Join-Path $destinationPath ($bundle.BaseName + ".receipt.json")
[System.IO.File]::WriteAllText($receiptPath, (($receipt | ConvertTo-Json -Depth 4) + "`n"), $utf8NoBom)

Write-Host ""
Write-Host "Encrypted backup PASSED" -ForegroundColor Green
Write-Host "  revision : $revision"
Write-Host "  source   : $($bundle.FullName) ($bundleHash)"
Write-Host "  cipher   : $cipher ($cipherHash)"
Write-Host "  receipt  : $receiptPath"
Write-Host "  mode     : $($receipt.mode)"
if ($Drill) {
    Write-Host "This was a drill; it is not evidence of an off-host copy." -ForegroundColor Yellow
}
