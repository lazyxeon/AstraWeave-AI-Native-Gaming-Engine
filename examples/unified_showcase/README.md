# AstraWeave Unified Showcase Demo

This demo showcases multiple AstraWeave engine features in one interactive 3D scene, demonstrating the integration of all core engine components.

## Features Demonstrated

### 🎮 Interactive 3D Scene

### 🔧 Engine Components

### 🎯 Enhanced Biome Experience

## Controls

| Input | Action |
|-------|--------|
| **Right Mouse + Move** | Look around / Camera pivot (smooth rotation) |
| **WASD** | Move camera (forward/left/back/right) |
| **Space** | Move camera up |
| **Ctrl** | Move camera down |
| **Mouse Wheel** | Zoom in/out (FreeFly: FOV, Orbit: distance) |
| **C** | Toggle camera mode (FreeFly ↔ Orbit) |
| **P** | Pause/resume physics simulation |
| **T** | Teleport sphere to camera position |
| **1** | Switch to grassland texture pack |
| **2** | Switch to desert texture pack |
| **3** | Switch to forest texture pack |
| **Esc** | Exit demo |

## Running the Demo

### Prerequisites
Ensure you have the required system dependencies installed:

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install -y build-essential pkg-config cmake ninja-build \
  libx11-dev libxi-dev libxcursor-dev libxrandr-dev libxinerama-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libx11-xcb-dev libxcb1-dev \
  libxcb-randr0-dev libxcb-xfixes0-dev libxcb-shape0-dev libxcb-xkb-dev \
  libgl1-mesa-dev libegl1-mesa-dev wayland-protocols libwayland-dev \
  libasound2-dev libpulse-dev libudev-dev mesa-vulkan-drivers vulkan-tools
```

### Build and Run

```bash
# Debug build (faster compilation)
cargo run -p unified_showcase

# Release build (better performance)
cargo run -p unified_showcase --release
```

### Optional Audio
To enable spatial audio, place an OGG audio file at `assets/sound.ogg`. The demo will automatically detect and play it with 3D spatial positioning.

## Implementation Details

### Architecture
The demo follows a self-contained design pattern that can easily be adapted to use the full AstraWeave engine crates:


### Performance

### Extensibility
The demo provides a foundation for adding:

## Code Structure

```
examples/unified_showcase/
├── Cargo.toml          # Dependencies and configuration
└── src/
    └── main.rs         # Complete demo implementation
                        # ├── Renderer setup (WGPU pipeline)
                        # ├── Physics world (Rapier3D)
                        # ├── Camera system (first-person)
                        # ├── Input handling (keyboard/mouse)
                        # ├── Audio system (Rodio)
                        # └── Main loop (event handling)
```

## Integration with AstraWeave

This demo serves as a reference implementation for integrating multiple engine systems. Key patterns demonstrated:

1. **Modular System Design**: Each subsystem (rendering, physics, audio) is cleanly separated
2. **Data Flow**: Efficient synchronization between physics simulation and rendering
3. **Event Handling**: Comprehensive input system with multiple interaction modes
4. **Performance Monitoring**: Built-in FPS tracking and status reporting

## Troubleshooting

### Graphics Issues

### Audio Issues

### Performance Issues

## Development Notes

This demo showcases the AstraWeave engine's capabilities while maintaining simplicity and readability. It serves as both a technical demonstration and a starting point for developers building games with the AstraWeave ecosystem.

The implementation emphasizes:

# Unified Showcase (HDR + PBR + Bloom)

A UE5-inspired physically based rendering demo using a two-pass HDR pipeline with ACES-like tonemapping, auto-exposure, multi-mip bloom, soft shadows, triplanar texturing, and a cube skybox.

## Build & Run (Windows PowerShell)

- Build and run only this example binary:

```
cargo run -p unified_showcase --release --bin unified_showcase
```

- Optional: enable wgpu logs for troubleshooting:

```
$env:RUST_LOG = "wgpu_core=warn,wgpu=info"; cargo run -p unified_showcase --release --bin unified_showcase
```

## Controls

- Movement/camera: `W/A/S/D` + mouse, right-click to look
- Camera mode toggle: `C`
- Physics pause: `P`
- Teleport sphere: `T`
- Apply impulse: `E`
- Texture packs: `1` = grassland, `2` = desert, `3` = forest
- Exposure: `=` to increase, `-` to decrease
- Bloom threshold: `[` decrease, `]` increase
- Bloom intensity: `;` decrease, `'` increase
- Exit: `Esc`

## Features

- Two-pass pipeline: HDR offscreen (Rgba16Float) → post to swapchain
- Full mip chain with runtime mip generation (used by bloom and auto-exposure)
- ACES-like tone mapping and auto-exposure in post
- Multi-mip bloom (thresholded, intensity adjustable)
- Cook–Torrance GGX with Schlick Fresnel + correlated Smith
- Triplanar sampling with micro-variation; anisotropic filtering = 16
- 16-tap Poisson PCF soft shadows
- Improved sky (large inverted cube skybox) and height fog

## Troubleshooting

- If you see validation errors about views or uniform sizes, ensure you’re on `wgpu 0.20` and use the exact bind groups shipped here; we separate a single-mip resolve view for main pass and a full-mip view for post.
- On first run, shader compilation may take a moment; keep the window focused.
- Avoid building the full workspace due to known broken examples. Target this bin explicitly as shown above.

## Known Limitations

- Some other binaries in this package may not compile; always target `--bin unified_showcase` for this demo.
- Visuals are tuned for demonstration and may be adjusted per GPU/driver.