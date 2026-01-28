# Phase 10A: P0 Tier Batch Mutation Testing Script
# Automates testing of all 12 P0 crates with optimal settings

$ErrorActionPreference = "Continue"

# P0 Crates ordered by size (smallest first to handle disk space efficiently)
$P0Crates = @(
    @{Name="astraweave-audio"; Mutants=117; Timeout=90},
    @{Name="astraweave-scene"; Mutants=120; Timeout=90},
    @{Name="astraweave-asset"; Mutants=150; Timeout=90},
    @{Name="astraweave-gameplay"; Mutants=200; Timeout=90},
    @{Name="astraweave-ui"; Mutants=200; Timeout=90},
    @{Name="astraweave-core"; Mutants=250; Timeout=90},
    @{Name="astraweave-terrain"; Mutants=250; Timeout=90},
    @{Name="astraweave-ecs"; Mutants=300; Timeout=90},
    @{Name="astraweave-physics"; Mutants=350; Timeout=120},
    @{Name="astraweave-render"; Mutants=400; Timeout=120}
)

# Already completed
$CompletedCrates = @("astraweave-math", "astraweave-nav")

Write-Host "=== PHASE 10A: P0 TIER BATCH TESTING ===" -ForegroundColor Cyan
Write-Host "Completed: $($CompletedCrates -join ', ')" -ForegroundColor Green
Write-Host "Remaining: $($P0Crates.Count) crates"
Write-Host ""

foreach ($crate in $P0Crates) {
    $crateName = $crate.Name
    $timeout = $crate.Timeout
    $estimatedMutants = $crate.Mutants
    
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host "Testing: $crateName" -ForegroundColor Cyan
    Write-Host "Estimated mutants: $estimatedMutants"
    Write-Host "Timeout: $timeout seconds"
    Write-Host "Settings: --jobs 4 --copy-target=false"
    Write-Host "========================================" -ForegroundColor Yellow
    Write-Host ""
    
    # Clean previous results
    if (Test-Path "mutants.out") {
        Write-Host "Cleaning previous results..." -ForegroundColor Yellow
        Remove-Item "mutants.out" -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    # Run mutation test
    $startTime = Get-Date
    Write-Host "Starting test at $(Get-Date -Format 'HH:mm:ss')..." -ForegroundColor Green
    
    $result = cargo mutants --package $crateName --timeout $timeout --jobs 4 --copy-target=false 2>&1
    $exitCode = $LASTEXITCODE
    
    $endTime = Get-Date
    $duration = $endTime - $startTime
    
    Write-Host ""
    Write-Host "Test completed in $($duration.ToString('hh\:mm\:ss'))" -ForegroundColor $(if ($exitCode -eq 0) { "Green" } else { "Red" })
    Write-Host "Exit code: $exitCode"
    Write-Host ""
    
    # Parse results
    if (Test-Path "mutants.out/outcomes.json") {
        Write-Host "Parsing results..." -ForegroundColor Cyan
        $json = Get-Content "mutants.out/outcomes.json" -Raw | ConvertFrom-Json
        $caught = ($json | Where-Object {$_.scenario.outcome -eq "Caught"}).Count
        $missed = ($json | Where-Object {$_.scenario.outcome -eq "Missed"}).Count
        $timeout = ($json | Where-Object {$_.scenario.outcome -eq "Timeout"}).Count
        $unviable = ($json | Where-Object {$_.scenario.outcome -eq "Unviable"}).Count
        $viable = $caught + $missed
        $score = if ($viable -gt 0) { [math]::Round(100.0 * $caught / $viable, 2) } else { 0 }
        
        Write-Host "=== RESULTS ===" -ForegroundColor Green
        Write-Host "Caught: $caught | Missed: $missed | Timeout: $timeout | Unviable: $unviable"
        Write-Host "Mutation Score: $score% (from $viable viable mutants)"
        Write-Host ""
        
        # Save summary
        $summary = @{
            Crate = $crateName
            MutationScore = $score
            Caught = $caught
            Missed = $missed
            Timeout = $timeout
            Unviable = $unviable
            Viable = $viable
            Duration = $duration.ToString('hh\:mm\:ss')
            Timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
        }
        
        $summaryPath = "docs/journey/phases/PHASE_10A_${crateName}_RESULTS.json"
        $summary | ConvertTo-Json | Out-File $summaryPath
        Write-Host "Summary saved to: $summaryPath" -ForegroundColor Green
        
    } else {
        Write-Host "WARNING: No results found - test may have failed" -ForegroundColor Red
    }
    
    Write-Host ""
    Write-Host "Pausing for 10 seconds before next test..." -ForegroundColor Yellow
    Start-Sleep -Seconds 10
}

Write-Host ""
Write-Host "=== ALL P0 TESTS COMPLETE ===" -ForegroundColor Green
Write-Host "Check docs/journey/phases/ for individual results"
