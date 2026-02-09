# astraweave-audio

Spatial audio engine with dialogue runtime and text-to-speech adapter.

## Overview

Built on **rodio**, provides a 4-bus mixer (master, music, SFX, voice) with 3D spatial panning, distance attenuation, dialogue audio playback, and TTS adapter support.

## Key Types

| Type | Description |
|------|-------------|
| `AudioEngine` | 4-bus mixer with 3D spatial panning |
| `DialoguePlayer` | Dialogue audio playback |
| `DialogueAudioMap` | Character → audio file mapping |
| `VoiceBank` | Voice sample management |
| `TtsAdapter` | Text-to-speech trait |

## Modules

- **`engine`** — `AudioEngine` with master/music/SFX/voice buses
- **`dialogue_runtime`** — Dialogue playback with character mapping
- **`voice`** — Voice bank loading and TTS adapter

## Feature Flags

| Feature | Description |
|---------|-------------|
| `mock_tts` | `SimpleSineTts` for testing without real TTS |

## License

MIT
