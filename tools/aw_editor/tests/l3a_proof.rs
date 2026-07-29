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
    let pitch = PITCH_DEG.to_radians();
    let yaw = YAW_DEG.to_radians();
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
