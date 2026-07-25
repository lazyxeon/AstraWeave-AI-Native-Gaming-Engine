//! T.2a terrain surface-quality A/B station harness.
//!
//! The beat's method spine requires *pinned* camera stations: every knob
//! change gets isolated before/after evidence from byte-identical viewpoints,
//! so the director can attribute an improvement to a specific knob rather
//! than to a blended "it looks better now".
//!
//! **The editor cannot provide that.** `OrbitCamera` has no numeric
//! position entry, no console command, and no persisted bookmark; the F1-F12
//! in-memory bookmarks are lost on restart (and `set_yaw`/`set_pitch` do not
//! update the smoothing targets, so a restore drifts back within ~50 ms); and
//! `PanelEvent::TerrainReady` calls `frame_terrain` on *every* generation,
//! which overwrites focal point, distance and pitch. Since each shader-constant
//! A/B leg requires a rebuild + restart (the WGSL is `include_str!`-compiled;
//! `ShaderManager`'s disk path has zero production callers), an editor-driven
//! station is not reproducible across legs.
//!
//! This harness renders the **same** editor viewport path offscreen with an
//! exactly-specified camera, so the only thing that differs between two legs
//! is the knob under test.
//!
//! ## What it renders
//!
//! Real generated terrain — `TerrainState::generate_terrain` with the
//! director's configuration (seed 12345, radius 6, the editor's noise
//! defaults) and the live 8-slot biomes pack — uploaded through
//! `EngineRenderAdapter::upload_terrain_chunks` and drawn by
//! `ViewportRenderer::render`. Readback is `engine_ldr_texture()`, the
//! overlay-free target the parity harness hashes, so no grid or gizmo
//! contaminates the comparison.
//!
//! Per-chunk generation is deterministic in `(world_seed, chunk_id)` and
//! rayon order-independent (`terrain_integration.rs:340-352`), so a station's
//! content is stable across runs. `smooth_shared_vertices` averages heights
//! only at shared edges, so stations are kept well inside the radius.
//!
//! ## Modes
//!
//! - `T2A_MODE=survey` — generate each world and report, per material slot,
//!   the vertex count and a representative interior world position. This is
//!   how the pinned station coordinates below were chosen; re-run it if the
//!   biome classification changes (Phase 3) and the stations stop framing
//!   their intended material.
//! - default — render every station to
//!   `$T2A_OUT/$T2A_LABEL/<station>.png` and append one row per station to
//!   `$T2A_OUT/$T2A_LABEL/metrics.csv`.
//!
//! Metrics are the quantitative half of the A/B: mean/stddev luminance plus
//! mean |Laplacian| (high-frequency energy). "Harsh up close" is a
//! high-frequency percept, so the Laplacian column is the one that moves when
//! `NORMAL_XY_STRENGTH` or the hex-tile sharpening changes.
//!
//! Ignored by default (needs a GPU and generates real terrain). Run:
//!
//! ```text
//! cargo test -p aw_editor --release --test terrain_ab_stations -- --ignored --nocapture
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

/// Noon — matches the editor's own default and removes time-of-day as a
/// variable between legs.
const TIME_OF_DAY: f32 = 12.0;

/// The director's configuration (terrain_panel.rs defaults + the T-series
/// standing seed/radius). `radius 6` is what every T.W observation pass used.
const SEED: u64 = 12345;
const RADIUS: i32 = 6;
const PRIMARY_BIOME: &str = "grassland";
const OCTAVES: usize = 6;
const LACUNARITY: f64 = 2.0;
const PERSISTENCE: f64 = 0.5;
const BASE_AMPLITUDE: f32 = 50.0;

/// A generated world. Stations are grouped by world so terrain generation
/// (the expensive step) happens once per archetype, not once per station.
struct WorldSpec {
    key: &'static str,
    archetype: WorldArchetypeId,
}

const WORLDS: &[WorldSpec] = &[
    WorldSpec {
        key: "med",
        archetype: WorldArchetypeId::Mediterranean,
    },
    WorldSpec {
        key: "ct",
        archetype: WorldArchetypeId::ContinentalTemperate,
    },
    WorldSpec {
        key: "desert",
        archetype: WorldArchetypeId::Desert,
    },
    WorldSpec {
        key: "boreal",
        archetype: WorldArchetypeId::BorealSubarctic,
    },
];

