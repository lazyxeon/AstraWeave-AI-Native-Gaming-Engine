---
layout: default
title: Rendering Subsystem
---

# Rendering (astraweave-render)

AstraWeave's renderer is built on **wgpu 25.0.2**, providing cross-platform GPU rendering with Vulkan, DX12, and Metal backends. The crate contains 62+ source files covering PBR materials, multiple rendering paths, post-processing, mesh optimization, animation, and environment systems.

## Feature Matrix

| Feature | Status | Module |
|---------|--------|--------|
| PBR materials (Cook-Torrance BRDF) | ✅ | `material.rs`, `material_extended.rs` |
| Extended materials (clearcoat, anisotropy, sheen, subsurface, transmission) | ✅ | `material_extended.rs` |
| TOML → GPU material pipeline | ✅ | `material_loader.rs`, `biome_material.rs` |
| Cascaded shadow maps (4 cascades, 2048px) | ✅ | `shadow_csm.rs` |
| Clustered forward lighting | ✅ | `clustered_forward.rs` |
| Clustered megalights | ✅ | `clustered_megalights.rs` |
| Deferred rendering path with G-Buffer | ✅ | `deferred.rs` |
| Post-processing: Bloom | ✅ | `post.rs` |
| SSAO (multiple quality presets) | ✅ | `ssao.rs` |
| Advanced post-FX: TAA, motion blur, DoF, color grading | ✅ | `advanced_post.rs` |
| GPU skinning (dual bone influence) | ✅ | `skinning_gpu.rs` |
| Skeletal animation (clips, channels, joints) | ✅ | `animation.rs` |
| Vertex compression (37.5% savings) | ✅ | `vertex_compression.rs` |
| LOD generation (quadric error metrics) | ✅ | `lod_generator.rs` |
| Instanced rendering | ✅ | `instancing.rs` |
| Image-based lighting (IBL) | ✅ | `ibl.rs` |
| HDRI catalog with day/night cycle | ✅ | `hdri_catalog.rs` |
| Skybox (procedural + cubemap + equirectangular) | ✅ | `environment.rs` |
| Weather system (7 weather types) | ✅ | `environment.rs`, `weather_system.rs` |
| GPU particle system (compute shader) | ✅ | `gpu_particles.rs` |
| Water rendering | ✅ | `water.rs` |
| Decal system (atlas-based) | ✅ | `decals.rs` |
| Biome transition effects (6 easing functions) | ✅ | `biome_transition.rs` |
| Texture streaming | ✅ | `texture_streaming.rs` |
| GPU memory management | ✅ | `gpu_memory.rs`, `residency.rs` |
| GPU frustum culling | ✅ | `culling.rs`, `culling_node.rs` |
| MSAA | ✅ | `msaa.rs` |
| Transparency management | ✅ | `transparency.rs` |
| Render graph | ✅ | `graph.rs`, `graph_adapter.rs` |
| Nanite-style GPU culling | ✅ | `nanite_gpu_culling.rs` (feature-gated) |
| Mesh loaders: glTF, OBJ | ✅ | `mesh_gltf.rs`, `mesh_obj.rs` (feature-gated) |
| Volumetric fog | 📋 Planned | — |

## Camera System

```rust
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}
```

Methods: `view_matrix()`, `proj_matrix()`, `vp()`, `dir(yaw, pitch) -> Vec3`.

`CameraController` supports orbit and fly modes with keyboard, mouse, and scroll input:
- `process_keyboard()`, `process_mouse_delta()`, `process_scroll()`
- `toggle_mode()` — switch between FPS and orbit
- `set_orbit_target()` — focus on a world point
- `update_camera()` — apply all pending inputs

## Material System

Materials are defined in TOML and compiled to GPU D2 array textures:

```
assets/materials/<biome>/
├── materials.toml    # Material definitions
└── arrays.toml       # Texture array config
```

### Standard Material (GPU)

```rust
pub struct MaterialGpu {
    // Bitfield: FLAG_HAS_ALBEDO | FLAG_HAS_NORMAL | FLAG_HAS_ORM | FLAG_TRIPLANAR
}
```

### Extended Materials

Advanced PBR features via `MaterialDefinitionExtended`:

| Feature | Flag |
|---------|------|
| Clearcoat | `MATERIAL_FLAG_CLEARCOAT` |
| Anisotropy | `MATERIAL_FLAG_ANISOTROPY` |
| Sheen | `MATERIAL_FLAG_SHEEN` |
| Subsurface scattering | `MATERIAL_FLAG_SUBSURFACE` |
| Transmission | `MATERIAL_FLAG_TRANSMISSION` |

### Material Manager

`MaterialManager` handles GPU resource creation and caching:
- `get_or_create_bind_group_layout()` — cached bind group layout
- `create_bind_group()` — assemble texture arrays into GPU bind groups
- `current_stats()` — `MaterialLoadStats` with `concise_summary()`

**WGSL Bindings (group=1)**:

| Binding | Resource |
|---------|----------|
| 0 | Albedo texture array |
| 1 | Sampler |
| 2 | Normal map array |
| 3 | Linear sampler |
| 4 | MRA (Metallic/Roughness/AO) array |

## Rendering Paths

### Clustered Forward

Default path using clustered light assignment for efficient multi-light rendering:

```rust
let renderer = ClusteredForwardRenderer::new(&device, config);
renderer.update_lights(&lights);
renderer.build_clusters();
```

