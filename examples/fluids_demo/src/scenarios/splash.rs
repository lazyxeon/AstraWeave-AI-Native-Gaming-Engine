use super::FluidScenario;
use astraweave_fluids::renderer::CameraUniform;
use astraweave_fluids::{FluidRenderer, FluidSystem, Particle};
use astraweave_physics::PhysicsWorld;
use glam::Vec3;

/// Splash interaction scenario demonstrating multi-phase fluids
/// with oil and water separation effects.
pub struct SplashScenario {
    name: String,
    time: f32,
}

impl SplashScenario {
    pub fn new() -> Self {
        Self {
            name: "Multi-Phase Splash".to_string(),
            time: 0.0,
        }
    }
}

impl FluidScenario for SplashScenario {
    fn name(&self) -> &str {
        &self.name
    }

    fn init(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        system: &mut FluidSystem,
        _physics: &mut PhysicsWorld,
    ) {
        // Configure for multi-phase simulation
        system.smoothing_radius = 0.5;
        system.target_density = 10.0;
        system.pressure_multiplier = 180.0;
        system.viscosity = 30.0;
        system.surface_tension = 0.1;
        system.gravity = -9.81;

        let particle_count = system.particle_count;
        let mut particles = Vec::with_capacity(particle_count as usize);

        let half = particle_count as usize / 2;
        let spacing = 0.45;

        // Phase 0: Water (blue) - lower layer
        for i in 0..half {
            let x = (i % 20) as f32 * spacing - 4.5;
            let y = ((i / 20) % 15) as f32 * spacing + 1.0;
            let z = (i / 300) as f32 * spacing - 2.0;

            particles.push(Particle {
                position: [x, y, z, 1.0],
                velocity: [0.0; 4],
                predicted_position: [x, y, z, 1.0],
                color: [0.2, 0.5, 1.0, 0.9], // Blue water
                lambda: 0.0,
                density: 0.0,
                phase: 0, // Water
                temperature: 293.0,
            });
        }

        // Phase 1: Oil (yellow) - upper layer (lighter, floats)
        for i in half..particle_count as usize {
            let idx = i - half;
            let x = (idx % 20) as f32 * spacing - 4.5;
            let y = ((idx / 20) % 10) as f32 * spacing + 8.0;
            let z = (idx / 200) as f32 * spacing - 2.0;

            particles.push(Particle {
                position: [x, y, z, 1.0],
                velocity: [0.0; 4],
                predicted_position: [x, y, z, 1.0],
                color: [0.9, 0.7, 0.2, 0.85], // Yellow oil
                lambda: 0.0,
                density: 0.0,
                phase: 1, // Oil
                temperature: 293.0,
            });
        }

        system.reset_particles(queue, &particles);
        self.time = 0.0;
    }

    fn update(
        &mut self,
        dt: f32,
        _system: &mut FluidSystem,
        _physics: &mut PhysicsWorld,
        _camera_pos: Vec3,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.time += dt;
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        scene_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        _depth: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        system: &FluidSystem,
        renderer: &FluidRenderer,
        camera_uniform: CameraUniform,
        skybox: &wgpu::TextureView,
    ) {
        renderer.render(
            encoder,
            view,
            scene_view,
            scene_depth_view,
            skybox,
            system.get_particle_buffer(),
            system.particle_count,
            system.secondary_particle_buffer(),
            system.secondary_particle_count(),
            camera_uniform,
            queue,
            device,
        );
    }
}
