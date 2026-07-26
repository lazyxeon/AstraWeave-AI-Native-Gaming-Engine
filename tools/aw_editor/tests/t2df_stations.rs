//! T.2d.F — before/after station captures + min-spec GPU timing for the
//! material-LOD tier retirement.
//!
//! The T.2d diagnosis (docs/audits/T2D_CAMERA_LIGHT.md §10) convicted
//! `compute_material_lod`'s LOD1|2 threshold (pixel footprint 2.0,
//! brdf_common.wgsl) as the director-observed camera-anchored boundary, plus
//! LOD 2's per-pixel tier dithering (40-55% of far-field high-frequency
//! energy). The ratified fix is to DELETE the tiers. This harness produces the
//! proof artifacts:
//!
//! * `t2df_capture_stations` — renders the pinned stations through the
//!   editor's own path (`ViewportRenderer::render` → `capture_frame_png`, the
//!   ED-2 capture function; the live widget only adds egui presentation on
//!   top, ED2_OUTCOME.md §3.1) and writes each frame's ED-2 `.camera.json`
//!   sidecar. Run once per leg:
//!
//!   ```text
//!   T2DF_LABEL=before cargo test -p aw_editor --profile release-fast \
//!       --test t2df_stations -- --ignored --nocapture t2df_capture_stations
//!   ```
//!
//! * `t2df_perf_main_pass` — min-spec GPU timing at the distance-heavy
//!   boundary framing, real `TIMESTAMP_QUERY` via the production
//!   `GpuProfiler`, medians over 300 frames after warm-up — the same
//!   methodology the water budget measurements used (water.md §"Measured on
//!   the documented min-spec").
//!
//! Station camera states are ED-2 `CameraState`s; the same five stations are
//! pinned in `.editor_preferences.json` (`camera_stations`) so the director's
//! editor has them under Camera → Go for the closing re-check.

use anyhow::{Context, Result};
use astraweave_core::World;
use astraweave_terrain::world_archetypes::WorldArchetypeId;
use aw_editor_lib::terrain_integration::TerrainState;
use aw_editor_lib::viewport::camera::{CameraState, CameraStation};
use aw_editor_lib::viewport::{OrbitCamera, ViewportRenderer};
use glam::Vec3;
use std::path::PathBuf;
use std::sync::Arc;

const TIME_OF_DAY: f32 = 12.0;
const SEED: u64 = 12345;
const PRIMARY_BIOME: &str = "grassland";
const OCTAVES: usize = 6;
const LACUNARITY: f64 = 2.0;
const PERSISTENCE: f64 = 0.5;
const BASE_AMPLITUDE: f32 = 50.0;

/// The live editor's world: chunk_radius 10 (441 chunks), the default the
/// director's frames were made against (`terrain_panel.rs` default).
const RADIUS: i32 = 10;

/// T.2a's pinned desert focal — a known point on the desert surface (ground
/// height 36.3 m, independently confirmed by the §10.3.1 one-parameter fit
/// coming out at 36.0 m).
const DESERT_FOCAL: [f32; 3] = [43.1, 36.3, -1961.8];

/// The recovered director camera (T2D_CAMERA_LIGHT.md §10.2/§10.8): a pure
/// orbit-zoom moves the eye along (cos y·cos p, sin p, sin y·cos p), and the
/// two Camera readouts differ along exactly one such vector — pitch 45.6°,
/// yaw 45.5°, focal on the desert ground plane.
const REC_FOCAL: [f32; 3] = [-1029.9, 36.3, 254.7];
const REC_YAW_DEG: f32 = 45.5;
const REC_PITCH_DEG: f32 = 45.6;

struct Station {
    name: &'static str,
    focal: [f32; 3],
    distance: f32,
    yaw_deg: f32,
    pitch_deg: f32,
    width: u32,
    height: u32,
    /// The eye this station must reproduce, when it is a recovered camera
    /// (asserted to 0.5 m); None for stations defined by their orbit directly.
    expect_eye: Option<[f32; 3]>,
    why: &'static str,
}

