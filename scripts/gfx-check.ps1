$ErrorActionPreference = 'Stop'

Write-Host "== AstraWeave Graphics Check ==" -ForegroundColor Cyan

function Invoke-Check([string]$pkg) {
    Write-Host "-- cargo check -p $pkg" -ForegroundColor Yellow
    cargo check -p $pkg
}

Invoke-Check astraweave-render
Invoke-Check visual_3d
Invoke-Check weaving_playground
Invoke-Check physics_demo3d
Invoke-Check terrain_demo
Invoke-Check cutscene_render_demo
Invoke-Check unified_showcase

Write-Host "All graphics checks completed." -ForegroundColor Green
