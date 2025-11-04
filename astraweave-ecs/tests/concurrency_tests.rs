//! Concurrency tests using loom model checker
//!
//! These tests validate thread-safety and detect data races in concurrent scenarios.
//! Loom exhaustively explores all possible thread interleavings to find bugs.
//!
//! Run with: cargo test --test concurrency_tests --release
//!
//! Note: Loom tests are slow (exponential in thread count) but exhaustive.

#[cfg(loom)]
use loom::{sync::Arc, thread};

#[cfg(not(loom))]
use std::{sync::Arc, thread};

use astraweave_ecs::{Component, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Velocity {
    dx: i32,
    dy: i32,
}

// ============================================================================
// Entity Allocation Concurrency Tests
// ============================================================================

#[test]
#[cfg(loom)]
fn concurrent_entity_spawn() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            w.spawn()
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            w.spawn()
        });

        let e1 = t1.join().unwrap();
        let e2 = t2.join().unwrap();

        // Entities should be unique
        assert_ne!(e1, e2);

        let w = world.lock().unwrap();
        assert!(w.is_alive(e1));
        assert!(w.is_alive(e2));
        assert_eq!(w.entity_count(), 2);
    });
}

#[test]
#[cfg(loom)]
fn concurrent_spawn_despawn() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));

        // Pre-spawn entity
        let entity = {
            let mut w = world.lock().unwrap();
            w.spawn()
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            w.spawn()
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            w.despawn(entity);
        });

        let new_entity = t1.join().unwrap();
        t2.join().unwrap();

        let w = world.lock().unwrap();
        assert!(w.is_alive(new_entity));
        assert!(!w.is_alive(entity));
    });
}

#[test]
#[cfg(loom)]
fn concurrent_spawn_many() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);
        let world3 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            w.spawn()
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            w.spawn()
        });

        let t3 = thread::spawn(move || {
            let mut w = world3.lock().unwrap();
            w.spawn()
        });

        let e1 = t1.join().unwrap();
        let e2 = t2.join().unwrap();
        let e3 = t3.join().unwrap();

        // All entities should be unique
        assert_ne!(e1, e2);
        assert_ne!(e2, e3);
        assert_ne!(e1, e3);

        let w = world.lock().unwrap();
        assert_eq!(w.entity_count(), 3);
    });
}

// ============================================================================
// Component Access Concurrency Tests
// ============================================================================

#[test]
#[cfg(loom)]
fn concurrent_component_insert() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let entity = {
            let mut w = world.lock().unwrap();
            w.spawn()
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            w.insert(entity, Position { x: 1, y: 2 });
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            w.insert(entity, Velocity { dx: 3, dy: 4 });
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let w = world.lock().unwrap();
        assert!(w.has::<Position>(entity));
        assert!(w.has::<Velocity>(entity));
    });
}

#[test]
#[cfg(loom)]
fn concurrent_component_read() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let entity = {
            let mut w = world.lock().unwrap();
            let e = w.spawn();
            w.insert(e, Position { x: 10, y: 20 });
            e
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let w = world1.lock().unwrap();
            w.get::<Position>(entity).map(|p| *p)
        });

        let t2 = thread::spawn(move || {
            let w = world2.lock().unwrap();
            w.get::<Position>(entity).map(|p| *p)
        });

        let pos1 = t1.join().unwrap();
        let pos2 = t2.join().unwrap();

        // Both reads should succeed
        assert_eq!(pos1, Some(Position { x: 10, y: 20 }));
        assert_eq!(pos2, Some(Position { x: 10, y: 20 }));
    });
}

#[test]
#[cfg(loom)]
fn concurrent_insert_remove() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let entity = {
            let mut w = world.lock().unwrap();
            let e = w.spawn();
            w.insert(e, Position { x: 1, y: 2 });
            e
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            w.insert(entity, Velocity { dx: 3, dy: 4 });
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            w.remove::<Position>(entity);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let w = world.lock().unwrap();
        assert!(w.has::<Velocity>(entity));
        assert!(!w.has::<Position>(entity));
    });
}

// ============================================================================
// Query Concurrency Tests
// ============================================================================

#[test]
#[cfg(loom)]
fn concurrent_query_and_modify() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let entity = {
            let mut w = world.lock().unwrap();
            let e = w.spawn();
            w.insert(e, Position { x: 0, y: 0 });
            e
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            if let Some(pos) = w.get_mut::<Position>(entity) {
                pos.x += 1;
            }
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            if let Some(pos) = w.get_mut::<Position>(entity) {
                pos.y += 1;
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let w = world.lock().unwrap();
        let pos = w.get::<Position>(entity).unwrap();

        // Both modifications should have been applied
        assert_eq!(pos.x, 1);
        assert_eq!(pos.y, 1);
    });
}

#[test]
#[cfg(loom)]
fn concurrent_multi_component_access() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let entity = {
            let mut w = world.lock().unwrap();
            let e = w.spawn();
            w.insert(e, Position { x: 10, y: 20 });
            w.insert(e, Velocity { dx: 1, dy: 2 });
            e
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            if let Some(pos) = w.get_mut::<Position>(entity) {
                pos.x += 5;
            }
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            if let Some(vel) = w.get_mut::<Velocity>(entity) {
                vel.dx += 3;
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let w = world.lock().unwrap();
        assert_eq!(w.get::<Position>(entity).unwrap().x, 15);
        assert_eq!(w.get::<Velocity>(entity).unwrap().dx, 4);
    });
}

