//! Type registry for dynamic component operations.
//!
//! Provides runtime type information and handlers for type-erased component
//! operations (insert, remove, drop). Used by CommandBuffer for deferred operations.

use crate::Component;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Handler for inserting type-erased components into World.
type InsertHandler = Box<dyn Fn(&mut crate::World, crate::Entity, Box<dyn Any + Send + Sync>) + Send + Sync>;

/// Handler for removing type-erased components from World.
type RemoveHandler = Box<dyn Fn(&mut crate::World, crate::Entity) + Send + Sync>;

/// Registry of component types and their handlers.
///
/// Allows CommandBuffer to perform type-erased operations (insert/remove)
/// without knowing concrete types at runtime.
pub struct TypeRegistry {
    pub(crate) insert_handlers: HashMap<TypeId, InsertHandler>,
    pub(crate) remove_handlers: HashMap<TypeId, RemoveHandler>,
    pub(crate) type_names: HashMap<TypeId, &'static str>,
}

impl TypeRegistry {
    /// Create a new empty type registry.
    pub fn new() -> Self {
        Self {
            insert_handlers: HashMap::new(),
            remove_handlers: HashMap::new(),
            type_names: HashMap::new(),
        }
    }

    /// Register a component type with insert/remove handlers.
    ///
    /// This allows CommandBuffer to perform operations on this type via TypeId.
    ///
    /// # Example
    /// ```
    /// # use astraweave_ecs::{World, TypeRegistry};
    /// # #[derive(Clone, Copy)]
    /// # struct Position { x: f32, y: f32 }
    /// let mut registry = TypeRegistry::new();
    /// registry.register::<Position>();
    /// ```
    pub fn register<T: Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        let type_name = std::any::type_name::<T>();

        // Insert handler: Downcast Box<dyn Any> → T, then call World::insert
        self.insert_handlers.insert(
            type_id,
            Box::new(
                |world: &mut crate::World, entity: crate::Entity, component: Box<dyn Any + Send + Sync>| {
                    if let Ok(component) = component.downcast::<T>() {
                        world.insert(entity, *component);
                    } else {
                        panic!(
                            "TypeRegistry: insert handler called with wrong type (expected {})",
                            std::any::type_name::<T>()
                        );
                    }
                },
            ),
        );

        // Remove handler: Call World::remove<T>
        self.remove_handlers.insert(
            type_id,
            Box::new(|world: &mut crate::World, entity: crate::Entity| {
                world.remove::<T>(entity);
            }),
        );

        self.type_names.insert(type_id, type_name);
    }

    /// Insert a type-erased component using registered handler.
    ///
    /// # Panics
    /// Panics if the type is not registered or if downcast fails.
    pub fn insert_boxed(
        &self,
        world: &mut crate::World,
        entity: crate::Entity,
        type_id: TypeId,
        component: Box<dyn Any + Send + Sync>,
    ) {
        if let Some(handler) = self.insert_handlers.get(&type_id) {
            handler(world, entity, component);
        } else {
            panic!(
                "TypeRegistry: type {:?} not registered (call register::<T>() first)",
                self.type_names.get(&type_id).unwrap_or(&"<unknown>")
            );
        }
    }

    /// Remove a component by TypeId using registered handler.
    ///
    /// # Panics
    /// Panics if the type is not registered.
    pub fn remove_by_type_id(
        &self,
        world: &mut crate::World,
        entity: crate::Entity,
        type_id: TypeId,
    ) {
        if let Some(handler) = self.remove_handlers.get(&type_id) {
            handler(world, entity);
        } else {
            panic!(
                "TypeRegistry: type {:?} not registered (call register::<T>() first)",
                self.type_names.get(&type_id).unwrap_or(&"<unknown>")
            );
        }
    }

    /// Check if a type is registered.
    pub fn is_registered(&self, type_id: TypeId) -> bool {
        self.insert_handlers.contains_key(&type_id)
    }

    /// Get the name of a registered type.
    pub fn type_name(&self, type_id: TypeId) -> Option<&'static str> {
        self.type_names.get(&type_id).copied()
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;

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

    #[test]
    fn test_type_registry_creation() {
        let registry = TypeRegistry::new();
        assert!(!registry.is_registered(TypeId::of::<Position>()));
    }

    #[test]
    fn test_register_type() {
        let mut registry = TypeRegistry::new();
        registry.register::<Position>();

        assert!(registry.is_registered(TypeId::of::<Position>()));
        assert_eq!(
            registry.type_name(TypeId::of::<Position>()),
            Some("astraweave_ecs::type_registry::tests::Position")
        );
    }

    #[test]
    fn test_insert_boxed() {
        let mut world = World::new();
        let mut registry = TypeRegistry::new();
        registry.register::<Position>();

        let entity = world.spawn();
        let component = Box::new(Position { x: 10.0, y: 20.0 });

        registry.insert_boxed(&mut world, entity, TypeId::of::<Position>(), component);

        assert_eq!(
            world.get::<Position>(entity),
            Some(&Position { x: 10.0, y: 20.0 })
        );
    }

    #[test]
    fn test_remove_by_type_id() {
        let mut world = World::new();
        let mut registry = TypeRegistry::new();
        registry.register::<Position>();

        let entity = world.spawn();
        world.insert(entity, Position { x: 10.0, y: 20.0 });

        assert!(world.has::<Position>(entity));

        registry.remove_by_type_id(&mut world, entity, TypeId::of::<Position>());

        assert!(!world.has::<Position>(entity));
    }

    #[test]
    #[should_panic(expected = "type")]
    fn test_insert_unregistered_type() {
        let mut world = World::new();
        let registry = TypeRegistry::new();

        let entity = world.spawn();
        let component = Box::new(Position { x: 10.0, y: 20.0 });

        registry.insert_boxed(&mut world, entity, TypeId::of::<Position>(), component);
    }

    #[test]
    #[should_panic(expected = "type")]
    fn test_remove_unregistered_type() {
        let mut world = World::new();
        let registry = TypeRegistry::new();

        let entity = world.spawn();

        registry.remove_by_type_id(&mut world, entity, TypeId::of::<Position>());
    }

    #[test]
    fn test_multiple_types() {
        let mut registry = TypeRegistry::new();
        registry.register::<Position>();
        registry.register::<Velocity>();

        assert!(registry.is_registered(TypeId::of::<Position>()));
        assert!(registry.is_registered(TypeId::of::<Velocity>()));
    }
}
