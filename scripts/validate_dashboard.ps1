# Validate Benchmark Dashboard Data & Configuration
# Quick diagnostic script to verify dashboard is working correctly

param([switch]$Verbose)

Write-Host "`n═══════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "   Benchmark Dashboard Validation" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════`n" -ForegroundColor Cyan

# 1. Check if benchmark data exists
Write-Host "1. Checking benchmark data files..." -ForegroundColor Yellow

$historyFile = "tools\benchmark-dashboard\benchmark-data\history.jsonl"
$metadataFile = "tools\benchmark-dashboard\benchmark-data\metadata.json"

if (Test-Path $historyFile) {
    Write-Host "   ✅ history.jsonl found" -ForegroundColor Green
    
    $lines = Get-Content $historyFile
    $totalEntries = $lines.Count
    
    Write-Host "   📊 Total entries: $totalEntries" -ForegroundColor White
    
    # Parse and analyze data
    try {
        $data = $lines | ForEach-Object { $_ | ConvertFrom-Json }
        
        # Count unique benchmarks
        $uniqueBenchmarks = $data | Select-Object -Property benchmark_name -Unique
        Write-Host "   📊 Unique benchmarks: $($uniqueBenchmarks.Count)" -ForegroundColor White
        
        # Count timestamps (runs)
        $timestamps = $data | Group-Object timestamp
        Write-Host "   📊 Benchmark runs: $($timestamps.Count)" -ForegroundColor White
        
        # Count by system
        $byCrate = $data | Group-Object crate | Sort-Object Count -Descending
        Write-Host "`n   Top crates:" -ForegroundColor White
        $byCrate | Select-Object -First 5 | ForEach-Object {
            Write-Host "      • $($_.Name): $($_.Count) entries" -ForegroundColor Gray
        }
        
        # Check for duplicates (same benchmark + timestamp)
        $duplicateCheck = $data | Group-Object { "$($_.benchmark_name)_$($_.timestamp)" } | Where-Object { $_.Count -gt 1 }
        if ($duplicateCheck) {
            Write-Host "`n   ⚠️  WARNING: $($duplicateCheck.Count) duplicate entries found!" -ForegroundColor Yellow
            if ($Verbose) {
                $duplicateCheck | Select-Object -First 3 | ForEach-Object {
                    Write-Host "      • $($_.Name)" -ForegroundColor Gray
                }
            }
        }
        else {
            Write-Host "`n   ✅ No duplicates detected" -ForegroundColor Green
        }
        
        # Check data freshness
        $latestTimestamp = ($data | Select-Object -Property timestamp -First 1).timestamp
        $newestEntry = [DateTime]::Parse($latestTimestamp)
        $age = (Get-Date) - $newestEntry
        
        if ($age.TotalHours -lt 24) {
            Write-Host "   ✅ Data is fresh (last run: $([math]::Round($age.TotalHours, 1)) hours ago)" -ForegroundColor Green
        }
        elseif ($age.TotalDays -lt 7) {
            Write-Host "   ⚠️  Data is $([math]::Round($age.TotalDays, 1)) days old" -ForegroundColor Yellow
        }
        else {
            Write-Host "   ⚠️  Data is stale ($([math]::Round($age.TotalDays, 1)) days old)" -ForegroundColor Yellow
        }
    }
    catch {
        Write-Host "   ❌ Error parsing JSONL data: $_" -ForegroundColor Red
    }
}
else {
    Write-Host "   ❌ history.jsonl NOT found" -ForegroundColor Red
    Write-Host "   Run: .\scripts\export_benchmark_jsonl.ps1" -ForegroundColor Yellow
}

if (Test-Path $metadataFile) {
    Write-Host "`n   ✅ metadata.json found" -ForegroundColor Green
}
else {
    Write-Host "`n   ⚠️  metadata.json NOT found" -ForegroundColor Yellow
}

# 2. Check Criterion benchmark results
Write-Host "`n2. Checking Criterion benchmark results..." -ForegroundColor Yellow

