//! System parameter abstraction for ergonomic system signatures.
//!
//! This allows systems to declare dependencies like Query, Res, ResMut, Events
//! in a type-safe way similar to Bevy.

use std::marker::PhantomData;

use crate::{Component, Entity, Events, Resource, World};

/// Trait for types that can be system parameters
pub trait SystemParam: Sized {
    /// Fetch the parameter from the world
    fn fetch(world: &World) -> Option<Self>;
    
    /// Fetch mutable parameter from the world
    fn fetch_mut(world: &mut World) -> Option<Self>;
}

/// Query system parameter for iterating over entities with components
pub struct Query<'w, T> {
    entities: Vec<Entity>,
    world_ptr: *const World,
    _marker: PhantomData<(&'w (), T)>,
}

impl<'w, T: Component> Query<'w, T> {
    pub fn new(world: &'w World) -> Self {
        let entities = world.entities_with::<T>();
        Self {
            entities,
            world_ptr: world as *const World,
            _marker: PhantomData,
        }
    }

    /// Iterate over all entities with component T
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> + '_ {
        let world = unsafe { &*self.world_ptr };
        self.entities.iter().filter_map(move |&entity| {
            world.get::<T>(entity).map(|comp| (entity, comp))
        })
    }

    /// Get component for a specific entity
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let world = unsafe { &*self.world_ptr };
        world.get::<T>(entity)
    }

    /// Count entities matching this query
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

/// Mutable query system parameter
pub struct QueryMut<'w, T> {
    entities: Vec<Entity>,
    world_ptr: *mut World,
    _marker: PhantomData<(&'w mut (), T)>,
}

impl<'w, T: Component> QueryMut<'w, T> {
    pub fn new(world: &'w mut World) -> Self {
        let entities = world.entities_with::<T>();
        Self {
            entities,
            world_ptr: world as *mut World,
            _marker: PhantomData,
        }
    }

    /// Iterate mutably over all entities with component T
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> + '_ {
        let world = unsafe { &mut *self.world_ptr };
        let entities = std::mem::take(&mut self.entities);
        
        entities.into_iter().filter_map(move |entity| {
            world.get_mut::<T>(entity).map(|comp| (entity, comp))
        })
    }

    /// Get mutable component for a specific entity
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let world = unsafe { &mut *self.world_ptr };
        world.get_mut::<T>(entity)
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

/// Query for multiple components (tuple support)
pub struct QueryTuple<'w, A, B> {
    entities: Vec<Entity>,
    world_ptr: *const World,
    _marker: PhantomData<(&'w (), A, B)>,
}

impl<'w, A: Component, B: Component> QueryTuple<'w, A, B> {
    pub fn new(world: &'w World) -> Self {
        // Find entities that have both A and B
        let entities_a = world.entities_with::<A>();
        let entities: Vec<Entity> = entities_a
            .into_iter()
            .filter(|&e| world.has::<B>(e))
            .collect();
        
        Self {
            entities,
            world_ptr: world as *const World,
            _marker: PhantomData,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &A, &B)> + '_ {
        let world = unsafe { &*self.world_ptr };
        self.entities.iter().filter_map(move |&entity| {
            let a = world.get::<A>(entity)?;
            let b = world.get::<B>(entity)?;
            Some((entity, a, b))
        })
    }
}

/// Mutable query for multiple components
pub struct QueryTupleMut<'w, A, B> {
    entities: Vec<Entity>,
    world_ptr: *mut World,
    _marker: PhantomData<(&'w mut (), A, B)>,
}

impl<'w, A: Component, B: Component> QueryTupleMut<'w, A, B> {
    pub fn new(world: &'w mut World) -> Self {
        let entities_a = world.entities_with::<A>();
        let entities: Vec<Entity> = entities_a
            .into_iter()
            .filter(|&e| world.has::<B>(e))
            .collect();
        
        Self {
            entities,
            world_ptr: world as *mut World,
            _marker: PhantomData,
        }
    }

    /// SAFETY: This requires careful handling of mutable borrows
    /// We return owned entity IDs and defer the mutable borrow to iteration time
    pub fn iter_mut(&mut self) -> QueryTupleMutIter<'_, A, B> {
        QueryTupleMutIter {
            entities: self.entities.clone(),
            index: 0,
            world_ptr: self.world_ptr,
            _marker: PhantomData,
        }
    }
}

