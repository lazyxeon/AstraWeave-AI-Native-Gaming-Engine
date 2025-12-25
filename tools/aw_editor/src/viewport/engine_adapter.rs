#![cfg(feature = "astraweave-render")]

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use wgpu;

use super::camera::OrbitCamera;

pub struct EngineRenderAdapter {
    renderer: astraweave_render::Renderer,
    initialized: bool,
}

impl EngineRenderAdapter {
    pub async fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        width: u32,
        height: u32,
    ) -> Result<Self> {
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let device_owned = (*device).clone();
        let queue_owned = (*queue).clone();

        let renderer =
            astraweave_render::Renderer::new_from_device(device_owned, queue_owned, None, config)
                .await
                .context("Failed to create engine renderer")?;

        Ok(Self {
            renderer,
            initialized: true,
        })
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn update_camera(&mut self, camera: &OrbitCamera) {
        let engine_camera = camera.to_engine_camera();
        self.renderer.update_camera(&engine_camera);
    }

    pub fn render_to_texture(
        &mut self,
        target: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()> {
        self.renderer
            .draw_into(target, encoder)
            .context("Engine draw_into failed")
    }

    pub fn renderer(&self) -> &astraweave_render::Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut astraweave_render::Renderer {
        &mut self.renderer
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.renderer.resize(width, height);
        }
    }

    pub fn load_gltf_model(&mut self, name: impl Into<String>, path: &Path) -> Result<()> {
        use astraweave_render::{mesh_gltf, Instance};

        let opts = mesh_gltf::GltfOptions::default();
        let cpu_meshes = mesh_gltf::load_gltf(path, &opts)
            .with_context(|| format!("Failed to load glTF: {}", path.display()))?;

        if cpu_meshes.is_empty() {
            anyhow::bail!("glTF file contains no meshes: {}", path.display());
        }

        let mesh = self.renderer.create_mesh_from_cpu_mesh(&cpu_meshes[0]);
        let instance = Instance::from_pos_scale_color(
            glam::Vec3::ZERO,
            glam::Vec3::ONE,
            [1.0, 1.0, 1.0, 1.0],
        );
        self.renderer.add_model(name, mesh, &[instance]);
        Ok(())
    }

    pub fn has_model(&self, name: &str) -> bool {
        self.renderer.has_model(name)
    }

    pub fn clear_model(&mut self, name: &str) {
        self.renderer.clear_model(name);
    }
}

#[cfg(not(feature = "astraweave-render"))]
pub struct EngineRenderAdapter;

#[cfg(not(feature = "astraweave-render"))]
impl EngineRenderAdapter {
    pub fn is_initialized(&self) -> bool {
        false
    }
}