/// A pinned camera. `focal` is a world-space point on the terrain surface,
/// `distance` the orbit radius, `yaw`/`pitch` in degrees.
///
/// Close-up stations sit at 20 units with a 55 deg look-down, which fills the
/// frame with ground: at fovy 60 deg the top of the frame is still 25 deg below
/// the horizon, so no sky is wasted and one texture repeat (4 m at tiling 128)
/// spans a meaningful fraction of the image — the range where the director's
/// "eyes hurt" percept lives. A shallower pitch spends most of the frame on
/// sky and fog. The overview station is far enough to judge biome composition
/// rather than surface detail.
struct Station {
    name: &'static str,
    world: &'static str,
    focal: [f32; 3],
    distance: f32,
    yaw_deg: f32,
    pitch_deg: f32,
}

/// Pinned stations. Coordinates were chosen from a `T2A_MODE=survey` run at
/// commit `89fbe97eb` (the T.2a baseline) so each close-up frames the
/// material named in its station name. Do not "tidy" these numbers — they
/// are the beat's A/B anchors and every captured leg is registered to them.
/// Two of the eight slots are unreachable as stations: **swamp (5) and river
/// (7) have a dominant-vertex count of exactly 0 in all four archetypes** at
/// seed 12345 / radius 6 (survey, 2026-07-24). A swamp close-up — which the
/// beat's station list asks for — cannot be framed on this world; noted in the
/// outcome rather than faked from a blended vertex.
///
/// Station 02 is the deliberate **control**: `tree_leaves` is the healthiest
/// material in the pack (AO sd 53.0, roughness sd 10.8, no placeholder
/// channel). A Phase-1 data fix must move stations 01 and 03 and leave 02
/// essentially untouched — that contrast is what makes the change attributable
/// to the data rather than to anything else drifting.
const STATIONS: &[Station] = &[
    // slot 0 grassland — the dominant biome AND the worst material in the
    // pack (roughness 99.5% flat, AO a hard constant).
    Station {
        name: "01_med_grassland_closeup",
        world: "med",
        focal: [0.0, 12.6, 2037.2],
        distance: 20.0,
        yaw_deg: 45.0,
        pitch_deg: 55.0,
    },
    // slot 2 forest — CONTROL (real measured channels).
    Station {
        name: "02_ct_forest_closeup",
        world: "ct",
        focal: [10.8, 26.6, 964.7],
        distance: 20.0,
        yaw_deg: 45.0,
        pitch_deg: 55.0,
    },
    // slot 3 mountain — flat AO (constant 140); tiling 64, the only slot
    // that differs, so it also probes the hex-tile knob at a second scale.
    Station {
        name: "03_med_mountain_closeup",
        world: "med",
        focal: [-10.8, 412.8, -441.9],
        distance: 30.0,
        yaw_deg: 30.0,
        pitch_deg: 40.0,
    },
    // slot 1 desert — 100% procedural source, near-flat normal map. Second
    // control for the data fix; primary probe for the normal-strength knob.
    Station {
        name: "04_desert_sand_closeup",
        world: "desert",
        focal: [43.1, 36.3, -1961.8],
        distance: 20.0,
        yaw_deg: 45.0,
        pitch_deg: 55.0,
    },
    // slot 4 tundra — also procedural; the Boreal world's ~93% surface.
    Station {
        name: "05_boreal_tundra_closeup",
        world: "boreal",
        focal: [177.9, 34.5, 544.3],
        distance: 20.0,
        yaw_deg: 45.0,
        pitch_deg: 55.0,
    },
    // Q5 judgement frame: is forest threading through the snow?
    Station {
        name: "06_boreal_overview",
        world: "boreal",
        focal: [0.0, 120.0, 0.0],
        distance: 2600.0,
        yaw_deg: 45.0,
        pitch_deg: 32.0,
    },
    // T.2c (2026-07-25) — slot 4 tundra, RE-PINNED.
    //
    // Station 05 above was pinned from the survey at `89fbe97eb`, when the
    // Boreal world was 93.3% tundra. T.2a's own Phase 3 then widened the boreal
    // band (`TUNDRA_MAX_TEMP_C` 0.0 -> -5.0), taking that world to 40.9% forest
    // / 52.4% tundra — and the ground under station 05's pinned focal point
    // reclassified to forest. Measured, not inferred: across the T.2c material
    // swap, station 05's frame is BYTE-IDENTICAL in both legs while the boreal
    // overview moves (lap 11.83 -> 19.38), which is only possible if 05 frames
    // no slot-4 texels at all.
    //
    // Station 05 is deliberately LEFT IN PLACE — it is a T.2a A/B anchor and
    // re-aiming it would silently invalidate the frames already registered to
    // it. This station is the slot-4 probe instead, aimed at the CURRENT tundra
    // representative vertex (weight >= 0.85, median-x) from a `T2A_MODE=survey`
    // run at T.2c HEAD.
    Station {
        name: "07_boreal_tundra_t2c",
        world: "boreal",
        focal: [-43.1, 48.4, -1627.6],
        distance: 20.0,
        yaw_deg: 45.0,
        pitch_deg: 55.0,
    },
];

