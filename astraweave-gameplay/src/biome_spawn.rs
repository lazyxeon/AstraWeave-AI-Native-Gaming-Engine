use crate::harvesting::ResourceNode;
use crate::types::WeaveConsequence;
use crate::ResourceKind;
use glam::Vec3;
use rand::{Rng, SeedableRng};

#[derive(Clone, Debug)]
pub struct BiomeRule {
    pub name: String,
    pub weights: Vec<(ResourceKind, f32)>, // sum not required; normalized per spawn
    pub base_amount: (u32, u32),
    pub respawn: (f32, f32),
}

pub fn spawn_resources(
    seed: u64,
    area_min: Vec3,
    area_max: Vec3,
    count: usize,
    biome: &BiomeRule,
    weave: Option<&WeaveConsequence>,
) -> Vec<ResourceNode> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let sum: f32 = biome.weights.iter().map(|(_, w)| *w).sum::<f32>().max(1e-6);
    let mut out = vec![];
    for _ in 0..count {
        let r = rng.random::<f32>() * sum;
        let mut acc = 0.0;
        let mut chosen = biome.weights[0].0;
        for (k, w) in &biome.weights {
            acc += *w;
            if r <= acc {
                chosen = *k;
                break;
            }
        }
        let amt_rng = rng.random_range(biome.base_amount.0..=biome.base_amount.1);
        let mul = weave.map(|w| w.drop_multiplier).unwrap_or(1.0);
        let amount = ((amt_rng as f32) * mul).round() as u32;

        let pos = Vec3::new(
            rng.random_range(area_min.x..area_max.x),
            area_min.y,
            rng.random_range(area_min.z..area_max.z),
        );
        let resp = rng.random_range(biome.respawn.0..=biome.respawn.1);
        out.push(ResourceNode {
            kind: chosen,
            pos,
            amount,
            respawn_time: resp,
            timer: 0.0,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec3;

    fn create_test_biome() -> BiomeRule {
        BiomeRule {
            name: "TestForest".to_string(),
            weights: vec![
                (ResourceKind::Wood, 50.0),
                (ResourceKind::Fiber, 30.0),
                (ResourceKind::Essence, 20.0),
            ],
            base_amount: (5, 15),
            respawn: (20.0, 40.0),
        }
    }

    #[test]
    fn test_spawn_resources_count() {
        let biome = create_test_biome();
        let resources = spawn_resources(
            12345,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            10,
            &biome,
            None,
        );

        assert_eq!(resources.len(), 10, "Should spawn exactly 10 resources");
    }

    #[test]
    fn test_spawn_resources_deterministic() {
        let biome = create_test_biome();
        let seed = 42;

        let resources1 = spawn_resources(
            seed,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            5,
            &biome,
            None,
        );

        let resources2 = spawn_resources(
            seed,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            5,
            &biome,
            None,
        );

        assert_eq!(resources1.len(), resources2.len());
        for (r1, r2) in resources1.iter().zip(resources2.iter()) {
            assert_eq!(
                r1.kind, r2.kind,
                "Same seed should produce same resource types"
            );
            assert_eq!(r1.pos, r2.pos, "Same seed should produce same positions");
            assert_eq!(
                r1.amount, r2.amount,
                "Same seed should produce same amounts"
            );
            assert_eq!(
                r1.respawn_time, r2.respawn_time,
                "Same seed should produce same respawn times"
            );
        }
    }

    #[test]
    fn test_spawn_resources_position_bounds() {
        let biome = create_test_biome();
        let area_min = vec3(-5.0, 1.0, -5.0);
        let area_max = vec3(5.0, 1.0, 5.0);

        let resources = spawn_resources(54321, area_min, area_max, 20, &biome, None);

        for node in &resources {
            assert!(
                node.pos.x >= area_min.x && node.pos.x <= area_max.x,
                "X position should be within bounds: {} not in [{}, {}]",
                node.pos.x,
                area_min.x,
                area_max.x
            );
            assert_eq!(node.pos.y, area_min.y, "Y position should match area_min.y");
            assert!(
                node.pos.z >= area_min.z && node.pos.z <= area_max.z,
                "Z position should be within bounds: {} not in [{}, {}]",
                node.pos.z,
                area_min.z,
                area_max.z
            );
        }
    }

    #[test]
    fn test_spawn_resources_amount_range() {
        let biome = create_test_biome();

        let resources = spawn_resources(
            99999,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            50,
            &biome,
            None,
        );

        for node in &resources {
            assert!(
                node.amount >= biome.base_amount.0 && node.amount <= biome.base_amount.1,
                "Amount should be in range [{}, {}], got {}",
                biome.base_amount.0,
                biome.base_amount.1,
                node.amount
            );
        }
    }

    #[test]
    fn test_spawn_resources_respawn_time_range() {
        let biome = create_test_biome();

        let resources = spawn_resources(
            77777,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            50,
            &biome,
            None,
        );

        for node in &resources {
            assert!(
                node.respawn_time >= biome.respawn.0 && node.respawn_time <= biome.respawn.1,
                "Respawn time should be in range [{}, {}], got {}",
                biome.respawn.0,
                biome.respawn.1,
                node.respawn_time
            );
        }
    }

    #[test]
    fn test_spawn_resources_with_weave_multiplier() {
        let biome = create_test_biome();
        let weave = WeaveConsequence {
            drop_multiplier: 2.0,
            faction_disposition: 0,
            weather_shift: None,
        };

        let resources_no_weave = spawn_resources(
            11111,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            10,
            &biome,
            None,
        );

        let resources_with_weave = spawn_resources(
            11111,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            10,
            &biome,
            Some(&weave),
        );

        for (no_weave, with_weave) in resources_no_weave.iter().zip(resources_with_weave.iter()) {
            let expected_amount = (no_weave.amount as f32 * 2.0).round() as u32;
            assert_eq!(
                with_weave.amount, expected_amount,
                "Weave multiplier should double resource amounts"
            );
        }
    }

    #[test]
    fn test_spawn_resources_resource_distribution() {
        let biome = create_test_biome();

        let resources = spawn_resources(
            22222,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            100,
            &biome,
            None,
        );

        let wood_count = resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Wood)
            .count();
        let fiber_count = resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Fiber)
            .count();
        let essence_count = resources
            .iter()
            .filter(|r| r.kind == ResourceKind::Essence)
            .count();

        // With weights [50, 30, 20], expect roughly 50%, 30%, 20% distribution
        // Allow some variance due to randomness
        assert!(
            wood_count > 40 && wood_count < 60,
            "Wood should be ~50% (got {})",
            wood_count
        );
        assert!(
            fiber_count > 20 && fiber_count < 40,
            "Fiber should be ~30% (got {})",
            fiber_count
        );
        assert!(
            essence_count > 10 && essence_count < 30,
            "Essence should be ~20% (got {})",
            essence_count
        );
    }

    #[test]
    fn test_spawn_resources_timer_initialized() {
        let biome = create_test_biome();

        let resources = spawn_resources(
            33333,
            vec3(-10.0, 0.0, -10.0),
            vec3(10.0, 0.0, 10.0),
            10,
            &biome,
            None,
        );

        for node in &resources {
            assert_eq!(node.timer, 0.0, "Timer should be initialized to 0");
        }
    }
}
