//! Additional tests for sparse_set.rs to improve coverage
//!
//! Target: Increase coverage from 58.25% (60/103) to 70-80% (72-82 lines)
//! Focus: Edge cases, capacity management, SparseSetData advanced usage

use astraweave_ecs::Entity;
use astraweave_ecs::sparse_set::{SparseSet, SparseSetData};

// ========== SparseSet Edge Cases ==========

#[test]
fn test_sparse_set_with_capacity() {
    let set = SparseSet::with_capacity(100);
    
    assert_eq!(set.len(), 0);
    assert!(set.is_empty());
    assert!(set.capacity() >= 100); // Should have reserved capacity
}

#[test]
fn test_sparse_set_double_insert() {
    let mut set = SparseSet::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    
    // First insert
    let idx1 = set.insert(e1);
    assert_eq!(idx1, 0);
    assert_eq!(set.len(), 1);
    
    // Second insert (should return existing index)
    let idx2 = set.insert(e1);
    assert_eq!(idx2, 0); // Same index
    assert_eq!(set.len(), 1); // Length unchanged
}

#[test]
fn test_sparse_set_large_id_gap() {
    let mut set = SparseSet::new();
    
    // Insert entity with very large ID (forces sparse array expansion)
    let e1 = unsafe { Entity::from_raw(1000) };
    let idx = set.insert(e1);
    
    assert_eq!(idx, 0);
    assert!(set.contains(e1));
    assert_eq!(set.len(), 1);
}

#[test]
fn test_sparse_set_reserve() {
    let mut set = SparseSet::new();
    
    let initial_capacity = set.capacity();
    
    // Reserve space
    set.reserve(500);
    
    // Capacity should have increased
    assert!(set.capacity() >= initial_capacity + 500);
}

#[test]
fn test_sparse_set_entities_slice() {
    let mut set = SparseSet::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e2 = unsafe { Entity::from_raw(10) };
    let e3 = unsafe { Entity::from_raw(15) };
    
    set.insert(e1);
    set.insert(e2);
    set.insert(e3);
    
    // Get packed entities slice
    let entities = set.entities();
    assert_eq!(entities.len(), 3);
    assert_eq!(entities[0], e1);
    assert_eq!(entities[1], e2);
    assert_eq!(entities[2], e3);
}

#[test]
fn test_sparse_set_remove_last_element() {
    let mut set = SparseSet::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e2 = unsafe { Entity::from_raw(10) };
    let e3 = unsafe { Entity::from_raw(15) };
    
    set.insert(e1);
    set.insert(e2);
    set.insert(e3);
    
    // Remove last element (no swap needed)
    let removed_idx = set.remove(e3);
    assert_eq!(removed_idx, Some(2));
    assert_eq!(set.len(), 2);
    
    // e1 and e2 should remain in original positions
    assert_eq!(set.get(e1), Some(0));
    assert_eq!(set.get(e2), Some(1));
    assert_eq!(set.get(e3), None);
}

#[test]
fn test_sparse_set_remove_nonexistent() {
    let mut set = SparseSet::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e_missing = unsafe { Entity::from_raw(99) };
    
    set.insert(e1);
    
    // Remove entity that doesn't exist
    let result = set.remove(e_missing);
    assert_eq!(result, None);
    assert_eq!(set.len(), 1); // Length unchanged
}

// ========== SparseSetData Edge Cases ==========

#[test]
fn test_sparse_set_data_with_capacity() {
    let set: SparseSetData<i32> = SparseSetData::with_capacity(50);
    
    assert_eq!(set.len(), 0);
    assert!(set.is_empty());
}

#[test]
fn test_sparse_set_data_get_mut() {
    let mut set = SparseSetData::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    set.insert(e1, 42);
    
    // Get mutable reference and modify
    {
        let value = set.get_mut(e1).unwrap();
        *value = 100;
    }
    
    assert_eq!(set.get(e1), Some(&100));
}

#[test]
fn test_sparse_set_data_get_nonexistent() {
    let mut set = SparseSetData::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e_missing = unsafe { Entity::from_raw(99) };
    
    set.insert(e1, "hello");
    
    // Get missing entity
    assert_eq!(set.get(e_missing), None);
    assert_eq!(set.get_mut(e_missing), None);
}

#[test]
fn test_sparse_set_data_contains() {
    let mut set = SparseSetData::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e2 = unsafe { Entity::from_raw(10) };
    
    set.insert(e1, 42);
    
    assert!(set.contains(e1));
    assert!(!set.contains(e2));
}

