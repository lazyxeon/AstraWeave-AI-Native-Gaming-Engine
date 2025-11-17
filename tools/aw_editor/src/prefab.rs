use anyhow::{Context, Result};
use astraweave_core::{Entity, Health, IVec2, Pose, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
pub struct PrefabInstance {
    pub source: PathBuf,
    pub root_entity: Entity,
    pub entity_mapping: HashMap<usize, Entity>,
    entity_lookup: HashMap<Entity, usize>,
    pub overrides: HashMap<Entity, EntityOverrides>,
    template_entities: Vec<PrefabEntityData>,
}

#[derive(Debug, Clone, Default)]
pub struct EntityOverrides {
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
    pub health: Option<i32>,
    pub max_health: Option<i32>,
}

#[derive(Clone, Copy, Debug)]
pub struct PrefabEntitySnapshot {
    pub entity: Entity,
    pub pose: Option<Pose>,
    pub health: Option<Health>,
}

pub type PrefabManagerHandle = Arc<Mutex<PrefabManager>>;

impl PrefabData {
    pub fn from_entity(world: &World, entity: Entity, name: String) -> Result<Self> {
        let mut entities = Vec::new();
        let mut entity_index_map = HashMap::new();

        Self::collect_entity_recursive(world, entity, &mut entities, &mut entity_index_map, 0)?;

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
        entities: &mut Vec<PrefabEntityData>,
        entity_index_map: &mut HashMap<Entity, usize>,
        current_index: usize,
    ) -> Result<()> {
        let name = world
            .name(entity)
            .context("Entity name not found")?
            .to_string();

        let pose = world.pose(entity).context("Entity pose not found")?;

        let health = world.health(entity).map(|h| h.hp).unwrap_or(100);

        let team_id = world.team(entity).map(|t| t.id as u32).unwrap_or(0);

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

        Ok(())
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
        let mut entity_lookup = HashMap::new();
        entity_lookup.insert(root_entity, self.root_entity_index);

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
            entity_lookup.insert(entity, idx);
        }

        Ok(PrefabInstance {
            source: PathBuf::new(),
            root_entity,
            entity_mapping,
            entity_lookup,
            overrides: HashMap::new(),
            template_entities: self.entities.clone(),
        })
    }
}

impl PrefabInstance {
    pub fn contains_entity(&self, entity: Entity) -> bool {
        self.entity_lookup.contains_key(&entity)
    }

    pub fn refresh_overrides(&mut self, world: &World) {
        self.overrides.clear();

        for (prefab_idx, entity) in &self.entity_mapping {
            if let Some(prefab_entity) = self.template_entities.get(*prefab_idx) {
                let mut overrides = EntityOverrides::default();

                if let Some(pose) = world.pose(*entity) {
                    if pose.pos.x != prefab_entity.pos_x {
                        overrides.pos_x = Some(pose.pos.x);
                    }
                    if pose.pos.y != prefab_entity.pos_y {
                        overrides.pos_y = Some(pose.pos.y);
                    }
                }

                if let Some(health) = world.health(*entity) {
                    if health.hp != prefab_entity.health {
                        overrides.health = Some(health.hp);
                    }
                    if health.hp != prefab_entity.max_health {
                        overrides.max_health = Some(health.hp);
                    }
                }

                if overrides.pos_x.is_some()
                    || overrides.pos_y.is_some()
                    || overrides.health.is_some()
                    || overrides.max_health.is_some()
                {
                    self.overrides.insert(*entity, overrides);
                }
            }
        }
    }

    pub fn track_override(&mut self, entity: Entity, world: &World) {
        let pose = world.pose(entity);
        let health = world.health(entity);
        self.track_override_snapshot(entity, pose, health);
    }

    pub fn track_override_snapshot(
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
        let prefab_data = PrefabData::load_from_file(&self.source)?;

        for (prefab_idx, entity) in &self.entity_mapping {
            if let Some(prefab_entity_data) = prefab_data.entities.get(*prefab_idx) {
                if let Some(pose) = world.pose_mut(*entity) {
                    pose.pos.x = prefab_entity_data.pos_x;
                    pose.pos.y = prefab_entity_data.pos_y;
                }
            }
        }

        self.overrides.clear();
        self.template_entities = prefab_data.entities.clone();
        Ok(())
    }

