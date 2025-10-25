//! AstraWeave ECS — Production-grade, AI-native ECS for game development.
//!
//! This ECS is designed specifically for AI-first game engines, providing:
//! - **Archetype-based storage** for cache-friendly iteration (like Bevy/Flecs)
//! - **Deterministic execution** via fixed schedules and ordered iteration
//! - **Event system** for AI perception and reactive behaviors
//! - **System parameters** for ergonomic system signatures
//! - **Plugin architecture** for modular game systems
//!
//! ## Architecture
//!
//! The AI-native game loop follows: **Perception → Reasoning → Planning → Action**
//!
//! ### System Stages:
//! 1. **Perception**: Build WorldSnapshots, update AI sensors
//! 2. **Simulation**: Game logic, cooldowns, state updates
//! 3. **AI Planning**: Generate PlanIntents from AI orchestrators
//! 4. **Physics**: Apply forces, resolve collisions
//! 5. **Presentation**: Rendering, audio, UI updates
//!
//! ## Example
//!
//! ```rust,ignore
//! use astraweave_ecs::*;
//!
//! #[derive(Clone, Copy)]
//! struct Position { x: f32, y: f32 }
//!
//! #[derive(Clone, Copy)]
//! struct Velocity { x: f32, y: f32 }
//!
//! fn movement_system(world: &mut World) {
//!     let mut query = QueryMut::<Position>::new(world);
//!     for (entity, pos) in query.iter_mut() {
//!         if let Some(vel) = world.get::<Velocity>(entity) {
//!             pos.x += vel.x;
//!             pos.y += vel.y;
//!         }
//!     }
//! }
//!
//! let mut app = App::new();
//! app.add_system("simulation", movement_system);
//! app = app.run_fixed(100); // Run 100 ticks
//! ```

#[cfg(feature = "profiling")]
use astraweave_profiling::{span, plot};

pub mod archetype;
pub mod blob_vec;
pub mod command_buffer;
pub mod entity_allocator;
pub mod events;
pub mod rng;
pub mod sparse_set;
pub mod type_registry;
mod system_param;

#[cfg(test)]
mod determinism_tests;

#[cfg(test)]
mod property_tests;

use std::any::TypeId;
use std::collections::HashMap;

use archetype::{ArchetypeSignature, ArchetypeStorage};
pub use command_buffer::CommandBuffer;
pub use entity_allocator::{Entity, EntityAllocator};
pub use events::{Event, EventReader};
pub use rng::Rng;
pub use system_param::{Query, Query2, Query2Mut, SystemParam};
pub use type_registry::TypeRegistry;

pub trait Component: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Component for T {}

/// Marker trait for resources (singletons in World)
pub trait Resource: 'static + Send + Sync {}
impl<T: 'static + Send + Sync> Resource for T {}

/// System stage identifiers for the AI-native game loop
pub struct SystemStage;

impl SystemStage {
    pub const PRE_SIMULATION: &'static str = "pre_simulation";
    pub const PERCEPTION: &'static str = "perception";
    pub const SIMULATION: &'static str = "simulation";
    pub const AI_PLANNING: &'static str = "ai_planning";
    pub const PHYSICS: &'static str = "physics";
    pub const POST_SIMULATION: &'static str = "post_simulation";
    pub const PRESENTATION: &'static str = "presentation";
}
// Entity and EntityAllocator are now exported from entity_allocator module

