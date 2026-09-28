[CmdletBinding()]
param(
    [switch]$SkipBrowserSmoke,
    [string]$BrowserExecutable,
    [int]$BrowserSmokeSeconds = 10,
    [int]$NativeViewerSmokeSeconds = 10,
    # auto tries the normal backend, then retries one error-free adapter-timeout
    # with SwiftShader. normal disables fallback; swiftshader uses --disable-gpu.
    [ValidateSet("auto", "normal", "swiftshader")][string]$BrowserBackend = "auto"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".." )).Path
Set-Location $repoRoot

$targetDir = Join-Path $repoRoot "crates/target"
$releaseLauncher = Join-Path $targetDir "release/inox_launcher.exe"
$dataPc = Join-Path $repoRoot "data/pc"
$dataWeb = Join-Path $repoRoot "data/web"
$outputDir = Join-Path $repoRoot "output"
$stagingDir = Join-Path ([System.IO.Path]::GetTempPath()) ("inox-validation-" + [guid]::NewGuid())
$outputWeb = Join-Path $stagingDir "web"

function Invoke-Gate {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Command,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )

    Write-Host "`n=== $Name ===" -ForegroundColor Cyan
    Write-Host "$Command $($Arguments -join ' ')"
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE. No artifacts will be published."
    }
}

function Assert-GeneratedPlatformDirectory {
    param([Parameter(Mandatory = $true)][ValidateSet("pc", "web")][string]$Platform)

    $path = if ($Platform -eq "pc") { $dataPc } else { $dataWeb }
    if (-not (Test-Path -LiteralPath $path -PathType Container)) {
        throw "Generated $Platform data folder is missing: $path"
    }
    $files = @(Get-ChildItem -LiteralPath $path -File -Recurse)
    if ($files.Count -eq 0) {
        throw "Generated $Platform data folder is empty: $path"
    }
    $emptyFiles = @($files | Where-Object { $_.Length -le 0 })
    if ($emptyFiles.Count -ne 0) {
        throw "Generated $Platform data contains empty files: $($emptyFiles[0].FullName)"
    }
    $config = Join-Path $path "config/inox_binarizer/binarizer.cfg"
    if (-not (Test-Path -LiteralPath $config -PathType Leaf)) {
        throw "Generated $Platform data is missing its binarizer configuration: $config"
    }
    $configText = Get-Content -LiteralPath $config -Raw
    if ($configText -notmatch '"end_on_completion"\s*:\s*true') {
        throw "Generated $Platform binarizer configuration is not configured for completion: $config"
    }
    Write-Host "Validated newly generated data/${Platform}: $($files.Count) non-empty files and platform config."
}

function Find-BrowserExecutable {
    if ($BrowserExecutable) {
        if (-not (Test-Path -LiteralPath $BrowserExecutable -PathType Leaf)) {
            throw "Browser executable does not exist: $BrowserExecutable"
        }
        return (Resolve-Path -LiteralPath $BrowserExecutable).Path
    }

    $chromeCommand = Get-Command chrome.exe -ErrorAction SilentlyContinue
    $edgeCommand = Get-Command msedge.exe -ErrorAction SilentlyContinue
    $candidates = @()
    if ($chromeCommand) { $candidates += $chromeCommand.Source }
    if ($edgeCommand) { $candidates += $edgeCommand.Source }
    foreach ($programFilesRoot in @(${env:ProgramFiles}, ${env:ProgramFiles(x86)})) {
        if ($programFilesRoot) {
            $candidates += Join-Path $programFilesRoot "Google/Chrome/Application/chrome.exe"
            $candidates += Join-Path $programFilesRoot "Microsoft/Edge/Application/msedge.exe"
        }
    }
    if (${env:LOCALAPPDATA}) {
        $candidates += Join-Path ${env:LOCALAPPDATA} "Google/Chrome/Application/chrome.exe"
        $candidates += Join-Path ${env:LOCALAPPDATA} "Microsoft/Edge/Application/msedge.exe"
    }
    $candidates = @($candidates | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Leaf) })

    if ($candidates.Count -eq 0) {
        throw "SKIPPED: Browser WebGPU smoke is unavailable: Chrome or Edge was not found."
    }
    return $candidates[0]
}