    pub fn apply_to_prefab(&mut self, world: &World) -> Result<()> {
        let mut prefab_data = PrefabData::load_from_file(&self.source)?;

        for (prefab_idx, entity) in &self.entity_mapping {
            if let Some(prefab_entity_data) = prefab_data.entities.get_mut(*prefab_idx) {
                if let Some(pose) = world.pose(*entity) {
                    prefab_entity_data.pos_x = pose.pos.x;
                    prefab_entity_data.pos_y = pose.pos.y;
                }

                if let Some(health) = world.health(*entity) {
                    prefab_entity_data.health = health.hp;
                    prefab_entity_data.max_health = health.hp;
                }
            }
        }

        prefab_data.save_to_file(&self.source)?;
        self.template_entities = prefab_data.entities.clone();
        self.overrides.clear();
        Ok(())
    }
}

pub struct PrefabManager {
    instances: Vec<PrefabInstance>,
    prefab_directory: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PrefabUiInfo {
    pub source: PathBuf,
    pub has_overrides: bool,
    pub overrides: Option<EntityOverrides>,
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

    pub fn shared<P: AsRef<Path>>(prefab_directory: P) -> PrefabManagerHandle {
        Arc::new(Mutex::new(Self::new(prefab_directory)))
    }

    pub fn create_prefab(&self, world: &World, entity: Entity, name: &str) -> Result<PathBuf> {
        let prefab_data = PrefabData::from_entity(world, entity, name.to_string())?;
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

    pub fn describe_instance(&mut self, entity: Entity, world: &World) -> Option<PrefabUiInfo> {
        let instance = self.find_instance_mut(entity)?;
        instance.refresh_overrides(world);
        let overrides = instance.overrides.get(&entity).cloned();
        Some(PrefabUiInfo {
            source: instance.source.clone(),
            has_overrides: instance.has_overrides(entity),
            overrides,
        })
    }

    pub fn apply_overrides(&mut self, entity: Entity, world: &World) -> Result<()> {
        let instance = self
            .find_instance_mut(entity)
            .context("Prefab instance not found")?;
        instance.apply_to_prefab(world)?;
        Ok(())
    }

    pub fn revert_overrides(&mut self, entity: Entity, world: &mut World) -> Result<()> {
        let instance = self
            .find_instance_mut(entity)
            .context("Prefab instance not found")?;
        instance.revert_to_prefab(world)?;
        Ok(())
    }

    pub fn find_instance(&self, entity: Entity) -> Option<&PrefabInstance> {
        self.instances
            .iter()
            .find(|inst| inst.contains_entity(entity))
    }

    pub fn find_instance_mut(&mut self, entity: Entity) -> Option<&mut PrefabInstance> {
        self.instances
            .iter_mut()
            .find(|inst| inst.contains_entity(entity))
    }

    pub fn get_all_prefab_files(&self) -> Result<Vec<PathBuf>> {
        let mut prefab_files = Vec::new();

        if !self.prefab_directory.exists() {
            return Ok(prefab_files);
        }

        for entry in std::fs::read_dir(&self.prefab_directory)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                if path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.ends_with(".prefab"))
                    .unwrap_or(false)
                {
                    prefab_files.push(path);
                }
            }
        }

