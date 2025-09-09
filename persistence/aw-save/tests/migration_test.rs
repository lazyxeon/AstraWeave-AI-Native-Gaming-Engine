use anyhow::Result;
use aw_save::{SaveBundleV1, SaveBundleV2, SaveManager, WorldState, PlayerInventory, CompanionProfile, SAVE_SCHEMA_VERSION};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

#[test]
fn test_v1_to_v2_migration() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let sm = SaveManager::new(temp_dir.path());
    
    // Create a V1 save bundle (simulating legacy save)
    let v1_bundle = SaveBundleV1 {
        player_id: "legacy_player".to_string(),
        slot: 2,
        created_at: OffsetDateTime::now_utc(),
        world: WorldState { 
            tick: 5000, 
            ecs_blob: vec![0xaa, 0xbb, 0xcc], 
            state_hash: 0x987654321 
        },
        inventory: PlayerInventory {
            credits: 1500,
            items: vec![],
        },
        // V1 had a single optional companion
        companion: Some(CompanionProfile {
            id: "legacy_companion".to_string(),
            name: "Old Echo".to_string(),
            level: 10,
            skills: vec!["Legacy Skill".to_string()],
            facts: vec!["Was saved in V1 format".to_string()],
            episodes_summarized: vec![],
        }),
        meta: {
            let mut meta = HashMap::new();
            meta.insert("legacy".to_string(), "true".to_string());
            meta
        },
    };
    
    // Convert to V2 via migration
    let v2_bundle = v1_bundle.into_v2();
    
    // Verify the migration worked correctly
    assert_eq!(v2_bundle.schema, SAVE_SCHEMA_VERSION);
    assert_eq!(v2_bundle.player_id, "legacy_player");
    assert_eq!(v2_bundle.slot, 2);
    assert_eq!(v2_bundle.world.tick, 5000);
    assert_eq!(v2_bundle.world.ecs_blob, vec![0xaa, 0xbb, 0xcc]);
    assert_eq!(v2_bundle.inventory.credits, 1500);
    
    // The single companion should be converted to a vec with one element
    assert_eq!(v2_bundle.companions.len(), 1);
    assert_eq!(v2_bundle.companions[0].name, "Old Echo");
    assert_eq!(v2_bundle.companions[0].level, 10);
    
    // Metadata should be preserved
    assert_eq!(v2_bundle.meta.get("legacy"), Some(&"true".to_string()));
    
    // The save_id should be auto-generated
    assert_ne!(v2_bundle.save_id, Uuid::nil());
    
    Ok(())
}

#[test]
fn test_empty_companion_migration() -> Result<()> {
    // Test V1 bundle with no companion
    let v1_bundle = SaveBundleV1 {
        player_id: "no_companion_player".to_string(),
        slot: 0,
        created_at: OffsetDateTime::now_utc(),
        world: WorldState { tick: 1, ecs_blob: vec![0x01], state_hash: 0 },
        inventory: PlayerInventory { credits: 0, items: vec![] },
        companion: None, // No companion in V1
        meta: HashMap::new(),
    };
    
    let v2_bundle = v1_bundle.into_v2();
    
    // Should result in an empty companions vector
    assert_eq!(v2_bundle.companions.len(), 0);
    assert_eq!(v2_bundle.player_id, "no_companion_player");
    
    Ok(())
}