function Invoke-NativeViewerSmoke {
    $viewer = $releaseLauncher
    if (-not (Test-Path -LiteralPath $viewer -PathType Leaf)) {
        throw "Native viewer smoke requires $viewer."
    }

    $smokeRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("inox-native-smoke-" + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $smokeRoot -Force | Out-Null
    $stdout = Join-Path $smokeRoot "stdout.log"
    $stderr = Join-Path $smokeRoot "stderr.log"
    $startupMarkerPattern = 'INOX_NATIVE_VIEWER_STARTUP_OK format=\S+'
    $queueNoOverflowMarkerPattern = 'INOX_NATIVE_WAVEFRONT_QUEUE_NO_OVERFLOW count=\d+ expected=\d+'
    $process = $null
    $startupSucceeded = $false
    try {
        Write-Host "Starting native viewer for $NativeViewerSmokeSeconds seconds."
        $process = Start-Process -FilePath $viewer -ArgumentList @("-plugin", "inox_viewer") `
            -RedirectStandardOutput $stdout -RedirectStandardError $stderr -PassThru -WindowStyle Hidden
        $deadline = (Get-Date).AddSeconds($NativeViewerSmokeSeconds)
        $startupMarker = $null
        while ((Get-Date) -lt $deadline) {
            if ($process.HasExited) {
                $stdoutText = if (Test-Path -LiteralPath $stdout) { Get-Content -LiteralPath $stdout -Raw } else { "" }
                $stderrText = if (Test-Path -LiteralPath $stderr) { Get-Content -LiteralPath $stderr -Raw } else { "" }
                throw "Native viewer exited with code $($process.ExitCode) before startup marker. stdout=$stdoutText stderr=$stderrText"
            }
            if (Test-Path -LiteralPath $stdout -PathType Leaf) {
                $stdoutText = [string](Get-Content -LiteralPath $stdout -Raw)
                $startupMatch = [regex]::Match($stdoutText, $startupMarkerPattern)
                if ($startupMatch.Success -and $startupMatch.Groups[0].Value -match 'format=\S+') {
                    $startupMarker = $startupMatch.Groups[0].Value
                    break
                }
            }
            Start-Sleep -Milliseconds 250
        }
        if (-not $startupMarker) {
            $stdoutText = if (Test-Path -LiteralPath $stdout) { Get-Content -LiteralPath $stdout -Raw } else { "" }
            $stderrText = if (Test-Path -LiteralPath $stderr) { Get-Content -LiteralPath $stderr -Raw } else { "" }
            throw "Native viewer did not emit a complete startup marker matching '$startupMarkerPattern' before the smoke deadline. stdout=$stdoutText stderr=$stderrText"
        }
        while ((Get-Date) -lt $deadline) {
            if ($process.HasExited) {
                $stdoutText = Get-Content -LiteralPath $stdout -Raw
                $stderrText = if (Test-Path -LiteralPath $stderr) { Get-Content -LiteralPath $stderr -Raw } else { "" }
                throw "Native viewer exited with code $($process.ExitCode) after startup marker. stdout=$stdoutText stderr=$stderrText"
            }
            Start-Sleep -Milliseconds 250
        }
        $stderrText = if (Test-Path -LiteralPath $stderr) { Get-Content -LiteralPath $stderr -Raw } else { "" }
        $queueNoOverflowMatch = [regex]::Match($stderrText, $queueNoOverflowMarkerPattern)
        if (-not $queueNoOverflowMatch.Success) {
            $stdoutText = if (Test-Path -LiteralPath $stdout) { Get-Content -LiteralPath $stdout -Raw } else { "" }
            throw "Native viewer did not emit a complete queue no-overflow marker. stdout=$stdoutText stderr=$stderrText"
        }
        $startupSucceeded = $true
        Write-Host "Native viewer emitted $startupMarker and $($queueNoOverflowMatch.Value); remained running through startup smoke."
    } finally {
        if ($process -and -not $process.HasExited) { Stop-Process -Id $process.Id -Force }
        if ($startupSucceeded) {
            if (Test-Path -LiteralPath $smokeRoot -PathType Container) {
                Remove-Item -LiteralPath $smokeRoot -Recurse -Force -ErrorAction SilentlyContinue
            }
        } else {
            Write-Host "Native viewer diagnostics preserved at $smokeRoot" -ForegroundColor Yellow
        }
    }
}

function Invoke-BrowserSmoke {
    $browser = Find-BrowserExecutable
    Write-Host "Using browser executable: $browser"
    $python = (Get-Command python.exe -ErrorAction SilentlyContinue).Source
    if (-not $python) {
        throw "SKIPPED: Browser WebGPU smoke is unavailable: python.exe is required for a local static server."
    }

    $server = $null
    $browserProfile = $null
    $port = 8765
    $server = Start-Process -FilePath $python -ArgumentList @("-m", "http.server", "$port", "--directory", $outputWeb) -PassThru -WindowStyle Hidden
    try {
        $url = "http://127.0.0.1:$port/smoke.html"
        $deadline = (Get-Date).AddSeconds(15)
        do {
            try {
                $response = Invoke-WebRequest -Uri $url -UseBasicParsing -TimeoutSec 2
                if ($response.StatusCode -eq 200) { break }
            } catch {
                if ((Get-Date) -ge $deadline) { throw "Browser smoke server did not start: $($_.Exception.Message)" }
            }
            Start-Sleep -Milliseconds 250
        } while ((Get-Date) -lt $deadline)

        $baseArguments = @(
            "--headless=new", "--disable-gpu-sandbox", "--enable-unsafe-webgpu",
            "--no-sandbox", "--disable-dev-shm-usage", "--enable-logging=stderr", "--log-level=0",
            "--virtual-time-budget=$($BrowserSmokeSeconds * 1000)"
        )
        $attempts = @()
        if ($BrowserBackend -ne "swiftshader") {
            $attempts += @{ Name = "normal"; Flags = @() }
        }
        if ($BrowserBackend -eq "swiftshader") {
            $attempts += @{ Name = "SwiftShader"; Flags = @("--disable-gpu") }
        } elseif ($BrowserBackend -eq "auto") {
            $attempts += @{ Name = "SwiftShader fallback"; Flags = @("--disable-gpu") }
        }

        for ($attemptIndex = 0; $attemptIndex -lt $attempts.Count; $attemptIndex++) {
            $attempt = $attempts[$attemptIndex]
            $browserProfile = Join-Path ([System.IO.Path]::GetTempPath()) ("inox-smoke-" + [guid]::NewGuid())
            $arguments = @($baseArguments + "--user-data-dir=$browserProfile" + $attempt.Flags + "--dump-dom" + $url)
            $oldErrorActionPreference = $ErrorActionPreference
            try {
                # Chrome writes informational diagnostics to stderr; capture them without
                # letting PowerShell's Stop policy abort before DOM validation runs.
                $ErrorActionPreference = "Continue"
                $browserOutput = (& $browser @arguments 2>&1 | Out-String)
            } finally {
                $ErrorActionPreference = $oldErrorActionPreference
            }
            $exitCode = $LASTEXITCODE
            if ($exitCode -ne 0) {
                throw "Browser WebGPU smoke ($($attempt.Name)) failed with exit code $exitCode. Browser output: $browserOutput"
            }
            $domText = $browserOutput
            $browserOutputLines = @($browserOutput -split '\r?\n' | Where-Object { $_ -ne '' })
            Write-Host "Browser smoke ($($attempt.Name)) returned $($browserOutputLines.Count) output lines."

            if ($domText -match 'data-inox-smoke="skipped"') {
                $skipReasonMatch = [regex]::Match($domText, 'data-inox-skip="([^"]+)"')
                if (-not $skipReasonMatch.Success) {
                    throw "Browser smoke reported skipped without an explicit capability reason. Browser output: $domText"
                }
                $skipReason = $skipReasonMatch.Groups[1].Value
                $statusTagMatch = [regex]::Match($domText, '<div id="status"[^>]*>')
                if (-not $statusTagMatch.Success) {
                    throw "Browser smoke did not return a status element for skip validation. Browser output: $domText"
                }
                $statusTag = $statusTagMatch.Value
                $adapterTimeoutWithoutErrors = $skipReason -eq "adapter-timeout" -and
                    $statusTag -match 'data-inox-instance="ready"' -and
                    $statusTag -match 'data-inox-adapter-requested="observed"' -and
                    $statusTag -match 'data-inox-adapter="pending"' -and
                    $statusTag -match 'data-inox-device-requested="pending"' -and
                    $statusTag -match 'data-inox-device="pending"' -and
                    $statusTag -match 'data-inox-error-count="0"' -and
                    $statusTag -match 'data-inox-device-failure="none"' -and
                    $statusTag -notmatch 'data-inox-smoke="failed"'
                if ($adapterTimeoutWithoutErrors -and $attemptIndex -eq 0 -and $attempts.Count -gt 1) {
                    Write-Host "Normal browser backend reported an error-free adapter timeout; retrying once with --disable-gpu (SwiftShader)." -ForegroundColor Yellow
                    Remove-Item -LiteralPath $browserProfile -Recurse -Force -ErrorAction SilentlyContinue
                    $browserProfile = $null
                    continue
                }
                if ($skipReason -eq "adapter-timeout") {
                    throw "SKIPPED: Browser WebGPU adapter request did not resolve before the smoke deadline; no usable browser backend was available. Browser output: $domText"
                }
                throw "SKIPPED: Browser WebGPU smoke is unavailable ($skipReason). Browser output: $domText"
            }
            if ($domText -notmatch 'data-inox-smoke="passed"') {
                throw "Browser smoke did not observe instance, adapter, device, graph order, dimensions, queue headers, finite/nonzero output contract, canvas ownership, submit, and frame signals. Browser output: $domText"
            }
            foreach ($marker in @(
                'data-inox-graph-order="[^"]+"',
                'data-inox-dimensions="[^"]+"',
                'data-inox-queue-headers="gpu-readback-validated"',
                'data-inox-output="finite-nonzero-validated"',
                'data-inox-canvas="owned"',
                'data-inox-instance="ready"',
                'data-inox-adapter-requested="observed"',
                'data-inox-adapter="ready"',
                'data-inox-device-requested="observed"',
                'data-inox-device="ready"',
                'data-inox-device-failure="[^"]*"',
                'data-inox-submit="observed"',
                'data-inox-frame="observed"'
            )) {
                if ($domText -notmatch $marker) {
                    throw "Browser smoke is missing required marker '$marker'. Browser output: $domText"
                }
            }
            if ($domText -notmatch 'data-inox-error-count="0"') {
                throw "Browser smoke captured page or console errors. Browser output: $domText"
            }
            Write-Host "Browser smoke ($($attempt.Name)) observed instance, adapter, device, graph, dimensions, queues, output, canvas, submit, and frame signals with no page or console errors."
            return
        }
        throw "Browser WebGPU smoke did not produce a result."
    } finally {
        if ($server -and -not $server.HasExited) { Stop-Process -Id $server.Id -Force }
        if ($browserProfile -and (Test-Path -LiteralPath $browserProfile -PathType Container)) {
            Remove-Item -LiteralPath $browserProfile -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

Write-Host "Cleaning generated data and unpublished output before validation."
Remove-Item -LiteralPath $dataPc, $dataWeb, $outputDir, $stagingDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
foreach ($platformData in @($dataPc, $dataWeb)) {
    New-Item -ItemType Directory -Path (Join-Path $platformData "config") -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $repoRoot "data_raw/config/inox_binarizer") -Destination (Join-Path $platformData "config") -Recurse -Force
}

if (-not (Test-Path -LiteralPath $releaseLauncher -PathType Leaf)) {
    throw "PC binarizer requires $releaseLauncher. Build the launcher before invoking this strict gate."
}

# Keep this order deliberate. Generated output is never copied to output/ until every gate succeeds.
Invoke-Gate "PC binarizer" $releaseLauncher @("-plugin", "inox_binarizer", "-platform", "pc")
Assert-GeneratedPlatformDirectory "pc"

$oldEncodedRustFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    # CARGO_ENCODED_RUSTFLAGS overrides the checked-in macOS-only rustflags.
    $env:CARGO_ENCODED_RUSTFLAGS = "--cfg=web_sys_unstable_apis"
    Invoke-Gate "Generated-data validation" "cargo" @("test", "-p", "inox_binarizer", "--tests", "--manifest-path", "./crates/Cargo.toml")
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $oldEncodedRustFlags
}

$nativeEnv = @{ CARGO_TARGET_DIR = "./crates/target"; CARGO_ENCODED_RUSTFLAGS = "--cfg=web_sys_unstable_apis" }
$oldCargoTarget = $env:CARGO_TARGET_DIR
$oldEncodedRustFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    $env:CARGO_TARGET_DIR = $nativeEnv.CARGO_TARGET_DIR
    $env:CARGO_ENCODED_RUSTFLAGS = $nativeEnv.CARGO_ENCODED_RUSTFLAGS
    Invoke-Gate "Native workspace tests" "cargo" @("test", "--workspace", "--manifest-path", "./crates/Cargo.toml")
    Invoke-Gate "Native clippy" "cargo" @("clippy", "--workspace", "--all-targets", "--manifest-path", "./crates/Cargo.toml", "--no-deps", "--", "-D", "warnings")
    Invoke-Gate "Native release build" "cargo" @("build", "--manifest-path", "./crates/Cargo.toml", "--release")
} finally {
    $env:CARGO_TARGET_DIR = $oldCargoTarget
    $env:CARGO_ENCODED_RUSTFLAGS = $oldEncodedRustFlags
}

Invoke-Gate "Formatting check" "cargo" @("fmt", "--manifest-path", "./crates/Cargo.toml", "--all", "--", "--check")
Invoke-NativeViewerSmoke

Invoke-Gate "Web binarizer" $releaseLauncher @("-plugin", "inox_binarizer", "-platform", "web")
Assert-GeneratedPlatformDirectory "web"

$wasmFlags = '--cfg=web_sys_unstable_apis' + [char]31 + '--cfg' + [char]31 + 'getrandom_backend="wasm_js"'
$oldEncodedRustFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    $env:CARGO_ENCODED_RUSTFLAGS = $wasmFlags
    Invoke-Gate "WASM release build" "cargo" @("build", "--manifest-path", "./crates/Cargo.toml", "--release", "--target", "wasm32-unknown-unknown")
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $oldEncodedRustFlags
}

$oldEncodedRustFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    $env:CARGO_ENCODED_RUSTFLAGS = '--cfg=web_sys_unstable_apis'
    Invoke-Gate "Pinned wasm-bindgen" "cargo" @("install", "-f", "wasm-bindgen-cli", "--version", "0.2.128")
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $oldEncodedRustFlags
}
New-Item -ItemType Directory -Path $outputWeb -Force | Out-Null
Invoke-Gate "WASM bindgen generation" "wasm-bindgen" @(
    "./crates/target/wasm32-unknown-unknown/release/inox_launcher.wasm",
    "--out-dir", $outputWeb, "--target", "web", "--no-typescript", "--no-demangle",
    "--keep-lld-exports", "--split-linked-modules"
)
Copy-Item -LiteralPath (Join-Path $repoRoot "web/smoke.html") -Destination $outputWeb -Force
Copy-Item -LiteralPath (Join-Path $repoRoot "data/web") -Destination (Join-Path $outputWeb "data") -Recurse -Force

if ($SkipBrowserSmoke) {
    Remove-Item -LiteralPath $stagingDir -Recurse -Force -ErrorAction SilentlyContinue
    throw "Browser WebGPU smoke was explicitly skipped; validation is incomplete and no artifacts were published."
} else {
    Invoke-BrowserSmoke
}

Move-Item -LiteralPath $stagingDir -Destination $outputDir
Write-Host "`nAll requested validation gates passed. Output is ready to publish." -ForegroundColor Green