#[test]
fn test_sparse_set_data_slice() {
    let mut set = SparseSetData::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e2 = unsafe { Entity::from_raw(10) };
    let e3 = unsafe { Entity::from_raw(15) };
    
    set.insert(e1, 100);
    set.insert(e2, 200);
    set.insert(e3, 300);
    
    // Get packed entities slice
    let entities = set.entities();
    assert_eq!(entities.len(), 3);
    
    // Get packed data slice
    let data = set.data();
    assert_eq!(data.len(), 3);
    assert_eq!(data[0], 100);
    assert_eq!(data[1], 200);
    assert_eq!(data[2], 300);
}

#[test]
fn test_sparse_set_data_data_mut() {
    let mut set = SparseSetData::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e2 = unsafe { Entity::from_raw(10) };
    
    set.insert(e1, 10);
    set.insert(e2, 20);
    
    // Get mutable data slice and modify
    {
        let data = set.data_mut();
        data[0] *= 2;
        data[1] *= 3;
    }
    
    assert_eq!(set.get(e1), Some(&20));
    assert_eq!(set.get(e2), Some(&60));
}

#[test]
fn test_sparse_set_data_large_id_gap() {
    let mut set = SparseSetData::new();
    
    // Insert with large ID gap
    let e1 = unsafe { Entity::from_raw(1) };
    let e2 = unsafe { Entity::from_raw(5000) };
    
    set.insert(e1, "first");
    set.insert(e2, "second");
    
    assert_eq!(set.get(e1), Some(&"first"));
    assert_eq!(set.get(e2), Some(&"second"));
    assert_eq!(set.len(), 2);
}

#[test]
fn test_sparse_set_data_remove_nonexistent() {
    let mut set = SparseSetData::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e_missing = unsafe { Entity::from_raw(99) };
    
    set.insert(e1, 42);
    
    // Remove entity that doesn't exist
    let result = set.remove(e_missing);
    assert_eq!(result, None);
    assert_eq!(set.len(), 1); // Length unchanged
}

#[test]
fn test_sparse_set_data_remove_last_element() {
    let mut set = SparseSetData::new();
    
    let e1 = unsafe { Entity::from_raw(5) };
    let e2 = unsafe { Entity::from_raw(10) };
    let e3 = unsafe { Entity::from_raw(15) };
    
    set.insert(e1, 100);
    set.insert(e2, 200);
    set.insert(e3, 300);
    
    // Remove last element (no swap needed)
    let removed = set.remove(e3);
    assert_eq!(removed, Some(300));
    assert_eq!(set.len(), 2);
    
    // Remaining elements should stay in original positions
    assert_eq!(set.get(e1), Some(&100));
    assert_eq!(set.get(e2), Some(&200));
}

#[test]
fn test_sparse_set_data_clear() {
    let mut set = SparseSetData::new();
    
    set.insert(unsafe { Entity::from_raw(1) }, "a");
    set.insert(unsafe { Entity::from_raw(2) }, "b");
    set.insert(unsafe { Entity::from_raw(3) }, "c");
    
    assert_eq!(set.len(), 3);
    
    set.clear();
    
    assert_eq!(set.len(), 0);
    assert!(set.is_empty());
    assert_eq!(set.entities().len(), 0);
    assert_eq!(set.data().len(), 0);
}

// ========== Integration Tests ==========

#[test]
fn test_sparse_set_stress_insert_remove() {
    let mut set = SparseSet::new();
    
    // Insert 100 entities
    for i in 0..100 {
        let entity = unsafe { Entity::from_raw(i) };
        set.insert(entity);
    }
    
    assert_eq!(set.len(), 100);
    
    // Remove every other entity
    for i in (0..100).step_by(2) {
        let entity = unsafe { Entity::from_raw(i) };
        set.remove(entity);
    }
    
    assert_eq!(set.len(), 50);
    
    // Verify remaining entities
    for i in (1..100).step_by(2) {
        let entity = unsafe { Entity::from_raw(i) };
        assert!(set.contains(entity));
    }
}

#[test]
fn test_sparse_set_data_iteration_consistency() {
    let mut set = SparseSetData::new();
    
    for i in 0..50 {
        let entity = unsafe { Entity::from_raw(i) };
        set.insert(entity, i as i32 * 10);
    }
    
    // Verify iteration
    let collected: Vec<_> = set.iter().collect();
    assert_eq!(collected.len(), 50);
    
    for (i, (entity, &value)) in collected.iter().enumerate() {
        assert_eq!(entity.id(), i as u32);
        assert_eq!(value, i as i32 * 10);
    }
}

#[test]
fn test_sparse_set_default_trait() {
    let set: SparseSet = Default::default();
    assert_eq!(set.len(), 0);
    assert!(set.is_empty());
    
    let set_data: SparseSetData<i32> = Default::default();
    assert_eq!(set_data.len(), 0);
    assert!(set_data.is_empty());
}
