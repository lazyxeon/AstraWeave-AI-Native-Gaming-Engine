param(
    [Parameter(Mandatory = $false)]
    [string]$OutDir = $(Join-Path "benchmark_results" (Get-Date -Format 'yyyy-MM-dd')),

    [Parameter(Mandatory = $false)]
    [switch]$NoPlot = $false,

    [Parameter(Mandatory = $false)]
    [string[]]$IncludePackages = @(),

    [Parameter(Mandatory = $false)]
    [string[]]$SkipPackages = @(),

    [Parameter(Mandatory = $false)]
    [switch]$ExportJsonl = $true,

    [Parameter(Mandatory = $false)]
    [switch]$LogVerbose
)

$ErrorActionPreference = "Stop"

function Write-Log {
    param([string]$Message)
    $ts = Get-Date -Format "HH:mm:ss"
    Write-Host "[$ts] $Message" -ForegroundColor Cyan
}

function Ensure-Dir {
    param([string]$Path)
    if (-not (Test-Path $Path)) {
        New-Item -ItemType Directory -Force -Path $Path | Out-Null
    }
}

function Append-File {
    param([string]$Path, [string]$Line)
    $Line | Add-Content -Encoding utf8 $Path
}

function Capture-Environment {
    param([string]$Path)

    "AstraWeave Benchmark Odyssey" | Out-File -Encoding utf8 $Path
    Append-File $Path "Timestamp (local): $(Get-Date -Format o)"
    Append-File $Path "Timestamp (UTC):   $((Get-Date).ToUniversalTime().ToString('o'))"

    try { Append-File $Path "OS: $([System.Environment]::OSVersion.VersionString)" } catch {}
    try { Append-File $Path "Machine: $env:COMPUTERNAME" } catch {}

    try {
        $cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1)
        Append-File $Path ("CPU: {0}" -f ($cpu.Name).Trim())
        Append-File $Path ("Logical processors: {0}" -f $cpu.NumberOfLogicalProcessors)
    } catch {}

    try {
        $mem = (Get-CimInstance Win32_ComputerSystem | Select-Object -ExpandProperty TotalPhysicalMemory)
        Append-File $Path ("RAM: {0:N1} GB" -f ($mem / 1GB))
    } catch {}

    try { Append-File $Path ("Rust: {0}" -f (rustc -Vv)) } catch {}
    try { Append-File $Path ("Cargo: {0}" -f (cargo -V)) } catch {}

    try {
        Append-File $Path ("Git SHA: {0}" -f (git rev-parse HEAD))
        Append-File $Path ("Git Branch: {0}" -f (git rev-parse --abbrev-ref HEAD))
        $dirty = ((git status --porcelain).Count -gt 0)
        Append-File $Path ("Git Dirty: {0}" -f $dirty)
    } catch {}
}

function Get-PackagesWithBenches {
    param([string]$MetadataPath)

    Write-Log "Generating cargo metadata (no deps)..."
    cargo metadata --no-deps --format-version 1 | Out-File -Encoding utf8 $MetadataPath

    $meta = Get-Content $MetadataPath -Raw | ConvertFrom-Json

    $pkgs = @()
    foreach ($p in $meta.packages) {
        $manifestDir = Split-Path -Parent $p.manifest_path
        if (Test-Path (Join-Path $manifestDir 'benches')) {
            $pkgs += $p.name
        }
    }

    return ($pkgs | Sort-Object -Unique)
}

function Invoke-Bench {
    param(
        [string]$Package,
        [string]$LogPath,
        [switch]$NoPlotFlag
    )

    if ($NoPlotFlag) {
        "=== cargo bench -p $Package -- --noplot ===" | Out-File -Encoding utf8 $LogPath
    } else {
        "=== cargo bench -p $Package ===" | Out-File -Encoding utf8 $LogPath
    }
    $sw = [System.Diagnostics.Stopwatch]::StartNew()

    $args = @('bench', '-p', $Package)
    if ($NoPlotFlag) {
        # Only forward extra args if explicitly requested.
        # NOTE: Some crates use libtest benches, which will reject Criterion-specific args.
        $args += '--'
        $args += '--noplot'
    }

    Write-Log "Running benches: $Package"

    try {
        # IMPORTANT (Windows PowerShell): native STDERR is surfaced as ErrorRecords, and with
        # $ErrorActionPreference = 'Stop' that can become terminating even when cargo exits 0.
        # Route through cmd.exe and merge 2>&1 at the process level.
        $cmdLine = ("cargo {0} 2>&1" -f ($args -join ' '))
        & cmd.exe /d /c $cmdLine | Tee-Object -FilePath $LogPath -Append | Out-Host
        $exitCode = $LASTEXITCODE
    } catch {
        $exitCode = 1
        "ERROR: $_" | Add-Content -Encoding utf8 $LogPath
    } finally {
        $sw.Stop()
        "Elapsed: $($sw.Elapsed.ToString())" | Add-Content -Encoding utf8 $LogPath
        "ExitCode: $exitCode" | Add-Content -Encoding utf8 $LogPath
    }

    return ($exitCode -eq 0)
}

