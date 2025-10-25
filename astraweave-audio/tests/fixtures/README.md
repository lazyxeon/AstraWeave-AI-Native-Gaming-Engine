# Test Fixtures for astraweave-audio Integration Tests

This directory contains minimal audio files for integration testing.

## Files

### Real Audio Files (User-Provided)
For full integration tests, place these files in this directory:

1. **music_test.ogg** (5 sec looped track, ~50 KB)
   - Format: OGG Vorbis
   - Duration: 5 seconds
   - Purpose: Music crossfade, loop boundary tests

2. **sfx_test.wav** (1 sec sound effect, ~30 KB)
   - Format: WAV PCM
   - Duration: 1 second
   - Purpose: SFX playback, spatial audio tests

3. **voice_test.wav** (2 sec voice line, ~40 KB)
   - Format: WAV PCM
   - Duration: 2 seconds
   - Purpose: Voice playback, subtitle callback tests

**Note**: These files are NOT checked into git (too large, binary). Tests will gracefully skip if files are missing.

## Generating Test Files

### Option 1: Using ffmpeg (Recommended)
```bash
# Generate 440 Hz tone as music_test.ogg (5 sec)
ffmpeg -f lavfi -i "sine=frequency=440:duration=5" -c:a libvorbis music_test.ogg

# Generate 880 Hz tone as sfx_test.wav (1 sec)
ffmpeg -f lavfi -i "sine=frequency=880:duration=1" -c:a pcm_s16le sfx_test.wav

# Generate 220 Hz tone as voice_test.wav (2 sec)
ffmpeg -f lavfi -i "sine=frequency=220:duration=2" -c:a pcm_s16le voice_test.wav
```

### Option 2: Using Audacity
1. Generate → Tone → 440 Hz, 5 sec → Export as OGG (music_test.ogg)
2. Generate → Tone → 880 Hz, 1 sec → Export as WAV (sfx_test.wav)
3. Generate → Tone → 220 Hz, 2 sec → Export as WAV (voice_test.wav)

### Option 3: Use Existing Audio
Copy any short audio files and rename them to match the expected filenames.

## Integration Test Behavior

**With Files Present**: Full integration tests run (crossfade, spatial audio, looping)

**Without Files**: Tests gracefully skip with `#[ignore]` or check for file existence

**CI/CD**: Tests should be marked as `#[ignore]` by default to avoid requiring audio files in CI.
