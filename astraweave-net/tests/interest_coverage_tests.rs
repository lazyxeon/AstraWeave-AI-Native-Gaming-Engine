use astraweave_core::*;
use astraweave_net::*;

/// Tests for FovLosInterest (Field-of-View with Line-of-Sight)
/// This covers complex branching logic in the Interest trait implementation
#[test]
fn test_fov_los_same_team() {
    let obstacles = std::collections::BTreeSet::new();
    let interest = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 }, // Facing right
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let teammate = EntityState {
        id: 2,
        pos: IVec2 { x: 50, y: 50 }, // Far away, same team
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    // Same team entities are ALWAYS visible (LOS doesn't matter)
    assert!(interest.include(&viewer, &teammate));
}

#[test]
fn test_fov_los_outside_radius() {
    let obstacles = std::collections::BTreeSet::new();
    let interest = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_far = EntityState {
        id: 3,
        pos: IVec2 { x: 50, y: 0 }, // Beyond radius=10
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // Outside radius, enemy team
    assert!(!interest.include(&viewer, &enemy_far));
}

#[test]
fn test_fov_los_blocked_by_obstacle() {
    let mut obstacles = std::collections::BTreeSet::new();
    obstacles.insert((5, 0)); // Obstacle directly between viewer and enemy
    
    let interest = FovLosInterest {
        radius: 20,
        half_angle_deg: 90.0, // Wide FOV
        facing: IVec2 { x: 1, y: 0 }, // Facing right
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_behind_wall = EntityState {
        id: 3,
        pos: IVec2 { x: 10, y: 0 }, // Directly behind obstacle
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // Enemy is within radius and FOV, but blocked by obstacle
    assert!(!interest.include(&viewer, &enemy_behind_wall));
}

#[test]
fn test_fov_los_clear_los() {
    let obstacles = std::collections::BTreeSet::new();
    
    let interest = FovLosInterest {
        radius: 15,
        half_angle_deg: 60.0,
        facing: IVec2 { x: 1, y: 0 }, // Facing right
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_visible = EntityState {
        id: 3,
        pos: IVec2 { x: 8, y: 2 }, // Within radius and FOV, no obstacles
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // Enemy is visible (in radius, in FOV, clear LOS)
    assert!(interest.include(&viewer, &enemy_visible));
}

#[test]
fn test_fov_los_outside_fov_angle() {
    let obstacles = std::collections::BTreeSet::new();
    
    let interest = FovLosInterest {
        radius: 15,
        half_angle_deg: 30.0, // Narrow FOV
        facing: IVec2 { x: 1, y: 0 }, // Facing right (+X direction)
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_behind = EntityState {
        id: 3,
        pos: IVec2 { x: -8, y: 0 }, // Directly behind viewer (opposite of facing)
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // Enemy is outside FOV angle (behind viewer)
    assert!(!interest.include(&viewer, &enemy_behind));
}

#[test]
fn test_fov_los_zero_facing_vector() {
    let obstacles = std::collections::BTreeSet::new();
    
    let interest = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 0, y: 0 }, // Zero facing vector (special case)
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_nearby = EntityState {
        id: 3,
        pos: IVec2 { x: 5, y: 5 }, // Within radius
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // With zero facing vector, should fall back to LOS check only
    assert!(interest.include(&viewer, &enemy_nearby));
}

#[test]
fn test_fov_los_zero_distance() {
    let obstacles = std::collections::BTreeSet::new();
    
    let interest = FovLosInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_same_pos = EntityState {
        id: 3,
        pos: IVec2 { x: 5, y: 5 }, // Exactly same position (vmag == 0)
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // Same position (zero distance) is always visible
    assert!(interest.include(&viewer, &enemy_same_pos));
}

#[test]
fn test_fov_los_diagonal_los_no_obstacle() {
    let mut obstacles = std::collections::BTreeSet::new();
    obstacles.insert((5, 5)); // Obstacle not on line
    
    let interest = FovLosInterest {
        radius: 20,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 1, y: 1 }, // Diagonal facing
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_diagonal = EntityState {
        id: 3,
        pos: IVec2 { x: 10, y: 2 }, // Diagonal but not through obstacle
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // Diagonal LOS that doesn't hit obstacle
    assert!(interest.include(&viewer, &enemy_diagonal));
}

#[test]
fn test_fov_los_multiple_obstacles_path() {
    let mut obstacles = std::collections::BTreeSet::new();
    obstacles.insert((2, 0));
    obstacles.insert((4, 0));
    obstacles.insert((6, 0)); // Multiple obstacles in line
    
    let interest = FovLosInterest {
        radius: 20,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_far = EntityState {
        id: 3,
        pos: IVec2 { x: 10, y: 0 },
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // LOS blocked by first obstacle
    assert!(!interest.include(&viewer, &enemy_far));
}

// EntityDeltaMask is private, cannot test directly

/// Test Delta operations with edge cases
#[test]
fn test_diff_snapshots_no_changes() {
    use astraweave_net::{diff_snapshots, Snapshot};
    
    let entities = vec![
        EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 1,
            ammo: 10,
        },
    ];
    
    let snapshot1 = Snapshot {
        version: 1,
        tick: 100,
        t: 10.0,
        seq: 1,
        world_hash: 12345,
        entities: entities.clone(),
    };
    
    let snapshot2 = Snapshot {
        version: 1,
        tick: 101,
        t: 10.1,
        seq: 2,
        world_hash: 12345,
        entities: entities.clone(), // Exact same entities
    };
    
    // Need viewer entity for interest filtering (diff_snapshots takes 4 args)
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    let delta = diff_snapshots(&snapshot1, &snapshot2, &FullInterest, &viewer);
    
    // No changes, no removals
    assert_eq!(delta.changed.len(), 0);
    assert_eq!(delta.removed.len(), 0);
    assert_eq!(delta.base_tick, 100);
    assert_eq!(delta.tick, 101);
}

#[test]
fn test_apply_delta_removes_entities() {
    use astraweave_net::{apply_delta, Delta, Snapshot};
    
    let mut snapshot = Snapshot {
        version: 1,
        tick: 100,
        t: 10.0,
        seq: 1,
        world_hash: 12345,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 0, y: 0 },
                hp: 100,
                team: 1,
                ammo: 10,
            },
            EntityState {
                id: 2,
                pos: IVec2 { x: 5, y: 5 },
                hp: 50,
                team: 2,
                ammo: 5,
            },
        ],
    };
    
    let delta = Delta {
        base_tick: 100,
        tick: 101,
        changed: vec![],
        removed: vec![2], // Remove entity 2
        head_hash: 54321,
    };
    
    apply_delta(&mut snapshot, &delta);
    
    assert_eq!(snapshot.entities.len(), 1);
    assert_eq!(snapshot.entities[0].id, 1);
    assert_eq!(snapshot.tick, 101);
    assert_eq!(snapshot.world_hash, 54321);
}

#[test]
fn test_apply_delta_partial_updates() {
    use astraweave_net::{apply_delta, Delta, EntityDelta, Snapshot};
    
    let mut snapshot = Snapshot {
        version: 1,
        tick: 100,
        t: 10.0,
        seq: 1,
        world_hash: 12345,
        entities: vec![EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 1,
            ammo: 10,
        }],
    };
    
    let delta = Delta {
        base_tick: 100,
        tick: 101,
        changed: vec![EntityDelta {
            id: 1,
            mask: 0b0011, // POS | HP
            pos: Some(IVec2 { x: 5, y: 5 }),
            hp: Some(80),
            team: None,
            ammo: None,
        }],
        removed: vec![],
        head_hash: 54321,
    };
    
    apply_delta(&mut snapshot, &delta);
    
    assert_eq!(snapshot.entities[0].pos.x, 5);
    assert_eq!(snapshot.entities[0].pos.y, 5);
    assert_eq!(snapshot.entities[0].hp, 80);
    assert_eq!(snapshot.entities[0].team, 1); // Unchanged
    assert_eq!(snapshot.entities[0].ammo, 10); // Unchanged
}

/// Test complex LOS scenarios with Bresenham line algorithm
#[test]
fn test_bresenham_steep_line() {
    let mut obstacles = std::collections::BTreeSet::new();
    obstacles.insert((1, 5)); // Obstacle on steep line
    
    let interest = FovLosInterest {
        radius: 20,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 0, y: 1 }, // Facing up
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_steep = EntityState {
        id: 3,
        pos: IVec2 { x: 2, y: 10 }, // Steep vertical line
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // LOS should be blocked by obstacle at (1, 5)
    assert!(!interest.include(&viewer, &enemy_steep));
}

#[test]
fn test_bresenham_horizontal_line() {
    let mut obstacles = std::collections::BTreeSet::new();
    obstacles.insert((5, 0)); // Obstacle on horizontal line
    
    let interest = FovLosInterest {
        radius: 20,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 1, y: 0 }, // Facing right
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_horizontal = EntityState {
        id: 3,
        pos: IVec2 { x: 10, y: 0 }, // Perfectly horizontal
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // LOS blocked
    assert!(!interest.include(&viewer, &enemy_horizontal));
}

#[test]
fn test_bresenham_vertical_line() {
    let mut obstacles = std::collections::BTreeSet::new();
    obstacles.insert((0, 5)); // Obstacle on vertical line
    
    let interest = FovLosInterest {
        radius: 20,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 0, y: 1 }, // Facing up
        obstacles,
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_vertical = EntityState {
        id: 3,
        pos: IVec2 { x: 0, y: 10 }, // Perfectly vertical
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // LOS blocked
    assert!(!interest.include(&viewer, &enemy_vertical));
}

/// Test FovInterest edge cases for coverage
#[test]
fn test_fov_interest_zero_facing() {
    use astraweave_net::FovInterest;
    
    let interest = FovInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 0, y: 0 }, // Zero facing
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy = EntityState {
        id: 2,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // With zero facing, should return true (all directions visible)
    assert!(interest.include(&viewer, &enemy));
}

#[test]
fn test_fov_interest_zero_distance() {
    use astraweave_net::FovInterest;
    
    let interest = FovInterest {
        radius: 10,
        half_angle_deg: 45.0,
        facing: IVec2 { x: 1, y: 0 },
    };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 5, y: 5 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_same_pos = EntityState {
        id: 2,
        pos: IVec2 { x: 5, y: 5 }, // Same position
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // Same position should be visible
    assert!(interest.include(&viewer, &enemy_same_pos));
}

/// Test RadiusTeamInterest edge cases
#[test]
fn test_radius_team_interest_exact_boundary() {
    use astraweave_net::RadiusTeamInterest;
    
    let interest = RadiusTeamInterest { radius: 10 };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_at_boundary = EntityState {
        id: 2,
        pos: IVec2 { x: 10, y: 0 }, // Distance = 10 (exactly at radius)
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // At radius^2 = 100, distance^2 = 100, should be included (<=)
    assert!(interest.include(&viewer, &enemy_at_boundary));
}

#[test]
fn test_radius_team_interest_just_outside() {
    use astraweave_net::RadiusTeamInterest;
    
    let interest = RadiusTeamInterest { radius: 10 };
    
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 1,
        ammo: 10,
    };
    
    let enemy_outside = EntityState {
        id: 2,
        pos: IVec2 { x: 11, y: 0 }, // Distance = 11 (just outside)
        hp: 100,
        team: 2,
        ammo: 10,
    };
    
    // distance^2 = 121 > 100, should NOT be included
    assert!(!interest.include(&viewer, &enemy_outside));
}
