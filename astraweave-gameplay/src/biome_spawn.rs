use glam::Vec3;
use rand::Rng;
use crate::{ResourceKind};
use crate::harvesting::ResourceNode;
use crate::weaving::WeaveConsequence;

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
    let sum: f32 = biome.weights.iter().map(|(_,w)| *w).sum::<f32>().max(1e-6);
    let mut out = vec![];
    for _ in 0..count {
        let r = rng.random::<f32>() * sum;
        let mut acc = 0.0;
        let mut chosen = biome.weights[0].0;
        for (k, w) in &biome.weights {
            acc += *w;
            if r <= acc { chosen = *k; break; }
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
        out.push(ResourceNode { kind: chosen, pos, amount, respawn_time: resp, timer: 0.0 });
    }
    out
}
