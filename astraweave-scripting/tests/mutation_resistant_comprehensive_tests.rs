// =============================================================================
// AstraWeave Scripting — Mutation-Resistant Comprehensive Tests
// =============================================================================
// Targets: ScriptCommand (6 variants), ScriptCommands (6 methods + Default),
//          RaycastHit (fields, clone, debug), ScriptEvent (4 variants),
//          CScript (new, defaults), CPhysicsBody (body_id),
//          ScriptCache (default), PhysicsProxy (null raycast),
//          NavMeshProxy (null find_path), ScriptingPlugin (build),
//          Integration: script execution, set_position, spawn
// =============================================================================

#![allow(clippy::unnecessary_get_then_check)]

use astraweave_ecs::{App, Entity};
use astraweave_scripting::api::{
    NavMeshProxy, PhysicsProxy, RaycastHit, ScriptCommand, ScriptCommands,
};
use astraweave_scripting::events::ScriptEvent;
use astraweave_scripting::{
    CPhysicsBody, CScript, ScriptCache, ScriptEngineResource, ScriptingPlugin,
};
use glam::Vec3;
use rhai::Dynamic;
use std::collections::HashMap;
use std::sync::Arc;

// Helper to create test entities
fn entity(id: u64) -> Entity {
    unsafe { Entity::from_raw(id) }
}

// ===========================================================================
// 1. CScript — construction and defaults
// ===========================================================================

#[test]
fn cscript_new_sets_path() {
    let s = CScript::new("scripts/test.rhai", "let x = 1;");
    assert_eq!(s.script_path, "scripts/test.rhai");
}

#[test]
fn cscript_new_sets_source() {
    let s = CScript::new("test.rhai", "let x = 42;");
    assert_eq!(s.source, "let x = 42;");
}

#[test]
fn cscript_new_enabled_true() {
    let s = CScript::new("test.rhai", "");
    assert!(s.enabled, "CScript.enabled should default to true");
}

#[test]
fn cscript_new_cached_ast_none() {
    let s = CScript::new("test.rhai", "");
    assert!(
        s.cached_ast.is_none(),
        "cached_ast should be None on creation"
    );
}

#[test]
fn cscript_new_script_state_empty() {
    let s = CScript::new("test.rhai", "");
    assert!(s.script_state.is_empty());
}

#[test]
fn cscript_new_empty_path() {
    let s = CScript::new("", "let x = 1;");
    assert_eq!(s.script_path, "");
    assert_eq!(s.source, "let x = 1;");
}

#[test]
fn cscript_new_empty_source() {
    let s = CScript::new("test.rhai", "");
    assert_eq!(s.source, "");
}

#[test]
fn cscript_state_insert_and_read() {
    let mut s = CScript::new("test.rhai", "");
    s.script_state
        .insert("counter".to_string(), Dynamic::from(42_i64));
    assert_eq!(s.script_state.len(), 1);
    let val = s.script_state.get("counter").unwrap().as_int().unwrap();
    assert_eq!(val, 42);
}

#[test]
fn cscript_state_missing_key() {
    let s = CScript::new("test.rhai", "");
    assert!(s.script_state.get("missing").is_none());
}

#[test]
fn cscript_enabled_can_be_disabled() {
    let mut s = CScript::new("test.rhai", "");
    assert!(s.enabled);
    s.enabled = false;
    assert!(!s.enabled);
}

#[test]
fn cscript_clone_preserves_all_fields() {
    let mut s = CScript::new("clone_test.rhai", "let a = 1;");
    s.script_state
        .insert("key".to_string(), Dynamic::from(10_i64));
    s.enabled = false;
    let c = s.clone();
    assert_eq!(c.script_path, "clone_test.rhai");
    assert_eq!(c.source, "let a = 1;");
    assert!(!c.enabled);
    assert_eq!(c.script_state.len(), 1);
}

// ===========================================================================
// 2. CPhysicsBody
// ===========================================================================

#[test]
fn cphysics_body_stores_id() {
    let b = CPhysicsBody { body_id: 999 };
    assert_eq!(b.body_id, 999);
}

#[test]
fn cphysics_body_zero_id() {
    let b = CPhysicsBody { body_id: 0 };
    assert_eq!(b.body_id, 0);
}

