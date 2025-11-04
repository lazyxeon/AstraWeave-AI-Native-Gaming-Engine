use crate::types::InstanceRaw;
use glam::{vec3, Mat4, Vec3};
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub enum WeatherKind {
    None,
    Rain,
    WindTrails,
}

pub struct WeatherFx {
    kind: WeatherKind,
    particles: Vec<Particle>,
    buf: wgpu::Buffer,
    max: usize,
}

#[derive(Clone, Copy, Debug)]
struct Particle {
    pos: Vec3,
    vel: Vec3,
    life: f32,
    color: [f32; 4],
    scale: Vec3,
}

impl WeatherFx {
    pub fn new(device: &wgpu::Device, max: usize) -> Self {
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("weather inst"),
            size: (max * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            kind: WeatherKind::None,
            particles: vec![],
            buf,
            max,
        }
    }

    pub fn set_kind(&mut self, kind: WeatherKind) {
        self.kind = kind;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: f32) {
        match self.kind {
            WeatherKind::None => {
                self.particles.clear();
            }
            WeatherKind::Rain => self.tick_rain(dt),
            WeatherKind::WindTrails => self.tick_wind(dt),
        }
        // upload
        let raws: Vec<InstanceRaw> = self
            .particles
            .iter()
            .map(|p| {
                let m = Mat4::from_scale_rotation_translation(p.scale, glam::Quat::IDENTITY, p.pos);
                InstanceRaw {
                    model: m.to_cols_array_2d(),
                    normal_matrix: [
                        m.inverse().transpose().x_axis.truncate().to_array(),
                        m.inverse().transpose().y_axis.truncate().to_array(),
                        m.inverse().transpose().z_axis.truncate().to_array(),
                    ],
                    color: p.color,
                    material_id: 0,
                    _padding: [0; 3],
                }
            })
            .collect();
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(&raws));
    }

    fn tick_rain(&mut self, dt: f32) {
        let mut rng = rand::rng();
        // spawn up to max
        while self.particles.len() < self.max {
            self.particles.push(Particle {
                pos: vec3(
                    rng.random_range(-25.0..25.0),
                    rng.random_range(8.0..18.0),
                    rng.random_range(-25.0..25.0),
                ),
                vel: vec3(0.0, -20.0, 0.0),
                life: rng.random_range(0.5..1.5),
                color: [0.7, 0.8, 1.0, 0.9],
                scale: vec3(0.02, 0.5, 0.02),
            });
        }
        // update
        self.particles.retain_mut(|p| {
            p.life -= dt;
            p.pos += p.vel * dt;
            p.pos.y > 0.0 && p.life > 0.0
        });
    }

    fn tick_wind(&mut self, dt: f32) {
        let mut rng = rand::rng();
        while self.particles.len() < self.max {
            self.particles.push(Particle {
                pos: vec3(
                    rng.random_range(-25.0..25.0),
                    rng.random_range(0.5..4.0),
                    rng.random_range(-25.0..25.0),
                ),
                vel: vec3(5.0, 0.0, 1.0),
                life: rng.random_range(1.0..3.0),
                color: [1.0, 1.0, 1.0, 0.3],
                scale: vec3(0.05, 0.05, 0.8),
            });
        }
        self.particles.retain_mut(|p| {
            p.life -= dt;
            p.pos += p.vel * dt;
            p.life > 0.0
        });
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buf
    }
    pub fn count(&self) -> u32 {
        self.particles.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: true,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("test_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: Default::default(),
            })
            .await
            .expect("Failed to create device")
    }

    #[test]
    fn test_weather_fx_new() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            let fx = WeatherFx::new(&device, 1000);

            assert_eq!(fx.max, 1000);
            assert_eq!(fx.count(), 0, "Should start with no particles");
        });
    }

    #[test]
    fn test_weather_fx_set_kind() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            let mut fx = WeatherFx::new(&device, 100);

            fx.set_kind(WeatherKind::Rain);
            fx.set_kind(WeatherKind::WindTrails);
            fx.set_kind(WeatherKind::None);

            // Should not crash
        });
    }

    #[test]
    fn test_weather_fx_update_none() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let mut fx = WeatherFx::new(&device, 100);

            fx.set_kind(WeatherKind::None);
            fx.update(&queue, 0.016); // One frame

            assert_eq!(fx.count(), 0, "None weather should have no particles");
        });
    }

    #[test]
    fn test_weather_fx_update_rain() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let mut fx = WeatherFx::new(&device, 100);

            fx.set_kind(WeatherKind::Rain);
            fx.update(&queue, 0.016);

            assert!(fx.count() > 0, "Rain should spawn particles");
            assert!(fx.count() <= 100, "Should not exceed max particles");
        });
    }

    #[test]
    fn test_weather_fx_update_wind() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let mut fx = WeatherFx::new(&device, 100);

            fx.set_kind(WeatherKind::WindTrails);
            fx.update(&queue, 0.016);

            assert!(fx.count() > 0, "Wind should spawn particles");
            assert!(fx.count() <= 100, "Should not exceed max particles");
        });
    }

    #[test]
    fn test_weather_fx_rain_spawns_up_to_max() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let max = 50;
            let mut fx = WeatherFx::new(&device, max);

            fx.set_kind(WeatherKind::Rain);

            // Update multiple times to fill particles
            for _ in 0..10 {
                fx.update(&queue, 0.016);
            }

            assert_eq!(fx.count(), max as u32, "Should fill to max capacity");
        });
    }

    #[test]
    fn test_weather_fx_particles_despawn() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let mut fx = WeatherFx::new(&device, 100);

            fx.set_kind(WeatherKind::Rain);
            fx.update(&queue, 0.016);

            // Update with large dt to age out particles
            for _ in 0..100 {
                fx.update(&queue, 1.0); // 1 second per frame
            }

            // Rain continuously spawns, so should maintain particles
            // (verifies spawning and despawning cycle works without crashing)
            // Count may fluctuate but system should be stable
            assert!(fx.count() <= 100, "Should not exceed max");
        });
    }

    #[test]
    fn test_weather_fx_switch_kind_clears() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let mut fx = WeatherFx::new(&device, 100);

            fx.set_kind(WeatherKind::Rain);
            fx.update(&queue, 0.016);
            assert!(fx.count() > 0, "Rain should spawn particles");

            fx.set_kind(WeatherKind::None);
            fx.update(&queue, 0.016);
            assert_eq!(fx.count(), 0, "None should clear all particles");
        });
    }

    #[test]
    fn test_weather_kind_debug() {
        let kinds = vec![
            WeatherKind::None,
            WeatherKind::Rain,
            WeatherKind::WindTrails,
        ];

        for kind in kinds {
            let debug_str = format!("{:?}", kind);
            assert!(!debug_str.is_empty(), "Debug should work");
        }
    }

    #[test]
    fn test_weather_fx_buffer_exists() {
        pollster::block_on(async {
            let (device, _queue) = create_test_device().await;
            let fx = WeatherFx::new(&device, 100);

            let _buf = fx.buffer();
            // Should return buffer reference without panic
        });
    }
}
