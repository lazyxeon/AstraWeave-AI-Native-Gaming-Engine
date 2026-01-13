use super::FluidScenario;
use astraweave_fluids::renderer::CameraUniform;
use astraweave_fluids::{FluidRenderer, FluidSystem, Particle};
use astraweave_physics::PhysicsWorld;
use glam::Vec3;

/// Waterfall scenario demonstrating continuous particle emission
/// and splash effects on impact with a basin.
pub struct WaterfallScenario {
    name: String,
    emit_timer: f32,
    emit_interval: f32,
    particles_per_emit: usize,
}

impl WaterfallScenario {
    pub fn new() -> Self {
        Self {
            name: "Waterfall".to_string(),
            emit_timer: 0.0,
            emit_interval: 0.05, // Emit every 50ms
            particles_per_emit: 50,
        }
    }
}

impl FluidScenario for WaterfallScenario {
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
        // Configure for waterfall simulation
        system.smoothing_radius = 0.6;
        system.target_density = 8.0;
        system.pressure_multiplier = 150.0;
        system.viscosity = 20.0;
        system.surface_tension = 0.05;
        system.gravity = -15.0; // Stronger gravity for waterfall

        // Initialize with some particles at the waterfall source
        let particle_count = system.particle_count;
        let mut particles = Vec::with_capacity(particle_count as usize);

        // Start with particles in waterfall column
        let spacing = 0.4;
        for i in 0..particle_count as usize {
            let x = (i % 5) as f32 * spacing - 1.0;
            let y = ((i / 5) % 30) as f32 * spacing + 15.0;
            let z = (i / 150) as f32 * spacing - 1.0;

            particles.push(Particle {
                position: [x, y, z, 1.0],
                velocity: [0.0, -5.0, 0.0, 0.0], // Initial downward velocity
                predicted_position: [x, y, z, 1.0],
                color: [0.4, 0.7, 1.0, 1.0], // Light blue water
                lambda: 0.0,
                density: 0.0,
                phase: 0,
                temperature: 288.0, // Cool waterfall water
            });
        }

        system.reset_particles(queue, &particles);
        self.emit_timer = 0.0;
    }

    fn update(
        &mut self,
        dt: f32,
        system: &mut FluidSystem,
        _physics: &mut PhysicsWorld,
        _camera_pos: Vec3,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        // Continuous emission from waterfall source
        self.emit_timer += dt;

        if self.emit_timer >= self.emit_interval {
            self.emit_timer = 0.0;

            // Spawn new particles at waterfall source
            let mut positions = Vec::with_capacity(self.particles_per_emit);
            let mut velocities = Vec::with_capacity(self.particles_per_emit);
            let mut colors = Vec::with_capacity(self.particles_per_emit);

            for i in 0..self.particles_per_emit {
                let x = (i % 5) as f32 * 0.3 - 0.6 + (rand_f32() - 0.5) * 0.2;
                let y = 20.0 + rand_f32() * 0.5;
                let z = (i / 5) as f32 * 0.3 - 0.6 + (rand_f32() - 0.5) * 0.2;

                positions.push([x, y, z]);
                velocities.push([
                    (rand_f32() - 0.5) * 2.0,
                    -8.0 - rand_f32() * 3.0,
                    (rand_f32() - 0.5) * 2.0,
                ]);
                colors.push([0.4, 0.75, 1.0, 1.0]);
            }

            system.spawn_particles(queue, &positions, &velocities, Some(&colors));
        }

        // Despawn particles that fall too low
        let _ = system.despawn_region(queue, [-50.0, -10.0, -50.0], [50.0, -5.0, 50.0]);
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

// Simple deterministic random for demo
fn rand_f32() -> f32 {
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        ((SEED >> 16) & 0x7FFF) as f32 / 32767.0
    }
}
