param(
  [switch]$Force
)
$ErrorActionPreference = 'Stop'

function Ensure-Dir($p) { if (-not (Test-Path $p)) { New-Item -ItemType Directory -Path $p | Out-Null } }

$root = Join-Path $PSScriptRoot '..'
$assets = Join-Path $root 'assets'
$matRoot = Join-Path $assets 'materials/polyhaven'
$hdrRoot = Join-Path $assets 'hdri/polyhaven'

Ensure-Dir $matRoot
Ensure-Dir $hdrRoot

$materials = @(
  @{ name='aerial_rocks'; url='https://polyhaven.com/a/aerial_rocks_02' },
  @{ name='metal_plate'; url='https://polyhaven.com/a/metal_plate' },
  @{ name='plastered_wall'; url='https://polyhaven.com/a/plastered_wall' },
  @{ name='wood_floor'; url='https://polyhaven.com/a/wood_floor_deck' },
  @{ name='cobblestone'; url='https://polyhaven.com/a/cobblestone_floor_01' }
)

$hdris = @(
  @{ name='spruit_sunrise'; url='https://polyhaven.com/a/spruit_sunrise' },
  @{ name='kloppenheim'; url='https://polyhaven.com/a/kloppenheim_06_puresky' },
  @{ name='venice_sunset'; url='https://polyhaven.com/a/venice_sunset' }
)

# Download helper
function Get-Url($url, $dest) {
  if ((-not (Test-Path $dest)) -or $Force) {
    Write-Host "Downloading $url"
    Invoke-WebRequest -Uri $url -OutFile $dest
  } else {
    Write-Host "Exists      $dest"
  }
}

# Extract and normalize a material zip into standard names
function Import-MaterialZip($zipPath, $name) {
  $dst = Join-Path $matRoot $name
  Ensure-Dir $dst
  $tmp = Join-Path $dst 'tmp'
  Ensure-Dir $tmp
  Expand-Archive -Path $zipPath -DestinationPath $tmp -Force
  # Find candidate files
  $albedo = Get-ChildItem $tmp -Recurse -Include *_diff_*.*, *albedo* -ErrorAction SilentlyContinue | Select-Object -First 1
  $normal = Get-ChildItem $tmp -Recurse -Include *_nor_gl_*.*, *normal* -ErrorAction SilentlyContinue | Select-Object -First 1
  $rough  = Get-ChildItem $tmp -Recurse -Include *rough* -ErrorAction SilentlyContinue | Select-Object -First 1
  $metal  = Get-ChildItem $tmp -Recurse -Include *metal* -ErrorAction SilentlyContinue | Select-Object -First 1
  $ao     = Get-ChildItem $tmp -Recurse -Include *ao* -ErrorAction SilentlyContinue | Select-Object -First 1
  if ($albedo) { Copy-Item $albedo.FullName (Join-Path $dst 'albedo.png') -Force }
  if ($normal) { Copy-Item $normal.FullName (Join-Path $dst 'normal.png') -Force }
  if ($rough)  { Copy-Item $rough.FullName  (Join-Path $dst 'roughness.png') -Force }
  if ($metal)  { Copy-Item $metal.FullName  (Join-Path $dst 'metallic.png') -Force }
  if ($ao)     { Copy-Item $ao.FullName     (Join-Path $dst 'ao.png') -Force }
  Remove-Item $tmp -Recurse -Force
}

# Materials flow - create placeholder structure and instructions
Write-Host "`n=== PolyHaven Material Setup ===" -ForegroundColor Cyan
Write-Host "Creating placeholder structure for manual download...`n"

foreach ($m in $materials) {
  $dst = Join-Path $matRoot $m.name
  Ensure-Dir $dst
  
  # Create README with download instructions
  $readme = @"
# $($m.name) - PolyHaven Material

Download from: $($m.url)

Instructions:
1. Visit the URL above
2. Select resolution: 2K PNG recommended
3. Download the ZIP file
4. Extract these files to this directory:
   - Diffuse/Albedo → rename to: albedo.png
   - Normal (OpenGL) → rename to: normal.png
   - Roughness → rename to: roughness.png
   - Metallic → rename to: metallic.png (if available)
   - AO → rename to: ao.png (if available)

License: CC0 (Public Domain)
"@
  $readme | Set-Content (Join-Path $dst 'README.md')
  Write-Host "Created: $dst/README.md" -ForegroundColor Green
}

# HDRIs - similar placeholder approach
Write-Host "`n=== PolyHaven HDRI Setup ===" -ForegroundColor Cyan
foreach ($h in $hdris) {
  $dst = Join-Path $hdrRoot "$($h.name)"
  Ensure-Dir $dst
  
  $readme = @"
# $($h.name) - PolyHaven HDRI

Download from: $($h.url)

Instructions:
1. Visit the URL above
2. Select resolution: 2K HDR recommended
3. Download the .hdr or .exr file
4. Rename to: $($h.name).hdr and place in this directory

License: CC0 (Public Domain)
"@
  $readme | Set-Content (Join-Path $dst 'README.md')
  Write-Host "Created: $dst/README.md" -ForegroundColor Green
}

# Update materials.toml with template entries
$matToml = Join-Path $matRoot 'materials.toml'
$lines = @()
$lines += "# PolyHaven Material Catalog"
$lines += "# Download materials from polyhaven.com and follow README.md in each folder"
$lines += "# CC0 License - Free for any use"
$lines += ""
foreach ($m in $materials) {
  $matName = $m.name
  $lines += "[material.$matName]"
  $lines += "# Download from: $($m.url)"
  $lines += "albedo = `"$matName/albedo.png`""
  $lines += "normal = `"$matName/normal.png`""
  $lines += "roughness = `"$matName/roughness.png`""
  $lines += "# metallic = `"$matName/metallic.png`"  # if available"
  $lines += "# ao = `"$matName/ao.png`"  # if available"
  $lines += ""
}
$lines | Set-Content -Encoding UTF8 $matToml
Write-Host "`nWrote $matToml" -ForegroundColor Green

Write-Host "`n=== Next Steps ===" -ForegroundColor Yellow
Write-Host "1. Visit the README.md files in each material folder"
Write-Host "2. Download textures from PolyHaven following the instructions"
Write-Host "3. Uncomment relevant lines in materials.toml as you add textures"
Write-Host "4. The renderer will load materials from: $matToml"
Write-Host "`nAll materials are CC0 licensed - free for any use!`n"