#[test]
fn cphysics_body_clone() {
    let b = CPhysicsBody { body_id: 42 };
    let c = b;
    assert_eq!(c.body_id, 42);
}

#[test]
fn cphysics_body_debug() {
    let b = CPhysicsBody { body_id: 7 };
    let s = format!("{:?}", b);
    assert!(s.contains("CPhysicsBody"), "got: {}", s);
    assert!(s.contains("7"), "got: {}", s);
}

// ===========================================================================
// 3. ScriptCache — default
// ===========================================================================

#[test]
fn script_cache_default_empty() {
    let c = ScriptCache::default();
    assert!(c.scripts.is_empty());
}

// ===========================================================================
// 4. ScriptCommand — all 6 variants, exact fields
// ===========================================================================

#[test]
fn script_command_spawn_fields() {
    let cmd = ScriptCommand::Spawn {
        prefab: "goblin".to_string(),
        position: Vec3::new(1.0, 2.0, 3.0),
    };
    if let ScriptCommand::Spawn { prefab, position } = &cmd {
        assert_eq!(prefab, "goblin");
        assert_eq!(*position, Vec3::new(1.0, 2.0, 3.0));
    } else {
        panic!("expected Spawn");
    }
}

#[test]
fn script_command_despawn_field() {
    let cmd = ScriptCommand::Despawn { entity: 77 };
    if let ScriptCommand::Despawn { entity } = &cmd {
        assert_eq!(*entity, 77);
    } else {
        panic!("expected Despawn");
    }
}

#[test]
fn script_command_set_position_fields() {
    let cmd = ScriptCommand::SetPosition {
        entity: 5,
        position: Vec3::new(10.0, 20.0, 30.0),
    };
    if let ScriptCommand::SetPosition { entity, position } = &cmd {
        assert_eq!(*entity, 5);
        assert_eq!(*position, Vec3::new(10.0, 20.0, 30.0));
    } else {
        panic!("expected SetPosition");
    }
}

#[test]
fn script_command_apply_damage_fields() {
    let cmd = ScriptCommand::ApplyDamage {
        entity: 3,
        amount: 25.5,
    };
    if let ScriptCommand::ApplyDamage { entity, amount } = &cmd {
        assert_eq!(*entity, 3);
        assert!((*amount - 25.5).abs() < f32::EPSILON);
    } else {
        panic!("expected ApplyDamage");
    }
}

#[test]
fn script_command_play_sound_field() {
    let cmd = ScriptCommand::PlaySound {
        path: "sfx/boom.wav".to_string(),
    };
    if let ScriptCommand::PlaySound { path } = &cmd {
        assert_eq!(path, "sfx/boom.wav");
    } else {
        panic!("expected PlaySound");
    }
}

#[test]
fn script_command_spawn_particle_fields() {
    let cmd = ScriptCommand::SpawnParticle {
        effect: "fire_burst".to_string(),
        position: Vec3::new(0.0, 5.0, 0.0),
    };
    if let ScriptCommand::SpawnParticle { effect, position } = &cmd {
        assert_eq!(effect, "fire_burst");
        assert_eq!(*position, Vec3::new(0.0, 5.0, 0.0));
    } else {
        panic!("expected SpawnParticle");
    }
}

#[test]
fn script_command_clone_spawn() {
    let cmd = ScriptCommand::Spawn {
        prefab: "archer".to_string(),
        position: Vec3::ONE,
    };
    let c = cmd.clone();
    if let ScriptCommand::Spawn { prefab, position } = c {
        assert_eq!(prefab, "archer");
        assert_eq!(position, Vec3::ONE);
    } else {
        panic!("expected Spawn after clone");
    }
}

#[test]
fn script_command_debug_despawn() {
    let cmd = ScriptCommand::Despawn { entity: 42 };
    let s = format!("{:?}", cmd);
    assert!(s.contains("Despawn"), "got: {}", s);
    assert!(s.contains("42"), "got: {}", s);
}

#[test]
fn script_command_debug_play_sound() {
    let cmd = ScriptCommand::PlaySound {
        path: "music.mp3".to_string(),
    };
    let s = format!("{:?}", cmd);
    assert!(s.contains("PlaySound"), "got: {}", s);
    assert!(s.contains("music.mp3"), "got: {}", s);
}

