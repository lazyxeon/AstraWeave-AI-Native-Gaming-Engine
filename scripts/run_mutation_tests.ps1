#!/usr/bin/env pwsh
# =============================================================================
# AstraWeave Mutation Testing Script — Sequential Targeted Runs
# =============================================================================
# Runs cargo-mutants with nextest on targeted function groups across terrain,
# render, and editor crates. Each run is isolated with cargo clean to manage
# disk space (~50 GB available). Results are logged per run.
#
# Usage: ./scripts/run_mutation_tests.ps1 [-SkipTerrain] [-SkipRender] [-SkipEditor]
# =============================================================================

param(
    [switch]$SkipTerrain,
    [switch]$SkipRender,
    [switch]$SkipEditor,
    [int]$Timeout = 600,
    [string]$ResultDir = "mutation_results"
)

$ErrorActionPreference = "Continue"
$env:CARGO_INCREMENTAL = "0"

# Ensure result directory exists
New-Item -ItemType Directory -Force -Path $ResultDir | Out-Null

function Get-DiskFreeGB {
    [math]::Round([System.IO.DriveInfo]::new('C').AvailableFreeSpace / 1GB, 1)
}

function Run-MutationBatch {
    param(
        [string]$Package,
        [string]$Regex,
        [string]$TestFilter,
        [string]$Label
    )

    $freeGB = Get-DiskFreeGB
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host "BATCH: $Label" -ForegroundColor Cyan
    Write-Host "Package: $Package | Filter: $TestFilter" -ForegroundColor Gray
    Write-Host "Disk free: ${freeGB} GB" -ForegroundColor Gray
    Write-Host "========================================" -ForegroundColor Cyan

    if ($freeGB -lt 10) {
        Write-Host "SKIP: Only ${freeGB} GB free (need 10+)" -ForegroundColor Red
        return @{ label = $Label; skipped = $true; reason = "low disk" }
    }

    # Clean up previous run artifacts
    Remove-Item -Recurse -Force "mutants.out" -ErrorAction SilentlyContinue
    git checkout HEAD -- . 2>$null

    $outFile = Join-Path $ResultDir "${Label}.txt"
    $startTime = Get-Date

    # Run mutation test
    $args = @(
        "mutants", "-p", $Package,
        "--test-tool", "nextest",
        "--in-place",
        "--timeout", $Timeout,
        "--re", $Regex,
        "--", "-E", $TestFilter
    )

    Write-Host "Running: cargo $($args -join ' ')" -ForegroundColor Yellow
    & cargo @args 2>&1 | Tee-Object -FilePath $outFile

    $elapsed = (Get-Date) - $startTime

    # Collect results
    $caught = if (Test-Path "mutants.out/caught.txt") {
        (Get-Content "mutants.out/caught.txt" | Where-Object { $_ -match '\S' }).Count
    } else { 0 }
    $missed = if (Test-Path "mutants.out/missed.txt") {
        (Get-Content "mutants.out/missed.txt" | Where-Object { $_ -match '\S' }).Count
    } else { 0 }
    $unviable = if (Test-Path "mutants.out/unviable.txt") {
        (Get-Content "mutants.out/unviable.txt" | Where-Object { $_ -match '\S' }).Count
    } else { 0 }
    $timeout = if (Test-Path "mutants.out/timeout.txt") {
        (Get-Content "mutants.out/timeout.txt" | Where-Object { $_ -match '\S' }).Count
    } else { 0 }

    $viable = $caught + $missed
    $killRate = if ($viable -gt 0) { [math]::Round($caught / $viable * 100, 1) } else { "N/A" }

    # Save detailed results
    $result = @{
        label    = $Label
        caught   = $caught
        missed   = $missed
        unviable = $unviable
        timeout  = $timeout
        viable   = $viable
        killRate = $killRate
        elapsed  = $elapsed.TotalMinutes
        skipped  = $false
    }

    # Copy mutants.out for this batch
    if (Test-Path "mutants.out") {
        $batchDir = Join-Path $ResultDir "${Label}_mutants.out"
        Copy-Item -Recurse "mutants.out" $batchDir -Force -ErrorAction SilentlyContinue
    }

    Write-Host "`nRESULT: $Label" -ForegroundColor Green
    Write-Host "  Caught: $caught | Missed: $missed | Unviable: $unviable | Timeout: $timeout"
    Write-Host "  Kill Rate: ${killRate}% ($caught/$viable viable)" -ForegroundColor $(if ($killRate -ge 90) { "Green" } else { "Red" })
    Write-Host "  Elapsed: $([math]::Round($elapsed.TotalMinutes, 1)) min"

    # Restore sources
    git checkout HEAD -- . 2>$null
    Remove-Item -Recurse -Force "mutants.out" -ErrorAction SilentlyContinue

    return $result
}

function Clean-BuildArtifacts {
    Write-Host "`nCleaning build artifacts..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force "target" -ErrorAction SilentlyContinue
    $freeGB = Get-DiskFreeGB
    Write-Host "Disk free after clean: ${freeGB} GB" -ForegroundColor Gray
}

# =============================================================================
# Main Execution
# =============================================================================

$allResults = @()

