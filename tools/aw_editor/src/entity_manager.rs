use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

/// Multi-selection state for the editor
/// 
/// Supports Ctrl+click (toggle), Shift+click (range), and maintains a primary selection
#[derive(Debug, Clone, Default)]
pub struct SelectionSet {
    /// All selected entity IDs
    pub entities: HashSet<EntityId>,
    
    /// Primary selection (last selected entity, used for gizmo placement)
    pub primary: Option<EntityId>,
}

impl SelectionSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add entity to selection
    /// 
    /// # Arguments
    /// 
    /// * `entity` - Entity ID to add
    /// * `is_primary` - Whether this becomes the primary selection
    pub fn add(&mut self, entity: EntityId, is_primary: bool) {
        self.entities.insert(entity);
        if is_primary {
            self.primary = Some(entity);
        }
    }

    /// Remove entity from selection
    pub fn remove(&mut self, entity: EntityId) {
        self.entities.remove(&entity);
        
        if self.primary == Some(entity) {
            self.primary = self.entities.iter().next().copied();
        }
    }

    /// Toggle entity selection (add if not selected, remove if selected)
    pub fn toggle(&mut self, entity: EntityId) {
        if self.entities.contains(&entity) {
            self.remove(entity);
        } else {
            self.add(entity, true);
        }
    }

    /// Clear all selections
    pub fn clear(&mut self) {
        self.entities.clear();
        self.primary = None;
    }

    /// Select only this entity (clear others)
    pub fn select_only(&mut self, entity: EntityId) {
        self.clear();
        self.add(entity, true);
    }

    /// Check if entity is selected
    pub fn is_selected(&self, entity: EntityId) -> bool {
        self.entities.contains(&entity)
    }

    /// Get selected entity count
    pub fn count(&self) -> usize {
        self.entities.len()
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Get all selected entity IDs as a Vec
    pub fn to_vec(&self) -> Vec<EntityId> {
        self.entities.iter().copied().collect()
    }

    /// Range selection between two entities (for Shift+click in hierarchy)
    /// 
    /// # Arguments
    /// 
    /// * `from` - Start entity ID
    /// * `to` - End entity ID
    /// * `all_ids` - Ordered list of all entity IDs (from hierarchy)
    pub fn select_range(&mut self, from: EntityId, to: EntityId, all_ids: &[EntityId]) {
        if let (Some(from_idx), Some(to_idx)) = (
            all_ids.iter().position(|&id| id == from),
            all_ids.iter().position(|&id| id == to),
        ) {
            let (start, end) = if from_idx < to_idx {
                (from_idx, to_idx)
            } else {
                (to_idx, from_idx)
            };

            for &id in &all_ids[start..=end] {
                self.entities.insert(id);
            }

            self.primary = Some(to);
        }
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

    #[test]
    fn test_selection_single() {
        let mut selection = SelectionSet::new();
        assert!(selection.is_empty());
        assert_eq!(selection.count(), 0);

        selection.add(1, true);
        assert!(!selection.is_empty());
        assert_eq!(selection.count(), 1);
        assert!(selection.is_selected(1));
        assert_eq!(selection.primary, Some(1));
    }

    #[test]
    fn test_selection_multiple() {
        let mut selection = SelectionSet::new();
        
        selection.add(1, true);
        selection.add(2, false);
        selection.add(3, false);
        
        assert_eq!(selection.count(), 3);
        assert!(selection.is_selected(1));
        assert!(selection.is_selected(2));
        assert!(selection.is_selected(3));
        assert_eq!(selection.primary, Some(1));
    }

    #[test]
    fn test_selection_toggle() {
        let mut selection = SelectionSet::new();
        
        selection.toggle(1);
        assert!(selection.is_selected(1));
        
        selection.toggle(1);
        assert!(!selection.is_selected(1));
        assert!(selection.is_empty());
    }

    #[test]
    fn test_selection_remove() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        selection.add(3, false);
        
        selection.remove(2);
        assert_eq!(selection.count(), 2);
        assert!(!selection.is_selected(2));
        assert_eq!(selection.primary, Some(1));
        
        selection.remove(1);
        assert_eq!(selection.count(), 1);
        assert!(selection.primary.is_some());
    }

    #[test]
    fn test_selection_clear() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        
        selection.clear();
        assert!(selection.is_empty());
        assert_eq!(selection.count(), 0);
        assert!(selection.primary.is_none());
    }

    #[test]
    fn test_selection_select_only() {
        let mut selection = SelectionSet::new();
        selection.add(1, true);
        selection.add(2, false);
        
        selection.select_only(3);
        assert_eq!(selection.count(), 1);
        assert!(selection.is_selected(3));
        assert!(!selection.is_selected(1));
        assert!(!selection.is_selected(2));
        assert_eq!(selection.primary, Some(3));
    }

    #[test]
    fn test_selection_range() {
        let mut selection = SelectionSet::new();
        let all_ids = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        
        selection.select_range(3, 7, &all_ids);
        
        assert_eq!(selection.count(), 5);
        assert!(selection.is_selected(3));
        assert!(selection.is_selected(4));
        assert!(selection.is_selected(5));
        assert!(selection.is_selected(6));
        assert!(selection.is_selected(7));
        assert!(!selection.is_selected(2));
        assert!(!selection.is_selected(8));
        assert_eq!(selection.primary, Some(7));
    }

    #[test]
    fn test_selection_range_reverse() {
        let mut selection = SelectionSet::new();
        let all_ids = vec![1, 2, 3, 4, 5];
        
        selection.select_range(5, 2, &all_ids);
        
        assert_eq!(selection.count(), 4);
        assert!(selection.is_selected(2));
        assert!(selection.is_selected(3));
        assert!(selection.is_selected(4));
        assert!(selection.is_selected(5));
        assert_eq!(selection.primary, Some(2));
    }
}
