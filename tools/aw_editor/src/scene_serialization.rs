use anyhow::{Context, Result};
use astraweave_core::{Entity, IVec2, World};
use astraweave_security::path::{safe_under, validate_extension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityData {
    pub id: Entity,
    pub name: String,
    pub pos: IVec2,
    pub rotation: f32,
    pub rotation_x: f32,
    pub rotation_z: f32,
    pub scale: f32,
    pub hp: i32,
    pub team_id: u8,
    pub ammo: i32,
    pub cooldowns: HashMap<String, f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneData {
    pub version: u32,
    pub time: f32,
    pub next_entity_id: Entity,
    pub entities: Vec<EntityData>,
    pub obstacles: Vec<(i32, i32)>,
}

impl SceneData {
    pub fn from_world(world: &World) -> Self {
        let mut entities = Vec::new();

        for entity_id in world.entities() {
            let pose = world.pose(entity_id).unwrap_or(astraweave_core::Pose {
                pos: IVec2 { x: 0, y: 0 },
                rotation: 0.0,
                rotation_x: 0.0,
                rotation_z: 0.0,
                scale: 1.0,
            });

            let health = world.health(entity_id).map(|h| h.hp).unwrap_or(100);
            let team = world.team(entity_id).map(|t| t.id).unwrap_or(0);
            let ammo_rounds = world.ammo(entity_id).map(|a| a.rounds).unwrap_or(0);
            let name = world.name(entity_id).unwrap_or("Unnamed").to_string();

            let cooldowns = world
                .cooldowns(entity_id)
                .map(|cd| cd.map.clone())
                .unwrap_or_default();

            entities.push(EntityData {
                id: entity_id,
                name,
                pos: pose.pos,
                rotation: pose.rotation,
                rotation_x: pose.rotation_x,
                rotation_z: pose.rotation_z,
                scale: pose.scale,
                hp: health,
                team_id: team,
                ammo: ammo_rounds,
                cooldowns,
            });
        }

        let obstacles: Vec<(i32, i32)> = world.obstacles.iter().copied().collect();

        SceneData {
            version: 1,
            time: world.t,
            next_entity_id: world.next_id,
            entities,
            obstacles,
        }
    }

    pub fn to_world(&self) -> World {
        let mut world = World::new();
        world.t = self.time;
        // We'll restore entities using their original IDs, then restore next_id
        let desired_next_id = self.next_entity_id;

        world.obstacles = self.obstacles.iter().copied().collect();

        for entity_data in &self.entities {
            // Ensure the next spawned entity reuses the recorded ID
            world.next_id = entity_data.id;
            let id = world.spawn(
                &entity_data.name,
                entity_data.pos,
                astraweave_core::Team {
                    id: entity_data.team_id,
                },
                entity_data.hp,
                entity_data.ammo,
            );

            if let Some(pose) = world.pose_mut(id) {
                pose.rotation = entity_data.rotation;
                pose.rotation_x = entity_data.rotation_x;
                pose.rotation_z = entity_data.rotation_z;
                pose.scale = entity_data.scale;
            }

            if let Some(cooldowns) = world.cooldowns_mut(id) {
                cooldowns.map = entity_data.cooldowns.clone();
            }
        }

        // Restore next entity id exactly as recorded
        world.next_id = desired_next_id;

        world
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Security: Validate path is within content directory and has correct extension
        let base = std::env::current_dir().context("Failed to get current directory")?;
        let content_base = base.join("content");

        // Create content directory if it doesn't exist
        fs::create_dir_all(&content_base).context("Failed to create content directory")?;

        // Validate path is within content directory
        let safe_path = safe_under(&content_base, path)
            .map_err(|e| anyhow::anyhow!("Invalid scene path: {}", e))?;

        // Validate extension
        validate_extension(&safe_path, &["ron", "json", "toml"])
            .map_err(|e| anyhow::anyhow!("Invalid scene file extension: {}", e))?;

        let ron_string = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .context("Failed to serialize scene to RON")?;

        if let Some(parent) = safe_path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create directory: {:?}", parent))?;
        }

        fs::write(&safe_path, ron_string)
            .context(format!("Failed to write scene to {:?}", safe_path))?;

        println!("💾 Saved scene to {:?}", safe_path);
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        // Security: Validate path is within content directory and has correct extension
        let base = std::env::current_dir().context("Failed to get current directory")?;
        let content_base = base.join("content");

        // Validate path is within content directory
        let safe_path = safe_under(&content_base, path)
            .map_err(|e| anyhow::anyhow!("Invalid scene path: {}", e))?;

        // Validate extension
        validate_extension(&safe_path, &["ron", "json", "toml"])
            .map_err(|e| anyhow::anyhow!("Invalid scene file extension: {}", e))?;

        let contents = fs::read_to_string(&safe_path)
            .context(format!("Failed to read scene from {:?}", safe_path))?;

        let scene: SceneData = ron::from_str(&contents)
            .context(format!("Failed to deserialize scene from {:?}", safe_path))?;

        println!("📂 Loaded scene from {:?}", safe_path);
        Ok(scene)
    }
}

pub fn save_scene(world: &World, path: &Path) -> Result<()> {
    let scene = SceneData::from_world(world);
    scene.save_to_file(path)
}

pub fn load_scene(path: &Path) -> Result<World> {
    let scene = SceneData::load_from_file(path)?;
    Ok(scene.to_world())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn test_scene_path(name: &str) -> PathBuf {
        let relative = PathBuf::from(format!("test_scenes/{name}"));
        let actual = Path::new("content").join(&relative);
        if let Some(parent) = actual.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        relative
    }

    fn remove_scene_file(relative: &Path) {
        let actual = Path::new("content").join(relative);
        let _ = fs::remove_file(actual);
    }

    #[test]
    fn test_scene_roundtrip() {
        let mut world = World::new();
        let e1 = world.spawn(
            "Player",
            IVec2 { x: 5, y: 10 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );
        let e2 = world.spawn(
            "Enemy",
            IVec2 { x: 20, y: 15 },
            astraweave_core::Team { id: 2 },
            50,
            15,
        );

        world.obstacles.insert((10, 10));
        world.obstacles.insert((11, 10));

        if let Some(pose) = world.pose_mut(e1) {
            pose.rotation = 1.57;
            pose.scale = 2.0;
        }

        let scene = SceneData::from_world(&world);
        let restored_world = scene.to_world();

        assert_eq!(restored_world.entities().len(), 2);
        assert_eq!(restored_world.obstacles.len(), 2);
        assert!(restored_world.obstacle(IVec2 { x: 10, y: 10 }));
        assert_eq!(restored_world.name(1), Some("Player"));
        assert_eq!(restored_world.name(2), Some("Enemy"));

        let pose = restored_world.pose(1).unwrap();
        assert_eq!(pose.pos, IVec2 { x: 5, y: 10 });
        assert!((pose.rotation - 1.57).abs() < 0.01);
        assert!((pose.scale - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_scene_serialization() {
        let mut world = World::new();
        world.spawn(
            "TestEntity",
            IVec2 { x: 0, y: 0 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );

        let scene = SceneData::from_world(&world);
        let ron_string =
            ron::ser::to_string_pretty(&scene, ron::ser::PrettyConfig::default()).unwrap();

        assert!(ron_string.contains("version"));
        assert!(ron_string.contains("TestEntity"));
        assert!(ron_string.contains("entities"));

        let deserialized: SceneData = ron::from_str(&ron_string).unwrap();
        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.entities.len(), 1);
        assert_eq!(deserialized.entities[0].name, "TestEntity");
    }

    #[test]
    fn test_empty_scene() {
        let world = World::new();
        let scene = SceneData::from_world(&world);

        assert_eq!(scene.entities.len(), 0);
        assert_eq!(scene.obstacles.len(), 0);
        assert_eq!(scene.version, 1);

        let restored = scene.to_world();
        assert_eq!(restored.entities().len(), 0);
    }

    #[test]
    fn test_scene_with_multiple_entities() {
        let mut world = World::new();

        for i in 0..10 {
            world.spawn(
                &format!("Entity{}", i),
                IVec2 { x: i, y: i * 2 },
                astraweave_core::Team { id: (i % 3) as u8 },
                100 - i,
                30 + i,
            );
        }

        let scene = SceneData::from_world(&world);
        assert_eq!(scene.entities.len(), 10);

        let restored = scene.to_world();
        assert_eq!(restored.entities().len(), 10);

        let restored_entities = restored.entities();
        for i in 0..10 {
            let entity_name = format!("Entity{}", i);
            let found = restored_entities
                .iter()
                .copied()
                .find(|&id| restored.name(id) == Some(entity_name.as_str()));
            assert!(found.is_some(), "Entity {} should exist", i);
        }
    }

    #[test]
    fn test_scene_with_all_components() {
        let mut world = World::new();
        let entity = world.spawn(
            "CompleteEntity",
            IVec2 { x: 5, y: 10 },
            astraweave_core::Team { id: 1 },
            75,
            25,
        );

        if let Some(pose) = world.pose_mut(entity) {
            pose.rotation = 3.14;
            pose.rotation_x = 1.57;
            pose.rotation_z = 0.78;
            pose.scale = 1.5;
        }

        let scene = SceneData::from_world(&world);
        let restored = scene.to_world();

        let entity_data = &scene.entities[0];
        assert_eq!(entity_data.name, "CompleteEntity");
        assert_eq!(entity_data.pos.x, 5);
        assert_eq!(entity_data.pos.y, 10);
        assert_eq!(entity_data.team_id, 1);
        assert_eq!(entity_data.hp, 75);
        assert_eq!(entity_data.ammo, 25);
        assert!((entity_data.rotation - 3.14).abs() < 0.01);
        assert!((entity_data.rotation_x - 1.57).abs() < 0.01);
        assert!((entity_data.rotation_z - 0.78).abs() < 0.01);
        assert!((entity_data.scale - 1.5).abs() < 0.01);

        let restored_pose = restored.pose(entity).unwrap();
        assert!((restored_pose.rotation - 3.14).abs() < 0.01);
        assert!((restored_pose.scale - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_scene_with_obstacles() {
        let mut world = World::new();

        for x in 0..5 {
            for y in 0..5 {
                world.obstacles.insert((x, y));
            }
        }

        let scene = SceneData::from_world(&world);
        assert_eq!(scene.obstacles.len(), 25);

        let restored = scene.to_world();
        assert_eq!(restored.obstacles.len(), 25);

        for x in 0..5 {
            for y in 0..5 {
                assert!(restored.obstacle(IVec2 { x, y }));
            }
        }
    }

    #[test]
    fn test_scene_file_io() {
        let mut world = World::new();
        world.spawn(
            "Player",
            IVec2 { x: 10, y: 20 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );
        world.obstacles.insert((5, 5));

        let scene_path = test_scene_path("aw_editor_test_scene.ron");

        let scene = SceneData::from_world(&world);
        scene.save_to_file(&scene_path).unwrap();

        assert!(Path::new("content").join(&scene_path).exists());

        let loaded_scene = SceneData::load_from_file(&scene_path).unwrap();
        assert_eq!(loaded_scene.entities.len(), 1);
        assert_eq!(loaded_scene.obstacles.len(), 1);
        assert_eq!(loaded_scene.entities[0].name, "Player");

        remove_scene_file(&scene_path);
    }

    #[test]
    fn test_save_and_load_scene() {
        let mut world = World::new();
        world.spawn(
            "Entity1",
            IVec2 { x: 1, y: 2 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );
        world.spawn(
            "Entity2",
            IVec2 { x: 3, y: 4 },
            astraweave_core::Team { id: 1 },
            50,
            15,
        );

        let scene_path = test_scene_path("aw_editor_test_save_load.ron");

        save_scene(&world, &scene_path).unwrap();
        assert!(Path::new("content").join(&scene_path).exists());

        let loaded_world = load_scene(&scene_path).unwrap();
        assert_eq!(loaded_world.entities().len(), 2);

        remove_scene_file(&scene_path);
    }

    #[test]
    fn test_scene_preserves_world_time() {
        let mut world = World::new();
        world.t = 123.45;

        let scene = SceneData::from_world(&world);
        assert!((scene.time - 123.45).abs() < 0.01);

        let restored = scene.to_world();
        assert!((restored.t - 123.45).abs() < 0.01);
    }

    #[test]
    fn test_scene_preserves_entity_ids() {
        let mut world = World::new();
        let e1 = world.spawn(
            "E1",
            IVec2 { x: 0, y: 0 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );
        let e2 = world.spawn(
            "E2",
            IVec2 { x: 1, y: 1 },
            astraweave_core::Team { id: 1 },
            50,
            15,
        );

        let scene = SceneData::from_world(&world);
        let restored = scene.to_world();

        assert!(restored.pose(e1).is_some());
        assert!(restored.pose(e2).is_some());
        assert_eq!(restored.name(e1), Some("E1"));
        assert_eq!(restored.name(e2), Some("E2"));
    }
}
