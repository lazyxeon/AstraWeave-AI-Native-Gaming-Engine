# AstraWeave Unified Showcase - Texture System

This document describes the texture system implemented for the AstraWeave-AI-Native-Gaming-Engine unified showcase example.

## System Components

The texture system consists of several components:

1. **TextureManager** - Core component that loads, manages, and organizes textures into atlases
2. **MaterialLibrary** - PBR material system with albedo, metallic, roughness, and normal mapping
3. **EnhancedShader** - Advanced WGSL shader with PBR lighting and normal mapping
4. **TextureUtils** - Utility functions for texture resolution and format conversion
5. **Mesh System** - Updated mesh system with proper UV coordinates and tangent space calculations

## Directory Structure

The textures are organized in the following directory structure:

```
assets/textures/
├── atlas_config.toml        # Configuration for texture atlases
├── atlas_diffuse.png        # Generated diffuse texture atlas
├── atlas_normal.png         # Generated normal map atlas
├── materials.json           # PBR material definitions
├── terrain/                 # Terrain textures (grass, dirt, sand, etc.)
├── structures/              # Building and object textures
├── characters/              # Character and NPC textures
├── skybox/                  # Skybox cube map textures
└── effects/                 # Visual effect textures
```

## Texture Atlas Configuration

The `atlas_config.toml` file defines how individual textures are packed into atlases:

```toml
# Texture Atlas Configuration
[atlas]
name = "unified_showcase_atlas"
description = "Complete texture atlas for the unified showcase demo"
version = "1.0.0"
dimensions = [2048, 2048]  # Width, height of the atlas

# Terrain textures
[terrain.grass]
diffuse = "terrain/grass.png"
normal = "terrain/grass_n.png"
roughness = 0.7
uv_rect = [0, 0, 256, 256]  # x, y, width, height in atlas

[terrain.dirt]
diffuse = "terrain/dirt.png"
normal = "terrain/dirt_n.png"
roughness = 0.8
uv_rect = [256, 0, 256, 256]
```

## Material System

The PBR material system provides:

1. **Material Properties**:
   - Base color (albedo)
   - Roughness
   - Metallic
   - Normal mapping
   - Emissive
   - Alpha mode (opaque, masked, blend)

2. **Environment-Specific Materials**:
   - Grassland (green grass, dirt, wood structures)
   - Desert (sand, stone, adobe)
   - Forest (dense vegetation, forest floor, log structures)

3. **Material GPU Representation**:
   - Compact representation for shader uniforms
   - Efficient flag-based feature toggles
   - Texture binding support

## Enhanced Shader Features

The enhanced WGSL shader (`enhanced_shader.wgsl`) provides:

1. **PBR Lighting**:
   - Cook-Torrance BRDF
   - GGX distribution
   - Smith geometry term
   - Fresnel-Schlick approximation

2. **Normal Mapping**:
   - Tangent space to world space transformation
   - Detail enhancement for terrain and objects
   - Adjustable normal intensity

3. **Advanced Effects**:
   - Sky atmosphere rendering
   - Time-of-day lighting changes
   - Environment-based terrain blending
   - Fog and distance effects

## Utility Tools

The texture system includes several utility tools:

1. **generate_atlas.rs** - Generates texture atlases from individual textures
2. **generate_textures.rs** - Procedurally generates textures using noise functions
3. **texture_system_demo.rs** - Demonstrates the texture system functionality
4. **texture_utils.rs** - Provides utility functions for texture operations

## Integration with Mesh System

The mesh system has been updated to provide proper UV coordinates and tangent vectors for normal mapping:

1. All meshes include:
   - Position
   - Normal vector
   - UV coordinates
   - (Tangent vectors calculated at runtime)

2. Custom UV mapping for:
   - Terrain meshes
   - Tree trunks and foliage
   - Buildings (walls and roofs)
   - Characters (with UV atlas for different body parts)
   - Skybox

## Usage in Application

To use the texture system in your application:

1. **Initialize**:
   ```rust
   // Load texture manager with atlas configuration
   let texture_manager = TextureManager::new(Path::new("assets/textures/atlas_config.toml"))?;
   
   // Load material library
   let mut material_library = MaterialLibrary::new();
   material_library.create_environment_materials("grassland");
   ```

2. **Render with PBR Materials**:
   ```rust
   // Get material for an object
   let material = material_library.get_material("grass");
   
   // Convert to GPU representation
   let material_gpu = MaterialGpu::from(material);
   
   // Update uniform buffer and bind textures
   queue.write_buffer(&material_buffer, 0, bytemuck::cast_slice(&[material_gpu]));
   ```

3. **Switch Environments**:
   ```rust
   // Change to desert environment
   material_library.create_environment_materials("desert");
   ```

## Future Improvements

Planned enhancements for the texture system:

1. **Runtime Texture Streaming** - Load textures on demand based on camera distance
2. **Procedural Material System** - Generate textures and materials procedurally
3. **Material Instancing** - Share base materials with instance-specific parameters
4. **Texture Compression** - Support for KTX2 with basis compression
5. **Custom Material Nodes** - Material graph system for complex surface effects