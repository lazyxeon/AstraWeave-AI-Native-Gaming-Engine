use anyhow::{Context, Result};
use astraweave_core::{Entity, Health, Pose, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Immutable snapshot of the scene hierarchy used when serializing prefabs.
#[derive(Debug, Clone, Default)]
pub struct PrefabHierarchySnapshot {
    children: HashMap<Entity, Vec<Entity>>,
}

impl PrefabHierarchySnapshot {
    /// Create an empty snapshot.
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
        }
    }

    /// Build a snapshot from an iterator of `(parent, children)` pairs.
    pub fn from_iter<T: IntoIterator<Item = (Entity, Vec<Entity>)>>(iter: T) -> Self {
        Self {
            children: iter.into_iter().collect(),
        }
    }

    /// Insert or replace the children recorded for `parent`.
    pub fn insert_children(&mut self, parent: Entity, children: Vec<Entity>) {
        self.children.insert(parent, children);
    }

    /// Append a single child to `parent` while preserving insertion order.
    pub fn add_child(&mut self, parent: Entity, child: Entity) {
        self.children.entry(parent).or_default().push(child);
    }

    /// Return the children recorded for `parent`.
    pub fn children_of(&self, parent: Entity) -> &[Entity] {
        // Reuse a single empty slice to avoid allocations for nodes without children.
        static EMPTY: [Entity; 0] = [];
        self.children
            .get(&parent)
            .map(|v| v.as_slice())
            .unwrap_or(&EMPTY)
    }
}

