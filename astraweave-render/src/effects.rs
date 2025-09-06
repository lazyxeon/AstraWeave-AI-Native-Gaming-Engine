use glam::{vec3, Mat4, Vec3};
use crate::types::{InstanceRaw};
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub enum WeatherKind { None, Rain, WindTrails }

pub struct WeatherFx {
    kind: WeatherKind,
    particles: Vec<Particle>,
    buf: wgpu::Buffer,
    max: usize,
}

#[derive(Clone, Copy, Debug)]
struct Particle { pos: Vec3, vel: Vec3, life: f32, color: [f32;4], scale: Vec3 }

impl WeatherFx {
    pub fn new(device: &wgpu::Device, max: usize) -> Self {
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("weather inst"),
            size: (max * std::mem::size_of::<InstanceRaw>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self { kind: WeatherKind::None, particles: vec![], buf, max }
    }

    pub fn set_kind(&mut self, kind: WeatherKind) { self.kind = kind; }

    pub fn update(&mut self, queue: &wgpu::Queue, dt: f32) {
        match self.kind {
            WeatherKind::None => { self.particles.clear(); }
            WeatherKind::Rain => self.tick_rain(dt),
            WeatherKind::WindTrails => self.tick_wind(dt),
        }
        // upload
        let raws: Vec<InstanceRaw> = self.particles.iter().map(|p| {
            let m = Mat4::from_scale_rotation_translation(p.scale, glam::Quat::IDENTITY, p.pos);
            InstanceRaw {
                model: m.to_cols_array_2d(),
                normal_matrix: [
                    m.inverse().transpose().x_axis.truncate().to_array(),
                    m.inverse().transpose().y_axis.truncate().to_array(),
                    m.inverse().transpose().z_axis.truncate().to_array(),
                ],
                color: p.color,
            }
        }).collect();
        queue.write_buffer(&self.buf, 0, bytemuck::cast_slice(&raws));
    }

    fn tick_rain(&mut self, dt: f32) {
        let mut rng = rand::thread_rng();
        // spawn up to max
        while self.particles.len() < self.max {
            self.particles.push(Particle {
                pos: vec3(rng.gen_range(-25.0..25.0), rng.gen_range(8.0..18.0), rng.gen_range(-25.0..25.0)),
                vel: vec3(0.0, -20.0, 0.0),
                life: rng.gen_range(0.5..1.5),
                color: [0.7,0.8,1.0,0.9],
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
        let mut rng = rand::thread_rng();
        while self.particles.len() < self.max {
            self.particles.push(Particle {
                pos: vec3(rng.gen_range(-25.0..25.0), rng.gen_range(0.5..4.0), rng.gen_range(-25.0..25.0)),
                vel: vec3(5.0, 0.0, 1.0),
                life: rng.gen_range(1.0..3.0),
                color: [1.0,1.0,1.0,0.3],
                scale: vec3(0.05, 0.05, 0.8),
            });
        }
        self.particles.retain_mut(|p| {
            p.life -= dt;
            p.pos += p.vel * dt;
            p.life > 0.0
        });
    }

    pub fn buffer(&self) -> &wgpu::Buffer { &self.buf }
    pub fn count(&self) -> u32 { self.particles.len() as u32 }
}
