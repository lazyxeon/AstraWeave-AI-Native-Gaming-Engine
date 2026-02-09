---
layout: default
title: Audio Subsystem
---

# Audio (astraweave-audio)

AstraWeave's audio engine provides spatial audio, a 4-bus mixer, dialogue runtime, and TTS integration built on **rodio 0.17**.

## Features

| Feature | Status |
|---------|--------|
| Master / Music / SFX / Voice buses | ✅ |
| Per-bus volume and mute | ✅ |
| Crossfading between tracks | ✅ |
| 3D spatial audio (HRTF-ready) | ✅ |
| Dialogue playback runtime | ✅ |
| Text-to-speech (TTS) overlay | ✅ |
| Audio occlusion (raycast) | 🔄 |
| Reverb zones | 📋 |

## Mixer Architecture

```
Sound Sources
   │
   ├── Music Bus ──────┐
   ├── SFX Bus ────────┤
   ├── Voice Bus ──────┤
   └── Ambient Bus ────┤
                       │
                  Master Bus ──► Output
```

Each bus supports:
- Independent volume (0.0–1.0)
- Mute toggle
- Crossfade transitions

## Spatial Audio

```rust
// Position a 3D sound source
audio_engine.play_spatial(
    "explosion.wav",
    position,  // world-space coordinates
    AudioBus::SFX,
);
```

## Dialogue Runtime

The dialogue system integrates with the AI subsystem for NPC conversations:

1. AI generates dialogue text via LLM or scripted trees
2. Audio engine plays voice clip or invokes TTS
3. UI displays subtitle text synced to audio playback
4. Player responses feed back into AI reasoning

## Integration Points

- **UI**: Volume sliders in settings menu (Phase 8.1 implemented)
- **Physics**: Occlusion raycasts use physics world
- **AI**: Dialogue generation connects to LLM persona system

[← Back to Home](index.html) · [Architecture](architecture.html) · [UI](ui.html)
