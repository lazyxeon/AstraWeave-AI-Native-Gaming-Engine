# Run All Working Benchmarks
# Automatically discovers and runs all compilable benchmarks across the workspace
# Skips any that fail to compile

param(
    [switch]$DryRun,
    [switch]$Verbose,
    [int]$TimeoutMinutes = 60
)

$ErrorActionPreference = "Continue"

function Write-Status {
    param([string]$Message, [string]$Type = "INFO")
    $colors = @{
        "INFO" = "Cyan"
        "SUCCESS" = "Green"
        "ERROR" = "Red"
        "WARN" = "Yellow"
        "SKIP" = "DarkGray"
    }
    $symbols = @{
        "INFO" = "ℹ️"
        "SUCCESS" = "✅"
        "ERROR" = "❌"
        "WARN" = "⚠️"
        "SKIP" = "⏭️"
    }
    Write-Host "$($symbols[$Type]) $Message" -ForegroundColor $colors[$Type]
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Magenta
Write-Host "   AstraWeave - Comprehensive Benchmark Runner" -ForegroundColor Magenta
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Magenta
Write-Host ""

# Step 1: Discover all crates with benchmarks
Write-Status "Discovering benchmark crates..." "INFO"

$benchmarkCrates = @()

# Search entire workspace for Cargo.toml files
Get-ChildItem -Path . -Recurse -Filter "Cargo.toml" -ErrorAction SilentlyContinue | Where-Object {
    # Skip target directory
    $_.FullName -notmatch '\\target\\'
} | ForEach-Object {
    $content = Get-Content $_.FullName -Raw
    if ($content -match '\[\[bench\]\]') {
        # Extract crate name from first 'name =' line
        $nameMatch = $content | Select-String -Pattern '(?m)^name\s*=\s*"([^"]+)"' -AllMatches
        if ($nameMatch.Matches.Count -gt 0) {
            $crateName = $nameMatch.Matches[0].Groups[1].Value
            $benchCount = ([regex]::Matches($content, '\[\[bench\]\]')).Count
            
            $benchmarkCrates += [PSCustomObject]@{
                Name = $crateName
                Path = $_.DirectoryName
                BenchCount = $benchCount
                Status = "Unknown"
            }
        }
    }
}

Write-Status "Found $($benchmarkCrates.Count) crates with $($benchmarkCrates | Measure-Object -Property BenchCount -Sum | Select-Object -ExpandProperty Sum) benchmark suites" "SUCCESS"

if ($Verbose) {
    Write-Host "`nDiscovered crates:" -ForegroundColor Yellow
    $benchmarkCrates | Sort-Object BenchCount -Descending | ForEach-Object {
        Write-Host "  • $($_.Name) ($($_.BenchCount) benches)" -ForegroundColor Gray
    }
    Write-Host ""
}

# Step 2: Test which benchmarks can compile
Write-Status "Testing benchmark compilation (this may take a few minutes)..." "INFO"
Write-Host ""

$workingCrates = @()
$failedCrates = @()
$tested = 0

foreach ($crate in $benchmarkCrates) {
    $tested++
    Write-Host "[$tested/$($benchmarkCrates.Count)] Testing $($crate.Name)... " -NoNewline -ForegroundColor White
    
    if ($DryRun) {
        Write-Host "SKIPPED (dry run)" -ForegroundColor DarkGray
        continue
    }
    
    # Try to compile benchmarks (don't run them)
    $output = cargo bench -p $crate.Name --no-run 2>&1 | Out-String
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ OK" -ForegroundColor Green
        $crate.Status = "Working"
        $workingCrates += $crate.Name
    }
    else {
        # Check if it's a feature flag issue
        if ($output -match "unresolved import|cannot find") {
            Write-Host "⚠️  COMPILE ERROR (likely missing feature)" -ForegroundColor Yellow
            if ($Verbose) {
                $output | Select-String "error\[" | Select-Object -First 2 | ForEach-Object {
                    Write-Host "     $($_.Line)" -ForegroundColor DarkYellow
                }
            }
        }
        else {
            Write-Host "❌ FAILED" -ForegroundColor Red
        }
        $crate.Status = "Failed"
        $failedCrates += $crate.Name
    }
}