// ===========================================================================
// 5. ScriptCommands — new/default, all 6 push methods, ordering
// ===========================================================================

#[test]
fn script_commands_new_empty() {
    let cmds = ScriptCommands::new();
    assert!(cmds.commands.is_empty());
    assert_eq!(cmds.commands.len(), 0);
}

#[test]
fn script_commands_default_empty() {
    let cmds = ScriptCommands::default();
    assert!(cmds.commands.is_empty());
}

#[test]
fn script_commands_spawn_adds_one() {
    let mut cmds = ScriptCommands::new();
    cmds.spawn("test", Vec3::ZERO);
    assert_eq!(cmds.commands.len(), 1);
}

#[test]
fn script_commands_spawn_correct_variant() {
    let mut cmds = ScriptCommands::new();
    cmds.spawn("enemy_grunt", Vec3::new(1.0, 2.0, 3.0));
    match &cmds.commands[0] {
        ScriptCommand::Spawn { prefab, position } => {
            assert_eq!(prefab, "enemy_grunt");
            assert_eq!(*position, Vec3::new(1.0, 2.0, 3.0));
        }
        _ => panic!("expected Spawn"),
    }
}

#[test]
fn script_commands_despawn_correct_variant() {
    let mut cmds = ScriptCommands::new();
    cmds.despawn(99);
    assert_eq!(cmds.commands.len(), 1);
    match &cmds.commands[0] {
        ScriptCommand::Despawn { entity } => assert_eq!(*entity, 99),
        _ => panic!("expected Despawn"),
    }
}

#[test]
fn script_commands_set_position_correct_variant() {
    let mut cmds = ScriptCommands::new();
    cmds.set_position(7, Vec3::new(4.0, 5.0, 6.0));
    assert_eq!(cmds.commands.len(), 1);
    match &cmds.commands[0] {
        ScriptCommand::SetPosition { entity, position } => {
            assert_eq!(*entity, 7);
            assert_eq!(*position, Vec3::new(4.0, 5.0, 6.0));
        }
        _ => panic!("expected SetPosition"),
    }
}

#[test]
fn script_commands_apply_damage_correct_variant() {
    let mut cmds = ScriptCommands::new();
    cmds.apply_damage(10, 50.0);
    assert_eq!(cmds.commands.len(), 1);
    match &cmds.commands[0] {
        ScriptCommand::ApplyDamage { entity, amount } => {
            assert_eq!(*entity, 10);
            assert!((*amount - 50.0).abs() < f32::EPSILON);
        }
        _ => panic!("expected ApplyDamage"),
    }
}

#[test]
fn script_commands_play_sound_correct_variant() {
    let mut cmds = ScriptCommands::new();
    cmds.play_sound("explosion.wav");
    assert_eq!(cmds.commands.len(), 1);
    match &cmds.commands[0] {
        ScriptCommand::PlaySound { path } => assert_eq!(path, "explosion.wav"),
        _ => panic!("expected PlaySound"),
    }
}

