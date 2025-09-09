# AstraWeave Engine Tools

This directory contains a set of tools designed to enhance the AstraWeave AI Native Gaming Engine with non-programmer authoring capabilities, asset pipeline management, and advanced debugging features.

## Overview

These tools implement three key pillars for the AstraWeave engine:

1. **Level & Encounter Editor** (`aw_editor`) - A GUI tool for non-programmers to create game content
2. **Asset Pipeline CLI** (`aw_asset_cli`) - A command-line tool for processing game assets
3. **Debug & Profiling Toolkit** (`aw_debug`) - A comprehensive toolkit for debugging and performance analysis

## Level & Encounter Editor

The Level & Encounter Editor provides a user-friendly interface for creating and editing game levels, encounters, and boss fights without requiring programming knowledge.

Key features:
- Visual editing of biomes, obstacles, and NPCs
- Fate thread system for trigger-based events
- Boss encounter configuration with Rhai scripting
- Hot-reload support for instant feedback

[Learn more about the Level & Encounter Editor](./aw_editor/README.md)

## Asset Pipeline CLI

The Asset Pipeline CLI processes and optimizes game assets for the AstraWeave engine, ensuring consistent and efficient asset loading.

Key features:
- Texture processing (PNG/JPEG → KTX2/Basis)
- Model processing (glTF/GLB → meshbin)
- Audio processing (WAV/MP3/OGG/FLAC → Ogg Vorbis)
- Asset manifest generation with SHA-256 hashes

[Learn more about the Asset Pipeline CLI](./aw_asset_cli/README.md)

## Debug & Profiling Toolkit

The Debug & Profiling Toolkit provides comprehensive tools for debugging, profiling, and monitoring the AstraWeave engine.

Key features:
- Performance HUD with FPS counter and frame time graph
- System timing visualization
- Entity count tracking
- Event logging system
- Chrome tracing integration
- Script hot-reload support

[Learn more about the Debug & Profiling Toolkit](./aw_debug/README.md)

## Integration with AstraWeave

These tools are designed to integrate seamlessly with the existing AstraWeave engine:

- The Level Editor saves levels in TOML format and generates Rhai scripts compatible with the engine's authoring system
- The Asset Pipeline produces optimized assets that can be loaded directly by the engine
- The Debug Toolkit integrates with the engine's existing egui overlay system

See the [debug_toolkit_demo](../examples/debug_toolkit_demo/README.md) for a complete integration example.

## Getting Started

To use these tools:

```bash
# Run the Level & Encounter Editor
cargo run -p aw_editor

# Process assets with the Asset Pipeline
cargo run -p aw_asset_cli

# Run the Debug Toolkit demo
cargo run -p debug_toolkit_demo
```

## Directory Structure

```
tools/
├── aw_editor/         # Level & Encounter Editor
├── aw_asset_cli/      # Asset Pipeline CLI
└── aw_debug/          # Debug & Profiling Toolkit
```

## Contributing

When contributing to these tools, please ensure:

1. All tools maintain compatibility with the core AstraWeave engine
2. The Level Editor remains accessible to non-programmers
3. The Asset Pipeline supports the engine's asset loading system
4. The Debug Toolkit integrates with the existing egui overlay system