# S14 / IMP-021: runtime-data restore drill.
# Synthetic, non-sensitive data only. Exercises the REAL server, the REAL
# dual JSONL chains, a real kill, a real file loss, a real restore, and
# observes both chain badges over HTTP. Nothing here is mocked.
#
# Usage:  powershell -ExecutionPolicy Bypass -File scripts/drill-runtime-data.ps1 [-Port 3917] [-KeepWorkDir]
# Exit:   0 = PASS, non-zero = FAIL/BLOCKED (reason printed).

param(
    [string]$Port = "3917",
    [switch]$KeepWorkDir
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Net.Http
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$repo = Split-Path -Parent $PSScriptRoot
$workDir = Join-Path $env:TEMP "ekpi-runtime-drill-$stamp"

function Fail([string]$step, [string]$reason) {
    Write-Output "DRILL FAIL: step '$step' - $reason"
    if (-not $KeepWorkDir) { }
    exit 5
}

# ── 0. preflight ────────────────────────────────────────────────────────
$exe = Join-Path $repo "target\debug\governance_web.exe"
if (-not (Test-Path $exe)) {
    Write-Output "DRILL BLOCKED: governance_web.exe not built (run: cargo build --locked -p ekp_platform --bin governance_web)"
    exit 3
}
$inUse = Get-NetTCPConnection -LocalPort ([int]$Port) -State Listen -ErrorAction SilentlyContinue
if ($inUse) {
    Write-Output "DRILL BLOCKED: port $Port already in use"
    exit 3
}
New-Item -ItemType Directory -Force -Path $workDir | Out-Null
$govData = Join-Path $workDir "governance-ledger.jsonl"
$apprData = Join-Path $workDir "approvals.jsonl"
$backupDir = Join-Path $workDir "backup"
Write-Output "DRILL: work dir $workDir (port $Port)"

# ── 1. first boot with synthetic data env ───────────────────────────────
$env:EKP_GOVERNANCE_DATA = $govData
$env:EKP_APPROVAL_DATA = $apprData
$env:EKPI_LOCAL_AUTH_USER = "drill-operator"
$env:EKPI_LOCAL_AUTH_PASSWORD = "drill-pass-2026"
$env:EKPI_LOCAL_AUTH_ROLE = "approver"
$env:EKPI_WEB_PORT = $Port

$proc1 = Start-Process -FilePath $exe -PassThru -NoNewWindow `
    -RedirectStandardOutput (Join-Path $workDir "boot1-out.log") `
    -RedirectStandardError (Join-Path $workDir "boot1-err.log")
try {
    $ready = $false
    foreach ($i in 1..40) {
        Start-Sleep -Milliseconds 500
        try {
            $health = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/healthz" -UseBasicParsing -TimeoutSec 2
            if ($health.StatusCode -eq 200) { $ready = $true; break }
        } catch { }
    }
    if (-not $ready) { Fail "boot1" "server did not become ready in 20s" }
    Write-Output "DRILL: boot1 ready"
} catch {
    if (-not $proc1.HasExited) { Stop-Process -Id $proc1.Id -Force -ErrorAction SilentlyContinue }
    Fail "boot1" "$_"
}

# ── 2. HTTP flow over both chains (HttpClient, no redirects) ────────────
$handler = New-Object System.Net.Http.HttpClientHandler
$handler.CookieContainer = New-Object System.Net.CookieContainer
$handler.AllowAutoRedirect = $false
$client = New-Object System.Net.Http.HttpClient($handler)
$client.Timeout = [TimeSpan]::FromSeconds(10)
$base = "http://127.0.0.1:$Port"
$client.DefaultRequestHeaders.Add("Origin", $base)

function Step([string]$name, [System.Net.Http.HttpResponseMessage]$response, [int]$expected) {
    if ([int]$response.StatusCode -ne $expected) {
        Fail $name "expected HTTP $expected, got $([int]$response.StatusCode)"
    }
    Write-Output "DRILL: $name -> $([int]$response.StatusCode)"
}

try {
    # 2a. login double-submit: GET /login first (fresh token in hidden field +
    # ekpi_csrf cookie), then POST both back. A direct POST without the dance
    # is correctly refused by login_post's own double-submit check (S1).
    $loginPage = $client.GetStringAsync("$base/login").Result
    $tokenMatch = [regex]::Match($loginPage, 'name="csrf_token" value="([0-9a-f]+)"')
    if (-not $tokenMatch.Success) { Fail "login-token" "hidden csrf_token not found on the login page" }
    $csrfToken = $tokenMatch.Groups[1].Value
    $loginForm = New-Object System.Net.Http.StringContent("csrf_token=$csrfToken&username=drill-operator&password=drill-pass-2026", [System.Text.Encoding]::ASCII, "application/x-www-form-urlencoded")
    Step "login" ($client.PostAsync("$base/login", $loginForm).Result) 303

    # 2b. governance chain: create a project
    $projectForm = New-Object System.Net.Http.StringContent("name=Drill-Project&department=Drill&owner=Drill-Operator&risk_tier=low", [System.Text.Encoding]::ASCII, "application/x-www-form-urlencoded")
    Step "project" ($client.PostAsync("$base/projects", $projectForm).Result) 303

    # 2c. approval chain: definition -> publish -> submit -> decide
    $workflowForm = New-Object System.Net.Http.StringContent("id=wf-drill&name=Drill-Flow&stages=s1&slas=3600&urgency=normal", [System.Text.Encoding]::ASCII, "application/x-www-form-urlencoded")
    Step "workflow" ($client.PostAsync("$base/approvals/workflows", $workflowForm).Result) 303
        $publishForm = New-Object System.Net.Http.StringContent("", [System.Text.Encoding]::ASCII, "application/x-www-form-urlencoded")
    Step "publish" ($client.PostAsync("$base/approvals/workflows/wf-drill/1/publish", $publishForm).Result) 303
    $submitForm = New-Object System.Net.Http.StringContent("workflow_ref=wf-drill@1&amount_cents=1&platform=drill&classification=drill&risk_score=0&urgency=normal", [System.Text.Encoding]::ASCII, "application/x-www-form-urlencoded")
    Step "submit" ($client.PostAsync("$base/approvals/submit", $submitForm).Result) 303
    $decideForm = New-Object System.Net.Http.StringContent("action=approve&note=drill-approval", [System.Text.Encoding]::ASCII, "application/x-www-form-urlencoded")
    Step "decide" ($client.PostAsync("$base/approvals/TSK-000001/decide", $decideForm).Result) 303

    # 2d. observe both badges over HTTP
    $audit = $client.GetAsync("$base/audit").Result
    Step "audit" $audit 200
    $auditHtml = (New-Object System.Net.Http.HttpClient).GetStringAsync("$base/audit").Result
    $auditHtml = $client.GetStringAsync("$base/audit").Result
    if ($auditHtml -notmatch "本機雜湊鏈驗證通過") { Fail "audit-badge" "governance chain badge missing after flow" }
    $approvalsHtml = $client.GetStringAsync("$base/approvals").Result
    if ($approvalsHtml -notmatch "鏈已驗證") { Fail "approvals-badge" "approval chain badge missing after flow" }
    if ($approvalsHtml -notmatch "TSK-000001") { Fail "approvals-content" "submitted task missing on /approvals" }
    Write-Output "DRILL: both chain badges verified over HTTP"
} catch {
    if (-not $proc1.HasExited) { Stop-Process -Id $proc1.Id -Force -ErrorAction SilentlyContinue }
    Fail "http-flow" "$_"
}

# ── 3. kill (crash-like), back up, lose the originals ───────────────────
Stop-Process -Id $proc1.Id -Force
Start-Sleep -Milliseconds 500
if ($proc1.HasExited -eq $false) { Fail "kill" "server did not stop" }
New-Item -ItemType Directory -Force -Path $backupDir | Out-Null
Copy-Item $govData (Join-Path $backupDir "governance-ledger.jsonl")
Copy-Item $apprData (Join-Path $backupDir "approvals.jsonl")
Remove-Item $govData -Force
Remove-Item $apprData -Force
if ((Test-Path $govData) -or (Test-Path $apprData)) { Fail "lose" "originals still present after delete" }
Write-Output "DRILL: originals deleted; backup holds both chains"

# ── 4. restore + reboot + verify over HTTP again ────────────────────────
Copy-Item (Join-Path $backupDir "governance-ledger.jsonl") $govData
Copy-Item (Join-Path $backupDir "approvals.jsonl") $apprData

$proc2 = Start-Process -FilePath $exe -PassThru -NoNewWindow `
    -RedirectStandardOutput (Join-Path $workDir "boot2-out.log") `
    -RedirectStandardError (Join-Path $workDir "boot2-err.log")
try {
    $ready = $false
    foreach ($i in 1..40) {
        Start-Sleep -Milliseconds 500
        try {
            $health = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/healthz" -UseBasicParsing -TimeoutSec 2
            if ($health.StatusCode -eq 200) { $ready = $true; break }
        } catch { }
    }
    if (-not $ready) { Fail "boot2" "restored server did not become ready (tamper would fail startup loudly)" }

    # new session: login double-submit against the restored data
    $handler2 = New-Object System.Net.Http.HttpClientHandler
    $handler2.CookieContainer = New-Object System.Net.CookieContainer
    $handler2.AllowAutoRedirect = $false
    $client2 = New-Object System.Net.Http.HttpClient($handler2)
    $client2.Timeout = [TimeSpan]::FromSeconds(10)
    $client2.DefaultRequestHeaders.Add("Origin", $base)
    $loginPage2 = $client2.GetStringAsync("$base/login").Result
    $tokenMatch2 = [regex]::Match($loginPage2, 'name="csrf_token" value="([0-9a-f]+)"')
    if (-not $tokenMatch2.Success) { Fail "boot2-login-token" "hidden csrf_token not found" }
    $csrfToken2 = $tokenMatch2.Groups[1].Value
    $loginForm2 = New-Object System.Net.Http.StringContent("csrf_token=$csrfToken2&username=drill-operator&password=drill-pass-2026", [System.Text.Encoding]::ASCII, "application/x-www-form-urlencoded")
    Step "boot2-login" ($client2.PostAsync("$base/login", $loginForm2).Result) 303

    $audit2 = $client2.GetStringAsync("$base/audit").Result
    if ($audit2 -notmatch "本機雜湊鏈驗證通過") { Fail "restore-audit-badge" "restored governance chain badge missing" }
    $approvals2 = $client2.GetStringAsync("$base/approvals").Result
    if ($approvals2 -notmatch "鏈已驗證") { Fail "restore-approvals-badge" "restored approval chain badge missing" }
    if ($approvals2 -notmatch "TSK-000001") { Fail "restore-content" "restored task missing" }
    Write-Output "DRILL: restore verified over HTTP (both chains + content)"
} catch {
    if (-not $proc2.HasExited) { Stop-Process -Id $proc2.Id -Force -ErrorAction SilentlyContinue }
    Fail "restore" "$_"
} finally {
    if (-not $proc2.HasExited) { Stop-Process -Id $proc2.Id -Force -ErrorAction SilentlyContinue }
}

Write-Output "DRILL PASS: runtime data restored; both chains verified over HTTP"
Write-Output "DRILL: artifacts at $workDir (synthetic non-sensitive data only)"
exit 0
