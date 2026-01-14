//! Charts panel demonstrating all chart widgets with realistic game engine data.

use super::Panel;
use astract::charts::{
    AxisConfig, Bar, BarChart, BarChartMode, BarGroup, LineChart, PointCluster, PointShape,
    ScatterPlot,
};
use egui::{Color32, Ui};
use std::time::Instant;

/// Panel demonstrating chart widgets with game engine metrics.
pub struct ChartsPanel {
    start_time: Instant,
    frame_count: u64,
    frame_history: Vec<(f64, f64)>, // (time, frame_time_ms)
    entity_counts: Vec<BarGroup>,
    spatial_data: Vec<PointCluster>,
}

impl ChartsPanel {
    pub fn new() -> Self {
        let mut panel = Self {
            start_time: Instant::now(),
            frame_count: 0,
            frame_history: Vec::new(),
            entity_counts: Vec::new(),
            spatial_data: Vec::new(),
        };

        panel.initialize_data();
        panel
    }

    fn initialize_data(&mut self) {
        // Initialize entity count data (bar chart)
        self.entity_counts = vec![
            BarGroup {
                category: "Scene 1".to_string(),
                bars: vec![
                    Bar {
                        label: "Enemies".to_string(),
                        value: 45.0,
                        color: Color32::from_rgb(220, 80, 80),
                    },
                    Bar {
                        label: "Allies".to_string(),
                        value: 12.0,
                        color: Color32::from_rgb(80, 220, 80),
                    },
                    Bar {
                        label: "NPCs".to_string(),
                        value: 8.0,
                        color: Color32::from_rgb(80, 160, 220),
                    },
                ],
            },
            BarGroup {
                category: "Scene 2".to_string(),
                bars: vec![
                    Bar {
                        label: "Enemies".to_string(),
                        value: 32.0,
                        color: Color32::from_rgb(220, 80, 80),
                    },
                    Bar {
                        label: "Allies".to_string(),
                        value: 18.0,
                        color: Color32::from_rgb(80, 220, 80),
                    },
                    Bar {
                        label: "NPCs".to_string(),
                        value: 15.0,
                        color: Color32::from_rgb(80, 160, 220),
                    },
                ],
            },
            BarGroup {
                category: "Scene 3".to_string(),
                bars: vec![
                    Bar {
                        label: "Enemies".to_string(),
                        value: 28.0,
                        color: Color32::from_rgb(220, 80, 80),
                    },
                    Bar {
                        label: "Allies".to_string(),
                        value: 10.0,
                        color: Color32::from_rgb(80, 220, 80),
                    },
                    Bar {
                        label: "NPCs".to_string(),
                        value: 20.0,
                        color: Color32::from_rgb(80, 160, 220),
                    },
                ],
            },
            BarGroup {
                category: "Scene 4".to_string(),
                bars: vec![
                    Bar {
                        label: "Enemies".to_string(),
                        value: 50.0,
                        color: Color32::from_rgb(220, 80, 80),
                    },
                    Bar {
                        label: "Allies".to_string(),
                        value: 15.0,
                        color: Color32::from_rgb(80, 220, 80),
                    },
                    Bar {
                        label: "NPCs".to_string(),
                        value: 5.0,
                        color: Color32::from_rgb(80, 160, 220),
                    },
                ],
            },
        ];

        // Initialize spatial data (scatter plot)
        self.spatial_data = vec![
            PointCluster::new(
                "Enemies",
                vec![
                    (10.0, 15.0),
                    (12.0, 18.0),
                    (11.0, 16.0),
                    (13.0, 14.0),
                    (9.0, 17.0),
                    (14.0, 15.0),
                    (10.5, 19.0),
                    (11.5, 13.0),
                ],
                Color32::from_rgb(220, 80, 80),
            )
            .shape(PointShape::Triangle)
            .size(5.0),
            PointCluster::new(
                "Allies",
                vec![
                    (25.0, 25.0),
                    (26.0, 27.0),
                    (24.0, 26.0),
                    (27.0, 24.0),
                    (23.0, 28.0),
                    (28.0, 25.0),
                ],
                Color32::from_rgb(80, 220, 80),
            )
            .shape(PointShape::Circle)
            .size(5.0),
            PointCluster::new(
                "NPCs",
                vec![
                    (5.0, 8.0),
                    (6.0, 9.0),
                    (4.0, 7.0),
                    (7.0, 10.0),
                    (5.5, 6.5),
                    (6.5, 8.5),
                    (4.5, 9.5),
                ],
                Color32::from_rgb(80, 160, 220),
            )
            .shape(PointShape::Square)
            .size(4.0),
            PointCluster::new(
                "Pickups",
                vec![
                    (18.0, 5.0),
                    (19.0, 6.0),
                    (17.0, 4.0),
                    (20.0, 7.0),
                    (16.0, 5.5),
                    (21.0, 6.5),
                ],
                Color32::from_rgb(255, 200, 80),
            )
            .shape(PointShape::Diamond)
            .size(4.0),
        ];
    }

