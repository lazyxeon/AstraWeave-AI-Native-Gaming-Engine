use crate::entity_manager::{EditorEntity, EntityId};
use crate::gizmo::scene_viewport::Transform;
use crate::gizmo::state::TransformSnapshot;
use astraweave_core::{Entity, IVec2, Pose, World};
use glam::{EulerRot, Quat, Vec3};
use std::collections::HashMap;

/// Abstraction used by the viewport to read and mutate scene data
/// without knowing about the concrete storage implementation.
pub trait TransformableScene {
    /// Access the underlying simulation world (read-only).
    fn world(&self) -> &World;

    /// Access the underlying simulation world (mutable).
    fn world_mut(&mut self) -> &mut World;

    /// Rebuild every cached entity (expensive; use after bulk edits).
    fn sync_all(&mut self);

    /// Rebuild a cached snapshot for the given entity so UI surfaces stay
    /// in sync with the authoritative `World` after edits.
    fn sync_entity(&mut self, entity: Entity);

    /// Build a transform snapshot for the requested entity.
    fn snapshot_for(&self, entity: Entity) -> Option<TransformSnapshot>;

    /// Apply a previously captured snapshot back onto the world/cache.
    fn apply_snapshot(&mut self, entity: Entity, snapshot: &TransformSnapshot);
}

/// Canonical scene data for Edit mode.
///
/// `EditorSceneState` owns the authoritative `World` plus lightweight UI caches
/// that expose 3D transforms to egui panels and the gizmo pipeline.
pub struct EditorSceneState {
    world: World,
    cache: HashMap<Entity, EditorEntity>,
}

impl EditorSceneState {
    /// Construct scene state from an existing world snapshot.
    pub fn new(world: World) -> Self {
        let mut state = Self {
            world,
            cache: HashMap::new(),
        };
        state.sync_all();
        state
    }

    /// Immutable access to the ECS world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Mutable access to the ECS world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Transform helper exposed to panels (position in 3D space).
    pub fn transform_for(&self, entity: Entity) -> Option<Transform> {
        self.world.pose(entity).map(pose_to_transform)
    }

    /// Apply a panel-authored transform back into the world/cache.
    pub fn apply_transform(&mut self, entity: Entity, transform: &Transform) {
        if let Some(pose) = self.world.pose_mut(entity) {
            pose.pos = IVec2 {
                x: transform.position.x.round() as i32,
                y: transform.position.z.round() as i32,
            };
            let (rx, ry, rz) = transform.rotation.to_euler(EulerRot::XYZ);
            pose.rotation_x = rx;
            pose.rotation = ry;
            pose.rotation_z = rz;
            pose.scale = transform.scale.x;
        }
        self.sync_entity(entity);
    }

    /// Sync every cached entity (used when loading a scene).
    pub fn sync_all(&mut self) {
        let entities = self.world.entities();
        for entity in entities {
            self.sync_entity(entity);
        }
    }

    fn upsert_cache_entry(&mut self, entity: Entity) -> Option<&mut EditorEntity> {
        let pose = match self.world.pose(entity) {
            Some(pose) => pose,
            None => {
                self.cache.remove(&entity);
                return None;
            }
        };

        let name = self
            .world
            .name(entity)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Entity_{}", entity));

        let entry = self.cache.entry(entity).or_insert_with(|| EditorEntity {
            id: entity as EntityId,
            name,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            mesh: None,
            components: HashMap::new(),
        });

        entry.position = Vec3::new(pose.pos.x as f32, 1.0, pose.pos.y as f32);
        entry.rotation = Quat::from_euler(
            EulerRot::XYZ,
            pose.rotation_x,
            pose.rotation,
            pose.rotation_z,
        );
        entry.scale = Vec3::splat(pose.scale);

        Some(entry)
    }
}

impl TransformableScene for EditorSceneState {
    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    fn sync_all(&mut self) {
        EditorSceneState::sync_all(self);
    }

    fn sync_entity(&mut self, entity: Entity) {
        if self.upsert_cache_entry(entity).is_none() {
            self.cache.remove(&entity);
        }
    }

    fn snapshot_for(&self, entity: Entity) -> Option<TransformSnapshot> {
        self.world.pose(entity).map(|pose| TransformSnapshot {
            position: Vec3::new(pose.pos.x as f32, 1.0, pose.pos.y as f32),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                pose.rotation_x,
                pose.rotation,
                pose.rotation_z,
            ),
            scale: Vec3::splat(pose.scale),
        })
    }

    fn apply_snapshot(&mut self, entity: Entity, snapshot: &TransformSnapshot) {
        if let Some(pose) = self.world.pose_mut(entity) {
            pose.pos = IVec2 {
                x: snapshot.position.x.round() as i32,
                y: snapshot.position.z.round() as i32,
            };
            let (rx, ry, rz) = snapshot.rotation.to_euler(EulerRot::XYZ);
            pose.rotation_x = rx;
            pose.rotation = ry;
            pose.rotation_z = rz;
            pose.scale = snapshot.scale.x;
        }
        self.sync_entity(entity);
    }
}

fn pose_to_transform(pose: Pose) -> Transform {
    Transform {
        position: Vec3::new(pose.pos.x as f32, 1.0, pose.pos.y as f32),
        rotation: Quat::from_euler(
            EulerRot::XYZ,
            pose.rotation_x,
            pose.rotation,
            pose.rotation_z,
        ),
        scale: Vec3::splat(pose.scale),
    }
}
