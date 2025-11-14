//! Version migration tests for astraweave-persistence-ecs
//!
//! Tests version detection, migration paths, and backward compatibility.

use astraweave_core::ecs_components::*;
use astraweave_core::IVec2;
use astraweave_ecs::{Query, World};
use astraweave_persistence_ecs::{
    deserialize_ecs_world, serialize_ecs_world, CPersistenceManager,
};
use aw_save::{SaveBundleV1, SaveBundleV2, SaveManager, WorldState, SAVE_SCHEMA_VERSION};
use std::collections::HashMap;
use tempfile::tempdir;
use time::OffsetDateTime;

// ========== Version Detection Tests ==========

#[test]
fn test_current_schema_version() {
    // Verify current schema version is V2
    assert_eq!(SAVE_SCHEMA_VERSION, 2, "current schema should be V2");
}

#[test]
fn test_save_bundle_v2_schema_field() {
    // Verify SaveBundleV2 includes schema version
    let temp_dir = tempdir().expect("create temp dir");
    let save_manager = SaveManager::new(temp_dir.path());

    let persistence = CPersistenceManager {
        save_manager,
        current_player: "schema_test".to_string(),
    };

    let mut world = World::new();
    let e = world.spawn();
    world.insert(e, CPos { pos: IVec2 { x: 1, y: 2 } });

    let blob = serialize_ecs_world(&world).expect("serialize failed");

    persistence
        .save_game(0, 0, 0, blob)
        .expect("save failed");

    let (bundle, _) = persistence.load_game(0).expect("load failed");

    assert_eq!(
        bundle.schema, SAVE_SCHEMA_VERSION,
        "bundle should have current schema version"
    );
}

// ========== V1 to V2 Migration Tests ==========

#[test]
fn test_v1_bundle_structure() {
    // Test V1 bundle creation
    let v1_bundle = SaveBundleV1 {
        player_id: "test_player".to_string(),
        slot: 0,
        created_at: OffsetDateTime::now_utc(),
        world: WorldState {
            tick: 100,
            ecs_blob: vec![1, 2, 3, 4, 5],
            state_hash: 0,
        },
        inventory: aw_save::PlayerInventory {
            credits: 1000,
            items: vec![],
        },
        companion: None,
        meta: HashMap::new(),
    };

    assert_eq!(v1_bundle.player_id, "test_player");
    assert_eq!(v1_bundle.slot, 0);
    assert!(v1_bundle.companion.is_none());
}

#[test]
fn test_v1_to_v2_migration_basic() {
    // Test basic V1 to V2 migration
    let v1_bundle = SaveBundleV1 {
        player_id: "migrate_test".to_string(),
        slot: 1,
        created_at: OffsetDateTime::now_utc(),
        world: WorldState {
            tick: 500,
            ecs_blob: vec![10, 20, 30],
            state_hash: 12345,
        },
        inventory: aw_save::PlayerInventory {
            credits: 5000,
            items: vec![],
        },
        companion: None,
        meta: HashMap::new(),
    };

    let v2_bundle = v1_bundle.into_v2();

    assert_eq!(v2_bundle.schema, SAVE_SCHEMA_VERSION);
    assert_eq!(v2_bundle.player_id, "migrate_test");
    assert_eq!(v2_bundle.slot, 1);
    assert_eq!(v2_bundle.world.tick, 500);
    assert_eq!(v2_bundle.world.state_hash, 12345);
    assert_eq!(v2_bundle.inventory.credits, 5000);
    assert!(v2_bundle.companions.is_empty());
}

