//! Additional tests for blob_vec.rs and entity_allocator.rs to improve coverage
//!
//! Target: Increase coverage to 80%+
//! - blob_vec.rs: 71.64% (48/67) → 80%+ (54+ lines)
//! - entity_allocator.rs: 87.50% (56/64) → 90%+ (58+ lines)

use astraweave_ecs::blob_vec::BlobVec;
use astraweave_ecs::entity_allocator::{Entity, EntityAllocator};

// ========== BlobVec Additional Tests ==========

#[test]
fn test_blob_vec_with_capacity() {
    let blob = BlobVec::with_capacity::<i32>(50);

    assert_eq!(blob.len(), 0);
    assert!(blob.is_empty());
    assert!(blob.capacity() >= 50);
}

#[test]
fn test_blob_vec_with_zero_capacity() {
    let blob = BlobVec::with_capacity::<i32>(0);

    assert_eq!(blob.len(), 0);
    assert_eq!(blob.capacity(), 0);
}

#[test]
fn test_blob_vec_get_out_of_bounds() {
    let mut blob = BlobVec::new::<i32>();

    unsafe {
        blob.push(42);

        // Valid index
        assert_eq!(blob.get::<i32>(0), Some(&42));

        // Out of bounds
        assert_eq!(blob.get::<i32>(1), None);
        assert_eq!(blob.get::<i32>(100), None);
    }
}

#[test]
fn test_blob_vec_get_mut_out_of_bounds() {
    let mut blob = BlobVec::new::<i32>();

    unsafe {
        blob.push(42);

        // Valid index
        assert_eq!(blob.get_mut::<i32>(0), Some(&mut 42));

        // Out of bounds
        assert_eq!(blob.get_mut::<i32>(1), None);
        assert_eq!(blob.get_mut::<i32>(100), None);
    }
}

#[test]
fn test_blob_vec_empty_slice() {
    let blob = BlobVec::new::<i32>();

    unsafe {
        let slice = blob.as_slice::<i32>();
        assert_eq!(slice.len(), 0);
    }
}

#[test]
fn test_blob_vec_reserve_no_realloc() {
    let mut blob = BlobVec::new::<i32>();

    // Reserve space
    blob.reserve(100);
    let capacity_after_reserve = blob.capacity();

    // Push some elements (less than reserved)
    unsafe {
        for i in 0..50 {
            blob.push(i);
        }
    }

    // Capacity should not have changed
    assert_eq!(blob.capacity(), capacity_after_reserve);
}

#[test]
fn test_blob_vec_push_triggers_realloc() {
    let mut blob = BlobVec::new::<i32>();

    // Push elements to trigger multiple reallocations
    unsafe {
        for i in 0..100 {
            blob.push(i as i32);
        }
    }

    assert_eq!(blob.len(), 100);

    // Verify all elements
    unsafe {
        let slice = blob.as_slice::<i32>();
        for i in 0..100 {
            assert_eq!(slice[i], i as i32);
        }
    }
}

#[test]
fn test_blob_vec_swap_remove_last() {
    let mut blob = BlobVec::new::<i32>();

    unsafe {
        blob.push(10);
        blob.push(20);
        blob.push(30);

        // Remove last element (no swap needed)
        let removed = blob.swap_remove::<i32>(2);
        assert_eq!(removed, 30);
        assert_eq!(blob.len(), 2);

        // Remaining elements unchanged
        assert_eq!(blob.get::<i32>(0), Some(&10));
        assert_eq!(blob.get::<i32>(1), Some(&20));
    }
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_blob_vec_swap_remove_out_of_bounds() {
    let mut blob = BlobVec::new::<i32>();

    unsafe {
        blob.push(42);

        // This should panic
        blob.swap_remove::<i32>(5);
    }
}

#[test]
fn test_blob_vec_clear_empty() {
    let mut blob = BlobVec::new::<i32>();

    // Clear empty BlobVec
    blob.clear();

    assert_eq!(blob.len(), 0);
    assert!(blob.is_empty());
}

#[test]
fn test_blob_vec_no_drop_type() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct NoDropStruct {
        value: i32,
    }

    let mut blob = BlobVec::new::<NoDropStruct>();

    unsafe {
        blob.push(NoDropStruct { value: 1 });
        blob.push(NoDropStruct { value: 2 });
    }

    // Clear should work even without drop implementation
    blob.clear();
    assert_eq!(blob.len(), 0);
}

// ========== EntityAllocator Additional Tests ==========

#[test]
fn test_entity_allocator_reserve() {
    let mut allocator = EntityAllocator::new();

    // Reserve capacity
    allocator.reserve(500);

    // Spawn entities (should not reallocate)
    for _ in 0..100 {
        allocator.spawn();
    }

    assert_eq!(allocator.alive_count(), 100);
}

#[test]
fn test_entity_generation_lookup() {
    let mut allocator = EntityAllocator::new();

    let e1 = allocator.spawn();
    assert_eq!(allocator.generation(e1.id()), Some(0));

    allocator.despawn(e1);
    assert_eq!(allocator.generation(e1.id()), Some(1)); // Generation incremented

    // Non-existent ID
    assert_eq!(allocator.generation(9999), None);
}

#[test]
fn test_entity_despawn_nonexistent() {
    let mut allocator = EntityAllocator::new();

    // Despawn twice should fail on second attempt
    let e1 = allocator.spawn();
    assert!(allocator.despawn(e1)); // First despawn succeeds
    assert!(!allocator.despawn(e1)); // Second despawn fails (stale generation)
}

