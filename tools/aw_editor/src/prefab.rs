use anyhow::{Context, Result};
use astraweave_core::{Entity, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    pub overrides: HashMap<Entity, EntityOverrides>,
}

#[derive(Debug, Clone, Default)]
pub struct EntityOverrides {
    pub pos_x: Option<i32>,
    pub pos_y: Option<i32>,
    pub health: Option<i32>,
    pub max_health: Option<i32>,
}

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
        let overrides = self.overrides.entry(entity).or_default();

        if let Some(pose) = world.pose(entity) {
            overrides.pos_x = Some(pose.pos.x);
            overrides.pos_y = Some(pose.pos.y);
        }

        if let Some(health) = world.health(entity) {
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
        Ok(())
    }

    pub fn apply_to_prefab(&self, world: &World) -> Result<()> {
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
        Ok(())
    }
}

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
}