    fn simulate_frame_timing(&mut self) {
        let elapsed = self.start_time.elapsed().as_secs_f64();

        // Simulate realistic frame timing (12-18ms range)
        let base_time = 14.0;
        let variance = (elapsed * 2.0).sin() * 2.0; // ±2ms oscillation
        let noise = (self.frame_count as f64 * 0.1).sin() * 0.5; // ±0.5ms noise
        let spike = if self.frame_count % 120 == 0 {
            3.0
        } else {
            0.0
        }; // Occasional spike

        let frame_time = base_time + variance + noise + spike;

        // Keep last 100 frames
        self.frame_history.push((elapsed, frame_time));
        if self.frame_history.len() > 100 {
            self.frame_history.remove(0);
        }

        self.frame_count += 1;
    }
}

impl Panel for ChartsPanel {
    fn name(&self) -> &str {
        "Charts"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.heading("📊 Chart Widgets Demo");
        ui.separator();

        // Frame timing line chart
        ui.label("Frame Timing (Line Chart)");
        let mut line_chart = LineChart::new("Frame Times Over Time")
            .height(180.0)
            .line_width(2.0)
            .show_points(false)
            .x_axis(AxisConfig::new("Time (s)"))
            .y_axis(AxisConfig::new("Frame Time (ms)"));

        if !self.frame_history.is_empty() {
            line_chart.add_series(
                "Frame Time",
                self.frame_history.clone(),
                Color32::from_rgb(100, 180, 255),
            );

            if let (Some(first), Some(last)) =
                (self.frame_history.first(), self.frame_history.last())
            {
                let min_time = first.0;
                let max_time = last.0;
                line_chart.add_series(
                    "60 FPS Target",
                    vec![(min_time, 16.67), (max_time, 16.67)],
                    Color32::from_rgb(80, 220, 80),
                );
            }
        }

        line_chart.show(ui);

        ui.add_space(10.0);

        // Entity counts bar chart
        ui.label("Entity Distribution (Bar Chart)");

        ui.horizontal(|ui| {
            if ui.button("Grouped").clicked() {
                // Already grouped
            }
            if ui.button("Stacked").clicked() {
                // Switch to stacked (future feature)
            }
        });

        let mut bar_chart = BarChart::new("Entities Per Scene")
            .height(180.0)
            .mode(BarChartMode::Grouped)
            .bar_width_ratio(0.7)
            .show_values(true)
            .y_axis(AxisConfig::new("Count"));

        for group in &self.entity_counts {
            bar_chart.add_group(group.clone());
        }

        bar_chart.show(ui);

        ui.add_space(10.0);

        // Spatial distribution scatter plot
        ui.label("Entity Positions (Scatter Plot)");

        let mut scatter_plot = ScatterPlot::new("Entity Spatial Distribution")
            .height(200.0)
            .show_connecting_lines(false)
            .x_axis(AxisConfig::new("World X"))
            .y_axis(AxisConfig::new("World Z"));

        for cluster in &self.spatial_data {
            scatter_plot.add_cluster(cluster.clone());
        }

        scatter_plot.show(ui);

        ui.add_space(10.0);

        // Stats
        ui.group(|ui| {
            ui.label(format!("📈 Frame Count: {}", self.frame_count));
            ui.label(format!(
                "⏱️  Runtime: {:.1}s",
                self.start_time.elapsed().as_secs_f64()
            ));
            if let Some((_, last_frame)) = self.frame_history.last() {
                let fps = 1000.0 / last_frame;
                let color = if fps >= 60.0 {
                    Color32::GREEN
                } else {
                    Color32::YELLOW
                };
                ui.colored_label(color, format!("🎮 Current FPS: {:.1}", fps));
            }
        });
    }

    fn update(&mut self) {
        self.simulate_frame_timing();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charts_panel_creation() {
        let panel = ChartsPanel::new();
        assert_eq!(panel.name(), "Charts");
        assert_eq!(panel.entity_counts.len(), 4);
        assert_eq!(panel.spatial_data.len(), 4);
    }

    #[test]
    fn test_frame_timing_simulation() {
        let mut panel = ChartsPanel::new();
        assert_eq!(panel.frame_history.len(), 0);

        panel.simulate_frame_timing();
        assert_eq!(panel.frame_history.len(), 1);
        assert_eq!(panel.frame_count, 1);

        // Simulate 150 frames (should keep only last 100)
        for _ in 0..149 {
            panel.simulate_frame_timing();
        }
        assert_eq!(panel.frame_history.len(), 100);
        assert_eq!(panel.frame_count, 150);
    }
}
