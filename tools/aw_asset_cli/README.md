# AstraWeave Asset Pipeline CLI

A command-line tool for processing and optimizing game assets for the AstraWeave engine.

## Features

- Texture processing (PNG/JPEG → KTX2/Basis)
- Model processing (glTF/GLB → meshbin)
- Audio processing (WAV/MP3/OGG/FLAC → Ogg Vorbis)
- Asset manifest generation with SHA-256 hashes
- Configurable pipeline rules

## Usage

```bash
# Using default config (aw_pipeline.toml)
cargo run -p aw_asset_cli

# Using custom config
cargo run -p aw_asset_cli my_pipeline_config.toml
```

## Configuration

The pipeline is configured using a TOML file:

```toml
source = "assets_src"
output = "assets"

[[rules]]
kind = "texture"
glob = "**/*.{png,jpg,jpeg}"
normal_map = false

[[rules]]
kind = "model"
glob = "**/*.{gltf,glb}"

[[rules]]
kind = "audio"
glob = "**/*.{wav,mp3,ogg,flac}"
```

## External Tools

The pipeline will use external tools if available:

- `toktx` for KTX2 texture conversion
- `basisu` as a fallback for texture conversion
- `oggenc` for audio conversion

If these tools are not available, the pipeline will fall back to simpler methods.

## Output

The pipeline generates:

1. Processed assets in the output directory
2. A `manifest.json` file with metadata and SHA-256 hashes

Example manifest:

```json
[
  {
    "src": "assets_src/textures/grass.png",
    "out": "assets/textures/grass.ktx2",
    "sha256": "a1b2c3d4e5f6...",
    "kind": "texture"
  },
  {
    "src": "assets_src/models/rock.glb",
    "out": "assets/models/rock.meshbin",
    "sha256": "f6e5d4c3b2a1...",
    "kind": "model"
  }
]
```

This manifest can be used for asset loading, caching, and version control.