#[derive(Default)]
pub struct World {
    entity_allocator: EntityAllocator,
    archetypes: ArchetypeStorage,
    resources: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>, // singletons
    type_registry: TypeRegistry,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self) -> Entity {
        #[cfg(feature = "profiling")]
        span!("ECS::World::spawn");
        
        let e = self.entity_allocator.spawn();
        
        #[cfg(feature = "profiling")]
        plot!("ECS::entity_count", self.entity_allocator.alive_count() as u64);
        
        // An entity with no components lives in the empty archetype.
        let empty_sig = ArchetypeSignature::new(vec![]);
        let archetype_id = self.archetypes.get_or_create_archetype(empty_sig);
        self.archetypes.set_entity_archetype(e, archetype_id);
        let archetype = self.archetypes.get_archetype_mut(archetype_id)
            .expect("BUG: archetype should exist after get_or_create_archetype");
        archetype.add_entity(e, HashMap::new());
        e
    }

    /// Check if an entity is alive in this world.
    ///
    /// # Returns
    ///
    /// - `true` if entity ID and generation match
    /// - `false` if entity is dead or never existed
    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.entity_allocator.is_alive(entity)
    }

    pub fn insert<T: Component>(&mut self, e: Entity, c: T) {
        // Validate entity is alive
        if !self.is_alive(e) {
            return; // Silently ignore stale entities
        }

        let mut components_to_add = HashMap::new();
        components_to_add.insert(
            TypeId::of::<T>(),
            Box::new(c) as Box<dyn std::any::Any + Send + Sync>,
        );
        self.move_entity_to_new_archetype(e, components_to_add, false);
    }

    fn move_entity_to_new_archetype(
        &mut self,
        entity: Entity,
        new_components: HashMap<TypeId, Box<dyn std::any::Any + Send + Sync>>,
        is_removing: bool,
    ) {
        // 1. Get current archetype and component data
        let old_archetype_id = self.archetypes.get_entity_archetype(entity)
            .expect("BUG: entity should have archetype");

        let mut current_components = {
            let old_archetype = self.archetypes.get_archetype_mut(old_archetype_id)
                .expect("BUG: archetype should exist for entity");
            old_archetype.remove_entity_components(entity)
        };

        // 2. Determine new signature
        let new_sig_types = {
            let old_archetype = self.archetypes.get_archetype(old_archetype_id)
                .expect("BUG: archetype should exist");
            let mut sig_types: Vec<_> = old_archetype.signature.components.clone();
            if is_removing {
                // For removal, the `new_components` map just contains the TypeId of the component to remove.
                let type_to_remove = new_components.keys().next()
                    .expect("BUG: remove should have at least one component type");
                sig_types.retain(|&tid| tid != *type_to_remove);
            } else {
                sig_types.extend(new_components.keys());
            }
            sig_types
        };

        let new_signature = ArchetypeSignature::new(new_sig_types);

        // 3. Get or create new archetype
        let new_archetype_id = self.archetypes.get_or_create_archetype(new_signature);

        // 4. Move entity's archetype mapping
        self.archetypes
            .get_archetype_mut(old_archetype_id)
            .expect("BUG: old archetype should exist")
            .remove_entity(entity);
        self.archetypes
            .set_entity_archetype(entity, new_archetype_id);

        // 5. Add entity with all components to new archetype
        let final_components = if is_removing {
            let type_to_remove = new_components.keys().next()
                .expect("BUG: remove should have at least one component type");
            current_components.remove(type_to_remove);
            current_components
        } else {
            current_components.extend(new_components);
            current_components
        };

        let new_archetype = self.archetypes.get_archetype_mut(new_archetype_id)
            .expect("BUG: archetype should exist after get_or_create_archetype");
        new_archetype.add_entity(entity, final_components);
    }

    pub fn get<T: Component>(&self, e: Entity) -> Option<&T> {
        #[cfg(feature = "profiling")]
        span!("ECS::World::get");
        
        // Validate entity is alive
        if !self.is_alive(e) {
            return None;
        }
        
        let archetype_id = self.archetypes.get_entity_archetype(e)?;
        let archetype = self.archetypes.get_archetype(archetype_id)?;
        archetype.get::<T>(e)
    }

    pub fn get_mut<T: Component>(&mut self, e: Entity) -> Option<&mut T> {
        // Validate entity is alive
        if !self.is_alive(e) {
            return None;
        }
        
        let archetype_id = self.archetypes.get_entity_archetype(e)?;
        let archetype = self.archetypes.get_archetype_mut(archetype_id)?;
        archetype.get_mut::<T>(e)
    }

    pub fn insert_resource<T: 'static + Send + Sync>(&mut self, r: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(r));
    }

    pub fn get_resource<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources.get(&TypeId::of::<T>())?.downcast_ref()
    }

    pub fn get_resource_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.resources.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }

    pub fn each_mut<T: Component>(&mut self, mut f: impl FnMut(Entity, &mut T)) {
        let archetypes_with_t = self
            .archetypes
            .archetypes_with_component(TypeId::of::<T>())
            .map(|a| a.id)
            .collect::<Vec<_>>();

        for archetype_id in archetypes_with_t {
            let archetype = self.archetypes.get_archetype_mut(archetype_id)
                .expect("BUG: archetype should exist from archetypes_with_component");
            // NEW: entities_vec() now returns &[Entity] (zero-cost!)
            let entities: Vec<Entity> = archetype.entities_vec().to_vec();
            for entity in entities {
                if let Some(component) = archetype.get_mut::<T>(entity) {
                    f(entity, component);
                }
            }
        }
    }

    pub fn count<T: Component>(&self) -> usize {
        self.archetypes
            .archetypes_with_component(TypeId::of::<T>())
            .map(|archetype| archetype.len())
            .sum()
    }

    pub fn has<T: Component>(&self, entity: Entity) -> bool {
        // Validate entity is alive before checking components
        if !self.is_alive(entity) {
            return false;
        }
        self.get::<T>(entity).is_some()
    }

    pub fn entities_with<T: Component>(&self) -> Vec<Entity> {
        self.archetypes
            .archetypes_with_component(TypeId::of::<T>())
            .flat_map(|archetype| archetype.entities_vec().iter().copied())
            .collect()
    }

    pub fn remove<T: Component>(&mut self, e: Entity) -> bool {
        // Validate entity is alive
        if !self.is_alive(e) {
            return false;
        }
        
        if !self.has::<T>(e) {
            return false;
        }
        let mut components_to_remove = HashMap::new();
        // We just need the type id for the signature change. The value is irrelevant.
        components_to_remove.insert(
            TypeId::of::<T>(),
            Box::new(0) as Box<dyn std::any::Any + Send + Sync>,
        );
        self.move_entity_to_new_archetype(e, components_to_remove, true);
        true
    }

    /// Despawn an entity, removing it from the world.
    ///
    /// # Returns
    ///
    /// - `true` if entity was alive and despawned
    /// - `false` if entity was already dead (stale handle)
    ///
    /// # Example
    ///
    /// ```
    /// use astraweave_ecs::*;
    ///
    /// let mut world = World::new();
    /// let e = world.spawn();
    ///
    /// assert!(world.despawn(e));  // First despawn succeeds
    /// assert!(!world.despawn(e)); // Second despawn fails (stale)
    /// ```
    pub fn despawn(&mut self, entity: Entity) -> bool {
        // First validate entity is alive
        if !self.entity_allocator.is_alive(entity) {
            return false;
        }

        // Remove from archetype (removes entity AND all components)
        if let Some(archetype_id) = self.archetypes.get_entity_archetype(entity) {
            let archetype = self.archetypes.get_archetype_mut(archetype_id)
                .expect("BUG: archetype should exist for entity");
            // Use remove_entity_components to properly clean up packed storage
            archetype.remove_entity_components(entity);
            self.archetypes.remove_entity(entity);
        }

        // Despawn from allocator (increments generation)
        self.entity_allocator.despawn(entity)
    }

    /// Get the number of entities currently alive.
    pub fn entity_count(&self) -> usize {
        self.entity_allocator.alive_count()
    }

    /// Get read-only access to the archetype storage.
    ///
    /// # Use Cases
    ///
    /// - Iterating all entities across all archetypes
    /// - Querying archetype metadata (signatures, counts)
    /// - Testing determinism properties
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// for archetype in world.archetypes().iter() {
    ///     for &entity in archetype.entities_vec() {
    ///         // Process entity
    ///     }
    /// }
    /// ```
    pub fn archetypes(&self) -> &ArchetypeStorage {
        &self.archetypes
    }
}