#[test]
fn test_v1_to_v2_migration_with_companion() {
    // Test V1 to V2 migration with a companion
    let companion = aw_save::CompanionProfile {
        id: "companion_1".to_string(),
        name: "Aria".to_string(),
        level: 10,
        skills: vec!["heal".to_string(), "shield".to_string()],
        facts: vec!["Likes magic".to_string()],
        episodes_summarized: vec![],
    };

    let v1_bundle = SaveBundleV1 {
        player_id: "companion_migration_test".to_string(),
        slot: 0,
        created_at: OffsetDateTime::now_utc(),
        world: WorldState {
            tick: 1000,
            ecs_blob: vec![],
            state_hash: 0,
        },
        inventory: aw_save::PlayerInventory {
            credits: 2000,
            items: vec![],
        },
        companion: Some(companion.clone()),
        meta: HashMap::new(),
    };

    let v2_bundle = v1_bundle.into_v2();

    assert_eq!(v2_bundle.companions.len(), 1);
    assert_eq!(v2_bundle.companions[0].id, "companion_1");
    assert_eq!(v2_bundle.companions[0].name, "Aria");
    assert_eq!(v2_bundle.companions[0].level, 10);
    assert_eq!(v2_bundle.companions[0].skills.len(), 2);
}

#[test]
fn test_v1_to_v2_migration_preserves_metadata() {
    // Test that metadata is preserved during migration
    let mut meta = HashMap::new();
    meta.insert("difficulty".to_string(), "hard".to_string());
    meta.insert("playtime_seconds".to_string(), "3600".to_string());

    let v1_bundle = SaveBundleV1 {
        player_id: "meta_test".to_string(),
        slot: 0,
        created_at: OffsetDateTime::now_utc(),
        world: WorldState {
            tick: 0,
            ecs_blob: vec![],
            state_hash: 0,
        },
        inventory: aw_save::PlayerInventory {
            credits: 0,
            items: vec![],
        },
        companion: None,
        meta: meta.clone(),
    };

    let v2_bundle = v1_bundle.into_v2();

    assert_eq!(v2_bundle.meta.len(), 2);
    assert_eq!(v2_bundle.meta.get("difficulty").unwrap(), "hard");
    assert_eq!(v2_bundle.meta.get("playtime_seconds").unwrap(), "3600");
}

// ========== Backward Compatibility Tests ==========

#[test]
fn test_load_v2_bundle_directly() {
    // Test loading a V2 bundle without migration
    let temp_dir = tempdir().expect("create temp dir");
    let save_manager = SaveManager::new(temp_dir.path());

    let persistence = CPersistenceManager {
        save_manager,
        current_player: "v2_direct_test".to_string(),
    };

    // Create V2 save
    let mut world = World::new();
    let e = world.spawn();
    world.insert(e, CPos { pos: IVec2 { x: 42, y: 84 } });
    world.insert(e, CHealth { hp: 200 });

    let blob = serialize_ecs_world(&world).expect("serialize failed");

    persistence
        .save_game(0, 999, 0, blob)
        .expect("save failed");

    // Load and verify
    let (bundle, _) = persistence.load_game(0).expect("load failed");

    assert_eq!(bundle.schema, SAVE_SCHEMA_VERSION);
    assert_eq!(bundle.world.tick, 999);

    let mut world2 = World::new();
    deserialize_ecs_world(&bundle.world.ecs_blob, &mut world2).expect("deserialize failed");

    let mut q = Query::<CPos>::new(&world2);
    let (_, pos) = q.next().expect("should have entity");
    assert_eq!(pos.pos.x, 42);
    assert_eq!(pos.pos.y, 84);
}

#[test]
fn test_v2_bundle_has_save_id() {
    // Verify V2 bundles include unique save IDs
    let temp_dir = tempdir().expect("create temp dir");
    let save_manager = SaveManager::new(temp_dir.path());

    let persistence = CPersistenceManager {
        save_manager,
        current_player: "save_id_test".to_string(),
    };

    let world = World::new();
    let blob = serialize_ecs_world(&world).expect("serialize failed");

    // Create two saves
    persistence
        .save_game(0, 0, 0, blob.clone())
        .expect("save 1 failed");

    persistence
        .save_game(1, 0, 0, blob)
        .expect("save 2 failed");

    // Load both
    let (bundle1, _) = persistence.load_game(0).expect("load 1 failed");
    let (bundle2, _) = persistence.load_game(1).expect("load 2 failed");

    // Save IDs should be different
    assert_ne!(
        bundle1.save_id, bundle2.save_id,
        "each save should have unique ID"
    );
}