/// The five T.2d.F stations.
///
/// Boundary pair at the director's exact eyes (962×501 = the measured live
/// viewport rect, aspect 1.920); one close-range station (< 50 m — LOD 0 is
/// where the §3.2 multiscatter step lives); one oblique mid station whose
/// frame CONTAINS the LOD0|1 footprint-0.5 contour (~100 m out at 30 m
/// altitude); one high oblique for the far half of the 5 m → 1500 m profile.
fn stations() -> Vec<Station> {
    vec![
        Station {
            name: "t2df_boundary_y414",
            focal: REC_FOCAL,
            distance: 529.6,
            yaw_deg: REC_YAW_DEG,
            pitch_deg: REC_PITCH_DEG,
            width: 962,
            height: 501,
            expect_eye: Some([-770.1, 414.5, 518.9]),
            why: "director frame 1 — LOD1|2 boundary at footprint 2.000",
        },
        Station {
            name: "t2df_boundary_y536",
            focal: REC_FOCAL,
            distance: 700.0,
            yaw_deg: REC_YAW_DEG,
            pitch_deg: REC_PITCH_DEG,
            width: 962,
            height: 501,
            expect_eye: Some([-686.5, 536.2, 603.9]),
            why: "director frame 2 — LOD1|2 boundary at footprint 1.984",
        },
        Station {
            name: "t2df_desert_close_20m",
            focal: DESERT_FOCAL,
            distance: 20.0,
            yaw_deg: 45.0,
            pitch_deg: 55.0,
            width: 1024,
            height: 768,
            expect_eye: None,
            why: "close range (T.2a station geometry) — LOD 0 / multiscatter territory",
        },
        Station {
            name: "t2df_lod01_contour",
            focal: DESERT_FOCAL,
            distance: 46.7, // altitude = d·sin(40°) ≈ 30 m
            yaw_deg: 45.0,
            pitch_deg: 40.0,
            width: 1024,
            height: 768,
            expect_eye: None,
            why: "oblique 30 m altitude — the LOD0|1 footprint-0.5 contour (~100 m) is IN frame",
        },
        Station {
            name: "t2df_profile_far",
            focal: DESERT_FOCAL,
            distance: 523.1, // altitude = d·sin(35°) ≈ 300 m
            yaw_deg: 45.0,
            pitch_deg: 35.0,
            width: 1024,
            height: 768,
            expect_eye: None,
            why: "high oblique — ground 140 m → 3400 m for the far half of the distance profile",
        },
    ]
}

fn out_root() -> PathBuf {
    PathBuf::from(std::env::var("T2DF_OUT").unwrap_or_else(|_| "d:/tmp/t2df_staging".to_string()))
}

