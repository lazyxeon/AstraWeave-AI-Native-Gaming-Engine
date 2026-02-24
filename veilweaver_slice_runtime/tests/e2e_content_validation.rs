//! Content validation integration tests — verify all Veilweaver assets exist
//! and are consistent with the runtime zone registry.
//!
//! Checks:
//! - All 6 zone RON descriptors exist on disk and are valid UTF-8
//! - All 6 greybox meshes exist on disk
//! - All 5 cinematic RON descriptors exist on disk
//! - Dialogue assets exist
//! - Zone registry has 6 entries matching content file names
//! - Zone RON file names map 1:1 to registry keys
//!
//! Runs entirely headless — no wgpu, no egui, pure file-system validation.

use std::path::{Path, PathBuf};
use veilweaver_slice_runtime::zone_transitions::ZoneRegistry;

// ── Helper: locate project `assets/` directory ─────────────────────────────

/// Walks up from `CARGO_MANIFEST_DIR` to find the workspace-root `assets/` folder.
fn assets_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // veilweaver_slice_runtime is one level below workspace root
    let workspace_root = manifest
        .parent()
        .expect("workspace root should exist above manifest dir");
    let assets = workspace_root.join("assets");
    assert!(
        assets.is_dir(),
        "assets/ directory not found at {}",
        assets.display()
    );
    assets
}

// ── Zone RON descriptors ───────────────────────────────────────────────────

const ZONE_RON_FILES: &[&str] = &[
    "cells/Z0_loomspire_sanctum.ron",
    "cells/Z1_echo_grove.ron",
    "cells/Z2_fractured_cliffs.ron",
    "cells/Z2a_side_alcove.ron",
    "cells/Z3_loom_crossroads.ron",
    "cells/Z4_boss_courtyard.ron",
];

#[test]
fn all_zone_descriptors_exist() {
    let assets = assets_dir();
    for ron in ZONE_RON_FILES {
        let path = assets.join(ron);
        assert!(path.exists(), "Missing zone descriptor: {}", path.display());
    }
}

#[test]
fn zone_descriptors_are_valid_utf8() {
    let assets = assets_dir();
    for ron in ZONE_RON_FILES {
        let path = assets.join(ron);
        let content = std::fs::read_to_string(&path);
        assert!(
            content.is_ok(),
            "Zone descriptor is not valid UTF-8: {} — {:?}",
            path.display(),
            content.err()
        );
        let text = content.unwrap();
        assert!(
            !text.is_empty(),
            "Zone descriptor is empty: {}",
            path.display()
        );
    }
}

#[test]
fn zone_descriptors_contain_zone_id() {
    let assets = assets_dir();
    for ron in ZONE_RON_FILES {
        let path = assets.join(ron);
        let text = std::fs::read_to_string(&path).unwrap();
        // Every zone RON should contain a `zone_id` field.
        assert!(
            text.contains("zone_id"),
            "Zone descriptor missing 'zone_id': {}",
            path.display()
        );
    }
}

// ── Greybox meshes ─────────────────────────────────────────────────────────

const GREYBOX_MESHES: &[&str] = &[
    "models/greybox/loomspire_sanctum_greybox.gltf",
    "models/greybox/echo_grove_greybox.gltf",
    "models/greybox/fractured_cliffs_greybox.gltf",
    "models/greybox/side_alcove_greybox.gltf",
    "models/greybox/loom_crossroads_greybox.gltf",
    "models/greybox/boss_courtyard_greybox.gltf",
];

#[test]
fn all_greybox_meshes_exist() {
    let assets = assets_dir();
    for mesh in GREYBOX_MESHES {
        let path = assets.join(mesh);
        assert!(path.exists(), "Missing greybox mesh: {}", path.display());
    }
}

#[test]
fn greybox_meshes_are_non_empty() {
    let assets = assets_dir();
    for mesh in GREYBOX_MESHES {
        let path = assets.join(mesh);
        let meta = std::fs::metadata(&path);
        assert!(
            meta.is_ok(),
            "Cannot read mesh metadata: {}",
            path.display()
        );
        assert!(
            meta.unwrap().len() > 0,
            "Greybox mesh is empty: {}",
            path.display()
        );
    }
}

// ── Cinematic RON descriptors ──────────────────────────────────────────────

const CINEMATIC_FILES: &[&str] = &[
    "cinematics/loom_awakening.ron",
    "cinematics/guided_approach.ron",
    "cinematics/vista_pan.ron",
    "cinematics/boss_intro.ron",
    "cinematics/debrief_resolution.ron",
];

#[test]
fn all_cinematics_exist() {
    let assets = assets_dir();
    for cine in CINEMATIC_FILES {
        let path = assets.join(cine);
        assert!(path.exists(), "Missing cinematic: {}", path.display());
    }
}

#[test]
fn cinematics_are_valid_utf8_and_non_empty() {
    let assets = assets_dir();
    for cine in CINEMATIC_FILES {
        let path = assets.join(cine);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("{} not valid UTF-8: {}", path.display(), e));
        assert!(!text.is_empty(), "Cinematic is empty: {}", path.display());
    }
}