# ---------------------------------------------------------------------------
# TERRAIN MUTATIONS
# ---------------------------------------------------------------------------
if (-not $SkipTerrain) {
    Write-Host "`n### TERRAIN CRATE MUTATIONS ###" -ForegroundColor Magenta

    # Batch 1: classify_whittaker_biome (all match guards + return replacements)
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "astraweave-terrain" `
        -Regex 'climate\.rs:.*classify_whittaker' `
        -TestFilter 'test(whittaker)' `
        -Label "terrain_climate_whittaker"

    # Batch 2: structures (typical_size, rarity, min_spacing, can_place_on_slope)
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "astraweave-terrain" `
        -Regex 'structures\.rs:.*typical_size|structures\.rs:.*rarity|structures\.rs:.*min_spacing|structures\.rs:.*can_place_on_slope' `
        -TestFilter 'test(structure_exact)' `
        -Label "terrain_structures"

    # Batch 3: streaming diagnostics
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "astraweave-terrain" `
        -Regex 'streaming_diagnostics\.rs:' `
        -TestFilter 'test(streaming_diagnostics)' `
        -Label "terrain_streaming"

    # Batch 4: heightmap
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "astraweave-terrain" `
        -Regex 'heightmap\.rs:.*Heightmap|heightmap\.rs:.*calculate_normal|heightmap\.rs:.*HeightmapConfig' `
        -TestFilter 'test(heightmap)' `
        -Label "terrain_heightmap"
}

# ---------------------------------------------------------------------------
# RENDER MUTATIONS
# ---------------------------------------------------------------------------
if (-not $SkipRender) {
    Write-Host "`n### RENDER CRATE MUTATIONS ###" -ForegroundColor Magenta

    # Batch 1: EasingFunction::apply
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "astraweave-render" `
        -Regex 'biome_transition\.rs:.*EasingFunction::apply' `
        -TestFilter 'test(easing)' `
        -Label "render_easing"

    # Batch 2: Camera::dir (only 5 mutants — quick)
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "astraweave-render" `
        -Regex 'camera\.rs:.*Camera::dir' `
        -TestFilter 'test(camera)' `
        -Label "render_camera"

    # Batch 3: TimeOfDay sun position
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "astraweave-render" `
        -Regex 'environment\.rs:.*get_sun_position' `
        -TestFilter 'test(time_of_day)' `
        -Label "render_time_of_day"

    # Batch 4: WeatherSystem
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "astraweave-render" `
        -Regex 'environment\.rs:.*set_weather' `
        -TestFilter 'test(weather)' `
        -Label "render_weather"
}

# ---------------------------------------------------------------------------
# EDITOR MUTATIONS
# ---------------------------------------------------------------------------
if (-not $SkipEditor) {
    Write-Host "`n### EDITOR CRATE MUTATIONS ###" -ForegroundColor Magenta

    # Batch 1: EditorMode (small — only 60 mutants in the file)
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "aw_editor" `
        -Regex 'editor_mode\.rs:.*EditorMode' `
        -TestFilter 'test(editor_mode)' `
        -Label "editor_mode"

    # Batch 2: Clipboard
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "aw_editor" `
        -Regex 'clipboard\.rs:.*Clipboard' `
        -TestFilter 'test(clipboard)' `
        -Label "editor_clipboard"

    # Batch 3: EntityManager
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "aw_editor" `
        -Regex 'entity_manager\.rs:.*EntityManager' `
        -TestFilter 'test(entity_manager)' `
        -Label "editor_entity_manager"

    # Batch 4: DockLayout
    Clean-BuildArtifacts
    $allResults += Run-MutationBatch `
        -Package "aw_editor" `
        -Regex 'dock_layout\.rs:.*DockLayout' `
        -TestFilter 'test(dock_layout)' `
        -Label "editor_dock_layout"
}

# =============================================================================
# SUMMARY REPORT
# =============================================================================
Write-Host "`n" -NoNewline
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "         MUTATION TESTING SUMMARY REPORT" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

$totalCaught = 0; $totalMissed = 0; $totalUnviable = 0

foreach ($r in $allResults) {
    if ($r.skipped) {
        Write-Host "  SKIP  $($r.label) — $($r.reason)" -ForegroundColor Yellow
        continue
    }
    $totalCaught += $r.caught
    $totalMissed += $r.missed
    $totalUnviable += $r.unviable
    $color = if ($r.killRate -ge 90) { "Green" } elseif ($r.killRate -ge 80) { "Yellow" } else { "Red" }
    Write-Host "  $($r.killRate.ToString().PadLeft(5))%  $($r.label) ($($r.caught)/$($r.viable) viable, $($r.unviable) unviable)" -ForegroundColor $color
}

$overallViable = $totalCaught + $totalMissed
$overallKill = if ($overallViable -gt 0) { [math]::Round($totalCaught / $overallViable * 100, 1) } else { "N/A" }

Write-Host "------------------------------------------------------------"
Write-Host "  OVERALL: ${overallKill}% kill rate ($totalCaught/$overallViable caught, $totalUnviable unviable)" -ForegroundColor $(if ($overallKill -ge 90) { "Green" } else { "Red" })
Write-Host "============================================================"

# Save summary to file
$summary = @"
# AstraWeave Mutation Testing Summary
# Generated: $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

Overall Kill Rate: ${overallKill}% ($totalCaught/$overallViable)

## Per-Batch Results
$(foreach ($r in $allResults) {
    if ($r.skipped) { "SKIP: $($r.label) — $($r.reason)" }
    else { "$($r.killRate)%  $($r.label)  caught=$($r.caught) missed=$($r.missed) unviable=$($r.unviable) elapsed=$([math]::Round($r.elapsed,1))min" }
})
"@
$summary | Out-File (Join-Path $ResultDir "SUMMARY.txt") -Encoding utf8
Write-Host "`nSummary saved to $ResultDir\SUMMARY.txt"
