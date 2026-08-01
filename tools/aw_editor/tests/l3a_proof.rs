//! L.3.A — the camera-anchored shadow boundary: measurement + motion-leg proof.
//!
//! The L.3 render gate was rejected from a moving survey camera (Y≈219):
//! "shadows seem to be attached to the camera boundary … always keep a sharp
//! boundary line." The live path's coverage is a camera-centered bubble —
//! `shadow_far = 500` view-distance cap with a 100 m fade band (400→500,
//! shadow_common.wgsl) — invisible at the static pinned stations (which sat
//! fully inside or outside it; y414's zero-pixel result was its fingerprint)
//! and unmistakable in motion. **The lesson: camera-fitted systems get
//! camera-MOTION verification legs.** This harness is that leg.
//!
//! One `#[ignore]`d GPU test, three parts, one world/viewport session
//! (desert radius 10, raking sun — elevation 20.0°, azimuth 51.3°, the L.3
//! rake convention):
//!
//! 1. **Radial profile at two survey altitudes** (~219 m — the rejection
//!    recording's route — and ~110 m): per-screen-row shadow effect
//!    (|on − off| mean luma, center columns), each row converted to
//!    view/ground distance by flat-plane ray intersection at the local
//!    terrain height. The farthest row with effect IS the boundary; a
//!    boundary at fixed VIEW distance across both altitudes is the
//!    camera-anchored conviction (ground-anchored would track terrain).
//! 2. **Motion leg**: 7 camera positions along a straight lateral sweep at
//!    Y≈219 (600 m — longer than the recording's ~470), fixed orientation;
//!    shadows on/off captured per point.
//! 3. **World-anchored features**: 2 strong-shadow world points chosen from
//!    the FINAL path point's difference map (unprojected to world), then
//!    tracked across every path frame by reprojection: their shadow state
//!    must be a function of the WORLD, not the camera position.
//!
//! `L3A_EXPECT=fixed` arms the post-fix assertions (coverage beyond 2 km,
//! features shadowed at every in-frame path point). Without it the test only
//! measures — the HEAD leg documents the defect.
//!
//! ```text
//! L3A_LABEL=<label> cargo test -p aw_editor --profile release-fast \
//!     --test l3a_proof -- --ignored --nocapture l3a_boundary_evidence
//! ```
//!
//! Output: `$L3A_OUT/<label>/` (default `d:/tmp/l3a_staging/<label>/`).

use anyhow::{Context, Result};
use astraweave_core::World;
use astraweave_terrain::world_archetypes::WorldArchetypeId;
use aw_editor_lib::terrain_integration::TerrainState;
use aw_editor_lib::viewport::types::TerrainLightingParams;
use aw_editor_lib::viewport::{OrbitCamera, ViewportRenderer};
use glam::Vec3;
use std::path::PathBuf;
use std::sync::Arc;

const SEED: u64 = 12345;
const PRIMARY_BIOME: &str = "grassland";
/// L.3 rake sun: elevation 20.0°, azimuth 51.3° (same as l3_proof).
const RAKE_SUN_DIR: [f32; 3] = [0.7335, 0.3420, 0.5870];
/// The desert ground anchor (t2df/l3 lineage; ground height ≈ 36.3).
const DESERT_FOCAL: [f32; 3] = [43.1, 36.3, -1961.8];
const GROUND_H: f32 = 36.3;
/// OrbitCamera's vertical FOV (camera.rs).
const FOVY: f32 = 1.047_197_6;
const W: u32 = 1024;
const H: u32 = 768;
/// Survey pitch for all legs (down-tilt; horizon in the upper frame).
const PITCH_DEG: f32 = 25.0;
const YAW_DEG: f32 = 45.0;
/// The rejection recording's altitude and a second altitude for the
/// view-vs-ground discrimination.
const ALT_HIGH: f32 = 219.0;
const ALT_LOW: f32 = 110.0;

fn out_root() -> PathBuf {
    PathBuf::from(std::env::var("L3A_OUT").unwrap_or_else(|_| "d:/tmp/l3a_staging".to_string()))
}
fn label() -> String {
    std::env::var("L3A_LABEL").unwrap_or_else(|_| "unlabelled".to_string())
}
fn expect_fixed() -> bool {
    std::env::var("L3A_EXPECT").as_deref() == Ok("fixed")
}

fn find_assets_dir() -> Option<PathBuf> {
    for candidate in ["assets", "../../assets"] {
        let p = PathBuf::from(candidate);
        if p.join("materials")
            .join("biomes")
            .join("materials.toml")
            .is_file()
        {
            return Some(p);
        }
    }
    None
}

async fn acquire_device() -> Result<(Arc<wgpu::Device>, Arc<wgpu::Queue>, wgpu::AdapterInfo)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .context("no suitable wgpu adapter for the L.3.A proof")?;
    let info = adapter.get_info();
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("l3a-proof device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits {
                max_bind_groups: 8,
                ..wgpu::Limits::default()
            },
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        })
        .await
        .context("request_device failed")?;
    Ok((Arc::new(device), Arc::new(queue), info))
}