impl FromIterator<(Entity, Vec<Entity>)> for PrefabHierarchySnapshot {
    fn from_iter<T: IntoIterator<Item = (Entity, Vec<Entity>)>>(iter: T) -> Self {
        Self {
            children: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabData {
    pub name: String,
    pub entities: Vec<PrefabEntityData>,
    pub root_entity_index: usize,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabEntityData {
    pub name: String,
    pub pos_x: i32,
    pub pos_y: i32,
    pub team_id: u32,
    pub health: i32,
    pub max_health: i32,
    pub children_indices: Vec<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefab_reference: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PrefabEntitySnapshot {
    pub entity: Entity,
    pub prefab_index: usize,
    pub pose: Pose,
    pub health: Option<Health>,
}

#[derive(Debug, Clone)]
pub struct PrefabInstanceSnapshot {
    pub root_entity: Entity,
    pub entities: Vec<PrefabEntitySnapshot>,
}

impl PrefabInstanceSnapshot {
    pub fn new(root_entity: Entity) -> Self {
        Self {
            root_entity,
            entities: Vec::new(),
        }
    }

    pub fn push(&mut self, snapshot: PrefabEntitySnapshot) {
        self.entities.push(snapshot);
    }
}

#[derive(Debug, Clone)]
pub struct PrefabInstance {
    pub source: PathBuf,
    pub root_entity: Entity,
    pub entity_mapping: HashMap<usize, Entity>,
    pub overrides: HashMap<Entity, EntityOverrides>,
}

#[derive(Debug, Clone, Default)]
pub struct EntityOverrides {
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
    pub health: Option<i32>,
    pub max_health: Option<i32>,
}

impl EntityOverrides {
    /// Check if Pose component has any overrides
    pub fn has_pose_override(&self) -> bool {
        self.pos_x.is_some() || self.pos_y.is_some()
    }

    /// Check if Health component has any overrides
    pub fn has_health_override(&self) -> bool {
        self.health.is_some() || self.max_health.is_some()
    }

    /// Check if any component is overridden
    pub fn has_any_override(&self) -> bool {
        self.has_pose_override() || self.has_health_override()
    }
}

impl PrefabData {
    pub fn from_entity(world: &World, entity: Entity, name: String) -> Result<Self> {
        Self::from_entity_with_hierarchy(world, entity, name, None)
    }

    pub fn from_entity_with_hierarchy(
        world: &World,
        entity: Entity,
        name: String,
        hierarchy: Option<&PrefabHierarchySnapshot>,
    ) -> Result<Self> {
        let mut entities = Vec::new();
        let mut entity_index_map = HashMap::new();

        Self::collect_entity_recursive(
            world,
            entity,
            hierarchy,
            &mut entities,
            &mut entity_index_map,
        )?;

        Ok(PrefabData {
            name,
            entities,
            root_entity_index: 0,
            version: "1.0".to_string(),
        })
    }

    fn collect_entity_recursive(
        world: &World,
        entity: Entity,
        hierarchy: Option<&PrefabHierarchySnapshot>,
        entities: &mut Vec<PrefabEntityData>,
        entity_index_map: &mut HashMap<Entity, usize>,
    ) -> Result<usize> {
        if let Some(&existing) = entity_index_map.get(&entity) {
            return Ok(existing);
        }

        let name = world
            .name(entity)
            .context("Entity name not found")?
            .to_string();

        let pose = world.pose(entity).context("Entity pose not found")?;

        let health = world.health(entity).map(|h| h.hp).unwrap_or(100);

        let team_id = world.team(entity).map(|t| t.id as u32).unwrap_or(0);

        let current_index = entities.len();
        entity_index_map.insert(entity, current_index);

        entities.push(PrefabEntityData {
            name,
            pos_x: pose.pos.x,
            pos_y: pose.pos.y,
            team_id,
            health,
            max_health: health,
            children_indices: Vec::new(),
            prefab_reference: None,
        });

        if let Some(snapshot) = hierarchy {
            let mut child_indices = Vec::new();
            for child in snapshot.children_of(entity) {
                // Skip children that no longer exist in the world.
                if world.pose(*child).is_none() {
                    continue;
                }
                let child_index = Self::collect_entity_recursive(
                    world,
                    *child,
                    hierarchy,
                    entities,
                    entity_index_map,
                )?;
                child_indices.push(child_index);
            }

            if let Some(prefab_entity) = entities.get_mut(current_index) {
                prefab_entity.children_indices = child_indices;
            }
        }

        Ok(current_index)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let ron_string = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .context("Failed to serialize prefab to RON")?;

        std::fs::write(path.as_ref(), ron_string).context("Failed to write prefab file")?;

        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let ron_string =
            std::fs::read_to_string(path.as_ref()).context("Failed to read prefab file")?;

        let prefab: PrefabData =
            ron::from_str(&ron_string).context("Failed to deserialize prefab from RON")?;

        Ok(prefab)
    }

    pub fn instantiate(&self, world: &mut World, spawn_pos: (i32, i32)) -> Result<PrefabInstance> {
        let mut entity_mapping = HashMap::new();

        if self.entities.is_empty() {
            anyhow::bail!("Prefab has no entities");
        }

        let root_data = &self.entities[self.root_entity_index];
        let root_entity = world.spawn(
            &root_data.name,
            astraweave_core::IVec2 {
                x: spawn_pos.0 + root_data.pos_x,
                y: spawn_pos.1 + root_data.pos_y,
            },
            astraweave_core::Team {
                id: root_data.team_id as u8,
            },
            root_data.health,
            0,
        );

        entity_mapping.insert(self.root_entity_index, root_entity);

        for (idx, entity_data) in self.entities.iter().enumerate().skip(1) {
            if entity_data.prefab_reference.is_some() {
                continue;
            }

            let entity = world.spawn(
                &entity_data.name,
                astraweave_core::IVec2 {
                    x: spawn_pos.0 + entity_data.pos_x,
                    y: spawn_pos.1 + entity_data.pos_y,
                },
                astraweave_core::Team {
                    id: entity_data.team_id as u8,
                },
                entity_data.health,
                0,
            );
            entity_mapping.insert(idx, entity);
        }

        Ok(PrefabInstance {
            source: PathBuf::new(),
            root_entity,
            entity_mapping,
            overrides: HashMap::new(),
        })
    }
}

impl PrefabInstance {
    pub fn track_override(&mut self, entity: Entity, world: &World) {
        let pose = world.pose(entity);
        let health = world.health(entity);
        self.track_override_with_values(entity, pose, health);
    }

    pub fn track_override_with_values(
        &mut self,
        entity: Entity,
        pose: Option<Pose>,
        health: Option<Health>,
    ) {
        let overrides = self.overrides.entry(entity).or_default();

        if let Some(pose) = pose {
            overrides.pos_x = Some(pose.pos.x);
            overrides.pos_y = Some(pose.pos.y);
        }

        if let Some(health) = health {
            overrides.health = Some(health.hp);
            overrides.max_health = Some(health.hp);
        }
    }

    pub fn has_overrides(&self, entity: Entity) -> bool {
        self.overrides.contains_key(&entity)
    }

    pub fn revert_to_prefab(&mut self, world: &mut World) -> Result<()> {
        // Validate prefab file exists and is readable
        if !self.source.exists() {
            anyhow::bail!(
                "Cannot revert: Prefab file does not exist: {}",
                self.source.display()
            );
        }

        let metadata = std::fs::metadata(&self.source)
            .context("Cannot revert: Unable to read prefab file metadata")?;

        if !metadata.is_file() {
            anyhow::bail!(
                "Cannot revert: Path is not a file: {}",
                self.source.display()
            );
        }

        let prefab_data = PrefabData::load_from_file(&self.source)
            .context("Cannot revert: Failed to load prefab file")?;

        if prefab_data.entities.is_empty() {
            anyhow::bail!("Cannot revert: Prefab file contains no entities");
        }

        let mut reverted_count = 0;
        for (prefab_idx, entity) in &self.entity_mapping {
            if let Some(prefab_entity_data) = prefab_data.entities.get(*prefab_idx) {
                // Restore pose (position)
                if let Some(pose) = world.pose_mut(*entity) {
                    pose.pos.x = prefab_entity_data.pos_x;
                    pose.pos.y = prefab_entity_data.pos_y;
                    reverted_count += 1;
                }

                // Restore health
                if let Some(health) = world.health_mut(*entity) {
                    health.hp = prefab_entity_data.health;
                }
            }
        }

        // Clear all overrides since we've reverted to prefab state
        self.overrides.clear();

        if reverted_count == 0 {
            anyhow::bail!("Cannot revert: No entities were reverted (possible data mismatch)");
        }

        Ok(())
    }

    pub fn apply_to_prefab(&mut self, world: &World) -> Result<()> {
        // Validate prefab file exists
        if !self.source.exists() {
            anyhow::bail!(
                "Cannot apply: Prefab file does not exist: {}",
                self.source.display()
            );
        }

        // Check if file is read-only
        let metadata = std::fs::metadata(&self.source)
            .context("Cannot apply: Unable to read prefab file metadata")?;

        if metadata.permissions().readonly() {
            anyhow::bail!(
                "Cannot apply: Prefab file is read-only: {}. Please change file permissions.",
                self.source.display()
            );
        }

        let mut prefab_data = PrefabData::load_from_file(&self.source)
            .context("Cannot apply: Failed to load prefab file")?;

        if prefab_data.entities.is_empty() {
            anyhow::bail!("Cannot apply: Prefab file contains no entities");
        }

        let mut applied_count = 0;
        for (prefab_idx, entity) in &self.entity_mapping {
            if let Some(prefab_entity_data) = prefab_data.entities.get_mut(*prefab_idx) {
                // Apply current pose to prefab
                if let Some(pose) = world.pose(*entity) {
                    prefab_entity_data.pos_x = pose.pos.x;
                    prefab_entity_data.pos_y = pose.pos.y;
                    applied_count += 1;
                }

                // Apply current health to prefab
                if let Some(health) = world.health(*entity) {
                    prefab_entity_data.health = health.hp;
                    prefab_entity_data.max_health = health.hp;
                }
            }
        }

        if applied_count == 0 {
            anyhow::bail!("Cannot apply: No entities were applied (possible data mismatch)");
        }

        // Save updated prefab to file
        prefab_data.save_to_file(&self.source).context(
            "Cannot apply: Failed to save prefab file (check disk space and permissions)",
        )?;

        // Clear overrides since current state is now the prefab state
        self.overrides.clear();

        Ok(())
    }

    /// Revert ALL entities in this prefab instance to their original prefab state
    pub fn revert_all_to_prefab(&mut self, world: &mut World) -> Result<()> {
        // Validate prefab file exists and is readable
        if !self.source.exists() {
            anyhow::bail!(
                "Cannot revert all: Prefab file does not exist: {}",
                self.source.display()
            );
        }

        let prefab_data = PrefabData::load_from_file(&self.source)
            .context("Cannot revert all: Failed to load prefab file")?;

        if prefab_data.entities.is_empty() {
            anyhow::bail!("Cannot revert all: Prefab file contains no entities");
        }

        if self.entity_mapping.is_empty() {
            anyhow::bail!("Cannot revert all: No entities in prefab instance");
        }

        let mut reverted_count = 0;

        for (prefab_idx, entity) in &self.entity_mapping {
            if let Some(prefab_entity_data) = prefab_data.entities.get(*prefab_idx) {
                // Restore pose (position)
                if let Some(pose) = world.pose_mut(*entity) {
                    pose.pos.x = prefab_entity_data.pos_x;
                    pose.pos.y = prefab_entity_data.pos_y;
                }

                // Restore health
                if let Some(health) = world.health_mut(*entity) {
                    health.hp = prefab_entity_data.health;
                }

                reverted_count += 1;
            }
        }

        if reverted_count == 0 {
            anyhow::bail!("Cannot revert all: No entities were reverted (possible data mismatch)");
        }

        // Clear all overrides since we've reverted all entities
        self.overrides.clear();

        debug!("✅ Reverted {} entities to prefab state", reverted_count);
        Ok(())
    }

    /// Apply ALL entities in this prefab instance to the prefab file
    pub fn apply_all_to_prefab(&mut self, world: &World) -> Result<()> {
        // Validate prefab file exists
        if !self.source.exists() {
            anyhow::bail!(
                "Cannot apply all: Prefab file does not exist: {}",
                self.source.display()
            );
        }

        // Check if file is read-only
        let metadata = std::fs::metadata(&self.source)
            .context("Cannot apply all: Unable to read prefab file metadata")?;

        if metadata.permissions().readonly() {
            anyhow::bail!(
                "Cannot apply all: Prefab file is read-only: {}. Please change file permissions.",
                self.source.display()
            );
        }

        let mut prefab_data = PrefabData::load_from_file(&self.source)
            .context("Cannot apply all: Failed to load prefab file")?;

        if prefab_data.entities.is_empty() {
            anyhow::bail!("Cannot apply all: Prefab file contains no entities");
        }

        if self.entity_mapping.is_empty() {
            anyhow::bail!("Cannot apply all: No entities in prefab instance");
        }

        let mut applied_count = 0;

        for (prefab_idx, entity) in &self.entity_mapping {
            if let Some(prefab_entity_data) = prefab_data.entities.get_mut(*prefab_idx) {
                // Apply current pose to prefab
                if let Some(pose) = world.pose(*entity) {
                    prefab_entity_data.pos_x = pose.pos.x;
                    prefab_entity_data.pos_y = pose.pos.y;
                }

                // Apply current health to prefab
                if let Some(health) = world.health(*entity) {
                    prefab_entity_data.health = health.hp;
                    prefab_entity_data.max_health = health.hp;
                }

                applied_count += 1;
            }
        }

        if applied_count == 0 {
            anyhow::bail!("Cannot apply all: No entities were applied (possible data mismatch)");
        }

        // Save updated prefab to file
        prefab_data.save_to_file(&self.source).context(
            "Cannot apply all: Failed to save prefab file (check disk space and permissions)",
        )?;

        // Clear all overrides since current state is now the prefab state
        self.overrides.clear();

        debug!("✅ Applied {} entities to prefab file", applied_count);
        Ok(())
    }
}

#[derive(Debug)]
pub struct PrefabManager {
    instances: Vec<PrefabInstance>,
    prefab_directory: PathBuf,
}

impl PrefabManager {
    pub fn new<P: AsRef<Path>>(prefab_directory: P) -> Self {
        let prefab_directory = prefab_directory.as_ref().to_path_buf();
        std::fs::create_dir_all(&prefab_directory).ok();

        PrefabManager {
            instances: Vec::new(),
            prefab_directory,
        }
    }

    pub fn shared<P: AsRef<Path>>(prefab_directory: P) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(prefab_directory)))
    }

    pub fn create_prefab(&self, world: &World, entity: Entity, name: &str) -> Result<PathBuf> {
        self.create_prefab_with_hierarchy(world, entity, name, None)
    }

    pub fn create_prefab_with_hierarchy(
        &self,
        world: &World,
        entity: Entity,
        name: &str,
        hierarchy: Option<&PrefabHierarchySnapshot>,
    ) -> Result<PathBuf> {
        let prefab_data =
            PrefabData::from_entity_with_hierarchy(world, entity, name.to_string(), hierarchy)?;
        let prefab_path = self.prefab_directory.join(format!("{}.prefab.ron", name));
        prefab_data.save_to_file(&prefab_path)?;
        Ok(prefab_path)
    }

    pub fn instantiate_prefab<P: AsRef<Path>>(
        &mut self,
        prefab_path: P,
        world: &mut World,
        spawn_pos: (i32, i32),
    ) -> Result<Entity> {
        let prefab_data = PrefabData::load_from_file(&prefab_path)?;
        let mut instance = prefab_data.instantiate(world, spawn_pos)?;
        instance.source = prefab_path.as_ref().to_path_buf();

        let root_entity = instance.root_entity;
        self.instances.push(instance);

        Ok(root_entity)
    }

    pub fn find_instance(&self, entity: Entity) -> Option<&PrefabInstance> {
        self.instances.iter().find(|inst| {
            inst.root_entity == entity || inst.entity_mapping.values().any(|&e| e == entity)
        })
    }

    pub fn find_instance_mut(&mut self, entity: Entity) -> Option<&mut PrefabInstance> {
        self.instances.iter_mut().find(|inst| {
            inst.root_entity == entity || inst.entity_mapping.values().any(|&e| e == entity)
        })
    }

    pub fn capture_snapshot(
        &self,
        world: &World,
        entity: Entity,
    ) -> Result<PrefabInstanceSnapshot> {
        let instance = self
            .find_instance(entity)
            .ok_or_else(|| anyhow::anyhow!("Entity {} is not a prefab instance", entity))?;

        let mut snapshot = PrefabInstanceSnapshot::new(instance.root_entity);
        for (&prefab_index, &instance_entity) in &instance.entity_mapping {
            let pose = world
                .pose(instance_entity)
                .with_context(|| format!("Missing pose for entity {}", instance_entity))?;
            let health = world.health(instance_entity);
            snapshot.push(PrefabEntitySnapshot {
                entity: instance_entity,
                prefab_index,
                pose,
                health,
            });
        }
        Ok(snapshot)
    }

    pub fn restore_snapshot(
        &mut self,
        snapshot: &PrefabInstanceSnapshot,
        world: &mut World,
    ) -> Result<()> {
        let instance = self
            .find_instance_mut(snapshot.root_entity)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Prefab instance for entity {} not found",
                    snapshot.root_entity
                )
            })?;

