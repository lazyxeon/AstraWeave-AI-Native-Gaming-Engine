use anyhow::Result;
use aw_save::{SaveBundleV2, SaveManager, WorldState, PlayerInventory, ItemStack, CompanionProfile, SAVE_SCHEMA_VERSION};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

#[test]
fn test_save_load_roundtrip() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let sm = SaveManager::new(temp_dir.path());
    
    // Create a test save bundle
    let original_bundle = SaveBundleV2 {
        schema: SAVE_SCHEMA_VERSION,
        save_id: Uuid::new_v4(),
        created_at: OffsetDateTime::now_utc(),
        player_id: "test_player".to_string(),
        slot: 5,
        world: WorldState { 
            tick: 12345, 
            ecs_blob: vec![0x10, 0x20, 0x30, 0x40], 
            state_hash: 0x1234567890abcdef 
        },
        inventory: PlayerInventory {
            credits: 9999,
            items: vec![
                ItemStack { 
                    kind: "sword".to_string(), 
                    qty: 1, 
                    attrs: {
                        let mut attrs = HashMap::new();
                        attrs.insert("damage".to_string(), 42);
                        attrs.insert("durability".to_string(), 100);
                        attrs
                    }
                },
                ItemStack { 
                    kind: "potion".to_string(), 
                    qty: 10, 
                    attrs: HashMap::new()
                },
            ],
        },
        companions: vec![
            CompanionProfile { 
                id: "companion_1".to_string(), 
                name: "Echo Prime".to_string(), 
                level: 25, 
                skills: vec!["Combat".to_string(), "Hacking".to_string()], 
                facts: vec!["Remembers the old world".to_string()], 
                episodes_summarized: vec!["Helped in the tutorial mission".to_string()] 
            }
        ],
        meta: {
            let mut meta = HashMap::new();
            meta.insert("difficulty".to_string(), "hard".to_string());
            meta.insert("chapter".to_string(), "3".to_string());
            meta
        },
    };
    
    // Save the bundle
    let save_path = sm.save("test_player", 5, original_bundle.clone())?;
    
    // Load it back
    let (loaded_bundle, loaded_path) = sm.load_latest_slot("test_player", 5)?;
    
    // Verify the paths match
    assert_eq!(save_path, loaded_path);
    
    // Verify the data matches
    assert_eq!(loaded_bundle.schema, original_bundle.schema);
    assert_eq!(loaded_bundle.save_id, original_bundle.save_id);
    assert_eq!(loaded_bundle.player_id, original_bundle.player_id);
    assert_eq!(loaded_bundle.slot, original_bundle.slot);
    assert_eq!(loaded_bundle.world.tick, original_bundle.world.tick);
    assert_eq!(loaded_bundle.world.ecs_blob, original_bundle.world.ecs_blob);
    assert_eq!(loaded_bundle.world.state_hash, original_bundle.world.state_hash);
    assert_eq!(loaded_bundle.inventory.credits, original_bundle.inventory.credits);
    assert_eq!(loaded_bundle.inventory.items.len(), original_bundle.inventory.items.len());
    assert_eq!(loaded_bundle.companions.len(), original_bundle.companions.len());
    assert_eq!(loaded_bundle.companions[0].name, original_bundle.companions[0].name);
    assert_eq!(loaded_bundle.meta, original_bundle.meta);
    
    Ok(())
}

#[test]
fn test_list_saves() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let sm = SaveManager::new(temp_dir.path());
    
    // Create several saves
    for slot in 0..3 {
        let bundle = SaveBundleV2 {
            schema: SAVE_SCHEMA_VERSION,
            save_id: Uuid::new_v4(),
            created_at: OffsetDateTime::now_utc(),
            player_id: "test_player".to_string(),
            slot,
            world: WorldState { tick: slot as u64, ecs_blob: vec![slot], state_hash: 0 },
            inventory: PlayerInventory { credits: slot as u64 * 100, items: vec![] },
            companions: vec![],
            meta: HashMap::new(),
        };
        sm.save("test_player", slot, bundle)?;
    }
    
    // List the saves
    let saves = sm.list_saves("test_player")?;
    assert_eq!(saves.len(), 3);
    
    // Verify they're sorted by slot
    for (i, save) in saves.iter().enumerate() {
        assert_eq!(save.slot, i as u8);
        assert_eq!(save.player_id, "test_player");
        assert_eq!(save.schema, SAVE_SCHEMA_VERSION);
    }
    
    Ok(())
}

#[test]
fn test_file_integrity() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let sm = SaveManager::new(temp_dir.path());
    
    let bundle = SaveBundleV2 {
        schema: SAVE_SCHEMA_VERSION,
        save_id: Uuid::new_v4(),
        created_at: OffsetDateTime::now_utc(),
        player_id: "test_player".to_string(),
        slot: 0,
        world: WorldState { tick: 100, ecs_blob: vec![1, 2, 3, 4, 5], state_hash: 0x123456 },
        inventory: PlayerInventory { credits: 500, items: vec![] },
        companions: vec![],
        meta: HashMap::new(),
    };
    
    let save_path = sm.save("test_player", 0, bundle)?;
    
    // Verify we can read it back
    let loaded = sm.migrate_file_to_latest(&save_path, false)?;
    assert_eq!(loaded.world.tick, 100);
    assert_eq!(loaded.world.ecs_blob, vec![1, 2, 3, 4, 5]);
    assert_eq!(loaded.inventory.credits, 500);
    
    Ok(())
}