fn out_root() -> PathBuf {
    PathBuf::from(
        std::env::var("T2A_OUT").unwrap_or_else(|_| "d:/tmp/t2a_staging/render".to_string()),
    )
}

fn label() -> String {
    std::env::var("T2A_LABEL").unwrap_or_else(|_| "unlabelled".to_string())
}

fn find_assets_dir() -> Option<PathBuf> {
    // Integration tests run with CWD = crate root (tools/aw_editor); the
    // editor runs from the workspace root. Accept both.
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

/// Build the world exactly as `TerrainPanel::regenerate_terrain` does.
fn generate_world(spec: &WorldSpec) -> Result<TerrainState> {
    let mut state = TerrainState::new();
    state.configure(SEED, PRIMARY_BIOME);
    state.set_noise_params(OCTAVES, LACUNARITY, PERSISTENCE, BASE_AMPLITUDE);
    state.set_world_archetype(spec.archetype.default_archetype());
    let n = state
        .generate_terrain(RADIUS)
        .with_context(|| format!("generate_terrain failed for world '{}'", spec.key))?;
    println!("[t2a] world '{}' generated {n} chunks", spec.key);
    Ok(state)
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
        .context("no suitable wgpu adapter for the T.2a station harness")?;
    let info = adapter.get_info();
    // Renderer::new requires max_bind_groups: 8 (renderer.rs).
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("t2a-station-harness device"),
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

/// Copy `tex` mip 0 into CPU memory, de-padding rows.
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
        label: Some("t2a-readback-staging"),
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

/// Mean luminance, stddev luminance, and mean |Laplacian| over the frame.
///
/// The Laplacian term is the beat's high-frequency proxy: the "harsh /
/// eyes hurt" percept is excess local contrast, so a knob that calms the
/// surface lowers this number while leaving mean luminance roughly alone.
fn frame_metrics(rgba: &[u8], width: u32, height: u32) -> (f64, f64, f64) {
    let w = width as usize;
    let h = height as usize;
    let luma: Vec<f64> = rgba
        .chunks_exact(4)
        .map(|p| 0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64)
        .collect();
    let n = luma.len() as f64;
    let mean = luma.iter().sum::<f64>() / n;
    let var = luma.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / n;

    let mut lap_sum = 0.0;
    let mut lap_n = 0.0;
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let l = 4.0 * luma[i] - luma[i - 1] - luma[i + 1] - luma[i - w] - luma[i + w];
            lap_sum += l.abs();
            lap_n += 1.0;
        }
    }
    (mean, var.sqrt(), lap_sum / lap_n)
}

const SLOT_NAMES: [&str; 8] = [
    "grassland",
    "desert",
    "forest",
    "mountain",
    "tundra",
    "swamp",
    "beach",
    "river",
];

