use crate::World;

pub struct SimConfig {
    pub dt: f32,
}

pub fn step(w: &mut World, cfg: &SimConfig) {
    w.tick(cfg.dt);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_config_creation() {
        let cfg = SimConfig { dt: 0.016 };
        assert_eq!(cfg.dt, 0.016);
    }

    #[test]
    fn test_sim_config_different_dt() {
        let cfg1 = SimConfig { dt: 0.016 };
        let cfg2 = SimConfig { dt: 0.033 };
        assert_eq!(cfg1.dt, 0.016);
        assert_eq!(cfg2.dt, 0.033);
    }

    #[test]
    fn test_step_doesnt_crash() {
        let mut world = World::new();
        let cfg = SimConfig { dt: 0.016 };
        step(&mut world, &cfg); // Should not crash
    }

    #[test]
    fn test_step_multiple_times() {
        let mut world = World::new();
        let cfg = SimConfig { dt: 0.016 };
        // Run 10 ticks without crashing
        for _ in 0..10 {
            step(&mut world, &cfg);
        }
    }

    #[test]
    fn test_step_with_different_dt_values() {
        let mut world = World::new();

        step(&mut world, &SimConfig { dt: 0.016 });
        step(&mut world, &SimConfig { dt: 0.033 });
        step(&mut world, &SimConfig { dt: 0.008 });
        // Should not crash with varying dt
    }
}
