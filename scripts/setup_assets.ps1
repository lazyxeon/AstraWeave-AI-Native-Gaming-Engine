param()

$ErrorActionPreference = 'Stop'

$root = Join-Path $PSScriptRoot '..'
$assets = Join-Path $root 'assets'

$dirs = @(
  'assets/hdri/polyhaven',
  'assets/materials/polyhaven',
  'assets/models/polyhaven',
  'assets/cache/ktx2'
)

foreach ($d in $dirs) {
  $path = Join-Path $root $d
  if (-not (Test-Path $path)) {
    New-Item -ItemType Directory -Path $path | Out-Null
    Write-Host "Created $path"
  } else {
    Write-Host "Exists  $path"
  }
}

# Starter materials.toml if missing
$matToml = Join-Path $assets 'materials/polyhaven/materials.toml'
if (-not (Test-Path $matToml)) {
  @"
# PolyHaven Material Catalog (starter)
# Add entries under [material.<name>]

[material.default]
albedo = "default/albedo.png"
normal = "default/normal.png"
roughness = "default/roughness.png"
metallic  = "default/metallic.png"
"@ | Set-Content -Encoding UTF8 $matToml
  Write-Host "Wrote $matToml"
} else {
  Write-Host "Exists  $matToml"
}
