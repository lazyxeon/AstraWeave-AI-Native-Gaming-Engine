#!/usr/bin/env pwsh
# =============================================================================
# AstraWeave PolyHaven Asset Fetcher
# =============================================================================
# Downloads HDRIs and PBR textures from PolyHaven (CC0 licensed) into the
# AstraWeave asset directory structure.
#
# Uses the public PolyHaven API: https://api.polyhaven.com
#
# Usage:
#   # Search for assets
#   ./scripts/fetch_polyhaven.ps1 -Search "forest" -Type hdris
#   ./scripts/fetch_polyhaven.ps1 -Search "brick" -Type textures
#
#   # Download specific assets by ID
#   ./scripts/fetch_polyhaven.ps1 -Download "kloppenheim_06_puresky" -Type hdris
#   ./scripts/fetch_polyhaven.ps1 -Download "forest_ground_04" -Type textures
#
#   # Download multiple assets
#   ./scripts/fetch_polyhaven.ps1 -Download "forest_ground_04,mossy_cobblestone" -Type textures
#
#   # Specify resolution (default: 2k for HDRIs, 2k for textures)
#   ./scripts/fetch_polyhaven.ps1 -Download "kloppenheim_06_puresky" -Type hdris -Resolution 4k
#
#   # Download from a wishlist file
#   ./scripts/fetch_polyhaven.ps1 -WishlistFile "scripts/polyhaven_wishlist.toml"
#
#   # List available categories
#   ./scripts/fetch_polyhaven.ps1 -ListCategories -Type hdris
#
#   # Dry run (show what would be downloaded)
#   ./scripts/fetch_polyhaven.ps1 -Download "forest_ground_04" -Type textures -DryRun
# =============================================================================

param(
    [string]$Search,
    [string]$Download,
    [string]$WishlistFile,
    [ValidateSet("hdris", "textures", "models")]
    [string]$Type = "hdris",
    [ValidateSet("1k", "2k", "4k", "8k")]
    [string]$Resolution = "2k",
    [switch]$ListCategories,
    [switch]$DryRun,
    [switch]$Force,
    [int]$MaxResults = 20
)

$ErrorActionPreference = "Stop"
$API_BASE = "https://api.polyhaven.com"
$assetsRoot = Join-Path $PSScriptRoot "..\assets"

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

function Write-Header($msg) { Write-Host "`n=== $msg ===" -ForegroundColor Magenta }
function Write-OK($msg)     { Write-Host "  [OK] $msg" -ForegroundColor Green }
function Write-Skip($msg)   { Write-Host "  [SKIP] $msg" -ForegroundColor Yellow }
function Write-Err($msg)    { Write-Host "  [ERROR] $msg" -ForegroundColor Red }
function Write-Info($msg)   { Write-Host "  $msg" -ForegroundColor Cyan }

function Invoke-PHApi($endpoint) {
    $url = "$API_BASE$endpoint"
    try {
        $response = Invoke-RestMethod -Uri $url -Method Get -TimeoutSec 30
        return $response
    }
    catch {
        Write-Err "API request failed: $url — $_"
        return $null
    }
}

function Format-FileSize($bytes) {
    if ($bytes -ge 1GB) { return "{0:N1} GB" -f ($bytes / 1GB) }
    if ($bytes -ge 1MB) { return "{0:N1} MB" -f ($bytes / 1MB) }
    if ($bytes -ge 1KB) { return "{0:N1} KB" -f ($bytes / 1KB) }
    return "$bytes bytes"
}

# ─────────────────────────────────────────────────────────────────────────────
# List Categories
# ─────────────────────────────────────────────────────────────────────────────

function Show-Categories {
    Write-Header "POLYHAVEN CATEGORIES ($Type)"
    $cats = Invoke-PHApi "/categories/$Type"
    if (-not $cats) { return }

    $cats.PSObject.Properties |
        Sort-Object { [int]$_.Value } -Descending |
        ForEach-Object {
            Write-Host ("  {0,-30} {1,5} assets" -f $_.Name, $_.Value)
        }
}

