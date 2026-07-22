//! TW1 dip census (T.W.1 work item 7 — measurement feeding the director's
//! final §2.3 ruling in `docs/audits/TWR_WATER_RECON.md`).
//!
//! Question: with the water plane at `SEA_LEVEL` (Y=2.0), how much terrain
//! floods, and how much of that flooding is HONEST coastal water (the
//! per-vertex `BiomeId` says Ocean/Coast) versus EMERGENT ponds/oases (the
//! id says terrestrial/beach — the class the director's §2.3 ruling covers)?
//!
//! The partition is by the chunk's own biome ids, NOT by re-sampling
//! continentalness: classification runs through the D.4 scattered-convolution
//! (jittered multi-sample at provisional elevation,
//! `biome_param_blending.rs:195-209`), so an independent point-sample of
//! `ClimateMap::continentalness` cannot reproduce the classifier's effective
//! value near the 0.40 gate — an early census draft tried and mis-bucketed
//! every flooded coastal vertex as inland. Biome ids are the direct
//! semantics.
//!
//! Mirrors the editor's generation faithfully: `WorldConfig::default()`
//! (seed 12345, the panel default), `config.climate.archetype` from the
//! archetype's `default_archetype()`, `generate_chunk_with_climate(id,
//! ClimateBias::Temperate)` per chunk (the editor derives Temperate from its
//! default `primary_biome = "grassland"`), chunk radius 6 (169 chunks — the
//! T-series director-repro world). Two known divergences from the editor
//! path, both height-neutral at census scale: the editor's shared-edge
//! height averaging (sub-meter, shared-edge vertices only) and its
//! legacy-`biome_map` primary override (does not touch heights or climate).
//!
//! Run explicitly (release strongly recommended):
//! `cargo test -p astraweave-terrain --release --test tw1_dip_census -- --ignored --nocapture`

use astraweave_terrain::world_archetypes::WorldArchetypeId;
use astraweave_terrain::{ChunkId, ClimateBias, WorldConfig, WorldGenerator, SEA_LEVEL};
use rayon::prelude::*;

const CHUNK_RADIUS: i32 = 6;

struct CensusRow {
    archetype: &'static str,
    total_vertices: usize,
    below_sea_total: usize,
    /// Below-sea vertices whose `BiomeId` is terrestrial/beach — the plane
    /// floods where the world says land: the emergent-pond/oasis class.
    below_sea_emergent: usize,
    /// Below-sea vertices whose `BiomeId` is Ocean/Coast — honest coastal water.
    below_sea_aquatic: usize,
    /// Vertices whose per-vertex `BiomeId` (classified on PROVISIONAL
    /// pre-erosion heights — the E3 seam-fix design) is Ocean or Coast.
    classified_aquatic: usize,
    /// Classified-aquatic vertices whose FINAL rendered height is below sea
    /// level — i.e., the classified water that actually floods at Y=2.0.
    classified_aquatic_flooded: usize,
    /// Min / median final height over classified-aquatic vertices (empty-safe).
    aquatic_final_min: f32,
    aquatic_final_median: f32,
}

fn census_for(archetype_id: WorldArchetypeId, label: &'static str) -> CensusRow {
    let mut config = WorldConfig::default(); // seed 12345 — the panel default
    assert_eq!(config.seed, 12345, "WorldConfig::default seed drifted");
    // Mirror the editor's `regenerate_terrain` exactly: the panel routes its
    // noise defaults through `set_noise_params(6, 2.0, 0.5, 50.0)`, which
    // overrides `NoiseConfig::default()`'s base_elevation (amplitude 100,
    // octaves 4). Without this the census world is NOT the editor world.
    config.noise.base_elevation.octaves = 6;
    config.noise.base_elevation.lacunarity = 2.0;
    config.noise.base_elevation.persistence = 0.5;
    config.noise.base_elevation.amplitude = 50.0;
    config.climate.archetype = archetype_id.default_archetype();

    let generator = WorldGenerator::new(config.clone());
    let _chunk_size = config.chunk_size;

    let chunk_ids: Vec<ChunkId> = (-CHUNK_RADIUS..=CHUNK_RADIUS)
        .flat_map(|x| (-CHUNK_RADIUS..=CHUNK_RADIUS).map(move |z| ChunkId { x, z }))
        .collect();

    type Partial = (usize, usize, usize, usize, usize, usize, Vec<f32>);
    let partials: Vec<Partial> = chunk_ids
        .into_par_iter()
        .map(|chunk_id| {
            let chunk = generator
                .generate_chunk_with_climate(chunk_id, ClimateBias::Temperate)
                .expect("chunk generation failed");
            let hm = chunk.heightmap();
            let res = hm.resolution();
            let biome_ids = chunk
                .biome_ids()
                .expect("climate path must produce per-vertex biome ids");

            let mut total = 0usize;
            let mut below = 0usize;
            let mut below_emergent = 0usize;
            let mut below_aquatic = 0usize;
            let mut aquatic = 0usize;
            let mut aquatic_flooded = 0usize;
            let mut aquatic_heights: Vec<f32> = Vec::new();
            for gz in 0..res {
                for gx in 0..res {
                    let idx = (gz * res + gx) as usize;
                    total += 1;
                    let y = hm.get_height(gx, gz);
                    let is_aquatic_id = matches!(
                        biome_ids[idx],
                        astraweave_terrain::biome_lookup::BiomeId::Ocean
                            | astraweave_terrain::biome_lookup::BiomeId::Coast
                    );
                    // Classification-vs-final-height divergence probe: biome
                    // ids were assigned on PROVISIONAL pre-erosion heights;
                    // `y` is the FINAL rendered height.
                    if is_aquatic_id {
                        aquatic += 1;
                        aquatic_heights.push(y);
                        if y < SEA_LEVEL {
                            aquatic_flooded += 1;
                        }
                    }
                    if y >= SEA_LEVEL {
                        continue;
                    }
                    below += 1;
                    if is_aquatic_id {
                        below_aquatic += 1;
                    } else {
                        below_emergent += 1;
                    }
                }
            }
            (
                total,
                below,
                below_emergent,
                below_aquatic,
                aquatic,
                aquatic_flooded,
                aquatic_heights,
            )
        })
        .collect();

    let mut row = CensusRow {
        archetype: label,
        total_vertices: 0,
        below_sea_total: 0,
        below_sea_emergent: 0,
        below_sea_aquatic: 0,
        classified_aquatic: 0,
        classified_aquatic_flooded: 0,
        aquatic_final_min: f32::INFINITY,
        aquatic_final_median: f32::NAN,
    };
    let mut all_aquatic_heights: Vec<f32> = Vec::new();
    for (t, b, be, ba, a, af, ah) in partials {
        row.total_vertices += t;
        row.below_sea_total += b;
        row.below_sea_emergent += be;
        row.below_sea_aquatic += ba;
        row.classified_aquatic += a;
        row.classified_aquatic_flooded += af;
        all_aquatic_heights.extend(ah);
    }
    if !all_aquatic_heights.is_empty() {
        all_aquatic_heights.sort_by(|a, b| a.total_cmp(b));
        row.aquatic_final_min = all_aquatic_heights[0];
        row.aquatic_final_median = all_aquatic_heights[all_aquatic_heights.len() / 2];
    }
    row
}

