# AstraWeave Save/Load System

The AstraWeave save/load system provides a robust, versioned, and atomic persistence layer for game state. It includes CRC32 integrity checking, LZ4 compression, and support for schema migrations.

## Features

- **Versioned saves**: Schema version tracking with explicit migration support
- **Atomic writes**: Crash-safe saves using temporary files and atomic rename
- **Data integrity**: CRC32 checksums protect against corruption
- **Compression**: LZ4 compression for efficient storage
- **Player organization**: Saves organized by player ID with sanitized filenames
- **Multiple slots**: Support for 256 save slots (0-255) per player
- **Quick access**: JSON index for fast save enumeration
- **CLI tools**: Command-line utilities for save management

## File Format

Save files use the `.awsv` extension with this binary format:

```
magic[4]="ASVS" | version u16 | codec u8(1=LZ4) | reserved u8=0 | data_len u32 | crc32 u32 | data[data_len]
```

Where `data` is postcard-serialized SaveBundleV{N} compressed with LZ4.

## Usage

### Basic Save/Load

```rust
use aw_save::{SaveManager, SaveBundleV2, WorldState, PlayerInventory, CompanionProfile, SAVE_SCHEMA_VERSION};
use time::OffsetDateTime;
use uuid::Uuid;
use std::collections::HashMap;

// Create a save manager
let save_manager = SaveManager::new("./saves");

// Create a save bundle
let bundle = SaveBundleV2 {
    schema: SAVE_SCHEMA_VERSION,
    save_id: Uuid::new_v4(),
    created_at: OffsetDateTime::now_utc(),
    player_id: "alice".to_string(),
    slot: 0,
    world: WorldState {
        tick: 12345,
        ecs_blob: serialize_your_world_state(),
        state_hash: calculate_world_hash(),
    },
    companions: vec![/* companion profiles */],
    inventory: PlayerInventory {
        credits: 1000,
        items: vec![/* items */],
    },
    meta: HashMap::new(),
};

// Save the game
let save_path = save_manager.save("alice", 0, bundle)?;
println!("Saved to: {}", save_path.display());

// Load the latest save for slot 0
let (loaded_bundle, path) = save_manager.load_latest_slot("alice", 0)?;
println!("Loaded from: {}", path.display());
```

### Listing Saves

```rust
// List all saves for a player
for save_meta in save_manager.list_saves("alice")? {
    println!("[Slot {:02}] {} - Schema v{}", 
             save_meta.slot, 
             save_meta.created_at, 
             save_meta.schema);
}
```

### Migration

```rust
// Migrate an old save file to the latest version
let updated_bundle = save_manager.migrate_file_to_latest(&old_save_path, true)?;
```

## CLI Tools

The `aw_save_cli` tool provides command-line access to save management:

```bash
# Create a demo save
cargo run -p aw_save_cli -- demo-save alice 0

# List saves for a player
cargo run -p aw_save_cli -- list alice

# Verify a save file
cargo run -p aw_save_cli -- verify saves/alice/slot00_*.awsv

# Migrate a save file
cargo run -p aw_save_cli -- migrate saves/alice/old_save.awsv
```

## Integration with AstraWeave

### World State Serialization

When integrating with AstraWeave's ECS system, you'll need to serialize your world state:

```rust
fn serialize_world_state(world: &World) -> Result<Vec<u8>> {
    // In practice, you might use bincode, postcard, or a custom format
    // This should include all ECS components, entities, and game state
    let state = YourWorldSnapshot {
        entities: collect_entities(world),
        components: collect_components(world),
        systems_state: collect_system_state(world),
        // ... other world data
    };
    Ok(bincode::serialize(&state)?)
}

fn deserialize_world_state(data: &[u8]) -> Result<World> {
    let snapshot: YourWorldSnapshot = bincode::deserialize(data)?;
    let mut world = World::new();
    restore_entities(&mut world, snapshot.entities);
    restore_components(&mut world, snapshot.components);
    // ... restore other state
    Ok(world)
}
```

### Companion Profiles

Convert your NPC/companion data to the save format:

```rust
fn companion_to_save_format(npc: &NpcProfile) -> CompanionProfile {
    CompanionProfile {
        id: npc.id.clone(),
        name: npc.persona.display_name.clone(),
        level: derive_level_from_npc(npc),
        skills: npc.persona.traits.clone(),
        facts: npc.memory.facts.clone(),
        episodes_summarized: npc.memory.episodes.clone(),
    }
}
```

## Schema Versioning

The save system supports schema evolution through explicit migrations:

### Current Schema (V2)
- Multiple companions (Vec<CompanionProfile>)
- UUID-based save identification
- Extended metadata support

### Legacy Schema (V1)
- Single optional companion
- Timestamp-based identification
- Basic metadata

### Adding New Schema Versions

1. Increment `SAVE_SCHEMA_VERSION` in `lib.rs`
2. Create a new `SaveBundleV{N}` struct
3. Implement migration from the previous version:

```rust
impl SaveBundleV2 {
    pub fn into_v3(self) -> SaveBundleV3 {
        SaveBundleV3 {
            // ... migration logic
        }
    }
}
```

4. Update the migration logic in `read_awsv()` and `migrate_file_to_latest()`

## Directory Structure

Saves are organized as follows:

```
saves/
├── alice/
│   ├── index.json
│   ├── slot00_2025-09-09T21:28:19.380848748Z_uuid.awsv
│   └── slot01_2025-09-09T21:30:15.123456789Z_uuid.awsv
└── bob/
    ├── index.json
    └── slot00_2025-09-09T21:25:10.987654321Z_uuid.awsv
```

- Player IDs are sanitized (alphanumeric, dash, underscore only)
- Files are named with slot, timestamp, and save UUID
- `index.json` provides quick metadata access

## Error Handling

The save system provides detailed error information:

- **Bad magic**: File is not a valid .awsv save
- **CRC mismatch**: File corruption detected
- **Unknown version**: Save file is from an unsupported version
- **Decode errors**: Corruption in the compressed/serialized data

## Performance Considerations

- LZ4 compression is optimized for speed over compression ratio
- Atomic writes ensure consistency but require temporary disk space
- CRC calculation adds minimal overhead
- Index files enable fast save listing without reading full save data

## Testing

The save system includes comprehensive tests:

```bash
# Run all save system tests
cargo test -p aw-save

# Run integration tests
cargo test -p save_integration
```

## Examples

See `examples/save_integration/` for a complete example of integrating the save system with AstraWeave's core World system.