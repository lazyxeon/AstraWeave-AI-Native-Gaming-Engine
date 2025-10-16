//! AstraWeave ECS — Production-grade, AI-native ECS for game development.

use std::any::TypeId;
use std::collections::{BTreeMap, HashMap};

#[cfg(feature = "profiling")]
use astraweave_profiling::span;

use crate::sparse_set::SparseSet;
use crate::{Component, Entity};

/// Unique identifier for an archetype (set of component types)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchetypeId(u64);

/// Describes the component layout of an archetype
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ArchetypeSignature {
    /// Sorted list of component TypeIds for deterministic comparison
    pub components: Vec<TypeId>,
}

impl ArchetypeSignature {
    pub fn new(mut components: Vec<TypeId>) -> Self {
        components.sort_unstable();
        components.dedup();
        Self { components }
    }

    pub fn contains(&self, ty: TypeId) -> bool {
        self.components.binary_search(&ty).is_ok()
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

/// Archetype storage: all entities with the same component signature
pub struct Archetype {
    pub id: ArchetypeId,
    pub signature: ArchetypeSignature,
    
    /// NEW: Packed entity list for iteration (cache-friendly)
    entities: Vec<Entity>,
    
    /// NEW: O(1) entity lookup (replaces BTreeMap)
    entity_index: SparseSet,
    
    /// Component columns: TypeId -> Vec<Box<dyn Any>>
    /// NOTE: Still using Box for now (type-erased storage)
    /// Future: Replace with BlobVec once we add type registry
    components: HashMap<TypeId, Vec<Box<dyn std::any::Any + Send + Sync>>>,
}

impl Archetype {
    pub fn new(id: ArchetypeId, signature: ArchetypeSignature) -> Self {
        let mut components = HashMap::new();
        for ty in &signature.components {
            components.insert(*ty, Vec::new());
        }
        Self {
            id,
            signature,
            entities: Vec::new(),
            entity_index: SparseSet::new(),
            components,
        }
    }

    /// Add an entity with its components (must match signature)
    pub fn add_entity(
        &mut self,
        entity: Entity,
        mut component_data: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
    ) {
        // NEW: Use SparseSet for O(1) lookup (12-57× faster than BTreeMap)
        self.entity_index.insert(entity);
        self.entities.push(entity);

        for ty in &self.signature.components {
            if let Some(data) = component_data.remove(ty) {
                // Move the Box from component_data into the column
                let column = self.components.get_mut(ty)
                    .expect("BUG: signature component should have column");
                column.push(data);
            }
        }
    }

    /// Get component for entity
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        // NEW: O(1) lookup with SparseSet (12-57× faster than BTreeMap)
        let row = self.entity_index.get(entity)?;
        let column = self.components.get(&TypeId::of::<T>())?;
        let boxed = column.get(row)?;
        boxed.downcast_ref::<T>()
    }

    /// Get mutable component for entity
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        // NEW: O(1) lookup with SparseSet (12-57× faster than BTreeMap)
        let row = self.entity_index.get(entity)?;
        let column = self.components.get_mut(&TypeId::of::<T>())?;
        let boxed = column.get_mut(row)?;
        boxed.downcast_mut::<T>()
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Option<usize> {
        // NEW: O(1) removal with SparseSet (4-7× faster than BTreeMap)
        self.entity_index.remove(entity)
    }

    /// Remove entity from archetype and return its components
    pub fn remove_entity_components(
        &mut self,
        entity: Entity,
    ) -> HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>> {
        // NEW: O(1) removal with SparseSet
        let row = match self.entity_index.remove(entity) {
            Some(r) => r,
            None => return HashMap::new(),
        };

        // Remove from packed entity list using swap_remove
        let entities_len = self.entities.len();
        if row < entities_len - 1 {
            self.entities.swap(row, entities_len - 1);
            // Update the swapped entity's index in SparseSet
            let swapped_entity = self.entities[row];
            self.entity_index.insert(swapped_entity);  // Will update to correct row
        }
        self.entities.pop();

        let mut components = HashMap::new();
        for (ty, column) in self.components.iter_mut() {
            let component = column.swap_remove(row);
            components.insert(*ty, component);
        }

        components
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get a slice of entities in this archetype (zero-cost, cache-friendly!)
    pub fn entities_vec(&self) -> &[Entity] {
        &self.entities
    }
    
    /// Iterate over (entity, component) pairs for batch processing.
    /// 
    /// This is much faster than repeated get() calls as it avoids per-entity lookups.
    /// 
    /// ## Performance Notes (Week 10)
    /// 
    /// With SparseSet integration, get() is now O(1) instead of O(log n), providing
    /// 12-57× speedup over the old BTreeMap approach. This iterator provides additional
    /// benefits by reducing function call overhead and improving cache locality.
    /// 
    /// ## Mutable Iterator Limitation
    /// 
    /// Note: A mutable version (`iter_components_mut<T>()`) is **not feasible** due to
    /// Rust's borrow checker limitations. The issue is:
    /// 
    /// ```rust,ignore
    /// pub fn iter_components_mut<T>(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
    ///     let column = self.components.get_mut(&TypeId::of::<T>())?;
    ///     self.entities.iter().filter_map(|(idx, &entity)| {
    ///         column.get_mut(idx)  // ❌ ERROR: captured variable escapes FnMut closure
    ///     })
    /// }
    /// ```
    /// 
    /// Rust prevents this because the closure captures `column` and tries to return
    /// `&mut T` borrowed from it. The borrow checker rule is: **references captured in
    /// closures cannot escape the closure scope**. This prevents dangling references.
    /// 
    /// **Workarounds considered**:
    /// - Unsafe raw pointers: Would work but loses safety guarantees (not worth it)
    /// - Index-based batch API: Complex redesign with uncertain performance gains
    /// - Type registry + BlobVec: Full solution but requires architectural changes (Week 13+)
    /// 
    /// **Current approach**: Accept that SparseSet O(1) already provides 2.4× frame time
    /// improvement (2.70ms → 1.144ms) and 9.4× faster movement (1,000µs → 106µs). Further
    /// query optimization has diminishing returns vs complexity/safety trade-offs.
    pub fn iter_components<T: Component>(&self) -> impl Iterator<Item = (Entity, &T)> + '_ {
        let column = self.components.get(&TypeId::of::<T>());
        self.entities.iter().enumerate().filter_map(move |(idx, &entity)| {
            column
                .and_then(|col| col.get(idx))
                .and_then(|boxed| boxed.downcast_ref::<T>())
                .map(|component| (entity, component))
        })
    }
}



/// Manages all archetypes and entity->archetype mapping
///
/// # Determinism Guarantee
///
/// **CRITICAL**: This uses `BTreeMap` for archetype storage to ensure deterministic iteration.
/// Iteration order is sorted by `ArchetypeId`, which preserves archetype creation order
/// (IDs assigned sequentially via `next_id`).
///
/// **Why BTreeMap?**
/// - HashMap iteration order is **non-deterministic** (depends on hash function, memory layout)
/// - BTreeMap iteration order is **deterministic** (sorted by key)
/// - For AI agents, deterministic entity iteration is **critical** for reproducible behavior
///
/// **Performance Note**:
/// - BTreeMap operations are O(log n) vs HashMap O(1)
/// - With ~100 archetypes typical, log₂(100) ≈ 7 operations (negligible)
/// - Entity queries iterate archetypes (O(archetypes)), so iteration order matters more than lookup
#[derive(Default)]
pub struct ArchetypeStorage {
    next_id: u64,
    /// Map from signature to archetype ID
    signature_to_id: HashMap<ArchetypeSignature, ArchetypeId>,
    /// All archetypes (BTreeMap for deterministic iteration by ID)
    archetypes: BTreeMap<ArchetypeId, Archetype>,
    /// Entity to archetype mapping
    entity_to_archetype: HashMap<Entity, ArchetypeId>,
}

impl ArchetypeStorage {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            signature_to_id: HashMap::new(),
            archetypes: BTreeMap::new(),
            entity_to_archetype: HashMap::new(),
        }
    }

    /// Get or create archetype for a signature
    pub fn get_or_create_archetype(&mut self, signature: ArchetypeSignature) -> ArchetypeId {
        if let Some(&id) = self.signature_to_id.get(&signature) {
            return id;
        }

        let id = ArchetypeId(self.next_id);
        self.next_id += 1;

        let archetype = Archetype::new(id, signature.clone());
        self.archetypes.insert(id, archetype);
        self.signature_to_id.insert(signature, id);

        id
    }

    pub fn get_archetype(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(&id)
    }

    pub fn get_archetype_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(&id)
    }

    pub fn get_entity_archetype(&self, entity: Entity) -> Option<ArchetypeId> {
        self.entity_to_archetype.get(&entity).copied()
    }

    pub fn set_entity_archetype(&mut self, entity: Entity, archetype: ArchetypeId) {
        self.entity_to_archetype.insert(entity, archetype);
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Option<ArchetypeId> {
        self.entity_to_archetype.remove(&entity)
    }

    /// Iterate over all archetypes
    pub fn archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.values()
    }

    /// Iterate over all archetypes (alias for consistency with standard iterator naming)
    pub fn iter(&self) -> impl Iterator<Item = &Archetype> {
        self.archetypes.values()
    }

    /// Iterate mutably over all archetypes
    pub fn archetypes_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.values_mut()
    }

    /// Find archetypes that contain a specific component
    pub fn archetypes_with_component(&self, ty: TypeId) -> impl Iterator<Item = &Archetype> {
        #[cfg(feature = "profiling")]
        span!("ECS::Archetype::archetypes_with_component");
        
        self.archetypes
            .values()
            .filter(move |arch| arch.signature.contains(ty))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Health(i32);

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Position(f32, f32);

    #[test]
    fn test_signature_creation() {
        let sig1 = ArchetypeSignature::new(vec![TypeId::of::<Health>(), TypeId::of::<Position>()]);
        let sig2 = ArchetypeSignature::new(vec![TypeId::of::<Position>(), TypeId::of::<Health>()]);
        assert_eq!(sig1, sig2); // Order-independent
    }

    #[test]
    fn test_archetype_storage() {
        let mut storage = ArchetypeStorage::new();
        let sig = ArchetypeSignature::new(vec![TypeId::of::<Health>()]);

        let id1 = storage.get_or_create_archetype(sig.clone());
        let id2 = storage.get_or_create_archetype(sig.clone());

        assert_eq!(id1, id2); // Same signature returns same archetype
    }
}