#[test]
fn script_commands_spawn_particle_correct_variant() {
    let mut cmds = ScriptCommands::new();
    cmds.spawn_particle("smoke", Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(cmds.commands.len(), 1);
    match &cmds.commands[0] {
        ScriptCommand::SpawnParticle { effect, position } => {
            assert_eq!(effect, "smoke");
            assert_eq!(*position, Vec3::new(1.0, 2.0, 3.0));
        }
        _ => panic!("expected SpawnParticle"),
    }
}

#[test]
fn script_commands_accumulate_count() {
    let mut cmds = ScriptCommands::new();
    cmds.spawn("a", Vec3::ZERO);
    assert_eq!(cmds.commands.len(), 1);
    cmds.despawn(1);
    assert_eq!(cmds.commands.len(), 2);
    cmds.apply_damage(1, 10.0);
    assert_eq!(cmds.commands.len(), 3);
    cmds.play_sound("beep");
    assert_eq!(cmds.commands.len(), 4);
    cmds.set_position(1, Vec3::ONE);
    assert_eq!(cmds.commands.len(), 5);
    cmds.spawn_particle("fx", Vec3::ZERO);
    assert_eq!(cmds.commands.len(), 6);
}

#[test]
fn script_commands_order_preserved() {
    let mut cmds = ScriptCommands::new();
    cmds.spawn("first", Vec3::ZERO);
    cmds.despawn(2);
    cmds.play_sound("third");

    assert!(matches!(&cmds.commands[0], ScriptCommand::Spawn { prefab, .. } if prefab == "first"));
    assert!(matches!(
        &cmds.commands[1],
        ScriptCommand::Despawn { entity: 2 }
    ));
    assert!(matches!(&cmds.commands[2], ScriptCommand::PlaySound { path } if path == "third"));
}

#[test]
fn script_commands_clone_preserves_commands() {
    let mut cmds = ScriptCommands::new();
    cmds.spawn("test", Vec3::ZERO);
    cmds.despawn(1);
    let c = cmds.clone();
    assert_eq!(c.commands.len(), 2);
}

// ===========================================================================
// 6. RaycastHit — fields, clone, debug
// ===========================================================================

#[test]
fn raycast_hit_entity_id() {
    let hit = RaycastHit {
        entity_id: 42,
        position: Vec3::ZERO,
        normal: Vec3::Y,
        distance: 1.0,
    };
    assert_eq!(hit.entity_id, 42);
}

#[test]
fn raycast_hit_position() {
    let hit = RaycastHit {
        entity_id: 0,
        position: Vec3::new(3.0, 4.0, 5.0),
        normal: Vec3::Y,
        distance: 1.0,
    };
    assert_eq!(hit.position, Vec3::new(3.0, 4.0, 5.0));
}

#[test]
fn raycast_hit_normal() {
    let hit = RaycastHit {
        entity_id: 0,
        position: Vec3::ZERO,
        normal: Vec3::new(0.0, 0.0, 1.0),
        distance: 1.0,
    };
    assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
}

#[test]
fn raycast_hit_distance() {
    let hit = RaycastHit {
        entity_id: 0,
        position: Vec3::ZERO,
        normal: Vec3::Y,
        distance: 12.75,
    };
    assert!((hit.distance - 12.75).abs() < f32::EPSILON);
}

#[test]
fn raycast_hit_zero_distance() {
    let hit = RaycastHit {
        entity_id: 1,
        position: Vec3::ZERO,
        normal: Vec3::Y,
        distance: 0.0,
    };
    assert_eq!(hit.distance, 0.0);
}

#[test]
fn raycast_hit_clone_independent() {
    let hit = RaycastHit {
        entity_id: 5,
        position: Vec3::new(1.0, 2.0, 3.0),
        normal: Vec3::Y,
        distance: 7.5,
    };
    let c = hit.clone();
    assert_eq!(c.entity_id, 5);
    assert_eq!(c.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(c.normal, Vec3::Y);
    assert!((c.distance - 7.5).abs() < f32::EPSILON);
}

#[test]
fn raycast_hit_debug() {
    let hit = RaycastHit {
        entity_id: 10,
        position: Vec3::ZERO,
        normal: Vec3::Z,
        distance: 2.5,
    };
    let s = format!("{:?}", hit);
    assert!(s.contains("RaycastHit"), "got: {}", s);
    assert!(s.contains("10"), "got: {}", s);
}

// ===========================================================================
// 7. ScriptEvent — all 4 variants, field preservation
// ===========================================================================

#[test]
fn script_event_on_spawn_entity() {
    let e = entity(1);
    let event = ScriptEvent::OnSpawn { entity: e };
    if let ScriptEvent::OnSpawn { entity: got } = event {
        assert_eq!(got, entity(1));
    } else {
        panic!("expected OnSpawn");
    }
}

#[test]
fn script_event_on_collision_both_entities() {
    let event = ScriptEvent::OnCollision {
        entity: entity(10),
        other: entity(20),
    };
    if let ScriptEvent::OnCollision {
        entity: e,
        other: o,
    } = event
    {
        assert_eq!(e, entity(10));
        assert_eq!(o, entity(20));
        assert_ne!(e, o);
    } else {
        panic!("expected OnCollision");
    }
}

#[test]
fn script_event_on_collision_self() {
    let event = ScriptEvent::OnCollision {
        entity: entity(5),
        other: entity(5),
    };
    if let ScriptEvent::OnCollision { entity, other } = event {
        assert_eq!(entity, other);
    } else {
        panic!("expected OnCollision");
    }
}

#[test]
fn script_event_on_trigger_name() {
    let event = ScriptEvent::OnTrigger {
        entity: entity(1),
        trigger_name: "door_open".to_string(),
    };
    if let ScriptEvent::OnTrigger { trigger_name, .. } = &event {
        assert_eq!(trigger_name, "door_open");
    } else {
        panic!("expected OnTrigger");
    }
}

#[test]
fn script_event_on_trigger_empty_name() {
    let event = ScriptEvent::OnTrigger {
        entity: entity(1),
        trigger_name: String::new(),
    };
    if let ScriptEvent::OnTrigger { trigger_name, .. } = &event {
        assert!(trigger_name.is_empty());
    } else {
        panic!("expected OnTrigger");
    }
}

#[test]
fn script_event_on_damage_exact_values() {
    let event = ScriptEvent::OnDamage {
        entity: entity(1),
        damage: 25.5,
        source: entity(2),
    };
    if let ScriptEvent::OnDamage {
        entity: e,
        damage,
        source,
    } = event
    {
        assert_eq!(e, entity(1));
        assert!((damage - 25.5).abs() < f32::EPSILON);
        assert_eq!(source, entity(2));
    } else {
        panic!("expected OnDamage");
    }
}

#[test]
fn script_event_on_damage_zero() {
    let event = ScriptEvent::OnDamage {
        entity: entity(1),
        damage: 0.0,
        source: entity(2),
    };
    if let ScriptEvent::OnDamage { damage, .. } = event {
        assert_eq!(damage, 0.0);
    } else {
        panic!("expected OnDamage");
    }
}

#[test]
fn script_event_on_damage_negative_healing() {
    let event = ScriptEvent::OnDamage {
        entity: entity(1),
        damage: -10.0,
        source: entity(2),
    };
    if let ScriptEvent::OnDamage { damage, .. } = event {
        assert!(damage < 0.0);
        assert!((damage - (-10.0)).abs() < f32::EPSILON);
    } else {
        panic!("expected OnDamage");
    }
}

#[test]
fn script_event_on_damage_self_damage() {
    let event = ScriptEvent::OnDamage {
        entity: entity(3),
        damage: 5.0,
        source: entity(3),
    };
    if let ScriptEvent::OnDamage {
        entity: e, source, ..
    } = event
    {
        assert_eq!(e, source);
    } else {
        panic!("expected OnDamage");
    }
}

#[test]
fn script_event_clone_on_spawn() {
    let event = ScriptEvent::OnSpawn { entity: entity(42) };
    let c = event.clone();
    if let ScriptEvent::OnSpawn { entity: e } = c {
        assert_eq!(e, entity(42));
    } else {
        panic!("expected OnSpawn after clone");
    }
}

#[test]
fn script_event_clone_on_trigger() {
    let event = ScriptEvent::OnTrigger {
        entity: entity(1),
        trigger_name: "zone_A".to_string(),
    };
    let c = event.clone();
    if let ScriptEvent::OnTrigger { trigger_name, .. } = c {
        assert_eq!(trigger_name, "zone_A");
    } else {
        panic!("expected OnTrigger after clone");
    }
}

#[test]
fn script_event_debug_on_spawn() {
    let event = ScriptEvent::OnSpawn {
        entity: entity(100),
    };
    let s = format!("{:?}", event);
    assert!(s.contains("OnSpawn"), "got: {}", s);
}

#[test]
fn script_event_debug_on_collision() {
    let event = ScriptEvent::OnCollision {
        entity: entity(1),
        other: entity(2),
    };
    let s = format!("{:?}", event);
    assert!(s.contains("OnCollision"), "got: {}", s);
}

#[test]
fn script_event_debug_on_trigger() {
    let event = ScriptEvent::OnTrigger {
        entity: entity(1),
        trigger_name: "myzone".to_string(),
    };
    let s = format!("{:?}", event);
    assert!(s.contains("OnTrigger"), "got: {}", s);
    assert!(s.contains("myzone"), "got: {}", s);
}

#[test]
fn script_event_debug_on_damage() {
    let event = ScriptEvent::OnDamage {
        entity: entity(1),
        damage: 15.0,
        source: entity(2),
    };
    let s = format!("{:?}", event);
    assert!(s.contains("OnDamage"), "got: {}", s);
}

// ===========================================================================
// 8. ScriptEvent — variant discrimination (pattern exhaustiveness)
// ===========================================================================

#[test]
fn script_event_variant_counting() {
    let events = vec![
        ScriptEvent::OnSpawn { entity: entity(1) },
        ScriptEvent::OnCollision {
            entity: entity(2),
            other: entity(3),
        },
        ScriptEvent::OnTrigger {
            entity: entity(4),
            trigger_name: "t".into(),
        },
        ScriptEvent::OnDamage {
            entity: entity(5),
            damage: 1.0,
            source: entity(6),
        },
    ];

    let mut spawn = 0;
    let mut collision = 0;
    let mut trigger = 0;
    let mut damage = 0;

    for e in &events {
        match e {
            ScriptEvent::OnSpawn { .. } => spawn += 1,
            ScriptEvent::OnCollision { .. } => collision += 1,
            ScriptEvent::OnTrigger { .. } => trigger += 1,
            ScriptEvent::OnDamage { .. } => damage += 1,
            _ => {}
        }
    }

    assert_eq!(spawn, 1);
    assert_eq!(collision, 1);
    assert_eq!(trigger, 1);
    assert_eq!(damage, 1);
    assert_eq!(events.len(), 4);
}

// ===========================================================================
// 9. PhysicsProxy — null pointer raycast returns Dynamic::UNIT
// ===========================================================================

#[test]
fn physics_proxy_null_raycast_returns_unit() {
    let mut proxy = PhysicsProxy {
        ptr: std::ptr::null(),
        body_map: Arc::new(HashMap::new()),
    };
    let result = proxy.raycast(Vec3::ZERO, Vec3::X, 100.0);
    // Dynamic::UNIT is the () unit type in Rhai
    assert!(
        result.is_unit(),
        "null physics proxy should return unit, got: {:?}",
        result
    );
}

// ===========================================================================
// 10. NavMeshProxy — null pointer find_path returns empty vec
// ===========================================================================

#[test]
fn nav_mesh_proxy_null_returns_empty() {
    let mut proxy = NavMeshProxy {
        ptr: std::ptr::null(),
    };
    let result = proxy.find_path(Vec3::ZERO, Vec3::ONE);
    assert!(
        result.is_empty(),
        "null navmesh proxy should return empty vec"
    );
}

// ===========================================================================
// 11. ScriptCommands — zero and edge cases
// ===========================================================================

#[test]
fn script_commands_despawn_zero_entity() {
    let mut cmds = ScriptCommands::new();
    cmds.despawn(0);
    match &cmds.commands[0] {
        ScriptCommand::Despawn { entity } => assert_eq!(*entity, 0),
        _ => panic!("expected Despawn"),
    }
}

#[test]
fn script_commands_apply_damage_zero() {
    let mut cmds = ScriptCommands::new();
    cmds.apply_damage(1, 0.0);
    match &cmds.commands[0] {
        ScriptCommand::ApplyDamage { entity, amount } => {
            assert_eq!(*entity, 1);
            assert_eq!(*amount, 0.0);
        }
        _ => panic!("expected ApplyDamage"),
    }
}

#[test]
fn script_commands_apply_damage_negative() {
    let mut cmds = ScriptCommands::new();
    cmds.apply_damage(1, -5.0);
    match &cmds.commands[0] {
        ScriptCommand::ApplyDamage { amount, .. } => {
            assert!((*amount - (-5.0)).abs() < f32::EPSILON);
        }
        _ => panic!("expected ApplyDamage"),
    }
}

#[test]
fn script_commands_spawn_empty_prefab() {
    let mut cmds = ScriptCommands::new();
    cmds.spawn("", Vec3::ZERO);
    match &cmds.commands[0] {
        ScriptCommand::Spawn { prefab, .. } => assert_eq!(prefab, ""),
        _ => panic!("expected Spawn"),
    }
}

#[test]
fn script_commands_play_sound_empty_path() {
    let mut cmds = ScriptCommands::new();
    cmds.play_sound("");
    match &cmds.commands[0] {
        ScriptCommand::PlaySound { path } => assert_eq!(path, ""),
        _ => panic!("expected PlaySound"),
    }
}

// ===========================================================================
// 12. ScriptCommand — ApplyDamage amount exact float
// ===========================================================================

#[test]
fn script_command_damage_large_value() {
    let cmd = ScriptCommand::ApplyDamage {
        entity: 1,
        amount: 99999.99,
    };
    if let ScriptCommand::ApplyDamage { amount, .. } = cmd {
        assert!((amount - 99999.99).abs() < 0.01);
    } else {
        panic!("expected ApplyDamage");
    }
}

// ===========================================================================
// 13. ScriptCommand — Spawn position components
// ===========================================================================

#[test]
fn script_command_spawn_position_components() {
    let cmd = ScriptCommand::Spawn {
        prefab: "test".to_string(),
        position: Vec3::new(1.5, 2.5, 3.5),
    };
    if let ScriptCommand::Spawn { position, .. } = cmd {
        assert_eq!(position.x, 1.5);
        assert_eq!(position.y, 2.5);
        assert_eq!(position.z, 3.5);
    } else {
        panic!("expected Spawn");
    }
}

// ===========================================================================
// 14. Integration — ScriptingPlugin builds App successfully
// ===========================================================================

#[test]
fn scripting_plugin_builds() {
    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);
    // If we get here, the plugin built successfully
    // Check that ScriptEngineResource was inserted
    assert!(
        app.world.get_resource::<ScriptEngineResource>().is_some(),
        "ScriptEngineResource should be inserted by plugin"
    );
}

#[test]
fn scripting_plugin_inserts_cache() {
    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);
    assert!(
        app.world.get_resource::<ScriptCache>().is_some(),
        "ScriptCache should be inserted by plugin"
    );
}

