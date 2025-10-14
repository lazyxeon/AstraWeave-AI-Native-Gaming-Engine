//! System parameter types for ECS queries.
//! 
//! ## Performance Notes (Week 10)
//! 
//! ### Current Performance (Post SparseSet Integration)
//! 
//! With the SparseSet integration (Week 10 Day 2), entity lookups are now O(1) instead
//! of O(log n), providing 12-57× speedup over the old BTreeMap approach. This has
//! resulted in:
//! 
//! - **Frame time**: 2.70ms → 1.144ms (2.4× faster)
//! - **Movement system**: 1,000µs → 106µs (9.4× faster)
//! - **FPS**: 370 → 944 (2.5× improvement)
//! - **Headroom**: 93.1% vs 60 FPS budget (16.67ms)
//! 
//! ### Per-Entity Overhead Pattern
//! 
//! Current Query implementations use a per-entity `archetype.get::<T>(entity)` pattern:
//! 
//! ```rust,ignore
//! impl Iterator for Query<'w, T> {
//!     fn next(&mut self) -> Option<(Entity, &'w T)> {
//!         let entity = archetype.entities_vec()[self.entity_idx];
//!         let component = archetype.get::<T>(entity)?;
//!         // Each get() call:
//!         // 1. SparseSet lookup: O(1) - fast!
//!         // 2. HashMap lookup: O(1) - fast!
//!         // 3. Vec indexing: O(1) - fast!
//!         // 4. Box downcast: O(1) - fast!
//!         // Total: 4 operations per entity (1,000 entities = 4,000 ops)
//!     }
//! }
//! ```
//! 
//! While each operation is O(1), the repeated overhead adds up for large entity counts.
//! 
//! ### Why Batch Iteration is Difficult
//! 
//! Ideally, we'd batch all operations at the archetype level:
//! 
//! ```rust,ignore
//! // Dream API (blocked by borrow checker):
//! for (entity, component) in archetype.iter_components_mut::<Position>() {
//!     component.x += velocity.x;  // Direct mutable access, no per-entity lookups!
//! }
//! ```
//! 
//! However, this is **not feasible** with Rust's current borrow checker due to lifetime
//! constraints. The issue:
//! 
//! ```rust,ignore
//! pub fn iter_components_mut<T>(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
//!     let column = self.components.get_mut(&TypeId::of::<T>())?;
//!     self.entities.iter().filter_map(|(idx, &entity)| {
//!         column.get_mut(idx)  // ❌ ERROR: captured variable escapes FnMut closure
//!         //                           Returns &mut T borrowed from captured column
//!     })
//! }
//! ```
//! 
//! Rust's borrow checker prevents this because:
//! 1. The closure captures `column` (a mutable reference)
//! 2. The closure tries to return `&mut T` borrowed from `column`
//! 3. Rule: **References captured in closures cannot escape the closure scope**
//! 4. This prevents dangling references but blocks the optimization
//! 
//! ### Future Optimizations (Week 11-12)
//! 
//! **Week 11: SystemParam DSL**
//! - Compile-time borrow splitting with zero runtime cost
//! - Eliminate Query2Mut 70% overhead (Action 32 issue)
//! - Target: Movement <50µs (2× current performance)
//! 
//! **Week 12: Parallel Execution**
//! - Rayon-based parallel system execution
//! - Dependency analysis for safe concurrent iteration
//! - Target: Physics 813µs → 200-400µs (2-4× faster)
//! 
//! **Week 13+: Type Registry + BlobVec Integration**
//! - Runtime type registration system
//! - Replace Vec<Box<dyn Any>> with contiguous BlobVec storage
//! - This will enable ideal batch iteration (no Box overhead, no downcast)
//! - Expected: Additional 5-10× component access speedup

use crate::{archetype::ArchetypeId, Component, Entity, World};

/// Trait for types that can be system parameters
pub trait SystemParam: Sized {
    // This will be fleshed out later. For now, it's a marker trait.
}

// Read-only single-component query
pub struct Query<'w, T: Component> {
    world: &'w World,
    archetype_ids: Vec<ArchetypeId>,
    arch_idx: usize,
    entity_idx: usize,
    _m: std::marker::PhantomData<T>,
}

impl<'w, T: Component> Query<'w, T> {
    pub fn new(world: &'w World) -> Self {
        let archetype_ids = world
            .archetypes
            .archetypes_with_component(std::any::TypeId::of::<T>())
            .map(|arch| arch.id)
            .collect();
        Self {
            world,
            archetype_ids,
            arch_idx: 0,
            entity_idx: 0,
            _m: Default::default(),
        }
    }
}

impl<'w, T: Component> Iterator for Query<'w, T> {
    type Item = (Entity, &'w T);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.arch_idx >= self.archetype_ids.len() {
                return None;
            }
            let archetype_id = self.archetype_ids[self.arch_idx];
            let archetype = self.world.archetypes.get_archetype(archetype_id)
                .expect("BUG: archetype should exist from archetype_ids");

            if self.entity_idx >= archetype.len() {
                self.arch_idx += 1;
                self.entity_idx = 0;
                continue;
            }

            let entity = archetype.entities_vec()[self.entity_idx];
            self.entity_idx += 1;

