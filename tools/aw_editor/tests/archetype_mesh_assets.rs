//! Asset-integrity regression test for the spawn-archetype default meshes.
//!
//! `archetype_mesh()` in `main.rs` (and the World Wizard starter spawn) map
//! archetype names to workspace-relative `.glb` paths. If any of these files
//! is missing or fails `load_gltf`, the editor silently degrades that
//! archetype to a placeholder cube (feed_entities fallback) — the exact
//! "green cube instead of my character" bug class. This test loads every
//! mapped asset headlessly (pure CPU, no GPU) so a broken path fails CI
//! instead of failing silently in the viewport.
//!
//! Keep this list in sync with `archetype_mesh()` in `tools/aw_editor/src/main.rs`.

use std::path::Path;

const ARCHETYPE_MESHES: &[(&str, &str)] = &[
    (
        "Player",
        "assets/The Complete KayKit Collection v4/KayKit Adventurers 2.0/Characters/gltf/Rogue.glb",
    ),
    (
        "Companion",
        "assets/The Complete KayKit Collection v4/KayKit Adventurers 2.0/Characters/gltf/Barbarian.glb",
    ),
    (
        "NPC",
        "assets/The Complete KayKit Collection v4/KayKit Adventurers 2.0/Characters/gltf/Mage.glb",
    ),
    (
        "Enemy",
        "assets/The Complete KayKit Collection v4/KayKit Skeletons 1.1/characters/gltf/Skeleton_Warrior.glb",
    ),
    (
        "Boss",
        "assets/The Complete KayKit Collection v4/KayKit Adventurers 2.0/Characters/gltf/Knight.glb",
    ),
    (
        "Building",
        "assets/3D assets/Castle Kit/Models/GLB format/tower-square.glb",
    ),
    (
        "Interactable",
        "assets/3D assets/Survival Kit/Models/GLB format/box-large.glb",
    ),
    (
        "Placeable",
        "assets/3D assets/Survival Kit/Models/GLB format/campfire-pit.glb",
    ),
    (
        "Resource",
        "assets/3D assets/Survival Kit/Models/GLB format/rock-a.glb",
    ),
    (
        "Support",
        "assets/3D assets/Fantasy Town Kit/Models/GLB format/cart.glb",
    ),
];

#[test]
fn archetype_meshes_exist_and_load() {
    // Tests run with CWD = tools/aw_editor; asset paths are workspace-relative.
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let mut failures = Vec::new();

    for (archetype, rel_path) in ARCHETYPE_MESHES {
        let path = workspace_root.join(rel_path);
        if !path.exists() {
            failures.push(format!("{archetype}: file missing — {rel_path}"));
            continue;
        }
        let opts = astraweave_render::mesh_gltf::GltfOptions::default();
        match astraweave_render::mesh_gltf::load_gltf(&path, &opts) {
            Ok(meshes) if !meshes.is_empty() => {
                let verts: usize = meshes.iter().map(|m| m.vertices.len()).sum();
                println!("{archetype}: OK — {} primitives, {verts} verts ({rel_path})", meshes.len());
            }
            Ok(_) => failures.push(format!("{archetype}: loaded but contains no meshes — {rel_path}")),
            Err(e) => failures.push(format!("{archetype}: load_gltf failed — {rel_path}: {e:#}")),
        }
    }

    assert!(
        failures.is_empty(),
        "archetype mesh assets failed to load (these degrade to placeholder cubes in the editor):\n{}",
        failures.join("\n")
    );
}
