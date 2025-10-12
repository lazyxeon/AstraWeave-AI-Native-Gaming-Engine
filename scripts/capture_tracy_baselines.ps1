# Week 8 Tracy Baseline Capture Script
# Automates running profiling_demo at 200, 500, 1000 entities for Tracy profiling

param(
    [switch]$SkipBuild,
    [int]$EntityCount = 0,  # 0 = run all configs
    [string]$TracyPath = "C:\Tools\Tracy\Tracy.exe"
)

# Color output helpers
function Write-Success { param($msg) Write-Host "✅ $msg" -ForegroundColor Green }
function Write-Info { param($msg) Write-Host "ℹ️  $msg" -ForegroundColor Cyan }
function Write-Warning { param($msg) Write-Host "⚠️  $msg" -ForegroundColor Yellow }
function Write-Error { param($msg) Write-Host "❌ $msg" -ForegroundColor Red }

# Configuration
$configs = @(
    @{Name = "Low Load"; Entities = 200; Description = "Baseline performance, minimal overhead"},
    @{Name = "Medium Load"; Entities = 500; Description = "Target capacity (60 FPS goal)"},
    @{Name = "High Load"; Entities = 1000; Description = "Stress test, identify scalability limits"}
)

# Header
Write-Host "`n🚀 Week 8 Tracy Baseline Capture" -ForegroundColor Magenta
Write-Host "================================`n" -ForegroundColor Magenta

# Step 1: Check Tracy installation
Write-Info "Checking Tracy installation..."
if (-Not (Test-Path $TracyPath)) {
    Write-Warning "Tracy not found at: $TracyPath"
    Write-Host "`n📥 Download Tracy from: https://github.com/wolfpld/tracy/releases/latest" -ForegroundColor Yellow
    Write-Host "   Extract to C:\Tools\Tracy\ (or update script with -TracyPath)`n" -ForegroundColor Yellow
    
    $continue = Read-Host "Continue without Tracy server? (y/n)"
    if ($continue -ne "y") {
        Write-Error "Aborted. Install Tracy and re-run this script."
        exit 1
    }
} else {
    Write-Success "Tracy found: $TracyPath"
    Write-Info "Please start Tracy.exe manually before running profiling_demo"
    Write-Info "Tracy connection: localhost (default)"
}

# Step 2: Build profiling_demo
if (-Not $SkipBuild) {
    Write-Info "Building profiling_demo in release mode with profiling features..."
    $buildStart = Get-Date
    
    cargo build -p profiling_demo --features profiling --release 2>&1 | Out-Null
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Build failed. Check compilation errors above."
        exit 1
    }
    
    $buildTime = (Get-Date) - $buildStart
    Write-Success "Build complete in $([math]::Round($buildTime.TotalSeconds, 2))s"
} else {
    Write-Info "Skipping build (--SkipBuild flag)"
}

# Step 3: Run profiling configurations
Write-Host "`n📊 Tracy Baseline Capture Workflow" -ForegroundColor Cyan
Write-Host "===================================`n" -ForegroundColor Cyan

foreach ($config in $configs) {
    # Skip if specific entity count requested
    if ($EntityCount -ne 0 -and $config.Entities -ne $EntityCount) {
        continue
    }
    
    Write-Host "`n🎯 Configuration: $($config.Name) ($($config.Entities) entities)" -ForegroundColor Yellow
    Write-Host "   $($config.Description)" -ForegroundColor Gray
    
    Write-Info "Starting profiling_demo..."
    Write-Host "   Target: Capture 1000+ frames (~16-30 seconds @ 60 FPS)" -ForegroundColor Gray
    Write-Host "   Tracy: Monitor connection, watch frame time graph" -ForegroundColor Gray
    Write-Host "   Save: File > Save Trace > baseline_$($config.Entities).tracy`n" -ForegroundColor Gray
    
    # Prompt user to ensure Tracy is ready
    Write-Host "   Press Enter when ready to start (Tracy connected)..." -ForegroundColor Green
    Read-Host
    
    # Run profiling_demo
    $cmd = "cargo run -p profiling_demo --features profiling --release -- --entities $($config.Entities)"
    Write-Info "Running: $cmd"
    
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -ne 0) {
        Write-Error "profiling_demo exited with error code $LASTEXITCODE"
        $continue = Read-Host "Continue to next configuration? (y/n)"
        if ($continue -ne "y") {
            exit 1
        }
    } else {
        Write-Success "profiling_demo completed successfully"
    }
    
    # Reminder to save trace
    Write-Host "`n   ⚠️  REMINDER: Save Tracy trace now!" -ForegroundColor Yellow
    Write-Host "      1. In Tracy: File > Save Trace" -ForegroundColor Gray
    Write-Host "      2. Filename: baseline_$($config.Entities).tracy" -ForegroundColor Gray
    Write-Host "      3. Location: AstraWeave root directory`n" -ForegroundColor Gray
    
    Read-Host "   Press Enter when trace saved (or skip if not ready)"
}

# Step 4: Summary
Write-Host "`n✅ Tracy Baseline Capture Complete!" -ForegroundColor Green
Write-Host "===================================`n" -ForegroundColor Green

Write-Info "Expected outputs:"
Write-Host "   - baseline_200.tracy  (Low load baseline)" -ForegroundColor Gray
Write-Host "   - baseline_500.tracy  (Target capacity)" -ForegroundColor Gray
Write-Host "   - baseline_1000.tracy (Stress test)`n" -ForegroundColor Gray

Write-Info "Next steps:"
Write-Host "   1. Open each .tracy file in Tracy server" -ForegroundColor Gray
Write-Host "   2. Analyze Statistics view (sort by Self time)" -ForegroundColor Gray
Write-Host "   3. Record top 10 hotspots (>5% frame time)" -ForegroundColor Gray
Write-Host "   4. Create PROFILING_BASELINE_WEEK_8.md report" -ForegroundColor Gray
Write-Host "   5. Define Week 8 optimization priorities`n" -ForegroundColor Gray

Write-Success "Week 8 Day 1 ready to proceed to analysis phase!"

# Usage examples
Write-Host "`n📖 Usage Examples:" -ForegroundColor Cyan
Write-Host "   .\scripts\capture_tracy_baselines.ps1                   # Run all 3 configs" -ForegroundColor Gray
Write-Host "   .\scripts\capture_tracy_baselines.ps1 -EntityCount 500  # Run 500-entity config only" -ForegroundColor Gray
Write-Host "   .\scripts\capture_tracy_baselines.ps1 -SkipBuild        # Skip rebuild (already built)" -ForegroundColor Gray
Write-Host "   .\scripts\capture_tracy_baselines.ps1 -TracyPath 'D:\Tracy\Tracy.exe'  # Custom Tracy path`n" -ForegroundColor Gray
