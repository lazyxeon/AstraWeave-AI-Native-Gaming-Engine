# AstraWeave Unified Showcase Demo

This demo showcases multiple AstraWeave engine features in one interactive 3D scene, demonstrating the integration of all core engine components.

## Features Demonstrated

### 🎮 Interactive 3D Scene
- **Real-time 3D rendering** using WGPU with instanced geometry
- **Dynamic camera controls** with smooth movement and mouse look
- **Physics simulation** using Rapier3D with realistic object interactions
- **Spatial audio system** (optional, requires `assets/sound.ogg`)

### 🔧 Engine Components
- **Rendering System**: WGPU-based 3D graphics pipeline
- **Physics System**: Rapier3D rigid body dynamics
- **Navigation System**: Basic navmesh structures (expandable)
- **Audio System**: Spatial audio with Rodio
- **Input System**: Comprehensive keyboard and mouse controls

### 🎯 Scene Contents
- **Ground plane**: Large static physics collider
- **Box stack**: 25 dynamic rigid body cubes (5x5 grid, 5 layers high)
- **Bouncing sphere**: Dynamic sphere with continuous collision detection
- **Camera**: Free-flying first-person camera with physics-independent movement

## Controls

| Input | Action |
|-------|--------|
| **Right Mouse + Move** | Look around (first-person camera) |
| **WASD** | Move camera (forward/left/back/right) |
| **Space** | Move camera up |
| **Ctrl** | Move camera down |
| **Mouse Wheel** | Adjust camera movement speed |
| **P** | Pause/resume physics simulation |
| **T** | Teleport sphere to camera position |
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

- **Current**: Direct integration with `wgpu`, `rapier3d`, `rodio`, `winit`
- **Future**: Can be refactored to use `astraweave-render`, `astraweave-physics`, `astraweave-audio`, etc.

### Performance
- **Target**: 60 FPS at 1280x720
- **Rendering**: Single-pass forward rendering with depth testing
- **Physics**: Fixed 60Hz timestep with continuous collision detection
- **Memory**: Efficient instanced rendering for batch geometry

### Extensibility
The demo provides a foundation for adding:
- Advanced rendering effects (PBR materials, lighting, shadows)
- Complex physics interactions (joints, constraints, forces)
- Navigation mesh pathfinding with AI agents
- Procedural content generation
- Networking for multiplayer scenarios

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
- Ensure your GPU supports Vulkan or OpenGL
- Update graphics drivers
- Check console output for WGPU adapter information

### Audio Issues
- Verify `assets/sound.ogg` exists and is a valid OGG file
- Check system audio configuration
- Audio is optional - demo runs without it

### Performance Issues
- Use release build: `cargo run -p unified_showcase --release`
- Reduce resolution scale (future UI control)
- Close other GPU-intensive applications

## Development Notes

This demo showcases the AstraWeave engine's capabilities while maintaining simplicity and readability. It serves as both a technical demonstration and a starting point for developers building games with the AstraWeave ecosystem.

The implementation emphasizes:
- **Clarity**: Well-documented, readable code
- **Performance**: Efficient rendering and physics
- **Extensibility**: Easy to modify and expand
- **Best Practices**: Modern Rust patterns and engine design principles