# ─────────────────────────────────────────────────────────────────────────────
# Search Assets
# ─────────────────────────────────────────────────────────────────────────────

function Search-Assets($query) {
    Write-Header "SEARCHING POLYHAVEN: '$query' (type=$Type)"
    $assets = Invoke-PHApi "/assets?type=$Type"
    if (-not $assets) { return }

    $results = $assets.PSObject.Properties |
        Where-Object {
            $id = $_.Name
            $data = $_.Value
            $matchId = $id -match $query
            $matchName = $data.name -match $query
            $matchTags = ($data.tags -join " ") -match $query
            $matchCats = ($data.categories -join " ") -match $query
            $matchId -or $matchName -or $matchTags -or $matchCats
        } |
        Sort-Object { $_.Value.download_count } -Descending |
        Select-Object -First $MaxResults

    if ($results.Count -eq 0) {
        Write-Info "No results found for '$query'"
        return
    }

    Write-Host ""
    Write-Host ("  {0,-40} {1,-10} {2,-15} {3}" -f "ID", "Downloads", "Categories", "Name") -ForegroundColor White
    Write-Host ("  " + "-" * 90) -ForegroundColor DarkGray

    foreach ($r in $results) {
        $id = $r.Name
        $data = $r.Value
        $cats = ($data.categories | Select-Object -First 3) -join ", "
        Write-Host ("  {0,-40} {1,-10} {2,-15} {3}" -f $id, $data.download_count, $cats, $data.name)
    }

    Write-Host ""
    Write-Info "Showing top $($results.Count) of matching assets. Use -Download `"<id>`" to fetch."
}

# ─────────────────────────────────────────────────────────────────────────────
# Download HDRI
# ─────────────────────────────────────────────────────────────────────────────

function Download-Hdri($assetId) {
    Write-Header "DOWNLOADING HDRI: $assetId ($Resolution)"

    # Get file info
    $files = Invoke-PHApi "/files/$assetId"
    if (-not $files) { return $false }

    # Navigate to the HDR file at the requested resolution
    $hdriFiles = $files.hdri
    if (-not $hdriFiles) {
        Write-Err "No HDRI files found for '$assetId'"
        return $false
    }

    # Try requested resolution, fallback to available
    $resNode = $hdriFiles.$Resolution
    if (-not $resNode) {
        $availRes = $hdriFiles.PSObject.Properties.Name
        Write-Err "Resolution '$Resolution' not available. Available: $($availRes -join ', ')"
        return $false
    }

    # Prefer .hdr format
    $fileInfo = $resNode.hdr
    if (-not $fileInfo) {
        $fileInfo = $resNode.exr
        $ext = "exr"
    }
    else {
        $ext = "hdr"
    }

    if (-not $fileInfo) {
        Write-Err "No HDR/EXR file found at $Resolution for '$assetId'"
        return $false
    }

    $url = $fileInfo.url
    $size = $fileInfo.size
    $md5 = $fileInfo.md5

    # Determine output path
    $outDir = Join-Path $assetsRoot "hdri\polyhaven\$assetId"
    $outFile = Join-Path $outDir "${assetId}_${Resolution}.$ext"

    if ((Test-Path $outFile) -and -not $Force) {
        Write-Skip "'$outFile' already exists (use -Force to re-download)"
        return $true
    }

    Write-Info "URL: $url"
    Write-Info "Size: $(Format-FileSize $size)"
    Write-Info "Dest: $outFile"

    if ($DryRun) {
        Write-Info "[DRY RUN] Would download $(Format-FileSize $size)"
        return $true
    }

    # Create directory and download
    New-Item -ItemType Directory -Path $outDir -Force | Out-Null
    Write-Host "  Downloading..." -ForegroundColor Gray -NoNewline
    Invoke-WebRequest -Uri $url -OutFile $outFile -TimeoutSec 300
    Write-Host " done." -ForegroundColor Green

    # Verify MD5 if available
    if ($md5) {
        $actualMd5 = (Get-FileHash $outFile -Algorithm MD5).Hash.ToLower()
        if ($actualMd5 -eq $md5) {
            Write-OK "MD5 verified: $md5"
        }
        else {
            Write-Err "MD5 mismatch! Expected: $md5, Got: $actualMd5"
            return $false
        }
    }

    Write-OK "Downloaded: $outFile ($(Format-FileSize (Get-Item $outFile).Length))"
    return $true
}

# ─────────────────────────────────────────────────────────────────────────────
# Download PBR Texture
# ─────────────────────────────────────────────────────────────────────────────

function Download-Texture($assetId) {
    Write-Header "DOWNLOADING TEXTURE: $assetId ($Resolution)"

    $files = Invoke-PHApi "/files/$assetId"
    if (-not $files) { return $false }

    # PBR maps we care about (PolyHaven naming → our naming)
    # PolyHaven uses varied capitalization; we normalize to lowercase first
    $mapMapping = [ordered]@{
        "diffuse"      = "albedo"
        "diff"         = "albedo"
        "color"        = "albedo"
        "nor_gl"       = "normal"
        "normal"       = "normal"
        "nor_dx"       = "normal_dx"
        "rough"        = "roughness"
        "metal"        = "metallic"
        "ao"           = "ao"
        "arm"          = "mra"
        "displacement" = "displacement"
        "disp"         = "displacement"
    }

    $outDir = Join-Path $assetsRoot "textures\polyhaven\$assetId"
    $downloaded = 0

    # Iterate over available maps
    foreach ($prop in $files.PSObject.Properties) {
        $mapName = $prop.Name

        # Skip non-map entries (blend, gltf, mtlx, etc.)
        if ($mapName -in @("blend", "gltf", "mtlx", "usd", "fbx")) { continue }

        $ourName = $null
        $mapNameLower = $mapName.ToLower()
        foreach ($kvp in $mapMapping.GetEnumerator()) {
            if ($mapNameLower -eq $kvp.Key) {
                $ourName = $kvp.Value
                break
            }
        }
        if (-not $ourName) { $ourName = $mapNameLower }

        $resolutions = $prop.Value
        if (-not $resolutions) { continue }

        # Try requested resolution
        $resNode = $resolutions.$Resolution
        if (-not $resNode) { continue }

        # Prefer PNG, then JPG, then EXR
        $fileInfo = $null
        $ext = "png"
        foreach ($fmt in @("png", "jpg", "exr")) {
            if ($resNode.$fmt) {
                $fileInfo = $resNode.$fmt
                $ext = $fmt
                break
            }
        }
        if (-not $fileInfo) { continue }

        $outFile = Join-Path $outDir "${assetId}_${ourName}_${Resolution}.$ext"

        if ((Test-Path $outFile) -and -not $Force) {
            Write-Skip "$ourName already exists"
            $downloaded++
            continue
        }

        $url = $fileInfo.url
        $size = $fileInfo.size

        Write-Info "$ourName ($mapName): $(Format-FileSize $size)"

        if ($DryRun) {
            Write-Info "[DRY RUN] Would download $ourName"
            $downloaded++
            continue
        }

        New-Item -ItemType Directory -Path $outDir -Force | Out-Null
        Invoke-WebRequest -Uri $url -OutFile $outFile -TimeoutSec 300

        # Verify MD5
        if ($fileInfo.md5) {
            $actualMd5 = (Get-FileHash $outFile -Algorithm MD5).Hash.ToLower()
            if ($actualMd5 -ne $fileInfo.md5) {
                Write-Err "MD5 mismatch for $ourName!"
                continue
            }
        }

        $downloaded++
    }

    if ($downloaded -gt 0) {
        Write-OK "Downloaded $downloaded maps to $outDir"
        return $true
    }
    else {
        Write-Err "No maps downloaded for '$assetId'"
        return $false
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# Wishlist Processing
# ─────────────────────────────────────────────────────────────────────────────

function Process-Wishlist($path) {
    Write-Header "PROCESSING WISHLIST: $path"

    if (-not (Test-Path $path)) {
        Write-Err "Wishlist file not found: $path"
        return
    }

    $content = Get-Content $path -Raw
    $success = 0
    $failed = 0

    # Parse simple TOML-like format:
    #   [[hdris]]
    #   id = "asset_name"
    #   resolution = "4k"
    #
    #   [[textures]]
    #   id = "texture_name"

    $currentType = $null
    $currentRes = $Resolution  # default

    foreach ($line in $content -split "`n") {
        $line = $line.Trim()
        if ($line -match '^\[\[(hdris|textures|models)\]\]') {
            $currentType = $Matches[1]
            $currentRes = $Resolution
            continue
        }
        if ($line -match '^resolution\s*=\s*"([^"]+)"') {
            $currentRes = $Matches[1]
            continue
        }
        if ($line -match '^id\s*=\s*"([^"]+)"') {
            $id = $Matches[1]
            if (-not $currentType) {
                Write-Err "No [[type]] section before id = '$id'"
                $failed++
                continue
            }

            $oldRes = $Resolution
            $Resolution = $currentRes

            $ok = $false
            switch ($currentType) {
                "hdris"    { $ok = Download-Hdri $id }
                "textures" { $ok = Download-Texture $id }
                default    { Write-Err "Unsupported type: $currentType"; $ok = $false }
            }

            $Resolution = $oldRes

            if ($ok) { $success++ } else { $failed++ }
        }
    }

    Write-Header "WISHLIST SUMMARY"
    Write-OK "$success succeeded"
    if ($failed -gt 0) { Write-Err "$failed failed" }
}

# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

Write-Host "============================================" -ForegroundColor White
Write-Host " AstraWeave PolyHaven Fetcher" -ForegroundColor White
Write-Host " $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Gray
Write-Host "============================================" -ForegroundColor White

if ($ListCategories) {
    Show-Categories
}
elseif ($Search) {
    Search-Assets $Search
}
elseif ($WishlistFile) {
    Process-Wishlist $WishlistFile
}
elseif ($Download) {
    $ids = $Download -split ","
    $total = 0
    $ok = 0

    foreach ($id in $ids) {
        $id = $id.Trim()
        if (-not $id) { continue }
        $total++

        switch ($Type) {
            "hdris"    { if (Download-Hdri $id)    { $ok++ } }
            "textures" { if (Download-Texture $id) { $ok++ } }
            "models"   { Write-Err "Model download not yet supported"; }
        }
    }

    Write-Header "DOWNLOAD SUMMARY"
    Write-Host "  $ok / $total assets downloaded successfully" -ForegroundColor $(if ($ok -eq $total) { "Green" } else { "Yellow" })
}
else {
    Write-Host @"

  Usage:
    # Search for HDRIs or textures
    .\fetch_polyhaven.ps1 -Search "forest" -Type hdris
    .\fetch_polyhaven.ps1 -Search "brick" -Type textures

    # Download specific assets
    .\fetch_polyhaven.ps1 -Download "kloppenheim_06_puresky" -Type hdris
    .\fetch_polyhaven.ps1 -Download "forest_ground_04" -Type textures -Resolution 4k

    # Download from wishlist
    .\fetch_polyhaven.ps1 -WishlistFile "scripts/polyhaven_wishlist.toml"

    # List categories
    .\fetch_polyhaven.ps1 -ListCategories -Type hdris

    # Dry run
    .\fetch_polyhaven.ps1 -Download "forest_ground_04" -Type textures -DryRun

"@ -ForegroundColor Cyan
}
