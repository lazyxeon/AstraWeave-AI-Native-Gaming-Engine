//! Viewport Toolbar
//!
//! Floating toolbar overlay for viewport controls and settings.
//! Provides quick access to common viewport operations.
//!
//! # Features
//!
//! - Shading mode toggle (wireframe/lit/unlit)
//! - Grid visibility toggle with type selector (infinite/crosshair)
//! - Snap-to-grid toggle and settings
//! - Performance stats display
//! - Camera bookmarks


/// Grid display type for viewport
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridType {
    /// Infinite ground plane grid with distance fading
    #[default]
    Infinite,

    /// Crosshair-style grid (XZ axis lines only, no full grid)
    Crosshair,

    /// No grid (same as show_grid = false, but separate option)
    None,
}

impl GridType {
    /// Cycle to next grid type
    pub fn cycle(&self) -> Self {
        match self {
            GridType::Infinite => GridType::Crosshair,
            GridType::Crosshair => GridType::None,
            GridType::None => GridType::Infinite,
        }
    }

    /// Display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            GridType::Infinite => "Infinite",
            GridType::Crosshair => "Crosshair",
            GridType::None => "None",
        }
    }
}

/// Viewport toolbar widget
///
/// Displays as floating panel over the viewport with professional controls.
#[derive(Debug, Clone)]
pub struct ViewportToolbar {
    /// Current shading mode
    pub shading_mode: ShadingMode,

    /// Grid visibility
    pub show_grid: bool,

    /// Grid display type (infinite plane vs crosshair)
    pub grid_type: GridType,

    /// Snap to grid enabled
    pub snap_enabled: bool,

    /// Grid snap size (meters)
    pub snap_size: f32,

    /// Angle snap enabled
    pub angle_snap_enabled: bool,

    /// Angle snap increment (degrees)
    pub angle_snap_degrees: f32,

    /// Show performance stats
    pub show_stats: bool,

    /// Performance stats (updated by viewport)
    pub stats: PerformanceStats,
}

impl Default for ViewportToolbar {
    fn default() -> Self {
        Self {
            shading_mode: ShadingMode::Lit,
            show_grid: true,
            grid_type: GridType::Infinite,
            snap_enabled: false,
            snap_size: 1.0,
            angle_snap_enabled: true,
            angle_snap_degrees: 15.0,
            show_stats: true,
            stats: PerformanceStats::default(),
        }
    }
}

impl ViewportToolbar {
    /// Render toolbar UI
    ///
    /// Displays as floating panel at top-left of viewport.
    pub fn ui(&mut self, ui: &mut egui::Ui, viewport_rect: egui::Rect) {
        // Position at top-left of viewport
        let toolbar_pos = viewport_rect.left_top() + egui::vec2(10.0, 10.0);

        egui::Area::new(egui::Id::new("viewport_toolbar"))
            .fixed_pos(toolbar_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_premultiplied(30, 30, 35, 230))
                    .corner_radius(4.0)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut self.show_grid, "Grid");

                            // Grid type selector (only show when grid is visible)
                            if self.show_grid {
                                egui::ComboBox::from_id_salt("grid_type")
                                    .selected_text(self.grid_type.name())
                                    .width(80.0)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut self.grid_type,
                                            GridType::Infinite,
                                            "Infinite",
                                        );
                                        ui.selectable_value(
                                            &mut self.grid_type,
                                            GridType::Crosshair,
                                            "Crosshair",
                                        );
                                    });
                            }

                            ui.separator();

                            ui.checkbox(&mut self.snap_enabled, "Grid Snap");
                            if self.snap_enabled {
                                if ui
                                    .small_button("0.5")
                                    .on_hover_text("Grid size: 0.5 units")
                                    .clicked()
                                {
                                    self.snap_size = 0.5;
                                }
                                if ui
                                    .small_button("1.0")
                                    .on_hover_text("Grid size: 1.0 units")
                                    .clicked()
                                {
                                    self.snap_size = 1.0;
                                }
                                if ui
                                    .small_button("2.0")
                                    .on_hover_text("Grid size: 2.0 units")
                                    .clicked()
                                {
                                    self.snap_size = 2.0;
                                }
                                ui.add(
                                    egui::DragValue::new(&mut self.snap_size)
                                        .speed(0.1)
                                        .range(0.1..=10.0)
                                        .suffix("m"),
                                );
                            }

                            ui.separator();

                            ui.checkbox(&mut self.angle_snap_enabled, "Angle Snap");
                            if self.angle_snap_enabled {
                                if ui
                                    .small_button("15°")
                                    .on_hover_text("Angle snap: 15 degrees")
                                    .clicked()
                                {
                                    self.angle_snap_degrees = 15.0;
                                }
                                if ui
                                    .small_button("45°")
                                    .on_hover_text("Angle snap: 45 degrees")
                                    .clicked()
                                {
                                    self.angle_snap_degrees = 45.0;
                                }
                                if ui
                                    .small_button("90°")
                                    .on_hover_text("Angle snap: 90 degrees")
                                    .clicked()
                                {
                                    self.angle_snap_degrees = 90.0;
                                }
                            }

                            ui.separator();

                            ui.checkbox(&mut self.show_stats, "Stats");
                        });
                    });
            });

        // Performance stats panel (bottom-left)
        if self.show_stats {
            let stats_pos = viewport_rect.left_bottom() + egui::vec2(10.0, -95.0);

            egui::Area::new(egui::Id::new("viewport_stats"))
                .fixed_pos(stats_pos)
                .show(ui.ctx(), |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_premultiplied(20, 20, 25, 200))
                        .corner_radius(4.0)
                        .inner_margin(6.0)
                        .show(ui, |ui| {
                            ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 2.0);
                            ui.label(egui::RichText::new("Performance").strong());
                            ui.separator();
                            ui.label(format!("FPS: {:.1}", self.stats.fps));
                            ui.label(format!("Frame: {:.2}ms", self.stats.frame_time_ms));
                            ui.label(format!("Entities: {}", self.stats.entity_count));
                            ui.label(format!("Triangles: {}K", self.stats.triangle_count / 1000));
                            ui.label(format!("Memory: {:.1} MB", self.stats.memory_usage_mb));
                        });
                });
        }
    }
}

/// Shading mode for viewport rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadingMode {
    /// Full lighting with shadows
    Lit,

    /// No lighting, flat colors
    Unlit,

    /// Wireframe overlay
    Wireframe,
}

/// Performance statistics for viewport
#[derive(Debug, Clone, Default)]
pub struct PerformanceStats {
    /// Frames per second
    pub fps: f32,

    /// Frame time in milliseconds
    pub frame_time_ms: f32,

    /// Number of entities rendered
    pub entity_count: u32,

    /// Number of triangles rendered
    pub triangle_count: u32,

    /// Memory usage in megabytes
    pub memory_usage_mb: f32,
}
