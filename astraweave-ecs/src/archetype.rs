use std::any::TypeId;
use std::collections::{BTreeMap, HashMap};

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
    /// Entities stored in this archetype (deterministic order via BTreeMap)
    pub entities: BTreeMap<Entity, usize>, // Entity -> row index
    /// Component columns: TypeId -> Vec<Box<dyn Any>>
    /// Each Vec has the same length as entities.len()
    pub components: HashMap<TypeId, Vec<Box<dyn std::any::Any + Send + Sync>>>,
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
            entities: BTreeMap::new(),
            components,
        }
    }

    /// Add an entity with its components (must match signature)
    pub fn add_entity(
        &mut self,
        entity: Entity,
        mut component_data: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
    ) {
        let row = self.entities.len();
        self.entities.insert(entity, row);

        for ty in &self.signature.components {
            if let Some(data) = component_data.remove(ty) {
                // Move the Box from component_data into the column
                let column = self.components.get_mut(ty).unwrap();
                column.push(data);
            }
        }
    }

    /// Get component for entity
    pub fn get<T: Component>(&self, entity: Entity) -> Option<&T> {
        let row = *self.entities.get(&entity)?;
        let column = self.components.get(&TypeId::of::<T>())?;
        let boxed = column.get(row)?;
        boxed.downcast_ref::<T>()
    }

    /// Get mutable component for entity
    pub fn get_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        let row = *self.entities.get(&entity)?;
        let column = self.components.get_mut(&TypeId::of::<T>())?;
        let boxed = column.get_mut(row)?;
        boxed.downcast_mut::<T>()
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Option<usize> {
        self.entities.remove(&entity)
    }

    /// Remove entity from archetype and return its components
    pub fn remove_entity_components(
        &mut self,
        entity: Entity,
    ) -> HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>> {
        let row = match self.entities.remove(&entity) {
            Some(r) => r,
            None => return HashMap::new(),
        };

        let mut components = HashMap::new();
        for (ty, column) in self.components.iter_mut() {
            let component = column.swap_remove(row);
            components.insert(*ty, component);
        }

        // Update the index of the entity that was swapped into the removed slot
        if row < self.entities.len() {
            // Find which entity was at the end and now is in the `row` slot
            let swapped_entity = self
                .entities
                .iter()
                .find(|(_, &r)| r == self.entities.len())
                .map(|(e, _)| *e);
            if let Some(swapped_entity) = swapped_entity {
                *self.entities.get_mut(&swapped_entity).unwrap() = row;
            }
        }
        components
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get a vector of entities in this archetype
    pub fn entities_vec(&self) -> Vec<Entity> {
        self.entities.keys().copied().collect()
    }
}

/// Manages all archetypes and entity->archetype mapping
#[derive(Default)]
pub struct ArchetypeStorage {
    next_id: u64,
    /// Map from signature to archetype ID
    signature_to_id: HashMap<ArchetypeSignature, ArchetypeId>,
    /// All archetypes
    archetypes: HashMap<ArchetypeId, Archetype>,
    /// Entity to archetype mapping
    entity_to_archetype: HashMap<Entity, ArchetypeId>,
}

impl ArchetypeStorage {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            signature_to_id: HashMap::new(),
            archetypes: HashMap::new(),
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

    /// Iterate mutably over all archetypes
    pub fn archetypes_mut(&mut self) -> impl Iterator<Item = &mut Archetype> {
        self.archetypes.values_mut()
    }

    /// Find archetypes that contain a specific component
    pub fn archetypes_with_component(&self, ty: TypeId) -> impl Iterator<Item = &Archetype> {
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