#[test]
fn test_entity_statistics() {
    let mut allocator = EntityAllocator::new();

    assert_eq!(allocator.spawned_count(), 0);
    assert_eq!(allocator.despawned_count(), 0);

    let e1 = allocator.spawn();
    let _e2 = allocator.spawn();

    assert_eq!(allocator.spawned_count(), 2);
    assert_eq!(allocator.despawned_count(), 0);

    allocator.despawn(e1);

    assert_eq!(allocator.spawned_count(), 2);
    assert_eq!(allocator.despawned_count(), 1);

    let _e3 = allocator.spawn(); // Reuses e1's slot

    assert_eq!(allocator.spawned_count(), 3);
    assert_eq!(allocator.despawned_count(), 1);
}

#[test]
fn test_entity_default_trait() {
    let allocator: EntityAllocator = Default::default();

    assert_eq!(allocator.alive_count(), 0);
    assert_eq!(allocator.capacity(), 0);
}

#[test]
fn test_entity_free_list_reuse_order() {
    let mut allocator = EntityAllocator::new();

    let e1 = allocator.spawn(); // ID 0
    let e2 = allocator.spawn(); // ID 1
    let e3 = allocator.spawn(); // ID 2

    // Despawn in specific order
    allocator.despawn(e1);
    allocator.despawn(e3);
    allocator.despawn(e2);

    // Spawn new entities - the order depends on free list implementation
    let e4 = allocator.spawn();
    let e5 = allocator.spawn();
    let e6 = allocator.spawn();

    // Verify IDs are reused (exact order may vary by implementation)
    let reused_ids = vec![e4.id(), e5.id(), e6.id()];
    assert!(reused_ids.contains(&0)); // IDs 0, 1, 2 should all be reused
    assert!(reused_ids.contains(&1));
    assert!(reused_ids.contains(&2));
}

#[test]
fn test_entity_generation_wrapping() {
    let mut allocator = EntityAllocator::new();

    // Test that generations increment correctly with spawn/despawn cycle
    let e1 = allocator.spawn(); // generation 0
    let id1 = e1.id();
    allocator.despawn(e1); // generation increments to 1

    let e2 = allocator.spawn(); // same ID, generation 1
    assert_eq!(e2.id(), id1); // ID reused
    assert_ne!(e2.generation(), e1.generation()); // Generation different

    // In practice, generations use wrapping_add so u32::MAX + 1 = 0
    // This test validates the generation increment behavior
}

#[test]
fn test_entity_hash() {
    use std::collections::HashSet;

    let mut allocator = EntityAllocator::new();

    // Test that Entity can be stored in HashSet (requires Hash)
    let e1 = allocator.spawn();
    let e2 = allocator.spawn();
    let e3 = allocator.spawn();

    let mut set = HashSet::new();
    set.insert(e1);
    set.insert(e2);
    set.insert(e3);

    assert_eq!(set.len(), 3);
    assert!(set.contains(&e1));
    assert!(set.contains(&e2));
    assert!(set.contains(&e3));
}

#[test]
fn test_entity_eq_ord() {
    let mut allocator = EntityAllocator::new();

    // Test Entity equality and ordering
    let e1 = allocator.spawn(); // Entity 0v0
    let e2 = allocator.spawn(); // Entity 1v0

    allocator.despawn(e1);
    let e3 = allocator.spawn(); // Entity 0v1 (reused ID, new generation)

    assert_ne!(e1, e3); // Same ID, different generation
    assert_ne!(e1, e2); // Different ID
    assert_eq!(e1, e1); // Same entity

    // Ordering: ID first, then generation
    assert!(e1.id() == e3.id() && e1.generation() < e3.generation());
}

// ========== Integration Tests ==========

#[test]
fn test_blob_vec_entity_storage() {
    // Use BlobVec to store entities (simulating archetype storage)
    let mut blob = BlobVec::new::<Entity>();
    let mut allocator = EntityAllocator::new();

    // Spawn and store entities
    for _ in 0..10 {
        let entity = allocator.spawn();
        unsafe {
            blob.push(entity);
        }
    }

    assert_eq!(blob.len(), 10);
    assert_eq!(allocator.alive_count(), 10);

    // Verify all entities are alive
    unsafe {
        let entities = blob.as_slice::<Entity>();
        for entity in entities {
            assert!(allocator.is_alive(*entity));
        }
    }

    // Despawn some entities
    unsafe {
        let entities = blob.as_slice::<Entity>();
        for i in (0..10).step_by(2) {
            allocator.despawn(entities[i]);
        }
    }

    assert_eq!(allocator.alive_count(), 5);
}

#[test]
fn test_allocator_stress_spawn_despawn() {
    let mut allocator = EntityAllocator::new();

    // Spawn 1000 entities
    let entities: Vec<_> = (0..1000).map(|_| allocator.spawn()).collect();

    assert_eq!(allocator.alive_count(), 1000);

    // Despawn every other entity
    for (i, entity) in entities.iter().enumerate() {
        if i % 2 == 0 {
            allocator.despawn(*entity);
        }
    }

    assert_eq!(allocator.alive_count(), 500);

    // Spawn 500 more (should reuse despawned IDs)
    let new_entities: Vec<_> = (0..500).map(|_| allocator.spawn()).collect();

    assert_eq!(allocator.alive_count(), 1000);

    // Verify old odd-indexed entities still alive
    for (i, entity) in entities.iter().enumerate() {
        if i % 2 == 1 {
            assert!(allocator.is_alive(*entity));
        } else {
            assert!(!allocator.is_alive(*entity));
        }
    }

    // Verify all new entities are alive
    for entity in new_entities {
        assert!(allocator.is_alive(entity));
    }
}
