#!/usr/bin/env pwsh
# Batch mutation testing for all remaining P0 crates
# Uses --in-place and --lib for speed, sharding for large crates

$ErrorActionPreference = "Continue"
$resultsDir = "mutation_results"
New-Item -ItemType Directory -Force -Path $resultsDir | Out-Null

function Run-MutantTest {
    param(
        [string]$Crate,
        [string]$Shard,  # e.g., "1/5" or "" for full
        [int]$Timeout = 30
    )
    
    $start = Get-Date
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "CRATE: $Crate $(if($Shard){"(shard $Shard)"}else{"(full)"})" -ForegroundColor Cyan
    Write-Host "Started: $start" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    
    # Clean previous mutants.out
    Remove-Item -Recurse -Force "mutants.out" -ErrorAction SilentlyContinue
    
    $args = @("mutants", "-p", $Crate, "--in-place", "--timeout", $Timeout)
    if ($Shard) {
        $args += "--shard"
        $args += $Shard
    }
    $args += "--"
    $args += "--lib"
    
    Write-Host "Running: cargo $($args -join ' ')" -ForegroundColor Yellow
    & cargo @args 2>&1 | Tee-Object -Variable output
    
    $elapsed = (Get-Date) - $start
    Write-Host "`nElapsed: $($elapsed.ToString('hh\:mm\:ss'))" -ForegroundColor Green
    
    # Save results
    $crateShort = $Crate -replace "astraweave-", ""
    $resultPrefix = "$resultsDir/$crateShort"
    
    foreach ($f in @("caught.txt", "missed.txt", "timeout.txt", "unviable.txt")) {
        $src = "mutants.out/$f"
        if (Test-Path $src) {
            Copy-Item $src "$resultPrefix.$f" -Force
            $count = (Get-Content $src | Measure-Object -Line).Lines
            Write-Host "${f}: $count" -ForegroundColor $(if($f -eq "missed.txt" -and $count -gt 0){"Red"}else{"Green"})
        }
    }
    
    # Check source is clean
    $dirty = Select-String -Path "$Crate\src\*.rs" -Pattern "changed by cargo-mutants" -ErrorAction SilentlyContinue
    if ($dirty) {
        Write-Host "WARNING: Mutant artifacts found in source!" -ForegroundColor Red
        $dirty | ForEach-Object { Write-Host $_.Line -ForegroundColor Red }
    }
    
    Write-Host "Done: $Crate" -ForegroundColor Green
}

# Order: smallest first, then by importance
# Audio: 131 mutants → FULL run
Run-MutantTest -Crate "astraweave-audio" -Timeout 30

# ECS: 498 mutants → shard 1/3  
Run-MutantTest -Crate "astraweave-ecs" -Shard "1/3" -Timeout 60

# Core: 762 mutants → shard 1/5
Run-MutantTest -Crate "astraweave-core" -Shard "1/5" -Timeout 30

# Gameplay: 615 mutants → shard 1/4
Run-MutantTest -Crate "astraweave-gameplay" -Shard "1/4" -Timeout 30

# Prompts: 791 mutants → shard 1/5
Run-MutantTest -Crate "astraweave-prompts" -Shard "1/5" -Timeout 30

# Physics: 2126 mutants → shard 1/10 (already have 12 caught from partial)
Run-MutantTest -Crate "astraweave-physics" -Shard "1/10" -Timeout 30

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "ALL MUTATION RUNS COMPLETE" -ForegroundColor Cyan
Write-Host "Results saved in: $resultsDir/" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Summary
Write-Host "`n=== SUMMARY ===" -ForegroundColor Yellow
Get-ChildItem "$resultsDir/*.caught.txt" | ForEach-Object {
    $crate = $_.BaseName -replace "\.caught$", ""
    $caught = (Get-Content $_.FullName | Measure-Object -Line).Lines
    $missedFile = $_.FullName -replace "\.caught\.txt$", ".missed.txt"
    $missed = if (Test-Path $missedFile) { (Get-Content $missedFile | Measure-Object -Line).Lines } else { 0 }
    $total = $caught + $missed
    $rate = if ($total -gt 0) { [math]::Round(($caught / $total) * 100, 1) } else { "N/A" }
    Write-Host "$crate : $caught caught, $missed missed, kill rate: $rate%"
}
