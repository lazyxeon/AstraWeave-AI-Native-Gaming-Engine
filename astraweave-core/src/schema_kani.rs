//! Kani formal verification proofs for core schema types
//!
//! These proofs verify:
//! - IVec2 mathematical properties (distance, operations)
//! - WorldSnapshot helper correctness
//! - Snapshot invariants
//!
//! Run with: `cargo kani --package astraweave-core`

use crate::schema::{CompanionState, EnemyState, IVec2, PlayerState, Poi, WorldSnapshot};
use std::collections::BTreeMap;

// ============================================================================
// IVec2 Mathematical Properties
// ============================================================================

/// Verify IVec2::new constructs correctly
#[kani::proof]
fn ivec2_new_constructs_correctly() {
    let x: i32 = kani::any();
    let y: i32 = kani::any();
    
    let v = IVec2::new(x, y);
    
    kani::assert(v.x == x, "x must match constructor argument");
    kani::assert(v.y == y, "y must match constructor argument");
}

/// Verify IVec2::zero returns (0, 0)
#[kani::proof]
fn ivec2_zero_is_zero() {
    let zero = IVec2::zero();
    
    kani::assert(zero.x == 0, "zero.x must be 0");
    kani::assert(zero.y == 0, "zero.y must be 0");
    kani::assert(zero.is_zero(), "zero.is_zero() must be true");
}

/// Verify is_zero only for (0, 0)
#[kani::proof]
fn ivec2_is_zero_correct() {
    let x: i32 = kani::any();
    let y: i32 = kani::any();
    
    let v = IVec2::new(x, y);
    
    if v.is_zero() {
        kani::assert(x == 0 && y == 0, "is_zero implies both components are 0");
    }
    
    if x == 0 && y == 0 {
        kani::assert(v.is_zero(), "(0,0) must be zero");
    }
}

/// Verify Manhattan distance is non-negative
#[kani::proof]
fn manhattan_distance_non_negative() {
    let ax: i32 = kani::any();
    let ay: i32 = kani::any();
    let bx: i32 = kani::any();
    let by: i32 = kani::any();
    
    // Bound inputs to prevent subtraction overflow and abs(i32::MIN) overflow
    // Range [-1_000_000_000, 1_000_000_000] ensures difference fits in i32
    kani::assume(ax > -1_000_000_000 && ax < 1_000_000_000);
    kani::assume(ay > -1_000_000_000 && ay < 1_000_000_000);
    kani::assume(bx > -1_000_000_000 && bx < 1_000_000_000);
    kani::assume(by > -1_000_000_000 && by < 1_000_000_000);
    
    let a = IVec2::new(ax, ay);
    let b = IVec2::new(bx, by);
    
    let dist = a.manhattan_distance(&b);
    
    kani::assert(dist >= 0, "Manhattan distance must be non-negative");
}

/// Verify Manhattan distance is symmetric
#[kani::proof]
fn manhattan_distance_symmetric() {
    let ax: i32 = kani::any();
    let ay: i32 = kani::any();
    let bx: i32 = kani::any();
    let by: i32 = kani::any();
    
    // Bound inputs to prevent overflow
    kani::assume(ax > -1_000_000_000 && ax < 1_000_000_000);
    kani::assume(ay > -1_000_000_000 && ay < 1_000_000_000);
    kani::assume(bx > -1_000_000_000 && bx < 1_000_000_000);
    kani::assume(by > -1_000_000_000 && by < 1_000_000_000);
    
    let a = IVec2::new(ax, ay);
    let b = IVec2::new(bx, by);
    
    let dist_ab = a.manhattan_distance(&b);
    let dist_ba = b.manhattan_distance(&a);
    
    kani::assert(dist_ab == dist_ba, "Manhattan distance must be symmetric");
}

