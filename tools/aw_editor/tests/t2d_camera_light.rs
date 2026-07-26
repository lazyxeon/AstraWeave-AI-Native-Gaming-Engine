//! T.2d — the camera-light defect: characterization instrument.
//!
//! The director, 2026-07-25: *"the camera itself is a light source — as I get
//! really close to the terrain it gets brighter and casts shadows at the
//! borders of the light boundary like a light source."*
//!
//! This harness answers the one question that decides everything downstream:
//! **is the brightness world-anchored or camera-anchored?** It renders the
//! editor's own viewport path offscreen (same contract as
//! `terrain_ab_stations.rs`) with exactly-specified cameras, because the
//! editor cannot pin a camera (T2A_OUTCOME.md §1).
//!
//! Three experiments:
//!
//! * **A — distance sweep.** Same focal point (so the *same world fragment*
//!   sits at screen centre) viewed from a sweep of camera distances. If the
//!   luminance of that fragment changes with camera distance, shading is
//!   camera-dependent. This is the decisive test; everything else refines it.
//! * **B — lateral translation.** Camera and focal translated together by a
//!   known offset at constant height/yaw/pitch. If the bright band stays at
//!   the same *screen row*, it is camera-anchored; if it tracks terrain, it is
//!   world-anchored.
//! * **C — row profile.** Mean luminance per screen row. With a look-down
//!   camera, row maps monotonically to distance-from-camera, so a hard
//!   shading step shows up as an abrupt jump between adjacent rows while fog
//!   or any smooth falloff does not. Reports the largest step and its row.
//!
//! ```text
//! cargo test -p aw_editor --profile release-fast --test t2d_camera_light -- --ignored --nocapture
//!   T2D_OUT=<dir>     output root (default d:/tmp/t2d_staging)
//!   T2D_LABEL=<leg>   subdirectory + CSV label for this leg
//! ```

use anyhow::{Context, Result};
use astraweave_core::World;
use astraweave_terrain::world_archetypes::WorldArchetypeId;
use aw_editor_lib::terrain_integration::TerrainState;
use aw_editor_lib::viewport::{OrbitCamera, ViewportRenderer};
use glam::Vec3;
use std::path::PathBuf;
use std::sync::Arc;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
const TIME_OF_DAY: f32 = 12.0;

const SEED: u64 = 12345;
const RADIUS: i32 = 6;
const PRIMARY_BIOME: &str = "grassland";
const OCTAVES: usize = 6;
const LACUNARITY: f64 = 2.0;
const PERSISTENCE: f64 = 0.5;
const BASE_AMPLITUDE: f32 = 50.0;

/// Mediterranean grassland — the T.2a station-01 focal point. Flat, dominant
/// (91.3% of that world), and the surface every close-up judgment was made on.
const FOCAL: [f32; 3] = [0.0, 12.6, 2037.2];

/// Experiment A: camera distances, metres. Spans "really close" (the
/// director's condition) through the T.2a station distance (20) to far.
const SWEEP_DISTANCES: &[f32] = &[3.0, 5.0, 8.0, 12.0, 16.0, 20.0, 30.0, 45.0, 70.0, 110.0];

/// **A steep look-down is required, not stylistic.** `OrbitCamera` places the
/// eye at `focal + distance * dir(yaw, pitch)`, so camera altitude above the
/// focal point is `distance * sin(pitch)`. At the 35 deg first tried here, a
/// 4-metre orbit puts the eye only 2.3 m over the surface — *inside* the local
/// relief (base amplitude 50), and the first run duly produced four
/// byte-identical all-fog frames. At 80 deg the altitude is 0.985 x distance,
/// so even the 3 m station clears the ground. `sky_frac` below is the guard
/// that makes any recurrence self-evident instead of silently numeric.
const SWEEP_PITCH_DEG: f32 = 80.0;
const SWEEP_YAW_DEG: f32 = 45.0;

/// Experiments B and C need a wide spread of camera-to-fragment distance
/// *within one frame* (that is what turns a distance-banded term into a
/// visible band), which means an oblique view — from an altitude that clears
/// the terrain. 90 x sin(25 deg) = 38 m over the focal point.
const OBLIQUE_DISTANCE: f32 = 90.0;
const OBLIQUE_PITCH_DEG: f32 = 25.0;

/// Experiment B: lateral offsets applied to BOTH camera and focal, so the
/// camera-relative geometry is identical and only the world content changes.
const LATERAL_OFFSETS: &[f32] = &[0.0, 15.0, 30.0, 60.0];

fn out_root() -> PathBuf {
    PathBuf::from(std::env::var("T2D_OUT").unwrap_or_else(|_| "d:/tmp/t2d_staging".to_string()))
}

