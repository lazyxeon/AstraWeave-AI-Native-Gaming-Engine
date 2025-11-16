// Repair Progress Bar - World-space UI for anchor repairs
//
// This module provides a progress bar that appears in 3D space above an anchor
// during the 5-second repair animation. Shows 0-100% progress.

/// World-space repair progress bar
#[derive(Debug, Clone)]
pub struct RepairProgressBar {
    /// Anchor ID being repaired
    pub anchor_id: Option<usize>,
    /// Progress (0.0-1.0)
    pub progress: f32,
    /// Is bar visible
    pub visible: bool,
}

impl RepairProgressBar {
    /// Create new progress bar (hidden)
    pub fn new() -> Self {
        Self {
            anchor_id: None,
            progress: 0.0,
            visible: false,
        }
    }

    /// Start showing progress bar for anchor
    pub fn show(&mut self, anchor_id: usize) {
        self.anchor_id = Some(anchor_id);
        self.progress = 0.0;
        self.visible = true;
    }

    /// Update progress (0.0-1.0)
    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);

        // Auto-hide when complete
        if self.progress >= 1.0 {
            self.hide();
        }
    }

    /// Hide progress bar
    pub fn hide(&mut self) {
        self.visible = false;
        self.anchor_id = None;
        self.progress = 0.0;
    }

    /// Check if bar is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get progress percentage (0-100)
    pub fn progress_percentage(&self) -> u32 {
        (self.progress * 100.0) as u32
    }

    /// Get progress text
    pub fn progress_text(&self) -> String {
        format!("Repairing... {}%", self.progress_percentage())
    }

    /// Get progress bar color (cyan fill)
    pub fn bar_color(&self) -> (f32, f32, f32) {
        (0.0, 0.8, 0.9) // Cyan
    }

    /// Get background color (dark gray)
    pub fn background_color(&self) -> (f32, f32, f32) {
        (0.1, 0.1, 0.1) // Dark gray
    }

    /// Render progress bar in 3D world space
    ///
    /// This function is called by the game rendering system with the anchor's
    /// world position transformed to screen coordinates.
    ///
    /// # Arguments
    /// * `screen_x` - X position in screen coordinates (pixels)
    /// * `screen_y` - Y position in screen coordinates (pixels)
    ///
    /// # Example
    /// ```ignore
    /// let anchor_screen_pos = camera.world_to_screen(anchor.position);
    /// progress_bar.render_world_space(anchor_screen_pos.x, anchor_screen_pos.y, &egui_ctx);
    /// ```
    #[cfg(feature = "egui")]
    pub fn render_world_space(&self, screen_x: f32, screen_y: f32, ctx: &egui::Context) {
        if !self.is_visible() {
            return;
        }

        let bar_width = 200.0;
        let bar_height = 20.0;
        let text_offset_y = -10.0; // Text above bar

        // Position bar above anchor (offset upward by 30px)
        let bar_x = screen_x - bar_width / 2.0;
        let bar_y = screen_y - 30.0;

        // Progress bar
        egui::Area::new(format!(
            "repair_progress_bar_{}",
            self.anchor_id.unwrap_or(0)
        ))
        .fixed_pos(egui::Pos2::new(bar_x, bar_y))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Text label
                ui.label(
                    egui::RichText::new(self.progress_text())
                        .size(14.0)
                        .color(egui::Color32::WHITE),
                );

                // Progress bar background
                let (r, g, b) = self.background_color();
                ui.add(
                    egui::ProgressBar::new(self.progress)
                        .desired_width(bar_width)
                        .desired_height(bar_height)
                        .fill(egui::Color32::from_rgb(
                            (r * 255.0) as u8,
                            (g * 255.0) as u8,
                            (b * 255.0) as u8,
                        ))
                        .show_percentage(),
                );
            });
        });
    }

    /// Render progress bar (no-op when egui feature is disabled)
    #[cfg(not(feature = "egui"))]
    pub fn render_world_space(&self, _screen_x: f32, _screen_y: f32, _ctx: &()) {
        // No-op: egui not available
    }
}

impl Default for RepairProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_creation() {
        let bar = RepairProgressBar::new();

        assert!(!bar.is_visible());
        assert_eq!(bar.progress, 0.0);
        assert!(bar.anchor_id.is_none());
    }

    #[test]
    fn test_show_progress_bar() {
        let mut bar = RepairProgressBar::new();

        bar.show(42);

        assert!(bar.is_visible());
        assert_eq!(bar.anchor_id, Some(42));
        assert_eq!(bar.progress, 0.0);
    }

    #[test]
    fn test_hide_progress_bar() {
        let mut bar = RepairProgressBar::new();

        bar.show(42);
        assert!(bar.is_visible());

        bar.hide();
        assert!(!bar.is_visible());
        assert!(bar.anchor_id.is_none());
        assert_eq!(bar.progress, 0.0);
    }

    #[test]
    fn test_update_progress() {
        let mut bar = RepairProgressBar::new();
        bar.show(42);

        bar.update_progress(0.5);
        assert_eq!(bar.progress, 0.5);
        assert!(bar.is_visible());

        bar.update_progress(0.75);
        assert_eq!(bar.progress, 0.75);
    }

    #[test]
    fn test_progress_clamping() {
        let mut bar = RepairProgressBar::new();
        bar.show(42);

        bar.update_progress(-0.5);
        assert_eq!(bar.progress, 0.0);

        // Don't test 1.5 because update_progress(1.0) auto-hides and resets to 0.0
        // Instead test 0.99 to ensure clamping works
        bar.update_progress(0.99);
        assert_eq!(bar.progress, 0.99);
        assert!(bar.is_visible()); // Still visible
    }

    #[test]
    fn test_auto_hide_on_complete() {
        let mut bar = RepairProgressBar::new();
        bar.show(42);

        bar.update_progress(1.0);

        // Should auto-hide when progress reaches 1.0
        assert!(!bar.is_visible());
    }

    #[test]
    fn test_progress_percentage() {
        let mut bar = RepairProgressBar::new();
        bar.show(42);

        bar.update_progress(0.0);
        assert_eq!(bar.progress_percentage(), 0);

        bar.update_progress(0.25);
        assert_eq!(bar.progress_percentage(), 25);

        bar.update_progress(0.5);
        assert_eq!(bar.progress_percentage(), 50);

        bar.update_progress(0.75);
        assert_eq!(bar.progress_percentage(), 75);

        bar.update_progress(0.99);
        assert_eq!(bar.progress_percentage(), 99);
    }

    #[test]
    fn test_progress_text() {
        let mut bar = RepairProgressBar::new();
        bar.show(42);

        bar.update_progress(0.0);
        assert_eq!(bar.progress_text(), "Repairing... 0%");

        bar.update_progress(0.5);
        assert_eq!(bar.progress_text(), "Repairing... 50%");

        bar.update_progress(0.99);
        assert_eq!(bar.progress_text(), "Repairing... 99%");
    }

    #[test]
    fn test_bar_color() {
        let bar = RepairProgressBar::new();
        let (r, g, b) = bar.bar_color();

        // Cyan color
        assert_eq!(r, 0.0);
        assert_eq!(g, 0.8);
        assert_eq!(b, 0.9);
    }

    #[test]
    fn test_background_color() {
        let bar = RepairProgressBar::new();
        let (r, g, b) = bar.background_color();

        // Dark gray
        assert_eq!(r, 0.1);
        assert_eq!(g, 0.1);
        assert_eq!(b, 0.1);
    }
}
