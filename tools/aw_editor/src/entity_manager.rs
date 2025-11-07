use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for entities in the editor
pub type EntityId = u64;

/// Entity representation in the editor
/// Stores transform, mesh reference, and arbitrary components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorEntity {
    pub id: EntityId,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub mesh: Option<String>,
    pub components: HashMap<String, serde_json::Value>,
}

impl EditorEntity {
    pub fn new(id: EntityId, name: String) -> Self {
        Self {
            id,
            name,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            mesh: None,
            components: HashMap::new(),
        }
    }

    /// Get the entity's transform as (position, rotation, scale)
    pub fn transform(&self) -> (Vec3, Quat, Vec3) {
        (self.position, self.rotation, self.scale)
    }

    /// Set the entity's transform
    pub fn set_transform(&mut self, position: Vec3, rotation: Quat, scale: Vec3) {
        self.position = position;
        self.rotation = rotation;
        self.scale = scale;
    }

    /// Get axis-aligned bounding box (AABB) for picking
    /// Returns (min, max) corners
    pub fn aabb(&self) -> (Vec3, Vec3) {
        // Simple unit cube scaled by entity scale
        let half_size = self.scale * 0.5;
        let min = self.position - half_size;
        let max = self.position + half_size;
        (min, max)
    }
}

/// Manages all entities in the editor scene
pub struct EntityManager {
    entities: HashMap<EntityId, EditorEntity>,
    next_id: EntityId,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new entity with auto-generated ID
    pub fn create(&mut self, name: String) -> EntityId {
        let id = self.next_id;
        self.next_id += 1;
        let entity = EditorEntity::new(id, name);
        self.entities.insert(id, entity);
        id
    }

    /// Add an existing entity (used for deserialization)
    pub fn add(&mut self, entity: EditorEntity) {
        let id = entity.id;
        self.entities.insert(id, entity);
        if id >= self.next_id {
            self.next_id = id + 1;
        }
    }

    /// Get entity by ID
    pub fn get(&self, id: EntityId) -> Option<&EditorEntity> {
        self.entities.get(&id)
    }

    /// Get mutable entity by ID
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut EditorEntity> {
        self.entities.get_mut(&id)
    }

    /// Remove entity by ID
    pub fn remove(&mut self, id: EntityId) -> Option<EditorEntity> {
        self.entities.remove(&id)
    }

    /// Get all entity IDs
    pub fn ids(&self) -> Vec<EntityId> {
        self.entities.keys().copied().collect()
    }

    /// Get all entities
    pub fn entities(&self) -> &HashMap<EntityId, EditorEntity> {
        &self.entities
    }

    /// Update entity transform
    pub fn update_transform(&mut self, id: EntityId, position: Vec3, rotation: Quat, scale: Vec3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.set_transform(position, rotation, scale);
        }
    }

    /// Update entity position only
    pub fn update_position(&mut self, id: EntityId, position: Vec3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.position = position;
        }
    }

    /// Update entity rotation only
    pub fn update_rotation(&mut self, id: EntityId, rotation: Quat) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.rotation = rotation;
        }
    }

    /// Update entity scale only
    pub fn update_scale(&mut self, id: EntityId, scale: Vec3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.scale = scale;
        }
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.entities.clear();
        self.next_id = 1;
    }

    /// Get entity count
    pub fn count(&self) -> usize {
        self.entities.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let mut manager = EntityManager::new();
        let id = manager.create("TestEntity".to_string());
        assert_eq!(id, 1);
        assert_eq!(manager.count(), 1);
        
        let entity = manager.get(id).unwrap();
        assert_eq!(entity.name, "TestEntity");
        assert_eq!(entity.position, Vec3::ZERO);
    }

    #[test]
    fn test_transform_update() {
        let mut manager = EntityManager::new();
        let id = manager.create("TestEntity".to_string());
        
        let new_pos = Vec3::new(1.0, 2.0, 3.0);
        manager.update_position(id, new_pos);
        
        let entity = manager.get(id).unwrap();
        assert_eq!(entity.position, new_pos);
    }

    #[test]
    fn test_aabb() {
        let mut manager = EntityManager::new();
        let id = manager.create("TestEntity".to_string());
        manager.update_scale(id, Vec3::new(2.0, 2.0, 2.0));
        
        let entity = manager.get(id).unwrap();
        let (min, max) = entity.aabb();
        assert_eq!(min, Vec3::new(-1.0, -1.0, -1.0));
        assert_eq!(max, Vec3::new(1.0, 1.0, 1.0));
    }
}
