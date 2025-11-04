use crate::{DamageType, Stats};
use astraweave_physics::PhysicsWorld;
use glam::Vec3;
use rapier3d::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct IFrame {
    pub time_left: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Parry {
    pub window: f32,
    pub active: bool,
}

pub struct Combatant {
    pub body: u64, // PhysicsWorld BodyId
    pub stats: Stats,
    pub iframes: Option<IFrame>,
    pub parry: Option<Parry>,
}

pub struct HitResult {
    pub target: u64,
    pub damage: i32,
    pub parried: bool,
}

/// Sweep a capsule from `from` to `to`, apply damage to first hit collider body that isn't `self_id`.
///
/// Uses Rapier3D 0.22 API with raycast as a simplified attack sweep.
/// Returns HitResult if a valid target is hit, considering parry windows and iframes.
#[allow(clippy::too_many_arguments)]
pub fn perform_attack_sweep(
    phys: &mut PhysicsWorld,
    self_id: u64,
    from: Vec3,
    to: Vec3,
    _radius: f32,
    base_damage: i32,
    dtype: DamageType,
    targets: &mut [Combatant],
) -> Option<HitResult> {
    // Calculate sweep direction and distance
    let dir = to - from;
    let distance = dir.length();

    // Early exit if sweep distance is negligible
    if distance <= 1e-3 {
        return None;
    }

    let dir_normalized = dir / distance;

    // Adjust raycast to character center height (assuming 2m tall characters)
    let ray_from = from + Vec3::new(0.0, 1.0, 0.0);

    // Use a raycast with radius check for simplified attack detection
    let ray = Ray::new(
        point![ray_from.x, ray_from.y, ray_from.z],
        vector![dir_normalized.x, dir_normalized.y, dir_normalized.z],
    );

    // Create filter to exclude self from raycast
    let filter = if let Some(self_handle) = phys.handle_of(self_id) {
        QueryFilter::default().exclude_rigid_body(self_handle)
    } else {
        QueryFilter::default()
    };

    // Cast ray and check hits within radius
    if let Some((collider_handle, hit)) = phys.query_pipeline.cast_ray_and_get_normal(
        &phys.bodies,
        &phys.colliders,
        &ray,
        distance,
        true, // stop at first hit
        filter,
    ) {
        // Get the rigid body that owns this collider
        if let Some(collider) = phys.colliders.get(collider_handle) {
            if let Some(body_handle) = collider.parent() {
                // Get the body ID from PhysicsWorld's internal mapping
                if let Some(target_id) = phys.id_of(body_handle) {
                    // Don't hit ourselves
                    if target_id == self_id {
                        return None;
                    }

                    // Check if hit is within attack radius (for sweep effect)
                    let hit_point_dist = hit.time_of_impact;
                    if hit_point_dist > distance {
                        return None;
                    }

                    // Filter by attack cone (check if target is within forward cone)
                    // This prevents hitting targets behind the attacker
                    let hit_point = ray_from + dir_normalized * hit_point_dist;
                    let to_target = (hit_point - ray_from).normalize_or_zero();
                    if to_target.length_squared() < 0.01 {
                        return None;
                    }
                    let dot = dir_normalized.dot(to_target);

                    // Only hit targets in front (dot > 0.5 = ~60 degree cone)
                    if dot < 0.5 {
                        return None;
                    }

                    // Find the matching combatant target
                    if let Some(target) = targets.iter_mut().find(|t| t.body == target_id) {
                        // Check parry window
                        if let Some(parry) = &mut target.parry {
                            if parry.active && parry.window > 0.0 {
                                parry.window = 0.0;
                                parry.active = false;
                                return Some(HitResult {
                                    target: target_id,
                                    damage: 0,
                                    parried: true,
                                });
                            }
                        }

                        // Check invincibility frames
                        if let Some(iframe) = &target.iframes {
                            if iframe.time_left > 0.0 {
                                return Some(HitResult {
                                    target: target_id,
                                    damage: 0,
                                    parried: false,
                                });
                            }
                        }

                        // Apply damage
                        let damage = target.stats.apply_damage(base_damage, dtype);
                        return Some(HitResult {
                            target: target_id,
                            damage,
                            parried: false,
                        });
                    }
                }
            }
        }
    }

    None
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a basic combatant for testing
    fn create_test_combatant(body_id: u64, hp: i32) -> Combatant {
        Combatant {
            body: body_id,
            stats: Stats {
                hp,
                stamina: 100,
                power: 10,
                defense: 0,
                echo_amp: 1.0,
                effects: vec![],
            },
            iframes: None,
            parry: None,
        }
    }

    /// Test 1: Single Enemy Hit
    /// Verifies that a basic attack sweep hits a single enemy and applies damage
    #[test]
    fn test_single_enemy_hit() {
        let mut phys = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Create attacker and target
        let attacker_id = phys.add_character(Vec3::ZERO, Vec3::new(0.5, 1.0, 0.5));
        let target_id = phys.add_character(Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

        // Step physics to update query pipeline
        phys.step();

        let mut targets = vec![create_test_combatant(target_id, 100)];

        // Perform attack sweep from attacker toward target
        let result = perform_attack_sweep(
            &mut phys,
            attacker_id,
            Vec3::ZERO,
            Vec3::new(3.0, 0.0, 0.0),
            0.5,
            20, // base damage
            DamageType::Physical,
            &mut targets,
        );

        // Verify hit was registered
        assert!(result.is_some(), "Attack should hit the target");
        let hit = result.unwrap();
        assert_eq!(hit.target, target_id);
        assert_eq!(hit.damage, 20);
        assert!(!hit.parried);

        // Verify target took damage
        assert_eq!(targets[0].stats.hp, 80);
    }

    /// Test 2: Cone Filtering
    /// Verifies that targets outside the attack cone are not hit
    #[test]
    fn test_cone_filtering() {
        let mut phys = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Create attacker and target behind attacker
        let attacker_id = phys.add_character(Vec3::ZERO, Vec3::new(0.5, 1.0, 0.5));
        let target_id = phys.add_character(Vec3::new(-2.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

        // Step physics to update query pipeline
        phys.step();

        let mut targets = vec![create_test_combatant(target_id, 100)];

        // Perform attack sweep forward (target is behind, should not hit)
        let result = perform_attack_sweep(
            &mut phys,
            attacker_id,
            Vec3::ZERO,
            Vec3::new(3.0, 0.0, 0.0), // Attacking forward
            0.5,
            20,
            DamageType::Physical,
            &mut targets,
        );

        // Verify no hit (target is behind attacker, outside cone)
        assert!(
            result.is_none(),
            "Attack should not hit target outside cone"
        );
        assert_eq!(targets[0].stats.hp, 100, "Target should take no damage");
    }

    /// Test 3: Multi-Hit Potential
    /// Verifies that attack stops at first hit (doesn't pierce through)
    #[test]
    fn test_first_hit_only() {
        let mut phys = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Create attacker and two targets in line
        let attacker_id = phys.add_character(Vec3::ZERO, Vec3::new(0.5, 1.0, 0.5));
        let target1_id = phys.add_character(Vec3::new(1.5, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        let target2_id = phys.add_character(Vec3::new(3.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

        // Step physics to update query pipeline
        phys.step();

        let mut targets = vec![
            create_test_combatant(target1_id, 100),
            create_test_combatant(target2_id, 100),
        ];

        // Perform attack sweep through both targets
        let result = perform_attack_sweep(
            &mut phys,
            attacker_id,
            Vec3::ZERO,
            Vec3::new(4.0, 0.0, 0.0),
            0.5,
            20,
            DamageType::Physical,
            &mut targets,
        );

        // Verify only first target was hit
        assert!(result.is_some(), "Attack should hit first target");
        let hit = result.unwrap();
        assert_eq!(hit.target, target1_id, "Should hit first target in line");

        // Verify only first target took damage
        assert_eq!(targets[0].stats.hp, 80, "First target should take damage");
        assert_eq!(targets[1].stats.hp, 100, "Second target should not be hit");
    }

    /// Test 4: Range Limiting
    /// Verifies that attacks don't hit targets beyond sweep distance
    #[test]
    fn test_range_limiting() {
        let mut phys = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Create attacker and distant target
        let attacker_id = phys.add_character(Vec3::ZERO, Vec3::new(0.5, 1.0, 0.5));
        let target_id = phys.add_character(Vec3::new(10.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

        // Step physics to update query pipeline
        phys.step();

        let mut targets = vec![create_test_combatant(target_id, 100)];

        // Perform short-range attack (target is too far)
        let result = perform_attack_sweep(
            &mut phys,
            attacker_id,
            Vec3::ZERO,
            Vec3::new(2.0, 0.0, 0.0), // Only 2 units range, target is at 10 units
            0.5,
            20,
            DamageType::Physical,
            &mut targets,
        );

        // Verify no hit due to range
        assert!(result.is_none(), "Attack should not reach distant target");
        assert_eq!(targets[0].stats.hp, 100, "Target should take no damage");
    }

    /// Test 5: Parry System
    /// Verifies that active parry windows block damage and are consumed
    #[test]
    fn test_parry_blocks_damage() {
        let mut phys = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Create attacker and target with active parry
        let attacker_id = phys.add_character(Vec3::ZERO, Vec3::new(0.5, 1.0, 0.5));
        let target_id = phys.add_character(Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

        // Step physics to update query pipeline
        phys.step();

        let mut targets = vec![Combatant {
            body: target_id,
            stats: Stats {
                hp: 100,
                stamina: 100,
                power: 10,
                defense: 0,
                echo_amp: 1.0,
                effects: vec![],
            },
            iframes: None,
            parry: Some(Parry {
                window: 0.2, // Active parry window
                active: true,
            }),
        }];

        // Perform attack sweep
        let result = perform_attack_sweep(
            &mut phys,
            attacker_id,
            Vec3::ZERO,
            Vec3::new(3.0, 0.0, 0.0),
            0.5,
            20,
            DamageType::Physical,
            &mut targets,
        );

        // Verify hit was parried
        assert!(result.is_some(), "Attack should register as hit");
        let hit = result.unwrap();
        assert_eq!(hit.target, target_id);
        assert_eq!(hit.damage, 0, "Parried attack should deal no damage");
        assert!(hit.parried, "Attack should be marked as parried");

        // Verify target took no damage and parry was consumed
        assert_eq!(
            targets[0].stats.hp, 100,
            "Target should take no damage from parry"
        );
        assert_eq!(
            targets[0].parry.as_ref().unwrap().window,
            0.0,
            "Parry window should be consumed"
        );
        assert!(
            !targets[0].parry.as_ref().unwrap().active,
            "Parry should be deactivated"
        );
    }

    /// Test 6: Invincibility Frames
    /// Verifies that iframes block damage without being consumed
    #[test]
    fn test_iframes_block_damage() {
        let mut phys = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

        // Create attacker and target with iframes
        let attacker_id = phys.add_character(Vec3::ZERO, Vec3::new(0.5, 1.0, 0.5));
        let target_id = phys.add_character(Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.5, 1.0, 0.5));

        // Step physics to update query pipeline
        phys.step();

        let mut targets = vec![Combatant {
            body: target_id,
            stats: Stats {
                hp: 100,
                stamina: 100,
                power: 10,
                defense: 0,
                echo_amp: 1.0,
                effects: vec![],
            },
            iframes: Some(IFrame { time_left: 0.5 }),
            parry: None,
        }];

        // Perform attack sweep
        let result = perform_attack_sweep(
            &mut phys,
            attacker_id,
            Vec3::ZERO,
            Vec3::new(3.0, 0.0, 0.0),
            0.5,
            20,
            DamageType::Physical,
            &mut targets,
        );

        // Verify hit was blocked by iframes
        assert!(result.is_some(), "Attack should register as hit");
        let hit = result.unwrap();
        assert_eq!(hit.damage, 0, "Attack during iframes should deal no damage");
        assert!(!hit.parried, "iframes are not parries");

        // Verify target took no damage and iframes persist
        assert_eq!(
            targets[0].stats.hp, 100,
            "Target should take no damage during iframes"
        );
        assert_eq!(
            targets[0].iframes.as_ref().unwrap().time_left,
            0.5,
            "iframes should not be consumed by attack"
        );
    }
}
