# AstraWeave Benchmark Dashboard Runner
# One-command solution to run benchmarks, export data, generate graphs, and open dashboard
#
# Usage:
#   .\scripts\run_benchmark_dashboard.ps1           # Run everything
#   .\scripts\run_benchmark_dashboard.ps1 -SkipBench  # Skip running benchmarks (use existing data)
#   .\scripts\run_benchmark_dashboard.ps1 -Port 8080  # Use custom port

param(
    [switch]$SkipBench,
    [int]$Port = 8000,
    [switch]$NoBrowser
)

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message, [string]$Status = "INFO")
    $timestamp = Get-Date -Format "HH:mm:ss"
    $symbols = @{
        "INFO" = "ℹ️"
        "SUCCESS" = "✅"
        "ERROR" = "❌"
        "RUNNING" = "⚙️"
        "SKIP" = "⏭️"
    }
    $colors = @{
        "INFO" = "Cyan"
        "SUCCESS" = "Green"
        "ERROR" = "Red"
        "RUNNING" = "Yellow"
        "SKIP" = "DarkGray"
    }
    
    $symbol = $symbols[$Status]
    $color = $colors[$Status]
    
    Write-Host "$symbol [$timestamp] $Message" -ForegroundColor $color
}

function Test-Command {
    param([string]$Command)
    try {
        $null = Get-Command $Command -ErrorAction Stop
        return $true
    }
    catch {
        return $false
    }
}

# Header
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Magenta
Write-Host "   AstraWeave Benchmark Dashboard Runner" -ForegroundColor Magenta
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Magenta
Write-Host ""

# Verify prerequisites
Write-Step "Checking prerequisites..." "RUNNING"

if (-not (Test-Command "cargo")) {
    Write-Step "Cargo not found. Install Rust from https://rustup.rs/" "ERROR"
    exit 1
}

if (-not (Test-Command "python")) {
    Write-Step "Python not found. Install Python 3.8+ from https://python.org/" "ERROR"
    exit 1
}

Write-Step "Prerequisites OK (cargo, python)" "SUCCESS"

# Step 1: Run benchmarks (unless skipped)
if ($SkipBench) {
    Write-Step "Skipping benchmark run (using existing data)" "SKIP"
}
else {
    Write-Step "Running core benchmarks (this may take 5-10 minutes)..." "RUNNING"
    Write-Host ""
    
    $benchPackages = @(
        "astraweave-ecs",
        "astraweave-ai",
        "astraweave-physics",
        "astraweave-math",
        "astraweave-terrain",
        "astraweave-input"
    )
    
    $benchArgs = @("bench", "--no-fail-fast")
    foreach ($pkg in $benchPackages) {
        $benchArgs += "--package"
        $benchArgs += $pkg
    }
    
    try {
        & cargo @benchArgs 2>&1 | Out-Null
        Write-Step "Benchmarks completed successfully" "SUCCESS"
    }
    catch {
        Write-Step "Benchmark run failed, but continuing with existing data..." "ERROR"
    }
}

# Step 2: Export benchmark data to JSONL
Write-Step "Exporting benchmark results to JSONL..." "RUNNING"

try {
    $exportOutput = & ".\scripts\export_benchmark_jsonl.ps1" 2>&1 | Out-String
    
    if ($LASTEXITCODE -eq 0) {
        # Extract benchmark count from output
        if ($exportOutput -match 'Total benchmarks exported: (\d+)') {
            $count = $matches[1]
            Write-Step "Exported $count benchmarks to target/benchmark-data/history.jsonl" "SUCCESS"
        }
        else {
            Write-Step "Benchmark data exported successfully" "SUCCESS"
        }
    }
    else {
        Write-Step "Export script failed" "ERROR"
        Write-Host $exportOutput -ForegroundColor Red
        exit 1
    }
}
catch {
    Write-Step "Failed to run export script: $_" "ERROR"
    exit 1
}

# Step 3: Generate graphs
Write-Step "Generating visualization graphs..." "RUNNING"

# Find Python executable
$pythonExe = "python"
if (Test-Command "python3") {
    $pythonExe = "python3"
}
if (Test-Path "C:/Users/pv2br/AppData/Local/Microsoft/WindowsApps/python3.13.exe") {
    $pythonExe = "C:/Users/pv2br/AppData/Local/Microsoft/WindowsApps/python3.13.exe"
}