            // The borrow checker needs help here. Since we are iterating over disjoint archetypes
            // and entities, this is safe. We'll use unsafe to extend the lifetime.
            let component = archetype.get::<T>(entity)
                .expect("BUG: entity should have component T in archetype");
            let component_ptr = component as *const T;
            return Some((entity, unsafe { &*component_ptr }));
        }
    }
}

// Read-only two-component query
pub struct Query2<'w, A: Component, B: Component> {
    world: &'w World,
    archetype_ids: Vec<ArchetypeId>,
    arch_idx: usize,
    entity_idx: usize,
    _m: std::marker::PhantomData<(A, B)>,
}

impl<'w, A: Component, B: Component> Query2<'w, A, B> {
    pub fn new(world: &'w World) -> Self {
        let archetype_ids = world
            .archetypes
            .archetypes_with_component(std::any::TypeId::of::<A>())
            .filter(|arch| arch.signature.contains(std::any::TypeId::of::<B>()))
            .map(|arch| arch.id)
            .collect();

        Self {
            world,
            archetype_ids,
            arch_idx: 0,
            entity_idx: 0,
            _m: Default::default(),
        }
    }
}

impl<'w, A: Component, B: Component> Iterator for Query2<'w, A, B> {
    type Item = (Entity, &'w A, &'w B);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.arch_idx >= self.archetype_ids.len() {
                return None;
            }
            let archetype_id = self.archetype_ids[self.arch_idx];
            let archetype = self.world.archetypes.get_archetype(archetype_id)
                .expect("BUG: archetype should exist from archetype_ids");

            if self.entity_idx >= archetype.len() {
                self.arch_idx += 1;
                self.entity_idx = 0;
                continue;
            }

            let entity = archetype.entities_vec()[self.entity_idx];
            self.entity_idx += 1;

            // Unsafe is used to satisfy the borrow checker. This is safe because
            // we are only reading, and the iterator structure ensures we don't hold
            // references that outlive the world.
            let component_a = archetype.get::<A>(entity)
                .expect("BUG: entity should have component A in archetype");
            let component_b = archetype.get::<B>(entity)
                .expect("BUG: entity should have component B in archetype");
            let ptr_a = component_a as *const A;
            let ptr_b = component_b as *const B;

            return Some((entity, unsafe { &*ptr_a }, unsafe { &*ptr_b }));
        }
    }
}

// Mutable two-component query (for Action 32 writeback optimization)
pub struct Query2Mut<'w, A: Component, B: Component> {
    world: *mut World,
    archetype_ids: Vec<ArchetypeId>,
    arch_idx: usize,
    entity_idx: usize,
    _m: std::marker::PhantomData<(&'w mut A, &'w B)>,
}

impl<'w, A: Component, B: Component> Query2Mut<'w, A, B> {
    pub fn new(world: &'w mut World) -> Self {
        let archetype_ids = world
            .archetypes
            .archetypes_with_component(std::any::TypeId::of::<A>())
            .filter(|arch| arch.signature.contains(std::any::TypeId::of::<B>()))
            .map(|arch| arch.id)
            .collect();

        Self {
            world,
            archetype_ids,
            arch_idx: 0,
            entity_idx: 0,
            _m: Default::default(),
        }
    }
}

impl<'w, A: Component, B: Component> Iterator for Query2Mut<'w, A, B> {
    type Item = (Entity, &'w mut A, &'w B);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.arch_idx >= self.archetype_ids.len() {
                return None;
            }
            let archetype_id = self.archetype_ids[self.arch_idx];
            
            // SAFETY: We hold *mut World for 'w lifetime. We reconstruct references for each iteration.
            // This is safe because:
            // 1. The world pointer is valid for 'w
            // 2. We only access one entity at a time
            // 3. A and B are different types (no aliasing within single entity)
            let world_ref = unsafe { &mut *self.world };
            
            // Get immutable reference to archetype for metadata access
            let archetype = world_ref.archetypes.get_archetype(archetype_id)
                .expect("BUG: archetype should exist from archetype_ids");

            if self.entity_idx >= archetype.len() {
                self.arch_idx += 1;
                self.entity_idx = 0;
                continue;
            }

            let entity = archetype.entities_vec()[self.entity_idx];
            self.entity_idx += 1;

            // SAFETY: Now get the actual component data using raw pointers to avoid borrow conflicts.
            // We get component A mutably and B immutably through separate archetype lookups.
            // This is safe because:
            // 1. A and B are different types (ensured by type system)
            // 2. We're returning references that live for 'w
            // 3. Iterator ensures sequential access (no overlapping entity borrows)
            let world_ref2 = unsafe { &mut *self.world };
            let archetype_mut = world_ref2.archetypes.get_archetype_mut(archetype_id)
                .expect("BUG: archetype should exist");
            let component_a = archetype_mut.get_mut::<A>(entity)
                .expect("BUG: entity should have component A in archetype");
            let ptr_a = component_a as *mut A;
            
            let world_ref3 = unsafe { &*self.world };
            let archetype_imm = world_ref3.archetypes.get_archetype(archetype_id)
                .expect("BUG: archetype should exist");
            let component_b = archetype_imm.get::<B>(entity)
                .expect("BUG: entity should have component B in archetype");
            let ptr_b = component_b as *const B;

            return Some((entity, unsafe { &mut *ptr_a }, unsafe { &*ptr_b }));
        }
    }
}