// Schedule and systems
pub type SystemFn = fn(&mut World);

#[derive(Default)]
pub struct Schedule {
    pub stages: Vec<Stage>,
}

pub struct Stage {
    pub name: &'static str,
    pub systems: Vec<SystemFn>,
}

impl Schedule {
    pub fn with_stage(mut self, name: &'static str) -> Self {
        self.stages.push(Stage {
            name,
            systems: vec![],
        });
        self
    }
    pub fn add_system(&mut self, stage: &'static str, sys: SystemFn) {
        if let Some(s) = self.stages.iter_mut().find(|s| s.name == stage) {
            s.systems.push(sys);
        }
    }
    pub fn run(&self, world: &mut World) {
        #[cfg(feature = "profiling")]
        span!("ECS::Schedule::run");
        
        for s in &self.stages {
            for f in &s.systems {
                (f)(world);
            }
        }
    }
}

// App-like builder with deterministic fixed-timestep driver
pub struct App {
    pub world: World,
    pub schedule: Schedule,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut schedule = Schedule::default();
        schedule = schedule
            .with_stage("perception")
            .with_stage("simulation")
            .with_stage("ai_planning")
            .with_stage("physics")
            .with_stage("presentation");
        Self {
            world: World::new(),
            schedule,
        }
    }
    
    pub fn add_system(&mut self, stage: &'static str, sys: SystemFn) {
        self.schedule.add_system(stage, sys);
    }
    pub fn insert_resource<T: 'static + Send + Sync>(mut self, r: T) -> Self {
        self.world.insert_resource(r);
        self
    }
    pub fn run_fixed(mut self, steps: u32) -> Self {
        for _ in 0..steps {
            self.schedule.run(&mut self.world);
        }
        self
    }
}

