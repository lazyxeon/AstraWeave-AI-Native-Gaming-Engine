#![no_main]

use libfuzzer_sys::fuzz_target;
use astraweave_ecs::World;

/// Fuzz Target 1: Entity Operations
/// 
/// Tests random sequences of spawn/despawn operations to find:
/// - Panics during entity allocation
/// - Memory corruption during entity recycling
/// - Generation counter overflows
/// - Entity ID conflicts
///
/// Input format: Stream of bytes where each byte represents an operation:
/// - 0-127: Spawn entity
/// - 128-255: Despawn entity at index (value - 128)

fuzz_target!(|data: &[u8]| {
    let mut world = World::new();
    let mut entities = Vec::new();

    for &byte in data {
        if byte < 128 {
            // Spawn entity
            let entity = world.spawn();
            entities.push(entity);
        } else {
            // Despawn entity
            if !entities.is_empty() {
                let idx = ((byte - 128) as usize) % entities.len();
                let entity = entities.swap_remove(idx);
                world.despawn(entity);
            }
        }
    }

    // Validate final state consistency
    assert_eq!(world.entity_count(), entities.len());
    for entity in entities {
        assert!(world.is_alive(entity));
    }
});
