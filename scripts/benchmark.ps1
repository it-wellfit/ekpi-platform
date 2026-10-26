#requires -Version 5.1
# Local performance evidence for the deterministic core and the workbench.
#
# This is a reproducible microbenchmark on one host. It is NOT a capacity model,
# a multi-tenant load test, a soak test or a production SLO. Record the host and
# revision, and rerun after a material change. Do not compare results across
# different hosts as if they were the same environment.

param(
    [ValidateSet("debug", "release")]
    [string]$Profile = "debug",
    [int]$Samples = 7,
    [int]$Warmup = 2,
    [int]$SequentialHttpRequests = 100,
    [int]$ConcurrentHttpRequests = 400,
    [int]$HttpConcurrency = 32,
    [switch]$SkipHttp
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

function Get-Percentile {
    param(
        [double[]]$Values,
        [double]$Quantile
    )
    $sorted = @($Values | Sort-Object)
    if ($sorted.Count -eq 0) { throw "No samples." }
    $index = [Math]::Min($sorted.Count - 1, [Math]::Max(0, [Math]::Ceiling($Quantile * $sorted.Count) - 1))
    return [Math]::Round($sorted[$index], 3)
}

function Get-Summary {
    param([double[]]$Values)
    $sorted = @($Values | Sort-Object)
    $sum = 0.0
    foreach ($value in $sorted) { $sum += $value }
    return [ordered]@{
        count = $sorted.Count
        min_ms = [Math]::Round($sorted[0], 3)
        mean_ms = [Math]::Round($sum / $sorted.Count, 3)
        p50_ms = Get-Percentile -Values $sorted -Quantile 0.50
        p95_ms = Get-Percentile -Values $sorted -Quantile 0.95
        max_ms = [Math]::Round($sorted[$sorted.Count - 1], 3)
    }
}

function Invoke-Checked {
    param(
        [string]$Label,
        [scriptblock]$Command
    )
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label exited $LASTEXITCODE"
    }
}

$artifacts = Join-Path $root "artifacts\benchmark"
New-Item -ItemType Directory -Force -Path $artifacts | Out-Null
$stamp = Get-Date -Format "yyyy-MM-dd-HHmmss"
$resultPath = Join-Path $artifacts "benchmark-$stamp.json"

$revision = (git -C $root rev-parse HEAD).Trim()
$dirty = @(git -C $root status --porcelain)
$dirtyCount = $dirty.Count

Write-Host "=== Build $Profile binaries ===" -ForegroundColor Cyan
$profileArgs = @()
if ($Profile -eq "release") { $profileArgs += "--release" }
$targetDir = if ($Profile -eq "release") { "release" } else { "debug" }
Push-Location (Join-Path $root "platform")
try {
    Invoke-Checked "cargo build" {
        cargo build @profileArgs --locked --bin ekp_platform --bin governance_web
    }
}
finally { Pop-Location }

$platformExe = Join-Path $root "platform\target\$targetDir\ekp_platform.exe"
$serverExe = Join-Path $root "platform\target\$targetDir\governance_web.exe"
foreach ($exe in @($platformExe, $serverExe)) {
    if (-not (Test-Path -LiteralPath $exe)) { throw "Missing binary: $exe" }
}

$testSet = Join-Path $root "platform\data\eval\search_eval.json"
if (-not (Test-Path -LiteralPath $testSet)) { throw "Missing evaluation set: $testSet" }

Write-Host "=== Evaluation microbenchmark ($Samples samples, $Warmup warmup) ===" -ForegroundColor Cyan
for ($i = 0; $i -lt $Warmup; $i++) {
    Invoke-Checked "warmup $i" {
        & $platformExe --eval --test-set $testSet *> $null
    }
}

$evalSamples = New-Object System.Collections.Generic.List[double]
for ($i = 0; $i -lt $Samples; $i++) {
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    Invoke-Checked "sample $i" {
        & $platformExe --eval --test-set $testSet *> $null
    }
    $timer.Stop()
    $evalSamples.Add($timer.Elapsed.TotalMilliseconds)
    Write-Host ("  sample {0}: {1:N1} ms" -f ($i + 1), $timer.Elapsed.TotalMilliseconds)
}
$evalSummary = Get-Summary -Values $evalSamples.ToArray()