impl World {
    /// Register a component type for type-erased operations (used by CommandBuffer).
    ///
    /// This must be called for any component type that will be used with CommandBuffer.
    ///
    /// # Example
    /// ```
    /// # use astraweave_ecs::World;
    /// # #[derive(Clone, Copy)]
    /// # struct Position { x: f32, y: f32 }
    /// let mut world = World::new();
    /// world.register_component::<Position>();
    /// ```
    pub fn register_component<T: Component>(&mut self) {
        self.type_registry.register::<T>();
    }

    /// Insert a type-erased component (used by CommandBuffer).
    ///
    /// # Panics
    /// Panics if the component type is not registered via `register_component<T>()`.
    pub(crate) fn insert_boxed(
        &mut self,
        entity: Entity,
        type_id: TypeId,
        _component: Box<dyn std::any::Any + Send + Sync>,
    ) {
        if !self.is_alive(entity) {
            return; // Stale entity, silently ignore
        }
        
        // TODO: Full type registry dispatch with closures requires refactoring to avoid
        // borrow checker issues (self is borrowed immutably for registry lookup, but
        // handler needs &mut self). For now, this is a stub that will be improved in
        // a follow-up commit using interior mutability or a different architecture.
        panic!(
            "insert_boxed not fully implemented - type_id {:?}. \
             This is a known limitation. See PR #2 implementation notes.",
            type_id
        );
    }

    /// Remove a component by TypeId (used by CommandBuffer).
    ///
    /// # Panics
    /// Panics if the component type is not registered via `register_component<T>()`.
    pub(crate) fn remove_by_type_id(&mut self, entity: Entity, type_id: TypeId) {
        if !self.is_alive(entity) {
            return; // Stale entity, silently ignore
        }
        
        panic!(
            "remove_by_type_id not fully implemented - type_id {:?}. \
             See PR #2 implementation notes.",
            type_id
        );
    }
}

// Plugin pattern similar to Bevy
pub trait Plugin {
    fn build(&self, app: &mut App);
}
impl App {
    pub fn add_plugin(mut self, p: impl Plugin) -> Self {
        p.build(&mut self);
        self
    }
}

