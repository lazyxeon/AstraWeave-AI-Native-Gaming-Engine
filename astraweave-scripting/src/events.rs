use astraweave_ecs::{Entity, Event};

#[derive(Clone, Debug)]
pub enum ScriptEvent {
    OnSpawn { entity: Entity },
    OnCollision { entity: Entity, other: Entity },
    OnTrigger { entity: Entity, trigger_name: String },
    OnDamage { entity: Entity, damage: f32, source: Entity },
}

impl Event for ScriptEvent {}
