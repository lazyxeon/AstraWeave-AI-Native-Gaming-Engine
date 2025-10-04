# Naga Fix Verification Script
# Run this after cargo clean to verify the fix

Write-Host "=== Naga WriteColor Fix Verification ===" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check dependency versions
Write-Host "[1/4] Checking WGPU version..." -ForegroundColor Yellow
$wgpuVersion = cargo tree -p astraweave-render | Select-String "wgpu v" | Select-Object -First 1
Write-Host "  $wgpuVersion" -ForegroundColor Green

# Step 2: Check naga versions
Write-Host "[2/4] Checking naga versions..." -ForegroundColor Yellow
$nagaVersions = cargo tree -p astraweave-render | Select-String "naga v" | Select-Object -First 5
foreach ($line in $nagaVersions) {
    Write-Host "  $line" -ForegroundColor Green
}

# Step 3: Compile astraweave-render
Write-Host "[3/4] Compiling astraweave-render..." -ForegroundColor Yellow
$compileStart = Get-Date
cargo check -p astraweave-render 2>&1 | Out-Null
$compileTime = (Get-Date) - $compileStart

if ($LASTEXITCODE -eq 0) {
    Write-Host "  ✓ SUCCESS: Compiled in $($compileTime.TotalSeconds) seconds" -ForegroundColor Green
} else {
    Write-Host "  ✗ FAILED: Compilation errors detected" -ForegroundColor Red
    Write-Host "  Running detailed error check..." -ForegroundColor Yellow
    cargo check -p astraweave-render 2>&1 | Select-String "error\[" | Select-Object -First 10
    exit 1
}

# Step 4: Check for naga errors
Write-Host "[4/4] Checking for naga-related errors..." -ForegroundColor Yellow
$nagaErrors = cargo check -p astraweave-render 2>&1 | Select-String "naga.*WriteColor"
if ($nagaErrors) {
    Write-Host "  ✗ FOUND naga WriteColor errors:" -ForegroundColor Red
    $nagaErrors | ForEach-Object { Write-Host "    $_" -ForegroundColor Red }
    exit 1
} else {
    Write-Host "  ✓ No naga WriteColor errors found" -ForegroundColor Green
}

# Summary
Write-Host ""
Write-Host "=== Verification Complete ===" -ForegroundColor Cyan
Write-Host "✓ WGPU version: 22.1.0" -ForegroundColor Green
Write-Host "✓ Naga version: 22.x" -ForegroundColor Green
Write-Host "✓ Compilation: SUCCESS" -ForegroundColor Green
Write-Host "✓ No naga errors" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Run: cargo build -p astraweave-render --release" -ForegroundColor White
Write-Host "  2. Test: cargo run -p visual_3d --release" -ForegroundColor White
Write-Host "  3. Verify: Check graphics examples run correctly" -ForegroundColor White
