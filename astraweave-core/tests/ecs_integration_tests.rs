//! Comprehensive tests for astraweave-core ECS integration
//! Tests cover: EntityBridge (ecs_bridge.rs), component types (ecs_components.rs), events (ecs_events.rs)

use astraweave_core::{ecs_bridge::EntityBridge, ecs_components::*, ecs_events::*, IVec2};
use astraweave_ecs as ecs;

// ============================================================================
// EntityBridge Tests (ecs_bridge.rs)
// ============================================================================

#[test]
fn test_entity_bridge_insert_and_get() {
    let mut bridge = EntityBridge::default();
    let mut world = ecs::World::new();

    let ecs_entity = world.spawn();
    let legacy_id = 42u32;

    bridge.insert(legacy_id, ecs_entity);

    assert_eq!(bridge.get(&legacy_id), Some(ecs_entity));
    assert_eq!(bridge.get_legacy(&ecs_entity), Some(legacy_id));
}

#[test]
fn test_entity_bridge_insert_pair() {
    let mut bridge = EntityBridge::default();
    let mut world = ecs::World::new();

    let ecs_entity = world.spawn();
    let legacy_id = 100u32;

    bridge.insert_pair(legacy_id, ecs_entity);

    assert_eq!(bridge.get_by_legacy(&legacy_id), Some(ecs_entity));
    assert_eq!(bridge.get_by_ecs(&ecs_entity), Some(legacy_id));
}

#[test]
fn test_entity_bridge_overwrite_mapping() {
    let mut bridge = EntityBridge::default();
    let mut world = ecs::World::new();

    let ecs_entity1 = world.spawn();
    let ecs_entity2 = world.spawn();
    let legacy_id = 50u32;

    // Insert first mapping
    bridge.insert(legacy_id, ecs_entity1);
    assert_eq!(bridge.get(&legacy_id), Some(ecs_entity1));

    // Overwrite with new ECS entity
    bridge.insert(legacy_id, ecs_entity2);
    assert_eq!(bridge.get(&legacy_id), Some(ecs_entity2));
    assert_eq!(bridge.get_legacy(&ecs_entity1), None); // Old mapping removed
    assert_eq!(bridge.get_legacy(&ecs_entity2), Some(legacy_id));
}

#[test]
fn test_entity_bridge_remove_by_legacy() {
    let mut bridge = EntityBridge::default();
    let mut world = ecs::World::new();

    let ecs_entity = world.spawn();
    let legacy_id = 75u32;

    bridge.insert(legacy_id, ecs_entity);
    assert_eq!(bridge.get(&legacy_id), Some(ecs_entity));

    bridge.remove_by_legacy(&legacy_id);
    assert_eq!(bridge.get(&legacy_id), None);
    assert_eq!(bridge.get_legacy(&ecs_entity), None);
}

#[test]
fn test_entity_bridge_remove_by_ecs() {
    let mut bridge = EntityBridge::default();
    let mut world = ecs::World::new();

    let ecs_entity = world.spawn();
    let legacy_id = 88u32;

    bridge.insert(legacy_id, ecs_entity);
    assert_eq!(bridge.get_legacy(&ecs_entity), Some(legacy_id));

    bridge.remove_by_ecs(&ecs_entity);
    assert_eq!(bridge.get(&legacy_id), None);
    assert_eq!(bridge.get_legacy(&ecs_entity), None);
}

#[test]
fn test_entity_bridge_ecs_entities_list() {
    let mut bridge = EntityBridge::default();
    let mut world = ecs::World::new();

    let ecs1 = world.spawn();
    let ecs2 = world.spawn();
    let ecs3 = world.spawn();

    bridge.insert(1, ecs1);
    bridge.insert(2, ecs2);
    bridge.insert(3, ecs3);

    let entities = bridge.ecs_entities();
    assert_eq!(entities.len(), 3);
    assert!(entities.contains(&ecs1));
    assert!(entities.contains(&ecs2));
    assert!(entities.contains(&ecs3));
}

