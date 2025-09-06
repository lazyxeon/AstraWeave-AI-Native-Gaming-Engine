use crate::World;

pub struct SimConfig {
    pub dt: f32,
}

pub fn step(w: &mut World, cfg: &SimConfig) {
    w.tick(cfg.dt);
}