fn label() -> String {
    std::env::var("T2DF_LABEL").unwrap_or_else(|_| "unlabelled".to_string())
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

fn generate_desert() -> Result<TerrainState> {
    let mut state = TerrainState::new();
    state.configure(SEED, PRIMARY_BIOME);
    state.set_noise_params(OCTAVES, LACUNARITY, PERSISTENCE, BASE_AMPLITUDE);
    state.set_world_archetype(WorldArchetypeId::Desert.default_archetype());
    let n = state
        .generate_terrain(RADIUS)
        .context("generate_terrain failed")?;
    println!("[t2df] desert (radius {RADIUS}) generated {n} chunks");
    Ok(state)
}

async fn acquire_device(
    want_timestamps: bool,
) -> Result<(Arc<wgpu::Device>, Arc<wgpu::Queue>, wgpu::AdapterInfo)> {
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
        .context("no suitable wgpu adapter for the T.2d.F harness")?;
    let info = adapter.get_info();
    let mut features = wgpu::Features::empty();
    if want_timestamps && adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        features |= wgpu::Features::TIMESTAMP_QUERY;
    }
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("t2df device"),
            required_features: features,
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

/// Build the station's camera through the ED-2 state machinery: construct the
/// orbit, `capture_state()` it, then `apply_state` onto a FRESH camera — so a
/// capture is proof that the pinned `CameraState` alone reproduces the frame
/// (the same contract the director's Camera → Go uses).
fn station_camera(s: &Station) -> (OrbitCamera, CameraState) {
    let mut authored = OrbitCamera::new(
        Vec3::from_array(s.focal),
        s.distance,
        s.yaw_deg.to_radians(),
        s.pitch_deg.to_radians(),
    );
    authored.set_aspect(s.width as f32, s.height as f32);
    let state = authored.capture_state();

    let mut cam = OrbitCamera::default();
    cam.set_aspect(s.width as f32, s.height as f32);
    cam.apply_state(&state);
    if let Some(expect) = s.expect_eye {
        let placed = cam.position();
        let err = (placed - Vec3::from_array(expect)).length();
        assert!(
            err < 0.5,
            "station {} misses the recovered eye by {err:.2} m ({placed:?} vs {expect:?})",
            s.name
        );
    }
    (cam, state)
}

#[test]
#[ignore = "GPU + radius-10 terrain generation; run explicitly (see module docs)"]
fn t2df_capture_stations() -> Result<()> {
    pollster::block_on(async {
        let leg = label();
        let out_dir = out_root().join(&leg);
        std::fs::create_dir_all(&out_dir)?;

        let assets = find_assets_dir().context("assets dir not found")?;
        let biome_dir = assets.join("materials").join("biomes");

        let (device, queue, info) = acquire_device(false).await?;
        println!(
            "[t2df] adapter: {} · {:?} · {:?} · driver {}",
            info.name, info.backend, info.device_type, info.driver_info
        );

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
        let state = generate_desert()?;
        viewport.upload_terrain_chunks_raw(&state.get_gpu_chunks());

        let world = World::new();
        // One render texture PER SIZE, reused across stations. The renderer
        // rebinds its output only on a size change, so a fresh same-size
        // texture object comes back black (the live widget never hits this —
        // it always passes the same object; ed2_capture.rs and Experiment E
        // reuse theirs the same way).
        let mut textures: std::collections::HashMap<(u32, u32), wgpu::Texture> =
            std::collections::HashMap::new();
        for s in stations() {
            let texture = match textures.entry((s.width, s.height)) {
                std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
                std::collections::hash_map::Entry::Vacant(v) => v.insert(
                    viewport
                        .create_render_texture(s.width, s.height)
                        .context("create_render_texture failed")?,
                ),
            };
            let (cam, cam_state) = station_camera(&s);
            for _ in 0..2 {
                viewport
                    .render(texture, &cam, &world, None, None, None, false, false, 0)
                    .with_context(|| format!("render failed at {}", s.name))?;
            }
            let png = out_dir.join(format!("{}.png", s.name));
            let (w, h) = viewport
                .capture_frame_png(texture, &png)
                .with_context(|| format!("capture failed at {}", s.name))?;
            // The ED-2 sidecar: the full CameraState beside the PNG.
            let sidecar = png.with_extension("camera.json");
            std::fs::write(&sidecar, serde_json::to_string_pretty(&cam_state)?)?;
            println!(
                "[t2df] {} {}x{} -> {}   ({})",
                s.name,
                w,
                h,
                png.display(),
                s.why
            );
        }
        println!("[t2df] leg '{leg}' -> {}", out_dir.display());
        Ok(())
    })
}

/// Emit the five stations as ED-2 `CameraStation` JSON, for merging into
/// `.editor_preferences.json` (`camera_stations`) so the director's editor has
/// them under Camera → Go.
#[test]
fn t2df_stations_serialize_for_preferences() -> Result<()> {
    let list: Vec<CameraStation> = stations()
        .iter()
        .map(|s| {
            let (_, state) = station_camera(s);
            CameraStation {
                name: s.name.to_string(),
                state,
            }
        })
        .collect();
    let json = serde_json::to_string_pretty(&list)?;
    // Round-trip proof, same contract as the ED-2 preferences tests.
    let back: Vec<CameraStation> = serde_json::from_str(&json)?;
    assert_eq!(back.len(), 5);
    assert_eq!(back[0].name, "t2df_boundary_y414");
    let out = out_root();
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("t2df_stations.json"), &json)?;
    println!(
        "[t2df] station JSON -> {}",
        out.join("t2df_stations.json").display()
    );
    Ok(())
}

