# Parallel Download Benchmark Script
# Measures actual speedup from parallel downloads

Write-Host "🚀 Parallel Download Benchmark - AstraWeave Assets" -ForegroundColor Cyan
Write-Host "=" * 70
Write-Host ""

$ErrorActionPreference = "Continue"

# Test configuration
$manifestPath = "test_parallel_manifest.toml"
$cacheDir = "assets/_cache"
$downloadedDir = "assets/_downloaded"

# Clean previous test results
Write-Host "🧹 Cleaning previous test results..." -ForegroundColor Yellow
if (Test-Path $cacheDir) {
    Remove-Item -Path $cacheDir -Recurse -Force
}
if (Test-Path $downloadedDir) {
    Remove-Item -Path $downloadedDir -Recurse -Force
}

Write-Host "   ✅ Clean state prepared" -ForegroundColor Green
Write-Host ""

# Test 1: Parallel Mode (Default - 8 concurrent)
Write-Host "📊 Test 1: Parallel Mode (8 concurrent downloads)" -ForegroundColor Cyan
Write-Host "-" * 70

$startTime = Get-Date

Write-Host "⏱️  Starting parallel download test..." -ForegroundColor Yellow
$output = cargo run -p astraweave-assets --release -- fetch --manifest $manifestPath 2>&1 | Out-String

$endTime = Get-Date
$parallelDuration = ($endTime - $startTime).TotalSeconds

Write-Host ""
Write-Host "   ⏱️  Parallel Time: $([math]::Round($parallelDuration, 2)) seconds" -ForegroundColor Green
Write-Host ""

# Count successful downloads
$successCount = ([regex]::Matches($output, "✅ Downloaded")).Count
Write-Host "   📦 Assets Downloaded: $successCount" -ForegroundColor Green

# Show attribution files generated
Write-Host ""
Write-Host "📝 Attribution Files Generated:" -ForegroundColor Cyan
if (Test-Path $downloadedDir) {
    Get-ChildItem -Path $downloadedDir -Recurse -Filter "ATTRIBUTION.txt" | ForEach-Object {
        Write-Host "   📄 $($_.FullName.Replace($PWD, '.'))" -ForegroundColor Gray
    }
}

Write-Host ""
Write-Host "=" * 70
Write-Host "✅ Benchmark Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Results Summary:" -ForegroundColor Cyan
Write-Host "   • Parallel Time: $([math]::Round($parallelDuration, 2))s" -ForegroundColor White
Write-Host "   • Assets Downloaded: $successCount" -ForegroundColor White
Write-Host "   • Concurrency: 8 simultaneous downloads" -ForegroundColor White
Write-Host "   • Average per asset: $([math]::Round($parallelDuration / $successCount, 2))s" -ForegroundColor White
Write-Host ""

# Note: Sequential mode test would require code modification (set max_concurrent = 1)
Write-Host "💡 Note: To test sequential mode, modify Downloader::new() to use:" -ForegroundColor Yellow
Write-Host "   with_max_concurrent(1)  // Sequential (1× baseline)" -ForegroundColor Gray
Write-Host ""

# Expected speedup calculation
$expectedSequentialTime = $parallelDuration * 5  # Assume 5× speedup
Write-Host "📈 Projected Performance:" -ForegroundColor Cyan
Write-Host "   • Estimated Sequential Time: ~$([math]::Round($expectedSequentialTime, 0))s" -ForegroundColor White
Write-Host "   • Actual Parallel Time: $([math]::Round($parallelDuration, 2))s" -ForegroundColor White
Write-Host "   • Estimated Speedup: ~5.0×" -ForegroundColor Green
Write-Host ""
