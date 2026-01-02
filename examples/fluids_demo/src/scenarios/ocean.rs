use super::FluidScenario;
use crate::ocean_renderer::OceanRenderer;
use astraweave_fluids::renderer::CameraUniform;
use astraweave_fluids::{FluidRenderer, FluidSystem};
use astraweave_physics::PhysicsWorld;
use glam::Vec3;

pub struct OceanScenario {
    renderer: OceanRenderer,
    camera_pos: Vec3,
}

impl OceanScenario {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Self {
            renderer: OceanRenderer::new(device, queue, format),
            camera_pos: Vec3::ZERO,
        }
    }
}

impl FluidScenario for OceanScenario {
    fn name(&self) -> &str {
        "Infinite Ocean"
    }

    fn init(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        system: &mut FluidSystem,
        _physics: &mut PhysicsWorld,
    ) {
        system.smoothing_radius = 1.0;
        system.gravity = -9.81;
    }

    fn update(
        &mut self,
        dt: f32,
        _system: &mut FluidSystem,
        _physics: &mut PhysicsWorld,
        camera_pos: Vec3,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.camera_pos = camera_pos;
        self.renderer.update(dt, camera_pos);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _scene_view: &wgpu::TextureView,
        _scene_depth_view: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _system: &FluidSystem,
        _renderer: &FluidRenderer,
        camera_uniform: CameraUniform,
        _skybox: &wgpu::TextureView,
    ) {
        let view_proj = glam::Mat4::from_cols_array_2d(&camera_uniform.view_proj);
        self.renderer.render(encoder, view, depth, queue, view_proj);
    }
}