#[test]
fn test_entity_bridge_multiple_mappings() {
    let mut bridge = EntityBridge::default();
    let mut world = ecs::World::new();

    let ecs1 = world.spawn();
    let ecs2 = world.spawn();
    let ecs3 = world.spawn();

    bridge.insert(10, ecs1);
    bridge.insert(20, ecs2);
    bridge.insert(30, ecs3);

    assert_eq!(bridge.get(&10), Some(ecs1));
    assert_eq!(bridge.get(&20), Some(ecs2));
    assert_eq!(bridge.get(&30), Some(ecs3));

    assert_eq!(bridge.get_legacy(&ecs1), Some(10));
    assert_eq!(bridge.get_legacy(&ecs2), Some(20));
    assert_eq!(bridge.get_legacy(&ecs3), Some(30));
}

// ============================================================================
// Component Tests (ecs_components.rs)
// ============================================================================

#[test]
fn test_cpos_component() {
    let pos = CPos {
        pos: IVec2 { x: 10, y: 20 },
    };

    assert_eq!(pos.pos.x, 10);
    assert_eq!(pos.pos.y, 20);
}

#[test]
fn test_chealth_component() {
    let health = CHealth { hp: 100 };
    assert_eq!(health.hp, 100);

    let dead = CHealth { hp: 0 };
    assert_eq!(dead.hp, 0);
}

#[test]
fn test_cteam_component() {
    let team1 = CTeam { id: 1 };
    let team2 = CTeam { id: 2 };

    assert_eq!(team1.id, 1);
    assert_eq!(team2.id, 2);
    assert_ne!(team1.id, team2.id);
}

#[test]
fn test_cammo_component() {
    let ammo = CAmmo { rounds: 30 };
    assert_eq!(ammo.rounds, 30);

    let empty = CAmmo { rounds: 0 };
    assert_eq!(empty.rounds, 0);
}

#[test]
fn test_cooldown_key_from_str() {
    let key: cooldowns::CooldownKey = "throw:smoke".into();
    assert_eq!(key, cooldowns::CooldownKey::ThrowSmoke);

    let custom_key: cooldowns::CooldownKey = "custom_ability".into();
    assert_eq!(
        custom_key,
        cooldowns::CooldownKey::Custom("custom_ability".to_string())
    );
}

#[test]
fn test_cooldown_key_display() {
    let key = cooldowns::CooldownKey::ThrowSmoke;
    assert_eq!(format!("{}", key), "throw:smoke");

    let custom = cooldowns::CooldownKey::Custom("test".to_string());
    assert_eq!(format!("{}", custom), "test");
}

#[test]
fn test_ccooldowns_component() {
    let mut cooldowns = CCooldowns::default();
    assert_eq!(cooldowns.map.len(), 0);

    cooldowns.map.insert("throw:smoke".into(), 2.5);
    cooldowns.map.insert("attack".into(), 1.0);

    assert_eq!(cooldowns.map.len(), 2);
    assert_eq!(cooldowns.map.get(&"throw:smoke".into()), Some(&2.5));
}

#[test]
fn test_cdesired_pos_component() {
    let desired = CDesiredPos {
        pos: IVec2 { x: 5, y: 10 },
    };

    assert_eq!(desired.pos.x, 5);
    assert_eq!(desired.pos.y, 10);
}

#[test]
fn test_clegacy_id_component() {
    let legacy = CLegacyId { id: 42 };
    assert_eq!(legacy.id, 42);
}

#[test]
fn test_component_defaults() {
    let pos = CPos::default();
    assert_eq!(pos.pos.x, 0);
    assert_eq!(pos.pos.y, 0);

    let health = CHealth::default();
    assert_eq!(health.hp, 0);

    let team = CTeam::default();
    assert_eq!(team.id, 0);

    let ammo = CAmmo::default();
    assert_eq!(ammo.rounds, 0);

    let cooldowns = CCooldowns::default();
    assert_eq!(cooldowns.map.len(), 0);
}

// ============================================================================
// Events Tests (ecs_events.rs)
// ============================================================================