Write-Host ""
Write-Status "Compilation test complete: $($workingCrates.Count) working, $($failedCrates.Count) failed" "SUCCESS"

if ($failedCrates.Count -gt 0 -and $Verbose) {
    Write-Host "`nFailed crates (will be skipped):" -ForegroundColor Yellow
    $failedCrates | ForEach-Object {
        Write-Host "  • $_" -ForegroundColor Red
    }
    Write-Host ""
}

if ($DryRun) {
    Write-Status "Dry run complete. Would run $($workingCrates.Count) benchmark crates." "INFO"
    exit 0
}

if ($workingCrates.Count -eq 0) {
    Write-Status "No working benchmarks found!" "ERROR"
    exit 1
}

# Step 3: Run all working benchmarks
Write-Host ""
Write-Status "Running benchmarks for $($workingCrates.Count) crates..." "INFO"
Write-Host "This may take $TimeoutMinutes+ minutes depending on benchmark complexity" -ForegroundColor Yellow
Write-Host ""

$startTime = Get-Date
$benchArgs = @("bench", "--no-fail-fast")
foreach ($pkg in $workingCrates) {
    $benchArgs += "--package"
    $benchArgs += $pkg
}

Write-Host "Command: cargo $($benchArgs -join ' ')" -ForegroundColor DarkGray
Write-Host ""

try {
    # Run benchmarks with timeout
    $job = Start-Job -ScriptBlock {
        param($Args)
        & cargo @Args
    } -ArgumentList (,$benchArgs)
    
    $job | Wait-Job -Timeout ($TimeoutMinutes * 60) | Out-Null
    
    if ($job.State -eq "Running") {
        Write-Status "Benchmark run exceeded $TimeoutMinutes minute timeout!" "WARN"
        Stop-Job $job
        Remove-Job $job
    }
    else {
        $output = Receive-Job $job
        Remove-Job $job
        
        if ($LASTEXITCODE -eq 0) {
            Write-Status "All benchmarks completed successfully!" "SUCCESS"
        }
        else {
            Write-Status "Some benchmarks failed, but continuing..." "WARN"
        }
    }
}
catch {
    Write-Status "Error running benchmarks: $_" "ERROR"
}

$elapsed = (Get-Date) - $startTime
Write-Host ""
Write-Status "Benchmark run completed in $([math]::Round($elapsed.TotalMinutes, 1)) minutes" "SUCCESS"

# Step 4: Export results
Write-Host ""
Write-Status "Exporting benchmark results to JSONL..." "INFO"

try {
    & ".\scripts\export_benchmark_jsonl.ps1" -Verbose:$Verbose
    
    if ($LASTEXITCODE -eq 0) {
        Write-Status "Export complete!" "SUCCESS"
    }
    else {
        Write-Status "Export failed!" "ERROR"
    }
}
catch {
    Write-Status "Export error: $_" "ERROR"
}

# Summary
Write-Host ""
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host "   Benchmark Run Complete" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════" -ForegroundColor Green
Write-Host ""
Write-Host "  Crates tested: $($benchmarkCrates.Count)" -ForegroundColor White
Write-Host "  Working: $($workingCrates.Count)" -ForegroundColor Green
Write-Host "  Failed: $($failedCrates.Count)" -ForegroundColor Red
Write-Host "  Runtime: $([math]::Round($elapsed.TotalMinutes, 1)) minutes" -ForegroundColor White
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. View results: .\scripts\validate_dashboard.ps1" -ForegroundColor Cyan
Write-Host "  2. Launch dashboard: .\Launch-Benchmark-Dashboard.bat" -ForegroundColor Cyan
Write-Host ""
