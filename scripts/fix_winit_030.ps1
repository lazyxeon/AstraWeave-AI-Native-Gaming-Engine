# Script to fix winit 0.30 API breaking changes across the workspace

Write-Host "Fixing winit 0.30 API changes..." -ForegroundColor Cyan

# Find all Rust files in examples
$files = Get-ChildItem -Path "examples", "tools" -Recurse -Filter "*.rs" | Where-Object {
    $_.FullName -notlike "*\target\*"
}

$fixCount = 0

foreach ($file in $files) {
    $content = Get-Content $file.FullName -Raw
    $modified = $false
    $original = $content

    # Fix 1: WindowBuilder import (winit 0.30 moved it to application module)
    if ($content -match 'use winit::window::WindowBuilder') {
        $content = $content -replace 'use winit::window::WindowBuilder', 'use winit::application::ApplicationHandler'
        $content = $content -replace 'WindowBuilder::new\(\)', 'winit::window::Window::default_attributes()'
        $modified = $true
        Write-Host "  Fixed WindowBuilder import in: $($file.Name)" -ForegroundColor Yellow
    }

    # Fix 2: EventLoop::run() → EventLoop::run_app()
    if ($content -match 'event_loop\.run\(') {
        # This needs manual review as run_app requires ApplicationHandler trait
        Write-Host "  [MANUAL] EventLoop::run needs conversion to run_app in: $($file.Name)" -ForegroundColor Magenta
    }

    # Fix 3: egui State::new parameters (6 params now)
    if ($content -match 'egui_winit::State::new\(' -and $content -notmatch 'Some\(winit::window::Theme') {
        Write-Host "  [MANUAL] egui_winit::State::new needs theme parameter in: $($file.Name)" -ForegroundColor Magenta
    }

    # Fix 4: EguiRenderer::new (5 params now - added srgb_support bool)
    if ($content -match 'EguiRenderer::new\([^,]+,[^,]+,[^,]+,[^,]+\)' -and $content -notmatch 'EguiRenderer::new\([^,]+,[^,]+,[^,]+,[^,]+,[^)]+\)') {
        Write-Host "  [MANUAL] EguiRenderer::new needs 5th parameter (srgb_support) in: $($file.Name)" -ForegroundColor Magenta
    }

    # Fix 5: begin_frame → begin_pass, end_frame → end_pass
    if ($content -match '\.begin_frame\(') {
        $content = $content -replace '\.begin_frame\(', '.begin_pass('
        $modified = $true
        Write-Host "  Fixed begin_frame → begin_pass in: $($file.Name)" -ForegroundColor Yellow
    }

    if ($content -match '\.end_frame\(') {
        $content = $content -replace '\.end_frame\(', '.end_pass('
        $modified = $true
        Write-Host "  Fixed end_frame → end_pass in: $($file.Name)" -ForegroundColor Yellow
    }

    # Save if modified
    if ($modified -and $content -ne $original) {
        Set-Content -Path $file.FullName -Value $content -NoNewline
        $fixCount++
    }
}

Write-Host "`nAutomated fixes applied to $fixCount files" -ForegroundColor Green
Write-Host "Note: Some changes require manual review (marked [MANUAL])" -ForegroundColor Yellow
Write-Host "`nNext steps:" -ForegroundColor Cyan
Write-Host "1. Review EventLoop::run() → run_app() conversions (requires ApplicationHandler trait)"
Write-Host "2. Update WindowBuilder usage to Window::default_attributes()"
Write-Host "3. Add missing parameters to egui_winit::State::new() and EguiRenderer::new()"
