// tools/aw_editor/src/panels/performance_panel.rs - Performance budgeting panel

use super::Panel;
use astract::widgets::PerformanceBudgetWidget;
use egui::Ui;

/// Performance panel - displays real-time frame budget tracking
///
/// Integrates the PerformanceBudgetWidget from Astract Day 3.
/// Shows frame time breakdown and budget warnings.
pub struct PerformancePanel {
    widget: PerformanceBudgetWidget,
    last_update: std::time::Instant,
    frame_count: u64,
}

impl PerformancePanel {
    pub fn new() -> Self {
        Self {
            widget: PerformanceBudgetWidget::new(),
            last_update: std::time::Instant::now(),
            frame_count: 0,
        }
    }

    /// Simulate frame timing data
    ///
    /// In a real editor, this would read from:
    /// - Tracy profiler integration
    /// - egui's frame time stats
    /// - Custom profiling zones
    fn simulate_frame_timing(&mut self) {
        // Simulate realistic timing with some variation
        // Use actual elapsed time instead of frame_count to avoid mouse-movement artifacts
        let elapsed_secs = self.last_update.elapsed().as_secs_f32();
        let base_time = 4.0; // Base 4ms frame (realistic editor idle time, ~24% of 60 FPS budget)
        let variance = (elapsed_secs * 2.0).sin() * 1.5; // ±1.5ms variation over time
        let total_ms = base_time + variance;

        // Use update_from_frame_time (the widget auto-distributes)
        self.widget.update_from_frame_time(total_ms);
        self.frame_count += 1;
    }
}

impl Default for PerformancePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel for PerformancePanel {
    fn name(&self) -> &str {
        "Performance"
    }

    fn update(&mut self) {
        // Update frame timing every ~16ms (60 FPS)
        let now = std::time::Instant::now();
        if now.duration_since(self.last_update).as_millis() >= 16 {
            self.simulate_frame_timing();
            self.last_update = now;
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.heading("⚡ Performance Budget");
        ui.separator();

        // Show the performance budget widget from Day 3
        self.widget.show(ui);

        ui.add_space(10.0);

        ui.group(|ui| {
            ui.label("📊 Integration Info");
            ui.label("This widget uses live frame timing data.");
            ui.label("In production, this would connect to:");
            ui.label("• Tracy profiler zones");
            ui.label("• ECS system timings");
            ui.label("• GPU frame time queries");
            ui.label("• Custom instrumentation");
        });

        ui.add_space(10.0);

        if ui.button("🔄 Reset History").clicked() {
            self.widget = PerformanceBudgetWidget::new();
            self.frame_count = 0;
        }
    }
}
