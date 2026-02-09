#!/usr/bin/env pwsh
# =============================================================================
# AstraWeave Asset Validation Script
# =============================================================================
# Validates the entire asset library for:
#   1. Biome material completeness (all 8 biomes have materials.toml + arrays.toml)
#   2. Texture integrity (albedo, normal, MRA triples exist and are non-stub)
#   3. HDRI catalog consistency (all referenced files exist)
#   4. Orphan detection (files not referenced by any manifest/config)
#   5. Naming convention compliance (snake_case, no spaces)
#
# Usage:
#   ./scripts/validate_assets.ps1                  # Full validation
#   ./scripts/validate_assets.ps1 -Section biomes  # Biomes only
#   ./scripts/validate_assets.ps1 -Fix             # Auto-fix where possible
# =============================================================================

param(
    [ValidateSet("all", "biomes", "textures", "hdri", "naming", "orphans")]
    [string]$Section = "all",
    [switch]$Fix,
    [switch]$Verbose,
    [switch]$IncludeThirdParty
)

$ErrorActionPreference = "Continue"
$script:errors = 0
$script:warnings = 0
$script:passes = 0

$assetsRoot = Join-Path $PSScriptRoot "..\assets"
$materialsRoot = Join-Path $assetsRoot "materials"
$hdriRoot = Join-Path $assetsRoot "hdri"

function Write-Pass($msg) { Write-Host "  [PASS] $msg" -ForegroundColor Green; $script:passes++ }
function Write-Fail($msg) { Write-Host "  [FAIL] $msg" -ForegroundColor Red; $script:errors++ }
function Write-Warn($msg) { Write-Host "  [WARN] $msg" -ForegroundColor Yellow; $script:warnings++ }
function Write-Info($msg) { if ($Verbose) { Write-Host "  [INFO] $msg" -ForegroundColor Cyan } }
function Write-Section($msg) { Write-Host "`n=== $msg ===" -ForegroundColor Magenta }

# =============================================================================
# 1. BIOME MATERIAL VALIDATION
# =============================================================================
function Test-BiomeMaterials {
    Write-Section "BIOME MATERIAL VALIDATION"

    $requiredBiomes = @("forest", "desert", "grassland", "mountain", "tundra", "swamp", "beach", "river", "terrain")

    foreach ($biome in $requiredBiomes) {
        $biomeDir = Join-Path $materialsRoot $biome
        $materialsToml = Join-Path $biomeDir "materials.toml"
        $arraysToml = Join-Path $biomeDir "arrays.toml"

        if (-not (Test-Path $biomeDir)) {
            Write-Fail "Biome folder missing: materials/$biome/"
            continue
        }

        if (-not (Test-Path $materialsToml)) {
            Write-Fail "Missing: materials/$biome/materials.toml"
        } else {
            # Validate materials.toml references
            $content = Get-Content $materialsToml -Raw
            $albedoRefs = [regex]::Matches($content, 'albedo\s*=\s*"([^"]+)"') | ForEach-Object { $_.Groups[1].Value }
            $normalRefs = [regex]::Matches($content, 'normal\s*=\s*"([^"]+)"') | ForEach-Object { $_.Groups[1].Value }
            $mraRefs = [regex]::Matches($content, 'mra\s*=\s*"([^"]+)"') | ForEach-Object { $_.Groups[1].Value }

            $allRefs = @($albedoRefs) + @($normalRefs) + @($mraRefs)
            $missing = 0
            foreach ($ref in $allRefs) {
                $fullPath = Join-Path $biomeDir $ref
                if (-not (Test-Path $fullPath)) {
                    Write-Fail "Texture not found: $ref (referenced in $biome/materials.toml)"
                    $missing++
                }
            }
            if ($missing -eq 0) {
                Write-Pass "Biome '$biome': materials.toml valid ($($albedoRefs.Count) layers, all textures exist)"
            }
        }

        if (-not (Test-Path $arraysToml)) {
            Write-Fail "Missing: materials/$biome/arrays.toml"
        } else {
            Write-Pass "Biome '$biome': arrays.toml present"
        }
    }
}