// ===========================================================================
// 15. Integration — simple script execution
// ===========================================================================

#[test]
fn integration_script_addition() {
    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let e = app.world.spawn();
    let source = r#"
        let a = 10;
        let b = 20;
        result = a + b;
    "#;
    let mut script = CScript::new("test_add.rhai", source);
    script
        .script_state
        .insert("result".to_string(), Dynamic::from(0_i64));
    app.world.insert(e, script);

    app.schedule.run(&mut app.world);

    let s = app.world.get::<CScript>(e).unwrap();
    let result = s.script_state.get("result").unwrap().as_int().unwrap();
    assert_eq!(result, 30);
}

#[test]
fn integration_script_multiplication() {
    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let e = app.world.spawn();
    let source = r#"
        result = 7 * 6;
    "#;
    let mut script = CScript::new("test_mul.rhai", source);
    script
        .script_state
        .insert("result".to_string(), Dynamic::from(0_i64));
    app.world.insert(e, script);

    app.schedule.run(&mut app.world);

    let s = app.world.get::<CScript>(e).unwrap();
    let result = s.script_state.get("result").unwrap().as_int().unwrap();
    assert_eq!(result, 42);
}

#[test]
fn integration_script_state_persistent() {
    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let e = app.world.spawn();
    let source = r#"
        counter += 1;
    "#;
    let mut script = CScript::new("counter.rhai", source);
    script
        .script_state
        .insert("counter".to_string(), Dynamic::from(0_i64));
    app.world.insert(e, script);

    // Run 3 ticks
    app.schedule.run(&mut app.world);
    app.schedule.run(&mut app.world);
    app.schedule.run(&mut app.world);

    let s = app.world.get::<CScript>(e).unwrap();
    let counter = s.script_state.get("counter").unwrap().as_int().unwrap();
    assert_eq!(counter, 3);
}

