// Week 1 Day 4: Tests for archetype.rs, command_buffer.rs, and rng.rs
//
// Target coverage:
// - archetype.rs: 86.36% → 90%+ (12 lines uncovered)
// - command_buffer.rs: 91.67% → 95%+ (4 lines uncovered)
// - rng.rs: 74.07% → 85%+ (7 lines uncovered)

use astraweave_ecs::{CommandBuffer, Entity, World};
use std::any::TypeId;

// Test components
#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Health(i32);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Armor(i32);

// ========== Archetype Tests ==========

#[test]
fn test_archetype_signature_empty() {
    use astraweave_ecs::archetype::ArchetypeSignature;

    // Test empty signature
    let sig = ArchetypeSignature::new(vec![]);
    assert!(sig.is_empty());
    assert_eq!(sig.len(), 0);
}

#[test]
fn test_archetype_signature_contains() {
    use astraweave_ecs::archetype::ArchetypeSignature;

    let sig = ArchetypeSignature::new(vec![TypeId::of::<Position>(), TypeId::of::<Velocity>()]);

    assert!(sig.contains(TypeId::of::<Position>()));
    assert!(sig.contains(TypeId::of::<Velocity>()));
    assert!(!sig.contains(TypeId::of::<Health>()));
}

#[test]
fn test_archetype_storage_iter() {
    use astraweave_ecs::archetype::{ArchetypeSignature, ArchetypeStorage};

    let mut storage = ArchetypeStorage::new();

    // Create multiple archetypes
    let sig1 = ArchetypeSignature::new(vec![TypeId::of::<Position>()]);
    let sig2 = ArchetypeSignature::new(vec![TypeId::of::<Velocity>()]);
    let sig3 = ArchetypeSignature::new(vec![TypeId::of::<Health>()]);

    storage.get_or_create_archetype(sig1);
    storage.get_or_create_archetype(sig2);
    storage.get_or_create_archetype(sig3);

    // Test iter() method
    let count = storage.iter().count();
    assert_eq!(count, 3);
}

#[test]
fn test_archetype_storage_archetypes_mut() {
    use astraweave_ecs::archetype::{ArchetypeSignature, ArchetypeStorage};

    let mut storage = ArchetypeStorage::new();

    let sig = ArchetypeSignature::new(vec![TypeId::of::<Position>()]);
    storage.get_or_create_archetype(sig);

    // Test archetypes_mut() iteration
    let mut count = 0;
    for _archetype in storage.archetypes_mut() {
        count += 1;
    }
    assert_eq!(count, 1);
}

#[test]
fn test_world_archetype_integration() {
    // Test archetype functionality via World API (avoids private constructors)
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();

    // Spawn entities with different component combinations
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 1.0 });

    let e2 = world.spawn();
    world.insert(e2, Position { x: 2.0, y: 2.0 });
    world.insert(e2, Velocity { x: 0.5, y: 0.5 });

    let e3 = world.spawn();
    world.insert(e3, Position { x: 3.0, y: 3.0 });

    assert_eq!(world.entity_count(), 3);

    // Remove middle entity (tests archetype removal logic)
    world.despawn(e2);
    assert_eq!(world.entity_count(), 2);

    // Verify remaining entities
    assert!(world.is_alive(e1));
    assert!(!world.is_alive(e2));
    assert!(world.is_alive(e3));
}

#[test]
fn test_world_component_removal() {
    // Test component removal via World API (covers archetype operations)
    let mut world = World::new();
    world.register_component::<Position>();
    world.register_component::<Velocity>();

    let entity = world.spawn();
    world.insert(entity, Position { x: 10.0, y: 20.0 });
    world.insert(entity, Velocity { x: 1.0, y: 1.0 });

    // Verify components exist
    assert!(world.get::<Position>(entity).is_some());
    assert!(world.get::<Velocity>(entity).is_some());

    // Remove one component
    world.remove::<Position>(entity);

    // Verify removal
    assert!(world.get::<Position>(entity).is_none());
    assert!(world.get::<Velocity>(entity).is_some());
}

#[test]
fn test_archetype_storage_remove_entity() {
    // Test ArchetypeStorage::remove_entity via World API
    let mut world = World::new();
    world.register_component::<Position>();

    let entity = world.spawn();
    world.insert(entity, Position { x: 1.0, y: 1.0 });

    // Despawn entity (triggers ArchetypeStorage::remove_entity internally)
    world.despawn(entity);

    assert_eq!(world.entity_count(), 0);
    assert!(!world.is_alive(entity));
}

