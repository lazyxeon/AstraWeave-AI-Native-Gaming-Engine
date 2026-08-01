//! L.3.C — relief census: is the dune world's relief representable by each cascade?
//!
//! The director's L.3.B ruling promoted the 500 m c1|c2 rendition seam to prime
//! suspect for the gate failure, with a falsifiable prediction:
//!
//! > min-castable-relief 1.42 -> 9.1 m across the seam, on a DUNE world whose
//! > relief is largely 2-8 m — dune shadows are unrepresentable beyond the
//! > seam, and the seam sweeps terrain as I fly. The hypothesis predicts dune
//! > shadows exist only c1-side.
//!
//! This test measures the left-hand side of that prediction on the RECORDING'S
//! world (seed 12345, **radius 8** = 289 chunks — the video's chunk count; the
//! L.3/L.3.A harnesses used radius 10 = 441 and were flying a different world).
//!
//! Method. Sample terrain height on a regular grid, then for each sample
//! compute LOCAL PROMINENCE at dune scale: the height of the sample above the
//! lowest ground within a disc of radius R. That is the drop a feature can cast
//! down, which is what has to exceed a cascade's minimum-castable-relief for a
//! shadow to exist at all. Reported as a distribution against the per-cascade
//! thresholds, so "largely 2-8 m" is confirmed or corrected with numbers rather
//! than assumed.
//!
//! No GPU: pure terrain generation + arithmetic.
//!
//! ```text
//! cargo test -p aw_editor --profile release-fast --test l3c_relief_census \
//!     -- --ignored --nocapture l3c_relief_census
//! ```

use anyhow::{Context, Result};
use astraweave_terrain::world_archetypes::WorldArchetypeId;
use aw_editor_lib::terrain_integration::TerrainState;

const SEED: u64 = 12345;
const PRIMARY_BIOME: &str = "grassland";
/// The recording's world: 289 chunks = (2*8+1)^2.
const RADIUS: i32 = 8;
/// The desert anchor the T/L-series stations use.
const FOCAL: [f32; 2] = [43.1, -1961.8];

/// Sampling grid: 4 m spacing over a 1.6 km box centred on the anchor —
/// dense enough to resolve dune-scale relief, cheap enough to run on CPU.
const GRID_STEP: f32 = 4.0;
const BOX_HALF: f32 = 800.0;

/// Dune-scale prominence radii (metres).
const PROMINENCE_RADII: [f32; 3] = [10.0, 25.0, 50.0];

/// Minimum castable relief per cascade: h_min = b * sin(elevation), where the
/// total depth slack b = receiver_bias_ndc * ortho_depth_range
///                     + caster_slope_scale * texel * cot(elevation).
///
/// Values are derived from the live fits (L3B_OUTCOME §4) at the editor's
/// delivered filter config: receiver bias 0.0005 NDC, caster DepthBiasState
/// slope_scale 2.0, and the editor's default sun elevation 43.14 deg.
fn min_castable_relief(texel_m: f64, depth_range_m: f64, sun_elev_deg: f64) -> f64 {
    let e = sun_elev_deg.to_radians();
    let receiver = 0.0005 * depth_range_m;
    let caster = 2.0 * texel_m * (1.0 / e.tan());
    (receiver + caster) * e.sin()
}

struct Cascade {
    name: &'static str,
    range: (f64, f64),
    radius_m: f64,
    texel_m: f64,
    depth_range_m: f64,
}

/// The THREE-cascade layout as shipped at L.3.A (the state the director's
/// recording was made against). Radii/texels/depth-ranges are the derived
/// values confirmed by measurement in L3B_OUTCOME §4.
fn shipped_three() -> Vec<Cascade> {
    vec![
        Cascade {
            name: "c0",
            range: (0.5, 86.1),
            radius_m: 102.5,
            texel_m: 0.1020,
            depth_range_m: 359.4,
        },
        Cascade {
            name: "c1",
            range: (86.1, 500.0),
            radius_m: 569.9,
            texel_m: 0.5585,
            depth_range_m: 1761.6,
        },
        Cascade {
            name: "c2 (survey)",
            range: (500.0, 3000.0),
            radius_m: 3424.3,
            texel_m: 3.7366,
            depth_range_m: 10724.8,
        },
    ]
}

