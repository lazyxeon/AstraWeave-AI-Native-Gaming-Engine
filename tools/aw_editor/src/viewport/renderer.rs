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
use super::entity_renderer::EntityRenderer;
use super::gizmo_renderer::GizmoRendererWgpu;
use super::grid_renderer::GridRenderer;
use super::skybox_renderer::SkyboxRenderer;
use crate::gizmo::{GizmoMode, GizmoState};
use astraweave_core::{Entity, World};

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
    skybox_renderer: SkyboxRenderer,
    entity_renderer: EntityRenderer,
    gizmo_renderer: GizmoRendererWgpu,

    /// Depth texture (shared across passes)
    depth_texture: Option<wgpu::Texture>,

    /// Depth texture view
    depth_view: Option<wgpu::TextureView>,

    /// Current viewport size
    size: (u32, u32),

    /// Currently selected entities (for highlighting) - supports multi-selection
    selected_entities: Vec<Entity>,
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
        let skybox_renderer =
            SkyboxRenderer::new(&device).context("Failed to create skybox renderer")?;
        let entity_renderer =
            EntityRenderer::new(&device, 10000).context("Failed to create entity renderer")?;
        let gizmo_renderer = GizmoRendererWgpu::new(device.clone(), queue.clone(), 10000)
            .context("Failed to create gizmo renderer")?;

        Ok(Self {
            device,
            queue,
            grid_renderer,
            skybox_renderer,
            entity_renderer,
            gizmo_renderer,
            depth_texture: None,
            depth_view: None,
            size: (0, 0),
            selected_entities: Vec::new(),
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

    /// Render the 3D scene
    ///
    /// Multi-pass rendering pipeline:
    /// 1. Skybox pass (clears depth and renders background gradient)
    /// 2. Grid pass (render floor grid)
    /// 3. Entity pass (render all world entities)
    /// 4. Gizmo pass (render transform gizmos if entity selected)
    ///
    /// # Arguments
    ///
    /// * `target` - Render target texture
    /// * `camera` - Camera for view-projection
    /// * `world` - Entity world state
    /// * `gizmo_state` - Optional gizmo state (for transform operations)
    pub fn render(
        &mut self,
        target: &wgpu::Texture,
        camera: &OrbitCamera,
        world: &World,
        gizmo_state: Option<&GizmoState>,
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

        // Pass 1: Skybox (clears color/depth and renders gradient background)
        self.skybox_renderer
            .render(&mut encoder, &target_view, depth_view, camera, &self.queue)
            .context("Skybox render failed")?;

        // Pass 2: Grid
        self.grid_renderer
            .render(&mut encoder, &target_view, depth_view, camera, &self.queue)
            .context("Grid render failed")?;

        // Pass 3: Entities
        self.entity_renderer
            .render(
                &mut encoder,
                &target_view,
                depth_view,
                camera,
                world,
                &self.selected_entities,
                &self.queue,
            )
            .context("Entity render failed")?;

        // Pass 4: Gizmos (if entity selected and gizmo active)
        if let (Some(selected), Some(gizmo)) = (self.selected_entity(), gizmo_state) {
            if gizmo.mode != crate::gizmo::GizmoMode::Inactive {
                // DEBUG: Log gizmo mode and constraint
                match &gizmo.mode {
                    crate::gizmo::GizmoMode::Rotate { constraint } => {
                        println!("🎨 Renderer: Rendering Rotate gizmo, constraint = {:?}", constraint);
                    }
                    _ => {}
                }
                
                // Get entity position from world (old astraweave-core API)
                if let Some(pose) = world.pose(selected) {
                    // Convert astraweave_core::IVec2 to glam::IVec2
                    let glam_pos = glam::IVec2::new(pose.pos.x, pose.pos.y);

                    self.gizmo_renderer
                        .render(
                            &mut encoder,
                            &target_view,
                            depth_view,
                            camera,
                            gizmo,
                            glam_pos,
                            &self.queue,
                        )
                        .context("Gizmo render failed")?;
                }
            }
        }

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

    /// Set selected entities (for highlighting) - supports multi-selection
    pub fn set_selected_entities(&mut self, entities: &[Entity]) {
        self.selected_entities = entities.to_vec();
    }
    
    /// Set selected entity (for backward compatibility)
    pub fn set_selected_entity(&mut self, entity: Option<Entity>) {
        self.selected_entities.clear();
        if let Some(e) = entity {
            self.selected_entities.push(e);
        }
    }

    /// Get selected entity (returns first selected for backward compatibility)
    pub fn selected_entity(&self) -> Option<Entity> {
        self.selected_entities.first().copied()
    }
    
    /// Get all selected entities
    pub fn selected_entities(&self) -> &[Entity] {
        &self.selected_entities
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
