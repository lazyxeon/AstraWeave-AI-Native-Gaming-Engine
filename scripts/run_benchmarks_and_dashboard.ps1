#!/usr/bin/env pwsh
# All-in-One Benchmark Dashboard Script
# Runs benchmarks, exports data, generates graphs, starts server, and opens browser
#
# Usage:
#   .\scripts\run_benchmarks_and_dashboard.ps1           # Full workflow (benchmarks + dashboard)
#   .\scripts\run_benchmarks_and_dashboard.ps1 -SkipBench   # Skip benchmarks, just regenerate + serve
#   .\scripts\run_benchmarks_and_dashboard.ps1 -Port 9000    # Use custom port

param(
    [switch]$SkipBench,      # Skip running cargo bench (use existing data)
    [int]$Port = 8000,       # HTTP server port
    [switch]$NoBrowser       # Don't auto-open browser
)

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "  $Message" -ForegroundColor Cyan
    Write-Host "========================================`n" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Info {
    param([string]$Message)
    Write-Host "ℹ $Message" -ForegroundColor Yellow
}

# Step 1: Run Benchmarks (optional)
if (-not $SkipBench) {
    Write-Step "Step 1/4: Running Benchmarks"
    Write-Info "This may take several minutes depending on your hardware..."
    
    try {
        cargo bench --package astraweave-ecs --package astraweave-ai --package astraweave-physics --package astraweave-math --package astraweave-terrain --package astraweave-input --no-fail-fast 2>&1 | Out-Null
        Write-Success "Benchmarks completed successfully"
    }
    catch {
        Write-Host "⚠️  Some benchmarks may have failed, but continuing..." -ForegroundColor Yellow
    }
} else {
    Write-Info "Skipping benchmark execution (using existing data)"
}

# Step 2: Export Benchmark Data
Write-Step "Step 2/4: Exporting Benchmark Data"

.\scripts\export_benchmark_jsonl.ps1
Write-Success "Benchmark data exported to target/benchmark-data/history.jsonl"

# Step 3: Generate Graphs
Write-Step "Step 3/4: Generating Graphs"

# Get Python executable path
$pythonExe = "C:/Users/pv2br/AppData/Local/Microsoft/WindowsApps/python3.13.exe"
if (-not (Test-Path $pythonExe)) {
    $pythonExe = "python"
}

& $pythonExe scripts/generate_benchmark_graphs.py --input target/benchmark-data/history.jsonl --out-dir gh-pages/graphs
Write-Success "Graphs generated in gh-pages/graphs/"

# Step 4: Start Server and Open Dashboard
Write-Step "Step 4/4: Starting Dashboard Server"

$dashboardPath = "tools/benchmark-dashboard"
$url = "http://localhost:$Port"

Write-Info "Starting HTTP server on port $Port..."
Write-Info "Dashboard will open at: $url"
Write-Host ""
Write-Host "📊 " -ForegroundColor Green -NoNewline
Write-Host "Dashboard Ready!" -ForegroundColor Cyan
Write-Host ""
Write-Host "   URL: " -NoNewline
Write-Host $url -ForegroundColor Yellow
Write-Host ""
Write-Host "Press " -NoNewline
Write-Host "Ctrl+C" -ForegroundColor Red -NoNewline
Write-Host " to stop the server"
Write-Host ""

# Open browser
if (-not $NoBrowser) {
    Start-Sleep -Milliseconds 500
    Start-Process $url
}

# Start HTTP server (blocks until Ctrl+C)
Push-Location $dashboardPath
try {
    & $pythonExe -m http.server $Port
}
finally {
    Pop-Location
}