        Ok(prefab_files)
    }

    pub fn instance_path(&self, entity: Entity) -> Option<PathBuf> {
        self.find_instance(entity).map(|inst| inst.source.clone())
    }

    pub fn reload_instance(&mut self, entity: Entity) -> Result<()> {
        let instance = self
            .find_instance_mut(entity)
            .context("Prefab instance not found for reload")?;
        let prefab_data = PrefabData::load_from_file(&instance.source)?;
        instance.template_entities = prefab_data.entities;
        Ok(())
    }

    pub fn track_override_snapshot(
        &mut self,
        entity: Entity,
        pose: Option<Pose>,
        health: Option<Health>,
    ) {
        if pose.is_none() && health.is_none() {
            return;
        }

        if let Some(instance) = self.find_instance_mut(entity) {
            instance.track_override_snapshot(entity, pose, health);
        }
    }

    pub fn refresh_instances(&mut self, world: &World) {
        for instance in &mut self.instances {
            instance.refresh_overrides(world);
        }
    }

    pub fn despawn_instance(&mut self, world: &mut World, entity: Entity) -> Result<()> {
        let index = self
            .instances
            .iter()
            .position(|inst| inst.contains_entity(entity))
            .context("Prefab instance not found for despawn")?;

        let instance = self.instances.remove(index);
        for world_entity in instance.entity_mapping.values() {
            if let Some(pose) = world.pose_mut(*world_entity) {
                *pose = Pose {
                    pos: IVec2 {
                        x: -10000,
                        y: -10000,
                    },
                    rotation: 0.0,
                    rotation_x: 0.0,
                    rotation_z: 0.0,
                    scale: 0.0,
                };
            }
        }
        Ok(())
    }

    pub fn capture_snapshot(
        &self,
        world: &World,
        entity: Entity,
    ) -> Option<Vec<PrefabEntitySnapshot>> {
        let instance = self.find_instance(entity)?;
        let mut snapshot = Vec::new();
        for world_entity in instance.entity_mapping.values() {
            snapshot.push(PrefabEntitySnapshot {
                entity: *world_entity,
                pose: world.pose(*world_entity),
                health: world.health(*world_entity),
            });
        }
        Some(snapshot)
    }

    pub fn restore_snapshot(
        &mut self,
        world: &mut World,
        root_entity: Entity,
        snapshot: &[PrefabEntitySnapshot],
    ) {
        for entry in snapshot {
            if let Some(saved_pose) = entry.pose {
                if let Some(world_pose) = world.pose_mut(entry.entity) {
                    *world_pose = saved_pose;
                }
            }

            if let Some(saved_health) = entry.health {
                if let Some(world_health) = world.health_mut(entry.entity) {
                    *world_health = saved_health;
                }
            }
        }

        if let Some(instance) = self.find_instance_mut(root_entity) {
            instance.refresh_overrides(world);
        }
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
        let prefab = PrefabData {
            name: "TestPrefab".into(),
            entities: vec![PrefabEntityData {
                name: "TestEntity".into(),
                pos_x: 0,
                pos_y: 0,
                team_id: 0,
                health: 100,
                max_health: 100,
                children_indices: vec![],
                prefab_reference: None,
            }],
            root_entity_index: 0,
            version: "1.0".into(),
        };

        let mut instance = prefab.instantiate(&mut world, (0, 0)).unwrap();
        instance.source = PathBuf::from("test.prefab.ron");

        let entity = instance.root_entity;
        if let Some(pose) = world.pose_mut(entity) {
            pose.pos.x = 10;
        }

        instance.track_override(entity, &world);
        assert!(instance.has_overrides(entity));
    }

    #[test]
    fn test_prefab_describe_instance_reports_overrides() {
        let mut world = World::new();
        let prefab = PrefabData {
            name: "TestPrefab".into(),
            entities: vec![PrefabEntityData {
                name: "TestEntity".into(),
                pos_x: 0,
                pos_y: 0,
                team_id: 0,
                health: 100,
                max_health: 100,
                children_indices: vec![],
                prefab_reference: None,
            }],
            root_entity_index: 0,
            version: "1.0".into(),
        };

        let mut instance = prefab.instantiate(&mut world, (0, 0)).unwrap();
        let entity = instance.root_entity;
        instance.source = PathBuf::from("test.prefab.ron");

        if let Some(health) = world.health_mut(entity) {
            health.hp = 80;
        }

        let mut manager = PrefabManager::new(std::env::temp_dir());
        manager.instances.push(instance);

        let info = manager
            .describe_instance(entity, &world)
            .expect("instance info");

        assert!(info.has_overrides);
        let overrides = info.overrides.expect("override payload");
        assert_eq!(overrides.health, Some(80));
    }
}
