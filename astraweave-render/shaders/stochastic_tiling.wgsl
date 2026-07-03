// stochastic_tiling.wgsl — Hex-tile stochastic texture sampling
//
// Breaks visible texture repetition by randomly rotating/offsetting samples
// within a hexagonal grid. Based on Heitz & Neyret 2018 "High-Performance
// By-Example Noise using an Histogram-Preserving Blending Operator."
//
// Usage: Replace `textureSample(tex, samp, uv)` with
//        `sample_stochastic(tex, samp, uv, seed)` wherever tiling artifacts
//        are visible (typically large terrain surfaces).

// ============================================================================
// Hex grid helpers
// ============================================================================

// Single hash from 2D coords — produces pseudo-random value in [0,1).
fn hash2(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

// Hash returning 2D pseudo-random offset.
fn hash22(p: vec2<f32>) -> vec2<f32> {
    let h = vec2<f32>(
        dot(p, vec2<f32>(127.1, 311.7)),
        dot(p, vec2<f32>(269.5, 183.3)),
    );
    return fract(sin(h) * 43758.5453);
}

// Hex grid coordinate decomposition. Returns the three nearest hex centers
// and barycentric-like blend weights.
struct HexSample {
    uv0: vec2<f32>,
    uv1: vec2<f32>,
    uv2: vec2<f32>,
    w0: f32,
    w1: f32,
    w2: f32,
}

fn hex_coords(uv: vec2<f32>) -> HexSample {
    // Transform to hexagonal grid (skewed coordinates)
    let sqrt3 = 1.7320508;
    let inv_sqrt3 = 0.57735026;

    let skewed = vec2<f32>(uv.x + uv.y * inv_sqrt3, uv.y * 2.0 * inv_sqrt3);
    let base = floor(skewed);
    let f = fract(skewed);

    // Determine which simplex triangle we are in
    var v0: vec2<f32>;
    var v1: vec2<f32>;
    var v2: vec2<f32>;

    if (f.x + f.y < 1.0) {
        v0 = base;
        v1 = base + vec2<f32>(1.0, 0.0);
        v2 = base + vec2<f32>(0.0, 1.0);
    } else {
        v0 = base + vec2<f32>(1.0, 1.0);
        v1 = base + vec2<f32>(1.0, 0.0);
        v2 = base + vec2<f32>(0.0, 1.0);
    }

    // Barycentric-like weights from distance to vertices
    let d0 = length(skewed - v0);
    let d1 = length(skewed - v1);
    let d2 = length(skewed - v2);

    // Inverse distance weighting
    let inv0 = 1.0 / max(d0, 0.0001);
    let inv1 = 1.0 / max(d1, 0.0001);
    let inv2 = 1.0 / max(d2, 0.0001);
    let inv_total = inv0 + inv1 + inv2;

    // Random UV offsets per hex cell
    let off0 = hash22(v0);
    let off1 = hash22(v1);
    let off2 = hash22(v2);

    // Random rotation per cell (applied as UV rotation)
    let a0 = hash2(v0 + vec2<f32>(73.0, 19.0)) * 6.2831853;
    let a1 = hash2(v1 + vec2<f32>(73.0, 19.0)) * 6.2831853;
    let a2 = hash2(v2 + vec2<f32>(73.0, 19.0)) * 6.2831853;

    var result: HexSample;
    result.uv0 = rotate_uv(uv, a0) + off0;
    result.uv1 = rotate_uv(uv, a1) + off1;
    result.uv2 = rotate_uv(uv, a2) + off2;
    result.w0 = inv0 / inv_total;
    result.w1 = inv1 / inv_total;
    result.w2 = inv2 / inv_total;

    return result;
}

// Rotate UV coordinates around (0.5, 0.5) center.
fn rotate_uv(uv: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    let centered = uv - vec2<f32>(0.5);
    return vec2<f32>(
        centered.x * c - centered.y * s,
        centered.x * s + centered.y * c,
    ) + vec2<f32>(0.5);
}

// ============================================================================
// Main sampling function
// ============================================================================

// Stochastic hex-tile texture sample. Drop-in replacement for textureSample
// that eliminates visible tiling patterns.
//
// `scale` controls hex cell density — higher values = smaller cells = less
// visible pattern but more texture lookups. Typical range: 0.5 - 2.0.
fn sample_stochastic(
    tex: texture_2d<f32>,
    samp: sampler,
    uv: vec2<f32>,
    scale: f32,
) -> vec4<f32> {
    let hex = hex_coords(uv * scale);

    let s0 = textureSample(tex, samp, hex.uv0);
    let s1 = textureSample(tex, samp, hex.uv1);
    let s2 = textureSample(tex, samp, hex.uv2);

    return s0 * hex.w0 + s1 * hex.w1 + s2 * hex.w2;
}

// Stochastic sample for texture arrays.
fn sample_stochastic_array(
    tex: texture_2d_array<f32>,
    samp: sampler,
    uv: vec2<f32>,
    layer: u32,
    scale: f32,
) -> vec4<f32> {
    let hex = hex_coords(uv * scale);

    let s0 = textureSample(tex, samp, hex.uv0, layer);
    let s1 = textureSample(tex, samp, hex.uv1, layer);
    let s2 = textureSample(tex, samp, hex.uv2, layer);

    return s0 * hex.w0 + s1 * hex.w1 + s2 * hex.w2;
}

// ============================================================================
// Grad-friendly cell variant (E3-terrain texturing, 2026-07-03)
// ============================================================================
// Returns per-hex-cell random rotation angles + UV translations + blend
// weights WITHOUT baking them into a sample UV. Callers apply the transform
// to their own texture-space UV and sample with `textureSampleGrad` — which
// is mandatory inside non-uniform loops (pbr_terrain_forward.wgsl's per-layer
// loop; plain `textureSample` there is undefined behavior).
//
// ROTATION IS LOAD-BEARING for periodic textures: a pure translation of a
// tiled texture is the same tiling phase-shifted — the repetition grid
// survives (confirmed visually, 2026-07-03 "still looks tiled" round).
// Rotation re-orients the lattice per cell, which is what actually breaks
// the pattern. Callers sampling tangent-space NORMAL maps must counter-rotate
// the sampled normal's XY by the same cell angle (rotate by -angle) to keep
// lighting consistent with the mesh TBN.
struct HexCells {
    off0: vec2<f32>,
    off1: vec2<f32>,
    off2: vec2<f32>,
    // cos/sin pairs of each cell's rotation angle.
    rot0: vec2<f32>,
    rot1: vec2<f32>,
    rot2: vec2<f32>,
    w0: f32,
    w1: f32,
    w2: f32,
}

// Rotate a direction vector (no center offset — for derivatives and normals).
fn rotate_dir(v: vec2<f32>, cs: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(v.x * cs.x - v.y * cs.y, v.x * cs.y + v.y * cs.x);
}

fn hex_cells(cell_uv: vec2<f32>) -> HexCells {
    let inv_sqrt3 = 0.57735026;
    let skewed = vec2<f32>(cell_uv.x + cell_uv.y * inv_sqrt3, cell_uv.y * 2.0 * inv_sqrt3);
    let base = floor(skewed);
    let f = fract(skewed);

    var v0: vec2<f32>;
    var v1: vec2<f32>;
    var v2: vec2<f32>;
    if (f.x + f.y < 1.0) {
        v0 = base;
        v1 = base + vec2<f32>(1.0, 0.0);
        v2 = base + vec2<f32>(0.0, 1.0);
    } else {
        v0 = base + vec2<f32>(1.0, 1.0);
        v1 = base + vec2<f32>(1.0, 0.0);
        v2 = base + vec2<f32>(0.0, 1.0);
    }

    let d0 = length(skewed - v0);
    let d1 = length(skewed - v1);
    let d2 = length(skewed - v2);
    let inv0 = 1.0 / max(d0, 0.0001);
    let inv1 = 1.0 / max(d1, 0.0001);
    let inv2 = 1.0 / max(d2, 0.0001);
    let inv_total = inv0 + inv1 + inv2;

    let a0 = hash2(v0 + vec2<f32>(73.0, 19.0)) * 6.2831853;
    let a1 = hash2(v1 + vec2<f32>(73.0, 19.0)) * 6.2831853;
    let a2 = hash2(v2 + vec2<f32>(73.0, 19.0)) * 6.2831853;

    var result: HexCells;
    result.off0 = hash22(v0);
    result.off1 = hash22(v1);
    result.off2 = hash22(v2);
    result.rot0 = vec2<f32>(cos(a0), sin(a0));
    result.rot1 = vec2<f32>(cos(a1), sin(a1));
    result.rot2 = vec2<f32>(cos(a2), sin(a2));
    result.w0 = inv0 / inv_total;
    result.w1 = inv1 / inv_total;
    result.w2 = inv2 / inv_total;
    return result;
}
