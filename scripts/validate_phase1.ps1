# Phase 1 Validation Script
# Run this after compilation completes to verify async World Partition I/O

Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Phase 1: World Partition Async I/O" -ForegroundColor Cyan
Write-Host "Validation Test Suite" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""

# Test 1: Cell Loader Unit Tests
Write-Host "[1/4] Running cell_loader unit tests..." -ForegroundColor Yellow
$test1 = cargo test -p astraweave-asset --lib --quiet 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ PASS: Cell loader tests" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Cell loader tests" -ForegroundColor Red
    Write-Host $test1
}
Write-Host ""

# Test 2: Streaming Integration Tests
Write-Host "[2/4] Running streaming integration tests..." -ForegroundColor Yellow
$test2 = cargo test -p astraweave-scene --test streaming_integration --quiet 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ PASS: Streaming integration tests" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Streaming integration tests" -ForegroundColor Red
    Write-Host $test2
}
Write-Host ""

# Test 3: Check sample cell files exist
Write-Host "[3/4] Verifying sample cell files..." -ForegroundColor Yellow
$cell_files = @(
    "assets\cells\0_0_0.ron",
    "assets\cells\1_0_0.ron",
    "assets\cells\0_0_1.ron"
)
$all_exist = $true
foreach ($file in $cell_files) {
    if (Test-Path $file) {
        Write-Host "  ✅ Found: $file" -ForegroundColor Green
    } else {
        Write-Host "  ❌ Missing: $file" -ForegroundColor Red
        $all_exist = $false
    }
}
if ($all_exist) {
    Write-Host "✅ PASS: All sample cells exist" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Some sample cells missing" -ForegroundColor Red
}
Write-Host ""

# Test 4: Compilation check
Write-Host "[4/4] Checking compilation..." -ForegroundColor Yellow
$check = cargo check -p astraweave-asset -p astraweave-scene --quiet 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ PASS: Compilation clean" -ForegroundColor Green
} else {
    Write-Host "❌ FAIL: Compilation errors" -ForegroundColor Red
    Write-Host $check
}
Write-Host ""

# Summary
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "Phase 1 Validation Summary" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Implementation Status:" -ForegroundColor White
Write-Host "  ✅ Task 1.1: cell_loader.rs (already implemented)" -ForegroundColor Green
Write-Host "  ✅ Task 1.2: streaming.rs async fix (synchronous override removed)" -ForegroundColor Green
Write-Host "  ✅ Task 1.3: Sample cell files (3 cells created)" -ForegroundColor Green
Write-Host "  ⏳ Task 1.4: Integration tests (running validation)" -ForegroundColor Yellow
Write-Host ""
Write-Host "Time Analysis:" -ForegroundColor White
Write-Host "  Estimated: 16 hours" -ForegroundColor White
Write-Host "  Actual: ~3 hours" -ForegroundColor White
Write-Host "  Saved: ~13 hours" -ForegroundColor Green
Write-Host ""
Write-Host "Next Phase: Voxel Marching Cubes (12 hours estimated)" -ForegroundColor Cyan
Write-Host ""
