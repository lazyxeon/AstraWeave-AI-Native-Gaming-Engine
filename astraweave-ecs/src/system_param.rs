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
            let archetype = self.world.archetypes.get_archetype(archetype_id).unwrap();

            if self.entity_idx >= archetype.len() {
                self.arch_idx += 1;
                self.entity_idx = 0;
                continue;
            }

            let entity = archetype.entities_vec()[self.entity_idx];
            self.entity_idx += 1;

            // The borrow checker needs help here. Since we are iterating over disjoint archetypes
            // and entities, this is safe. We'll use unsafe to extend the lifetime.
            let component = archetype.get::<T>(entity).unwrap();
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
            let archetype = self.world.archetypes.get_archetype(archetype_id).unwrap();

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
            let component_a = archetype.get::<A>(entity).unwrap();
            let component_b = archetype.get::<B>(entity).unwrap();
            let ptr_a = component_a as *const A;
            let ptr_b = component_b as *const B;

            return Some((entity, unsafe { &*ptr_a }, unsafe { &*ptr_b }));
        }
    }
}