/// Verify distance to self is zero
#[kani::proof]
fn manhattan_distance_to_self_is_zero() {
    let a = IVec2::new(kani::any(), kani::any());
    
    let dist = a.manhattan_distance(&a);
    
    kani::assert(dist == 0, "Distance to self must be zero");
}

/// Verify squared distance is non-negative
#[kani::proof]
fn distance_squared_non_negative() {
    let ax: i32 = kani::any();
    let ay: i32 = kani::any();
    let bx: i32 = kani::any();
    let by: i32 = kani::any();
    
    // Bound to avoid overflow: max diff = 60000, max squared sum = 2*60000^2 = 7.2B > i32::MAX
    // Use max diff = 30000, max squared sum = 2*30000^2 = 1.8B < i32::MAX
    kani::assume(ax > -15000 && ax < 15000);
    kani::assume(ay > -15000 && ay < 15000);
    kani::assume(bx > -15000 && bx < 15000);
    kani::assume(by > -15000 && by < 15000);
    
    let a = IVec2::new(ax, ay);
    let b = IVec2::new(bx, by);
    
    let dist_sq = a.distance_squared(&b);
    
    kani::assert(dist_sq >= 0, "Squared distance must be non-negative");
}

/// Verify squared distance is symmetric
#[kani::proof]
fn distance_squared_symmetric() {
    let ax: i32 = kani::any();
    let ay: i32 = kani::any();
    let bx: i32 = kani::any();
    let by: i32 = kani::any();
    
    // Bound to avoid overflow (same as above)
    kani::assume(ax > -15000 && ax < 15000);
    kani::assume(ay > -15000 && ay < 15000);
    kani::assume(bx > -15000 && bx < 15000);
    kani::assume(by > -15000 && by < 15000);
    
    let a = IVec2::new(ax, ay);
    let b = IVec2::new(bx, by);
    
    let dist_ab = a.distance_squared(&b);
    let dist_ba = b.distance_squared(&a);
    
    kani::assert(dist_ab == dist_ba, "Squared distance must be symmetric");
}

/// Verify squared distance to self is zero
#[kani::proof]
fn distance_squared_to_self_is_zero() {
    let a = IVec2::new(kani::any(), kani::any());
    
    let dist = a.distance_squared(&a);
    
    kani::assert(dist == 0, "Squared distance to self must be zero");
}

/// Verify Add is commutative
#[kani::proof]
fn ivec2_add_commutative() {
    let ax: i32 = kani::any();
    let ay: i32 = kani::any();
    let bx: i32 = kani::any();
    let by: i32 = kani::any();
    
    // Bound to avoid overflow: sum of two values must fit in i32
    kani::assume(ax > -1_000_000_000 && ax < 1_000_000_000);
    kani::assume(ay > -1_000_000_000 && ay < 1_000_000_000);
    kani::assume(bx > -1_000_000_000 && bx < 1_000_000_000);
    kani::assume(by > -1_000_000_000 && by < 1_000_000_000);
    
    let a = IVec2::new(ax, ay);
    let b = IVec2::new(bx, by);
    
    let ab = a + b;
    let ba = b + a;
    
    kani::assert(ab.x == ba.x && ab.y == ba.y, "Addition must be commutative");
}

/// Verify Add identity (zero)
#[kani::proof]
fn ivec2_add_identity() {
    let a = IVec2::new(kani::any(), kani::any());
    let zero = IVec2::zero();
    
    let result = a + zero;
    
    kani::assert(result.x == a.x && result.y == a.y, "Adding zero must return original");
}

/// Verify Sub inverse
#[kani::proof]
fn ivec2_sub_inverse() {
    let ax: i32 = kani::any();
    let ay: i32 = kani::any();
    
    let a = IVec2::new(ax, ay);
    
    // a - a is always 0, no overflow possible since same values cancel
    let result = a - a;
    
    kani::assert(result.x == 0 && result.y == 0, "a - a must be zero");
}