# --- Main ---
Ensure-Dir $OutDir

$readmePath = Join-Path $OutDir 'README.txt'
"AstraWeave Benchmark Odyssey" | Out-File -Encoding utf8 $readmePath
Append-File $readmePath ("OutDir: {0}" -f $OutDir)

$envPath = Join-Path $OutDir 'environment.txt'
Capture-Environment $envPath

$metadataPath = Join-Path $OutDir 'cargo_metadata.json'
$benchPkgs = Get-PackagesWithBenches $metadataPath

$pkgsPath = Join-Path $OutDir 'packages_with_benches.txt'
$benchPkgs | Out-File -Encoding utf8 $pkgsPath
Append-File $readmePath ("Packages with benches: {0}" -f $benchPkgs.Count)

# Apply include/skip filters
if ($IncludePackages.Count -gt 0) {
    $benchPkgs = $benchPkgs | Where-Object { $IncludePackages -contains $_ }
}
if ($SkipPackages.Count -gt 0) {
    $benchPkgs = $benchPkgs | Where-Object { -not ($SkipPackages -contains $_) }
}

$priority = @(
    'astraweave-core','astraweave-ecs','astraweave-ai','astraweave-physics','astraweave-nav',
    'astraweave-render','astraweave-ui','astraweave-audio','astraweave-gameplay',
    'astraweave-persistence-ecs','astraweave-net-ecs','astraweave-terrain',
    'astraweave-math','astraweave-input','astraweave-pcg','astraweave-weaving',
    'astraweave-memory','astraweave-context','astraweave-persona','astraweave-prompts','astraweave-rag',
    'astraweave-scripting','astraweave-sdk','astraweave-stress-test','astract',
    'aw_build','aw_editor','aw-save'
)

$runList = @()
foreach ($p in $priority) {
    if ($benchPkgs -contains $p) { $runList += $p }
}
foreach ($p in $benchPkgs) {
    if (-not ($runList -contains $p)) { $runList += $p }
}

$runOrderPath = Join-Path $OutDir 'run_order.txt'
$runList | Out-File -Encoding utf8 $runOrderPath

$results = @()
foreach ($pkg in $runList) {
    $logPath = Join-Path $OutDir ("bench_{0}.log" -f $pkg)
    $ok = Invoke-Bench -Package $pkg -LogPath $logPath -NoPlotFlag:$NoPlot
    $results += [PSCustomObject]@{ package = $pkg; success = $ok }
}

$resultsPath = Join-Path $OutDir 'run_results.json'
$results | ConvertTo-Json | Out-File -Encoding utf8 $resultsPath

$okCount = ($results | Where-Object { $_.success }).Count
$failCount = $results.Count - $okCount
Append-File $readmePath ("Bench runs: {0} total, {1} success, {2} failed" -f $results.Count, $okCount, $failCount)

if ($ExportJsonl) {
    $jsonlPath = Join-Path $OutDir 'history.jsonl'
    $exportLog = Join-Path $OutDir 'export_history.log'

    if (Test-Path 'target/criterion') {
        Write-Log "Exporting Criterion estimates to JSONL..."
        try {
            & .\scripts\export_benchmark_jsonl.ps1 -BenchmarkDir 'target/criterion' -OutputFile $jsonlPath -MaxAgeDays 365 2>&1 |
                Tee-Object -FilePath $exportLog | Out-Host
            Append-File $readmePath ("Exported JSONL: {0}" -f $jsonlPath)
        } catch {
            "ERROR: JSONL export failed: $_" | Out-File -Encoding utf8 $exportLog
            Append-File $readmePath "JSONL export failed (see export_history.log)"
        }
    } else {
        Append-File $readmePath "No Criterion output found at target/criterion (skipped JSONL export)"
    }
}

Write-Log "Benchmark odyssey complete: $okCount succeeded, $failCount failed."