#[test]
fn integration_set_position_command() {
    use astraweave_core::{CPos, IVec2};

    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let e = app.world.spawn();
    app.world.insert(
        e,
        CPos {
            pos: IVec2::new(0, 0),
        },
    );

    let source = r#"
        let target = vec3(10.0, 0.0, 20.0);
        commands.set_position(entity_id, target);
    "#;
    let script = CScript::new("move.rhai", source);
    app.world.insert(e, script);

    app.schedule.run(&mut app.world);

    let pos = app.world.get::<CPos>(e).unwrap().pos;
    assert_eq!(pos.x, 10);
    assert_eq!(pos.y, 20);
}

#[test]
fn integration_spawn_prefab() {
    use astraweave_core::{CHealth, CPos};

    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let e = app.world.spawn();
    let source = r#"
        let pos = vec3(5.0, 0.0, 5.0);
        commands.spawn_prefab("enemy_grunt", pos);
    "#;
    let script = CScript::new("spawn.rhai", source);
    app.world.insert(e, script);

    app.schedule.run(&mut app.world);

    // Check spawned entity exists with CHealth + CPos at (5,5)
    let mut found = false;
    for entity in app.world.entities_with::<CHealth>() {
        if let Some(pos) = app.world.get::<CPos>(entity) {
            if pos.pos.x == 5 && pos.pos.y == 5 {
                found = true;
                break;
            }
        }
    }
    assert!(found, "spawned enemy_grunt not found at (5,5)");
}

