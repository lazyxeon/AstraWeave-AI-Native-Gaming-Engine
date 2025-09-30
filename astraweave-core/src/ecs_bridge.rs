use astraweave_ecs as ecs;
use std::collections::BTreeMap;

/// Public bridge resource mapping legacy `Entity` ids from `World` to ECS entity handles.
/// Also stores the reverse mapping (ecs -> legacy) for round-trip lookups.
#[derive(Default, Debug)]
pub struct EntityBridge {
    // legacy id -> ecs::Entity
    map: BTreeMap<crate::Entity, ecs::Entity>,
    // ecs::Entity -> legacy id
    rev: BTreeMap<ecs::Entity, crate::Entity>,
}

impl EntityBridge {
    /// Insert a pair mapping and maintain the reverse index.
    pub fn insert_pair(&mut self, legacy: crate::Entity, ecs_e: ecs::Entity) {
        // If this legacy id was previously mapped to another ecs entity, remove that reverse mapping
        if let Some(old_ecs) = self.map.get(&legacy).copied() {
            if old_ecs != ecs_e {
                self.rev.remove(&old_ecs);
            }
        }
        // If this ecs entity was previously mapped to another legacy id, remove that forward mapping
        if let Some(old_legacy) = self.rev.get(&ecs_e).copied() {
            if old_legacy != legacy {
                self.map.remove(&old_legacy);
            }
        }

        // Insert the new pair into both maps
        self.map.insert(legacy, ecs_e);
        self.rev.insert(ecs_e, legacy);
    }

    /// Alias with clearer naming: insert a mapping and maintain reverse index.
    pub fn insert(&mut self, legacy: crate::Entity, ecs_e: ecs::Entity) {
        self.insert_pair(legacy, ecs_e);
    }

    /// Remove mapping by legacy id, cleaning up reverse index.
    pub fn remove_legacy(&mut self, legacy: &crate::Entity) {
        if let Some(e) = self.map.remove(legacy) {
            self.rev.remove(&e);
        }
    }

    /// Remove mapping by ECS entity, cleaning up forward index.
    pub fn remove_ecs(&mut self, ecs_e: &ecs::Entity) {
        if let Some(legacy) = self.rev.remove(ecs_e) {
            self.map.remove(&legacy);
        }
    }

    pub fn get(&self, legacy: &crate::Entity) -> Option<ecs::Entity> {
        self.map.get(legacy).copied()
    }

    pub fn get_legacy(&self, ecs_e: &ecs::Entity) -> Option<crate::Entity> {
        self.rev.get(ecs_e).copied()
    }

    /// Get ECS entity by legacy id (named accessor)
    pub fn get_by_legacy(&self, legacy: &crate::Entity) -> Option<ecs::Entity> {
        self.get(legacy)
    }

    /// Get legacy id by ECS entity (named accessor)
    pub fn get_by_ecs(&self, ecs_e: &ecs::Entity) -> Option<crate::Entity> {
        self.get_legacy(ecs_e)
    }

    /// Remove mapping by legacy id (named)
    pub fn remove_by_legacy(&mut self, legacy: &crate::Entity) {
        self.remove_legacy(legacy);
    }

    /// Remove mapping by ECS entity (named)
    pub fn remove_by_ecs(&mut self, ecs_e: &ecs::Entity) {
        self.remove_ecs(ecs_e);
    }

    /// Return a list of ECS entities currently referenced by the bridge.
    pub fn ecs_entities(&self) -> Vec<ecs::Entity> {
        self.rev.keys().copied().collect()
    }
}