# Ensure required Python packages are installed
Write-Step "Checking Python dependencies..." "RUNNING"
try {
    & $pythonExe -c "import pandas, matplotlib, seaborn" 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Step "Installing required Python packages (pandas, matplotlib, seaborn)..." "RUNNING"
        & $pythonExe -m pip install pandas matplotlib seaborn --quiet
    }
}
catch {
    Write-Step "Installing required Python packages..." "RUNNING"
    & $pythonExe -m pip install pandas matplotlib seaborn --quiet 2>&1 | Out-Null
}

try {
    $graphOutput = & $pythonExe "scripts/generate_benchmark_graphs.py" `
        --input "target/benchmark-data/history.jsonl" `
        --out-dir "gh-pages/graphs" 2>&1 | Out-String
    
    if ($LASTEXITCODE -eq 0) {
        Write-Step "Generated graphs: top_series_time.png, distribution_latest.png, heatmap.png" "SUCCESS"
    }
    else {
        Write-Step "Graph generation failed (dashboard will still work)" "ERROR"
        Write-Host $graphOutput -ForegroundColor Yellow
    }
}
catch {
    Write-Step "Graph generation failed: $_ (dashboard will still work)" "ERROR"
}

# Step 4: Copy data to dashboard directory for easier access
Write-Step "Preparing dashboard data..." "RUNNING"

$dashboardDir = "tools/benchmark-dashboard"
$targetDataDir = "$dashboardDir/benchmark-data"

# Create symlink or copy data to dashboard directory
if (-not (Test-Path $targetDataDir)) {
    New-Item -ItemType Directory -Path $targetDataDir -Force | Out-Null
}

Copy-Item "target/benchmark-data/history.jsonl" "$targetDataDir/history.jsonl" -Force
Copy-Item "target/benchmark-data/metadata.json" "$targetDataDir/metadata.json" -Force

# Copy graphs if they exist
if (Test-Path "gh-pages/graphs") {
    if (-not (Test-Path "$dashboardDir/graphs")) {
        New-Item -ItemType Directory -Path "$dashboardDir/graphs" -Force | Out-Null
    }
    
    Get-ChildItem "gh-pages/graphs/*.png" | ForEach-Object {
        Copy-Item $_.FullName "$dashboardDir/graphs/" -Force
    }
}

Write-Step "Dashboard data prepared" "SUCCESS"

# Step 5: Start HTTP server
Write-Step "Starting HTTP server on port $Port..." "RUNNING"

$serverJob = Start-Job -ScriptBlock {
    param($Dir, $Port)
    Set-Location $Dir
    python -m http.server $Port
} -ArgumentList (Resolve-Path $dashboardDir).Path, $Port

Start-Sleep -Seconds 2

# Verify server started
try {
    $response = Invoke-WebRequest -Uri "http://localhost:$Port" -TimeoutSec 5 -UseBasicParsing -ErrorAction Stop
    Write-Step "HTTP server started successfully" "SUCCESS"
}
catch {
    Write-Step "Failed to verify HTTP server: $_" "ERROR"
    Stop-Job $serverJob
    Remove-Job $serverJob
    exit 1
}

# Step 6: Open browser
$url = "http://localhost:$Port"

if (-not $NoBrowser) {
    Write-Step "Opening browser to $url..." "RUNNING"
    Start-Process $url
    Write-Step "Browser launched" "SUCCESS"
}

# Summary
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "   Dashboard is running!" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host ""
Write-Host "  URL: " -NoNewline
Write-Host $url -ForegroundColor Cyan
Write-Host ""
Write-Host "  Data: " -NoNewline
Write-Host "target/benchmark-data/history.jsonl" -ForegroundColor Cyan
Write-Host "  Graphs: " -NoNewline
Write-Host "gh-pages/graphs/*.png" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Ctrl+C to stop the server..." -ForegroundColor Yellow
Write-Host ""

# Keep server running and wait for Ctrl+C
try {
    while ($true) {
        Start-Sleep -Seconds 1
        
        # Check if job is still running
        if ((Get-Job -Id $serverJob.Id).State -ne "Running") {
            Write-Step "Server stopped unexpectedly" "ERROR"
            break
        }
    }
}
finally {
    Write-Host ""
    Write-Step "Shutting down server..." "RUNNING"
    Stop-Job $serverJob -ErrorAction SilentlyContinue
    Remove-Job $serverJob -ErrorAction SilentlyContinue
    Write-Step "Server stopped" "SUCCESS"
}
