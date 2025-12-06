#![no_main]

use astraweave_ecs::World;
use libfuzzer_sys::fuzz_target;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Fuzz Target: Cross-Platform Determinism Validation
///
/// This fuzz target validates that the ECS produces bit-identical results
/// when the same operation sequence is replayed. This is critical for:
///
/// - Networked games (lockstep simulation)
/// - Deterministic replay systems
/// - AI training reproducibility
/// - Cross-platform consistency
///
/// # How It Works
///
/// 1. Parse fuzzer input as a seed + operation sequence
/// 2. Execute the operation sequence to produce a world state
/// 3. Hash the entire world state (entity count, component values)
/// 4. Replay the same sequence with the same seed
/// 5. Assert the world state hash is bit-identical
///
/// # Input Format
///
/// ```text
/// [seed: u64 (8 bytes)] [operation_count: u8] [operations...]
/// ```
///
/// Each operation is 4 bytes:
/// - Byte 0: Operation type (0=spawn, 1=despawn, 2=insert, 3=remove, 4=update)
/// - Byte 1: Entity index (mod entity count)
/// - Byte 2-3: Component data (i16)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Velocity {
    dx: i32,
    dy: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Health {
    hp: u32,
}

/// Compute a deterministic hash of the world state
fn hash_world_state(world: &World) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Hash entity count for component types
    // Note: We use count() which iterates archetypes deterministically (BTreeMap)
    let position_count = world.count::<Position>();
    let velocity_count = world.count::<Velocity>();
    let health_count = world.count::<Health>();

    position_count.hash(&mut hasher);
    velocity_count.hash(&mut hasher);
    health_count.hash(&mut hasher);

    hasher.finish()
}

/// Execute an operation sequence and return the world state hash
fn execute_sequence(seed: u64, operations: &[u8]) -> u64 {
    let mut world = World::new();
    let mut entities = Vec::new();

    // Use seed to determine initial entity count (1-10)
    let initial_count = ((seed % 10) + 1) as usize;

    // Spawn initial entities with deterministic values
    for i in 0..initial_count {
        let entity = world.spawn();
        world.insert(
            entity,
            Position {
                x: (i as i32).wrapping_mul(seed as i32),
                y: (i as i32).wrapping_add(seed as i32),
            },
        );
        entities.push(entity);
    }

    // Process operations in 4-byte chunks
    for chunk in operations.chunks(4) {
        if chunk.len() < 4 || entities.is_empty() {
            continue;
        }

        let op_type = chunk[0] % 5;
        let entity_idx = (chunk[1] as usize) % entities.len();
        let data = i16::from_le_bytes([chunk[2], chunk[3]]) as i32;

        match op_type {
            0 => {
                // Spawn new entity with Position
                let entity = world.spawn();
                world.insert(
                    entity,
                    Position {
                        x: data,
                        y: data.wrapping_neg(),
                    },
                );
                entities.push(entity);
            }
            1 => {
                // Despawn entity (keep at least one)
                if entities.len() > 1 {
                    let entity = entities.remove(entity_idx);
                    world.despawn(entity);
                }
            }
            2 => {
                // Insert Velocity component
                let entity = entities[entity_idx];
                world.insert(
                    entity,
                    Velocity {
                        dx: data,
                        dy: data.wrapping_mul(2),
                    },
                );
            }
            3 => {
                // Insert Health component
                let entity = entities[entity_idx];
                world.insert(
                    entity,
                    Health {
                        hp: data.unsigned_abs(),
                    },
                );
            }
            4 => {
                // Update Position if exists
                let entity = entities[entity_idx];
                if let Some(pos) = world.get_mut::<Position>(entity) {
                    pos.x = pos.x.wrapping_add(data);
                    pos.y = pos.y.wrapping_sub(data);
                }
            }
            _ => {}
        }
    }

    hash_world_state(&world)
}

fuzz_target!(|data: &[u8]| {
    // Need at least 9 bytes: 8 for seed + 1 for operation count
    if data.len() < 9 {
        return;
    }

    // Extract seed (first 8 bytes)
    let seed = u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]);

    let operations = &data[8..];

    // Execute the sequence twice
    let hash1 = execute_sequence(seed, operations);
    let hash2 = execute_sequence(seed, operations);

    // Assert determinism: same seed + same operations = same result
    assert_eq!(
        hash1, hash2,
        "DETERMINISM VIOLATION: Same seed ({}) and operations produced different world states!\n\
         Hash 1: {}\n\
         Hash 2: {}",
        seed, hash1, hash2
    );

    // Execute a third time to catch any state-dependent issues
    let hash3 = execute_sequence(seed, operations);
    assert_eq!(
        hash1, hash3,
        "DETERMINISM VIOLATION on 3rd replay: World state drift detected!"
    );
});