fn main_grid(state: &TerrainState) -> Vec<Vec<f32>> {
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

/// Local prominence: height above the minimum within a disc of `radius`.
fn prominence(grid: &[Vec<f32>], radius: f32) -> Vec<f64> {
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

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[i]
}

#[test]
#[ignore = "terrain generation (CPU, ~1 min); run explicitly (see module docs)"]
fn l3c_relief_census() -> Result<()> {
    let mut state = TerrainState::new();
    state.configure(SEED, PRIMARY_BIOME);
    state.set_noise_params(6, 2.0, 0.5, 50.0);
    state.set_world_archetype(WorldArchetypeId::Desert.default_archetype());
    let n = state
        .generate_terrain(RADIUS)
        .context("generate_terrain failed")?;
    println!("[l3c] Desert seed {SEED} radius {RADIUS}: {n} chunks (the recording's world)");
    anyhow::ensure!(
        n == 289,
        "expected 289 chunks for radius 8 (the video's count), got {n}"
    );

    let grid = main_grid(&state);
    let samples: usize = grid.iter().flatten().filter(|h| !h.is_nan()).count();
    println!(
        "[l3c] height grid: {}x{} at {GRID_STEP} m over a {} m box, {samples} valid samples",
        grid.len(),
        grid.len(),
        BOX_HALF * 2.0
    );

    // Thresholds for the SHIPPED three-cascade layout, at the editor's default
    // sun (43.14 deg) and at the harness rake sun (20 deg).
    for sun in [43.14_f64, 20.0] {
        println!("\n[l3c] minimum castable relief per cascade, sun elevation {sun:.1} deg:");
        for c in shipped_three() {
            let h = min_castable_relief(c.texel_m, c.depth_range_m, sun);
            println!(
                "[l3c]   {:<12} range {:>6.0}..{:<6.0} m  texel {:>6.3} m  depth {:>8.0} m  ->  h_min {:>6.2} m",
                c.name, c.range.0, c.range.1, c.texel_m, c.depth_range_m, h
            );
        }
    }

    println!("\n[l3c] dune relief distribution (local prominence):");
    let sun = 43.14_f64;
    let three = shipped_three();
    let h_c1 = min_castable_relief(three[1].texel_m, three[1].depth_range_m, sun);
    let h_c2 = min_castable_relief(three[2].texel_m, three[2].depth_range_m, sun);
    for r in PROMINENCE_RADII {
        let mut p = prominence(&grid, r);
        p.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let below_c1 = p.iter().filter(|v| **v < h_c1).count() as f64 / p.len() as f64 * 100.0;
        let below_c2 = p.iter().filter(|v| **v < h_c2).count() as f64 / p.len() as f64 * 100.0;
        println!(
            "[l3c]   R={r:>4.0} m: p10 {:>5.2}  p25 {:>5.2}  p50 {:>5.2}  p75 {:>5.2}  p90 {:>5.2}  max {:>6.2} m",
            percentile(&p, 0.10),
            percentile(&p, 0.25),
            percentile(&p, 0.50),
            percentile(&p, 0.75),
            percentile(&p, 0.90),
            p.last().copied().unwrap_or(0.0)
        );
        println!(
            "[l3c]            unrepresentable c1-side (< {h_c1:.2} m): {below_c1:>5.1}%   \
             unrepresentable c2-side (< {h_c2:.2} m): {below_c2:>5.1}%   \
             LOST ACROSS THE SEAM: {:>5.1} pp",
            below_c2 - below_c1
        );
    }

    println!(
        "\n[l3c] VERDICT INPUT: the director's hypothesis predicts a large 'lost across the seam' \
         figure — features that cast on the c1 side and cannot cast on the c2 side."
    );
    Ok(())
}

/// The renderer's own cascade fit, reproduced on the CPU so candidate layouts
/// can be priced before any of them is built.
///
/// `frustum_center` is the mean of the 8 slice corners — for a symmetric
/// frustum that is `(0, 0, (near+far)/2)` in view space — and `sphere_radius`
/// is the max corner distance from it, which the FAR corners always win:
///     r = sqrt(far^2 * K + ((far-near)/2)^2)
/// K = tan^2(fovy/2) * (1 + aspect^2) is calibrated to the LIVE fits rather
/// than assumed: K = 1.129 reproduces the renderer's measured radii for all
/// three shipped cascades (c0 101.0 vs 102.5, c1 570.2 vs 569.9, c2 3424.3 vs
/// the telemetry's 3424) — see L3B_OUTCOME §4 for the measurements.
const FIT_K: f64 = 1.129;

fn cascade_radius(near: f64, far: f64) -> f64 {
    (far * far * FIT_K + ((far - near) * 0.5).powi(2)).sqrt()
}

struct LayoutCascade {
    name: String,
    near: f64,
    far: f64,
    texel_m: f64,
    depth_range_m: f64,
    h_min: f64,
}

/// Price a candidate layout. `pad` is the per-cascade ortho margin (the cached
/// survey cascade carries a 400 m drift pad; every-frame cascades carry 2 m).
/// `world_bias_m`: `None` keeps the shipped single 0.0005 NDC receiver bias
/// (whose WORLD magnitude therefore scales with each cascade's depth range —
/// 30x across the set: 0.18 m at c0, 5.36 m at the survey cascade);
/// `Some(b)` caps every cascade at the same world bias.
///
/// The cap is a MINIMUM of the two, never an increase: cascades whose shipped
/// bias is already tighter than `b` (c0 at 0.18 m) keep theirs untouched. That
/// is what lets this change preserve the L.3 station byte-identity guarantee —
/// c0 and c1 render exactly as before and only the far cascades' slack falls.
fn price_layout(
    splits: &[f64],
    pads: &[f64],
    resolution: f64,
    sun_elev_deg: f64,
    world_bias_m: Option<f64>,
) -> Vec<LayoutCascade> {
    let e: f64 = sun_elev_deg.to_radians();
    let mut out = Vec::new();
    for i in 0..splits.len() - 1 {
        let (near, far) = (splits[i], splits[i + 1]);
        let r = cascade_radius(near, far);
        let pad = pads[i];
        let light_dist = r * 2.0 + 50.0;
        let depth_range = light_dist + r + pad - 0.1;
        let texel = 2.0 * (r + pad) / resolution;
        let shipped = 0.0005 * depth_range;
        let receiver = match world_bias_m {
            Some(b) => shipped.min(b),
            None => shipped,
        };
        let caster = 2.0 * texel * (1.0 / e.tan());
        out.push(LayoutCascade {
            name: format!("c{i}"),
            near,
            far,
            texel_m: texel,
            depth_range_m: depth_range,
            h_min: (receiver + caster) * e.sin(),
        });
    }
    out
}

/// Prices the shipped 3-cascade layout, the director-authorised 4-cascade
/// layout, and the 4-cascade layout with the receiver bias normalised to a
/// constant WORLD magnitude — reporting, for each seam, how much of the dune
/// world stops being able to cast.
#[test]
#[ignore = "terrain generation (CPU); run explicitly (see module docs)"]
fn l3c_layout_comparison() -> Result<()> {
    let mut state = TerrainState::new();
    state.configure(SEED, PRIMARY_BIOME);
    state.set_noise_params(6, 2.0, 0.5, 50.0);
    state.set_world_archetype(WorldArchetypeId::Desert.default_archetype());
    let n = state.generate_terrain(RADIUS).context("generate_terrain")?;
    anyhow::ensure!(
        n == 289,
        "expected the recording's 289-chunk world, got {n}"
    );
    let grid = main_grid(&state);
    let mut p25 = prominence(&grid, 25.0);
    p25.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let frac_below =
        |t: f64| p25.iter().filter(|v| **v < t).count() as f64 / p25.len() as f64 * 100.0;

    let sun = 43.14;
    // c1's shipped world-space receiver bias: 0.0005 NDC x its 1762 m depth
    // range = 0.88 m. Normalising the far cascades to THAT keeps c0/c1
    // untouched (station byte-identity) while removing the silent 30x scaling.
    let c1_world_bias = 0.0005 * 1761.6;

    let layouts: Vec<(&str, Vec<f64>, Vec<f64>, Option<f64>)> = vec![
        (
            "shipped 3-cascade (L.3.A)",
            vec![0.5, 86.1, 500.0, 3000.0],
            vec![2.0, 2.0, 400.0],
            None,
        ),
        (
            "authorised 4-cascade",
            vec![0.5, 86.1, 500.0, 1400.0, 3000.0],
            vec![2.0, 2.0, 2.0, 400.0],
            None,
        ),
        (
            "4-cascade + world-normalised receiver bias",
            vec![0.5, 86.1, 500.0, 1400.0, 3000.0],
            vec![2.0, 2.0, 2.0, 400.0],
            Some(c1_world_bias),
        ),
    ];

    for (name, splits, pads, bias) in layouts {
        println!("\n[l3c] === {name} === (sun {sun:.1} deg, 2048^2, dune scale R=25 m)");
        let cs = price_layout(&splits, &pads, 2048.0, sun, bias);
        let mut prev: Option<&LayoutCascade> = None;
        for c in &cs {
            println!(
                "[l3c]   {:<4} {:>6.0}..{:<6.0} m  texel {:>6.3} m  depth {:>7.0} m  h_min {:>5.2} m  \
                 unrepresentable {:>5.1}%",
                c.name, c.near, c.far, c.texel_m, c.depth_range_m, c.h_min, frac_below(c.h_min)
            );
            if let Some(p) = prev {
                println!(
                    "[l3c]        ^ seam at {:>6.0} m: castable relief {:>5.2} -> {:>5.2} m  \
                     ({:>4.1}x)  LOST ACROSS SEAM {:>5.1} pp",
                    c.near,
                    p.h_min,
                    c.h_min,
                    c.h_min / p.h_min,
                    frac_below(c.h_min) - frac_below(p.h_min)
                );
            }
            prev = Some(c);
        }
    }
    println!(
        "\n[l3c] The 500 m seam is the one the survey camera crosses constantly; the 1400 m seam \
         sits where dune shadows subtend few pixels."
    );
    Ok(())
}
