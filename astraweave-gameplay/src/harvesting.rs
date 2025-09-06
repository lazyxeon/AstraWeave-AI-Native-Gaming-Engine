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

    pub fn tick(&mut self, dt: f32) {
        if self.amount == 0 {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.amount = 1 + (3 * rand::random::<u8>() as u32 % 5);
                self.timer = 0.0;
            }
        }
    }
}
