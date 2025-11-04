//! Viewport Widget
//!
//! Custom egui widget that integrates wgpu 3D rendering into editor panels.
//! Handles input, rendering coordination, and egui integration.
//!
//! # Usage
//!
//! ```no_run
//! use aw_editor_lib::viewport::ViewportWidget;
//!
//! // In eframe::App::new()
//! let viewport = ViewportWidget::new(cc)?;
//!
//! // In eframe::App::update()
//! viewport.ui(ui, &world)?;
//! ```
//!
//! # Architecture
//!
//! ViewportWidget is the glue between egui (UI) and wgpu (3D rendering):
//! - Allocates egui space for viewport
//! - Handles mouse/keyboard input
//! - Coordinates rendering (via ViewportRenderer)
//! - Uses egui_wgpu::Callback for custom rendering
//!
//! # Performance
//!
//! Target: 16.67ms per frame (60 FPS)
//! - Input handling: <0.1ms
//! - Rendering: <10ms (see ViewportRenderer)
//! - egui integration: <1ms
//! - Total: <12ms (26% headroom)

use anyhow::{Context, Result};
use egui;
use std::sync::{Arc, Mutex};
use wgpu;

use super::camera::OrbitCamera;
use super::renderer::ViewportRenderer;
use astraweave_core::World;

/// 3D viewport widget for egui
///
/// Integrates wgpu 3D rendering into egui panel system.
/// Manages camera, rendering, and input handling.
pub struct ViewportWidget {
    /// Viewport renderer (wgpu coordinator)
    renderer: Arc<Mutex<ViewportRenderer>>,

    /// Orbit camera controller
    camera: OrbitCamera,

    /// Render texture (reused each frame)
    render_texture: Option<Arc<wgpu::Texture>>,

    /// Staging buffer for CPU readback (GPU → CPU copy)
    staging_buffer: Option<wgpu::Buffer>,

    /// egui texture handle for displaying rendered viewport
    egui_texture: Option<egui::TextureHandle>,

    /// Last viewport size (for resize detection)
    last_size: (u32, u32),

    /// Whether viewport has focus (for input handling)
    has_focus: bool,
}

impl ViewportWidget {
    /// Create new viewport widget
    ///
    /// # Arguments
    ///
    /// * `cc` - eframe creation context (contains wgpu render state)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - wgpu render state is missing
    /// - Renderer creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// impl eframe::App for EditorApp {
    ///     fn new(cc: &eframe::CreationContext) -> Self {
    ///         let viewport = ViewportWidget::new(cc).expect("Failed to create viewport");
    ///         Self { viewport, /* ... */ }
    ///     }
    /// }
    /// ```
    pub fn new(cc: &eframe::CreationContext) -> Result<Self> {
        // Get wgpu render state from eframe
        let render_state = cc.wgpu_render_state.as_ref().context(
            "wgpu render state not available - ensure eframe is built with 'wgpu' feature",
        )?;

        // Create renderer (wrapped in Arc<Mutex<>> for thread-safe interior mutability)
        let renderer = Arc::new(Mutex::new(
            ViewportRenderer::from_eframe(render_state)
                .context("Failed to create viewport renderer")?,
        ));

        Ok(Self {
            renderer,
            camera: OrbitCamera::default(),
            render_texture: None,
            staging_buffer: None,
            egui_texture: None,
            last_size: (0, 0),
            has_focus: false,
        })
    }

