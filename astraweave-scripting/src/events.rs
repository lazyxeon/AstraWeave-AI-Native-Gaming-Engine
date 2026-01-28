use astraweave_ecs::{Entity, Event};

#[derive(Clone, Debug)]
pub enum ScriptEvent {
    OnSpawn { entity: Entity },
    OnCollision { entity: Entity, other: Entity },
    OnTrigger { entity: Entity, trigger_name: String },
    OnDamage { entity: Entity, damage: f32, source: Entity },
}

impl Event for ScriptEvent {}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create test entities
    fn entity(id: u64) -> Entity {
        // SAFETY: Test-only usage, entity IDs are not dereferenced
        unsafe { Entity::from_raw(id) }
    }

    // ScriptEvent::OnSpawn tests
    #[test]
    fn test_on_spawn_creation() {
        let event = ScriptEvent::OnSpawn { entity: entity(1) };
        if let ScriptEvent::OnSpawn { entity: e } = event {
            assert_eq!(e, entity(1));
        } else {
            panic!("Expected OnSpawn");
        }
    }

    #[test]
    fn test_on_spawn_clone() {
        let event = ScriptEvent::OnSpawn { entity: entity(42) };
        let cloned = event.clone();
        if let ScriptEvent::OnSpawn { entity: e } = cloned {
            assert_eq!(e, entity(42));
        } else {
            panic!("Expected OnSpawn");
        }
    }

    #[test]
    fn test_on_spawn_debug() {
        let event = ScriptEvent::OnSpawn { entity: entity(100) };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("OnSpawn"));
    }

    // ScriptEvent::OnCollision tests
    #[test]
    fn test_on_collision_creation() {
        let event = ScriptEvent::OnCollision {
            entity: entity(1),
            other: entity(2),
        };
        if let ScriptEvent::OnCollision { entity: e, other: o } = event {
            assert_eq!(e, entity(1));
            assert_eq!(o, entity(2));
        } else {
            panic!("Expected OnCollision");
        }
    }

    #[test]
    fn test_on_collision_symmetric_entities() {
        let e1 = entity(5);
        let e2 = entity(10);
        let event = ScriptEvent::OnCollision {
            entity: e1,
            other: e2,
        };
        if let ScriptEvent::OnCollision { entity, other } = event {
            assert_ne!(entity, other);
        } else {
            panic!("Expected OnCollision");
        }
    }

    #[test]
    fn test_on_collision_same_entity() {
        // Self-collision edge case
        let event = ScriptEvent::OnCollision {
            entity: entity(1),
            other: entity(1),
        };
        if let ScriptEvent::OnCollision { entity, other } = event {
            assert_eq!(entity, other);
        } else {
            panic!("Expected OnCollision");
        }
    }

    // ScriptEvent::OnTrigger tests
    #[test]
    fn test_on_trigger_creation() {
        let event = ScriptEvent::OnTrigger {
            entity: entity(1),
            trigger_name: "door_enter".to_string(),
        };
        if let ScriptEvent::OnTrigger { entity: e, trigger_name } = event {
            assert_eq!(e, entity(1));
            assert_eq!(trigger_name, "door_enter");
        } else {
            panic!("Expected OnTrigger");
        }
    }

    #[test]
    fn test_on_trigger_empty_name() {
        let event = ScriptEvent::OnTrigger {
            entity: entity(1),
            trigger_name: String::new(),
        };
        if let ScriptEvent::OnTrigger { trigger_name, .. } = event {
            assert!(trigger_name.is_empty());
        } else {
            panic!("Expected OnTrigger");
        }
    }

    #[test]
    fn test_on_trigger_special_characters() {
        let event = ScriptEvent::OnTrigger {
            entity: entity(1),
            trigger_name: "zone_A2:exit!".to_string(),
        };
        if let ScriptEvent::OnTrigger { trigger_name, .. } = event {
            assert_eq!(trigger_name, "zone_A2:exit!");
        } else {
            panic!("Expected OnTrigger");
        }
    }

    // ScriptEvent::OnDamage tests
    #[test]
    fn test_on_damage_creation() {
        let event = ScriptEvent::OnDamage {
            entity: entity(1),
            damage: 25.5,
            source: entity(2),
        };
        if let ScriptEvent::OnDamage { entity: e, damage, source } = event {
            assert_eq!(e, entity(1));
            assert!((damage - 25.5).abs() < f32::EPSILON);
            assert_eq!(source, entity(2));
        } else {
            panic!("Expected OnDamage");
        }
    }

    #[test]
    fn test_on_damage_zero() {
        let event = ScriptEvent::OnDamage {
            entity: entity(1),
            damage: 0.0,
            source: entity(2),
        };
        if let ScriptEvent::OnDamage { damage, .. } = event {
            assert!((damage - 0.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected OnDamage");
        }
    }

    #[test]
    fn test_on_damage_negative() {
        // Negative damage = healing
        let event = ScriptEvent::OnDamage {
            entity: entity(1),
            damage: -10.0,
            source: entity(2),
        };
        if let ScriptEvent::OnDamage { damage, .. } = event {
            assert!(damage < 0.0);
        } else {
            panic!("Expected OnDamage");
        }
    }

    #[test]
    fn test_on_damage_self_damage() {
        let e = entity(1);
        let event = ScriptEvent::OnDamage {
            entity: e,
            damage: 5.0,
            source: e,
        };
        if let ScriptEvent::OnDamage { entity, source, .. } = event {
            assert_eq!(entity, source);
        } else {
            panic!("Expected OnDamage");
        }
    }

    // Pattern matching and variant exhaustiveness
    #[test]
    fn test_pattern_matching() {
        let events = vec![
            ScriptEvent::OnSpawn { entity: entity(1) },
            ScriptEvent::OnCollision { entity: entity(2), other: entity(3) },
            ScriptEvent::OnTrigger { entity: entity(4), trigger_name: "test".into() },
            ScriptEvent::OnDamage { entity: entity(5), damage: 10.0, source: entity(6) },
        ];

        let mut spawn_count = 0;
        let mut collision_count = 0;
        let mut trigger_count = 0;
        let mut damage_count = 0;

        for event in events {
            match event {
                ScriptEvent::OnSpawn { .. } => spawn_count += 1,
                ScriptEvent::OnCollision { .. } => collision_count += 1,
                ScriptEvent::OnTrigger { .. } => trigger_count += 1,
                ScriptEvent::OnDamage { .. } => damage_count += 1,
            }
        }

        assert_eq!(spawn_count, 1);
        assert_eq!(collision_count, 1);
        assert_eq!(trigger_count, 1);
        assert_eq!(damage_count, 1);
    }
}