$httpResult = [ordered]@{ status = "skipped"; reason = "requested by caller" }
if (-not $SkipHttp) {
    Write-Host "=== Workbench loopback load test ===" -ForegroundColor Cyan
    $port = 3000
    $url = "http://127.0.0.1:$port/healthz"
    $existing = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue
    if ($existing) {
        $httpResult = [ordered]@{
            status = "skipped"
            reason = "port $port is already in use; refusing to measure or stop an unknown process"
        }
        Write-Host "SKIP: $($httpResult.reason)" -ForegroundColor Yellow
    }
    else {
        $ledger = Join-Path $env:TEMP ("ekpi-benchmark-" + [guid]::NewGuid().ToString("N") + ".jsonl")
        $stdout = Join-Path $env:TEMP ("ekpi-benchmark-" + $stamp + ".out.log")
        $stderr = Join-Path $env:TEMP ("ekpi-benchmark-" + $stamp + ".err.log")
        $previousLedger = $env:EKP_GOVERNANCE_DATA
        $env:EKP_GOVERNANCE_DATA = $ledger
        $process = $null
        try {
            $process = Start-Process -FilePath $serverExe -PassThru -WindowStyle Hidden `
                -RedirectStandardOutput $stdout -RedirectStandardError $stderr
            $env:EKP_GOVERNANCE_DATA = $previousLedger

            $ready = $false
            for ($attempt = 0; $attempt -lt 100; $attempt++) {
                Start-Sleep -Milliseconds 100
                try {
                    $probe = Invoke-WebRequest -UseBasicParsing -Uri $url -TimeoutSec 1
                    if ($probe.StatusCode -eq 200) { $ready = $true; break }
                }
                catch { }
            }
            if (-not $ready) {
                $tail = if (Test-Path -LiteralPath $stderr) { Get-Content -LiteralPath $stderr -Tail 20 } else { @() }
                throw "Workbench did not become healthy on $url. stderr: $($tail -join ' | ')"
            }

            for ($i = 0; $i -lt 10; $i++) {
                $null = Invoke-WebRequest -UseBasicParsing -Uri $url -TimeoutSec 2
            }

            $sequential = New-Object System.Collections.Generic.List[double]
            for ($i = 0; $i -lt $SequentialHttpRequests; $i++) {
                $timer = [System.Diagnostics.Stopwatch]::StartNew()
                $response = Invoke-WebRequest -UseBasicParsing -Uri $url -TimeoutSec 10
                $timer.Stop()
                if ($response.StatusCode -ne 200) { throw "HTTP $($response.StatusCode) on $url" }
                $sequential.Add($timer.Elapsed.TotalMilliseconds)
            }
            $sequentialSummary = Get-Summary -Values $sequential.ToArray()

            Add-Type -AssemblyName System.Net.Http
            $handler = New-Object System.Net.Http.HttpClientHandler
            $handler.MaxConnectionsPerServer = $HttpConcurrency
            $client = New-Object System.Net.Http.HttpClient($handler)
            $client.Timeout = [TimeSpan]::FromSeconds(30)
            $tasks = New-Object System.Collections.ArrayList
            $timer = [System.Diagnostics.Stopwatch]::StartNew()
            for ($i = 0; $i -lt $ConcurrentHttpRequests; $i++) {
                [void]$tasks.Add($client.GetAsync($url))
            }
            [System.Threading.Tasks.Task]::WaitAll([System.Threading.Tasks.Task[]]$tasks.ToArray())
            $timer.Stop()
            $ok = 0
            foreach ($task in $tasks) {
                if ($task.Result.IsSuccessStatusCode) { $ok++ }
            }
            $client.Dispose()
            $handler.Dispose()
            if ($ok -ne $ConcurrentHttpRequests) {
                throw "$ok of $ConcurrentHttpRequests concurrent requests succeeded."
            }

            $elapsedSeconds = $timer.Elapsed.TotalSeconds
            $httpResult = [ordered]@{
                status = "pass"
                url = $url
                sequential = $sequentialSummary
                concurrent = [ordered]@{
                    requests = $ConcurrentHttpRequests
                    concurrency = $HttpConcurrency
                    successes = $ok
                    elapsed_s = [Math]::Round($elapsedSeconds, 3)
                    requests_per_s = [Math]::Round($ConcurrentHttpRequests / $elapsedSeconds, 2)
                }
            }
            Write-Host ("  sequential p50/p95: {0}/{1} ms; concurrent {2:N1} req/s" -f `
                $sequentialSummary.p50_ms, $sequentialSummary.p95_ms, $httpResult.concurrent.requests_per_s) `
                -ForegroundColor Green
        }
        finally {
            $env:EKP_GOVERNANCE_DATA = $previousLedger
            if ($process -and -not $process.HasExited) {
                Stop-Process -Id $process.Id -Force
                $process.WaitForExit(5000) | Out-Null
            }
            Remove-Item -LiteralPath $ledger -Force -ErrorAction SilentlyContinue
        }
    }
}

$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$os = Get-CimInstance Win32_OperatingSystem
$record = [ordered]@{
    schema = "ekpi.benchmark.local.v1"
    created_at_utc = (Get-Date).ToUniversalTime().ToString("o")
    revision = $revision
    uncommitted_paths = $dirtyCount
    environment = [ordered]@{
        os = $os.Caption
        os_version = $os.Version
        cpu = $cpu.Name
        logical_processors = $cpu.NumberOfLogicalProcessors
        memory_gib = [Math]::Round($os.TotalVisibleMemorySize / 1MB, 1)
        rustc = (& rustc --version)
        powershell = $PSVersionTable.PSVersion.ToString()
    }
    evaluation = [ordered]@{
        profile = $Profile
        command = "ekp_platform --eval --test-set platform/data/eval/search_eval.json"
        warmup = $Warmup
        samples = $evalSummary
    }
    http = $httpResult
    limitations = @(
        "Loopback single-host measurement; not a multi-user or multi-tenant load test.",
        "No soak test, bottleneck profile, capacity model, cost model or approved SLO.",
        "The evaluation command is a deterministic lexical regression, not an LLM quality benchmark.",
        $(if ($Profile -eq "debug") { "Debug-profile build: latency is not representative of an optimized deployment." } else { "Release-profile build." })
    )
}
[System.IO.File]::WriteAllText($resultPath, (($record | ConvertTo-Json -Depth 6) + "`n"), $utf8NoBom)

Write-Host ""
Write-Host "Benchmark evidence written: $resultPath" -ForegroundColor Green
Write-Host ("  revision: {0} ({1} uncommitted paths)" -f $revision, $dirtyCount)
Write-Host ("  evaluation p50/p95: {0}/{1} ms" -f $evalSummary.p50_ms, $evalSummary.p95_ms)
if ($httpResult.status -eq "pass") {
    Write-Host ("  http p50/p95: {0}/{1} ms; {2} req/s" -f `
        $httpResult.sequential.p50_ms, $httpResult.sequential.p95_ms, $httpResult.concurrent.requests_per_s)
}
else {
    Write-Host ("  http: {0} ({1})" -f $httpResult.status, $httpResult.reason) -ForegroundColor Yellow
}
