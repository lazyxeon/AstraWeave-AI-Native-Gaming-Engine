// Adapted from Bevy Engine (https://github.com/bevyengine/bevy)
// Copyright (c) 2020 Carter Anderson
// Licensed under MIT OR Apache-2.0
// Original: bevy/crates/bevy_pbr/src/render/shadows.wgsl
//
// MODIFICATIONS for AstraWeave:
// - Simplified to directional lights only (removed point/spot light functions)
// - Adapted uniform bindings to match AstraWeave's CsmRenderer
// - Removed mesh_view_bindings dependency (use direct cascade uniforms)

// Cascade shadow map data (matches astraweave-render/src/shadow_csm.rs)
struct CascadeData {
    view_proj: mat4x4<f32>,
    texel_size: f32,
    far_bound: f32,
    _padding: vec2<f32>,
}

// Matches CsmUniforms in shadow_csm.rs
struct CsmUniforms {
    cascades: array<CascadeData, 4>,
    num_cascades: u32,
    shadow_normal_bias: f32,
    shadow_depth_bias: f32,
    cascades_overlap_proportion: f32,
    direction_to_light: vec3<f32>,
    _padding: f32,
}

@group(1) @binding(0) var<uniform> csm: CsmUniforms;
@group(1) @binding(1) var shadow_atlas: texture_depth_2d_array;
@group(1) @binding(2) var shadow_sampler: sampler_comparison;

// Get cascade index based on view-space depth
fn get_cascade_index(view_z: f32) -> u32 {
    for (var i: u32 = 0u; i < csm.num_cascades; i = i + 1u) {
        if (-view_z < csm.cascades[i].far_bound) {
            return i;
        }
    }
    return csm.num_cascades;
}

// Converts from world space to the uv position in the light's shadow map.
// The depth is stored in the return value's z coordinate. If the return value's
// w coordinate is 0.0, then we landed outside the shadow map entirely.
fn world_to_directional_light_local(
    cascade_index: u32,
    offset_position: vec4<f32>
) -> vec4<f32> {
    let cascade = &csm.cascades[cascade_index];
    
    let offset_position_clip = (*cascade).view_proj * offset_position;
    if (offset_position_clip.w <= 0.0) {
        return vec4(0.0);
    }
    let offset_position_ndc = offset_position_clip.xyz / offset_position_clip.w;
    // No shadow outside the orthographic projection volume
    if (any(offset_position_ndc.xy < vec2<f32>(-1.0)) || offset_position_ndc.z < 0.0
            || any(offset_position_ndc > vec3<f32>(1.0))) {
        return vec4(0.0);
    }

    // Compute texture coordinates for shadow lookup, compensating for the Y-flip difference
    // between the NDC and texture coordinates
    let flip_correction = vec2<f32>(0.5, -0.5);
    let light_local = offset_position_ndc.xy * flip_correction + vec2<f32>(0.5, 0.5);

    let depth = offset_position_ndc.z;

    return vec4(light_local, depth, 1.0);
}

// Hardware PCF shadow sampling
fn sample_shadow_map_hardware(light_local: vec2<f32>, depth: f32, cascade_index: i32) -> f32 {
    return textureSampleCompareLevel(
        shadow_atlas,
        shadow_sampler,
        light_local,
        cascade_index,
        depth,
    );
}