/// Per-archetype biome census on the **real generation path**, plus a
/// representative interior position per slot.
///
/// This is the beat's census instrument. The only pre-existing "census" in the
/// repo (`astraweave-terrain/tests/tw1_dip_census.rs`) counts *water* against
/// the sea level, not biome coverage, and the 10K-sample `distribution_for`
/// helper in `world_archetypes.rs` is inside a `#[cfg(test)] mod tests` (so
/// unreachable from an integration test) and samples climate space rather than
/// generated terrain — it never sees the elevation overlays or the coastal
/// gate. Counting dominant material slots over the generated vertices measures
/// what the director actually looks at.
///
/// Two numbers per slot:
/// * **census %** — share of all interior vertices whose dominant slot is this
///   one. This is the before/after figure for a classification change.
/// * **rep** — the median-x position among *strongly* dominated vertices
///   (weight >= 0.85), used to pin a station. A station framed on a 50/50
///   blend does not show the material it is named after.
fn survey(state: &TerrainState, key: &str) {
    // Stay inside the generated radius so `smooth_shared_vertices` edge
    // averaging and the outer chunk ring cannot influence a station.
    let inner = (RADIUS - 2) as f32 * 512.0;
    let mut per_slot: Vec<Vec<[f32; 3]>> = vec![Vec::new(); 8];
    let mut census = [0usize; 8];
    let mut total = 0usize;
    for (verts, _) in state.get_gpu_chunks() {
        for v in verts {
            if v.position[0].abs() > inner || v.position[2].abs() > inner {
                continue;
            }
            // Dominant slot = highest weight among the 4 authored pairs.
            let mut best = 0usize;
            let mut best_w = -1.0f32;
            for i in 0..4 {
                if v.material_weights[i] > best_w {
                    best_w = v.material_weights[i];
                    best = i;
                }
            }
            let slot = v.material_ids[best] as usize;
            if slot >= 8 {
                continue;
            }
            total += 1;
            census[slot] += 1;
            if best_w >= 0.85 {
                per_slot[slot].push(v.position);
            }
        }
    }
    println!(
        "[t2a] --- census + slot survey: world '{key}' (|x|,|z| <= {inner:.0}, n={total}) ---"
    );
    for (slot, name) in SLOT_NAMES.iter().enumerate() {
        let pct = if total == 0 {
            0.0
        } else {
            census[slot] as f64 * 100.0 / total as f64
        };
        let pts = &mut per_slot[slot];
        if pts.is_empty() {
            println!(
                "[t2a]   slot {slot} {name:<10} census={pct:6.3}% ({:>7})  dominant=0",
                census[slot]
            );
            continue;
        }
        let count = pts.len();
        // Median-of-x then the point at that index: a robust interior
        // representative, immune to the outliers a centroid would chase.
        pts.sort_by(|a, b| a[0].total_cmp(&b[0]));
        let m = pts[count / 2];
        println!(
            "[t2a]   slot {slot} {name:<10} census={pct:6.3}% ({:>7})  dominant={count:<8} rep=[{:.1}, {:.1}, {:.1}]",
            census[slot], m[0], m[1], m[2]
        );
    }
}