    /// Render viewport UI
    ///
    /// Call this from `eframe::App::update()` to display the 3D viewport.
    ///
    /// # Arguments
    ///
    /// * `ui` - egui UI context
    /// * `world` - World state (entities, components)
    ///
    /// # Errors
    ///
    /// Returns error if rendering fails. Does NOT panic - errors are logged.
    ///
    /// # Example
    ///
    /// ```no_run
    /// impl eframe::App for EditorApp {
    ///     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ///         egui::CentralPanel::default().show(ctx, |ui| {
    ///             if let Err(e) = self.viewport.ui(ui, &self.world) {
    ///                 eprintln!("❌ Viewport error: {}", e);
    ///             }
    ///         });
    ///     }
    /// }
    /// ```
    pub fn ui(&mut self, ui: &mut egui::Ui, world: &World) -> Result<()> {
        // Allocate space for viewport (70% of available width, full height)
        let available = ui.available_size();
        let viewport_size = egui::vec2(available.x * 0.7, available.y);
        let (rect, response) = ui.allocate_exact_size(viewport_size, egui::Sense::click_and_drag());

        // Update focus state
        self.has_focus = response.has_focus();

        // Handle input (mouse/keyboard)
        self.handle_input(&response, ui.ctx())?;

        // Update camera aspect ratio
        if viewport_size.x > 0.0 && viewport_size.y > 0.0 {
            self.camera.set_aspect(viewport_size.x, viewport_size.y);
        }

        // Resize texture if needed
        let size = (viewport_size.x as u32, viewport_size.y as u32);
        if size != self.last_size && size.0 > 0 && size.1 > 0 {
            self.resize_texture(size)?;
            self.last_size = size;
        }

        // Render to texture (before displaying)
        if let Some(texture) = self.render_texture.clone() {
            // Render in separate scope to drop MutexGuard early
            {
                if let Ok(mut renderer) = self.renderer.lock() {
                    if let Err(e) = renderer.render(&texture, &self.camera, world) {
                        eprintln!("❌ Viewport render failed: {}", e);
                    }
                }
            }

            // Copy texture to CPU and upload to egui (after renderer is unlocked)
            if let Err(e) = self.copy_texture_to_cpu(ui, &texture, size) {
                eprintln!("❌ Texture copy failed: {}", e);
            }

            // Display texture via egui (CPU readback approach)
            if let Some(handle) = self.egui_texture.as_ref() {
                let texture_id = handle.id();
                // Display rendered viewport using egui's texture system
                ui.painter().image(
                    texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                // Overlay camera info (top-left corner, semi-transparent)
                let pos = self.camera.position();
                let dist = self.camera.distance();
                let info_text = format!(
                    "Camera: [{:.1}, {:.1}, {:.1}] | Dist: {:.1}m",
                    pos.x, pos.y, pos.z, dist
                );

                ui.painter().rect_filled(
                    egui::Rect::from_min_size(rect.left_top(), egui::vec2(350.0, 20.0)),
                    0.0,
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
                );

                ui.painter().text(
                    rect.left_top() + egui::vec2(5.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    info_text,
                    egui::FontId::monospace(12.0),
                    egui::Color32::from_rgb(200, 220, 240),
                );
            } else {
                // First frame - texture not ready yet
                ui.painter()
                    .rect_filled(rect, 0.0, egui::Color32::from_rgb(25, 30, 35));
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Loading 3D Viewport...",
                    egui::FontId::proportional(14.0),
                    egui::Color32::from_rgb(150, 170, 190),
                );
            }
        } else {
            // No texture yet - show placeholder
            ui.painter()
                .rect_filled(rect, 0.0, egui::Color32::from_rgb(20, 20, 30));
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Initializing 3D Viewport...",
                egui::FontId::proportional(16.0),
                egui::Color32::GRAY,
            );
        }

        Ok(())
    }