/// Verify offset is equivalent to add
#[kani::proof]
fn ivec2_offset_correct() {
    let ax: i32 = kani::any();
    let ay: i32 = kani::any();
    let dx: i32 = kani::any();
    let dy: i32 = kani::any();
    
    // Bound to avoid overflow
    kani::assume(ax > -1_000_000_000 && ax < 1_000_000_000);
    kani::assume(ay > -1_000_000_000 && ay < 1_000_000_000);
    kani::assume(dx > -1_000_000_000 && dx < 1_000_000_000);
    kani::assume(dy > -1_000_000_000 && dy < 1_000_000_000);
    
    let a = IVec2::new(ax, ay);
    
    let offset_result = a.offset(dx, dy);
    let add_result = a + IVec2::new(dx, dy);
    
    kani::assert(
        offset_result.x == add_result.x && offset_result.y == add_result.y,
        "offset must be equivalent to add"
    );
}

// ============================================================================
// WorldSnapshot Helper Properties
// ============================================================================

/// Verify enemy_count matches Vec::len
#[kani::proof]
fn enemy_count_matches_len() {
    let snap = WorldSnapshot::default();
    
    kani::assert(
        snap.enemy_count() == snap.enemies.len(),
        "enemy_count must match Vec::len"
    );
}

/// Verify has_no_enemies consistency
#[kani::proof]
fn has_no_enemies_correct() {
    let snap = WorldSnapshot::default();
    
    kani::assert(
        snap.has_no_enemies() == snap.enemies.is_empty(),
        "has_no_enemies must match is_empty"
    );
}

/// Verify has_ammo consistency
#[kani::proof]
fn has_ammo_correct() {
    let ammo: i32 = kani::any();
    
    let mut snap = WorldSnapshot::default();
    snap.me.ammo = ammo;
    
    kani::assert(
        snap.has_ammo() == (ammo > 0),
        "has_ammo must be true iff ammo > 0"
    );
}

/// Verify has_pois consistency
#[kani::proof]
fn has_pois_correct() {
    let snap = WorldSnapshot::default();
    
    kani::assert(
        snap.has_pois() == !snap.pois.is_empty(),
        "has_pois must match !is_empty"
    );
}

/// Verify has_objective consistency
#[kani::proof]
fn has_objective_correct() {
    let snap = WorldSnapshot::default();
    
    kani::assert(
        snap.has_objective() == snap.objective.is_some(),
        "has_objective must match is_some"
    );
}

// ============================================================================
// Default Implementation Properties
// ============================================================================

/// Verify PlayerState default has valid health
#[kani::proof]
fn player_state_default_valid() {
    let player = PlayerState::default();
    
    kani::assert(player.hp == 100, "Default HP must be 100");
    kani::assert(player.pos.x == 0 && player.pos.y == 0, "Default pos must be origin");
}

/// Verify CompanionState default has valid ammo
#[kani::proof]
fn companion_state_default_valid() {
    let companion = CompanionState::default();
    
    kani::assert(companion.ammo == 10, "Default ammo must be 10");
    kani::assert(companion.morale == 1.0, "Default morale must be 1.0");
    kani::assert(companion.cooldowns.is_empty(), "Default cooldowns must be empty");
}

/// Verify EnemyState default has valid health
#[kani::proof]
fn enemy_state_default_valid() {
    let enemy = EnemyState::default();
    
    kani::assert(enemy.hp == 100, "Default HP must be 100");
    kani::assert(enemy.id == 0, "Default ID must be 0");
}

/// Verify WorldSnapshot default is consistent
#[kani::proof]
fn world_snapshot_default_consistent() {
    let snap = WorldSnapshot::default();
    
    kani::assert(snap.t == 0.0, "Default time must be 0.0");
    kani::assert(snap.has_no_enemies(), "Default must have no enemies");
    kani::assert(!snap.has_pois(), "Default must have no POIs");
    kani::assert(!snap.has_objective(), "Default must have no objective");
}
