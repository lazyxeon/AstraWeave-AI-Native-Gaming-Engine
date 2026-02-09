#!/usr/bin/env pwsh
# =============================================================================
# generate_asset_index.ps1 — Auto-generate assets/asset_index.toml from disk
# =============================================================================
# Scans the assets/ directory tree and produces a fully reproducible
# asset_index.toml.  Run from the repo root or pass -Root explicitly.
#
# Usage:
#   ./scripts/generate_asset_index.ps1                  # Write to assets/asset_index.toml
#   ./scripts/generate_asset_index.ps1 -DryRun          # Print to stdout, don't write
#   ./scripts/generate_asset_index.ps1 -Diff             # Show changes vs current file
# =============================================================================

param(
    [string]$Root = (Split-Path -Parent $PSScriptRoot),
    [switch]$DryRun,
    [switch]$Diff
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$assetsDir = Join-Path $Root 'assets'
$outPath = Join-Path $assetsDir 'asset_index.toml'
$today = Get-Date -Format 'yyyy-MM-dd'

# ─── Helpers ──────────────────────────────────────────────────────────────────

function Escape-Toml([string]$s) { $s -replace '\\', '/' }

function Has-File($dir, $pattern) {
    @(Get-ChildItem -Path $dir -Filter $pattern -File -ErrorAction SilentlyContinue).Count -gt 0
}

# ──────────────────────────────────────────────────────────────────────────────
# 1. Material Sets — dirs under materials/ that contain materials.toml
# ──────────────────────────────────────────────────────────────────────────────

$materialSets = @()
$matRoot = Join-Path $assetsDir 'materials'
$skipMatDirs = @('baked', 'polyhaven')  # Not biomes — intermediate/third-party

foreach ($biomeDir in Get-ChildItem -Path $matRoot -Directory) {
    if ($biomeDir.Name -in $skipMatDirs) { continue }
    $matToml = Join-Path $biomeDir.FullName 'materials.toml'
    if (-not (Test-Path $matToml)) { continue }

    # Count layers from materials.toml [[layer]] entries
    $layerCount = (Select-String -Path $matToml -Pattern '^\[\[layer\]\]' -AllMatches).Matches.Count
    if ($layerCount -eq 0) { $layerCount = 5 }  # default assumption

    $biomeName = $biomeDir.Name
    $relDir = "materials/$biomeName"

    $materialSets += [PSCustomObject]@{
        Biome  = $biomeName
        Dir    = $relDir
        Layers = $layerCount
    }
}

# Sort alphabetically for deterministic output
$materialSets = $materialSets | Sort-Object -Property Biome

# ──────────────────────────────────────────────────────────────────────────────
# 2. Textures — look for *_albedo.png, *_n.png, *_mra.png patterns, or
#    individual PBR sets directly in materials/ root
# ──────────────────────────────────────────────────────────────────────────────

$textures = @()
$textureRoot = Join-Path $assetsDir 'textures'

# Strategy: Find unique base names from albedo PNGs in textures/ root
# Also scan per-biome texture subdirs (textures/forest/, etc.)

$knownTextureNames = @{}

# Scan textures/ root for PBR sets in the materials/ root (existing pattern)
# Pattern: {name}.png = albedo, {name}_n.png = normal, {name}_mra.png = MRA
foreach ($albedo in Get-ChildItem -Path $matRoot -Filter '*.png' -File) {
    $baseName = $albedo.BaseName
    # Skip _n and _mra suffixed files — they're maps, not albedos
    if ($baseName -match '_(n|mra|normal|roughness|metallic|ao)$') { continue }

    $name = $baseName
    $hasNormal = Test-Path (Join-Path $matRoot "${name}_n.png")
    $hasMra    = Test-Path (Join-Path $matRoot "${name}_mra.png")
    $hasKtx2   = Test-Path (Join-Path $matRoot "${name}.ktx2")

    if (-not $hasNormal -and -not $hasMra) { continue }  # Not a PBR set

    $knownTextureNames[$name] = $true

    # Probe resolution from image if possible; default 1024
    $resolution = "1024x1024"

    $maps = @('albedo')
    if ($hasNormal) { $maps += 'normal' }
    if ($hasMra) { $maps += 'mra' }

    $textures += [PSCustomObject]@{
        Name       = $name
        Dir        = "materials"   # lives in materials/ root (flat layout)
        Maps       = $maps
        HasKtx2    = $hasKtx2
        Resolution = $resolution
    }
}

# Also scan textures/ subdirs for per-biome textures
if (Test-Path $textureRoot) {
    foreach ($subDir in Get-ChildItem -Path $textureRoot -Directory) {
        # Per-biome texture subdirs are tracked via material sets, skip here
    }
}

$textures = $textures | Sort-Object -Property Name

# ──────────────────────────────────────────────────────────────────────────────
# 3. HDRIs — parse hdri_catalog.toml for biome→file mappings
# ──────────────────────────────────────────────────────────────────────────────

$hdris = @()
$catalogPath = Join-Path $assetsDir 'hdri' 'hdri_catalog.toml'

if (Test-Path $catalogPath) {
    $catalogContent = Get-Content $catalogPath -Raw

    # Parse [[hdri]] entries from the catalog
    # Simple regex-based parser for the TOML structure
    $hdriBlocks = [regex]::Matches($catalogContent, '(?s)\[\[hdri\]\](.*?)(?=\[\[hdri\]\]|\z)')

    foreach ($block in $hdriBlocks) {
        $text = $block.Groups[1].Value

        $name = ''
        $file = ''
        $time = ''
        $biomes = @()

        if ($text -match 'name\s*=\s*"([^"]+)"') { $name = $Matches[1] }
        if ($text -match 'file\s*=\s*"([^"]+)"') { $file = "hdri/$($Matches[1])" }
        if ($text -match 'time_of_day\s*=\s*"([^"]+)"') { $time = $Matches[1] }
        if ($text -match 'biomes\s*=\s*\[([^\]]+)\]') {
            $biomeStr = $Matches[1]
            $biomes = [regex]::Matches($biomeStr, '"([^"]+)"') | ForEach-Object { $_.Groups[1].Value }
        }

        if (-not $name -or -not $file) { continue }

        # Verify the file exists on disk
        $fullHdri = Join-Path $assetsDir $file
        $exists = Test-Path $fullHdri

        $hdris += [PSCustomObject]@{
            Name   = $name
            File   = $file
            Time   = $time
            Biomes = $biomes
            Exists = $exists
        }
    }
}

# Also pick up any HDRIs on disk not in the catalog
$hdriRoot = Join-Path $assetsDir 'hdri'
$catalogNamesList = @($hdris | ForEach-Object { $_.Name })

# sky_equirect.png (legacy fallback)
$skyEq = Join-Path $hdriRoot 'sky_equirect.png'
if ((Test-Path $skyEq) -and ('sky_equirect' -notin $catalogNamesList)) {
    $hdris += [PSCustomObject]@{
        Name   = 'sky_equirect'
        File   = 'hdri/sky_equirect.png'
        Time   = 'day'
        Biomes = @('grassland')
        Exists = $true
    }
}

# ──────────────────────────────────────────────────────────────────────────────
# 4. Models — detect known model packs
# ──────────────────────────────────────────────────────────────────────────────

$models = @()
$modelRoot = Join-Path $assetsDir 'models'

# Amber NPC
$amberDir = Join-Path $modelRoot 'Amber-Npc'
if (Test-Path $amberDir) {
    $fbxCount = @(Get-ChildItem -Path $amberDir -Filter '*.fbx' -Recurse -File).Count
    $glbCount = @(Get-ChildItem -Path $amberDir -Filter '*.glb' -Recurse -File).Count
    $modelCount = $fbxCount + $glbCount
    $fmt = if ($fbxCount -gt $glbCount) { 'fbx' } else { 'glb' }
    $models += [PSCustomObject]@{
        Name    = 'amber_npc'
        Dir     = 'models/Amber-Npc'
        Format  = $fmt
        Source  = 'Character Creator 3'
        License = 'CC-BY'
        Note    = "Animated NPC character ($modelCount model files)"
    }
}

# Dungeon/environment kit (loose models in models/ root)
$looseGlb = @(Get-ChildItem -Path $modelRoot -Filter '*.glb' -File).Count
$looseFbx = @(Get-ChildItem -Path $modelRoot -Filter '*.fbx' -File).Count
if (($looseGlb + $looseFbx) -gt 10) {
    $models += [PSCustomObject]@{
        Name    = 'dungeon_kit'
        Dir     = 'models'
        Format  = 'glb+fbx'
        Source  = 'KayKit/Kenney'
        License = 'CC0'
        Note    = "$($looseGlb + $looseFbx) modular pieces (walls, floors, stairs, doors, etc.)"
    }
}

# Greybox kit
$greyboxDir = Join-Path $modelRoot 'greybox'
if (Test-Path $greyboxDir) {
    $gbCount = @(Get-ChildItem -Path $greyboxDir -Filter '*.glb' -Recurse -File).Count +
               @(Get-ChildItem -Path $greyboxDir -Filter '*.gltf' -Recurse -File).Count
    if ($gbCount -gt 0) {
        $models += [PSCustomObject]@{
            Name    = 'greybox_kit'
            Dir     = 'models/greybox'
            Format  = 'glb'
            Source  = 'procedural'
            License = 'MIT'
            Note    = "$gbCount greybox placeholder models"
        }
    }
}

# ──────────────────────────────────────────────────────────────────────────────
# 5. Audio packs — detect subdirectories with audio files
# ──────────────────────────────────────────────────────────────────────────────

$audioPacks = @()
$audioRoot = Join-Path $assetsDir 'audio'

if (Test-Path $audioRoot) {
    foreach ($pack in Get-ChildItem -Path $audioRoot -Directory) {
        $wavCount = @(Get-ChildItem -Path $pack.FullName -Filter '*.wav' -Recurse -File -ErrorAction SilentlyContinue).Count
        $mp3Count = @(Get-ChildItem -Path $pack.FullName -Filter '*.mp3' -Recurse -File -ErrorAction SilentlyContinue).Count
        $oggCount = @(Get-ChildItem -Path $pack.FullName -Filter '*.ogg' -Recurse -File -ErrorAction SilentlyContinue).Count

        $totalTracks = $wavCount + $mp3Count + $oggCount
        if ($totalTracks -eq 0) { continue }

        $formats = @()
        if ($mp3Count -gt 0) { $formats += 'mp3' }
        if ($wavCount -gt 0) { $formats += 'wav' }
        if ($oggCount -gt 0) { $formats += 'ogg' }

        $packName = $pack.Name
        $relDir = "audio/$packName"

        $audioPacks += [PSCustomObject]@{
            Name    = $packName
            Dir     = $relDir
            Formats = $formats
            Tracks  = $totalTracks
        }
    }
}

$audioPacks = $audioPacks | Sort-Object -Property Name

# ──────────────────────────────────────────────────────────────────────────────
# 6. Generate TOML output
# ──────────────────────────────────────────────────────────────────────────────

$sb = [System.Text.StringBuilder]::new(8192)

[void]$sb.AppendLine(@"
# =============================================================================
# AstraWeave Runtime Asset Index
# =============================================================================
# AUTO-GENERATED by scripts/generate_asset_index.ps1 on $today
# Do NOT edit manually — re-run the script after adding/removing assets.
#
# Usage (Rust):
#   let index = AssetIndex::load("assets/asset_index.toml")?;
#   let forest_mat = index.material_set("forest");
#   let day_hdri  = index.hdri("forest");
# =============================================================================

[index]
version = 1
generated = "$today"
asset_root = "assets"
"@)

# ── Material Sets ─────────────────────────────────────────────────────────────

[void]$sb.AppendLine(@"

# ---------------------------------------------------------------------------
# Biome Material Sets
# ---------------------------------------------------------------------------
# Each biome has a material set with terrain layers (splat-mapped).
# The renderer loads materials.toml + arrays.toml from each directory.
"@)

foreach ($m in $materialSets) {
    [void]$sb.AppendLine(@"

[[material_set]]
biome = "$($m.Biome)"
dir = "$($m.Dir)"
layers = $($m.Layers)
"@)
}

# ── Textures ──────────────────────────────────────────────────────────────────

[void]$sb.AppendLine(@"

# ---------------------------------------------------------------------------
# PBR Texture Library
# ---------------------------------------------------------------------------
# Individual PBR texture sets (albedo + normal + MRA).
"@)

foreach ($t in $textures) {
    $mapsStr = ($t.Maps | ForEach-Object { "`"$_`"" }) -join ', '
    $ktx2 = if ($t.HasKtx2) { 'true' } else { 'false' }

    [void]$sb.AppendLine(@"

[[texture]]
name = "$($t.Name)"
dir = "$($t.Dir)"
maps = [$mapsStr]
has_ktx2 = $ktx2
resolution = "$($t.Resolution)"
"@)
}

# ── HDRIs ─────────────────────────────────────────────────────────────────────

[void]$sb.AppendLine(@"

# ---------------------------------------------------------------------------
# HDRI Environment Maps
# ---------------------------------------------------------------------------
# See hdri/hdri_catalog.toml for full biome x time-of-day mappings.
"@)

foreach ($h in $hdris) {
    $biomesStr = ($h.Biomes | ForEach-Object { "`"$_`"" }) -join ', '
    $existsMark = if ($h.Exists) { '' } else { '  # WARNING: file missing on disk!' }

    [void]$sb.AppendLine(@"

[[hdri]]
name = "$($h.Name)"
file = "$(Escape-Toml $h.File)"
time = "$($h.Time)"
biomes = [$biomesStr]$existsMark
"@)
}

# ── Models ────────────────────────────────────────────────────────────────────

[void]$sb.AppendLine(@"

# ---------------------------------------------------------------------------
# 3D Models
# ---------------------------------------------------------------------------
"@)

foreach ($m in $models) {
    [void]$sb.AppendLine(@"

[[model]]
name = "$($m.Name)"
dir = "$($m.Dir)"
format = "$($m.Format)"
source = "$($m.Source)"
license = "$($m.License)"
note = "$($m.Note)"
"@)
}

# ── Audio ─────────────────────────────────────────────────────────────────────

[void]$sb.AppendLine(@"

# ---------------------------------------------------------------------------
# Audio
# ---------------------------------------------------------------------------
"@)

foreach ($a in $audioPacks) {
    $fmtStr = ($a.Formats | ForEach-Object { "`"$_`"" }) -join ', '

    [void]$sb.AppendLine(@"

[[audio_pack]]
name = "$($a.Name)"
dir = "$($a.Dir)"
formats = [$fmtStr]
tracks = $($a.Tracks)
"@)
}

# ──────────────────────────────────────────────────────────────────────────────
# 7. Output
# ──────────────────────────────────────────────────────────────────────────────

$output = $sb.ToString().TrimEnd() + "`n"

if ($DryRun) {
    Write-Output $output
    return
}

if ($Diff) {
    if (Test-Path $outPath) {
        $existing = Get-Content $outPath -Raw
        if ($existing -eq $output) {
            Write-Host "`e[32m[OK]`e[0m asset_index.toml is already up-to-date." -ForegroundColor Green
        } else {
            Write-Host "`e[33m[DIFF]`e[0m asset_index.toml would change:" -ForegroundColor Yellow
            # Simple line-count comparison
            $oldLines = ($existing -split "`n").Count
            $newLines = ($output -split "`n").Count
            Write-Host "  Old: $oldLines lines"
            Write-Host "  New: $newLines lines"
            Write-Host "  Material sets: $($materialSets.Count)"
            Write-Host "  Textures:      $($textures.Count)"
            Write-Host "  HDRIs:         $($hdris.Count)"
            Write-Host "  Models:        $($models.Count)"
            Write-Host "  Audio packs:   $($audioPacks.Count)"
        }
    } else {
        Write-Host "`e[33m[NEW]`e[0m asset_index.toml does not exist yet." -ForegroundColor Yellow
    }
    return
}

# Write the file
$output | Set-Content -Path $outPath -NoNewline -Encoding utf8NoBOM
$lineCount = ($output -split "`n").Count

Write-Host ""
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host "  Asset Index Generated: $today" -ForegroundColor Cyan
Write-Host "=============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Output:        $(Escape-Toml $outPath)"
Write-Host "  Lines:         $lineCount"
Write-Host "  Material sets: $($materialSets.Count)"
Write-Host "  Textures:      $($textures.Count)"
Write-Host "  HDRIs:         $($hdris.Count)"
Write-Host "  Models:        $($models.Count)"
Write-Host "  Audio packs:   $($audioPacks.Count)"
Write-Host ""

$missingHdri = @($hdris | Where-Object { -not $_.Exists })
if ($missingHdri.Count -gt 0) {
    Write-Host "  [WARN] Missing HDRI files:" -ForegroundColor Yellow
    foreach ($h in $missingHdri) {
        Write-Host "    - $($h.File)" -ForegroundColor Yellow
    }
    Write-Host ""
}

Write-Host "  Done." -ForegroundColor Green