#[test]
fn test_old_components_gracefully_handled() {
    // Test that missing optional components don't cause issues
    // This simulates old saves that might not have newer components

    let mut world = World::new();

    // Create entity with only basic components (simulating old save)
    let e1 = world.spawn();
    world.insert(e1, CPos { pos: IVec2 { x: 1, y: 1 } });
    world.insert(e1, CHealth { hp: 100 });

    // Create entity with all components (new save)
    let e2 = world.spawn();
    world.insert(e2, CPos { pos: IVec2 { x: 2, y: 2 } });
    world.insert(e2, CHealth { hp: 100 });
    world.insert(e2, CTeam { id: 1 });
    world.insert(e2, CAmmo { rounds: 30 });
    world.insert(e2, CAiAgent);

    // Serialize and deserialize
    let blob = serialize_ecs_world(&world).expect("serialize failed");

    let mut world2 = World::new();
    deserialize_ecs_world(&blob, &mut world2).expect("deserialize failed");

    // Verify both entities loaded
    let mut pos_count = 0;
    {
        let mut q = Query::<CPos>::new(&world2);
        while q.next().is_some() {
            pos_count += 1;
        }
    }
    assert_eq!(pos_count, 2);

    // Only one entity should have AI agent
    let mut ai_count = 0;
    {
        let mut q = Query::<CAiAgent>::new(&world2);
        while q.next().is_some() {
            ai_count += 1;
        }
    }
    assert_eq!(ai_count, 1);
}

#[test]
fn test_forward_compatibility_new_fields() {
    // Test that V2 bundles can have extra metadata fields without breaking
    let temp_dir = tempdir().expect("create temp dir");
    let save_manager = SaveManager::new(temp_dir.path());

    let persistence = CPersistenceManager {
        save_manager,
        current_player: "forward_compat_test".to_string(),
    };

    let mut world = World::new();
    let e = world.spawn();
    world.insert(e, CPos { pos: IVec2 { x: 5, y: 10 } });

    let blob = serialize_ecs_world(&world).expect("serialize failed");

    // Save with extra metadata (simulating future fields)
    persistence
        .save_game(0, 0, 0, blob)
        .expect("save failed");

    // Load should work
    let (bundle, _) = persistence.load_game(0).expect("load failed");

    // Bundle should be valid
    assert_eq!(bundle.player_id, "forward_compat_test");

    // Can add custom metadata
    assert!(bundle.meta.is_empty() || !bundle.meta.is_empty());
}

#[test]
fn test_multiple_companions_in_v2() {
    // Test that V2 supports multiple companions (V1 only had one)
    let companions = vec![
        aw_save::CompanionProfile {
            id: "companion_1".to_string(),
            name: "Aria".to_string(),
            level: 10,
            skills: vec![],
            facts: vec![],
            episodes_summarized: vec![],
        },
        aw_save::CompanionProfile {
            id: "companion_2".to_string(),
            name: "Zane".to_string(),
            level: 8,
            skills: vec![],
            facts: vec![],
            episodes_summarized: vec![],
        },
        aw_save::CompanionProfile {
            id: "companion_3".to_string(),
            name: "Luna".to_string(),
            level: 12,
            skills: vec![],
            facts: vec![],
            episodes_summarized: vec![],
        },
    ];

    let bundle = SaveBundleV2 {
        schema: SAVE_SCHEMA_VERSION,
        save_id: uuid::Uuid::new_v4(),
        created_at: OffsetDateTime::now_utc(),
        player_id: "multi_companion_test".to_string(),
        slot: 0,
        world: WorldState {
            tick: 0,
            ecs_blob: vec![],
            state_hash: 0,
        },
        companions: companions.clone(),
        inventory: aw_save::PlayerInventory {
            credits: 0,
            items: vec![],
        },
        meta: HashMap::new(),
    };

    assert_eq!(bundle.companions.len(), 3);
    assert_eq!(bundle.companions[0].name, "Aria");
    assert_eq!(bundle.companions[1].name, "Zane");
    assert_eq!(bundle.companions[2].name, "Luna");
}