#[test]
fn integration_disabled_script_not_executed() {
    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let e = app.world.spawn();
    let source = r#"
        result = 999;
    "#;
    let mut script = CScript::new("disabled.rhai", source);
    script
        .script_state
        .insert("result".to_string(), Dynamic::from(0_i64));
    script.enabled = false;
    app.world.insert(e, script);

    app.schedule.run(&mut app.world);

    let s = app.world.get::<CScript>(e).unwrap();
    let result = s.script_state.get("result").unwrap().as_int().unwrap();
    assert_eq!(result, 0, "disabled script should NOT execute");
}

#[test]
fn integration_empty_source_not_executed() {
    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let e = app.world.spawn();
    let mut script = CScript::new("", "");
    script
        .script_state
        .insert("x".to_string(), Dynamic::from(0_i64));
    app.world.insert(e, script);

    // Should not panic
    app.schedule.run(&mut app.world);

    let s = app.world.get::<CScript>(e).unwrap();
    let x = s.script_state.get("x").unwrap().as_int().unwrap();
    assert_eq!(x, 0);
}

// ===========================================================================
// 16. RaycastHit — negative entity_id
// ===========================================================================

#[test]
fn raycast_hit_negative_entity_id() {
    let hit = RaycastHit {
        entity_id: -1,
        position: Vec3::ZERO,
        normal: Vec3::Y,
        distance: 0.0,
    };
    assert_eq!(hit.entity_id, -1);
}

