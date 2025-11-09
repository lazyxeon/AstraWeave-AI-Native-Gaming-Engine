//! Viewport Widget
//!
//! Custom egui widget that integrates wgpu 3D rendering into editor panels.
//! Handles input, rendering coordination, and egui integration.

#![allow(dead_code)]
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
use super::toolbar::ViewportToolbar;
use crate::entity_manager::EntityManager;
use crate::gizmo::{GizmoMode, GizmoState, TransformSnapshot};
use astraweave_core::{Entity, Team, World};

/// Camera bookmark for F1-F12 quick recall
#[derive(Clone, Debug)]
struct CameraBookmark {
    focal_point: glam::Vec3,
    distance: f32,
    yaw: f32,
    pitch: f32,
}

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

    /// Viewport toolbar
    toolbar: ViewportToolbar,

    /// Currently selected entities (supports multi-selection)
    selected_entities: Vec<Entity>,

    /// Track if left mouse button was pressed (for click detection)
    mouse_pressed_pos: Option<egui::Pos2>,

    /// Frame time tracking for FPS calculation
    last_frame_time: std::time::Instant,
    frame_times: Vec<f32>,

    /// Gizmo state (for transform manipulation)
    gizmo_state: GizmoState,

    /// Grid snap size (1.0 = snap to integer grid)
    grid_snap_size: f32,

    /// Angle snap increment in radians (default: 15° = 0.2617994 rad)
    angle_snap_increment: f32,

    /// Camera bookmarks (F1-F12)
    camera_bookmarks: [Option<CameraBookmark>; 12],
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
            toolbar: ViewportToolbar::default(),
            selected_entities: Vec::new(),
            mouse_pressed_pos: None,
            last_frame_time: std::time::Instant::now(),
            frame_times: Vec::with_capacity(60),
            gizmo_state: GizmoState::new(),
            grid_snap_size: 1.0, // Default: snap to 1 unit grid
            angle_snap_increment: 15.0_f32.to_radians(), // 15 degrees
            camera_bookmarks: [None, None, None, None, None, None, None, None, None, None, None, None],
        })
    }

    /// Render viewport and handle input
    ///
    /// # Arguments
    ///
    /// * `ui` - egui UI context
    /// * `world` - Game world (for entity data)
    /// * `entity_manager` - Entity manager (for transforms and picking)
    ///
    /// # Example
    ///
    /// ```no_run
    /// impl eframe::App for EditorApp {
    ///     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    ///         egui::CentralPanel::default().show(ctx, |ui| {
    ///             self.viewport.ui(ui, &self.world, &mut self.entity_manager)?;
    ///         });
    ///     }
    /// }
    /// ```
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        world: &mut World,
        entity_manager: &mut EntityManager,
        undo_stack: &mut crate::command::UndoStack, // Phase 2.1: Command integration
    ) -> Result<()> {
        // Update frame time tracking
        let now = std::time::Instant::now();
        let frame_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }

        // Calculate FPS
        let avg_frame_time = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
        let fps = if avg_frame_time > 0.0 {
            1.0 / avg_frame_time
        } else {
            0.0
        };

        // Allocate space for viewport (full available space)
        let available = ui.available_size();
        let viewport_size = egui::vec2(available.x, available.y);
        let (rect, response) = ui.allocate_exact_size(viewport_size, egui::Sense::click_and_drag());

        // Request focus on hover or click (enables camera controls)
        if response.hovered() || response.clicked() {
            println!(
                "🖱️ Viewport: hovered={}, clicked={}",
                response.hovered(),
                response.clicked()
            );
            response.request_focus();
        }

        // Update focus state
        self.has_focus = response.has_focus();

        // Debug: Log response state
        if response.hovered() {
            println!(
                "🎯 Viewport hovered, has_focus={}, dragged={}",
                self.has_focus,
                response.dragged_by(egui::PointerButton::Primary)
            );
        }

        // Handle input (mouse/keyboard) - always process, but camera only moves if focused
        self.handle_input(&response, ui.ctx(), world, entity_manager, undo_stack)?;

        // Request continuous repaint to update viewport every frame
        ui.ctx().request_repaint();

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

        // Update renderer selected entities
        {
            if let Ok(mut renderer) = self.renderer.lock() {
                renderer.set_selected_entities(&self.selected_entities);
            }
        }

        // Render to texture (before displaying)
        if let Some(texture) = self.render_texture.clone() {
            // Render in separate scope to drop MutexGuard early
            {
                if let Ok(mut renderer) = self.renderer.lock() {
                    if let Err(e) =
                        renderer.render(&texture, &self.camera, world, Some(&self.gizmo_state))
                    {
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

                // TODO: Add visual border for focus/hover states
                // (egui 0.32 API for borders needs verification)

                // Display rendered viewport using egui's texture system
                ui.painter().image(
                    texture_id,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                // Overlay camera info (top-right corner, semi-transparent)
                let pos = self.camera.position();
                let dist = self.camera.distance();
                let info_text = format!(
                    "Camera: [{:.1}, {:.1}, {:.1}] | Dist: {:.1}m",
                    pos.x, pos.y, pos.z, dist
                );

                let info_width = 350.0;
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(
                        rect.right_top() + egui::vec2(-info_width - 10.0, 10.0),
                        egui::vec2(info_width, 20.0),
                    ),
                    0.0,
                    egui::Color32::from_rgba_premultiplied(0, 0, 0, 180),
                );

                ui.painter().text(
                    rect.right_top() + egui::vec2(-info_width - 5.0, 12.0),
                    egui::Align2::LEFT_TOP,
                    info_text,
                    egui::FontId::monospace(12.0),
                    egui::Color32::from_rgb(200, 220, 240),
                );

                // Camera reset button (below camera info, top-right)
                let button_rect = egui::Rect::from_min_size(
                    rect.right_top() + egui::vec2(-120.0, 40.0),
                    egui::vec2(110.0, 25.0),
                );
                
                // Check if mouse is over button
                let pointer_pos = ui.ctx().pointer_latest_pos();
                let is_hovering = pointer_pos.map_or(false, |pos| button_rect.contains(pos));
                
                // Button background (highlight on hover)
                let button_color = if is_hovering {
                    egui::Color32::from_rgba_premultiplied(60, 120, 180, 220)
                } else {
                    egui::Color32::from_rgba_premultiplied(40, 80, 140, 200)
                };
                
                ui.painter().rect_filled(button_rect, 3.0, button_color);
                
                // Button text
                ui.painter().text(
                    button_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "🎯 Reset Camera",
                    egui::FontId::proportional(13.0),
                    egui::Color32::WHITE,
                );
                
                // Handle click
                if is_hovering && ui.ctx().input(|i| i.pointer.primary_clicked()) {
                    self.camera.reset_to_origin();
                    println!("📷 Camera reset to origin");
                }

                // Snapping indicator (top-right, below camera info)
                if self.gizmo_state.is_active() {
                    let snap_enabled = ui.ctx().input(|i| i.modifiers.ctrl || i.modifiers.command);
                    
                    if snap_enabled {
                        let snap_text = match self.gizmo_state.mode {
                            crate::gizmo::GizmoMode::Translate { .. } => {
                                format!("📐 Grid Snap: {:.2}m", self.grid_snap_size)
                            }
                            crate::gizmo::GizmoMode::Rotate { .. } => {
                                format!("🔄 Angle Snap: {}°", self.angle_snap_increment.to_degrees() as i32)
                            }
                            _ => String::new(),
                        };
                        
                        if !snap_text.is_empty() {
                            let snap_width = 200.0;
                            let snap_rect = egui::Rect::from_min_size(
                                rect.right_top() + egui::vec2(-snap_width - 10.0, 75.0),
                                egui::vec2(snap_width, 25.0),
                            );
                            
                            // Bright background to indicate active snapping
                            ui.painter().rect_filled(
                                snap_rect,
                                3.0,
                                egui::Color32::from_rgba_premultiplied(100, 200, 100, 220),
                            );
                            
                            ui.painter().text(
                                snap_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                snap_text,
                                egui::FontId::proportional(13.0),
                                egui::Color32::BLACK,
                            );
                        }
                    }
                }

                // Update and display toolbar
                self.toolbar.stats.fps = fps;
                self.toolbar.stats.frame_time_ms = avg_frame_time * 1000.0;
                // TODO: Get actual entity/triangle counts from renderer
                self.toolbar.stats.entity_count = 100; // Placeholder
                self.toolbar.stats.triangle_count = 3600; // 100 cubes × 36 triangles

                self.toolbar.ui(ui, rect);
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
    /// - Click: Select entity
    fn handle_input(
        &mut self,
        response: &egui::Response,
        ctx: &egui::Context,
        world: &mut World,
        entity_manager: &mut EntityManager,
        undo_stack: &mut crate::command::UndoStack, // Phase 2.1: Command integration
    ) -> Result<()> {
        use crate::gizmo::GizmoMode;
        
        // Update gizmo state with current mouse position
        if let Some(pos) = response.hover_pos() {
            let mouse_pos = glam::Vec2::new(pos.x, pos.y);
            self.gizmo_state.update_mouse(mouse_pos);
        }

        // Gizmo transform application (if active and dragging)
        if self.gizmo_state.is_active() && response.dragged_by(egui::PointerButton::Primary) {
            if let Some(selected_id) = self.selected_entity() {
                // Get entity's current pose
                if let Some(pose) = world.pose(selected_id) {
                    let mouse_delta = self.gizmo_state.mouse_delta();
                    
                    match self.gizmo_state.mode {
                        GizmoMode::Translate { constraint: _ } => {
                            // Read CURRENT constraint (not captured at match time!)
                            let constraint = match self.gizmo_state.mode {
                                GizmoMode::Translate { constraint: c } => c,
                                _ => crate::gizmo::AxisConstraint::None,
                            };
                            
                            if constraint == crate::gizmo::AxisConstraint::None {
                                // FREE MOVEMENT: Entity follows mouse pointer on ground plane
                                // Get current mouse position in screen space
                                if let Some(mouse_pos_abs) = response.hover_pos() {
                                    let viewport_size = response.rect.size();
                                    // Convert absolute screen position to viewport-relative (0,0 = top-left of viewport)
                                    let mouse_pos = egui::Pos2 {
                                        x: mouse_pos_abs.x - response.rect.min.x,
                                        y: mouse_pos_abs.y - response.rect.min.y,
                                    };
                                    
                                    // Cast ray from mouse through camera
                                    let ray = self.camera.ray_from_screen(mouse_pos, viewport_size);
                                    
                                    // Intersect ray with ground plane (Y=0)
                                    let plane_normal = glam::Vec3::Y;
                                    let plane_point = glam::Vec3::ZERO;
                                    let denom = ray.direction.dot(plane_normal);
                                    
                                    if denom.abs() > 0.0001 {
                                        let t = (plane_point - ray.origin).dot(plane_normal) / denom;
                                        if t >= 0.0 {
                                            // Ground plane intersection point
                                            let world_pos = ray.origin + ray.direction * t;
                                            
                                            // Check if Ctrl is held for grid snapping
                                            let snap_enabled = ctx.input(|i| i.modifiers.ctrl || i.modifiers.command);
                                            
                                            // Apply grid snapping if enabled
                                            let final_x = if snap_enabled {
                                                self.snap_to_grid(world_pos.x)
                                            } else {
                                                world_pos.x
                                            };
                                            let final_z = if snap_enabled {
                                                self.snap_to_grid(world_pos.z)
                                            } else {
                                                world_pos.z
                                            };
                                            
                                            // Set entity position directly (no delta, just follow mouse)
                                            let new_x = final_x.round() as i32;
                                            let new_z = final_z.round() as i32;
                                            
                                            if let Some(pose_mut) = world.pose_mut(selected_id) {
                                                pose_mut.pos.x = new_x;
                                                pose_mut.pos.y = new_z; // IVec2.y = world Z
                                                
                                                println!(
                                                    "🔧 Translate (FREE{}): entity={}, mouse_abs=({:.1}, {:.1}), mouse_rel=({:.1}, {:.1}), world=({:.2}, {:.2}), new_pos=({}, {})",
                                                    if snap_enabled { " + SNAP" } else { "" },
                                                    selected_id, mouse_pos_abs.x, mouse_pos_abs.y,
                                                    mouse_pos.x, mouse_pos.y,
                                                    world_pos.x, world_pos.z, new_x, new_z
                                                );
                                            }
                                        }
                                    }
                                }
                            } else {
                                // CONSTRAINED MOVEMENT: Raycast to ground plane, then project onto constraint axis
                                if let Some(mouse_pos_abs) = response.hover_pos() {
                                    let viewport_size = response.rect.size();
                                    // Convert absolute screen position to viewport-relative
                                    let mouse_pos = egui::Pos2 {
                                        x: mouse_pos_abs.x - response.rect.min.x,
                                        y: mouse_pos_abs.y - response.rect.min.y,
                                    };
                                    
                                    // Get start position from snapshot (for the locked axis)
                                    let start_pos = if let Some(snapshot) = &self.gizmo_state.start_transform {
                                        (snapshot.position.x, snapshot.position.z)
                                    } else {
                                        (pose.pos.x as f32, pose.pos.y as f32)
                                    };
                                    
                                    // Cast ray from mouse through camera
                                    let ray = self.camera.ray_from_screen(mouse_pos, viewport_size);
                                    
                                    // Intersect ray with ground plane (Y=0)
                                    let plane_normal = glam::Vec3::Y;
                                    let plane_point = glam::Vec3::ZERO;
                                    let denom = ray.direction.dot(plane_normal);
                                    
                                    if denom.abs() > 0.0001 {
                                        let t = (plane_point - ray.origin).dot(plane_normal) / denom;
                                        if t >= 0.0 {
                                            // Ground plane intersection point
                                            let world_pos = ray.origin + ray.direction * t;
                                            
                                            // Check if Ctrl is held for grid snapping
                                            let snap_enabled = ctx.input(|i| i.modifiers.ctrl || i.modifiers.command);
                                            
                                            // Apply snapping to world position before constraints
                                            let snapped_x = if snap_enabled {
                                                self.snap_to_grid(world_pos.x)
                                            } else {
                                                world_pos.x
                                            };
                                            let snapped_z = if snap_enabled {
                                                self.snap_to_grid(world_pos.z)
                                            } else {
                                                world_pos.z
                                            };
                                            
                                            // Project onto constraint axis (lock one component to start position)
                                            let (new_x, new_z) = match constraint {
                                                crate::gizmo::AxisConstraint::X => {
                                                    // X-axis only: follow mouse X, lock Z to start
                                                    (snapped_x.round() as i32, start_pos.1 as i32)
                                                }
                                                crate::gizmo::AxisConstraint::Z => {
                                                    // Z-axis only: lock X to start, follow mouse Z
                                                    (start_pos.0 as i32, snapped_z.round() as i32)
                                                }
                                                crate::gizmo::AxisConstraint::Y => {
                                                    // Y-axis constrained (ground plane - no movement)
                                                    (start_pos.0 as i32, start_pos.1 as i32)
                                                }
                                                _ => {
                                                    // Planar constraints: use both axes
                                                    (snapped_x.round() as i32, snapped_z.round() as i32)
                                                }
                                            };
                                            
                                            if let Some(pose_mut) = world.pose_mut(selected_id) {
                                                pose_mut.pos.x = new_x;
                                                pose_mut.pos.y = new_z; // IVec2.y = world Z
                                                
                                                println!(
                                                    "🔧 Translate (CONSTRAINED{}): entity={}, constraint={:?}, start=({:.1}, {:.1}), world=({:.2}, {:.2}), new=({}, {})",
                                                    if snap_enabled { " + SNAP" } else { "" },
                                                    selected_id, constraint, start_pos.0, start_pos.1,
                                                    world_pos.x, world_pos.z, new_x, new_z
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        GizmoMode::Rotate { constraint: _ } => {
                            // IMPORTANT: We need to remember the start rotation when drag begins
                            // For now, we'll store it in the TransformSnapshot rotation field
                            
                            // Try to get start rotation from snapshot (stored as Quat)
                            let (start_x, start_y, start_z) = if let Some(snapshot) = &self.gizmo_state.start_transform {
                                // Extract XYZ rotations from quaternion
                                snapshot.rotation.to_euler(glam::EulerRot::XYZ)
                            } else {
                                // Fallback: capture current as start
                                (pose.rotation_x, pose.rotation, pose.rotation_z)
                            };
                            
                            // Calculate rotation angle from TOTAL mouse movement since drag started
                            let rotation_sensitivity = 0.005; // 200px = 1 radian (57.3°)
                            
                            // CRITICAL FIX: Read CURRENT constraint (not captured at match time!)
                            // This allows mid-drag constraint changes via X/Y/Z keys
                            let constraint = match self.gizmo_state.mode {
                                GizmoMode::Rotate { constraint: c } => c,
                                _ => crate::gizmo::AxisConstraint::None,
                            };
                            
                            let (rotation_angle, target_axis) = match constraint {
                                crate::gizmo::AxisConstraint::None => {
                                    // No explicit constraint - default to Y-axis (yaw) but don't highlight
                                    (mouse_delta.x * rotation_sensitivity, "Y")
                                }
                                crate::gizmo::AxisConstraint::Y => {
                                    // Y-axis explicitly selected: horizontal mouse movement
                                    (mouse_delta.x * rotation_sensitivity, "Y")
                                }
                                crate::gizmo::AxisConstraint::X => {
                                    // X-axis (pitch): vertical mouse movement (inverted for intuitive control)
                                    (-mouse_delta.y * rotation_sensitivity, "X")
                                }
                                crate::gizmo::AxisConstraint::Z => {
                                    // Z-axis (roll): vertical mouse movement (same direction as X but different axis)
                                    (mouse_delta.y * rotation_sensitivity, "Z")
                                }
                                _ => (0.0, "None"), // No rotation for planar constraints
                            };
                            
                            // Check if Ctrl is held for angle snapping
                            let snap_enabled = ctx.input(|i| i.modifiers.ctrl || i.modifiers.command);
                            
                            // Apply angle snapping if enabled (snap the delta, not the total)
                            let final_angle = if snap_enabled {
                                self.snap_angle(rotation_angle)
                            } else {
                                rotation_angle
                            };
                            
                            // Set rotation to START + TOTAL_ANGLE (not accumulate frame by frame!)
                            if let Some(pose_mut) = world.pose_mut(selected_id) {
                                match target_axis {
                                    "X" => pose_mut.rotation_x = start_x + final_angle,
                                    "Y" => pose_mut.rotation = start_y + final_angle,
                                    "Z" => pose_mut.rotation_z = start_z + final_angle,
                                    _ => {}
                                }
                                
                                println!(
                                    "🔧 Rotate{}: entity={}, axis={}, start={:.1}°, mouse_delta=({:.1}, {:.1}), angle={:.1}°, new={:.1}°",
                                    if snap_enabled { " + SNAP" } else { "" },
                                    selected_id, target_axis, 
                                    match target_axis {
                                        "X" => start_x.to_degrees(),
                                        "Y" => start_y.to_degrees(),
                                        "Z" => start_z.to_degrees(),
                                        _ => 0.0,
                                    },
                                    mouse_delta.x, mouse_delta.y,
                                    final_angle.to_degrees(),
                                    match target_axis {
                                        "X" => pose_mut.rotation_x.to_degrees(),
                                        "Y" => pose_mut.rotation.to_degrees(),
                                        "Z" => pose_mut.rotation_z.to_degrees(),
                                        _ => 0.0,
                                    }
                                );
                            }
                        }
                        GizmoMode::Scale { constraint: _, uniform: _ } => {
                            // SCALE MODE: Uses scroll wheel (handled above), not mouse drag
                            // No-op here - scaling happens via scroll wheel in the zoom section
                        }
                        GizmoMode::Inactive => {}
                    }
                }
            }
        }

        // Camera controls (middle mouse and scroll work even during gizmo mode)
        // Only left-drag is captured by gizmo
        let can_control_camera = response.hovered() || self.has_focus;

        // Orbit camera (left mouse drag) - DISABLED during gizmo operation
        if can_control_camera 
            && response.dragged_by(egui::PointerButton::Primary) 
            && !self.gizmo_state.is_active() // Don't orbit while gizmo active
        {
            let delta = response.drag_delta();
            println!(
                "🔄 Orbit: delta=({:.2}, {:.2}), yaw={:.2}, pitch={:.2}",
                delta.x,
                delta.y,
                self.camera.yaw(),
                self.camera.pitch()
            );
            self.camera.orbit(delta.x, delta.y);
        }

        // Pan camera (middle mouse drag)
        if can_control_camera && response.dragged_by(egui::PointerButton::Middle) {
            let delta = response.drag_delta();
            println!(
                "📐 Pan: delta=({:.2}, {:.2}), focal={:?}",
                delta.x,
                delta.y,
                self.camera.target()
            );
            self.camera.pan(delta.x, delta.y);
        }

        // Zoom camera OR scale entity (scroll wheel) - only when hovered over viewport
        if response.hovered() {
            ctx.input(|i| {
                // Use raw scroll delta
                let scroll = i.smooth_scroll_delta.y;
                if scroll.abs() > 0.1 {
                    // Check if we're in Scale mode
                    if matches!(self.gizmo_state.mode, GizmoMode::Scale { .. }) {
                        // SCALE MODE: Adjust entity scale with scroll wheel
                        if let Some(selected_id) = self.selected_entity() {
                            if let Some(pose_mut) = world.pose_mut(selected_id) {
                                // Scale by 1% per scroll tick (very smooth, gradual scaling)
                                let scale_delta = 1.0 + (scroll * 0.01);
                                let new_scale = (pose_mut.scale * scale_delta).max(0.1).min(10.0);
                                pose_mut.scale = new_scale;
                                
                                println!(
                                    "🔧 Scale (scroll): entity={}, delta={:.3}x, new_scale={:.3}x",
                                    selected_id, scale_delta, new_scale
                                );
                            }
                        }
                    } else {
                        // CAMERA ZOOM: Normal camera control
                        // Clamp to ±1.0 for very smooth zoom
                        let clamped_scroll = scroll.clamp(-1.0, 1.0);
                        self.camera.zoom(clamped_scroll);
                    }
                }
            });
        }
        
        // Sync selected entity to gizmo state
        self.gizmo_state.selected_entity = self.selected_entity();
        
        // Clear gizmo state if entity deselected
        if self.selected_entity().is_none() && self.gizmo_state.is_active() {
            self.gizmo_state.mode = GizmoMode::Inactive;
            self.gizmo_state.start_transform = None;
        }
        
        // Capture start transform when beginning a new operation
        if self.gizmo_state.is_active() && self.gizmo_state.start_transform.is_none() {
            if let Some(selected_id) = self.selected_entity() {
                // Try to capture from World entity first (for actual transforms)
                if let Some(pose) = world.pose(selected_id) {
                    let x = pose.pos.x as f32;
                    let z = pose.pos.y as f32;
                    // Create quaternion from XYZ Euler angles
                    let rotation_quat = glam::Quat::from_euler(
                        glam::EulerRot::XYZ,
                        pose.rotation_x,
                        pose.rotation,
                        pose.rotation_z
                    );
                    self.gizmo_state.start_transform = Some(TransformSnapshot {
                        position: glam::Vec3::new(x, 1.0, z),
                        rotation: rotation_quat, // Store all 3 rotation axes
                        scale: glam::Vec3::splat(pose.scale),
                    });
                    println!("📸 Captured World start transform: pos=({}, {}), rot=({:.1}°, {:.1}°, {:.1}°), scale={:.2}", 
                        x, z, pose.rotation_x.to_degrees(), pose.rotation.to_degrees(), pose.rotation_z.to_degrees(), pose.scale);
                } else if let Some(entity) = entity_manager.get(selected_id as u64) {
                    // Fallback to EntityManager
                    self.gizmo_state.start_transform = Some(TransformSnapshot {
                        position: entity.position,
                        rotation: entity.rotation,
                        scale: entity.scale,
                    });
                    println!("📸 Captured EntityManager start transform: {:?}", entity.position);
                }
            }
        }

        // Gizmo hotkeys (G/R/S for translate/rotate/scale, X/Y/Z for axis constraints, Enter/Escape)
        ctx.input(|i| {
            use winit::keyboard::KeyCode;

            // Handle gizmo mode keys first
            if i.key_pressed(egui::Key::G) {
                self.gizmo_state.handle_key(KeyCode::KeyG);
                println!("🔧 Gizmo mode: Translate (G)");
            }
            if i.key_pressed(egui::Key::R) {
                self.gizmo_state.handle_key(KeyCode::KeyR);
                println!("🔧 Gizmo mode: Rotate (R)");
            }
            if i.key_pressed(egui::Key::S) {
                // Check if already in scale mode (to toggle off)
                let was_scaling = matches!(self.gizmo_state.mode, GizmoMode::Scale { .. });
                self.gizmo_state.handle_key(KeyCode::KeyS);
                if was_scaling {
                    println!("🔧 Scale mode: OFF (camera control restored)");
                } else {
                    println!("🔧 Scale mode: ON (use scroll wheel to scale, S to exit)");
                }
            }

            // Axis constraints (X/Y/Z)
            if i.key_pressed(egui::Key::X) {
                self.gizmo_state.handle_key(KeyCode::KeyX);
                println!("🔧 Axis constraint: X");
            }
            if i.key_pressed(egui::Key::Y) {
                self.gizmo_state.handle_key(KeyCode::KeyY);
                println!("🔧 Axis constraint: Y");
            }
            if i.key_pressed(egui::Key::Z) {
                self.gizmo_state.handle_key(KeyCode::KeyZ);
                println!("🔧 Axis constraint: Z");
            }

            // Confirm/cancel gizmo operation
            if i.key_pressed(egui::Key::Enter) {
                self.gizmo_state.handle_key(KeyCode::Enter);
                println!("✅ Gizmo: Confirm");
            }
            if i.key_pressed(egui::Key::Escape) {
                self.gizmo_state.handle_key(KeyCode::Escape);
                println!("❌ Gizmo: Cancel");
            }

            // Undo/Redo (Ctrl+Z / Ctrl+Y or Ctrl+Shift+Z)
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::Z) {
                if i.modifiers.shift {
                    // Ctrl+Shift+Z: Redo
                    if let Err(e) = undo_stack.redo(world) {
                        eprintln!("❌ Redo failed: {}", e);
                    } else if let Some(desc) = undo_stack.redo_description() {
                        println!("⏭️  Redo: {}", desc);
                    }
                } else {
                    // Ctrl+Z: Undo
                    if let Err(e) = undo_stack.undo(world) {
                        eprintln!("❌ Undo failed: {}", e);
                    } else if let Some(desc) = undo_stack.undo_description() {
                        println!("⏮️  Undo: {}", desc);
                    }
                }
            }
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::Y) {
                // Ctrl+Y: Redo (alternative to Ctrl+Shift+Z)
                if let Err(e) = undo_stack.redo(world) {
                    eprintln!("❌ Redo failed: {}", e);
                } else if let Some(desc) = undo_stack.redo_description() {
                    println!("⏭️  Redo: {}", desc);
                }
            }

            // Copy/Paste/Duplicate/Delete (multi-selection support)
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::C) {
                // Ctrl+C: Copy selected entities
                if !self.selected_entities.is_empty() {
                    self.copy_selection(world);
                    println!("📋 Copied {} entities", self.selected_entities.len());
                }
            }
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::V) {
                // Ctrl+V: Paste entities
                self.paste_selection(world, undo_stack);
            }
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::D) {
                // Ctrl+D: Duplicate selected entities
                if !self.selected_entities.is_empty() {
                    self.duplicate_selection(world, undo_stack);
                    println!("📑 Duplicated {} entities", self.selected_entities.len());
                }
            }
            if i.key_pressed(egui::Key::Delete) {
                // Delete: Remove selected entities
                if !self.selected_entities.is_empty() {
                    self.delete_selection(world, undo_stack);
                    println!("🗑️  Deleted {} entities", self.selected_entities.len());
                }
            }
            // Select All
            if i.key_pressed(egui::Key::A) {
                println!("🔍 A key pressed! modifiers: ctrl={}, command={}, shift={}", 
                    i.modifiers.ctrl, i.modifiers.command, i.modifiers.shift);
                
                if i.modifiers.command || i.modifiers.ctrl {
                    // Ctrl+A: Select all entities
                    self.select_all(world);
                    println!("🎯 Selected all entities ({} total)", self.selected_entities.len());
                }
            }

            // Frame selected
            if i.key_pressed(egui::Key::F) {
                if let Some(selected_id) = self.selected_entity() {
                    // Frame World entity (match rendering position)
                    if let Some(pose) = world.pose(selected_id) {
                        let x = pose.pos.x as f32;
                        let z = pose.pos.y as f32;
                        let position = glam::Vec3::new(x, 1.0, z); // Y=1.0 (raised position)
                        let entity_radius = 0.866; // Half diagonal of 1x1x1 cube = sqrt(3)/2
                        
                        // Frame entity in camera view
                        self.camera.frame_entity(position, entity_radius);
                        
                        println!(
                            "🎯 Frame selected World entity {} at {:.2?}",
                            selected_id, position
                        );
                    } else {
                        println!("⚠️  Frame selected: Entity {} not found in World", selected_id);
                    }
                } else {
                    println!("⚠️  Frame selected: No entity selected");
                }
            }

            // Grid size controls: [ = decrease, ] = increase
            if i.key_pressed(egui::Key::OpenBracket) {
                // Cycle down: 2.0 → 1.0 → 0.5 → 0.25 → 2.0
                self.grid_snap_size = match self.grid_snap_size {
                    x if (x - 2.0).abs() < 0.01 => 1.0,
                    x if (x - 1.0).abs() < 0.01 => 0.5,
                    x if (x - 0.5).abs() < 0.01 => 0.25,
                    _ => 2.0,
                };
                println!("📐 Grid snap size: {:.2}m", self.grid_snap_size);
            }

            if i.key_pressed(egui::Key::CloseBracket) {
                // Cycle up: 0.25 → 0.5 → 1.0 → 2.0 → 0.25
                self.grid_snap_size = match self.grid_snap_size {
                    x if (x - 0.25).abs() < 0.01 => 0.5,
                    x if (x - 0.5).abs() < 0.01 => 1.0,
                    x if (x - 1.0).abs() < 0.01 => 2.0,
                    _ => 0.25,
                };
                println!("📐 Grid snap size: {:.2}m", self.grid_snap_size);
            }

            // Camera bookmarks: F1-F12 (restore), Shift+F1-F12 (save)
            let bookmark_keys = [
                egui::Key::F1, egui::Key::F2, egui::Key::F3, egui::Key::F4,
                egui::Key::F5, egui::Key::F6, egui::Key::F7, egui::Key::F8,
                egui::Key::F9, egui::Key::F10, egui::Key::F11, egui::Key::F12,
            ];

            for (slot, key) in bookmark_keys.iter().enumerate() {
                if i.key_pressed(*key) {
                    if i.modifiers.shift {
                        // SAVE bookmark
                        self.camera_bookmarks[slot] = Some(CameraBookmark {
                            focal_point: self.camera.focal_point(),
                            distance: self.camera.distance(),
                            yaw: self.camera.yaw(),
                            pitch: self.camera.pitch(),
                        });
                        println!("💾 Saved camera bookmark F{}", slot + 1);
                    } else if let Some(bookmark) = &self.camera_bookmarks[slot] {
                        // RESTORE bookmark
                        self.camera.set_focal_point(bookmark.focal_point);
                        self.camera.set_distance(bookmark.distance);
                        self.camera.set_yaw(bookmark.yaw);
                        self.camera.set_pitch(bookmark.pitch);
                        println!("📷 Restored camera bookmark F{}", slot + 1);
                    } else {
                        println!("⚠️  Camera bookmark F{} not set (use Shift+F{} to save)", slot + 1, slot + 1);
                    }
                }
            }
        });
        
        // Handle gizmo confirm/cancel
        if self.gizmo_state.confirmed {
            // Phase 2.1: Transform confirmed - create undo command
            if let Some(snapshot) = &self.gizmo_state.start_transform {
                if let Some(selected_id) = self.selected_entity() {
                    // Capture final state from World
                    if let Some(pose) = world.pose(selected_id) {
                        // Calculate old position from snapshot (stored as Vec3)
                        let old_pos = astraweave_core::IVec2 {
                            x: snapshot.position.x.round() as i32,
                            y: snapshot.position.z.round() as i32, // IVec2.y = world Z
                        };
                        let new_pos = pose.pos; // Already in IVec2 format

                        // Check if we're in move/rotate/scale mode
                        match &self.gizmo_state.mode {
                            GizmoMode::Translate { .. } => {
                                if old_pos != new_pos {
                                    let cmd = crate::command::MoveEntityCommand::new(
                                        selected_id,
                                        old_pos,
                                        new_pos,
                                    );
                                    undo_stack.push_executed(cmd);
                                    println!("📝 Recorded move: {:?} → {:?}", old_pos, new_pos);
                                }
                            }
                            GizmoMode::Rotate { .. } => {
                                let old_rot = snapshot.rotation.to_euler(glam::EulerRot::XYZ);
                                let new_rot = (pose.rotation_x, pose.rotation, pose.rotation_z);

                                let changed = (old_rot.0 - new_rot.0).abs() > 0.01
                                    || (old_rot.1 - new_rot.1).abs() > 0.01
                                    || (old_rot.2 - new_rot.2).abs() > 0.01;

                                if changed {
                                    let cmd = crate::command::RotateEntityCommand::new(
                                        selected_id,
                                        old_rot,
                                        new_rot,
                                    );
                                    undo_stack.push_executed(cmd);
                                    println!(
                                        "📝 Recorded rotation: ({:.1}°, {:.1}°, {:.1}°) → ({:.1}°, {:.1}°, {:.1}°)",
                                        old_rot.0.to_degrees(), old_rot.1.to_degrees(), old_rot.2.to_degrees(),
                                        new_rot.0.to_degrees(), new_rot.1.to_degrees(), new_rot.2.to_degrees()
                                    );
                                }
                            }
                            GizmoMode::Scale { .. } => {
                                let old_scale = snapshot.scale.x;
                                let new_scale = pose.scale;

                                if (old_scale - new_scale).abs() > 0.01 {
                                    let cmd = crate::command::ScaleEntityCommand::new(
                                        selected_id,
                                        old_scale,
                                        new_scale,
                                    );
                                    undo_stack.push_executed(cmd);
                                    println!("📝 Recorded scale: {:.2} → {:.2}", old_scale, new_scale);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            println!("✅ Transform confirmed");
            self.gizmo_state.confirmed = false;
        }
        
        if self.gizmo_state.cancelled {
            // Transform cancelled - revert to start_transform (NO undo command created)
            if let Some(snapshot) = &self.gizmo_state.start_transform {
                if let Some(selected_id) = self.selected_entity() {
                    if let Some(entity) = entity_manager.get_mut(selected_id as u64) {
                        entity.position = snapshot.position;
                        entity.rotation = snapshot.rotation;
                        entity.scale = snapshot.scale;
                        println!("❌ Transform cancelled - reverted to {:?}", snapshot.position);
                    }
                }
            }
            self.gizmo_state.cancelled = false;
        }

        // Selection (ray-casting entity picking)
        // Track mouse press/release manually since egui's clicked() doesn't work with drag detection
        let pointer_over_viewport = response.hovered() || response.rect.contains(
            ctx.input(|i| i.pointer.interact_pos()).unwrap_or(egui::Pos2::ZERO)
        );
        
        let mouse_pressed = ctx.input(|i| i.pointer.button_pressed(egui::PointerButton::Primary));
        let mouse_released = ctx.input(|i| i.pointer.button_released(egui::PointerButton::Primary));
        let current_pos = ctx.input(|i| i.pointer.interact_pos());
        
        // Track where mouse was pressed
        if mouse_pressed && pointer_over_viewport {
            self.mouse_pressed_pos = current_pos;
            println!("🖱️ Mouse pressed at: {:?}", current_pos);
        }
        
        // Check for click (press and release at same location without drag)
        let clicked = if mouse_released && self.mouse_pressed_pos.is_some() {
            let press_pos = self.mouse_pressed_pos.unwrap();
            let release_pos = current_pos.unwrap_or(press_pos);
            let drag_distance = (release_pos - press_pos).length();
            let is_click = drag_distance < 5.0; // 5 pixel threshold
            
            println!("🖱️ Mouse released: press={:?}, release={:?}, drag_dist={:.1}, is_click={}", 
                press_pos, release_pos, drag_distance, is_click);
            
            self.mouse_pressed_pos = None; // Clear press state
            is_click && pointer_over_viewport && !self.gizmo_state.is_active()
        } else {
            false
        };
        
        println!("🔍 Selection check: clicked={}, pointer_over={}, gizmo_active={}", 
            clicked, pointer_over_viewport, self.gizmo_state.is_active());
        
        if clicked {
            println!("✅ Click detected for selection!");
            
            if let Some(pos) = current_pos {
                let viewport_pos_vec = pos - response.rect.min;
                let viewport_pos = egui::Pos2::new(viewport_pos_vec.x, viewport_pos_vec.y);
                let ray = self
                    .camera
                    .ray_from_screen(viewport_pos, response.rect.size());

                // Pick World entities (which are actually rendered)
                let mut closest_entity: Option<(Entity, f32)> = None; // (entity_id, distance)

                // Check all World entities (matching entity_renderer logic)
                for entity_id in 1..100 {
                    let entity: Entity = entity_id;
                    
                    if let Some(pose) = world.pose(entity) {
                        // Match entity_renderer position calculation
                        let x = pose.pos.x as f32;
                        let z = pose.pos.y as f32;
                        let position = glam::Vec3::new(x, 1.0, z); // Y=1.0 (raised position)
                        
                        // Create AABB for 1x1x1 cube centered at position
                        let aabb_min = position - glam::Vec3::splat(0.5);
                        let aabb_max = position + glam::Vec3::splat(0.5);
                        
                        if let Some(distance) = Self::ray_intersects_aabb(
                            ray.origin,
                            ray.direction,
                            aabb_min,
                            aabb_max,
                        ) {
                            // Found intersection - keep closest
                            if closest_entity.is_none() || distance < closest_entity.unwrap().1 {
                                closest_entity = Some((entity, distance));
                            }
                        }
                    }
                }

                // Update selection based on modifier keys
                if let Some((entity_id, distance)) = closest_entity {
                    let modifiers = ctx.input(|i| i.modifiers);
                    
                    // Debug: Print modifier state
                    println!("🔍 Modifiers: ctrl={}, shift={}, alt={}, command={}", 
                        modifiers.ctrl, modifiers.shift, modifiers.alt, modifiers.command);
                    
                    if modifiers.ctrl || modifiers.command {
                        // Ctrl+Click: Toggle selection (multi-select)
                        println!("🎯 Before toggle: selected_entities = {:?}", self.selected_entities);
                        self.toggle_selection(entity_id);
                        println!(
                            "🎯 Toggled World entity {} (now {} entities selected): {:?}",
                            entity_id,
                            self.selected_entities.len(),
                            self.selected_entities
                        );
                    } else if modifiers.shift {
                        // Shift+Click: Add to selection
                        println!("🎯 Before add: selected_entities = {:?}", self.selected_entities);
                        self.add_to_selection(entity_id);
                        println!(
                            "🎯 Added World entity {} to selection ({} entities selected): {:?}",
                            entity_id,
                            self.selected_entities.len(),
                            self.selected_entities
                        );
                    } else {
                        // Regular click: Single select (clears others)
                        self.set_selected_entity(Some(entity_id));
                        println!(
                            "🎯 Selected World entity {} at distance {:.2}",
                            entity_id, distance
                        );
                    }
                } else {
                    // Clicked empty space - clear selection
                    self.clear_selection();
                    // Clear gizmo state when deselecting entity
                    if self.gizmo_state.is_active() {
                        self.gizmo_state.mode = GizmoMode::Inactive;
                        self.gizmo_state.start_transform = None;
                    }
                    println!("🎯 Click at ({:.1}, {:.1}) - No entity hit (selection cleared)", viewport_pos.x, viewport_pos.y);
                }
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

    /// Ray-AABB intersection test for entity picking
    ///
    /// Returns distance to intersection point if ray hits AABB, None otherwise.
    ///
    /// # Arguments
    ///
    /// * `ray_origin` - Ray starting point
    /// * `ray_dir` - Ray direction (normalized)
    /// * `aabb_min` - AABB minimum corner
    /// * `aabb_max` - AABB maximum corner
    fn ray_intersects_aabb(
        ray_origin: glam::Vec3,
        ray_dir: glam::Vec3,
        aabb_min: glam::Vec3,
        aabb_max: glam::Vec3,
    ) -> Option<f32> {
        // Slab method: test ray against each axis pair
        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;

        for i in 0..3 {
            let origin = ray_origin[i];
            let dir = ray_dir[i];
            let min = aabb_min[i];
            let max = aabb_max[i];

            if dir.abs() < 1e-6 {
                // Ray parallel to slab - check if origin is inside
                if origin < min || origin > max {
                    return None;
                }
            } else {
                // Calculate intersection distances
                let inv_dir = 1.0 / dir;
                let mut t1 = (min - origin) * inv_dir;
                let mut t2 = (max - origin) * inv_dir;

                if t1 > t2 {
                    std::mem::swap(&mut t1, &mut t2);
                }

                tmin = tmin.max(t1);
                tmax = tmax.min(t2);

                if tmin > tmax {
                    return None;
                }
            }
        }

        // Return closest intersection (positive distance only)
        if tmin >= 0.0 {
            Some(tmin)
        } else if tmax >= 0.0 {
            Some(tmax)
        } else {
            None
        }
    }

    /// Get the primary selected entity (for single-selection compatibility)
    pub fn selected_entity(&self) -> Option<Entity> {
        self.selected_entities.first().copied()
    }

    /// Get all selected entities
    pub fn selected_entities(&self) -> &[Entity] {
        &self.selected_entities
    }

    /// Set the selected entities (replaces current selection)
    pub fn set_selected_entities(&mut self, entities: Vec<Entity>) {
        self.selected_entities = entities;
    }

    /// Set a single selected entity (clears other selections)
    pub fn set_selected_entity(&mut self, entity: Option<Entity>) {
        self.selected_entities.clear();
        if let Some(e) = entity {
            self.selected_entities.push(e);
        }
    }

    /// Add an entity to the selection (for multi-select)
    pub fn add_to_selection(&mut self, entity: Entity) {
        if !self.selected_entities.contains(&entity) {
            self.selected_entities.push(entity);
        }
    }

    /// Remove an entity from the selection
    pub fn remove_from_selection(&mut self, entity: Entity) {
        self.selected_entities.retain(|&e| e != entity);
    }

    /// Toggle entity selection
    pub fn toggle_selection(&mut self, entity: Entity) {
        if self.selected_entities.contains(&entity) {
            self.remove_from_selection(entity);
        } else {
            self.add_to_selection(entity);
        }
    }

    /// Clear all selections
    pub fn clear_selection(&mut self) {
        self.selected_entities.clear();
    }

    /// Check if an entity is selected
    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected_entities.contains(&entity)
    }

    /// Copy selected entities to clipboard
    fn copy_selection(&mut self, _world: &World) {
        // TODO: Implement clipboard storage
        // For now, we'll store copied entities in a Vec<EntitySnapshot>
        // This will be expanded in Phase 2.2 with full serialization
        println!("📋 Copy: {} entities (clipboard not yet implemented)", self.selected_entities.len());
    }

    /// Paste entities from clipboard
    fn paste_selection(&mut self, _world: &mut World, _undo_stack: &mut crate::command::UndoStack) {
        // TODO: Implement paste from clipboard
        // Create new entities with offset position
        println!("📋 Paste: clipboard not yet implemented");
    }

    /// Duplicate selected entities (creates copies at offset position)
    fn duplicate_selection(&mut self, world: &mut World, _undo_stack: &mut crate::command::UndoStack) {
        if self.selected_entities.is_empty() {
            println!("⚠️  duplicate_selection: No entities selected");
            return;
        }

        println!("🔍 duplicate_selection: Starting duplication of {} entities: {:?}", 
            self.selected_entities.len(), 
            self.selected_entities);

        let mut new_entities = Vec::new();
        
        // Duplicate each selected entity
        for &entity_id in &self.selected_entities {
            println!("  🔍 Processing entity {}", entity_id);
            
            if let Some(pose) = world.pose(entity_id) {
                // Create new entity at offset position (2 units right)
                let new_pos = astraweave_core::IVec2 {
                    x: pose.pos.x + 2,
                    y: pose.pos.y,
                };
                
                // Get original entity's properties
                let health = world.health(entity_id);
                let team = world.team(entity_id);
                let ammo = world.ammo(entity_id);
                let name = world.name(entity_id).unwrap_or("Entity");

                println!("    📋 Entity {} properties: name={}, team={:?}, health={:?}, ammo={:?}", 
                    entity_id, name, team, health, ammo);

                // Create new entity using spawn
                let new_id = world.spawn(
                    &format!("{}_copy", name),
                    new_pos,
                    team.unwrap_or(Team { id: 0 }),
                    health.map(|h| h.hp).unwrap_or(100),
                    ammo.map(|a| a.rounds).unwrap_or(0),
                );
                
                // Copy transform properties
                if let Some(new_pose) = world.pose_mut(new_id) {
                    new_pose.rotation = pose.rotation;
                    new_pose.rotation_x = pose.rotation_x;
                    new_pose.rotation_z = pose.rotation_z;
                    new_pose.scale = pose.scale;
                }
                
                new_entities.push(new_id);
                println!("    ✅ Duplicated entity {} → {} at {:?}", entity_id, new_id, new_pos);
            } else {
                println!("    ⚠️  Entity {} has no pose, skipping", entity_id);
            }
        }

        // Select the duplicated entities
        if !new_entities.is_empty() {
            self.selected_entities = new_entities;
            println!("🎯 duplicate_selection: Complete. New selection: {:?}", self.selected_entities);
        } else {
            println!("⚠️  duplicate_selection: No entities were duplicated!");
        }

        // TODO: Add DuplicateCommand to undo stack (Phase 2.1 extension)
    }

    /// Delete selected entities
    fn delete_selection(&mut self, _world: &mut World, _undo_stack: &mut crate::command::UndoStack) {
        if self.selected_entities.is_empty() {
            return;
        }

        // TODO: World doesn't have destroy_entity yet - this is a placeholder
        // For now, just log what would be deleted
        for &entity_id in &self.selected_entities {
            println!("🗑️  Would delete entity {}", entity_id);
        }

        // Clear selection after "deletion"
        self.clear_selection();

        // Clear gizmo state
        if self.gizmo_state.is_active() {
            self.gizmo_state.mode = GizmoMode::Inactive;
            self.gizmo_state.start_transform = None;
        }

        // TODO: Add DeleteCommand to undo stack with entity snapshots (Phase 2.1 extension)
        // TODO: Implement actual entity removal in World API
    }

    /// Select all entities in the world
    fn select_all(&mut self, world: &World) {
        self.selected_entities.clear();
        
        println!("🔍 select_all: Starting scan for entities...");
        
        // Iterate through all entities (World doesn't expose entity list, so we try a range)
        // This is a workaround - ideally World would have an entities() iterator
        for entity_id in 0..1000 {
            if world.pose(entity_id).is_some() {
                self.selected_entities.push(entity_id);
                println!("  ✅ Found entity {}", entity_id);
            }
        }
        
        println!("🎯 select_all: Selected {} entities total: {:?}", 
            self.selected_entities.len(), 
            self.selected_entities);
    }

    /// Snap a float value to the grid
    fn snap_to_grid(&self, value: f32) -> f32 {
        if self.grid_snap_size > 0.0 {
            (value / self.grid_snap_size).round() * self.grid_snap_size
        } else {
            value
        }
    }

    /// Snap an angle to the nearest increment
    fn snap_angle(&self, angle: f32) -> f32 {
        if self.angle_snap_increment > 0.0 {
            (angle / self.angle_snap_increment).round() * self.angle_snap_increment
        } else {
            angle
        }
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