fn label() -> String {
    std::env::var("T2D_LABEL").unwrap_or_else(|_| "unlabelled".to_string())
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

fn generate_world_at_radius(
    archetype: WorldArchetypeId,
    key: &str,
    radius: i32,
) -> Result<TerrainState> {
    generate_world_tuned(archetype, key, radius, BASE_AMPLITUDE)
}

/// Same as [`generate_world_at_radius`] but with the noise amplitude exposed.
///
/// Experiment F wanted this because `fwidth(world_pos)` includes the **height**
/// derivative: on rough ground the pixel footprint swings across
/// `compute_material_lod`'s thresholds from one pixel to the next and the tier
/// boundary disintegrates into salt-and-pepper, whereas on smooth ground the
/// same threshold is crossed monotonically and resolves into a clean contour.
///
/// **It does not actually work, and that is a finding rather than a bug here.**
/// The amplitude written by `set_noise_params` is dead under the climate path —
/// see the `F_AMPLITUDES` docs for the full trace. The parameter is kept, and
/// the sweep kept, because the DEGENERATE flag it triggers is the evidence.
fn generate_world_tuned(
    archetype: WorldArchetypeId,
    key: &str,
    radius: i32,
    amplitude: f32,
) -> Result<TerrainState> {
    let mut state = TerrainState::new();
    state.configure(SEED, PRIMARY_BIOME);
    state.set_noise_params(OCTAVES, LACUNARITY, PERSISTENCE, amplitude);
    state.set_world_archetype(archetype.default_archetype());
    let n = state
        .generate_terrain(radius)
        .context("generate_terrain failed")?;
    println!("[t2d] world '{key}' (radius {radius}) generated {n} chunks");
    Ok(state)
}

fn generate_world_for(archetype: WorldArchetypeId, key: &str) -> Result<TerrainState> {
    generate_world_at_radius(archetype, key, RADIUS)
}

fn generate_world() -> Result<TerrainState> {
    generate_world_for(WorldArchetypeId::Mediterranean, "med")
}

/// Experiment D — the director's actual observing altitudes (2026-07-25).
///
/// The first pass of this beat swept 12-110 m and found only a smooth ~6%
/// gradient. The director's frames are from **camera Y 414.5 and 536.2** over
/// Desert, where the relevant ground distances run to many hundreds of metres —
/// entirely outside that range. Two facts make this range the interesting one:
/// `compute_material_lod`'s footprint thresholds land here, and `shadow_csm`'s
/// cascade splits are 10 / 50 / 200 / **1000** m.
const DIRECTOR_CAMERA_Y: &[f32] = &[414.5, 536.2];

/// T.2a's pinned desert close-up focal — a known point on the desert surface.
const DESERT_FOCAL: [f32; 3] = [43.1, 36.3, -1961.8];

/// Screen row -> horizontal ground distance from the camera, for locally flat
/// ground at the focal height.
///
/// Camera altitude above ground `h`, look-down `pitch`, vertical FOV `fovy`.
/// For row `y` the ray's depression below horizontal is
/// `pitch - atan(ndc * tan(fovy/2))`, and flat-ground range is `h / tan(depr)`.
/// Rows at or above the horizon return `None`.
fn row_to_ground_distance(y: usize, height: u32, h: f32, pitch_rad: f32, fovy: f32) -> Option<f32> {
    let ndc = 1.0 - 2.0 * (y as f32 + 0.5) / height as f32;
    let depression = pitch_rad - (ndc * (fovy * 0.5).tan()).atan();
    if depression <= 0.001 {
        return None;
    }
    let d = h / depression.tan();
    if d.is_finite() && d > 0.0 {
        Some(d)
    } else {
        None
    }
}

async fn acquire_device() -> Result<(Arc<wgpu::Device>, Arc<wgpu::Queue>)> {
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
        .context("no suitable wgpu adapter for the T.2d harness")?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("t2d-camera-light device"),
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
    Ok((Arc::new(device), Arc::new(queue)))
}

fn readback_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tex: &wgpu::Texture,
    width: u32,
    height: u32,
    mut encoder: wgpu::CommandEncoder,
) -> Result<Vec<u8>> {
    let bytes_per_pixel = 4u32;
    let unpadded = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("t2d-readback-staging"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    let (tx, rx) = std::sync::mpsc::channel();
    staging
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
    loop {
        if device.poll(wgpu::MaintainBase::Wait).is_ok() {
            if let Ok(r) = rx.try_recv() {
                r.context("map_async failed")?;
                break;
            }
        }
    }
    let mapped = staging.slice(..).get_mapped_range();
    let mut out = Vec::with_capacity((unpadded * height) as usize);
    for y in 0..height as usize {
        let src = y * padded as usize;
        out.extend_from_slice(&mapped[src..src + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    Ok(out)
}

fn luma_plane(rgba: &[u8]) -> Vec<f64> {
    rgba.chunks_exact(4)
        .map(|p| 0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64)
        .collect()
}

/// Mean luminance of a centred patch — the SAME world fragment across an
/// experiment-A sweep, because every sweep frame shares the focal point.
fn centre_patch_luma(luma: &[f64], w: usize, h: usize, half: usize) -> f64 {
    let cx = w / 2;
    let cy = h / 2;
    let mut sum = 0.0f64;
    let mut n = 0.0f64;
    for y in cy.saturating_sub(half)..(cy + half).min(h) {
        for x in cx.saturating_sub(half)..(cx + half).min(w) {
            sum += luma[y * w + x];
            n += 1.0;
        }
    }
    sum / n.max(1.0)
}

/// Fraction of the frame that is sky/fog rather than ground.
///
/// **This guard exists because the first run of this harness silently produced
/// four byte-identical all-fog frames** — the camera was inside the terrain and
/// the metrics looked like data. Ground here renders around luma 60; the fog
/// void around 172-185. Anything over ~0.9 means the frame is not a
/// measurement of terrain shading and must not be reported as one.
fn sky_fraction(luma: &[f64]) -> f64 {
    luma.iter().filter(|v| **v > 140.0).count() as f64 / luma.len() as f64
}

/// Mean luminance per screen row.
fn row_profile(luma: &[f64], w: usize, h: usize) -> Vec<f64> {
    (0..h)
        .map(|y| luma[y * w..(y + 1) * w].iter().sum::<f64>() / w as f64)
        .collect()
}

/// Largest single-row jump in the profile, and where it is.
///
/// A hard `if`-threshold shading term produces a step between adjacent rows;
/// fog, attenuation, and any smooth falloff do not. Rows near the horizon are
/// skipped because the terrain/sky boundary is a legitimate large step.
fn max_step(profile: &[f64], skip_top: usize) -> (f64, usize) {
    let mut best = 0.0f64;
    let mut at = 0usize;
    for y in skip_top..profile.len().saturating_sub(1) {
        let d = (profile[y + 1] - profile[y]).abs();
        if d > best {
            best = d;
            at = y;
        }
    }
    (best, at)
}

struct Frame {
    luma: Vec<f64>,
    rgba: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
async fn render_at(
    viewport: &mut ViewportRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &wgpu::Texture,
    world: &World,
    focal: Vec3,
    distance: f32,
    yaw_deg: f32,
    pitch_deg: f32,
) -> Result<Frame> {
    let mut camera = OrbitCamera::new(
        focal,
        distance,
        yaw_deg.to_radians(),
        pitch_deg.to_radians(),
    );
    camera.set_aspect(WIDTH as f32, HEIGHT as f32);
    // Two-frame settle, same contract as the T.2a harness.
    for _ in 0..2 {
        viewport
            .render(target, &camera, world, None, None, None, false, false, 0)
            .context("render failed")?;
    }
    let ldr = viewport
        .engine_ldr_texture()
        .context("engine_ldr_texture unavailable")?;
    let enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("t2d-readback"),
    });
    let rgba = readback_texture(device, queue, ldr, WIDTH, HEIGHT, enc)?;
    let luma = luma_plane(&rgba);
    Ok(Frame { luma, rgba })
}

#[test]
#[ignore = "GPU + full terrain generation; run explicitly (see module docs)"]
fn t2d_characterize_camera_light() -> Result<()> {
    pollster::block_on(async {
        let leg = label();
        let out_dir = out_root().join(&leg);
        std::fs::create_dir_all(&out_dir)?;

        let assets = find_assets_dir().context("assets dir not found")?;
        let biome_dir = assets.join("materials").join("biomes");

        let state = generate_world()?;
        let (device, queue) = acquire_device().await?;

        let mut viewport = ViewportRenderer::new(device.clone(), queue.clone())
            .context("ViewportRenderer::new failed")?;
        viewport
            .init_engine_adapter()
            .await
            .context("init_engine_adapter failed")?;
        {
            let adapter = viewport
                .engine_adapter_mut()
                .context("engine adapter missing")?;
            adapter.set_time_of_day(TIME_OF_DAY);
            adapter.set_biome_pack(Some(biome_dir.clone()));
        }
        viewport.upload_terrain_chunks_raw(&state.get_gpu_chunks());

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("t2d-target"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        let world = World::new();
        let w = WIDTH as usize;
        let h = HEIGHT as usize;
        let focal = Vec3::from_array(FOCAL);

        let mut csv = String::from(
            "experiment,param,centre_luma,frame_mean,max_row_step,step_row,step_frac_of_mean\n",
        );

        // ---- Experiment A: distance sweep, same world fragment at centre ----
        println!(
            "\n[t2d] === A: distance sweep (focal fixed, so screen centre is ONE world point) ==="
        );
        println!(
            "[t2d] {:>8}  {:>12}  {:>11}  {:>9}  {:>13}  {:>7}",
            "dist", "centre_luma", "frame_mean", "sky_frac", "max_row_step", "at_row"
        );
        for &d in SWEEP_DISTANCES {
            let f = render_at(
                &mut viewport,
                &device,
                &queue,
                &target,
                &world,
                focal,
                d,
                SWEEP_YAW_DEG,
                SWEEP_PITCH_DEG,
            )
            .await?;
            let centre = centre_patch_luma(&f.luma, w, h, 32);
            let mean = f.luma.iter().sum::<f64>() / f.luma.len() as f64;
            let prof = row_profile(&f.luma, w, h);
            let (step, row) = max_step(&prof, h / 4);
            println!(
                "[t2d] {d:>8.1}  {centre:>12.4}  {mean:>11.4}  {sky:>9.4}  {step:>13.4}  {row:>7}{flag}",
                sky = sky_fraction(&f.luma),
                flag = if sky_fraction(&f.luma) > 0.9 {
                    "  <-- DEGENERATE: camera inside terrain, NOT a measurement"
                } else {
                    ""
                }
            );
            csv.push_str(&format!(
                "sweep,{d},{centre:.4},{mean:.4},{step:.4},{row},{:.5}\n",
                step / mean.max(1e-6)
            ));
            image::RgbaImage::from_raw(WIDTH, HEIGHT, f.rgba)
                .context("RgbaImage::from_raw")?
                .save(out_dir.join(format!("A_dist_{:03.0}.png", d)))?;
        }

        // ---- Experiment B: lateral translation at constant camera geometry ----
        println!("\n[t2d] === B: lateral translation (camera geometry identical, world content differs) ===");
        println!(
            "[t2d] {:>8}  {:>12}  {:>11}  {:>9}  {:>13}  {:>7}",
            "offset", "centre_luma", "frame_mean", "sky_frac", "max_row_step", "at_row"
        );
        let mut lateral_rows: Vec<usize> = Vec::new();
        for &off in LATERAL_OFFSETS {
            let f = render_at(
                &mut viewport,
                &device,
                &queue,
                &target,
                &world,
                focal + Vec3::new(off, 0.0, 0.0),
                OBLIQUE_DISTANCE,
                SWEEP_YAW_DEG,
                OBLIQUE_PITCH_DEG,
            )
            .await?;
            let centre = centre_patch_luma(&f.luma, w, h, 32);
            let mean = f.luma.iter().sum::<f64>() / f.luma.len() as f64;
            let prof = row_profile(&f.luma, w, h);
            let (step, row) = max_step(&prof, h / 4);
            lateral_rows.push(row);
            println!(
                "[t2d] {off:>8.1}  {centre:>12.4}  {mean:>11.4}  {sky:>9.4}  {step:>13.4}  {row:>7}{flag}",
                sky = sky_fraction(&f.luma),
                flag = if sky_fraction(&f.luma) > 0.9 {
                    "  <-- DEGENERATE: camera inside terrain, NOT a measurement"
                } else {
                    ""
                }
            );
            csv.push_str(&format!(
                "lateral,{off},{centre:.4},{mean:.4},{step:.4},{row},{:.5}\n",
                step / mean.max(1e-6)
            ));
            image::RgbaImage::from_raw(WIDTH, HEIGHT, f.rgba)
                .context("RgbaImage::from_raw")?
                .save(out_dir.join(format!("B_lat_{:03.0}.png", off)))?;
        }

        // ---- Experiment C: full row profile at one close station ----
        let f = render_at(
            &mut viewport,
            &device,
            &queue,
            &target,
            &world,
            focal,
            OBLIQUE_DISTANCE,
            SWEEP_YAW_DEG,
            OBLIQUE_PITCH_DEG,
        )
        .await?;
        let prof = row_profile(&f.luma, w, h);
        let mut prof_csv = String::from("row,mean_luma,delta_to_next\n");
        for y in 0..prof.len() {
            let d = if y + 1 < prof.len() {
                prof[y + 1] - prof[y]
            } else {
                0.0
            };
            prof_csv.push_str(&format!("{y},{:.5},{:.5}\n", prof[y], d));
        }
        std::fs::write(out_dir.join("C_row_profile.csv"), prof_csv)?;
        println!(
            "\n[t2d] C: row profile -> {}",
            out_dir.join("C_row_profile.csv").display()
        );

        println!("\n[t2d] lateral step rows: {lateral_rows:?}");
        println!(
            "[t2d]   identical rows across lateral offsets => the boundary is CAMERA-anchored;"
        );
        println!("[t2d]   rows that track terrain => WORLD-anchored.");

        // ---- Experiment D: the director's altitudes, Desert, out to ~1000 m ----
        println!("\n[t2d] === D: DESERT at the director's camera altitudes (414.5 / 536.2) ===");
        let desert = generate_world_for(WorldArchetypeId::Desert, "desert")?;
        viewport.upload_terrain_chunks_raw(&desert.get_gpu_chunks());
        let d_focal = Vec3::from_array(DESERT_FOCAL);
        let fovy = 60_f32.to_radians();

        for &cam_y in DIRECTOR_CAMERA_Y {
            for &pitch_deg in &[20.0_f32, 30.0] {
                let pitch = pitch_deg.to_radians();
                let altitude = cam_y - d_focal.y;
                // Choose the orbit radius that puts the eye at this altitude.
                let dist = altitude / pitch.sin();
                let f = render_at(
                    &mut viewport,
                    &device,
                    &queue,
                    &target,
                    &world,
                    d_focal,
                    dist,
                    SWEEP_YAW_DEG,
                    pitch_deg,
                )
                .await?;
                let prof = row_profile(&f.luma, w, h);
                let sky = sky_fraction(&f.luma);

                // Largest step, reported in GROUND DISTANCE rather than row.
                let mut best = 0.0f64;
                let mut best_row = 0usize;
                for y in 0..prof.len().saturating_sub(1) {
                    // Only consider rows that actually see ground.
                    if row_to_ground_distance(y, HEIGHT, altitude, pitch, fovy).is_none() {
                        continue;
                    }
                    let dd = (prof[y + 1] - prof[y]).abs();
                    if dd > best {
                        best = dd;
                        best_row = y;
                    }
                }
                let best_dist = row_to_ground_distance(best_row, HEIGHT, altitude, pitch, fovy);
                let mean = f.luma.iter().sum::<f64>() / f.luma.len() as f64;
                println!(
                    "[t2d] camY={cam_y:>7.1} pitch={pitch_deg:>4.0} orbit={dist:>7.1}  mean={mean:>7.3}  sky={sky:>5.3}  \
                     max_step={best:>6.3} at row {best_row} (~{:.0} m)",
                    best_dist.unwrap_or(f32::NAN)
                );
                csv.push_str(&format!(
                    "director,{cam_y}/{pitch_deg},{:.4},{mean:.4},{sky:.4},{best:.4},{best_row},{:.5}\n",
                    centre_patch_luma(&f.luma, w, h, 32),
                    best / mean.max(1e-6)
                ));

                // Full luma-vs-ground-distance profile: this is the artifact that
                // shows a boundary as a step against DISTANCE, not against row.
                let name = format!("D_desert_y{cam_y:.0}_p{pitch_deg:.0}");
                let mut dcsv = String::from("row,ground_dist_m,mean_luma,delta_to_next\n");
                for y in 0..prof.len() {
                    let gd = row_to_ground_distance(y, HEIGHT, altitude, pitch, fovy);
                    let dd = if y + 1 < prof.len() {
                        prof[y + 1] - prof[y]
                    } else {
                        0.0
                    };
                    dcsv.push_str(&format!(
                        "{y},{},{:.5},{:.5}\n",
                        gd.map(|v| format!("{v:.2}")).unwrap_or_default(),
                        prof[y],
                        dd
                    ));
                }
                std::fs::write(out_dir.join(format!("{name}.csv")), dcsv)?;
                image::RgbaImage::from_raw(WIDTH, HEIGHT, f.rgba)
                    .context("RgbaImage::from_raw")?
                    .save(out_dir.join(format!("{name}.png")))?;
            }
        }

        std::fs::write(out_dir.join("metrics.csv"), csv)?;
        println!("[t2d] metrics -> {}", out_dir.join("metrics.csv").display());
        Ok(())
    })
}

// ======================================================================
// Experiment E — the director's actual frames, reproduced, then
//                discriminated on RENDER RESOLUTION
// ======================================================================
//
// Passes 1 and 2 measured LUMINANCE. Inspection of the two director frames
// (`d:/tmp/t2d_staging/boundary_y{414,536}.png`, contrast-stretched) shows the
// boundary is not primarily a brightness edge at all: **near terrain carries
// fine high-frequency normal-map grain and far terrain does not**, with a sharp
// transition. Row-mean luminance is nearly blind to that; the metric here is
// grain energy (see `grain_profile`).
//
// ## Recovering the camera from two position readouts
//
// The readouts give eye positions only — no yaw/pitch. But `OrbitCamera` places
// the eye at `focal + distance * (cos y cos p, sin p, sin y cos p)`, so a pure
// zoom (the mouse wheel; it changes `distance` alone) translates the eye exactly
// along that unit vector. The two eyes differ by (83.6, 121.7, 85.0), |d|=170.4:
//
//   sin(pitch) = 121.7/170.4 = 0.7142  -> pitch = 45.6 deg
//   cos(yaw)   = (83.6/170.4)/cos(p)   -> yaw   = 45.5 deg
//   sin(yaw)   = (85.0/170.4)/cos(p)   -> yaw   = 45.4 deg   (agrees)
//
// Two independent components agreeing, and landing on `OrbitCamera::default()`s
// 45 deg yaw, is what makes this a recovery rather than a guess. The focal point
// is underdetermined (any focal along the same ray reproduces the eye) and does
// not need determining: the image depends only on eye + yaw + pitch + fov +
// aspect.
const E_YAW_DEG: f32 = 45.5;
const E_PITCH_DEG: f32 = 45.6;

/// The two eye positions, verbatim from the in-frame Camera readout.
const E_EYES: &[[f32; 3]] = &[[-770.1, 414.5, 518.9], [-686.5, 536.2, 603.9]];

/// Viewport aspect measured off the screenshots: the render area is x 224..1186,
/// y 105..606 -> 962 x 501 in both frames.
const E_ASPECT: f32 = 962.0 / 501.0;

/// **The discriminator.** `compute_material_lod` keys on
/// `length(fwidth(world_pos))` — the world-space size of one pixel — which is
/// inversely proportional to render height. Double the height and every
/// footprint threshold moves twice as far away, so a footprint-keyed boundary
/// migrates toward the horizon. Anything keyed to view-space depth or true
/// distance (cascade splits, fog, a chunk-distance test) is resolution-invariant
/// and does not move at all. Reported as a NORMALISED row so the two cases are
/// directly comparable.
const E_HEIGHTS: &[u32] = &[334, 501, 1002];

/// The live editor generates at chunk_radius 10 (21x21 = 441 chunks) —
/// `terrain_panel.rs` default, and exactly the "Terrain (441 chunks)" the
/// Hierarchy panel reads in both frames. Both existing harnesses use 6, which is
/// a different world; that is one of two candidate reasons pass 2 did not
/// reproduce the boundary.
const E_RADIUS: i32 = 10;

/// Grain energy per screen row: mean |L - 3x3 box mean| over the central half of
/// the columns.
///
/// This is the metric passes 1 and 2 lacked. A detail cutoff (normal-map relief
/// present on one side of a line, absent on the other) moves row-mean luminance
/// by well under a percent — §2.2 measured exactly that and correctly concluded
/// "no boundary" *for luminance*. The same cutoff moves grain energy by tens of
/// percent.
fn grain_profile(luma: &[f64], w: usize, h: usize) -> Vec<f64> {
    let x0 = (w / 4).max(1);
    let x1 = (w - w / 4).min(w - 1);
    (0..h)
        .map(|y| {
            if y == 0 || y + 1 >= h || x1 <= x0 {
                return 0.0;
            }
            let mut sum = 0.0f64;
            let mut n = 0.0f64;
            for x in x0..x1 {
                let mut box_sum = 0.0f64;
                for dy in 0..3 {
                    for dx in 0..3 {
                        box_sum += luma[(y + dy - 1) * w + (x + dx - 1)];
                    }
                }
                sum += (luma[y * w + x] - box_sum / 9.0).abs();
                n += 1.0;
            }
            sum / n.max(1.0)
        })
        .collect()
}

/// Row of the largest sustained rise in grain going *down* the screen, using
/// windows of `h/8`. Sustained-window rather than adjacent-row because the
/// single largest adjacent step is always terrain content.
fn grain_edge(profile: &[f64]) -> (usize, f64, f64, f64) {
    let h = profile.len();
    let win = (h / 8).max(8);
    let mut best = f64::NEG_INFINITY;
    let mut at = 0usize;
    let (mut lo, mut hi) = (0.0, 0.0);
    if h < 2 * win + 1 {
        return (0, 0.0, 0.0, 0.0);
    }
    for t in win..h - win {
        let a = profile[t - win..t].iter().sum::<f64>() / win as f64;
        let b = profile[t..t + win].iter().sum::<f64>() / win as f64;
        if b - a > best {
            best = b - a;
            at = t;
            lo = a;
            hi = b;
        }
    }
    (at, best, lo, hi)
}

#[allow(clippy::too_many_arguments)]
async fn render_eye(
    viewport: &mut ViewportRenderer,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target: &wgpu::Texture,
    world: &World,
    eye: Vec3,
    yaw_deg: f32,
    pitch_deg: f32,
    ground_y: f32,
    width: u32,
    height: u32,
) -> Result<Frame> {
    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians();
    // Any (focal, distance) pair on the same ray reproduces this eye; choose the
    // one whose focal sits at `ground_y`, so the centre ray lands on the surface.
    let distance = (eye.y - ground_y) / pitch.sin();
    let dir = Vec3::new(
        yaw.cos() * pitch.cos(),
        pitch.sin(),
        yaw.sin() * pitch.cos(),
    );
    let focal = eye - dir * distance;
    let mut camera = OrbitCamera::new(focal, distance, yaw, pitch);
    camera.set_aspect(width as f32, height as f32);
    let placed = camera.position();
    assert!(
        (placed - eye).length() < 0.5,
        "camera placement missed the requested eye: {placed:?} vs {eye:?}"
    );
    for _ in 0..2 {
        viewport
            .render(target, &camera, world, None, None, None, false, false, 0)
            .context("render failed")?;
    }
    let ldr = viewport
        .engine_ldr_texture()
        .context("engine_ldr_texture unavailable")?;
    let enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("t2d-e-readback"),
    });
    let rgba = readback_texture(device, queue, ldr, width, height, enc)?;
    let luma = luma_plane(&rgba);
    Ok(Frame { luma, rgba })
}

#[test]
#[ignore = "GPU + radius-10 terrain generation; run explicitly (see module docs)"]
fn t2d_e_reproduce_and_discriminate() -> Result<()> {
    pollster::block_on(async {
        let leg = label();
        let out_dir = out_root().join(format!("E_{leg}"));
        std::fs::create_dir_all(&out_dir)?;

        let assets = find_assets_dir().context("assets dir not found")?;
        let biome_dir = assets.join("materials").join("biomes");

        let (device, queue) = acquire_device().await?;
        let mut viewport = ViewportRenderer::new(device.clone(), queue.clone())
            .context("ViewportRenderer::new failed")?;
        viewport
            .init_engine_adapter()
            .await
            .context("init_engine_adapter failed")?;
        {
            let adapter = viewport
                .engine_adapter_mut()
                .context("engine adapter missing")?;
            adapter.set_time_of_day(TIME_OF_DAY);
            adapter.set_biome_pack(Some(biome_dir.clone()));
        }

        let world = World::new();
        let mut csv = String::from(
            "archetype,eye_y,height,edge_row,edge_norm,grain_above,grain_below,ratio,sky_frac\n",
        );

        // Both archetypes: the editor default is ContinentalTemperate, but the
        // frames read as desert. Rendering both settles it by eye rather than by
        // assumption.
        for (arch, aname) in [
            (WorldArchetypeId::Desert, "desert"),
            (WorldArchetypeId::ContinentalTemperate, "continental"),
        ] {
            let state = generate_world_at_radius(arch, aname, E_RADIUS)?;
            viewport.upload_terrain_chunks_raw(&state.get_gpu_chunks());

            for &height in E_HEIGHTS {
                let width = ((height as f32) * E_ASPECT).round() as u32;
                let target = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("t2d-e-target"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
                });

                for eye_arr in E_EYES {
                    let eye = Vec3::from_array(*eye_arr);
                    let f = render_eye(
                        &mut viewport,
                        &device,
                        &queue,
                        &target,
                        &world,
                        eye,
                        E_YAW_DEG,
                        E_PITCH_DEG,
                        DESERT_FOCAL[1],
                        width,
                        height,
                    )
                    .await?;
                    let w = width as usize;
                    let h = height as usize;
                    let g = grain_profile(&f.luma, w, h);
                    let (row, rise, lo, hi) = grain_edge(&g);
                    let sky = sky_fraction(&f.luma);
                    println!(
                        "[t2d-E] {aname:>12} eyeY={:>6.1} {width}x{height}  edge row {row:>4} \
                         (norm {:.3})  grain {lo:.3} -> {hi:.3} (x{:.2}, +{rise:.3})  sky={sky:.3}",
                        eye.y,
                        row as f64 / h as f64,
                        hi / lo.max(1e-6),
                    );
                    csv.push_str(&format!(
                        "{aname},{:.1},{height},{row},{:.4},{lo:.4},{hi:.4},{:.4},{sky:.4}\n",
                        eye.y,
                        row as f64 / h as f64,
                        hi / lo.max(1e-6)
                    ));

                    let name = format!("E_{aname}_y{:.0}_h{height}", eye.y);
                    image::RgbaImage::from_raw(width, height, f.rgba)
                        .context("RgbaImage::from_raw")?
                        .save(out_dir.join(format!("{name}.png")))?;
                    let mut gcsv = String::from("row,row_norm,grain\n");
                    for (y, v) in g.iter().enumerate() {
                        gcsv.push_str(&format!("{y},{:.5},{v:.5}\n", y as f64 / h as f64));
                    }
                    std::fs::write(out_dir.join(format!("{name}.csv")), gcsv)?;
                }
            }
        }

        std::fs::write(out_dir.join("E_metrics.csv"), csv)?;
        println!("[t2d-E] -> {}", out_dir.display());
        Ok(())
    })
}

// ======================================================================
// Experiment F — the roughness variable: does the tier boundary become a
//                LINE on smooth ground?
// ======================================================================
//
// Experiment E reproduced the director's camera exactly (`sky_frac` 0.000, as in
// both of the frames) but produced no clean boundary — instead the LOD2 region
// carried per-pixel salt-and-pepper. The A/B (`mat_lod = 1u`) showed that
// pinning the tier *removes* 40-55% of far-field high-frequency energy, i.e.
// `compute_material_lod` was ADDING noise, not cutting detail.
//
// The reconciliation is that `fwidth(world_pos)` is a derivative of the **full
// world position, height included**. On rough ground the height derivative
// dominates and swings the footprint across the 2.0 threshold from one pixel to
// the next, so the tier boundary disintegrates into dithering spread over a wide
// band. On smooth ground the footprint grows monotonically with distance and the
// same threshold resolves into a single clean contour.
//
// E generated the editor's default radius-10 Desert, which is a mountainous
// world; the director's frames are gentle dunes. So this sweeps amplitude with
// everything else held identical. Prediction: at low amplitude the grain edge
// sharpens and lands at the row where the flat-ground footprint model says
// footprint = 2.0.
/// **Result: this sweep is INERT, and that is itself the finding.** All three
/// amplitudes produce byte-identical frames. The chain is
/// `terrain_panel.rs:1064` (the "Base Amplitude" slider, 10..200) ->
/// `terrain_panel.rs:2016` `set_noise_params(..)` ->
/// `terrain_integration.rs:167` `config.noise.base_elevation.amplitude`, but
/// `noise_gen.rs:575` documents that read as **replaced** by
/// `params.base_elevation_amplitude`, and `regional_archetype_mask.rs:469`
/// blends that from archetype splines — never from `config`. Octaves,
/// lacunarity and persistence are still read; amplitude alone is dead.
///
/// So the editor's Base Amplitude slider does nothing under the climate path
/// (the default). That is an Integration-Completeness §3 violation
/// (settable-but-never-read), and it is reported rather than worked around
/// here because working around it would hide it.
///
/// The `identical to previous leg` guard below is the generalisation of this
/// harness's `sky_frac` rule: **a swept parameter that produces identical
/// output is not a measurement.**
const F_AMPLITUDES: &[f32] = &[50.0, 20.0, 6.0];

/// Flat-ground pixel footprint, the quantity `compute_material_lod` thresholds.
///
/// One pixel spans `d*k` across the view and `d*k/sin(alpha)` along it
/// (foreshortened), with `d = h/sin(alpha)`; `fwidth` is `|dpdx| + |dpdy|`
/// componentwise followed by `length()`, so the two combine as
/// `h*k*sqrt(1 + sin^2 a) / sin^2 a`.
fn flat_ground_footprint(h: f32, alpha: f32, height_px: u32, fovy: f32) -> f32 {
    let k = 2.0 * (fovy * 0.5).tan() / height_px as f32;
    let s = alpha.sin();
    h * k * (1.0 + s * s).sqrt() / (s * s)
}

/// Normalised screen row at which the flat-ground footprint crosses `target`.
/// Returns `None` if the threshold is not reached anywhere in the frame.
fn row_of_footprint(h: f32, target: f32, height_px: u32, fovy: f32, pitch: f32) -> Option<f32> {
    let alpha_of = |norm: f32| pitch - (((1.0 - 2.0 * norm) * (fovy * 0.5).tan()).atan());
    if flat_ground_footprint(h, alpha_of(0.999), height_px, fovy) > target {
        return None;
    }
    let (mut lo, mut hi) = (0.001f32, 0.999f32);
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if flat_ground_footprint(h, alpha_of(mid), height_px, fovy) > target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(0.5 * (lo + hi))
}

/// Sharpness of the grain edge: the largest single-row change in the smoothed
/// grain profile, normalised by the profile's own mean. A clean contour scores
/// high; dithering spread over a band scores low.
fn edge_sharpness(profile: &[f64]) -> (f64, usize) {
    let n = profile.len();
    if n < 32 {
        return (0.0, 0);
    }
    let sm: Vec<f64> = (0..n)
        .map(|i| {
            let a = i.saturating_sub(3);
            let b = (i + 4).min(n);
            profile[a..b].iter().sum::<f64>() / (b - a) as f64
        })
        .collect();
    let mean = sm.iter().sum::<f64>() / n as f64;
    let mut best = 0.0f64;
    let mut at = 0usize;
    for i in n / 10..n - n / 10 - 1 {
        let d = (sm[i + 1] - sm[i]).abs();
        if d > best {
            best = d;
            at = i;
        }
    }
    (best / mean.max(1e-9), at)
}

#[test]
#[ignore = "GPU + radius-10 terrain generation; run explicitly (see module docs)"]
fn t2d_f_amplitude_sweep() -> Result<()> {
    pollster::block_on(async {
        let leg = label();
        let out_dir = out_root().join(format!("F_{leg}"));
        std::fs::create_dir_all(&out_dir)?;

        let assets = find_assets_dir().context("assets dir not found")?;
        let biome_dir = assets.join("materials").join("biomes");

        let (device, queue) = acquire_device().await?;
        let mut viewport = ViewportRenderer::new(device.clone(), queue.clone())
            .context("ViewportRenderer::new failed")?;
        viewport
            .init_engine_adapter()
            .await
            .context("init_engine_adapter failed")?;
        {
            let adapter = viewport
                .engine_adapter_mut()
                .context("engine adapter missing")?;
            adapter.set_time_of_day(TIME_OF_DAY);
            adapter.set_biome_pack(Some(biome_dir.clone()));
        }

        let height = 501u32;
        let width = ((height as f32) * E_ASPECT).round() as u32;
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("t2d-f-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        let world = World::new();
        let fovy = 60_f32.to_radians();
        let pitch = E_PITCH_DEG.to_radians();

        let mut csv = String::from(
            "amplitude,eye_y,edge_row,edge_norm,predicted_norm,delta_px,sharpness,grain_ratio\n",
        );
        // Keyed by eye, so each amplitude leg is compared against the previous
        // leg at the SAME camera.
        let mut previous: std::collections::HashMap<String, Vec<f64>> =
            std::collections::HashMap::new();
        println!(
            "\n[t2d-F] === amplitude sweep: does the LOD1|2 contour sharpen on smooth ground? ==="
        );
        println!("[t2d-F] director's observed edges: y414 norm 0.527, y536 norm 0.749");

        for &amp in F_AMPLITUDES {
            let state = generate_world_tuned(
                WorldArchetypeId::Desert,
                &format!("desert_a{amp:.0}"),
                E_RADIUS,
                amp,
            )?;
            viewport.upload_terrain_chunks_raw(&state.get_gpu_chunks());

            for eye_arr in E_EYES {
                let eye = Vec3::from_array(*eye_arr);
                let f = render_eye(
                    &mut viewport,
                    &device,
                    &queue,
                    &target,
                    &world,
                    eye,
                    E_YAW_DEG,
                    E_PITCH_DEG,
                    DESERT_FOCAL[1],
                    width,
                    height,
                )
                .await?;
                let w = width as usize;
                let h = height as usize;
                let g = grain_profile(&f.luma, w, h);
                let (row, _rise, lo, hi) = grain_edge(&g);
                let (sharp, sharp_row) = edge_sharpness(&g);
                // The flat-ground model uses the height above the LOCAL ground,
                // which for a low-amplitude world is close to the focal height.
                let cam_h = eye.y - DESERT_FOCAL[1];
                let pred = row_of_footprint(cam_h, 2.0, height, fovy, pitch);
                let pred_s = pred
                    .map(|p| format!("{p:.3}"))
                    .unwrap_or_else(|| "unreached".into());
                let delta = pred
                    .map(|p| (p - sharp_row as f32 / h as f32) * h as f32)
                    .unwrap_or(f32::NAN);
                // The generalised degenerate guard: a swept parameter that
                // leaves the frame untouched has not been measured.
                let key = format!("{:.0}", eye.y);
                let degenerate = previous
                    .get(&key)
                    .is_some_and(|prev| prev.len() == f.luma.len() && *prev == f.luma);
                previous.insert(key, f.luma.clone());

                println!(
                    "[t2d-F] amp={amp:>5.1} eyeY={:>6.1}  grain-edge row {row:>4} (norm {:.3})  \
                     sharpest row {sharp_row:>4} sharpness {sharp:.4}  predicted {pred_s} \
                     (delta {delta:>6.1} px)  grain {lo:.3}->{hi:.3}{}",
                    eye.y,
                    row as f64 / h as f64,
                    if degenerate {
                        "   <-- DEGENERATE: byte-identical to the previous amplitude, \
                         the swept knob is INERT (see F_AMPLITUDES docs)"
                    } else {
                        ""
                    },
                );
                csv.push_str(&format!(
                    "{amp},{:.1},{row},{:.4},{pred_s},{delta:.1},{sharp:.5},{:.4}\n",
                    eye.y,
                    row as f64 / h as f64,
                    hi / lo.max(1e-6)
                ));

                let name = format!("F_desert_a{amp:.0}_y{:.0}", eye.y);
                image::RgbaImage::from_raw(width, height, f.rgba)
                    .context("RgbaImage::from_raw")?
                    .save(out_dir.join(format!("{name}.png")))?;
                let mut gcsv = String::from("row,row_norm,grain\n");
                for (y, v) in g.iter().enumerate() {
                    gcsv.push_str(&format!("{y},{:.5},{v:.5}\n", y as f64 / h as f64));
                }
                std::fs::write(out_dir.join(format!("{name}.csv")), gcsv)?;
            }
        }

        std::fs::write(out_dir.join("F_metrics.csv"), csv)?;
        println!("[t2d-F] -> {}", out_dir.display());
        Ok(())
    })
}