// Castano '13 13-tap PCF
// https://web.archive.org/web/20230210095515/http://the-witness.net/news/2013/09/shadow-mapping-summary-part-1
fn sample_shadow_map_pcf(light_local: vec2<f32>, depth: f32, cascade_index: i32) -> f32 {
    let shadow_map_size = vec2<f32>(textureDimensions(shadow_atlas));
    let inv_shadow_map_size = 1.0 / shadow_map_size;

    let uv = light_local * shadow_map_size;
    var base_uv = floor(uv + 0.5);
    let s = (uv.x + 0.5 - base_uv.x);
    let t = (uv.y + 0.5 - base_uv.y);
    base_uv -= 0.5;
    base_uv *= inv_shadow_map_size;

    let uw0 = (4.0 - 3.0 * s);
    let uw1 = 7.0;
    let uw2 = (1.0 + 3.0 * s);

    let u0 = (3.0 - 2.0 * s) / uw0 - 2.0;
    let u1 = (3.0 + s) / uw1;
    let u2 = s / uw2 + 2.0;

    let vw0 = (4.0 - 3.0 * t);
    let vw1 = 7.0;
    let vw2 = (1.0 + 3.0 * t);

    let v0 = (3.0 - 2.0 * t) / vw0 - 2.0;
    let v1 = (3.0 + t) / vw1;
    let v2 = t / vw2 + 2.0;

    var sum = 0.0;

    sum += uw0 * vw0 * sample_shadow_map_hardware(base_uv + (vec2(u0, v0) * inv_shadow_map_size), depth, cascade_index);
    sum += uw1 * vw0 * sample_shadow_map_hardware(base_uv + (vec2(u1, v0) * inv_shadow_map_size), depth, cascade_index);
    sum += uw2 * vw0 * sample_shadow_map_hardware(base_uv + (vec2(u2, v0) * inv_shadow_map_size), depth, cascade_index);

    sum += uw0 * vw1 * sample_shadow_map_hardware(base_uv + (vec2(u0, v1) * inv_shadow_map_size), depth, cascade_index);
    sum += uw1 * vw1 * sample_shadow_map_hardware(base_uv + (vec2(u1, v1) * inv_shadow_map_size), depth, cascade_index);
    sum += uw2 * vw1 * sample_shadow_map_hardware(base_uv + (vec2(u2, v1) * inv_shadow_map_size), depth, cascade_index);

    sum += uw0 * vw2 * sample_shadow_map_hardware(base_uv + (vec2(u0, v2) * inv_shadow_map_size), depth, cascade_index);
    sum += uw1 * vw2 * sample_shadow_map_hardware(base_uv + (vec2(u1, v2) * inv_shadow_map_size), depth, cascade_index);
    sum += uw2 * vw2 * sample_shadow_map_hardware(base_uv + (vec2(u2, v2) * inv_shadow_map_size), depth, cascade_index);

    return sum * (1.0 / 144.0);
}

// Sample single cascade with biasing
fn sample_directional_cascade(
    cascade_index: u32,
    frag_position: vec4<f32>,
    surface_normal: vec3<f32>,
) -> f32 {
    let cascade = &csm.cascades[cascade_index];
    
    // The normal bias is scaled to the texel size.
    let normal_offset = csm.shadow_normal_bias * (*cascade).texel_size * surface_normal.xyz;
    let depth_offset = csm.shadow_depth_bias * csm.direction_to_light.xyz;
    let offset_position = vec4<f32>(frag_position.xyz + normal_offset + depth_offset, frag_position.w);

    let light_local = world_to_directional_light_local(cascade_index, offset_position);
    if (light_local.w == 0.0) {
        return 1.0;
    }

    let array_index = i32(cascade_index);
    
    // Use 13-tap PCF for quality
    return sample_shadow_map_pcf(light_local.xy, light_local.z, array_index);
}

// Main directional shadow sampling with cascade selection and blending
fn fetch_directional_shadow(
    frag_position: vec4<f32>,
    surface_normal: vec3<f32>,
    view_z: f32
) -> f32 {
    let cascade_index = get_cascade_index(view_z);

    if (cascade_index >= csm.num_cascades) {
        return 1.0;
    }

    var shadow = sample_directional_cascade(cascade_index, frag_position, surface_normal);

    // Blend with the next cascade, if there is one.
    let next_cascade_index = cascade_index + 1u;
    if (next_cascade_index < csm.num_cascades) {
        let this_far_bound = csm.cascades[cascade_index].far_bound;
        let next_near_bound = (1.0 - csm.cascades_overlap_proportion) * this_far_bound;
        if (-view_z >= next_near_bound) {
            let next_shadow = sample_directional_cascade(next_cascade_index, frag_position, surface_normal);
            shadow = mix(shadow, next_shadow, (-view_z - next_near_bound) / (this_far_bound - next_near_bound));
        }
    }
    return shadow;
}
