# Skinning Demo

**Phase 2 Task 5 (Phase F)**: Interactive skeletal animation demonstration

## Features

- **Skeletal Animation**: Load and playback skeletal animation clips
- **CPU Skinning (Default)**: Deterministic, CI-friendly animation playback
- **GPU Skinning (Optional)**: High-performance GPU-accelerated skinning
- **Bone Attachments**: Visualize entities attached to skeleton joints
- **HUD Overlay**: Real-time statistics and current state

## Controls

| Key | Action |
|-----|--------|
| **Space** | Play/Pause animation |
| **[** | Slow down (0.5× speed) |
| **]** | Speed up (2.0× speed) |
| **R** | Reset to t=0 |
| **G** | Toggle CPU/GPU (requires `--features skinning-gpu`) |
| **ESC** | Exit |

## Running

```powershell
# CPU skinning (default, deterministic)
cargo run -p skinning_demo

# GPU skinning (requires hardware)
cargo run -p skinning_demo --features skinning-gpu

# Release mode (better performance)
cargo run -p skinning_demo --release
```

## HUD Information

- **Mode**: CPU or GPU skinning
- **Joint Count**: Number of skeleton joints
- **Clip**: Current animation clip name
- **Time**: Current animation time / duration
- **Speed**: Playback speed multiplier
- **Status**: Playing or Paused

## Implementation Notes

- Uses simple procedural skeleton (3 joints) for demonstration
- Animation: 90° rotation over 2 seconds
- Deterministic by default (CPU path)
- GPU path requires `skinning-gpu` feature flag
