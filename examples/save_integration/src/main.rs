//! Save Integration Example for AstraWeave
//! 
//! This example demonstrates how to integrate the aw-save system with 
//! AstraWeave's core World system.

use anyhow::Result;
use astraweave_core::{World, Team, IVec2};
use aw_save::{SaveManager, SaveBundleV2, WorldState, PlayerInventory, ItemStack, CompanionProfile, SAVE_SCHEMA_VERSION};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

fn main() -> Result<()> {
    println!("=== AstraWeave Save Integration Example ===");
    
    // Create a sample game world
    let mut world = World::new();
    
    // Spawn some entities
    let player = world.spawn("Player", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
    let companion = world.spawn("Echo", IVec2 { x: 1, y: 0 }, Team { id: 1 }, 80, 25);
    let enemy = world.spawn("Sentinel", IVec2 { x: 10, y: 5 }, Team { id: 2 }, 60, 15);
    
    println!("Created world with {} entities", 3);
    println!("  Player at {:?} with {} HP", world.pos_of(player).unwrap(), world.health(player).unwrap().hp);
    println!("  Companion at {:?} with {} HP", world.pos_of(companion).unwrap(), world.health(companion).unwrap().hp);
    println!("  Enemy at {:?} with {} HP", world.pos_of(enemy).unwrap(), world.health(enemy).unwrap().hp);
    
    // Simulate some game time
    for _ in 0..10 {
        world.tick(1.0);
    }
    
    println!("After 10 ticks, world time: {:.1}", world.t);
    
    // Create companion profiles (simplified for this example)
    let echo_profile = CompanionProfile {
        id: "echo_001".to_string(),
        name: "Echo".to_string(),
        level: 5,
        skills: vec!["Tactical Analysis".to_string(), "Combat Support".to_string()],
        facts: vec![
            "Player's trusted companion".to_string(),
            "Has advanced combat protocols".to_string(),
            "Remembers pre-war technology".to_string(),
        ],
        episodes_summarized: vec![
            "Tutorial: First activation and bonding".to_string(),
            "Mission 1: Facility escape operation".to_string(),
        ],
    };
    
    // Convert AstraWeave data to save format
    let save_bundle = create_save_bundle(
        "demo_player",
        0, // slot 0
        &world,
        vec![echo_profile],
    )?;
    
    // Save the game
    let save_manager = SaveManager::new("./saves");
    let save_path = save_manager.save("demo_player", 0, save_bundle.clone())?;
    println!("\n✅ Game saved to: {}", save_path.display());
    
    // Load the game back
    let (loaded_bundle, _) = save_manager.load_latest_slot("demo_player", 0)?;
    println!("✅ Game loaded successfully!");
    
    // Verify the loaded data
    println!("\n=== Verification ===");
    println!("Original world tick: {}", world.t as u64);
    println!("Loaded world tick: {}", loaded_bundle.world.tick);
    println!("Original inventory credits: {}", save_bundle.inventory.credits);
    println!("Loaded inventory credits: {}", loaded_bundle.inventory.credits);
    println!("Companions count: {}", loaded_bundle.companions.len());
    
    if !loaded_bundle.companions.is_empty() {
        println!("Companion name: {}", loaded_bundle.companions[0].name);
        println!("Companion facts: {:?}", loaded_bundle.companions[0].facts);
    }
    
    // Demonstrate world restoration
    let restored_world = restore_world_from_save(&loaded_bundle)?;
    println!("\n=== World Restoration ===");
    println!("Restored world time: {:.1}", restored_world.t);
    
    // List all saves for this player
    println!("\n=== All Saves ===");
    for save_meta in save_manager.list_saves("demo_player")? {
        println!("[Slot {:02}] {} - Schema v{}", 
                 save_meta.slot, 
                 save_meta.created_at, 
                 save_meta.schema);
    }
    
    println!("\n🎮 Save integration example completed successfully!");
    Ok(())
}

/// Convert AstraWeave World to a save bundle
fn create_save_bundle(
    player_id: &str,
    slot: u8,
    world: &World,
    companions: Vec<CompanionProfile>,
) -> Result<SaveBundleV2> {
    // Serialize the world state (in a real implementation, you'd use a more sophisticated approach)
    let ecs_blob = serialize_world_state(world)?;
    let state_hash = calculate_world_hash(world);
    
    // Use the provided companions directly
    let companions = companions;
    
    // Create sample inventory
    let inventory = PlayerInventory {
        credits: 1000,
        items: vec![
            ItemStack {
                kind: "health_pack".to_string(),
                qty: 5,
                attrs: HashMap::new(),
            },
            ItemStack {
                kind: "energy_cell".to_string(),
                qty: 10,
                attrs: {
                    let mut attrs = HashMap::new();
                    attrs.insert("energy".to_string(), 100);
                    attrs
                },
            },
        ],
    };
    
    // Create metadata
    let mut meta = HashMap::new();
    meta.insert("game_version".to_string(), "0.4.0".to_string());
    meta.insert("level".to_string(), "tutorial".to_string());
    
    Ok(SaveBundleV2 {
        schema: SAVE_SCHEMA_VERSION,
        save_id: Uuid::new_v4(),
        created_at: OffsetDateTime::now_utc(),
        player_id: player_id.to_string(),
        slot,
        world: WorldState {
            tick: world.t as u64,
            ecs_blob,
            state_hash,
        },
        companions,
        inventory,
        meta,
    })
}

/// Serialize world state to bytes (simplified example)
fn serialize_world_state(world: &World) -> Result<Vec<u8>> {
    // In a real implementation, you'd serialize the ECS components properly
    // For this example, we'll just encode the world time and basic info
    let simplified_state = (world.t, world.next_id);
    Ok(bincode::serialize(&simplified_state).unwrap_or_default())
}

/// Calculate a hash of the world state for quick comparison
fn calculate_world_hash(world: &World) -> u64 {
    // Simple hash based on world time and entity count for demo purposes
    // In reality, you'd hash all the important world state
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    world.t.to_bits().hash(&mut hasher);
    world.next_id.hash(&mut hasher);
    hasher.finish()
}

/// Restore world from save data (simplified example)
fn restore_world_from_save(bundle: &SaveBundleV2) -> Result<World> {
    let mut world = World::new();
    
    // Restore basic world state
    if !bundle.world.ecs_blob.is_empty() {
        if let Ok((time, next_id)) = bincode::deserialize::<(f32, u32)>(&bundle.world.ecs_blob) {
            world.t = time;
            world.next_id = next_id;
        }
    }
    
    // In a real implementation, you'd restore all ECS components, entities, etc.
    // For this demo, we just restore the basic state
    
    Ok(world)
}