// ── Dialogue assets ────────────────────────────────────────────────────────

#[test]
fn dialogue_intro_exists() {
    let assets = assets_dir();
    let path = assets.join("dialogue_intro.toml");
    assert!(path.exists(), "Missing dialogue asset: {}", path.display());
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(!text.is_empty(), "dialogue_intro.toml is empty");
}

// ── Zone registry consistency ──────────────────────────────────────────────

#[test]
fn zone_registry_has_six_entries() {
    let reg = ZoneRegistry::veilweaver_default();
    assert_eq!(reg.len(), 6, "Expected 6 zones, got {}", reg.len());
}

#[test]
fn zone_registry_names_match_ron_files() {
    let reg = ZoneRegistry::veilweaver_default();
    // Each registry key should correspond to a RON file in assets/cells/.
    let expected_names = [
        "Z0_loomspire_sanctum",
        "Z1_echo_grove",
        "Z2_fractured_cliffs",
        "Z2a_side_alcove",
        "Z3_loom_crossroads",
        "Z4_boss_courtyard",
    ];
    for name in &expected_names {
        assert!(
            reg.coord_for(name).is_some(),
            "Zone '{}' missing from ZoneRegistry",
            name
        );
    }
}

#[test]
fn zone_ron_filenames_align_with_registry() {
    let reg = ZoneRegistry::veilweaver_default();
    // Verify for each zone RON file that stripping the path and extension
    // yields a name present in the registry.
    for ron in ZONE_RON_FILES {
        let stem = Path::new(ron)
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("invalid RON filename");
        assert!(
            reg.coord_for(stem).is_some(),
            "RON file '{}' (stem='{}') has no registry entry",
            ron,
            stem
        );
    }
}

#[test]
fn zone_ron_mesh_paths_reference_existing_meshes() {
    let assets = assets_dir();
    for ron in ZONE_RON_FILES {
        let path = assets.join(ron);
        let text = std::fs::read_to_string(&path).unwrap();
        // Extract mesh_path line if present.
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("mesh_path:") {
                let mesh_rel = trimmed
                    .trim_start_matches("mesh_path:")
                    .trim()
                    .trim_end_matches(',')
                    .trim()
                    .trim_matches('"');
                if !mesh_rel.is_empty() {
                    let mesh_path = assets.join(mesh_rel);
                    assert!(
                        mesh_path.exists(),
                        "Zone {} references non-existent mesh: {}",
                        ron,
                        mesh_path.display()
                    );
                }
            }
        }
    }
}

// ── Navmesh stubs ──────────────────────────────────────────────────────────

const NAVMESH_STUBS: &[&str] = &[
    "navmesh/Z0_loomspire_sanctum_nav.ron",
    "navmesh/Z1_echo_grove_nav.ron",
    "navmesh/Z2_fractured_cliffs_nav.ron",
    "navmesh/Z2a_side_alcove_nav.ron",
    "navmesh/Z3_loom_crossroads_nav.ron",
    "navmesh/Z4_boss_courtyard_nav.ron",
];

#[test]
fn all_navmesh_stubs_exist() {
    let assets = assets_dir();
    for nav in NAVMESH_STUBS {
        let path = assets.join(nav);
        assert!(path.exists(), "Missing navmesh stub: {}", path.display());
    }
}

#[test]
fn navmesh_stubs_contain_zone_id() {
    let assets = assets_dir();
    for nav in NAVMESH_STUBS {
        let path = assets.join(nav);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(
            text.contains("zone_id"),
            "Navmesh stub missing 'zone_id': {}",
            path.display()
        );
    }
}

// ── Aggregate summary ──────────────────────────────────────────────────────

#[test]
fn content_manifest_complete() {
    let assets = assets_dir();

    let zones_ok = ZONE_RON_FILES.iter().all(|f| assets.join(f).exists());
    let meshes_ok = GREYBOX_MESHES.iter().all(|f| assets.join(f).exists());
    let cines_ok = CINEMATIC_FILES.iter().all(|f| assets.join(f).exists());
    let navmesh_ok = NAVMESH_STUBS.iter().all(|f| assets.join(f).exists());
    let dialogue_ok = assets.join("dialogue_intro.toml").exists();

    assert!(zones_ok, "Some zone descriptors missing");
    assert!(meshes_ok, "Some greybox meshes missing");
    assert!(cines_ok, "Some cinematics missing");
    assert!(navmesh_ok, "Some navmesh stubs missing");
    assert!(dialogue_ok, "Dialogue intro missing");

    // Total asset count.
    let total = ZONE_RON_FILES.len()
        + GREYBOX_MESHES.len()
        + CINEMATIC_FILES.len()
        + NAVMESH_STUBS.len()
        + 1;
    assert_eq!(total, 24, "Expected 24 content assets (6+6+5+6+1)");
}