# =============================================================================
# 2. TEXTURE INTEGRITY VALIDATION
# =============================================================================
function Test-TextureIntegrity {
    Write-Section "TEXTURE INTEGRITY VALIDATION"

    # Check all PBR texture triples (albedo, normal, MRA)
    $albedos = Get-ChildItem $materialsRoot -Filter "*.png" -File |
        Where-Object { $_.Name -notmatch "_n\.png$" -and $_.Name -notmatch "_mra\.png$" }

    foreach ($albedo in $albedos) {
        $baseName = $albedo.BaseName
        $normalPath = Join-Path $materialsRoot "${baseName}_n.png"
        $mraPath = Join-Path $materialsRoot "${baseName}_mra.png"

        $hasNormal = Test-Path $normalPath
        $hasMra = Test-Path $mraPath

        if (-not $hasNormal) {
            Write-Fail "Missing normal map: ${baseName}_n.png"
        }
        if (-not $hasMra) {
            Write-Fail "Missing MRA map: ${baseName}_mra.png"
        }

        # Check for stub files (< 500 bytes is suspicious for a 1024x1024 texture)
        if ($albedo.Length -lt 500) {
            Write-Warn "Possible stub: $($albedo.Name) ($($albedo.Length) bytes)"
        }
        if ($hasMra) {
            $mraSize = (Get-Item $mraPath).Length
            if ($mraSize -lt 500) {
                Write-Fail "Stub MRA detected: ${baseName}_mra.png ($mraSize bytes — needs regeneration)"
            }
        }
        if ($hasNormal) {
            $normalSize = (Get-Item $normalPath).Length
            if ($normalSize -lt 500) {
                Write-Fail "Stub normal map detected: ${baseName}_n.png ($normalSize bytes)"
            }
        }

        if ($hasNormal -and $hasMra -and $albedo.Length -ge 500) {
            Write-Info "Texture triple OK: $baseName"
            $script:passes++
        }
    }

    # Check baked KTX2 files
    $ktx2Files = Get-ChildItem (Join-Path $materialsRoot "baked") -Filter "*.ktx2" -File -ErrorAction SilentlyContinue
    $stubKtx2 = $ktx2Files | Where-Object { $_.Length -lt 500 }
    if ($stubKtx2.Count -gt 0) {
        Write-Warn "$($stubKtx2.Count) KTX2 files are stubs (<500 bytes) — need rebaking from source PNGs"
        foreach ($stub in $stubKtx2) {
            Write-Info "  Stub KTX2: $($stub.Name) ($($stub.Length) bytes)"
        }
    }
    $validKtx2 = $ktx2Files | Where-Object { $_.Length -ge 500 }
    Write-Pass "$($validKtx2.Count) valid KTX2 baked textures, $($stubKtx2.Count) stubs"
}