/// Survey camera: eye at `altitude` above ground, orbiting `focal` at the
/// fixed survey pitch/yaw. OrbitCamera eye = focal + dist·(cos y·cos p,
/// sin p, sin y·cos p) (T2D §10.2 recovery math, harness-shared since).
fn survey_camera(focal: Vec3, altitude: f32) -> (OrbitCamera, Vec3) {
    survey_camera_yaw(focal, altitude, YAW_DEG)
}

/// L.3.B: survey camera with an explicit yaw, for the PAN (sweep) route.
/// Panning is the regime with the strongest coupling to the cached survey
/// window: the window's centre sits ~1750 m ahead of the eye, so one degree of
/// yaw moves it ~30 m — a few degrees per frame trips the 300 m drift trigger
/// every few frames AND rotates the frozen coverage relative to the view,
/// which a straight-line strafe barely exercises.
fn survey_camera_yaw(focal: Vec3, altitude: f32, yaw_deg: f32) -> (OrbitCamera, Vec3) {
    let pitch = PITCH_DEG.to_radians();
    let yaw = yaw_deg.to_radians();
    let distance = (altitude - focal.y) / pitch.sin();
    let mut cam = OrbitCamera::new(focal, distance, yaw, pitch);
    cam.set_aspect(W as f32, H as f32);
    let eye = focal
        + distance
            * Vec3::new(
                yaw.cos() * pitch.cos(),
                pitch.sin(),
                yaw.sin() * pitch.cos(),
            );
    (cam, eye)
}

/// Camera basis for the survey framing (right-handed, world up).
fn camera_basis(eye: Vec3, focal: Vec3) -> (Vec3, Vec3, Vec3) {
    let f = (focal - eye).normalize();
    let r = f.cross(Vec3::Y).normalize();
    let u = r.cross(f).normalize();
    (f, r, u)
}

/// Ray through pixel center → flat ground plane at `h`. Returns
/// (view_distance, ground_distance, world_point) or None if the ray
/// doesn't hit below the horizon.
fn pixel_to_ground(
    eye: Vec3,
    basis: (Vec3, Vec3, Vec3),
    px: f32,
    py: f32,
    h: f32,
) -> Option<(f32, f32, Vec3)> {
    let (f, r, u) = basis;
    let aspect = W as f32 / H as f32;
    let t_half = (FOVY * 0.5).tan();
    let ndc_x = 2.0 * (px + 0.5) / W as f32 - 1.0;
    let ndc_y = 1.0 - 2.0 * (py + 0.5) / H as f32;
    let dir = (f + r * (ndc_x * t_half * aspect) + u * (ndc_y * t_half)).normalize();
    if dir.y >= -1e-4 {
        return None;
    }
    let t = (h - eye.y) / dir.y;
    if t <= 0.0 {
        return None;
    }
    let p = eye + dir * t;
    let gd = ((p.x - eye.x).powi(2) + (p.z - eye.z).powi(2)).sqrt();
    Some((t, gd, p))
}

/// World point → pixel under the same camera model. None if behind or
/// outside the frame.
fn world_to_pixel(eye: Vec3, basis: (Vec3, Vec3, Vec3), p: Vec3) -> Option<(f32, f32)> {
    let (f, r, u) = basis;
    let d = p - eye;
    let z = d.dot(f);
    if z <= 0.1 {
        return None;
    }
    let aspect = W as f32 / H as f32;
    let t_half = (FOVY * 0.5).tan();
    let ndc_x = d.dot(r) / (z * t_half * aspect);
    let ndc_y = d.dot(u) / (z * t_half);
    if ndc_x.abs() > 1.0 || ndc_y.abs() > 1.0 {
        return None;
    }
    Some((
        (ndc_x + 1.0) * 0.5 * W as f32 - 0.5,
        (1.0 - ndc_y) * 0.5 * H as f32 - 0.5,
    ))
}

fn luma_of(img: &image::RgbaImage, x: u32, y: u32) -> f64 {
    let p = img.get_pixel(x, y);
    0.2126 * p.0[0] as f64 + 0.7152 * p.0[1] as f64 + 0.0722 * p.0[2] as f64
}

/// Mean |on−off| luma over a (2k+1)² patch centered at (x, y).
fn patch_diff(on: &image::RgbaImage, off: &image::RgbaImage, x: u32, y: u32, k: u32) -> f64 {
    let (mut sum, mut n) = (0.0f64, 0u32);
    for yy in y.saturating_sub(k)..=(y + k).min(H - 1) {
        for xx in x.saturating_sub(k)..=(x + k).min(W - 1) {
            sum += (luma_of(on, xx, yy) - luma_of(off, xx, yy)).abs();
            n += 1;
        }
    }
    sum / n as f64
}

struct Shot {
    on: image::RgbaImage,
    off: image::RgbaImage,
}

fn grab(
    viewport: &mut ViewportRenderer,
    texture: &wgpu::Texture,
    cam: &OrbitCamera,
    out: &std::path::Path,
    name: &str,
) -> Result<image::RgbaImage> {
    let world = World::new();
    for _ in 0..2 {
        viewport
            .render(texture, cam, &world, None, None, None, false, false, 0)
            .with_context(|| format!("render failed at {name}"))?;
    }
    let png = out.join(format!("{name}.png"));
    viewport.capture_frame_png(texture, &png)?;
    Ok(image::open(&png)?.to_rgba8())
}