pub struct QueryTupleMutIter<'w, A, B> {
    entities: Vec<Entity>,
    index: usize,
    world_ptr: *mut World,
    _marker: PhantomData<(&'w mut (), A, B)>,
}

impl<'w, A: Component, B: Component> Iterator for QueryTupleMutIter<'w, A, B> {
    type Item = (Entity, &'w mut A, &'w mut B);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.entities.len() {
            let entity = self.entities[self.index];
            self.index += 1;

            let world = unsafe { &mut *self.world_ptr };
            
            // Get both mutable references
            // SAFETY: We guarantee no aliasing because each entity is visited once
            let a_ptr = world.get_mut::<A>(entity)? as *mut A;
            let b_ptr = world.get_mut::<B>(entity)? as *mut B;

            // Extend lifetimes (safe because we own the iteration)
            let a = unsafe { &mut *a_ptr };
            let b = unsafe { &mut *b_ptr };

            return Some((entity, a, b));
        }
        None
    }
}

/// Immutable resource parameter
pub struct Res<'w, T: Resource> {
    value: &'w T,
}

impl<'w, T: Resource> Res<'w, T> {
    pub fn new(world: &'w World) -> Option<Self> {
        world.get_resource::<T>().map(|value| Self { value })
    }
}

impl<'w, T: Resource> std::ops::Deref for Res<'w, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

/// Mutable resource parameter
pub struct ResMut<'w, T: Resource> {
    value: &'w mut T,
}

impl<'w, T: Resource> ResMut<'w, T> {
    pub fn new(world: &'w mut World) -> Option<Self> {
        world.get_resource_mut::<T>().map(|value| Self { value })
    }
}

impl<'w, T: Resource> std::ops::Deref for ResMut<'w, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'w, T: Resource> std::ops::DerefMut for ResMut<'w, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

/// Events parameter for reading/sending events
pub struct EventsParam<'w> {
    events: &'w mut Events,
}

impl<'w> EventsParam<'w> {
    pub fn new(world: &'w mut World) -> Option<Self> {
        world.get_resource_mut::<Events>().map(|events| Self { events })
    }

    pub fn send<E: crate::Event>(&mut self, event: E) {
        self.events.send(event);
    }

    pub fn read<E: crate::Event>(&self) -> impl Iterator<Item = &E> {
        self.events.read::<E>()
    }

    pub fn drain<E: crate::Event>(&mut self) -> impl Iterator<Item = E> + '_ {
        self.events.drain::<E>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Position(f32, f32);

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Velocity(f32, f32);

    #[test]
    fn test_query_iter() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        world.insert(e1, Position(1.0, 2.0));
        world.insert(e2, Position(3.0, 4.0));

        let query = Query::<Position>::new(&world);
        let positions: Vec<_> = query.iter().map(|(_, p)| *p).collect();

        assert_eq!(positions.len(), 2);
        assert!(positions.contains(&Position(1.0, 2.0)));
        assert!(positions.contains(&Position(3.0, 4.0)));
    }

    #[test]
    fn test_query_tuple() {
        let mut world = World::new();
        let e1 = world.spawn();
        let e2 = world.spawn();
        
        world.insert(e1, Position(1.0, 2.0));
        world.insert(e1, Velocity(0.5, 0.5));
        
        world.insert(e2, Position(3.0, 4.0));
        // e2 doesn't have Velocity

        let query = QueryTuple::<Position, Velocity>::new(&world);
        let count = query.iter().count();

        assert_eq!(count, 1); // Only e1 has both
    }

    #[derive(Debug, PartialEq)]
    struct TestResource {
        value: i32,
    }

    #[test]
    fn test_res_param() {
        let mut world = World::new();
        world.insert_resource(TestResource { value: 42 });

        let res = Res::<TestResource>::new(&world).unwrap();
        assert_eq!(res.value, 42);
    }

    #[test]
    fn test_res_mut_param() {
        let mut world = World::new();
        world.insert_resource(TestResource { value: 42 });

        let mut res = ResMut::<TestResource>::new(&mut world).unwrap();
        res.value = 100;

        assert_eq!(world.get_resource::<TestResource>().unwrap().value, 100);
    }
}
