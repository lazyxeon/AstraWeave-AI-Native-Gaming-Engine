use anyhow::{Context, Result};
use astraweave_core::{Entity, IVec2, World};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardEntityData {
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
    pub behavior_graph: Option<astraweave_behavior::BehaviorGraph>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClipboardData {
    pub entities: Vec<ClipboardEntityData>,
}

impl ClipboardData {
    pub fn from_entities(world: &World, entity_ids: &[Entity]) -> Self {
        let mut entities = Vec::new();

        for &entity_id in entity_ids {
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
            
            let behavior_graph = world.behavior_graph(entity_id).cloned();

            entities.push(ClipboardEntityData {
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
                behavior_graph,
            });
        }

        ClipboardData { entities }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).context("Failed to serialize clipboard data")
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to deserialize clipboard data")
    }

    pub fn spawn_entities(&self, world: &mut World, offset: IVec2) -> Result<Vec<Entity>> {
        let mut spawned = Vec::new();

        for entity_data in &self.entities {
            let new_pos = IVec2 {
                x: entity_data.pos.x + offset.x,
                y: entity_data.pos.y + offset.y,
            };

            let id = world.spawn(
                &entity_data.name,
                new_pos,
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

            if let Some(bg) = &entity_data.behavior_graph {
                world.set_behavior_graph(id, bg.clone());
            }

            spawned.push(id);
        }

        Ok(spawned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_from_entities() {
        let mut world = World::new();
        let e1 = world.spawn(
            "Entity1",
            IVec2 { x: 5, y: 10 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );
        let e2 = world.spawn(
            "Entity2",
            IVec2 { x: 15, y: 20 },
            astraweave_core::Team { id: 1 },
            80,
            20,
        );

        let clipboard = ClipboardData::from_entities(&world, &[e1, e2]);

        assert_eq!(clipboard.entities.len(), 2);
        assert_eq!(clipboard.entities[0].name, "Entity1");
        assert_eq!(clipboard.entities[1].name, "Entity2");
    }

    #[test]
    fn test_clipboard_json_serialization() {
        let mut world = World::new();
        let e1 = world.spawn(
            "TestEntity",
            IVec2 { x: 0, y: 0 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );

        let clipboard = ClipboardData::from_entities(&world, &[e1]);
        let json = clipboard.to_json().unwrap();

        assert!(json.contains("TestEntity"));

        let restored = ClipboardData::from_json(&json).unwrap();
        assert_eq!(restored.entities.len(), 1);
        assert_eq!(restored.entities[0].name, "TestEntity");
    }

    #[test]
    fn test_spawn_entities_with_offset() {
        let mut world = World::new();
        let e1 = world.spawn(
            "Original",
            IVec2 { x: 10, y: 10 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );

        let clipboard = ClipboardData::from_entities(&world, &[e1]);

        let spawned = clipboard
            .spawn_entities(&mut world, IVec2 { x: 5, y: 5 })
            .unwrap();

        assert_eq!(spawned.len(), 1);

        let new_pos = world.pose(spawned[0]).unwrap().pos;
        assert_eq!(new_pos.x, 15);
        assert_eq!(new_pos.y, 15);
    }

    #[test]
    fn test_multiple_entities_spawn() {
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
            IVec2 { x: 5, y: 5 },
            astraweave_core::Team { id: 1 },
            50,
            15,
        );

        let clipboard = ClipboardData::from_entities(&world, &[e1, e2]);
        let spawned = clipboard
            .spawn_entities(&mut world, IVec2 { x: 10, y: 10 })
            .unwrap();

        assert_eq!(spawned.len(), 2);
        assert_eq!(world.entities().len(), 4);
    }

    #[test]
    fn test_preserve_all_properties() {
        let mut world = World::new();
        let entity = world.spawn(
            "CompleteEntity",
            IVec2 { x: 5, y: 10 },
            astraweave_core::Team { id: 2 },
            75,
            25,
        );

        if let Some(pose) = world.pose_mut(entity) {
            pose.rotation = 1.57;
            pose.rotation_x = 0.78;
            pose.rotation_z = 0.39;
            pose.scale = 2.5;
        }

        let clipboard = ClipboardData::from_entities(&world, &[entity]);
        let entity_data = &clipboard.entities[0];

        assert_eq!(entity_data.name, "CompleteEntity");
        assert_eq!(entity_data.hp, 75);
        assert_eq!(entity_data.team_id, 2);
        assert_eq!(entity_data.ammo, 25);
        assert!((entity_data.rotation - 1.57).abs() < 0.01);
        assert!((entity_data.scale - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_empty_clipboard() {
        let world = World::new();
        let clipboard = ClipboardData::from_entities(&world, &[]);

        assert_eq!(clipboard.entities.len(), 0);

        let json = clipboard.to_json().unwrap();
        let restored = ClipboardData::from_json(&json).unwrap();
        assert_eq!(restored.entities.len(), 0);
    }

    #[test]
    fn test_behavior_graph_preservation() {
        use astraweave_behavior::{BehaviorGraph, BehaviorNode};

        let mut world = World::new();
        let entity = world.spawn(
            "AIEntity",
            IVec2 { x: 0, y: 0 },
            astraweave_core::Team { id: 0 },
            100,
            30,
        );

        // Create a simple behavior graph
        let root = BehaviorNode::Sequence(vec![
            BehaviorNode::Action("patrol".into()),
            BehaviorNode::Action("attack".into()),
        ]);
        let graph = BehaviorGraph::new(root);
        world.set_behavior_graph(entity, graph);

        // Copy to clipboard
        let clipboard = ClipboardData::from_entities(&world, &[entity]);
        assert!(clipboard.entities[0].behavior_graph.is_some());

        // Spawn from clipboard
        let spawned = clipboard
            .spawn_entities(&mut world, IVec2 { x: 10, y: 10 })
            .unwrap();

        // Verify BehaviorGraph was restored
        let restored_graph = world.behavior_graph(spawned[0]);
        assert!(restored_graph.is_some(), "BehaviorGraph should be restored after paste");
    }
}
