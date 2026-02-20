# ═══════════════════════════════════════════════════════════════════════════════════
# Terrain Mutation Sweep — All 22 Shards Sequential
# ═══════════════════════════════════════════════════════════════════════════════════
# Usage: .\scripts\run_terrain_sweep.ps1 [-StartShard 0] [-EndShard 21]
# Results aggregated into C:\temp\terrain_results\

param(
    [int]$StartShard = 0,
    [int]$EndShard = 21,
    [int]$TotalShards = 22,
    [string]$OutBase = "C:\temp\terrain_results"
)

$ErrorActionPreference = "Continue"
$workspace = "c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine"

Set-Location $workspace

# Create output directory
New-Item -ItemType Directory -Force -Path $OutBase | Out-Null

$totalMissed = 0
$totalCaught = 0
$totalTimeout = 0
$totalUnviable = 0
$startTime = Get-Date

Write-Host "═══════════════════════════════════════════════════════════════"
Write-Host "  Terrain Mutation Sweep: Shards $StartShard..$EndShard / $TotalShards"
Write-Host "  Started: $startTime"
Write-Host "═══════════════════════════════════════════════════════════════"

for ($shard = $StartShard; $shard -le $EndShard; $shard++) {
    $shardDir = "$OutBase\shard_$shard"
    $shardStart = Get-Date
    
    Write-Host "`n─── Shard $shard/$TotalShards ($shardStart) ───"
    
    # Clean previous output
    Remove-Item "C:\temp\mutants.out" -Recurse -Force -ErrorAction SilentlyContinue
    
    # Run the shard (--tests includes both lib and integration tests for maximum kill rate)
    # --gitignore true: skip target/ copies (reduces 16GB → ~50MB), enables viable -j 4
    & cargo mutants -p astraweave-terrain --shard "$shard/$TotalShards" --timeout 300 -j 4 --gitignore true -o "C:\temp" -- --tests 2>&1 | Tee-Object -Variable shardOutput
    
    $shardEnd = Get-Date
    $elapsed = $shardEnd - $shardStart
    
    # Save results
    if (Test-Path "C:\temp\mutants.out") {
        Copy-Item "C:\temp\mutants.out" -Destination $shardDir -Recurse -Force
        
        # Count outcomes
        if (Test-Path "C:\temp\mutants.out\outcomes.json") {
            $outcomes = Get-Content "C:\temp\mutants.out\outcomes.json" | ConvertFrom-Json
            $missed = ($outcomes | Where-Object { $_.summary -eq "MissedMutant" }).Count
            $caught = ($outcomes | Where-Object { $_.summary -eq "CaughtMutant" }).Count
            $timeout = ($outcomes | Where-Object { $_.summary -eq "Timeout" }).Count
            $unviable = ($outcomes | Where-Object { $_.summary -eq "Unviable" }).Count
            
            $totalMissed += $missed
            $totalCaught += $caught
            $totalTimeout += $timeout
            $totalUnviable += $unviable
            
            Write-Host "  Caught: $caught | Missed: $missed | Timeout: $timeout | Unviable: $unviable | Time: $($elapsed.ToString('hh\:mm\:ss'))"
        }
    } else {
        Write-Host "  WARNING: No output for shard $shard"
    }
}

$endTime = Get-Date
$totalElapsed = $endTime - $startTime
$totalTested = $totalCaught + $totalMissed + $totalTimeout + $totalUnviable
$killRate = if ($totalTested -gt 0) { [math]::Round(($totalCaught / ($totalCaught + $totalMissed)) * 100, 2) } else { 0 }

Write-Host "`n═══════════════════════════════════════════════════════════════"
Write-Host "  TERRAIN SWEEP COMPLETE"
Write-Host "  Total Time: $($totalElapsed.ToString('hh\:mm\:ss'))"
Write-Host "  Caught: $totalCaught | Missed: $totalMissed | Timeout: $totalTimeout | Unviable: $totalUnviable"
Write-Host "  Kill Rate: $killRate%"
Write-Host "═══════════════════════════════════════════════════════════════"

# Save summary
@{
    crate = "astraweave-terrain"
    shards = "$StartShard..$EndShard/$TotalShards"
    total_tested = $totalTested
    caught = $totalCaught
    missed = $totalMissed
    timeout = $totalTimeout
    unviable = $totalUnviable
    kill_rate = $killRate
    start_time = $startTime.ToString("o")
    end_time = $endTime.ToString("o")
    elapsed = $totalElapsed.ToString('hh\:mm\:ss')
} | ConvertTo-Json | Set-Content "$OutBase\summary.json"

Write-Host "Summary saved to $OutBase\summary.json"