/// Ignored by default: generates 4 × 169 chunks with erosion (~editor-scale
/// work × 4). Run with `--release ... -- --ignored --nocapture`.
#[test]
#[ignore = "TW1 dip census: heavy 4-archetype full-world generation; run explicitly with --release"]
fn tw1_dip_census_seed12345_radius6() {
    let archetypes = [
        (
            WorldArchetypeId::ContinentalTemperate,
            "ContinentalTemperate",
        ),
        (WorldArchetypeId::Mediterranean, "Mediterranean"),
        (WorldArchetypeId::Desert, "Desert"),
        (WorldArchetypeId::BorealSubarctic, "BorealSubarctic"),
    ];

    println!();
    println!(
        "TW1 dip census — seed 12345, radius {CHUNK_RADIUS} ({} chunks), sea level {SEA_LEVEL}; partition by per-vertex BiomeId",
        (2 * CHUNK_RADIUS + 1) * (2 * CHUNK_RADIUS + 1)
    );
    println!(
        "{:<22} {:>10} {:>12} {:>8} {:>14} {:>9} {:>14} {:>9}",
        "archetype", "vertices", "below-sea", "%", "emergent-dips", "%", "aquatic-flood", "%"
    );
    let mut rows = Vec::new();
    for (id, label) in archetypes {
        let r = census_for(id, label);
        let pct = |n: usize| 100.0 * n as f64 / r.total_vertices as f64;
        println!(
            "{:<22} {:>10} {:>12} {:>7.2}% {:>14} {:>8.3}% {:>14} {:>8.2}%",
            r.archetype,
            r.total_vertices,
            r.below_sea_total,
            pct(r.below_sea_total),
            r.below_sea_emergent,
            pct(r.below_sea_emergent),
            r.below_sea_aquatic,
            pct(r.below_sea_aquatic),
        );
        // Sanity: partition exact; the aquatic side must equal the divergence
        // table's flooded-aquatic column (same criterion, two paths).
        assert_eq!(r.below_sea_emergent + r.below_sea_aquatic, r.below_sea_total);
        assert_eq!(r.below_sea_aquatic, r.classified_aquatic_flooded);
        rows.push(r);
    }

    println!();
    println!("classification-vs-final-height divergence (biome ids classify on PROVISIONAL pre-erosion heights; heights below are FINAL rendered):");
    println!(
        "{:<22} {:>16} {:>9} {:>16} {:>9} {:>12} {:>14}",
        "archetype", "aquatic-ids", "%", "flooded@Y=2", "%", "final-min", "final-median"
    );
    for r in &rows {
        let pct_total = 100.0 * r.classified_aquatic as f64 / r.total_vertices as f64;
        let pct_flooded = if r.classified_aquatic > 0 {
            100.0 * r.classified_aquatic_flooded as f64 / r.classified_aquatic as f64
        } else {
            0.0
        };
        println!(
            "{:<22} {:>16} {:>8.2}% {:>16} {:>8.2}% {:>12.2} {:>14.2}",
            r.archetype,
            r.classified_aquatic,
            pct_total,
            r.classified_aquatic_flooded,
            pct_flooded,
            r.aquatic_final_min,
            r.aquatic_final_median,
        );
    }
}
