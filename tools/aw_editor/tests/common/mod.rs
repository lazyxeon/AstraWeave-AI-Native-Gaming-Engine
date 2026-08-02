//! Shared L-series relief arithmetic — the single source of truth for "can a
//! feature of this size cast a shadow in this cascade?".
//!
//! Included by `mod common;` from both `l3c_relief_census.rs` (CPU census, no
//! GPU) and `l3a_proof.rs` (GPU harness, which prices the same quantities from
//! the renderer's EMITTED fits). It exists so those two cannot disagree: the
//! L.3.C outcome published a `h_min` derived from a modelled ortho pad while
//! the renderer used a different one, and the only structural defence against
//! a repeat is one implementation fed by measured inputs.
//!
//! Not a test target — Cargo auto-discovers `tests/*.rs`, not `tests/*/mod.rs`.

#![allow(dead_code)]

use aw_editor_lib::terrain_integration::TerrainState;

/// The recording's world: seed 12345, Desert, radius 8 = 289 chunks.
pub const SEED: u64 = 12345;
pub const PRIMARY_BIOME: &str = "grassland";
pub const RADIUS: i32 = 8;
/// The desert anchor the T/L-series stations use (x, z).
pub const FOCAL: [f32; 2] = [43.1, -1961.8];

/// Sampling grid: 4 m spacing over a 1.6 km box centred on the anchor —
/// dense enough to resolve dune-scale relief, cheap enough to run on CPU.
pub const GRID_STEP: f32 = 4.0;
pub const BOX_HALF: f32 = 800.0;

/// Dune-scale prominence radii (metres).
pub const PROMINENCE_RADII: [f32; 3] = [10.0, 25.0, 50.0];

/// The editor's delivered shadow filter config, as the fits are priced against:
/// receiver bias in NDC, caster `DepthBiasState` slope scale, default sun.
pub const RECEIVER_BIAS_NDC: f64 = 0.0005;
pub const CASTER_SLOPE_SCALE: f64 = 2.0;
pub const DEFAULT_SUN_ELEV_DEG: f64 = 43.14;

/// Minimum castable relief for a cascade: `h_min = b * sin(elevation)`, where
/// the total depth slack `b = receiver_bias * depth_range + slope_scale * texel
/// * cot(elevation)`. A feature whose local prominence is below `h_min` cannot
/// cast a shadow in that cascade at all — the bias swallows it.
///
/// `receiver_bias_ndc` is the bias the shader actually applies to this cascade.
/// Since the L.3.C resolution that is the shipped NDC bias times a per-cascade
/// cap (`min(1, c1_depth / this_depth)`), delivered in `MainLightUbo.bias_scales`
/// — pass the capped value, not the raw one.
pub fn min_castable_relief(
    texel_m: f64,
    depth_range_m: f64,
    receiver_bias_ndc: f64,
    sun_elev_deg: f64,
) -> f64 {
    let e = sun_elev_deg.to_radians();
    let receiver = receiver_bias_ndc * depth_range_m;
    let caster = CASTER_SLOPE_SCALE * texel_m * (1.0 / e.tan());
    (receiver + caster) * e.sin()
}

/// Sample terrain height on the census grid. `f32::NAN` where the world has no
/// height (outside generated chunks).
pub fn main_grid(state: &TerrainState) -> Vec<Vec<f32>> {
    let n = ((BOX_HALF * 2.0) / GRID_STEP) as usize;
    let mut grid = vec![vec![f32::NAN; n]; n];
    for (iz, row) in grid.iter_mut().enumerate() {
        let z = FOCAL[1] - BOX_HALF + iz as f32 * GRID_STEP;
        for (ix, cell) in row.iter_mut().enumerate() {
            let x = FOCAL[0] - BOX_HALF + ix as f32 * GRID_STEP;
            if let Some(h) = state.get_height_at(x, z) {
                *cell = h;
            }
        }
    }
    grid
}

/// Local prominence: height above the minimum within a disc of `radius` — the
/// drop a feature can cast down, which is the quantity `min_castable_relief`
/// has to be compared against.
pub fn prominence(grid: &[Vec<f32>], radius: f32) -> Vec<f64> {
    let k = (radius / GRID_STEP).round() as isize;
    let n = grid.len() as isize;
    let mut out = Vec::new();
    for iz in 0..n {
        for ix in 0..n {
            let h = grid[iz as usize][ix as usize];
            if h.is_nan() {
                continue;
            }
            let mut lo = f32::INFINITY;
            for dz in -k..=k {
                for dx in -k..=k {
                    if dx * dx + dz * dz > k * k {
                        continue;
                    }
                    let (zz, xx) = (iz + dz, ix + dx);
                    if zz < 0 || xx < 0 || zz >= n || xx >= n {
                        continue;
                    }
                    let v = grid[zz as usize][xx as usize];
                    if !v.is_nan() && v < lo {
                        lo = v;
                    }
                }
            }
            if lo.is_finite() {
                out.push((h - lo) as f64);
            }
        }
    }
    out
}

pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[i]
}

/// Fraction of `sorted` prominences (percent) strictly below `threshold` — the
/// share of dune features a cascade cannot render a shadow for.
pub fn frac_below(sorted: &[f64], threshold: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted.iter().filter(|v| **v < threshold).count() as f64 / sorted.len() as f64 * 100.0
}
