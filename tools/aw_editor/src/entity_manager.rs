use glam::{Quat, Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Unique identifier for entities in the editor
pub type EntityId = u64;

// ============================================================================
// MATERIAL SYSTEM - PBR material slots for texture assignment
// ============================================================================

/// Material texture slots for PBR (Physically Based Rendering) workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
pub enum MaterialSlot {
    /// Base color / albedo texture
    Albedo,
    /// Normal map (tangent space)
    Normal,
    /// Roughness texture
    Roughness,
    /// Metallic texture
    Metallic,
    /// Ambient occlusion
    AO,
    /// Combined ORM (Occlusion-Roughness-Metallic)
    ORM,
    /// Emissive/glow texture
    Emission,
    /// Height/displacement map
    Height,
}

/// Material data for an entity with PBR textures and properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMaterial {
    /// Material name or path
    pub name: String,
    /// Texture paths assigned to each material slot
    pub texture_slots: HashMap<MaterialSlot, PathBuf>,
    /// Base color tint (multiplied with albedo texture)
    pub base_color: Vec4,
    /// Metallic factor (0.0 = dielectric, 1.0 = metal)
    pub metallic: f32,
    /// Roughness factor (0.0 = smooth, 1.0 = rough)
    pub roughness: f32,
    /// Emissive color (HDR capable)
    pub emissive: Vec3,
    /// Normal map strength
    pub normal_strength: f32,
}

impl Default for EntityMaterial {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            texture_slots: HashMap::new(),
            base_color: Vec4::ONE, // White
            metallic: 0.0,         // Dielectric by default
            roughness: 0.5,        // Medium roughness
            emissive: Vec3::ZERO,  // No emission
            normal_strength: 1.0,  // Full normal strength
        }
    }
}

impl EntityMaterial {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a texture for a specific material slot
    pub fn set_texture(&mut self, slot: MaterialSlot, path: PathBuf) {
        self.texture_slots.insert(slot, path);
    }

    /// Remove a texture from a slot
    pub fn clear_texture(&mut self, slot: MaterialSlot) {
        self.texture_slots.remove(&slot);
    }

    /// Get the texture path for a slot
    pub fn get_texture(&self, slot: MaterialSlot) -> Option<&PathBuf> {
        self.texture_slots.get(&slot)
    }

    /// Check if material has any textures assigned
    pub fn has_textures(&self) -> bool {
        !self.texture_slots.is_empty()
    }
}

// ============================================================================
// ENTITY - Core entity representation
// ============================================================================

/// Entity representation in the editor
/// Stores transform, mesh reference, material, and arbitrary components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorEntity {
    pub id: EntityId,
    pub name: String,
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub mesh: Option<String>,
    pub material: EntityMaterial,
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
            material: EntityMaterial::new(),
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

    /// Set a texture on the entity's material
    pub fn set_texture(&mut self, slot: MaterialSlot, path: PathBuf) {
        self.material.set_texture(slot, path);
    }

    /// Set the entire material
    pub fn set_material(&mut self, material: EntityMaterial) {
        self.material = material;
    }

    /// Set the mesh path
    pub fn set_mesh(&mut self, mesh_path: String) {
        self.mesh = Some(mesh_path);
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

    #[test]
    fn test_material_texture_management() {
        let mut entity = EditorEntity::new(1, "Test".to_string());
        let path = PathBuf::from("path/to/texture.png");
        
        // Initial state
        assert!(!entity.material.has_textures());
        assert!(entity.material.get_texture(MaterialSlot::Albedo).is_none());

        // Set texture
        entity.set_texture(MaterialSlot::Albedo, path.clone());
        assert!(entity.material.has_textures());
        assert_eq!(entity.material.get_texture(MaterialSlot::Albedo), Some(&path));

        // Clear texture
        entity.material.clear_texture(MaterialSlot::Albedo);
        assert!(!entity.material.has_textures());
        assert!(entity.material.get_texture(MaterialSlot::Albedo).is_none());
    }

    #[test]
    fn test_entity_transform_getters_setters() {
        let mut entity = EditorEntity::new(1, "Test".to_string());
        
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let rot = Quat::IDENTITY;
        let scale = Vec3::splat(2.0);

        entity.set_transform(pos, rot, scale);
        
        let (p, r, s) = entity.transform();
        assert_eq!(p, pos);
        assert_eq!(r, rot);
        assert_eq!(s, scale);
    }

    #[test]
    fn test_entity_manager_crud() {
        let mut manager = EntityManager::new();
        
        // Create
        let id1 = manager.create("E1".to_string());
        let id2 = manager.create("E2".to_string());
        assert_ne!(id1, id2);

        // Read
        assert!(manager.get(id1).is_some());
        assert!(manager.get(id2).is_some());
        assert!(manager.get(999).is_none());

        // Update
        manager.update_position(id1, Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(manager.get(id1).unwrap().position.x, 10.0);

        // Delete
        let removed = manager.remove(id1);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, id1);
        assert!(manager.get(id1).is_none());
        assert!(manager.remove(999).is_none());
    }

    #[test]
    fn test_entity_manager_clear_reset() {
        let mut manager = EntityManager::new();
        let id1 = manager.create("E1".to_string());
        assert_eq!(id1, 1);
        
        manager.clear();
        assert_eq!(manager.count(), 0);
        assert!(manager.entities.is_empty());
        
        // Next ID should be reset to 1
        let id2 = manager.create("E2".to_string());
        assert_eq!(id2, 1); 
    }

    #[test]
    fn test_selection_range_invalid() {
        let mut selection = SelectionSet::new();
        let all_ids = vec![1, 2, 3];
        
        // Try to select range with invalid IDs
        selection.select_range(1, 99, &all_ids);
        assert!(selection.is_empty()); // Should do nothing

        selection.select_range(99, 1, &all_ids);
        assert!(selection.is_empty()); // Should do nothing
    }

    #[test]
    fn test_component_handling() {
        let mut entity = EditorEntity::new(1, "CompEntity".to_string());
        assert!(entity.components.is_empty());

        let comp_data = serde_json::json!({ "health": 100, "speed": 5.0 });
        entity.components.insert("Stats".to_string(), comp_data);

        assert_eq!(entity.components.len(), 1);
        assert!(entity.components.contains_key("Stats"));
        
        let stats = entity.components.get("Stats").unwrap();
        assert_eq!(stats["health"], 100);
    }
}