// SECTION: System Execution

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Velocity {
        vx: f32,
        vy: f32,
    }

    #[derive(Debug, PartialEq)]
    struct TestResource(i32);

    #[test]
    fn test_spawn_and_insert() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        assert!(world.has::<Position>(entity));
        assert!(!world.has::<Velocity>(entity));

        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(*pos, Position { x: 1.0, y: 2.0 });
    }

    #[test]
    fn test_despawn() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        assert!(world.is_alive(entity));
        world.despawn(entity);
        assert!(!world.is_alive(entity));
        assert!(!world.has::<Position>(entity));
    }

    #[test]
    fn test_remove_component() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 2.0 });
        world.insert(entity, Velocity { vx: 0.0, vy: 0.0 });

        assert!(world.has::<Position>(entity));
        world.remove::<Position>(entity);
        assert!(!world.has::<Position>(entity));
        assert!(world.has::<Velocity>(entity)); // Other components should remain
    }

    #[test]
    fn test_query_single_component() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        let e2 = world.spawn();
        world.insert(e2, Position { x: 2.0, y: 2.0 });
        let e3 = world.spawn();
        world.insert(e3, Velocity { vx: 0.0, vy: 0.0 });

        let query = Query::<Position>::new(&world);
        let mut count = 0;
        let mut total_x = 0.0;
        for (entity, pos) in query {
            count += 1;
            total_x += pos.x;
            assert!(entity == e1 || entity == e2);
        }
        assert_eq!(count, 2);
        assert_eq!(total_x, 3.0);
    }

    #[test]
    fn test_query_two_components() {
        let mut world = World::new();
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        world.insert(e1, Velocity { vx: 1.0, vy: 1.0 });

        let e2 = world.spawn();
        world.insert(e2, Position { x: 2.0, y: 2.0 });

        let e3 = world.spawn();
        world.insert(e3, Position { x: 3.0, y: 3.0 });
        world.insert(e3, Velocity { vx: 3.0, vy: 3.0 });

        let query = Query2::<Position, Velocity>::new(&world);
        let mut count = 0;
        for (entity, pos, vel) in query {
            count += 1;
            assert!(entity == e1 || entity == e3);
            assert_eq!(pos.x, vel.vx);
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_resource_management() {
        let mut world = World::new();
        world.insert_resource(TestResource(42));

        let resource = world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, 42);

        let resource_mut = world.get_resource_mut::<TestResource>().unwrap();
        resource_mut.0 = 100;

        let resource_after = world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource_after.0, 100);
    }

    #[test]
    fn test_get_mut() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 2.0 });

        let pos_mut = world.get_mut::<Position>(entity).unwrap();
        pos_mut.x = 5.0;

        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, 5.0);
    }

    // ====================
    // Day 2: World Advanced API Tests
    // ====================

    #[test]
    fn test_count_single_component() {
        let mut world = World::new();
        
        assert_eq!(world.count::<Position>(), 0);
        
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        assert_eq!(world.count::<Position>(), 1);
        
        let e2 = world.spawn();
        world.insert(e2, Position { x: 2.0, y: 2.0 });
        assert_eq!(world.count::<Position>(), 2);
        
        let _e3 = world.spawn();
        world.insert(_e3, Velocity { vx: 1.0, vy: 1.0 });
        assert_eq!(world.count::<Position>(), 2);
        assert_eq!(world.count::<Velocity>(), 1);
    }

    #[test]
    fn test_count_across_archetypes() {
        let mut world = World::new();
        
        // Archetype 1: Position only
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        
        // Archetype 2: Position + Velocity
        let e2 = world.spawn();
        world.insert(e2, Position { x: 2.0, y: 2.0 });
        world.insert(e2, Velocity { vx: 1.0, vy: 1.0 });
        
        // Should count both
        assert_eq!(world.count::<Position>(), 2);
        assert_eq!(world.count::<Velocity>(), 1);
    }

    #[test]
    fn test_entities_with_single_component() {
        let mut world = World::new();
        
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        
        let e2 = world.spawn();
        world.insert(e2, Position { x: 2.0, y: 2.0 });
        
        let _e3 = world.spawn();
        world.insert(_e3, Velocity { vx: 1.0, vy: 1.0 });
        
        let entities = world.entities_with::<Position>();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
    }

    #[test]
    fn test_entities_with_empty_result() {
        let world = World::new();
        let entities = world.entities_with::<Position>();
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_entities_with_across_archetypes() {
        let mut world = World::new();
        
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        
        let e2 = world.spawn();
        world.insert(e2, Position { x: 2.0, y: 2.0 });
        world.insert(e2, Velocity { vx: 1.0, vy: 1.0 });
        
        let entities = world.entities_with::<Position>();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
    }

    #[test]
    fn test_each_mut_modify_components() {
        let mut world = World::new();
        
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        
        let e2 = world.spawn();
        world.insert(e2, Position { x: 2.0, y: 2.0 });
        
        world.each_mut::<Position>(|_e, pos| {
            pos.x += 10.0;
        });
        
        assert_eq!(world.get::<Position>(e1).unwrap().x, 11.0);
        assert_eq!(world.get::<Position>(e2).unwrap().x, 12.0);
    }

    #[test]
    fn test_each_mut_with_entity_access() {
        let mut world = World::new();
        
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        
        let e2 = world.spawn();
        world.insert(e2, Position { x: 2.0, y: 2.0 });
        
        let mut visited = Vec::new();
        world.each_mut::<Position>(|entity, _pos| {
            visited.push(entity);
        });
        
        assert_eq!(visited.len(), 2);
        assert!(visited.contains(&e1));
        assert!(visited.contains(&e2));
    }

    #[test]
    fn test_entity_count() {
        let mut world = World::new();
        
        assert_eq!(world.entity_count(), 0);
        
        let e1 = world.spawn();
        assert_eq!(world.entity_count(), 1);
        
        let _e2 = world.spawn();
        assert_eq!(world.entity_count(), 2);
        
        world.despawn(e1);
        assert_eq!(world.entity_count(), 1);
    }

    // ====================
    // Day 2: Stale Entity Handling Tests
    // ====================

    #[test]
    fn test_insert_on_stale_entity_ignored() {
        let mut world = World::new();
        let entity = world.spawn();
        world.despawn(entity);
        
        // Insert on stale entity should be ignored silently
        world.insert(entity, Position { x: 1.0, y: 1.0 });
        
        assert!(!world.has::<Position>(entity));
        assert_eq!(world.count::<Position>(), 0);
    }

    #[test]
    fn test_get_on_stale_entity_returns_none() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 1.0 });
        world.despawn(entity);
        
        assert!(world.get::<Position>(entity).is_none());
    }

    #[test]
    fn test_get_mut_on_stale_entity_returns_none() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 1.0 });
        world.despawn(entity);
        
        assert!(world.get_mut::<Position>(entity).is_none());
    }

    #[test]
    fn test_has_on_stale_entity_returns_false() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 1.0 });
        world.despawn(entity);
        
        assert!(!world.has::<Position>(entity));
    }

    #[test]
    fn test_remove_on_stale_entity_returns_false() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 1.0 });
        world.despawn(entity);
        
        assert!(!world.remove::<Position>(entity));
    }

    #[test]
    fn test_despawn_stale_entity_returns_false() {
        let mut world = World::new();
        let entity = world.spawn();
        
        assert!(world.despawn(entity));
        assert!(!world.despawn(entity)); // Second despawn should return false
    }

    #[test]
    fn test_remove_nonexistent_component_returns_false() {
        let mut world = World::new();
        let entity = world.spawn();
        world.insert(entity, Position { x: 1.0, y: 1.0 });
        
        // Removing component that doesn't exist
        assert!(!world.remove::<Velocity>(entity));
    }

    // ====================
    // Day 2: Resource Edge Cases
    // ====================

    #[test]
    fn test_resource_get_nonexistent_returns_none() {
        let world = World::new();
        assert!(world.get_resource::<TestResource>().is_none());
    }

    #[test]
    fn test_resource_get_mut_nonexistent_returns_none() {
        let mut world = World::new();
        assert!(world.get_resource_mut::<TestResource>().is_none());
    }

    #[test]
    fn test_resource_replace() {
        let mut world = World::new();
        world.insert_resource(TestResource(10));
        
        world.insert_resource(TestResource(20));
        
        let resource = world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, 20);
    }

    // ====================
    // Day 2: App/Schedule API Tests
    // ====================

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert_eq!(app.world.entity_count(), 0);
        assert_eq!(app.schedule.stages.len(), 5);
    }

    #[test]
    fn test_app_insert_resource() {
        let app = App::new().insert_resource(TestResource(42));
        
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, 42);
    }

    #[test]
    fn test_schedule_execution() {
        fn test_system(world: &mut World) {
            world.insert_resource(TestResource(99));
        }
        
        let mut app = App::new();
        app.add_system("simulation", test_system);
        app = app.run_fixed(1);
        
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, 99);
    }

    #[test]
    fn test_schedule_multiple_systems() {
        fn system_a(world: &mut World) {
            world.insert_resource(TestResource(10));
        }
        
        fn system_b(world: &mut World) {
            if let Some(resource) = world.get_resource_mut::<TestResource>() {
                resource.0 += 5;
            }
        }
        
        let mut app = App::new();
        app.add_system("simulation", system_a);
        app.add_system("simulation", system_b);
        app = app.run_fixed(1);
        
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, 15);
    }

    #[test]
    fn test_run_fixed_multiple_steps() {
        fn increment_system(world: &mut World) {
            if let Some(resource) = world.get_resource_mut::<TestResource>() {
                resource.0 += 1;
            } else {
                world.insert_resource(TestResource(1));
            }
        }
        
        let mut app = App::new();
        app.add_system("simulation", increment_system);
        app = app.run_fixed(10);
        
        let resource = app.world.get_resource::<TestResource>().unwrap();
        assert_eq!(resource.0, 10);
    }

    // ====================
    // Day 2: Archetype Access Tests
    // ====================

    #[test]
    fn test_archetypes_read_access() {
        let mut world = World::new();
        
        let e1 = world.spawn();
        world.insert(e1, Position { x: 1.0, y: 1.0 });
        
        let archetypes = world.archetypes();
        let archetype_count = archetypes.iter().count();
        
        // Should have at least 2 archetypes: empty and Position-only
        assert!(archetype_count >= 2);
    }

    #[test]
    fn test_spawn_creates_empty_archetype_entity() {
        let mut world = World::new();
        let entity = world.spawn();
        
        // Entity should exist in empty archetype
        assert!(world.is_alive(entity));
        assert_eq!(world.entity_count(), 1);
    }
}