        for entry in &snapshot.entities {
            if let Some(pose) = world.pose_mut(entry.entity) {
                *pose = entry.pose;
            }

            if let (Some(saved), Some(current)) = (entry.health, world.health_mut(entry.entity)) {
                current.hp = saved.hp;
            }

            instance.track_override_with_values(entry.entity, Some(entry.pose), entry.health);
        }

        Ok(())
    }

    pub fn track_override_snapshot(
        &mut self,
        entity: Entity,
        pose: Option<Pose>,
        health: Option<Health>,
    ) {
        if let Some(instance) = self.find_instance_mut(entity) {
            instance.track_override_with_values(entity, pose, health);
        }
    }

    pub fn get_all_prefab_files(&self) -> Result<Vec<PathBuf>> {
        let mut prefab_files = Vec::new();

        if !self.prefab_directory.exists() {
            return Ok(prefab_files);
        }

        for entry in std::fs::read_dir(&self.prefab_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("ron")
                && path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.ends_with(".prefab"))
                    .unwrap_or(false)
                {
                    prefab_files.push(path);
                }
        }

        Ok(prefab_files)
    }

    /// Remove a tracked prefab instance (for undo support)
    pub fn remove_instance(&mut self, entity: Entity) {
        self.instances.retain(|inst| {
            inst.root_entity != entity && !inst.entity_mapping.values().any(|&e| e == entity)
        });
    }

    /// Apply overrides from an instance back to its prefab file
    pub fn apply_overrides_to_prefab(&mut self, entity: Entity, world: &World) -> Result<()> {
        let instance = self
            .find_instance(entity)
            .ok_or_else(|| anyhow::anyhow!("Entity {} is not a prefab instance", entity))?;

        let prefab_path = instance.source.clone();
        let mut prefab_data = PrefabData::load_from_file(&prefab_path)?;

        // Apply overrides from the instance to the prefab data
        for (&inst_entity, overrides) in &instance.overrides {
            // Find which prefab entity this corresponds to
            for (&prefab_idx, &mapping_entity) in &instance.entity_mapping {
                if mapping_entity == inst_entity {
                    if let Some(prefab_entity_data) = prefab_data.entities.get_mut(prefab_idx) {
                        if let Some(x) = overrides.pos_x {
                            prefab_entity_data.pos_x = x;
                        }
                        if let Some(y) = overrides.pos_y {
                            prefab_entity_data.pos_y = y;
                        }
                        if let Some(hp) = overrides.health {
                            prefab_entity_data.health = hp;
                        }
                        if let Some(max_hp) = overrides.max_health {
                            prefab_entity_data.max_health = max_hp;
                        }
                    }
                }
            }
        }

        // Also capture current world state for root entity
        if let Some(pose) = world.pose(entity) {
            if let Some(prefab_entity_data) = prefab_data.entities.get_mut(0) {
                prefab_entity_data.pos_x = pose.pos.x;
                prefab_entity_data.pos_y = pose.pos.y;
            }
        }

        prefab_data.save_to_file(&prefab_path)?;

        // Clear overrides since they're now in the prefab
        if let Some(instance) = self.find_instance_mut(entity) {
            instance.overrides.clear();
        }

        Ok(())
    }

    /// Revert an instance to match its prefab file
    pub fn revert_instance_to_prefab(&mut self, entity: Entity, world: &mut World) -> Result<()> {
        let instance = self
            .find_instance_mut(entity)
            .ok_or_else(|| anyhow::anyhow!("Entity {} is not a prefab instance", entity))?;

        instance.revert_to_prefab(world)?;
        Ok(())
    }

    /// Clear all tracked prefab instances
    ///
    /// Call this when unloading a scene or starting fresh to prevent memory leaks.
    pub fn clear_instances(&mut self) {
        let count = self.instances.len();
        self.instances.clear();
        debug!("Cleared {} prefab instances", count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{IVec2, Team};

    #[test]
    fn test_prefab_serialization() {
        let prefab = PrefabData {
            name: "TestPrefab".to_string(),
            entities: vec![PrefabEntityData {
                name: "TestEntity".to_string(),
                pos_x: 10,
                pos_y: 20,
                team_id: 1,
                health: 100,
                max_health: 100,
                children_indices: vec![],
                prefab_reference: None,
            }],
            root_entity_index: 0,
            version: "1.0".to_string(),
        };

        let ron_string =
            ron::ser::to_string_pretty(&prefab, ron::ser::PrettyConfig::default()).unwrap();
        assert!(ron_string.contains("TestPrefab"));
        assert!(ron_string.contains("TestEntity"));

        let deserialized: PrefabData = ron::from_str(&ron_string).unwrap();
        assert_eq!(deserialized.name, "TestPrefab");
        assert_eq!(deserialized.entities.len(), 1);
        assert_eq!(deserialized.entities[0].name, "TestEntity");
    }

    #[test]
    fn test_prefab_instance_override_tracking() {
        let mut world = World::new();
        let entity = world.spawn("TestEntity", IVec2 { x: 5, y: 5 }, Team { id: 0 }, 100, 100);

        let mut instance = PrefabInstance {
            source: PathBuf::from("test.prefab.ron"),
            root_entity: entity,
            entity_mapping: HashMap::new(),
            overrides: HashMap::new(),
        };

        instance.track_override(entity, &world);
        assert!(instance.has_overrides(entity));
    }

    #[test]
    fn prefab_serialization_records_hierarchy() {
        let mut world = World::new();
        let root = world.spawn("Root", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 0);
        let child_a = world.spawn("ChildA", IVec2 { x: 1, y: 1 }, Team { id: 0 }, 100, 0);
        let child_b = world.spawn("ChildB", IVec2 { x: -1, y: 1 }, Team { id: 0 }, 100, 0);
        let grandchild = world.spawn("Grandchild", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);

        let snapshot = PrefabHierarchySnapshot::from_iter([
            (root, vec![child_a, child_b]),
            (child_a, vec![grandchild]),
        ]);

        let prefab = PrefabData::from_entity_with_hierarchy(
            &world,
            root,
            "HierPrefab".into(),
            Some(&snapshot),
        )
        .expect("prefab builds");

        assert_eq!(prefab.entities.len(), 4);

        let root_data = &prefab.entities[prefab.root_entity_index];
        assert_eq!(root_data.children_indices.len(), 2);

        let first_child_idx = root_data.children_indices[0];
        let second_child_idx = root_data.children_indices[1];
        assert_eq!(prefab.entities[first_child_idx].name, "ChildA");
        assert_eq!(prefab.entities[second_child_idx].name, "ChildB");

        let grandchild_indices = &prefab.entities[first_child_idx].children_indices;
        assert_eq!(grandchild_indices.len(), 1);
        let grandchild_index = prefab
            .entities
            .iter()
            .position(|e| e.name == "Grandchild")
            .expect("grandchild entity present");
        assert_eq!(grandchild_indices[0], grandchild_index);
    }
}