if (Test-Path "target\criterion") {
    $criterionBenchmarks = Get-ChildItem "target\criterion" -Recurse -Filter "estimates.json" -File | 
                          Where-Object { $_.DirectoryName -match '[/\\]base$' }
    
    Write-Host "   ✅ Criterion directory found" -ForegroundColor Green
    Write-Host "   📊 Available benchmarks: $($criterionBenchmarks.Count)" -ForegroundColor White
    
    if ($criterionBenchmarks.Count -eq 0) {
        Write-Host "   ⚠️  No benchmark results found" -ForegroundColor Yellow
        Write-Host "   Run: cargo bench" -ForegroundColor Yellow
    }
}
else {
    Write-Host "   ❌ Criterion directory NOT found" -ForegroundColor Red
    Write-Host "   Run: cargo bench" -ForegroundColor Yellow
}

# 3. Check dashboard files
Write-Host "`n3. Checking dashboard files..." -ForegroundColor Yellow

$requiredFiles = @(
    "tools\benchmark-dashboard\index.html",
    "tools\benchmark-dashboard\dashboard.js"
)

$allFilesPresent = $true
foreach ($file in $requiredFiles) {
    if (Test-Path $file) {
        Write-Host "   ✅ $(Split-Path $file -Leaf)" -ForegroundColor Green
    }
    else {
        Write-Host "   ❌ $(Split-Path $file -Leaf) NOT found" -ForegroundColor Red
        $allFilesPresent = $false
    }
}

# 4. Check for common issues
Write-Host "`n4. Checking for common issues..." -ForegroundColor Yellow

# Check if port 8000 is in use
try {
    $portInUse = Get-NetTCPConnection -LocalPort 8000 -ErrorAction SilentlyContinue
    if ($portInUse) {
        Write-Host "   ⚠️  Port 8000 is in use (dashboard may already be running)" -ForegroundColor Yellow
    }
    else {
        Write-Host "   ✅ Port 8000 is available" -ForegroundColor Green
    }
}
catch {
    Write-Host "   ⚠️  Could not check port 8000" -ForegroundColor Yellow
}

# 5. Summary and recommendations
Write-Host "`n═══════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "   Summary & Recommendations" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════`n" -ForegroundColor Cyan

if ($allFilesPresent -and (Test-Path $historyFile)) {
    Write-Host "✅ Dashboard is ready to use!" -ForegroundColor Green
    Write-Host "`nTo launch:" -ForegroundColor Yellow
    Write-Host "  .\Launch-Benchmark-Dashboard.bat" -ForegroundColor Cyan
    Write-Host "  OR" -ForegroundColor Yellow
    Write-Host "  .\scripts\run_benchmark_dashboard.ps1 -SkipBench`n" -ForegroundColor Cyan
}
else {
    Write-Host "⚠️  Dashboard has issues that need attention" -ForegroundColor Yellow
    Write-Host "`nRecommended actions:" -ForegroundColor Yellow
    
    if (-not (Test-Path $historyFile)) {
        Write-Host "  1. Run benchmarks: cargo bench" -ForegroundColor Cyan
        Write-Host "  2. Export data: .\scripts\export_benchmark_jsonl.ps1`n" -ForegroundColor Cyan
    }
}

# Optional: Show sample data
if ($Verbose -and (Test-Path $historyFile)) {
    Write-Host "`n═══════════════════════════════════════════════════════" -ForegroundColor Cyan
    Write-Host "   Sample Data (First 5 Benchmarks)" -ForegroundColor Cyan
    Write-Host "═══════════════════════════════════════════════════════`n" -ForegroundColor Cyan
    
    Get-Content $historyFile | Select-Object -First 5 | ForEach-Object {
        $entry = $_ | ConvertFrom-Json
        Write-Host "  • $($entry.display_name)" -ForegroundColor White
        Write-Host "    Value: $($entry.value) $($entry.unit) | Crate: $($entry.crate)" -ForegroundColor Gray
    }
    Write-Host ""
}
