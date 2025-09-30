# Authored Materials: Canonical Arrays + WGSL Bindings

This engine uses authored material packs with stable texture arrays for albedo, normal, and MRA (Metallic/Roughness/AO). Packs live under one of these roots:

- assets/materials/<biome>/{materials.toml, arrays.toml}
- assets/textures/<biome>/{materials.toml, arrays.toml}

Prefer assets/materials as canonical. If both exist, the loader prioritizes assets/materials.

## Files
- materials.toml: declares layers and per-layer files (albedo, normal, mra or metallic/roughness/ao). Optional fields: tiling, triplanar_scale, atlas.
- arrays.toml: maps layer keys to fixed array indices. Indices must be stable to keep bindless-style references consistent across frames.

Example arrays.toml:

```
[layers]
grass = 0
sand = 1
rock = 2
roof_tile = 10
```

## GPU layout
- Albedo array: RGBA8 sRGB, D2Array, with mipmaps
- Normal array: RG8 (Z reconstructed in shader), D2Array, with mipmaps
- MRA array: RGBA8 (R=metallic, G=roughness, B=AO, A=unused), D2Array, with mipmaps
- Samplers: albedo (sRGB filtering), linear (for normal and MRA)
- Anisotropy: 16

## WGSL bindings (group=1)

```
@group(1) @binding(0) var material_albedo: texture_2d_array<f32>;
@group(1) @binding(1) var material_albedo_sampler: sampler;
@group(1) @binding(2) var material_normal: texture_2d_array<f32>;
@group(1) @binding(3) var material_normal_sampler: sampler; // also used for MRA
@group(1) @binding(4) var material_mra: texture_2d_array<f32>;
```

Normals are stored as RG; reconstruct Z in shader:

```
fn reconstruct_normal_from_rg(nrg: vec2<f32>) -> vec3<f32> {
  let nxy = nrg * 2.0 - 1.0;
  let nz = sqrt(max(0.0, 1.0 - dot(nxy, nxy)));
  return vec3<f32>(nxy, nz);
}
```

## Engine API
Use `MaterialManager::load_pack_from_toml` to build arrays and stats. For apps/examples, prefer `MaterialIntegrator` (see unified_showcase) which caches packs and provides a ready-to-bind `BindGroupLayout` and `BindGroup`.

Telemetry helper is available via `MaterialLoadStats::concise_summary()`.

## Fallbacks & packing
- Missing albedo/normal/MRA fall back to neutral patterns; warnings are logged.
- If `mra` is missing, but `metallic`, `roughness`, and `ao` are provided, the loader packs them into MRA (R/G/B) automatically.

## Hot reload
Examples can hot reload by re-calling the integrator’s `load` for the current biome and rebinding the material bind group. See unified_showcase (Shift+R).
