use crate::entity_manager::{EditorEntity, EntityId};
use crate::gizmo::scene_viewport::Transform;
use crate::gizmo::snapping::SnappingConfig;
use crate::gizmo::state::TransformSnapshot;
use crate::telemetry::{self, EditorTelemetryEvent};
use astraweave_core::{Entity, IVec2, Pose, World};
use glam::{EulerRot, Quat, Vec3};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use tracing::{info, info_span};

#[derive(Clone)]
pub struct SnapEventHub {
    inner: Arc<Mutex<SnappingConfig>>,
    listeners: Arc<Mutex<Vec<mpsc::Sender<SnappingConfig>>>>,
}

impl SnapEventHub {
    pub fn new(config: SnappingConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(config)),
            listeners: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn current(&self) -> SnappingConfig {
        *self.inner.lock().expect("snap hub poisoned")
    }

    pub fn update<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut SnappingConfig),
    {
        let mut guard = self.inner.lock().expect("snap hub poisoned");
        let before = *guard;
        update_fn(&mut guard);
        let latest = *guard;
        drop(guard);

        if before != latest {
            let span = info_span!(
                "aw_editor.grid.update",
                grid_enabled = tracing::field::Empty,
                snap_size = tracing::field::Empty,
                angle_enabled = tracing::field::Empty,
                angle_increment = tracing::field::Empty
            );
            span.record("grid_enabled", latest.grid_enabled);
            span.record("snap_size", latest.grid_size);
            span.record("angle_enabled", latest.angle_enabled);
            span.record("angle_increment", latest.angle_increment);
            let _guard = span.enter();
            info!(
                target: "aw_editor::snapping",
                grid_enabled = latest.grid_enabled,
                snap_size = latest.grid_size,
                angle_enabled = latest.angle_enabled,
                angle_increment = latest.angle_increment,
                "snapping_config_changed"
            );
            telemetry::record(EditorTelemetryEvent::GridSettingsChanged {
                grid_enabled: latest.grid_enabled,
                snap_size: latest.grid_size,
                angle_enabled: latest.angle_enabled,
                angle_increment: latest.angle_increment,
            });
        }

        let mut listeners = self.listeners.lock().expect("snap hub listeners poisoned");
        listeners.retain(|tx| tx.send(latest).is_ok());
    }

    pub fn subscribe(&self) -> mpsc::Receiver<SnappingConfig> {
        let (tx, rx) = mpsc::channel();
        // Seed subscriber with current config
        let _ = tx.send(self.current());
        self.listeners
            .lock()
            .expect("snap hub listeners poisoned")
            .push(tx);
        rx
    }
}

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
    snap_hub: SnapEventHub,
}

impl EditorSceneState {
    /// Construct scene state from an existing world snapshot.
    pub fn new(world: World) -> Self {
        let mut state = Self {
            world,
            cache: HashMap::new(),
            snap_hub: SnapEventHub::new(SnappingConfig::default()),
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

    /// Current snapping configuration.
    pub fn snapping_config(&self) -> SnappingConfig {
        self.snap_hub.current()
    }

    /// Update snapping settings and broadcast to listeners.
    pub fn update_snapping<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut SnappingConfig),
    {
        self.snap_hub.update(update_fn);
    }

    /// Subscribe to snapping updates (used by renderer/viewport).
    pub fn subscribe_snap_changes(&self) -> mpsc::Receiver<SnappingConfig> {
        self.snap_hub.subscribe()
    }

    /// Share the underlying hub for systems that need live access.
    pub fn snapping_hub(&self) -> SnapEventHub {
        self.snap_hub.clone()
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

impl TransformableScene for World {
    fn world(&self) -> &World {
        self
    }

    fn world_mut(&mut self) -> &mut World {
        self
    }

    fn sync_all(&mut self) {}

    fn sync_entity(&mut self, _entity: Entity) {}

    fn snapshot_for(&self, entity: Entity) -> Option<TransformSnapshot> {
        self.pose(entity).map(|pose| TransformSnapshot {
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
        if let Some(pose) = self.pose_mut(entity) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry;
    use crate::telemetry::EditorTelemetryEvent;
    use astraweave_core::World;

    #[test]
    fn snapping_updates_emit_grid_event() {
        let world = World::new();
        let state = EditorSceneState::new(world);
        let _guard = telemetry::enable_capture();

        state.update_snapping(|cfg| {
            cfg.grid_enabled = false;
            cfg.grid_size = 2.0;
            cfg.angle_enabled = false;
            cfg.angle_increment = 45.0;
        });

        let events = telemetry::drain_captured_events();
        assert!(events.iter().any(|event| matches!(
            event,
            EditorTelemetryEvent::GridSettingsChanged {
                grid_enabled,
                snap_size,
                angle_enabled,
                angle_increment,
            } if !grid_enabled
                && (*snap_size - 2.0).abs() < f32::EPSILON
                && !angle_enabled
                && (*angle_increment - 45.0).abs() < f32::EPSILON
        )));
    }
}
