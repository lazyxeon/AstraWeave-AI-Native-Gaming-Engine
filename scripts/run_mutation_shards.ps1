#!/usr/bin/env pwsh
# =============================================================================
# AstraWeave Mutation Testing - Sharded Parallel Execution
# Targets: terrain, render, editor | Goal: 90%+ kill rate
# Uses cargo-mutants with nextest for parallel test execution
# =============================================================================

param(
    [int]$Shards = 4,
    [int]$TimeoutMultiplier = 3,
    [string]$OutputDir = "mutants-results"
)

$ErrorActionPreference = "Continue"
$Root = $PSScriptRoot | Split-Path -Parent
Set-Location $Root

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$jobs = @()

# ==============================
# TERRAIN (4,629 mutants, 4 shards)
# ==============================
Write-Host "=== Launching TERRAIN mutation shards ===" -ForegroundColor Cyan
for ($i = 0; $i -lt $Shards; $i++) {
    $shardId = "$i/$Shards"
    $outDir = "$OutputDir/terrain-shard$i"
    $jobName = "terrain-shard-$i"
    
    $sb = [scriptblock]::Create(@"
        Set-Location '$Root'
        cargo mutants --package astraweave-terrain --test-tool nextest --in-place --timeout-multiplier $TimeoutMultiplier --shard $shardId --output '$outDir' --json 2>&1 | Out-File '$outDir.log' -Encoding UTF8
"@)
    
    $job = Start-Job -Name $jobName -ScriptBlock $sb
    $jobs += $job
    Write-Host "  Started $jobName (shard $shardId)" -ForegroundColor Green
}

# ==============================
# RENDER (2,889 non-GPU mutants, 4 shards)
# ==============================
Write-Host "`n=== Launching RENDER mutation shards ===" -ForegroundColor Cyan
$renderExclude = "renderer\.rs|ibl\.rs|skybox|clustered_forward|deferred|depth|gi/|nanite_gpu|nanite_render|nanite_visibility|residency|skinning_gpu|water\.rs|gpu_particles|gpu_memory|msaa|shadow_csm|transparency|overlay|effects|debug_quad|decals"

for ($i = 0; $i -lt $Shards; $i++) {
    $shardId = "$i/$Shards"
    $outDir = "$OutputDir/render-shard$i"
    $jobName = "render-shard-$i"
    
    $sb = [scriptblock]::Create(@"
        Set-Location '$Root'
        cargo mutants --package astraweave-render --test-tool nextest --in-place --timeout-multiplier $TimeoutMultiplier --shard $shardId --exclude-re '$renderExclude' --output '$outDir' --json 2>&1 | Out-File '$outDir.log' -Encoding UTF8
"@)
    
    $job = Start-Job -Name $jobName -ScriptBlock $sb
    $jobs += $job
    Write-Host "  Started $jobName (shard $shardId)" -ForegroundColor Green
}

# ==============================
# EDITOR (6,659 key-file mutants, 4 shards)
# ==============================
Write-Host "`n=== Launching EDITOR mutation shards ===" -ForegroundColor Cyan
$editorFilter = "entity_manager|command\.rs|plugin\.rs|dock_layout|prefab\.rs|scene_state|runtime\.rs|tab_viewer|editor_mode|interaction|clipboard|terrain_integration|material_inspector|level_doc|asset_pack|game_project|scene_serialization"

for ($i = 0; $i -lt $Shards; $i++) {
    $shardId = "$i/$Shards"
    $outDir = "$OutputDir/editor-shard$i"
    $jobName = "editor-shard-$i"
    
    $sb = [scriptblock]::Create(@"
        Set-Location '$Root'
        cargo mutants --package aw_editor --test-tool nextest --in-place --timeout-multiplier $TimeoutMultiplier --shard $shardId --re '$editorFilter' --output '$outDir' --json 2>&1 | Out-File '$outDir.log' -Encoding UTF8
"@)
    
    $job = Start-Job -Name $jobName -ScriptBlock $sb
    $jobs += $job
    Write-Host "  Started $jobName (shard $shardId)" -ForegroundColor Green
}

# ==============================
# Monitor progress
# ==============================
Write-Host "`n=== Monitoring $($jobs.Count) mutation shards ===" -ForegroundColor Yellow
Write-Host "Output directory: $OutputDir"
Write-Host "Press Ctrl+C to stop monitoring (jobs continue in background)`n"

$startTime = Get-Date
while ($jobs | Where-Object { $_.State -eq 'Running' }) {
    $running = ($jobs | Where-Object { $_.State -eq 'Running' }).Count
    $completed = ($jobs | Where-Object { $_.State -eq 'Completed' }).Count
    $failed = ($jobs | Where-Object { $_.State -eq 'Failed' }).Count
    $elapsed = ((Get-Date) - $startTime).ToString("hh\:mm\:ss")
    
    Write-Host "`r[$elapsed] Running: $running | Completed: $completed | Failed: $failed" -NoNewline
    Start-Sleep -Seconds 30
}

Write-Host "`n`n=== All shards complete ===" -ForegroundColor Green

# ==============================
# Collect results
# ==============================
$summary = @()
foreach ($job in $jobs) {
    $result = Receive-Job -Job $job -ErrorAction SilentlyContinue
    $summary += [PSCustomObject]@{
        Name = $job.Name
        State = $job.State
    }
}

Write-Host "`n=== Shard Summary ===" -ForegroundColor Cyan
$summary | Format-Table -AutoSize

# Parse outcomes.json from each shard
Write-Host "`n=== Mutation Results ===" -ForegroundColor Cyan
foreach ($crate in @("terrain", "render", "editor")) {
    $totalCaught = 0; $totalMissed = 0; $totalTimeout = 0; $totalUnviable = 0
    
    for ($i = 0; $i -lt $Shards; $i++) {
        $jsonPath = "$OutputDir/$crate-shard$i/outcomes.json"
        if (Test-Path $jsonPath) {
            $data = Get-Content $jsonPath -Raw | ConvertFrom-Json
            $outcomes = $data.outcomes | Where-Object { $_.scenario -ne "Baseline" }
            $caught = ($outcomes | Where-Object { $_.summary -eq "CaughtMutant" }).Count
            $missed = ($outcomes | Where-Object { $_.summary -eq "MissedMutant" }).Count
            $timeout = ($outcomes | Where-Object { $_.summary -eq "Timeout" }).Count
            $unviable = ($outcomes | Where-Object { $_.summary -eq "Unviable" }).Count
            
            $totalCaught += $caught
            $totalMissed += $missed
            $totalTimeout += $timeout
            $totalUnviable += $unviable
            
            Write-Host "  $crate shard $i: caught=$caught missed=$missed timeout=$timeout unviable=$unviable"
        } else {
            Write-Host "  $crate shard $i: NO RESULTS (check $crate-shard$i.log)" -ForegroundColor Red
        }
    }
    
    $viable = $totalCaught + $totalMissed
    if ($viable -gt 0) {
        $killRate = [math]::Round(($totalCaught / $viable) * 100, 1)
    } else {
        $killRate = "N/A"
    }
    
    Write-Host "  --- $crate TOTAL: caught=$totalCaught missed=$totalMissed timeout=$totalTimeout unviable=$totalUnviable | Kill Rate: $killRate%" -ForegroundColor $(if ($killRate -ge 90) { "Green" } elseif ($killRate -ge 80) { "Yellow" } else { "Red" })
}

# Dump all MISSED mutants for remediation
Write-Host "`n=== MISSED Mutants (need remediation) ===" -ForegroundColor Red
foreach ($crate in @("terrain", "render", "editor")) {
    $missedFile = "$OutputDir/$crate-missed.txt"
    $allMissed = @()
    
    for ($i = 0; $i -lt $Shards; $i++) {
        $jsonPath = "$OutputDir/$crate-shard$i/outcomes.json"
        if (Test-Path $jsonPath) {
            $data = Get-Content $jsonPath -Raw | ConvertFrom-Json
            $missed = $data.outcomes | Where-Object { $_.scenario -ne "Baseline" -and $_.summary -eq "MissedMutant" }
            foreach ($m in $missed) {
                $allMissed += "$($m.scenario)"
            }
        }
    }
    
    if ($allMissed.Count -gt 0) {
        $allMissed | Out-File $missedFile -Encoding UTF8
        Write-Host "  $crate`: $($allMissed.Count) missed mutants -> $missedFile"
    } else {
        Write-Host "  $crate`: 0 missed mutants" -ForegroundColor Green
    }
}

Write-Host "`nDone! Check $OutputDir/ for detailed results." -ForegroundColor Green