WGSL shader: `CLUSTERED_LIGHTING_SHADER`

### Deferred

G-Buffer based deferred rendering for scenes with many lights:

```rust
let deferred = DeferredRenderer::new(&device, size);
// Geometry pass fills G-Buffer
let gbuffer = deferred.gbuffer();
// Light pass reads G-Buffer
deferred.light_pass(&lights);
```

### Shadow Mapping (CSM)

4-cascade cascaded shadow maps (2048px resolution per cascade):

```rust
let csm = CsmRenderer::new(&device);
csm.update_cascades(camera, sun_direction);
csm.render_shadow_maps(&scene_objects);
```

Constants: `CASCADE_COUNT=4`, `CASCADE_RESOLUTION=2048`, `DEPTH_BIAS=0.005`

## Post-Processing Pipeline

### Bloom

```rust
let bloom = BloomPipeline::new(&device, bloom_config);
bloom.execute(&render_pass, &scene_texture);
```

4-stage WGSL pipeline: threshold → downsample → upsample → composite.

### SSAO

Screen-space ambient occlusion with quality presets:

```rust
pub enum SsaoQuality {
    Low,     // fewer samples, smaller radius
    Medium,
    High,    // more samples, larger blur kernel
}
```

### Advanced Post-FX

Feature-gated (`advanced-post`) pipeline with TAA, motion blur, depth of field, and color grading:

```rust
pub struct TaaConfig {
    pub enabled: bool,
    pub blend_factor: f32,    // default 0.95
    pub jitter_scale: f32,
}

pub struct DofConfig {
    pub enabled: bool,
    pub focus_distance: f32,  // default 10.0
    pub focus_range: f32,     // default 5.0
    pub bokeh_size: f32,      // default 2.0
}

pub struct MotionBlurConfig {
    pub enabled: bool,
    pub sample_count: u32,    // default 8
    pub strength: f32,
}

pub struct ColorGradingConfig {
    pub enabled: bool,
    pub exposure: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub temperature: f32,
    pub tint: f32,
}
```

## Environment System

### Time of Day

```rust
pub struct TimeOfDay {
    pub current_time: f32,
    pub time_scale: f32,
}
```

Methods: `get_sun_position()`, `get_moon_position()`, `get_light_direction()`, `get_light_color()`, `is_day()`, `is_night()`, `is_twilight()`.

### Sky Rendering

`SkyRenderer` supports three sky modes:
- **Procedural**: Rayleigh/Mie scattering with sun/moon discs and clouds
- **Textured cubemap**: Traditional skybox from 6 textures
- **Equirectangular HDRI**: Panoramic HDR image projection

Integrated with `HdriCatalog` for automatic day/night HDRI selection by `DayPeriod`.

### Weather System

7 weather types with full GPU particle effects:

```rust
pub enum WeatherType {
    Clear, Cloudy, Rain, Storm, Snow, Fog, Sandstorm,
}
```

`WeatherSystem`: `set_weather()`, `current_weather()`, `get_rain_intensity()`, and more.

## GPU Skinning

```rust
pub struct SkinnedVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub bone_indices: [u32; 2],  // dual bone influence
    pub bone_weights: [f32; 2],
}
```

Full skeletal animation pipeline: `AnimationClip` → `AnimationState` → `JointPalette` → GPU upload via `JointPaletteManager`. Supports up to `MAX_JOINTS` joints per skeleton.

## Mesh Optimization

| Optimization | Savings | Module |
|--------------|---------|--------|
| Vertex compression | 37.5% memory (32 → 20 bytes) | `vertex_compression.rs` |
| Octahedral normals | 12 → 2 bytes per normal | `vertex_compression.rs` |
| Half-float UVs | 8 → 4 bytes per UV pair | `vertex_compression.rs` |
| LOD generation | 3-5 levels via quadric error metrics | `lod_generator.rs` |
| Instanced rendering | 10–100× fewer draw calls | `instancing.rs` |
| GPU frustum culling | CPU-side + GPU-side culling | `culling.rs` |

### Instance Manager

```rust
let mut manager = InstanceManager::new(&device);
manager.add_instance(mesh_id, transform);
manager.update_buffers(&device, &queue);
// Batch rendered with a single draw call per mesh type
```

## Biome Transitions

Smooth world-space transitions between biome visual styles:

```rust
pub enum EasingFunction {
    Linear, SmoothStep, SmootherStep, EaseIn, EaseOut, EaseInOut,
}

pub struct BiomeVisuals {
    // Per-biome: fog color/density, sky tint, water color, cloud coverage
}

pub struct TransitionConfig {
    pub easing: EasingFunction,
    pub duration: f32,
}
```

## GPU Particle System

Double-buffered compute shader particle system:

```rust
pub struct GpuParticle {
    // 32 bytes, Pod + Zeroable for GPU mapping
}

pub struct EmitterParams {
    pub emission_rate: f32,
    pub gravity: f32,
    pub lifetime: f32,
}

let particles = GpuParticleSystem::new(&device, max_particles);
```

Uses ping-pong buffers for GPU-side simulation.

## Frame Budget

At 1,000 entities:

- **Frame time**: 2.70 ms
- **FPS**: 370
- **Budget headroom**: 84% vs 60 FPS target

## Test Coverage

- **806 tests** (lib + integration)
- Mutation testing campaigns for camera, weather, easing, and environment subsystems

[← Back to Home](index.html) · [Architecture](architecture.html) · [Physics](physics.html)