async fn run() -> Result<()> {
    let mode = std::env::var("T2A_MODE").unwrap_or_default();
    let survey_only = mode == "survey";

    let assets = find_assets_dir();
    if assets.is_none() {
        anyhow::bail!("could not locate assets/materials/biomes from CWD");
    }
    let biome_dir = assets.unwrap().join("materials").join("biomes");

    let (device, queue, info) = if survey_only {
        // Survey needs no GPU, but keeping one code path is simpler than
        // making the device optional; skip acquisition instead.
        (None, None, None)
    } else {
        let (d, q, i) = acquire_device().await?;
        (Some(d), Some(q), Some(i))
    };
    if let Some(i) = &info {
        println!(
            "[t2a] adapter: {:?} {} ({:?})",
            i.backend, i.name, i.device_type
        );
    }

    let out_dir = out_root().join(label());
    if !survey_only {
        std::fs::create_dir_all(&out_dir)
            .with_context(|| format!("create_dir_all {}", out_dir.display()))?;
    }
    let mut csv = String::from("station,world,mean_luma,std_luma,mean_abs_laplacian\n");

    for spec in WORLDS {
        let stations: Vec<&Station> = STATIONS.iter().filter(|s| s.world == spec.key).collect();
        if stations.is_empty() {
            continue;
        }
        let t0 = std::time::Instant::now();
        let state = generate_world(spec)?;
        println!("[t2a] world '{}' gen took {:?}", spec.key, t0.elapsed());

        // The census is cheap (CPU only) and is the Phase-3 before/after
        // evidence, so print it on every run, not just in survey mode.
        survey(&state, spec.key);
        if survey_only {
            continue;
        }

        let device = device.clone().expect("device present outside survey mode");
        let queue = queue.clone().expect("queue present outside survey mode");

        let mut viewport = ViewportRenderer::new(device.clone(), queue.clone())
            .context("ViewportRenderer::new failed")?;
        viewport
            .init_engine_adapter()
            .await
            .context("init_engine_adapter failed")?;
        {
            let adapter = viewport
                .engine_adapter_mut()
                .context("engine adapter missing after init")?;
            adapter.set_time_of_day(TIME_OF_DAY);
            // Same call the editor makes at main.rs:5532 — the pack is staged
            // and applied by the upload below.
            adapter.set_biome_pack(Some(biome_dir.clone()));
        }
        // The editor's own production upload path (main.rs:5356 on
        // `PanelEvent::TerrainReady`). Note `terrain_integration::TerrainVertex`
        // and `viewport::types::TerrainVertex` are two distinct mirrored
        // structs; `_raw` is the one that takes the generator's output
        // directly, so the harness performs no conversion the editor doesn't.
        viewport.upload_terrain_chunks_raw(&state.get_gpu_chunks());

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("t2a-station-display-target"),
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

        for st in stations {
            let mut camera = OrbitCamera::new(
                Vec3::from_array(st.focal),
                st.distance,
                st.yaw_deg.to_radians(),
                st.pitch_deg.to_radians(),
            );
            camera.set_aspect(WIDTH as f32, HEIGHT as f32);

            // Two-frame settle: frame 1 does the lazy resize (depth +
            // ENGINE_LDR_TARGET + EDITOR_OVERLAY_TARGET + composite pipeline)
            // and the adapter's first-frame clustered-lights / IBL bake.
            // Measurement is frame 2 — same contract the parity harness uses.
            for _ in 0..2 {
                viewport
                    .render(&target, &camera, &world, None, None, None, false, false, 0)
                    .with_context(|| format!("render failed at station {}", st.name))?;
            }

            let ldr = viewport
                .engine_ldr_texture()
                .context("engine_ldr_texture unavailable (overlay-free readback target)")?;
            let enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("t2a-readback"),
            });
            let bytes = readback_texture(&device, &queue, ldr, WIDTH, HEIGHT, enc)?;

            let (mean, std, lap) = frame_metrics(&bytes, WIDTH, HEIGHT);
            let png = out_dir.join(format!("{}.png", st.name));
            let img = image::RgbaImage::from_raw(WIDTH, HEIGHT, bytes)
                .context("RgbaImage::from_raw size mismatch")?;
            img.save(&png)
                .with_context(|| format!("saving {}", png.display()))?;
            println!(
                "[t2a] {:<26} mean={mean:7.3} std={std:7.3} lap={lap:7.4}  -> {}",
                st.name,
                png.display()
            );
            csv.push_str(&format!(
                "{},{},{mean:.4},{std:.4},{lap:.5}\n",
                st.name, st.world
            ));
        }
    }

    if !survey_only {
        let csv_path = out_dir.join("metrics.csv");
        std::fs::write(&csv_path, csv)
            .with_context(|| format!("writing {}", csv_path.display()))?;
        println!("[t2a] metrics -> {}", csv_path.display());
    }
    Ok(())
}

#[test]
#[ignore = "needs a GPU and generates real terrain; run explicitly for T.2a A/B legs"]
fn t2a_render_ab_stations() {
    pollster::block_on(run()).expect("T.2a station harness failed");
}