fn set_shadows(viewport: &mut ViewportRenderer, on: bool) -> Result<()> {
    viewport
        .engine_adapter_mut()
        .context("engine adapter missing")?
        .renderer_mut()
        .set_shadows_enabled(on);
    Ok(())
}

fn shoot_pair(
    viewport: &mut ViewportRenderer,
    texture: &wgpu::Texture,
    cam: &OrbitCamera,
    out: &std::path::Path,
    name: &str,
) -> Result<Shot> {
    set_shadows(viewport, true)?;
    let on = grab(viewport, texture, cam, out, name)?;
    set_shadows(viewport, false)?;
    let off = grab(viewport, texture, cam, out, &format!("{name}_off"))?;
    set_shadows(viewport, true)?;
    Ok(Shot { on, off })
}

/// L.3.B: render ONE frame and capture it — the continuous leg's primitive.
///
/// Deliberately single-frame and toggle-free. The L.3.A motion leg's
/// two-frame settle plus its shadows-on/off pair is exactly what hid this
/// beat's defect: settling gives the survey-cascade cache a chance to refresh
/// at every sampled point, and toggling shadows invalidates the cache under
/// test (`set_shadows_enabled(false)` clears `c2_valid`). Here each capture is
/// one frame of a continuous flight, cache state carried forward untouched.
fn grab_one(
    viewport: &mut ViewportRenderer,
    texture: &wgpu::Texture,
    cam: &OrbitCamera,
    out: &std::path::Path,
    name: &str,
) -> Result<image::RgbaImage> {
    let world = World::new();
    viewport
        .render(texture, cam, &world, None, None, None, false, false, 0)
        .with_context(|| format!("render failed at {name}"))?;
    let png = out.join(format!("{name}.png"));
    viewport.capture_frame_png(texture, &png)?;
    Ok(image::open(&png)?.to_rgba8())
}

/// Mean luma over a (2k+1)² patch centred at (x, y) — the per-frame observable
/// for a world-anchored feature during continuous flight (no off-reference is
/// available mid-flight, so stability is judged frame-to-frame).
fn patch_luma(img: &image::RgbaImage, x: u32, y: u32, k: u32) -> f64 {
    let (mut sum, mut n) = (0.0f64, 0u32);
    for yy in y.saturating_sub(k)..=(y + k).min(H - 1) {
        for xx in x.saturating_sub(k)..=(x + k).min(W - 1) {
            sum += luma_of(img, xx, yy);
            n += 1;
        }
    }
    sum / n as f64
}

fn survey_telemetry(viewport: &mut ViewportRenderer) -> Result<(u32, &'static str, f32, bool)> {
    Ok(viewport
        .engine_adapter_mut()
        .context("engine adapter missing")?
        .renderer()
        .shadow_survey_telemetry())
}

/// L.3.B band metrics. The whole-frame mean is a weak detector for a far-field
/// band (most pixels are near/mid field, governed by cascades 0/1 which are not
/// cached), so the continuous leg measures the SURVEY CASCADE'S OWN DOMAIN:
/// the screen rows whose ground intersection lies in [`split1`, `shadow_far`]
/// = 500..3000 m view distance.
///
/// Returns `(band_mean_luma, shadow_area_fraction)`. The shadow-area fraction —
/// pixels more than `SHADOW_DROP` luma below the band median — is the direct
/// "shadows appeared / vanished" signal the director's recording describes;
/// its frame-to-frame delta is far more sensitive than any mean.
fn band_metrics(
    img: &image::RgbaImage,
    eye: Vec3,
    basis: (Vec3, Vec3, Vec3),
    near_m: f32,
    far_m: f32,
) -> (f64, f64) {
    const SHADOW_DROP: f64 = 12.0;
    let mut lumas: Vec<f64> = Vec::new();
    for y in 0..H {
        let Some((view_d, _, _)) = pixel_to_ground(eye, basis, (W / 2) as f32, y as f32, GROUND_H)
        else {
            continue;
        };
        if view_d < near_m || view_d > far_m {
            continue;
        }
        for x in (0..W).step_by(2) {
            lumas.push(luma_of(img, x, y));
        }
    }
    if lumas.is_empty() {
        return (0.0, 0.0);
    }
    let mean = lumas.iter().sum::<f64>() / lumas.len() as f64;
    let mut sorted = lumas.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[sorted.len() / 2];
    let shadowed = lumas.iter().filter(|l| **l < median - SHADOW_DROP).count();
    (mean, shadowed as f64 / lumas.len() as f64)
}