// ============================================================================
// Archetype Transition Concurrency Tests
// ============================================================================

#[test]
#[cfg(loom)]
fn concurrent_archetype_transition() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let entity = {
            let mut w = world.lock().unwrap();
            let e = w.spawn();
            w.insert(e, Position { x: 0, y: 0 });
            e
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            w.insert(entity, Velocity { dx: 1, dy: 2 });
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            w.remove::<Position>(entity);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        let w = world.lock().unwrap();
        assert!(w.is_alive(entity));
        // One of the operations succeeded
        // The exact final state depends on interleaving
    });
}

// ============================================================================
// Entity Lifecycle Concurrency Tests
// ============================================================================

#[test]
#[cfg(loom)]
fn concurrent_is_alive_check() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let entity = {
            let mut w = world.lock().unwrap();
            w.spawn()
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let w = world1.lock().unwrap();
            w.is_alive(entity)
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            w.despawn(entity);
        });

        let alive = t1.join().unwrap();
        t2.join().unwrap();

        // is_alive result depends on interleaving
        // but should not panic or corrupt state
        let w = world.lock().unwrap();
        assert!(!w.is_alive(entity));
    });
}

#[test]
#[cfg(loom)]
fn concurrent_double_despawn() {
    loom::model(|| {
        let world = Arc::new(std::sync::Mutex::new(World::new()));
        let entity = {
            let mut w = world.lock().unwrap();
            w.spawn()
        };

        let world1 = Arc::clone(&world);
        let world2 = Arc::clone(&world);

        let t1 = thread::spawn(move || {
            let mut w = world1.lock().unwrap();
            w.despawn(entity);
        });

        let t2 = thread::spawn(move || {
            let mut w = world2.lock().unwrap();
            w.despawn(entity);
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // Double despawn should be safe (idempotent)
        let w = world.lock().unwrap();
        assert!(!w.is_alive(entity));
    });
}

// ============================================================================
// Helper: Standard library fallback tests
// ============================================================================

// These tests run with std when loom is not available
// They don't provide exhaustive checking but validate basic thread-safety

#[test]
#[cfg(not(loom))]
fn std_concurrent_entity_spawn() {
    let world = Arc::new(std::sync::Mutex::new(World::new()));
    let mut handles = vec![];

    for _ in 0..4 {
        let world_clone = Arc::clone(&world);
        let handle = thread::spawn(move || {
            let mut w = world_clone.lock().unwrap();
            w.spawn()
        });
        handles.push(handle);
    }

    let entities: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All entities should be unique
    for i in 0..entities.len() {
        for j in (i + 1)..entities.len() {
            assert_ne!(entities[i], entities[j]);
        }
    }

    let w = world.lock().unwrap();
    assert_eq!(w.entity_count(), 4);
}

#[test]
#[cfg(not(loom))]
fn std_concurrent_component_operations() {
    let world = Arc::new(std::sync::Mutex::new(World::new()));
    let entity = {
        let mut w = world.lock().unwrap();
        w.spawn()
    };

    let world1 = Arc::clone(&world);
    let world2 = Arc::clone(&world);

    let t1 = thread::spawn(move || {
        for i in 0..100 {
            let mut w = world1.lock().unwrap();
            w.insert(entity, Position { x: i, y: i });
        }
    });

    let t2 = thread::spawn(move || {
        for i in 0..100 {
            let mut w = world2.lock().unwrap();
            w.insert(entity, Velocity { dx: i, dy: i });
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();

    let w = world.lock().unwrap();
    assert!(w.has::<Position>(entity));
    assert!(w.has::<Velocity>(entity));
}

// ============================================================================
// Test Summary
// ============================================================================

#[cfg(test)]
mod test_summary {
    //! Summary of concurrency tests:
    //!
    //! Entity Allocation (3 tests):
    //! - concurrent_entity_spawn: Two threads spawn entities simultaneously
    //! - concurrent_spawn_despawn: Spawn and despawn in parallel
    //! - concurrent_spawn_many: Three threads spawn entities
    //!
    //! Component Access (4 tests):
    //! - concurrent_component_insert: Insert different components in parallel
    //! - concurrent_component_read: Concurrent reads of same component
    //! - concurrent_insert_remove: Insert one component while removing another
    //! - concurrent_query_and_modify: Concurrent modifications to same component
    //!
    //! Query Operations (2 tests):
    //! - concurrent_multi_component_access: Access different components in parallel
    //! - concurrent_archetype_transition: Trigger archetype transitions in parallel
    //!
    //! Entity Lifecycle (2 tests):
    //! - concurrent_is_alive_check: Check liveness during despawn
    //! - concurrent_double_despawn: Despawn same entity twice
    //!
    //! Total: 11 loom tests + 2 std fallback tests = 13 concurrency tests
    //!
    //! All tests validate:
    //! - No data races
    //! - No panics under concurrent access
    //! - Proper synchronization via Mutex
    //! - State consistency after concurrent operations
}
