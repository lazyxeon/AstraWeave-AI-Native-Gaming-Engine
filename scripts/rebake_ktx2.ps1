<#
.SYNOPSIS
    Rebakes source PNG textures to GPU-compressed KTX2 format.

.DESCRIPTION
    Scans assets/materials/ for source PNGs and bakes them to KTX2 using toktx.
    Detects stub/placeholder KTX2 files (< 1 KB) and prioritizes rebaking those.
    Requires: toktx (from KTX-Software, https://github.com/KhronosGroup/KTX-Software)

.PARAMETER Force
    Rebake all textures, not just stubs and missing ones.

.PARAMETER DryRun
    Show what would be baked without actually running toktx.

.PARAMETER Filter
    Only process textures matching this pattern (e.g., "*_mra*" for MRA maps only).

.EXAMPLE
    .\scripts\rebake_ktx2.ps1                    # Bake stubs + missing only
    .\scripts\rebake_ktx2.ps1 -Force              # Rebake everything
    .\scripts\rebake_ktx2.ps1 -DryRun             # Preview what would be done
    .\scripts\rebake_ktx2.ps1 -Filter "*_mra*"    # Only MRA textures
#>

param(
    [switch]$Force,
    [switch]$DryRun,
    [string]$Filter = "*"
)

$ErrorActionPreference = "Stop"

$assetsRoot = Join-Path $PSScriptRoot ".." "assets"
$materialsDir = Join-Path $assetsRoot "materials"
$bakedDir = Join-Path $materialsDir "baked"

# Ensure baked directory exists
if (-not (Test-Path $bakedDir)) {
    New-Item -ItemType Directory -Path $bakedDir -Force | Out-Null
}

# Check for toktx
$toktx = Get-Command "toktx" -ErrorAction SilentlyContinue
if (-not $toktx) {
    Write-Host ""
    Write-Host "ERROR: toktx not found in PATH." -ForegroundColor Red
    Write-Host ""
    Write-Host "Install KTX-Software from: https://github.com/KhronosGroup/KTX-Software/releases"
    Write-Host "  - Windows: Download the .exe installer or add bin/ to PATH"
    Write-Host "  - macOS:   brew install ktx-software"
    Write-Host "  - Linux:   See GitHub releases for .deb/.rpm packages"
    Write-Host ""
    if (-not $DryRun) {
        Write-Host "Running in audit-only mode (no toktx available)..." -ForegroundColor Yellow
        Write-Host ""
    }
}

# Gather source PNGs
$sourcePngs = Get-ChildItem $materialsDir -Filter "*.png" -File | Where-Object { $_.Name -like $Filter }

Write-Host "=== KTX2 Rebake Tool ===" -ForegroundColor Cyan
Write-Host "Source directory : $materialsDir"
Write-Host "Baked directory  : $bakedDir"
Write-Host "Source PNGs found: $($sourcePngs.Count)"
Write-Host ""

$stats = @{
    Stubs    = 0
    Missing  = 0
    Current  = 0
    Baked    = 0
    Failed   = 0
    Skipped  = 0
}

$toBake = @()

foreach ($png in $sourcePngs) {
    $baseName = [System.IO.Path]::GetFileNameWithoutExtension($png.Name)
    $ktx2Name = "$baseName.ktx2"
    $ktx2Path = Join-Path $bakedDir $ktx2Name

    if (Test-Path $ktx2Path) {
        $ktx2File = Get-Item $ktx2Path
        $isStub = $ktx2File.Length -lt 1024  # < 1 KB = stub

        if ($isStub) {
            $stats.Stubs++
            $toBake += @{ Source = $png.FullName; Output = $ktx2Path; Reason = "stub ($($ktx2File.Length) bytes)" }
        }
        elseif ($Force) {
            $toBake += @{ Source = $png.FullName; Output = $ktx2Path; Reason = "force rebake" }
        }
        else {
            $stats.Current++
        }
    }
    else {
        $stats.Missing++
        $toBake += @{ Source = $png.FullName; Output = $ktx2Path; Reason = "missing" }
    }
}

# Report current state
Write-Host "KTX2 Status:" -ForegroundColor Cyan
Write-Host "  Current (valid) : $($stats.Current)" -ForegroundColor Green
Write-Host "  Stubs (< 1 KB)  : $($stats.Stubs)" -ForegroundColor Yellow
Write-Host "  Missing          : $($stats.Missing)" -ForegroundColor Yellow
Write-Host "  To bake          : $($toBake.Count)" -ForegroundColor Cyan
Write-Host ""

if ($toBake.Count -eq 0) {
    Write-Host "Nothing to bake. All KTX2 files are current." -ForegroundColor Green
    exit 0
}

# List what will be baked
Write-Host "Bake queue:" -ForegroundColor Cyan
foreach ($item in $toBake) {
    $srcName = [System.IO.Path]::GetFileName($item.Source)
    $outName = [System.IO.Path]::GetFileName($item.Output)
    Write-Host "  $srcName -> $outName ($($item.Reason))"
}
Write-Host ""

if ($DryRun) {
    Write-Host "DRY RUN: No files were modified." -ForegroundColor Yellow
    exit 0
}

if (-not $toktx) {
    Write-Host "SKIPPED: Install toktx to perform actual baking." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Manual bake command for each texture:" -ForegroundColor Cyan
    foreach ($item in $toBake) {
        Write-Host "  toktx --t2 --encode uastc --genmipmap `"$($item.Output)`" `"$($item.Source)`""
    }
    exit 1
}

# Perform baking
Write-Host "Baking..." -ForegroundColor Cyan
foreach ($item in $toBake) {
    $srcName = [System.IO.Path]::GetFileName($item.Source)
    $outName = [System.IO.Path]::GetFileName($item.Output)
    Write-Host -NoNewline "  $srcName -> $outName ... "

    # Determine encoding based on texture type
    $isNormal = $srcName -match "_n\.(png|jpg)$"
    $isMRA = $srcName -match "_mra\.(png|jpg)$"

    $args = @("--t2", "--encode", "uastc", "--genmipmap")

    # Normal maps: linear color space
    if ($isNormal -or $isMRA) {
        $args += "--assign_oetf"
        $args += "linear"
    }

    $args += $item.Output
    $args += $item.Source

    try {
        $result = & toktx @args 2>&1
        if ($LASTEXITCODE -eq 0) {
            $newSize = (Get-Item $item.Output).Length
            Write-Host "OK ($([math]::Round($newSize / 1MB, 2)) MB)" -ForegroundColor Green
            $stats.Baked++
        }
        else {
            Write-Host "FAILED" -ForegroundColor Red
            Write-Host "    $result" -ForegroundColor Red
            $stats.Failed++
        }
    }
    catch {
        Write-Host "ERROR: $_" -ForegroundColor Red
        $stats.Failed++
    }
}

Write-Host ""
Write-Host "=== Rebake Summary ===" -ForegroundColor Cyan
Write-Host "  Baked successfully : $($stats.Baked)" -ForegroundColor Green
Write-Host "  Failed             : $($stats.Failed)" -ForegroundColor $(if ($stats.Failed -gt 0) { "Red" } else { "Green" })
Write-Host "  Already current    : $($stats.Current)" -ForegroundColor Green
Write-Host ""

if ($stats.Failed -gt 0) {
    exit 1
}
