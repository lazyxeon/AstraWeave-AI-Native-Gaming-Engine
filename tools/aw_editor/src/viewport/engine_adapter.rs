#![cfg(feature = "astraweave-render")]

use anyhow::{Context, Result};
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
}

#[cfg(not(feature = "astraweave-render"))]
pub struct EngineRenderAdapter;

#[cfg(not(feature = "astraweave-render"))]
impl EngineRenderAdapter {
    pub fn is_initialized(&self) -> bool {
        false
    }
}