// ===========================================================================
// 17. ScriptCommand — SetPosition with negative coords
// ===========================================================================

#[test]
fn script_command_set_position_negative_coords() {
    let cmd = ScriptCommand::SetPosition {
        entity: 1,
        position: Vec3::new(-5.0, -10.0, -15.0),
    };
    if let ScriptCommand::SetPosition { position, .. } = cmd {
        assert_eq!(position.x, -5.0);
        assert_eq!(position.y, -10.0);
        assert_eq!(position.z, -15.0);
    } else {
        panic!("expected SetPosition");
    }
}

// ===========================================================================
// 18. ScriptEvent — OnTrigger with special characters
// ===========================================================================

#[test]
fn script_event_on_trigger_special_chars() {
    let event = ScriptEvent::OnTrigger {
        entity: entity(1),
        trigger_name: "zone_A2:exit!".to_string(),
    };
    if let ScriptEvent::OnTrigger { trigger_name, .. } = event {
        assert_eq!(trigger_name, "zone_A2:exit!");
    } else {
        panic!("expected OnTrigger");
    }
}

// ===========================================================================
// 19. Integration — sandboxing (infinite loop terminated)
// ===========================================================================

#[test]
fn integration_sandboxing_infinite_loop() {
    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let e = app.world.spawn();
    let source = r#"
        let x = 0;
        loop { x = x + 1; }
    "#;
    let script = CScript::new("loop.rhai", source);
    app.world.insert(e, script);

    // Should not hang — max_operations limit kills infinite loop
    app.schedule.run(&mut app.world);
    // Test passes if we reach here
}

// ===========================================================================
// 20. Integration — stale entity validation
// ===========================================================================

#[test]
fn integration_stale_entity_no_panic() {
    use astraweave_core::CHealth;

    let mut app = App::new();
    app = app.add_plugin(ScriptingPlugin);

    let victim = app.world.spawn();
    app.world.insert(victim, CHealth { hp: 100 });
    let victim_id = victim.to_raw() as i64;
    app.world.despawn(victim);

    let e = app.world.spawn();
    let source = format!(r#"commands.apply_damage({}, 50.0);"#, victim_id);
    let script = CScript::new("stale.rhai", &source);
    app.world.insert(e, script);

    // Should not panic — system validates entity liveness
    app.schedule.run(&mut app.world);
}