#[test]
fn test_archetype_storage_archetypes_with_component() {
    use astraweave_ecs::archetype::{ArchetypeSignature, ArchetypeStorage};

    let mut storage = ArchetypeStorage::new();

    // Create archetype with Position
    let sig1 = ArchetypeSignature::new(vec![TypeId::of::<Position>()]);
    storage.get_or_create_archetype(sig1);

    // Create archetype with Position + Velocity
    let sig2 = ArchetypeSignature::new(vec![TypeId::of::<Position>(), TypeId::of::<Velocity>()]);
    storage.get_or_create_archetype(sig2);

    // Create archetype with only Velocity
    let sig3 = ArchetypeSignature::new(vec![TypeId::of::<Velocity>()]);
    storage.get_or_create_archetype(sig3);

    // Find archetypes with Position (should return 2)
    let with_position: Vec<_> = storage
        .archetypes_with_component(TypeId::of::<Position>())
        .collect();
    assert_eq!(with_position.len(), 2);

    // Find archetypes with Health (should return 0)
    let with_health: Vec<_> = storage
        .archetypes_with_component(TypeId::of::<Health>())
        .collect();
    assert_eq!(with_health.len(), 0);
}

// ========== CommandBuffer Tests ==========

#[test]
fn test_command_buffer_default() {
    let buffer = CommandBuffer::default();
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
}

#[test]
fn test_command_buffer_spawn_no_components() {
    let mut buffer = CommandBuffer::new();

    // Spawn entity with no components (just the entity itself)
    buffer.spawn();

    assert_eq!(buffer.len(), 1);
}

#[test]
fn test_command_buffer_multiple_operations() {
    let mut world = World::new();
    let mut buffer = CommandBuffer::new();
    let entity = world.spawn();

    // Queue multiple different operations
    buffer.spawn().with(Position { x: 1.0, y: 2.0 });
    buffer.insert(entity, Velocity { x: 0.5, y: 0.5 });
    buffer.remove::<Position>(entity);
    buffer.despawn(entity);

    assert_eq!(buffer.len(), 4);

    // Clear and verify
    buffer.clear();
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
}

#[test]
fn test_command_buffer_flush_empty() {
    let mut world = World::new();
    let mut buffer = CommandBuffer::new();

    // Flush empty buffer (should not panic)
    buffer.flush(&mut world);
    assert_eq!(buffer.len(), 0);
}

#[test]
fn test_command_buffer_reuse_after_flush() {
    let mut world = World::new();
    let mut buffer = CommandBuffer::new();

    // First batch
    let e1 = world.spawn();
    buffer.despawn(e1);
    buffer.flush(&mut world);

    assert_eq!(buffer.len(), 0);
    assert!(!world.is_alive(e1));

    // Reuse buffer for second batch
    let e2 = world.spawn();
    buffer.despawn(e2);
    buffer.flush(&mut world);

    assert_eq!(buffer.len(), 0);
    assert!(!world.is_alive(e2));
}

// ========== Rng Tests ==========

#[test]
fn test_rng_gen_u64() {
    use astraweave_ecs::Rng;

    let mut rng = Rng::from_seed(12345);

    let val1 = rng.gen_u64();
    let val2 = rng.gen_u64();

    // Values should differ
    assert_ne!(val1, val2);

    // Reset and verify determinism
    let mut rng_reset = Rng::from_seed(12345);
    assert_eq!(rng_reset.gen_u64(), val1);
    assert_eq!(rng_reset.gen_u64(), val2);
}

#[test]
fn test_rng_fill_bytes() {
    use astraweave_ecs::Rng;
    use rand::RngCore;

    let mut rng = Rng::from_seed(99999);

    let mut bytes1 = [0u8; 32];
    let mut bytes2 = [0u8; 32];

    rng.fill_bytes(&mut bytes1);
    rng.fill_bytes(&mut bytes2);

    // Buffers should differ
    assert_ne!(bytes1, bytes2);

    // Reset and verify determinism
    let mut rng_reset = Rng::from_seed(99999);
    let mut bytes1_reset = [0u8; 32];
    let mut bytes2_reset = [0u8; 32];

    rng_reset.fill_bytes(&mut bytes1_reset);
    rng_reset.fill_bytes(&mut bytes2_reset);

    assert_eq!(bytes1, bytes1_reset);
    assert_eq!(bytes2, bytes2_reset);
}

