---
layout: default
title: Rendering Subsystem
---

# Rendering (astraweave-render)

AstraWeave's renderer is built on **wgpu 25.0.2**, providing cross-platform GPU rendering with Vulkan, DX12, and Metal backends.

## Features

| Feature | Status |
|---------|--------|
| PBR materials (Cook-Torrance BRDF) | ✅ |
| TOML → GPU material pipeline | ✅ |
| Cascaded shadow maps (CSM) | ✅ |
| Post-processing (bloom, tonemapping) | ✅ |
| GPU skinning (dual bone influence) | ✅ |
| Vertex compression (37.5% savings) | ✅ |
| LOD generation (quadric error metrics) | ✅ |
| Instanced rendering | ✅ |
| Image-based lighting (IBL) | ✅ |
| Skybox & atmospheric scattering | 🔄 |
| GPU particle system | 🔄 |
| Volumetric fog | 📋 |

## Material System

Materials are defined in TOML and compiled to GPU D2 array textures:

```
assets/materials/<biome>/
├── materials.toml    # Material definitions
└── arrays.toml       # Texture array config
```

**WGSL Bindings (group=1)**:

| Binding | Resource |
|---------|----------|
| 0 | Albedo texture array |
| 1 | Sampler |
| 2 | Normal map array |
| 3 | Linear sampler |
| 4 | MRA (Metallic/Roughness/AO) array |

## GPU Skinning

```rust
// SkinnedVertex with dual bone influence
pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bone_indices: [u32; 2],
    pub bone_weights: [f32; 2],
}
```

## Mesh Optimization

| Optimization | Savings | Module |
|--------------|---------|--------|
| Vertex compression | 37.5% memory | `vertex_compression.rs` |
| Octahedral normals | 12→2 bytes | `vertex_compression.rs` |
| Half-float UVs | 8→4 bytes | `vertex_compression.rs` |
| LOD generation | 3-5 levels | `lod_generator.rs` |
| Instanced rendering | 10-100× fewer draw calls | `instancing.rs` |

## Frame Budget

At 1,000 entities:

- **Frame time**: 2.70 ms
- **FPS**: 370
- **Budget headroom**: 84% vs 60 FPS target

[← Back to Home](index.html) · [Architecture](architecture.html) · [Physics](physics.html)