#[test]
fn test_events_send_and_drain() {
    let mut events: Events<MovedEvent> = Events::default();

    let mut world = ecs::World::new();
    let entity = world.spawn();

    // Send event
    {
        let mut writer = events.writer();
        writer.send(MovedEvent {
            entity,
            from: IVec2 { x: 0, y: 0 },
            to: IVec2 { x: 5, y: 5 },
        });
    }

    assert_eq!(events.len(), 1);
    assert!(!events.is_empty());

    // Drain events
    {
        let mut reader = events.reader();
        let drained: Vec<_> = reader.drain().collect();

        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].from.x, 0);
        assert_eq!(drained[0].to.x, 5);
    }

    assert!(events.is_empty());
}

#[test]
fn test_events_multiple_send() {
    let mut events: Events<AiPlannedEvent> = Events::default();
    let mut world = ecs::World::new();

    let e1 = world.spawn();
    let e2 = world.spawn();
    let e3 = world.spawn();

    {
        let mut writer = events.writer();
        writer.send(AiPlannedEvent {
            entity: e1,
            target: IVec2 { x: 1, y: 1 },
        });
        writer.send(AiPlannedEvent {
            entity: e2,
            target: IVec2 { x: 2, y: 2 },
        });
        writer.send(AiPlannedEvent {
            entity: e3,
            target: IVec2 { x: 3, y: 3 },
        });
    }

    assert_eq!(events.len(), 3);

    {
        let mut reader = events.reader();
        let drained: Vec<_> = reader.drain().collect();
        assert_eq!(drained.len(), 3);
    }

    assert_eq!(events.len(), 0);
}

#[test]
fn test_events_clear() {
    let mut events: Events<HealthChangedEvent> = Events::default();
    let mut world = ecs::World::new();

    let entity = world.spawn();

    {
        let mut writer = events.writer();
        writer.send(HealthChangedEvent {
            entity,
            old_hp: 100,
            new_hp: 75,
        });
        writer.send(HealthChangedEvent {
            entity,
            old_hp: 75,
            new_hp: 50,
        });
    }

    assert_eq!(events.len(), 2);

    events.clear();
    assert_eq!(events.len(), 0);
    assert!(events.is_empty());
}

#[test]
fn test_moved_event() {
    let mut world = ecs::World::new();
    let entity = world.spawn();

    let event = MovedEvent {
        entity,
        from: IVec2 { x: 10, y: 10 },
        to: IVec2 { x: 15, y: 15 },
    };

    assert_eq!(event.entity, entity);
    assert_eq!(event.from.x, 10);
    assert_eq!(event.to.x, 15);
}

#[test]
fn test_ai_planned_event() {
    let mut world = ecs::World::new();
    let entity = world.spawn();

    let event = AiPlannedEvent {
        entity,
        target: IVec2 { x: 20, y: 20 },
    };

    assert_eq!(event.entity, entity);
    assert_eq!(event.target.x, 20);
}

#[test]
fn test_ai_planning_failed_event() {
    let mut world = ecs::World::new();
    let entity = world.spawn();

    let event = AiPlanningFailedEvent {
        entity,
        reason: "No path found".to_string(),
    };

    assert_eq!(event.entity, entity);
    assert_eq!(event.reason, "No path found");
}

#[test]
fn test_tool_validation_failed_event() {
    let mut world = ecs::World::new();
    let entity = world.spawn();

    let event = ToolValidationFailedEvent {
        entity,
        tool_verb: "MoveTo".to_string(),
        reason: "Path blocked".to_string(),
    };

    assert_eq!(event.entity, entity);
    assert_eq!(event.tool_verb, "MoveTo");
    assert_eq!(event.reason, "Path blocked");
}

#[test]
fn test_health_changed_event() {
    let mut world = ecs::World::new();
    let entity = world.spawn();

    let event = HealthChangedEvent {
        entity,
        old_hp: 100,
        new_hp: 85,
    };

    assert_eq!(event.entity, entity);
    assert_eq!(event.old_hp, 100);
    assert_eq!(event.new_hp, 85);
}
