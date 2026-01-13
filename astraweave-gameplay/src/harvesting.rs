use crate::{Inventory, ResourceKind};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceNode {
    pub kind: ResourceKind,
    pub pos: Vec3,
    pub amount: u32,
    pub respawn_time: f32,
    pub timer: f32,
}

impl ResourceNode {
    pub fn harvest(&mut self, inv: &mut Inventory, n: u32) -> u32 {
        let take = n.min(self.amount);
        self.amount -= take;
        inv.add_resource(self.kind, take);
        if self.amount == 0 {
            self.timer = self.respawn_time;
        }
        take
    }

    /// Non-deterministic tick (legacy compatibility).
    /// For deterministic behavior, use `tick_seeded` instead.
    #[deprecated(note = "Use tick_seeded for deterministic behavior")]
    pub fn tick(&mut self, dt: f32) {
        if self.amount == 0 {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.amount = 1 + (3 * rand::random::<u8>() as u32 % 5);
                self.timer = 0.0;
            }
        }
    }

    /// Deterministic tick using seeded RNG.
    /// 
    /// # Determinism
    /// 
    /// This method guarantees identical results given:
    /// - Same initial state
    /// - Same delta time
    /// - Same RNG state
    /// 
    /// Use this for gameplay systems requiring determinism (multiplayer, replay).
    pub fn tick_seeded<R: rand::Rng>(&mut self, dt: f32, rng: &mut R) {
        if self.amount == 0 {
            self.timer -= dt;
            if self.timer <= 0.0 {
                // Deterministic respawn amount: 1 + (0-4) * 3 = 1, 4, 7, 10, or 13
                self.amount = 1 + (rng.random::<u8>() as u32 % 5) * 3;
                self.timer = 0.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec3;

    fn create_test_inventory() -> Inventory {
        Inventory {
            items: vec![],
            resources: vec![],
        }
    }

    fn create_test_node() -> ResourceNode {
        ResourceNode {
            kind: ResourceKind::Wood,
            pos: vec3(10.0, 0.0, 5.0),
            amount: 10,
            respawn_time: 30.0,
            timer: 0.0,
        }
    }

    #[test]
    fn test_harvest_full_amount() {
        let mut node = create_test_node();
        let mut inv = create_test_inventory();

        let harvested = node.harvest(&mut inv, 5);

        assert_eq!(harvested, 5, "Should harvest requested amount");
        assert_eq!(node.amount, 5, "Node should have 5 resources left");

        // Verify inventory received resources
        assert_eq!(inv.resources.len(), 1);
        assert_eq!(inv.resources[0].0, ResourceKind::Wood);
        assert_eq!(inv.resources[0].1, 5);
    }

    #[test]
    fn test_harvest_exceeds_available() {
        let mut node = create_test_node();
        node.amount = 3; // Only 3 available
        let mut inv = create_test_inventory();

        let harvested = node.harvest(&mut inv, 10);

        assert_eq!(harvested, 3, "Should only harvest what's available");
        assert_eq!(node.amount, 0, "Node should be depleted");
        assert_eq!(inv.resources[0].1, 3);
    }

    #[test]
    fn test_harvest_depletes_node_starts_timer() {
        let mut node = create_test_node();
        node.amount = 5;
        let mut inv = create_test_inventory();

        node.harvest(&mut inv, 5); // Deplete completely

        assert_eq!(node.amount, 0, "Node should be depleted");
        assert_eq!(node.timer, 30.0, "Timer should be set to respawn_time");
    }

    #[test]
    fn test_harvest_multiple_times() {
        let mut node = create_test_node();
        let mut inv = create_test_inventory();

        node.harvest(&mut inv, 3);
        node.harvest(&mut inv, 4);

        assert_eq!(node.amount, 3, "Should have 3 left (10-3-4)");
        assert_eq!(inv.resources[0].1, 7, "Inventory should have 7 total (3+4)");
    }

    #[test]
    fn test_tick_with_resources_available() {
        let mut node = create_test_node();

        node.tick(1.0);

        assert_eq!(
            node.amount, 10,
            "Amount should not change when resources available"
        );
        assert_eq!(node.timer, 0.0, "Timer should remain at 0");
    }

    #[test]
    fn test_tick_depleted_node_countdown() {
        let mut node = create_test_node();
        node.amount = 0;
        node.timer = 30.0;

        node.tick(5.0);

        assert_eq!(node.timer, 25.0, "Timer should count down");
        assert_eq!(node.amount, 0, "Should still be depleted");
    }

    #[test]
    fn test_tick_respawn_triggers() {
        let mut node = create_test_node();
        node.amount = 0;
        node.timer = 1.0;

        #[allow(deprecated)]
        node.tick(1.5); // Tick past respawn time

        assert!(node.amount > 0, "Should respawn resources");
        assert!(
            node.amount >= 1 && node.amount <= 15,
            "Should respawn 1-15 resources (1 + rand % 5)"
        );
        assert_eq!(node.timer, 0.0, "Timer should reset");
    }

    #[test]
    fn test_tick_seeded_determinism() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        // Run same sequence 3 times with same seed
        let mut results = Vec::new();
        for _ in 0..3 {
            let mut node = create_test_node();
            node.amount = 0;
            node.timer = 1.0;
            let mut rng = StdRng::seed_from_u64(12345);

            node.tick_seeded(1.5, &mut rng);
            results.push(node.amount);
        }

        // All runs should produce identical amount
        assert_eq!(results[0], results[1], "Run 0 and 1 should match");
        assert_eq!(results[1], results[2], "Run 1 and 2 should match");
        assert!(results[0] > 0, "Should respawn resources");
    }
}
