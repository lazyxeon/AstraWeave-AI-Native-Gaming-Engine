# Audio File Generation Script
# Run this to create test audio files for integration tests

Write-Host "=== AstraWeave Audio Test Fixture Generator ===" -ForegroundColor Cyan
Write-Host ""

$fixturesDir = "astraweave-audio\tests\fixtures"

# Check if ffmpeg is available
$ffmpegExists = Get-Command ffmpeg -ErrorAction SilentlyContinue

if ($ffmpegExists) {
    Write-Host "✅ ffmpeg found! Generating audio files..." -ForegroundColor Green
    
    # Generate music_test.ogg (5 sec, 440 Hz)
    Write-Host "  Generating music_test.ogg (5 sec, 440 Hz)..."
    ffmpeg -f lavfi -i "sine=frequency=440:duration=5" -c:a libvorbis -y "$fixturesDir\music_test.ogg" 2>$null
    
    # Generate sfx_test.wav (1 sec, 880 Hz)
    Write-Host "  Generating sfx_test.wav (1 sec, 880 Hz)..."
    ffmpeg -f lavfi -i "sine=frequency=880:duration=1" -c:a pcm_s16le -y "$fixturesDir\sfx_test.wav" 2>$null
    
    # Generate voice_test.wav (2 sec, 220 Hz)
    Write-Host "  Generating voice_test.wav (2 sec, 220 Hz)..."
    ffmpeg -f lavfi -i "sine=frequency=220:duration=2" -c:a pcm_s16le -y "$fixturesDir\voice_test.wav" 2>$null
    
    Write-Host "✅ Audio files generated successfully!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Files created:" -ForegroundColor Yellow
    Get-ChildItem "$fixturesDir\*.ogg", "$fixturesDir\*.wav" | ForEach-Object {
        $sizeKB = [math]::Round($_.Length / 1KB, 2)
        Write-Host "  - $($_.Name) ($sizeKB KB)"
    }
} else {
    Write-Host "⚠️  ffmpeg not found!" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Alternative methods to create test audio files:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Option 1: Install ffmpeg" -ForegroundColor White
    Write-Host "  - Download: https://ffmpeg.org/download.html"
    Write-Host "  - Or via winget: winget install ffmpeg"
    Write-Host "  - Or via chocolatey: choco install ffmpeg"
    Write-Host ""
    Write-Host "Option 2: Use Audacity (https://www.audacityteam.org/)" -ForegroundColor White
    Write-Host "  1. Generate → Tone → 440 Hz, 5 sec → Export as OGG (music_test.ogg)"
    Write-Host "  2. Generate → Tone → 880 Hz, 1 sec → Export as WAV (sfx_test.wav)"
    Write-Host "  3. Generate → Tone → 220 Hz, 2 sec → Export as WAV (voice_test.wav)"
    Write-Host ""
    Write-Host "Option 3: Use online tone generators" -ForegroundColor White
    Write-Host "  - Visit: https://www.szynalski.com/tone-generator/"
    Write-Host "  - Generate tones and download as WAV/OGG"
    Write-Host ""
    Write-Host "Option 4: Copy existing audio files" -ForegroundColor White
    Write-Host "  - Copy any short audio files and rename to:"
    Write-Host "    * music_test.ogg (any ~5 sec music)"
    Write-Host "    * sfx_test.wav (any ~1 sec sound effect)"
    Write-Host "    * voice_test.wav (any ~2 sec voice/sound)"
    Write-Host ""
    Write-Host "After creating files, run:" -ForegroundColor Cyan
    Write-Host "  cargo test -p astraweave-audio --test integration_tests -- --include-ignored"
    Write-Host ""
}

Write-Host "Current fixture status:" -ForegroundColor Cyan
$musicExists = Test-Path "$fixturesDir\music_test.ogg"
$sfxExists = Test-Path "$fixturesDir\sfx_test.wav"
$voiceExists = Test-Path "$fixturesDir\voice_test.wav"

Write-Host "  music_test.ogg: $(if ($musicExists) { '✅ Present' } else { '❌ Missing' })"
Write-Host "  sfx_test.wav:   $(if ($sfxExists) { '✅ Present' } else { '❌ Missing' })"
Write-Host "  voice_test.wav: $(if ($voiceExists) { '✅ Present' } else { '❌ Missing' })"
Write-Host ""

$allPresent = $musicExists -and $sfxExists -and $voiceExists
if ($allPresent) {
    Write-Host "✅ All test fixtures present! Integration tests can run." -ForegroundColor Green
    Write-Host ""
    Write-Host "Run tests with:" -ForegroundColor Cyan
    Write-Host "  cargo test -p astraweave-audio --test integration_tests -- --include-ignored"
} else {
    Write-Host "⚠️  Some fixtures missing. Integration tests will be skipped." -ForegroundColor Yellow
    Write-Host "    (Tests will still pass, but 8 tests will be ignored)" -ForegroundColor Gray
}