/// Per-row shadow effect over the center half of the columns; returns the
/// farthest view-distance with effect > 1.0 luma and prints the profile.
fn radial_profile(shot: &Shot, eye: Vec3, basis: (Vec3, Vec3, Vec3), tag: &str) -> Option<f32> {
    let mut max_view: Option<f32> = None;
    println!("[l3a] {tag}: row profile (row, view_m, ground_m, |on-off| mean):");
    for y in (0..H).step_by(8) {
        let (mut sum, mut n) = (0.0f64, 0u32);
        for x in (W / 4)..(3 * W / 4) {
            sum += (luma_of(&shot.on, x, y) - luma_of(&shot.off, x, y)).abs();
            n += 1;
        }
        let d = sum / n as f64;
        let geo = pixel_to_ground(eye, basis, (W / 2) as f32, y as f32, GROUND_H);
        if let Some((view_d, ground_d, _)) = geo {
            if y % 40 == 0 || d > 1.0 {
                println!("[l3a]   {y:4}  {view_d:8.1}  {ground_d:8.1}  {d:7.2}");
            }
            if d > 1.0 {
                max_view = Some(max_view.map_or(view_d, |m: f32| m.max(view_d)));
            }
        }
    }
    println!(
        "[l3a] {tag}: farthest shadow effect at view distance {:?} m",
        max_view
    );
    max_view
}

#[test]
#[ignore = "GPU + radius-10 terrain generation; run explicitly (see module docs)"]
fn l3a_boundary_evidence() -> Result<()> {
    pollster::block_on(async {
        let out = out_root().join(label());
        std::fs::create_dir_all(&out)?;
        let (device, queue, info) = acquire_device().await?;
        println!(
            "[l3a] adapter: {} · {:?} · driver {}",
            info.name, info.backend, info.driver_info
        );

        let assets = find_assets_dir().context("assets dir not found")?;
        let biome_dir = assets.join("materials").join("biomes");
        let mut viewport =
            ViewportRenderer::new(device, queue).context("ViewportRenderer::new failed")?;
        viewport.init_engine_adapter().await?;
        {
            let adapter = viewport
                .engine_adapter_mut()
                .context("engine adapter missing")?;
            adapter.set_biome_pack(Some(biome_dir));
            let baked = adapter.wait_env_bake(std::time::Duration::from_secs(60));
            println!("[l3a] ibl baked: {baked}");
        }
        let mut state = TerrainState::new();
        state.configure(SEED, PRIMARY_BIOME);
        state.set_noise_params(6, 2.0, 0.5, 50.0);
        state.set_world_archetype(WorldArchetypeId::Desert.default_archetype());
        let n = state.generate_terrain(10).context("generate_terrain")?;
        println!("[l3a] Desert radius 10: {n} chunks");
        viewport.upload_terrain_chunks_raw(&state.get_gpu_chunks());
        // Raking sun through the honest panel path (TimeOfDay is inert here).
        viewport
            .engine_adapter_mut()
            .context("engine adapter missing")?
            .set_lighting_params(&TerrainLightingParams {
                sun_dir: RAKE_SUN_DIR,
                ..TerrainLightingParams::default()
            });
        let texture = viewport.create_render_texture(W, H)?;
        let focal = Vec3::from_array(DESERT_FOCAL);

        // ── Part 1: radial profile at two altitudes ─────────────────────
        let mut boundaries = Vec::new();
        for (alt, tag) in [(ALT_HIGH, "alt219"), (ALT_LOW, "alt110")] {
            let (cam, eye) = survey_camera(focal, alt);
            let basis = camera_basis(eye, focal);
            // Self-check: the focal must project to the frame center.
            let c = world_to_pixel(eye, basis, focal).context("focal off-frame?!")?;
            anyhow::ensure!(
                (c.0 - (W as f32 - 1.0) * 0.5).abs() < 4.0
                    && (c.1 - (H as f32 - 1.0) * 0.5).abs() < 4.0,
                "camera model self-check failed: focal projected to {c:?}"
            );
            let shot = shoot_pair(&mut viewport, &texture, &cam, &out, tag)?;
            let b = radial_profile(&shot, eye, basis, tag);
            boundaries.push((alt, b));
        }
        if expect_fixed() {
            for (alt, b) in &boundaries {
                let b = b.context("no shadow effect found at all?!")?;
                anyhow::ensure!(
                    b > 2000.0,
                    "alt {alt}: farthest shadow effect at {b:.0} m — coverage boundary still inside 2 km"
                );
            }
        }

        // ── Part 2 + 3: motion leg with world-anchored features ─────────
        // Straight lateral sweep: focal translates along -X (600 m total),
        // orientation fixed — the rejection recording's regime at Y≈219.
        let path: Vec<Vec3> = (0..7)
            .map(|i| focal + Vec3::new(-100.0 * i as f32, 0.0, 0.0))
            .collect();
        let mut shots = Vec::new();
        let mut cams = Vec::new();
        for (i, f) in path.iter().enumerate() {
            let (cam, eye) = survey_camera(*f, ALT_HIGH);
            let basis = camera_basis(eye, *f);
            let shot = shoot_pair(&mut viewport, &texture, &cam, &out, &format!("path{i}"))?;
            shots.push(shot);
            cams.push((eye, basis));
        }
        // Choose 2 strong shadow features from the FINAL frame's diff map,
        // ≥600 px apart, from the mid/far field (upper half of the ground
        // rows) so they sit ~500-900 m from the path START camera.
        let (eye_last, basis_last) = cams[path.len() - 1];
        let mut candidates: Vec<(f64, u32, u32)> = Vec::new();
        for y in (H / 5..H / 2).step_by(4) {
            for x in (40..W - 40).step_by(4) {
                let d = patch_diff(
                    &shots[path.len() - 1].on,
                    &shots[path.len() - 1].off,
                    x,
                    y,
                    7,
                );
                candidates.push((d, x, y));
            }
        }
        candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let first = candidates.first().copied().context("no candidates")?;
        let second = candidates
            .iter()
            .find(|c| {
                ((c.1 as f32 - first.1 as f32).powi(2) + (c.2 as f32 - first.2 as f32).powi(2))
                    .sqrt()
                    > 250.0
            })
            .copied()
            .context("no second feature")?;
        let mut features = Vec::new();
        for (d, x, y) in [first, second] {
            let (view_d, ground_d, wp) =
                pixel_to_ground(eye_last, basis_last, x as f32, y as f32, GROUND_H)
                    .context("feature ray missed ground")?;
            println!(
                "[l3a] feature at world ({:.0},{:.0},{:.0}) — chosen from final frame px ({x},{y}), diff {d:.1}, view {view_d:.0} m ground {ground_d:.0} m",
                wp.x, wp.y, wp.z
            );
            features.push(wp);
        }
        // Track each feature across every path frame.
        let mut all_ok = true;
        for (fi, fp) in features.iter().enumerate() {
            print!("[l3a] feature{fi} shadow-effect by path point:");
            for (i, shot) in shots.iter().enumerate() {
                let (eye, basis) = cams[i];
                match world_to_pixel(eye, basis, *fp) {
                    Some((px, py)) => {
                        let d = patch_diff(&shot.on, &shot.off, px as u32, py as u32, 7);
                        let (vd, _, _) = pixel_to_ground(eye, basis, px, py, GROUND_H).unwrap_or((
                            0.0,
                            0.0,
                            Vec3::ZERO,
                        ));
                        print!("  p{i}:{d:.1}@{vd:.0}m");
                        if d < 1.0 {
                            all_ok = false;
                        }
                    }
                    None => print!("  p{i}:off-frame"),
                }
            }
            println!();
        }
        if expect_fixed() {
            anyhow::ensure!(
                all_ok,
                "a world-anchored shadow feature lost its shadow at some path point — \
                 shadow state still depends on the camera"
            );
            println!("[l3a] MOTION LEG PASS: world-anchored shadow state at every path point");
        } else {
            println!("[l3a] measurement leg complete (L3A_EXPECT not set — no assertions armed)");
        }
        println!("[l3a] label '{}' -> {}", label(), out.display());
        Ok(())
    })
}