#[test]
#[ignore = "GPU + radius-10 terrain generation; run explicitly (see module docs)"]
fn t2df_perf_main_pass() -> Result<()> {
    pollster::block_on(async {
        let leg = label();
        // TIMESTAMP_QUERY is opt-in (T2DF_TS=1): the first run of this test
        // hung on this driver with timestamps requested, before terrain
        // generation finished — wall-clock timing is the dependable default.
        let want_ts = std::env::var("T2DF_TS").is_ok_and(|v| v == "1");
        let (device, queue, info) = acquire_device(want_ts).await?;
        println!(
            "[t2df-perf] adapter: {} · {:?} · {:?} · driver {} · timestamps {}",
            info.name,
            info.backend,
            info.device_type,
            info.driver_info,
            if device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
                "ON"
            } else {
                "off"
            }
        );

        let assets = find_assets_dir().context("assets dir not found")?;
        let biome_dir = assets.join("materials").join("biomes");
        let mut viewport = ViewportRenderer::new(device.clone(), queue.clone())?;
        viewport.init_engine_adapter().await?;
        {
            let adapter = viewport
                .engine_adapter_mut()
                .context("engine adapter missing")?;
            adapter.set_time_of_day(TIME_OF_DAY);
            adapter.set_biome_pack(Some(biome_dir));
        }
        println!("[t2df-perf] engine adapter ready; generating terrain…");
        let state = generate_desert()?;
        viewport.upload_terrain_chunks_raw(&state.get_gpu_chunks());
        println!("[t2df-perf] terrain uploaded; entering render loop");

        // Min-spec framing: the y414 boundary station at 1920×1080 — the
        // distance-heaviest configuration (75.3% of terrain pixels were LOD 2
        // here, §10.3.3), i.e. where the tiers previously saved the most.
        let (width, height) = (1920u32, 1080u32);
        let texture = viewport.create_render_texture(width, height)?;
        let s = &stations()[0];
        let mut cam = OrbitCamera::new(
            Vec3::from_array(s.focal),
            s.distance,
            s.yaw_deg.to_radians(),
            s.pitch_deg.to_radians(),
        );
        cam.set_aspect(width as f32, height as f32);
        let world = World::new();

        const WARMUP: usize = 60;
        const TIMED: usize = 300;
        let mut wall_ms: Vec<f64> = Vec::with_capacity(TIMED);
        let mut main_ms: Vec<f64> = Vec::new();
        let mut total_gpu_ms: Vec<f64> = Vec::new();

        for i in 0..(WARMUP + TIMED) {
            let t0 = std::time::Instant::now();
            viewport
                .render(&texture, &cam, &world, None, None, None, false, false, 0)
                .context("render failed")?;
            device
                .poll(wgpu::MaintainBase::Wait)
                .ok()
                .context("device poll failed")?;
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            if i % 60 == 0 {
                println!("[t2df-perf] frame {i}/{} ({dt:.1} ms)", WARMUP + TIMED);
            }
            if i >= WARMUP {
                wall_ms.push(dt);
                if let Some(renderer) = viewport.engine_adapter_mut().map(|a| a.renderer()) {
                    if let Some(p) = renderer.gpu_profiler() {
                        let map = p.results_map();
                        if let Some(v) = map.get("main_pass") {
                            main_ms.push(*v as f64);
                        }
                        total_gpu_ms.push(p.total_gpu_ms() as f64);
                    }
                }
            }
        }

        fn stats(mut v: Vec<f64>) -> Option<(f64, f64, f64, usize)> {
            if v.is_empty() {
                return None;
            }
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let n = v.len();
            Some((v[n / 2], v[n / 10], v[n - 1 - n / 10], n))
        }

        println!("[t2df-perf] leg '{leg}' at {width}x{height}, {TIMED} timed frames after {WARMUP} warm-up:");
        if let Some((med, p10, p90, n)) = stats(wall_ms) {
            println!("[t2df-perf]   frame wall (render+GPU wait): median {med:.3} ms  p10 {p10:.3}  p90 {p90:.3}  (n={n})");
        }
        match stats(main_ms) {
            Some((med, p10, p90, n)) => println!(
                "[t2df-perf]   GPU main_pass (terrain lives here): median {med:.3} ms  p10 {p10:.3}  p90 {p90:.3}  (n={n})"
            ),
            None => println!("[t2df-perf]   GPU main_pass: TIMESTAMP_QUERY unavailable on this adapter"),
        }
        if let Some((med, p10, p90, n)) = stats(total_gpu_ms) {
            println!("[t2df-perf]   GPU total (all timestamped passes): median {med:.3} ms  p10 {p10:.3}  p90 {p90:.3}  (n={n})");
        }
        Ok(())
    })
}
