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
    Write-Step "Running comprehensive benchmarks across all systems (this may take 15-25 minutes)..." "RUNNING"
    Write-Host ""
    
    # COMPREHENSIVE BENCHMARK SUITE - Covering all 20+ crates with benchmarks
    # Organized by system for clarity
    
    $benchPackages = @(
        # === CORE ENGINE ===
        "astraweave-ecs",              # ECS archetype system, entity spawn/despawn, query performance
        "astraweave-core",             # Core data structures, world management
        "astraweave-stress-test",      # High-load scenarios, network/persistence/ECS stress
        
        # === AI SYSTEMS ===
        "astraweave-ai",               # AI orchestrator, core loop, planning (5 benchmarks)
        "astraweave-behavior",         # Behavior trees, GOAP, utility AI
        "astraweave-context",          # Context management for AI decisions
        "astraweave-memory",           # AI memory systems
        "astraweave-llm",              # LLM integration (3 benchmarks)
        "astraweave-llm-eval",         # LLM evaluation metrics
        "astraweave-prompts",          # Prompt engineering systems
        "astraweave-persona",          # NPC personality simulation
        "astraweave-rag",              # RAG (Retrieval-Augmented Generation)
        
        # === PHYSICS & NAVIGATION ===
        "astraweave-physics",          # Rapier3D wrapper, collision, character controller (4 benchmarks)
        "astraweave-nav",              # Navmesh, pathfinding (crates/astraweave-nav)
        
        # === RENDERING ===
        "astraweave-render",           # wgpu rendering, mesh optimization (2 benchmarks)
        
        # === MATH & PERFORMANCE ===
        "astraweave-math",             # SIMD vector/matrix ops (4 benchmarks)
        
        # === WORLD & CONTENT ===
        "astraweave-terrain",          # Terrain generation, voxel/polygon hybrid
        "astraweave-pcg",              # Procedural content generation
        "astraweave-weaving",          # Fate-weaving system (2 benchmarks)
        
        # === PERSISTENCE & NETWORKING ===
        "astraweave-persistence-ecs",  # Save/load systems (2 benchmarks)
        "astraweave-net-ecs",          # Networking with ECS integration
        "aw-save",                     # Save system (persistence/aw-save)
        
        # === INPUT & AUDIO ===
        "astraweave-input",            # Input binding system
        "astraweave-audio",            # Spatial audio
        
        # === UI & TOOLS ===
        "astraweave-ui",               # UI systems
        "astraweave-sdk",              # SDK C ABI exports
        "astract",                     # Gizmo/widget system (crates/astract)
        "aw_editor",                   # Editor gizmo performance (tools/aw_editor)
        "aw_build"                     # Build system hash performance
    )
    
    Write-Step "Running benchmarks for $($benchPackages.Count) packages..." "INFO"
    Write-Host "  Packages: $($benchPackages -join ', ')" -ForegroundColor DarkGray
    Write-Host ""
    
    $benchArgs = @("bench", "--no-fail-fast")
    foreach ($pkg in $benchPackages) {
        $benchArgs += "--package"
        $benchArgs += $pkg
    }
    
    try {
        Write-Host "Executing: cargo $($benchArgs -join ' ')" -ForegroundColor DarkGray
        Write-Host ""
        & cargo @benchArgs
        
        if ($LASTEXITCODE -eq 0) {
            Write-Step "Benchmarks completed successfully" "SUCCESS"
        } else {
            Write-Step "Some benchmarks failed (exit code $LASTEXITCODE), but continuing with available data..." "ERROR"
        }
    }
    catch {
        Write-Step "Benchmark run encountered errors: $_ (continuing with existing data...)" "ERROR"
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

# Use Start-Process instead of Start-Job for better reliability
$dashboardPath = (Resolve-Path $dashboardDir).Path
$serverProcess = Start-Process -FilePath "python" -ArgumentList "-m", "http.server", $Port -WorkingDirectory $dashboardPath -WindowStyle Minimized -PassThru

# Wait a bit longer and check multiple times for server to start
$maxAttempts = 10
$attempt = 0
$serverStarted = $false

while ($attempt -lt $maxAttempts -and -not $serverStarted) {
    $attempt++
    Start-Sleep -Seconds 1
    
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:$Port" -TimeoutSec 2 -UseBasicParsing -ErrorAction Stop
        $serverStarted = $true
        Write-Step "HTTP server started successfully (attempt $attempt/$maxAttempts)" "SUCCESS"
    }
    catch {
        if ($attempt -eq $maxAttempts) {
            Write-Step "Failed to verify HTTP server after $maxAttempts attempts: $_" "ERROR"
            Write-Step "Server process may still be starting. Check manually at http://localhost:$Port" "ERROR"
            
            # Try to get more details
            if ($serverProcess -and -not $serverProcess.HasExited) {
                Write-Host "Server process is running (PID: $($serverProcess.Id))" -ForegroundColor Yellow
                Write-Host "Try accessing http://localhost:$Port manually in your browser" -ForegroundColor Yellow
            }
            else {
                Write-Host "Server process exited unexpectedly" -ForegroundColor Red
                exit 1
            }
        }
    }
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
        
        # Check if process is still running
        if ($serverProcess.HasExited) {
            Write-Step "Server stopped unexpectedly" "ERROR"
            break
        }
    }
}
finally {
    Write-Host ""
    Write-Step "Shutting down server..." "RUNNING"
    if ($serverProcess -and -not $serverProcess.HasExited) {
        Stop-Process -Id $serverProcess.Id -Force -ErrorAction SilentlyContinue
    }
    Write-Step "Server stopped" "SUCCESS"
}