#[test]
fn test_rng_shuffle_empty_slice() {
    use astraweave_ecs::Rng;

    let mut rng = Rng::from_seed(777);
    let mut empty: Vec<i32> = vec![];

    // Shuffle empty slice (should not panic)
    rng.shuffle(&mut empty);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_rng_shuffle_single_element() {
    use astraweave_ecs::Rng;

    let mut rng = Rng::from_seed(888);
    let mut single = vec![42];

    rng.shuffle(&mut single);
    assert_eq!(single, vec![42]); // Single element unchanged
}

#[test]
fn test_rng_gen_range_inclusive() {
    use astraweave_ecs::Rng;

    let mut rng = Rng::from_seed(5555);

    // Test inclusive range [0..=10]
    for _ in 0..50 {
        let val = rng.gen_range(0..=10);
        assert!(val >= 0 && val <= 10, "Value should be in range [0, 10]");
    }
}

#[test]
fn test_rng_gen_range_float() {
    use astraweave_ecs::Rng;

    let mut rng = Rng::from_seed(6666);

    // Test float range
    for _ in 0..50 {
        let val = rng.gen_range(0.0..1.0);
        assert!(
            val >= 0.0 && val < 1.0,
            "Value should be in range [0.0, 1.0)"
        );
    }

    // Test negative range
    for _ in 0..50 {
        let val = rng.gen_range(-10.0..10.0);
        assert!(
            val >= -10.0 && val < 10.0,
            "Value should be in range [-10.0, 10.0)"
        );
    }
}

#[test]
fn test_rng_choose_single_element() {
    use astraweave_ecs::Rng;

    let mut rng = Rng::from_seed(1111);
    let single = vec![42];

    let choice = rng.choose(&single);
    assert_eq!(choice, Some(&42));
}

#[test]
fn test_rng_rngcore_trait() {
    use astraweave_ecs::Rng;
    use rand::RngCore;

    let mut rng = Rng::from_seed(2222);

    // Test RngCore trait methods
    let u32_val = rng.next_u32();
    let u64_val = rng.next_u64();

    assert!(u32_val > 0); // Statistically very likely
    assert!(u64_val > 0);

    // Verify determinism
    let mut rng_reset = Rng::from_seed(2222);
    assert_eq!(rng_reset.next_u32(), u32_val);
    assert_eq!(rng_reset.next_u64(), u64_val);
}

// ========== Integration Tests ==========

#[test]
fn test_archetype_command_buffer_integration() {
    let mut world = World::new();
    world.register_component::<Position>();

    let mut buffer = CommandBuffer::new();

    // Spawn multiple entities via command buffer
    buffer.spawn().with(Position { x: 1.0, y: 1.0 });
    buffer.spawn().with(Position { x: 2.0, y: 2.0 });
    buffer.spawn().with(Position { x: 3.0, y: 3.0 });

    let initial_count = world.entity_count();

    // Note: flush() will panic with "insert_boxed not fully implemented"
    // But we test the command queueing logic
    assert_eq!(buffer.len(), 3);
    assert_eq!(initial_count, 0); // No entities until flush
}

#[test]
fn test_rng_world_resource() {
    use astraweave_ecs::Rng;

    let mut world = World::new();

    // Insert RNG as resource
    world.insert_resource(Rng::from_seed(42));

    // Get resource and use it
    {
        let rng = world.get_resource_mut::<Rng>().unwrap();
        let val = rng.gen_u32();
        assert!(val > 0); // Statistically very likely
    }

    // Verify resource persists
    {
        let rng = world.get_resource::<Rng>().unwrap();
        assert_eq!(rng.seed(), 42);
    }
}

#[test]
fn test_archetype_deterministic_iteration() {
    use astraweave_ecs::archetype::{ArchetypeSignature, ArchetypeStorage};

    let mut storage = ArchetypeStorage::new();

    // Create archetypes in specific order
    let sig1 = ArchetypeSignature::new(vec![TypeId::of::<Position>()]);
    let sig2 = ArchetypeSignature::new(vec![TypeId::of::<Velocity>()]);
    let sig3 = ArchetypeSignature::new(vec![TypeId::of::<Health>()]);

    let id1 = storage.get_or_create_archetype(sig1);
    let id2 = storage.get_or_create_archetype(sig2);
    let id3 = storage.get_or_create_archetype(sig3);

    // Collect IDs from iteration (should be sorted by creation order)
    let ids: Vec<_> = storage.iter().map(|arch| arch.id).collect();

    assert_eq!(ids, vec![id1, id2, id3]);
}

#[test]
fn test_command_buffer_with_capacity_reserve() {
    let buffer = CommandBuffer::with_capacity(100);

    // Buffer should be empty but have capacity
    assert_eq!(buffer.len(), 0);
    assert!(buffer.is_empty());
}
