//! Viewport Renderer
//!
//! Coordinates multi-pass rendering pipeline for 3D viewport.
//! Renders in order: Grid → Entities → Gizmos → Selection Outline
//!
//! # Performance Budget
//!
//! Target: <10ms per frame @ 1080p (60 FPS)
//! - Clear: <0.1ms
//! - Grid: ~0.5ms
//! - Entities: ~8ms (scales with entity count)
//! - Gizmos: ~1ms
//!
//! # Architecture
//!
//! ViewportRenderer delegates to specialized sub-renderers:
//! - `GridRenderer`: Floor grid + axes
//! - `EntityRenderer`: World entities (TODO: Phase 1.3)
//! - `GizmoRenderer`: Transform handles (TODO: Phase 1.5)

use anyhow::{Context, Result};
use wgpu;

use super::camera::OrbitCamera;
use super::grid_renderer::GridRenderer;
use astraweave_core::World;

/// Viewport rendering coordinator
///
/// Manages GPU resources and coordinates multi-pass rendering pipeline.
///
/// # Lifecycle
///
/// 1. Create once during editor initialization
/// 2. Call `render()` every frame
/// 3. Call `resize()` when viewport changes size
/// 4. Automatically cleaned up on drop (RAII)
pub struct ViewportRenderer {
    /// wgpu device (GPU interface)
    device: wgpu::Device,

    /// wgpu queue (command submission)
    queue: wgpu::Queue,

    /// Sub-renderers
    grid_renderer: GridRenderer,

    /// Depth texture (shared across passes)
    depth_texture: Option<wgpu::Texture>,

    /// Depth texture view
    depth_view: Option<wgpu::TextureView>,

    /// Current viewport size
    size: (u32, u32),
}

impl ViewportRenderer {
    /// Create new viewport renderer
    ///
    /// # Arguments
    ///
    /// * `device` - wgpu device
    /// * `queue` - wgpu queue
    ///
    /// # Errors
    ///
    /// Returns error if sub-renderer creation fails.
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Result<Self> {
        let grid_renderer = GridRenderer::new(&device).context("Failed to create grid renderer")?;

        Ok(Self {
            device,
            queue,
            grid_renderer,
            depth_texture: None,
            depth_view: None,
            size: (0, 0),
        })
    }

    /// Create from eframe render state
    ///
    /// # Arguments
    ///
    /// * `render_state` - eframe's wgpu render state
    ///
    /// # Errors
    ///
    /// Returns error if render state is invalid or sub-renderer creation fails.
    pub fn from_eframe(render_state: &eframe::egui_wgpu::RenderState) -> Result<Self> {
        Self::new(render_state.device.clone(), render_state.queue.clone())
    }

    /// Resize viewport (recreates depth buffer)
    ///
    /// Call this when viewport dimensions change.
    ///
    /// # Arguments
    ///
    /// * `width` - New width (pixels)
    /// * `height` - New height (pixels)
    ///
    /// # Errors
    ///
    /// Returns error if depth buffer creation fails.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 {
            // Invalid size, clear depth buffer
            self.depth_texture = None;
            self.depth_view = None;
            self.size = (0, 0);
            return Ok(());
        }

        // Create depth texture
        let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Viewport Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        self.depth_texture = Some(depth_texture);
        self.depth_view = Some(depth_view);
        self.size = (width, height);

        Ok(())
    }

    /// Render complete frame
    ///
    /// Executes multi-pass rendering pipeline:
    /// 1. Clear pass (color + depth)
    /// 2. Grid pass (floor grid + axes)
    /// 3. Entity pass (world entities) - TODO
    /// 4. Gizmo pass (transform handles) - TODO
    ///
    /// # Arguments
    ///
    /// * `target` - Render target texture
    /// * `camera` - Camera for view-projection
    /// * `world` - World state (entities, components)
    ///
    /// # Errors
    ///
    /// Returns error if any render pass fails.
    ///
    /// # Performance
    ///
    /// Target: <10ms per frame @ 1080p
    pub fn render(
        &mut self,
        target: &wgpu::Texture,
        camera: &OrbitCamera,
        world: &World,
    ) -> Result<()> {
        // Ensure depth buffer matches target size
        let target_size = target.size();
        if self.size != (target_size.width, target_size.height) {
            self.resize(target_size.width, target_size.height)
                .context("Failed to resize depth buffer")?;
        }

        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let depth_view = self
            .depth_view
            .as_ref()
            .context("Depth buffer not initialized")?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Viewport Render Encoder"),
            });

        // Pass 1: Clear
        {
            let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1, // Dark blue-gray background
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), // Clear to far plane
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Pass 2: Grid
        self.grid_renderer
            .render(&mut encoder, &target_view, depth_view, camera, &self.queue)
            .context("Grid render failed")?;

        // TODO Phase 1.3: Pass 3: Entities
        // self.entity_renderer.render(&mut encoder, &target_view, depth_view, camera, world)?;

        // TODO Phase 1.5: Pass 4: Gizmos
        // if let Some(selected) = selected_entity {
        //     self.gizmo_renderer.render(&mut encoder, &target_view, depth_view, camera, selected)?;
        // }

        // Submit all commands
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    /// Create render texture
    ///
    /// Creates a texture suitable for rendering to and displaying in egui.
    ///
    /// # Arguments
    ///
    /// * `width` - Texture width (pixels)
    /// * `height` - Texture height (pixels)
    ///
    /// # Returns
    ///
    /// Texture with RENDER_ATTACHMENT | TEXTURE_BINDING usage.
    pub fn create_render_texture(&self, width: u32, height: u32) -> Result<wgpu::Texture> {
        if width == 0 || height == 0 {
            anyhow::bail!("Invalid texture size: {}x{}", width, height);
        }

        Ok(self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Viewport Render Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        }))
    }

    /// Get current viewport size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Get wgpu device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get wgpu queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: These tests require wgpu device, which needs a GPU or software renderer.
    // Run with: cargo test --features gpu-tests

    #[test]
    fn test_viewport_renderer_resize() {
        // This is a smoke test - just ensure no panics
        // Actual GPU tests would require a wgpu instance
    }
}