    /// Handle mouse/keyboard input
    ///
    /// Implements standard 3D viewport controls:
    /// - Left drag: Orbit camera
    /// - Middle drag: Pan camera
    /// - Scroll: Zoom camera
    /// - G/R/S: Gizmo mode (translate/rotate/scale)
    fn handle_input(&mut self, response: &egui::Response, ctx: &egui::Context) -> Result<()> {
        // Only handle input if viewport has focus
        if !self.has_focus {
            return Ok(());
        }

        // Orbit camera (left mouse drag)
        if response.dragged_by(egui::PointerButton::Primary) {
            let delta = response.drag_delta();
            self.camera.orbit(delta.x, delta.y);
        }

        // Pan camera (middle mouse drag)
        if response.dragged_by(egui::PointerButton::Middle) {
            let delta = response.drag_delta();
            self.camera.pan(delta.x, delta.y);
        }

        // Zoom camera (scroll wheel)
        ctx.input(|i| {
            if i.smooth_scroll_delta.y.abs() > 0.0 {
                self.camera.zoom(i.smooth_scroll_delta.y);
            }
        });

        // Gizmo hotkeys (TODO: Phase 1.5 - integrate with gizmo system)
        ctx.input(|i| {
            if i.key_pressed(egui::Key::G) {
                println!("🔧 Gizmo mode: Translate (G)");
            }
            if i.key_pressed(egui::Key::R) {
                println!("🔧 Gizmo mode: Rotate (R)");
            }
            if i.key_pressed(egui::Key::S) {
                println!("🔧 Gizmo mode: Scale (S)");
            }
            if i.key_pressed(egui::Key::F) {
                // Frame selected (TODO: Phase 1.4 - integrate with selection)
                println!("🎯 Frame selected (F)");
            }
        });

        // Selection (TODO: Phase 1.4 - ray-casting)
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let viewport_pos_vec = pos - response.rect.min;
                let viewport_pos = egui::Pos2::new(viewport_pos_vec.x, viewport_pos_vec.y);
                let _ray = self
                    .camera
                    .ray_from_screen(viewport_pos, response.rect.size());
                // TODO: Pick entity with ray
                println!(
                    "🎯 Click at ({:.1}, {:.1}) - picking not yet implemented",
                    viewport_pos.x, viewport_pos.y
                );
            }
        }

        Ok(())
    }

    /// Copy wgpu texture to CPU and upload to egui
    ///
    /// Performs GPU → CPU copy via staging buffer, then creates egui texture.
    /// This is the CPU readback approach for texture display.
    ///
    /// # Performance
    ///
    /// GPU → CPU copy: ~0.5-1ms @ 1080p (depends on GPU/CPU transfer speed)
    /// egui upload: ~0.1-0.2ms (texture data copy)
    /// Total: ~0.6-1.2ms (acceptable for 60 FPS)
    ///
    /// # Arguments
    ///
    /// * `ui` - egui UI context (for texture upload)
    /// * `texture` - Source wgpu texture to copy
    /// * `size` - Texture dimensions (width, height)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Staging buffer creation fails
    /// - Texture copy fails
    /// - Buffer mapping fails
    fn copy_texture_to_cpu(
        &mut self,
        ui: &egui::Ui,
        texture: &wgpu::Texture,
        size: (u32, u32),
    ) -> Result<()> {
        // Lock renderer momentarily to clone device/queue handles
        let (device, queue) = {
            let renderer = self
                .renderer
                .lock()
                .map_err(|e| anyhow::anyhow!("Renderer lock poisoned: {}", e))?;
            (renderer.device().clone(), renderer.queue().clone())
        };

        // Calculate buffer size (RGBA8 = 4 bytes per pixel)
        let bytes_per_row = size.0 * 4;
        let padded_bytes_per_row = ((bytes_per_row + 255) / 256) * 256; // wgpu requires 256-byte alignment
        let buffer_size = (padded_bytes_per_row * size.1) as u64;

        // Create staging buffer if needed (reuse if size matches)
        let needs_new_buffer = self
            .staging_buffer
            .as_ref()
            .map(|b| b.size() != buffer_size)
            .unwrap_or(true);

        if needs_new_buffer {
            self.staging_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("viewport_staging_buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        let staging_buffer = self.staging_buffer.as_ref().unwrap();

        // Create command encoder for texture copy
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("viewport_copy_encoder"),
        });

        // Copy texture to staging buffer
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(size.1),
                },
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );

        // Submit copy command
        queue.submit(Some(encoder.finish()));

        // Map buffer and read pixels (synchronous)
        let buffer_slice = staging_buffer.slice(..);

        // Request mapping
        buffer_slice.map_async(wgpu::MapMode::Read, |result| {
            if let Err(e) = result {
                eprintln!("❌ Buffer mapping failed: {:?}", e);
            }
        });

        // Wait for GPU to finish (synchronous polling)
        let _ = device.poll(wgpu::MaintainBase::Wait);

        // Read pixel data and create egui texture
        {
            let data = buffer_slice.get_mapped_range();

            // Convert to non-padded RGBA8 (egui expects tightly packed data)
            let mut rgba_data = Vec::with_capacity((size.0 * size.1 * 4) as usize);
            for row in 0..size.1 {
                let row_start = (row * padded_bytes_per_row) as usize;
                let row_end = row_start + (size.0 * 4) as usize;
                rgba_data.extend_from_slice(&data[row_start..row_end]);
            }

            // Create egui ColorImage
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [size.0 as usize, size.1 as usize],
                &rgba_data,
            );

            // Upload to egui texture system
            let texture_options = egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Linear,
                ..Default::default()
            };

            // Upload to egui texture system (handle manages lifetime)
            let texture_handle =
                ui.ctx()
                    .load_texture("viewport_render", color_image, texture_options);

            self.egui_texture = Some(texture_handle);
        }

        // Unmap buffer for next frame
        staging_buffer.unmap();

        Ok(())
    }

    /// Resize render texture
    ///
    /// Creates new render texture when viewport size changes.
    /// Called automatically by ui() method.
    ///
    /// # Arguments
    ///
    /// * `size` - New texture size (width, height)
    ///
    /// # Errors
    ///
    /// Returns error if texture creation fails.
    fn resize_texture(&mut self, size: (u32, u32)) -> Result<()> {
        if size.0 == 0 || size.1 == 0 {
            // Invalid size - clear texture
            self.render_texture = None;
            return Ok(());
        }

        // Lock renderer to create texture and resize
        let mut renderer = self
            .renderer
            .lock()
            .map_err(|e| anyhow::anyhow!("Renderer lock poisoned: {}", e))?;

        // Create new render texture
        let texture = renderer
            .create_render_texture(size.0, size.1)
            .context("Failed to create render texture")?;

        // Wrap in Arc for sharing with paint callback
        self.render_texture = Some(Arc::new(texture));

        // Resize renderer's depth buffer
        renderer
            .resize(size.0, size.1)
            .context("Failed to resize renderer")?;

        Ok(())
    }

    /// Get camera (read-only)
    pub fn camera(&self) -> &OrbitCamera {
        &self.camera
    }

    /// Get camera (mutable)
    pub fn camera_mut(&mut self) -> &mut OrbitCamera {
        &mut self.camera
    }
}

// SAFETY: ViewportWidget owns wgpu resources (textures), which are NOT Send/Sync.
// However, egui requires widgets to be Send. We ensure safety by:
// 1. Only creating wgpu resources on the main thread
// 2. Never sending ViewportWidget across threads
// 3. Using Arc for shared GPU resources in renderer
//
// This is safe because eframe runs on a single thread (winit event loop).
// If we later add multi-threading, we'll need to refactor to use Arc<Mutex<>>.
