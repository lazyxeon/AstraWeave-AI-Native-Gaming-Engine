use astraweave_ui::hud::*;

/// Test quest with 2 objectives
pub fn test_quest() -> Quest {
    Quest {
        id: 1,
        title: "Test Quest".into(),
        description: "Test description".into(),
        objectives: vec![
            Objective { 
                id: 1, 
                description: "Kill 5 enemies".into(), 
                completed: false, 
                progress: Some((0, 5)) 
            },
            Objective { 
                id: 2, 
                description: "Talk to NPC".into(), 
                completed: false, 
                progress: None 
            },
        ],
    }
}

/// Test enemy at origin
pub fn test_enemy(id: u32) -> EnemyData {
    EnemyData::new(id, (0.0, 1.0, 0.0), 100.0, EnemyFaction::Hostile)
}

/// Float comparison with epsilon
pub fn assert_float_eq(a: f32, b: f32, epsilon: f32) {
    assert!((a - b).abs() < epsilon, "Expected {}, got {} (diff > {})", a, b, epsilon);
}

/// Time-stepped update helper
pub fn advance_time(hud: &mut HudManager, dt: f32) {
    hud.update(dt);
}