/// L.3.B — THE CONTINUOUS LEG (the verification standard the L.3.A gate minted).
///
/// L.3.A's motion leg sampled 7 stations with a restore-settle-capture at each,
/// which guarantees the survey-cascade cache is fresh at every sample — so a
/// defect that lives BETWEEN refreshes is structurally invisible to it. This
/// test instead flies one continuous path, captures EVERY frame with no settle
/// and no shadow toggling, and judges stability frame-over-frame:
///
/// * per frame: survey-cascade refresh telemetry (count + trigger + window
///   drift) beside the frame's own metrics, so visual steps can be correlated
///   against refresh events (the director's diagnosis step);
/// * per frame: mean luma of world-anchored feature patches, reprojected into
///   that frame's camera — a world feature must not jump when the cache
///   refreshes;
/// * per frame: whole-frame mean luma, as a global step detector.
///
/// The metric is the frame-to-frame DELTA: smooth flight produces small deltas,
/// a cache snap produces a spike at exactly the refresh frame.
/// `L3B_EXPECT=stable` arms the assertions.
///
/// ```text
/// L3A_LABEL=<label> cargo test -p aw_editor --profile release-fast \
///     --test l3a_proof -- --ignored --nocapture l3b_continuous_flight
/// ```
#[test]
#[ignore = "GPU + radius-10 terrain generation; run explicitly (see module docs)"]
fn l3b_continuous_flight() -> Result<()> {
    pollster::block_on(async {
        let out = out_root().join(label());
        std::fs::create_dir_all(&out)?;
        let (device, queue, info) = acquire_device().await?;
        println!(
            "[l3b] adapter: {} · {:?} · driver {}",
            info.name, info.backend, info.driver_info
        );

        let assets = find_assets_dir().context("assets dir not found")?;
        let biome_dir = assets.join("materials").join("biomes");
        let mut viewport =
            ViewportRenderer::new(device, queue).context("ViewportRenderer::new failed")?;
        viewport.init_engine_adapter().await?;
        {
            let adapter = viewport
                .engine_adapter_mut()
                .context("engine adapter missing")?;
            adapter.set_biome_pack(Some(biome_dir));
            let baked = adapter.wait_env_bake(std::time::Duration::from_secs(60));
            println!("[l3b] ibl baked: {baked}");
        }
        let mut state = TerrainState::new();
        state.configure(SEED, PRIMARY_BIOME);
        state.set_noise_params(6, 2.0, 0.5, 50.0);
        state.set_world_archetype(WorldArchetypeId::Desert.default_archetype());
        // L.3.C: the RECORDING's world is radius 8 = 289 chunks (the director's
        // video chunk count). The L.3/L.3.A harnesses flew radius 10 = 441 and
        // were therefore measuring a different world than the gate rejected.
        let radius: i32 = std::env::var("L3B_RADIUS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8);
        let n = state.generate_terrain(radius).context("generate_terrain")?;
        println!("[l3b] Desert radius {radius}: {n} chunks");
        viewport.upload_terrain_chunks_raw(&state.get_gpu_chunks());
        viewport
            .engine_adapter_mut()
            .context("engine adapter missing")?
            .set_lighting_params(&TerrainLightingParams {
                sun_dir: RAKE_SUN_DIR,
                ..TerrainLightingParams::default()
            });
        let texture = viewport.create_render_texture(W, H)?;
        let focal0 = Vec3::from_array(DESERT_FOCAL);

        // L.3.B: `L3B_SHADOWS=off` flies the SAME deterministic path with
        // shadows disabled for the whole run. Differencing the two runs
        // frame-by-frame isolates the SHADOW CONTRIBUTION: terrain texture
        // mips, hex-tile phase, LOD and all other view-dependent shading are
        // identical in both runs and cancel exactly. This is the only metric
        // that survives realistic flight speed — at 40 m/frame a raw luma
        // patch moves several luma per frame from shading alone (measured: a
        // world point at 660 m stepped 6.35 luma with no boundary crossed and
        // with the cache frozen, i.e. with its shadow factor provably
        // constant). The toggle is applied ONCE here, never mid-flight, so it
        // cannot perturb the survey cache under test.
        let shadows_off = std::env::var("L3B_SHADOWS").as_deref() == Ok("off");
        if shadows_off {
            set_shadows(&mut viewport, false)?;
            println!("[l3b] shadows DISABLED for the whole flight (difference reference leg)");
        }

        // The director's route: Y~219 held, large lateral sweep, continuous.
        // Speed is parameterised (`L3B_STEP_M`, default 40 m/frame) because the
        // two refresh triggers ALIAS at some speeds: at 40 m/frame the 300 m
        // drift limit and the 8-frame tick both fire every 8 frames, so they
        // cannot be told apart. Slow (10 m/frame) isolates the TICK; fast
        // (120 m/frame) isolates DRIFT. `L3B_FRAMES` sets the path length.
        let step_m: f32 = std::env::var("L3B_STEP_M")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(40.0);
        let frames: usize = std::env::var("L3B_FRAMES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(40);
        // `L3B_YAW_STEP` (degrees/frame) switches from strafing to PANNING —
        // the camera holds position and sweeps its view. This is the regime the
        // director's "large lateral sweep" most likely means, and the one that
        // stresses the cached survey window hardest (see survey_camera_yaw).
        let yaw_step: f32 = std::env::var("L3B_YAW_STEP")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        if yaw_step != 0.0 {
            println!(
                "[l3b] route: {frames} frames PANNING at {yaw_step} deg/frame (position held)"
            );
        } else {
            println!("[l3b] route: {frames} frames at {step_m} m/frame");
        }

        // Warm-up so the first captured frame is not the pipeline-creation
        // frame (the L.3.A first-frame caster latency).
        let (cam0, eye0) = survey_camera(focal0, ALT_HIGH);
        for _ in 0..3 {
            let world = World::new();
            viewport.render(&texture, &cam0, &world, None, None, None, false, false, 0)?;
        }

        // World-anchored features: chosen from the first frame's mid/far field,
        // then held fixed in WORLD space for the whole flight.
        let basis0 = camera_basis(eye0, focal0);
        let mut features: Vec<Vec3> = Vec::new();
        for (fx, fy) in [(W / 2, H / 3), (W / 3, H / 4)] {
            if let Some((_, _, wp)) = pixel_to_ground(eye0, basis0, fx as f32, fy as f32, GROUND_H)
            {
                features.push(wp);
            }
        }
        println!("[l3b] tracking {} world-anchored features", features.len());

        // The survey window's own geometry, straight from the renderer — the
        // decisive numbers for whether its UV edge-fade band and its coverage
        // boundary can intrude on the VISIBLE far field (rather than sitting
        // safely outside it).
        {
            let (centre, radius) = viewport
                .engine_adapter_mut()
                .context("engine adapter missing")?
                .renderer()
                .shadow_survey_window();
            let half_extent = radius + 400.0; // C2_DRIFT_MARGIN
            let texel_m = 2.0 * half_extent / 2048.0;
            // csm_shadow_factor's edge fade: min(uv, 1-uv) * 10 clamped to 1,
            // i.e. shadow is attenuated across the outer 10% of UV on each side.
            let edge_fade_m = 0.10 * 2.0 * half_extent;
            println!(
                "[l3b] survey window: centre ({:.0},{:.0},{:.0}) radius {:.0} m -> half-extent \
                 {:.0} m ({:.0} m across), texel {:.2} m, UV edge-fade band {:.0} m per side",
                centre.x,
                centre.y,
                centre.z,
                radius,
                half_extent,
                2.0 * half_extent,
                texel_m,
                edge_fade_m
            );
            println!(
                "[l3b]   window centre is ~{:.0} m from the camera; the visible survey slice \
                 spans 500..3000 m, so the fade band starts {:.0} m from the window edge",
                (centre - eye0).length(),
                edge_fade_m
            );
        }

        struct FrameRec {
            idx: usize,
            mean: f64,
            band_mean: f64,
            band_shadow: f64,
            near_mean: f64,
            near_shadow: f64,
            feats: Vec<Option<f64>>,
            feat_dist: Vec<f32>,
            refreshes: u32,
            reason: &'static str,
            drift: f32,
        }
        let mut recs: Vec<FrameRec> = Vec::new();

        for i in 0..frames {
            let (focal, cam, eye) = if yaw_step != 0.0 {
                let yaw = YAW_DEG + yaw_step * i as f32;
                let (cam, eye) = survey_camera_yaw(focal0, ALT_HIGH, yaw);
                (focal0, cam, eye)
            } else {
                let focal = focal0 + Vec3::new(-step_m * i as f32, 0.0, 0.0);
                let (cam, eye) = survey_camera(focal, ALT_HIGH);
                (focal, cam, eye)
            };
            let basis = camera_basis(eye, focal);
            let img = grab_one(&mut viewport, &texture, &cam, &out, &format!("f{i:03}"))?;
            let (refreshes, reason, drift, _pending) = survey_telemetry(&mut viewport)?;
            let mut mean = 0.0f64;
            for p in img.pixels() {
                mean += 0.2126 * p.0[0] as f64 + 0.7152 * p.0[1] as f64 + 0.0722 * p.0[2] as f64;
            }
            mean /= (W * H) as f64;
            // Survey band = c2's domain (500..3000 m); near band = c0+c1's
            // domain (0..500 m), which measures whether the every-frame-refit
            // near pair CRAWLS during translation (director item 3).
            let (band_mean, band_shadow) = band_metrics(&img, eye, basis, 500.0, 3000.0);
            let (near_mean, near_shadow) = band_metrics(&img, eye, basis, 0.0, 500.0);
            // Feature patches only count while comfortably inside the frame —
            // an edge-clipped patch measures the frame border, not the world.
            let feats: Vec<Option<f64>> = features
                .iter()
                .map(|fp| {
                    world_to_pixel(eye, basis, *fp).and_then(|(px, py)| {
                        let inside =
                            px > 48.0 && py > 48.0 && px < W as f32 - 48.0 && py < H as f32 - 48.0;
                        inside.then(|| patch_luma(&img, px as u32, py as u32, 6))
                    })
                })
                .collect();
            // L.3.B: each feature's VIEW DISTANCE, so a luma step can be
            // attributed to the camera-relative boundary it crossed. The three
            // boundaries in the live path are all camera-relative: the 500 m
            // c1|c2 hard cascade select (an 11x texel-density jump, 0.34 m ->
            // 3.73 m), the 2100 m distance-fade start, and the 3000 m cutoff.
            let feat_dist: Vec<f32> = features.iter().map(|fp| (*fp - eye).length()).collect();
            recs.push(FrameRec {
                idx: i,
                mean,
                band_mean,
                band_shadow,
                near_mean,
                near_shadow,
                feats,
                feat_dist,
                refreshes,
                reason,
                drift,
            });
        }

        // Report. `dBandSh` (survey-band shadow-area, percentage points) is the
        // primary detector; refresh frames are flagged so steps can be
        // attributed to refresh events or to the intervals between them.
        println!(
            "[l3b] frame  band_mean dBandMean  bandSh% dBandSh%  dMean  refresh  trigger   drift  feats"
        );
        let mut worst_mean_step = 0.0f64;
        let mut worst_band_step = 0.0f64;
        let mut worst_shadow_step = 0.0f64;
        let mut worst_feat_step = 0.0f64;
        let mut worst_at = (0usize, "none", false);
        // Split the worst-step statistics by whether the step landed ON a
        // refresh frame — the discrimination the director asked for.
        let mut worst_shadow_on_refresh = 0.0f64;
        let mut worst_shadow_off_refresh = 0.0f64;
        let mut worst_near_shadow_step = 0.0f64;
        let mut worst_near_mean_step = 0.0f64;
        let mut worst_feat_detail = String::new();
        for w in recs.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            let d_mean = (b.mean - a.mean).abs();
            let d_band = (b.band_mean - a.band_mean).abs();
            let d_shadow = (b.band_shadow - a.band_shadow).abs() * 100.0;
            let refreshed = b.refreshes > a.refreshes;
            let mut feat_str = String::new();
            let mut max_feat = 0.0f64;
            for (i, (fa, fb)) in a.feats.iter().zip(b.feats.iter()).enumerate() {
                match (fa, fb) {
                    (Some(x), Some(y)) => {
                        let d = (y - x).abs();
                        max_feat = max_feat.max(d);
                        // Attribute the step: which camera-relative boundary
                        // did this feature cross between these two frames?
                        let (da, db) = (a.feat_dist[i], b.feat_dist[i]);
                        let crossed = |bound: f32| (da - bound).signum() != (db - bound).signum();
                        let tag = if crossed(500.0) {
                            "|c1c2"
                        } else if crossed(2100.0) {
                            "|fade"
                        } else if crossed(3000.0) {
                            "|cut "
                        } else {
                            "     "
                        };
                        feat_str.push_str(&format!(" {d:6.2}@{db:.0}m{tag}"));
                        if d > 6.0 {
                            worst_feat_detail = format!(
                                "frame {} feature{i}: {d:.2} luma step at view distance \
                                 {da:.0} -> {db:.0} m{}",
                                b.idx,
                                if tag.trim().is_empty() {
                                    String::new()
                                } else {
                                    format!(" (crossed {})", tag.trim_matches(['|', ' ']))
                                }
                            );
                        }
                    }
                    _ => feat_str.push_str("      off      "),
                }
            }
            println!(
                "[l3b] {:5}  {:9.2} {:9.2}  {:7.2} {:8.2} {:6.2}  {:7}  {:>7}  {:6.1} {}",
                b.idx,
                b.band_mean,
                d_band,
                b.band_shadow * 100.0,
                d_shadow,
                d_mean,
                if refreshed { "REFRESH" } else { "" },
                b.reason,
                b.drift,
                feat_str
            );
            if d_shadow > worst_shadow_step {
                worst_shadow_step = d_shadow;
                worst_at = (b.idx, b.reason, refreshed);
            }
            if refreshed {
                worst_shadow_on_refresh = worst_shadow_on_refresh.max(d_shadow);
            } else {
                worst_shadow_off_refresh = worst_shadow_off_refresh.max(d_shadow);
            }
            worst_mean_step = worst_mean_step.max(d_mean);
            worst_band_step = worst_band_step.max(d_band);
            worst_feat_step = worst_feat_step.max(max_feat);
            // Near band (c0+c1, refit every frame, no texel snapping): its
            // frame-to-frame instability is the crawl/shimmer measurement.
            worst_near_shadow_step =
                worst_near_shadow_step.max((b.near_shadow - a.near_shadow).abs() * 100.0);
            worst_near_mean_step = worst_near_mean_step.max((b.near_mean - a.near_mean).abs());
        }
        let total_refreshes = recs.last().map(|r| r.refreshes).unwrap_or(0)
            - recs.first().map(|r| r.refreshes).unwrap_or(0);
        println!("[l3b] {frames} frames at {step_m} m/frame: {total_refreshes} survey refreshes");
        println!(
            "[l3b] worst steps — survey-band shadow area {worst_shadow_step:.2} pp \
             (frame {}, trigger {}, on-refresh {}); band mean {worst_band_step:.2}; \
             whole-frame mean {worst_mean_step:.2}; feature {worst_feat_step:.2} luma",
            worst_at.0, worst_at.1, worst_at.2
        );
        println!(
            "[l3b] ATTRIBUTION — worst band-shadow step ON refresh frames: \
             {worst_shadow_on_refresh:.2} pp | BETWEEN refreshes: {worst_shadow_off_refresh:.2} pp"
        );
        println!(
            "[l3b] NEAR BAND (0..500 m, cascades 0+1 refit every frame): worst shadow-area step \
             {worst_near_shadow_step:.2} pp, worst mean step {worst_near_mean_step:.2} luma \
             — this is the c0/c1 crawl measurement (no texel snapping in the live path)"
        );
        if !worst_feat_detail.is_empty() {
            println!("[l3b] FEATURE STEP ATTRIBUTION — {worst_feat_detail}");
        }

        if std::env::var("L3B_EXPECT").as_deref() == Ok("stable") {
            // The survey band's shadow AREA is the acceptance metric: shadows
            // must not appear/vanish discretely. A smooth flight moves the
            // shadowed fraction gradually as the view changes; a cache snap or
            // coverage slide shows up as a spike.
            anyhow::ensure!(
                worst_shadow_step < 1.5,
                "survey-band shadow area stepped {worst_shadow_step:.2} percentage points \
                 between adjacent frames (frame {}, trigger {}, on-refresh {}) — shadows are \
                 still appearing/vanishing discretely during continuous flight",
                worst_at.0,
                worst_at.1,
                worst_at.2
            );
            anyhow::ensure!(
                worst_band_step < 2.0,
                "survey-band mean luma stepped {worst_band_step:.2} between adjacent frames"
            );
            anyhow::ensure!(
                worst_feat_step < 4.0,
                "world-anchored feature stepped {worst_feat_step:.2} luma between adjacent \
                 frames — a cached-cascade snap is still visible during continuous flight"
            );
            println!("[l3b] CONTINUOUS LEG PASS: no frame-to-frame shadow step");
        } else {
            println!("[l3b] measurement leg (L3B_EXPECT unset — no assertions armed)");
        }
        println!("[l3b] label '{}' -> {}", label(), out.display());
        Ok(())
    })
}
