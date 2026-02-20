# ═══════════════════════════════════════════════════════════════════════════════════
# Render Mutation Sweep — All 20 Shards Sequential
# ═══════════════════════════════════════════════════════════════════════════════════
# Usage: .\scripts\run_render_sweep.ps1 [-StartShard 0] [-EndShard 19]
# Results aggregated into C:\temp\render_results\

param(
    [int]$StartShard = 0,
    [int]$EndShard = 19,
    [int]$TotalShards = 20,
    [string]$OutBase = "C:\temp\render_results"
)

$ErrorActionPreference = "Continue"
$workspace = "c:\Users\pv2br\AstraWeave-AI-Native-Gaming-Engine"
$features = "textures,nanite,ssao,bloom,skinning-gpu"

Set-Location $workspace

New-Item -ItemType Directory -Force -Path $OutBase | Out-Null

$totalMissed = 0
$totalCaught = 0
$totalTimeout = 0
$totalUnviable = 0
$startTime = Get-Date

Write-Host "═══════════════════════════════════════════════════════════════"
Write-Host "  Render Mutation Sweep: Shards $StartShard..$EndShard / $TotalShards"
Write-Host "  Features: $features"
Write-Host "  Started: $startTime"
Write-Host "═══════════════════════════════════════════════════════════════"

for ($shard = $StartShard; $shard -le $EndShard; $shard++) {
    $shardDir = "$OutBase\shard_$shard"
    $shardStart = Get-Date

    Write-Host "`n─── Shard $shard/$TotalShards ($shardStart) ───"

    Remove-Item "C:\temp\mutants.out" -Recurse -Force -ErrorAction SilentlyContinue

    & cargo mutants -p astraweave-render --shard "$shard/$TotalShards" --timeout 300 -j 1 -o "C:\temp" --features $features -- --lib 2>&1 | Tee-Object -Variable shardOutput

    $shardEnd = Get-Date
    $elapsed = $shardEnd - $shardStart

    if (Test-Path "C:\temp\mutants.out") {
        Copy-Item "C:\temp\mutants.out" -Destination $shardDir -Recurse -Force

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
Write-Host "  RENDER SWEEP COMPLETE"
Write-Host "  Total Time: $($totalElapsed.ToString('hh\:mm\:ss'))"
Write-Host "  Caught: $totalCaught | Missed: $totalMissed | Timeout: $totalTimeout | Unviable: $totalUnviable"
Write-Host "  Kill Rate: $killRate%"
Write-Host "═══════════════════════════════════════════════════════════════"

@{
    crate = "astraweave-render"
    features = $features
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