# =============================================================================
# 3. HDRI CATALOG VALIDATION
# =============================================================================
function Test-HdriCatalog {
    Write-Section "HDRI CATALOG VALIDATION"

    $catalogPath = Join-Path $hdriRoot "hdri_catalog.toml"
    if (-not (Test-Path $catalogPath)) {
        Write-Fail "HDRI catalog missing: hdri/hdri_catalog.toml"
        return
    }

    $content = Get-Content $catalogPath -Raw
    $fileRefs = [regex]::Matches($content, 'file\s*=\s*"([^"]+)"') | ForEach-Object { $_.Groups[1].Value }

    $existing = 0
    $missing = 0
    foreach ($ref in $fileRefs) {
        $fullPath = Join-Path $hdriRoot $ref
        if (Test-Path $fullPath) {
            $existing++
            $size = [math]::Round((Get-Item $fullPath).Length / 1MB, 1)
            Write-Info "HDRI OK: $ref ($size MB)"
        } else {
            Write-Fail "HDRI file missing: $ref (referenced in hdri_catalog.toml)"
            $missing++
        }
    }

    if ($missing -eq 0) {
        Write-Pass "All $existing HDRIs referenced in catalog exist"
    }

    # Check for HDR files not listed in catalog
    $allHdris = Get-ChildItem $hdriRoot -Recurse -Include "*.hdr","*.exr","*.png" -File
    $catalogFiles = $fileRefs | ForEach-Object { (Resolve-Path (Join-Path $hdriRoot $_) -ErrorAction SilentlyContinue).Path }
    $uncataloged = $allHdris | Where-Object { $_.FullName -notin $catalogFiles }
    if ($uncataloged.Count -gt 0) {
        Write-Warn "$($uncataloged.Count) HDRI files not referenced in catalog:"
        foreach ($f in $uncataloged) {
            Write-Info "  Uncataloged: $($f.Name)"
        }
    }

    # Check time-of-day coverage
    $timeSlots = @("day", "morning", "evening", "night")
    foreach ($slot in $timeSlots) {
        $count = ([regex]::Matches($content, "time_of_day\s*=\s*`"$slot`"")).Count
        if ($count -eq 0) {
            Write-Warn "No HDRI covers time_of_day = '$slot'"
        } else {
            Write-Pass "Time-of-day '$slot': $count HDRIs available"
        }
    }
}

# =============================================================================
# 4. NAMING CONVENTION VALIDATION
# =============================================================================
function Test-NamingConventions {
    Write-Section "NAMING CONVENTION VALIDATION"

    # Known third-party asset folders (upstream naming, not ours to rename)
    $thirdPartyPatterns = @(
        '\\Amber-Npc\\'  # Character Creator 3 export (CC3 naming convention)
        '\\Amber-Npc$'   # The folder itself
    )

    # Check for folders with spaces or uppercase
    $badFolders = Get-ChildItem $assetsRoot -Directory -Recurse |
        Where-Object { $_.Name -match '\s' -or $_.Name -cmatch '[A-Z]' } |
        Where-Object { $_.FullName -notmatch '\\PBR_' }  # Allow PBR_2K/PBR_4K

    if (-not $IncludeThirdParty) {
        $excluded = 0
        $badFolders = $badFolders | Where-Object {
            $path = $_.FullName
            $isThirdParty = $false
            foreach ($pat in $thirdPartyPatterns) {
                if ($path -match $pat) { $isThirdParty = $true; break }
            }
            if ($isThirdParty) { $excluded++ }
            -not $isThirdParty
        }
        if ($excluded -gt 0) {
            Write-Info "Excluded $excluded third-party folders from naming check (use -IncludeThirdParty to include)"
        }
    }

    if ($badFolders.Count -gt 0) {
        foreach ($f in $badFolders) {
            $rel = $f.FullName -replace [regex]::Escape($assetsRoot), "assets"
            Write-Warn "Non-standard folder name: $rel (use snake_case)"
        }
    } else {
        Write-Pass "All folders follow snake_case convention"
    }

    # Known third-party file path patterns (upstream naming, not ours to rename)
    $thirdPartyFilePatterns = @(
        '\\Amber-Npc\\'              # Character Creator 3 export
        '\\audio\\'                  # All third-party audio (AlkaKrab, water ambient, etc.)
        '\\models\\'                 # Third-party 3D models (Kenney, KayKit, etc.)
    )

    # Check for files with spaces
    $badFiles = Get-ChildItem $assetsRoot -File -Recurse |
        Where-Object { $_.Name -match '\s' }

    if (-not $IncludeThirdParty) {
        $excludedFiles = 0
        $badFiles = $badFiles | Where-Object {
            $path = $_.FullName
            $isThirdParty = $false
            foreach ($pat in $thirdPartyFilePatterns) {
                if ($path -match $pat) { $isThirdParty = $true; break }
            }
            if ($isThirdParty) { $excludedFiles++ }
            -not $isThirdParty
        }
        if ($excludedFiles -gt 0) {
            Write-Info "Excluded $excludedFiles third-party files from naming check"
        }
    }

    if ($badFiles.Count -gt 0) {
        Write-Warn "$($badFiles.Count) files have spaces in names"
    } else {
        Write-Pass "No files with spaces in names"
    }
}

# =============================================================================
# 5. ORPHAN DETECTION
# =============================================================================
function Test-Orphans {
    Write-Section "ORPHAN / LOOSE FILE DETECTION"

    # Check for loose files in assets root (should only be configs)
    $looseFiles = Get-ChildItem $assetsRoot -File |
        Where-Object { $_.Extension -notin @('.toml', '.md', '.json', '.ron') }
    if ($looseFiles.Count -gt 0) {
        Write-Warn "$($looseFiles.Count) non-config files in assets root:"
        foreach ($f in $looseFiles) {
            Write-Info "  Loose: $($f.Name) ($($f.Length) bytes)"
        }
    } else {
        Write-Pass "Assets root contains only config files"
    }

    # Check for empty directories
    $emptyDirs = Get-ChildItem $assetsRoot -Directory -Recurse |
        Where-Object { (Get-ChildItem $_.FullName -File -Recurse).Count -eq 0 }
    if ($emptyDirs.Count -gt 0) {
        Write-Warn "$($emptyDirs.Count) empty directories detected"
        if ($Fix) {
            foreach ($d in $emptyDirs) {
                Remove-Item $d.FullName -Force -Recurse
                Write-Host "  [FIXED] Removed empty: $($d.FullName -replace [regex]::Escape($assetsRoot), 'assets')" -ForegroundColor Blue
            }
        }
    } else {
        Write-Pass "No empty directories"
    }
}

# =============================================================================
# MAIN
# =============================================================================
Write-Host "============================================" -ForegroundColor White
Write-Host " AstraWeave Asset Validation" -ForegroundColor White
Write-Host " $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Gray
Write-Host "============================================" -ForegroundColor White

switch ($Section) {
    "all" {
        Test-BiomeMaterials
        Test-TextureIntegrity
        Test-HdriCatalog
        Test-NamingConventions
        Test-Orphans
    }
    "biomes"   { Test-BiomeMaterials }
    "textures" { Test-TextureIntegrity }
    "hdri"     { Test-HdriCatalog }
    "naming"   { Test-NamingConventions }
    "orphans"  { Test-Orphans }
}

# Summary
Write-Host "`n============================================" -ForegroundColor White
Write-Host " VALIDATION SUMMARY" -ForegroundColor White
Write-Host "============================================" -ForegroundColor White
Write-Host "  Passes:   $($script:passes)" -ForegroundColor Green
Write-Host "  Warnings: $($script:warnings)" -ForegroundColor Yellow
Write-Host "  Errors:   $($script:errors)" -ForegroundColor $(if ($script:errors -gt 0) { "Red" } else { "Green" })

if ($script:errors -gt 0) {
    Write-Host "`n  STATUS: FAILED ($($script:errors) issues to fix)" -ForegroundColor Red
    exit 1
} elseif ($script:warnings -gt 0) {
    Write-Host "`n  STATUS: PASSED with warnings" -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "`n  STATUS: ALL CLEAR" -ForegroundColor Green
    exit 0
}
