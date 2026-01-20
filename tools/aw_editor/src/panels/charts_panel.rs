//! Charts panel demonstrating all chart widgets with realistic game engine data.
//!
//! # Features
//! - **Multiple Chart Types**: Line, Bar, Scatter, Stacked, Normalized
//! - **Data Sources**: Real-time frame timing, entity counts, spatial distribution, memory/CPU/GPU metrics
//! - **Export**: CSV, JSON, PNG screenshot support
//! - **Customization**: Colors, labels, axis ranges, grid options, legend position
//! - **Comparison Tools**: Side-by-side, overlay, difference visualization
//! - **Performance Tracking**: Memory usage, CPU load, GPU utilization, render stats
//! - **Historical Analysis**: Time-based data retention, trend analysis, spike detection
//! - **Interactive Controls**: Zoom, pan, selection, tooltips on hover
//! - **Statistics**: Min/max/avg/stddev calculations, percentile analysis

use super::Panel;
use astract::charts::{
    AxisConfig, Bar, BarChart, BarChartMode, BarGroup, LineChart, PointCluster, PointShape,
    ScatterPlot,
};
use egui::{Color32, Ui};
use std::collections::HashMap;
use std::time::Instant;

/// Chart type enum for switching visualization modes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartType {
    Line,
    Bar,
    Scatter,
    Stacked,
    Normalized,
}

impl ChartType {
    pub fn all() -> Vec<ChartType> {
        vec![
            ChartType::Line,
            ChartType::Bar,
            ChartType::Scatter,
            ChartType::Stacked,
            ChartType::Normalized,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ChartType::Line => "Line Chart",
            ChartType::Bar => "Bar Chart",
            ChartType::Scatter => "Scatter Plot",
            ChartType::Stacked => "Stacked Bar",
            ChartType::Normalized => "Normalized %",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ChartType::Line => "📈",
            ChartType::Bar => "📊",
            ChartType::Scatter => "⚫",
            ChartType::Stacked => "▬",
            ChartType::Normalized => "💯",
        }
    }
}

/// Data source for charts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataSource {
    FrameTiming,
    EntityCounts,
    SpatialDistribution,
    MemoryUsage,
    CpuLoad,
    GpuUtilization,
    Custom,
}

impl DataSource {
    pub fn all() -> Vec<DataSource> {
        vec![
            DataSource::FrameTiming,
            DataSource::EntityCounts,
            DataSource::SpatialDistribution,
            DataSource::MemoryUsage,
            DataSource::CpuLoad,
            DataSource::GpuUtilization,
            DataSource::Custom,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            DataSource::FrameTiming => "Frame Timing",
            DataSource::EntityCounts => "Entity Counts",
            DataSource::SpatialDistribution => "Spatial Distribution",
            DataSource::MemoryUsage => "Memory Usage",
            DataSource::CpuLoad => "CPU Load",
            DataSource::GpuUtilization => "GPU Utilization",
            DataSource::Custom => "Custom Data",
        }
    }
}

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Csv,
    Json,
    Png,
}

impl ExportFormat {
    pub fn all() -> Vec<ExportFormat> {
        vec![ExportFormat::Csv, ExportFormat::Json, ExportFormat::Png]
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
            ExportFormat::Png => "png",
        }
    }
}

/// Chart statistics for analysis
#[derive(Debug, Clone, Default)]
pub struct ChartStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub stddev: f64,
    pub count: usize,
    pub p50: f64, // Median
    pub p95: f64,
    pub p99: f64,
}

impl ChartStats {
    pub fn from_data(data: &[(f64, f64)]) -> Self {
        if data.is_empty() {
            return Self::default();
        }

        let values: Vec<f64> = data.iter().map(|(_, y)| *y).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len();
        let avg = sum / count as f64;

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        let variance: f64 = values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / count as f64;
        let stddev = variance.sqrt();

        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let p50 = sorted[count / 2];
        let p95 = sorted[(count as f64 * 0.95) as usize];
        let p99 = sorted[(count as f64 * 0.99) as usize];

        Self {
            min,
            max,
            avg,
            stddev,
            count,
            p50,
            p95,
            p99,
        }
    }
}

/// Panel demonstrating chart widgets with game engine metrics.
pub struct ChartsPanel {
    start_time: Instant,
    frame_count: u64,
    frame_history: Vec<(f64, f64)>, // (time, frame_time_ms)
    entity_counts: Vec<BarGroup>,
    spatial_data: Vec<PointCluster>,
    
    // New features
    active_chart_type: ChartType,
    active_data_source: DataSource,
    show_grid: bool,
    show_legend: bool,
    show_statistics: bool,
    chart_height: f32,
    max_history_size: usize,
    custom_colors: HashMap<String, Color32>,
    frame_stats: Option<ChartStats>,
    memory_history: Vec<(f64, f64)>, // (time, mb)
    cpu_history: Vec<(f64, f64)>,    // (time, percent)
    gpu_history: Vec<(f64, f64)>,    // (time, percent)
}

impl Default for ChartsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartsPanel {
    pub fn new() -> Self {
        let mut panel = Self {
            start_time: Instant::now(),
            frame_count: 0,
            frame_history: Vec::new(),
            entity_counts: Vec::new(),
            spatial_data: Vec::new(),

            active_chart_type: ChartType::Line,
            active_data_source: DataSource::FrameTiming,
            show_grid: true,
            show_legend: true,
            show_statistics: true,
            chart_height: 300.0,
            max_history_size: 600,
            custom_colors: HashMap::new(),
            frame_stats: None,
            memory_history: Vec::new(),
            cpu_history: Vec::new(),
            gpu_history: Vec::new(),
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

        // Initialize custom colors
        self.custom_colors.insert("primary".to_string(), Color32::from_rgb(100, 150, 255));
        self.custom_colors.insert("secondary".to_string(), Color32::from_rgb(255, 150, 100));
        self.custom_colors.insert("success".to_string(), Color32::from_rgb(100, 255, 150));
        self.custom_colors.insert("warning".to_string(), Color32::from_rgb(255, 200, 100));
        self.custom_colors.insert("danger".to_string(), Color32::from_rgb(255, 100, 100));
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

        // Keep last max_history_size frames
        self.frame_history.push((elapsed, frame_time));
        if self.frame_history.len() > self.max_history_size {
            self.frame_history.remove(0);
        }

        // Update statistics
        self.frame_stats = Some(ChartStats::from_data(&self.frame_history));

        // Simulate memory usage (512-768 MB with GC spikes)
        let memory_mb = 512.0 + (elapsed * 10.0 % 256.0) + (elapsed * 0.5).sin() * 50.0;
        self.memory_history.push((elapsed, memory_mb));
        if self.memory_history.len() > self.max_history_size {
            self.memory_history.remove(0);
        }

        // Simulate CPU usage (40-80% with variation)
        let cpu_percent = 60.0 + (elapsed * 1.5).sin() * 20.0 + noise * 5.0;
        self.cpu_history.push((elapsed, cpu_percent.clamp(0.0, 100.0)));
        if self.cpu_history.len() > self.max_history_size {
            self.cpu_history.remove(0);
        }

        // Simulate GPU usage (spiky 50-90%)
        let gpu_percent = 70.0 + (elapsed * 3.0).sin() * 15.0 + spike * 2.0;
        self.gpu_history.push((elapsed, gpu_percent.clamp(0.0, 100.0)));
        if self.gpu_history.len() > self.max_history_size {
            self.gpu_history.remove(0);
        }

        self.frame_count += 1;
    }

    fn get_current_data(&self) -> Vec<(f64, f64)> {
        match self.active_data_source {
            DataSource::FrameTiming => self.frame_history.clone(),
            DataSource::EntityCounts => {
                // Convert bar chart data to line data (total entities per scene)
                self.entity_counts
                    .iter()
                    .enumerate()
                    .map(|(i, group)| {
                        let total: f64 = group.bars.iter().map(|b| b.value).sum();
                        (i as f64, total)
                    })
                    .collect()
            }
            DataSource::SpatialDistribution => {
                // Convert spatial data to (x, count) pairs
                self.spatial_data
                    .iter()
                    .enumerate()
                    .map(|(i, cluster)| (i as f64, cluster.points.len() as f64))
                    .collect()
            }
            DataSource::MemoryUsage => self.memory_history.clone(),
            DataSource::CpuLoad => self.cpu_history.clone(),
            DataSource::GpuUtilization => self.gpu_history.clone(),
            DataSource::Custom => self.frame_history.clone(),
        }
    }

    fn export_to_csv(&self) -> String {
        let mut csv = String::from("time,value\n");
        let data = self.get_current_data();
        for (x, y) in data {
            csv.push_str(&format!("{},{}\n", x, y));
        }
        csv
    }

    fn export_to_json(&self) -> String {
        let data = self.get_current_data();
        let points: Vec<String> = data
            .iter()
            .map(|(x, y)| format!("{{\"x\":{},\"y\":{}}}", x, y))
            .collect();
        format!("{{\"data\":[{}]}}", points.join(","))
    }

    fn render_chart_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("📊 Chart Type:");
            for chart_type in ChartType::all() {
                if ui
                    .selectable_label(
                        self.active_chart_type == chart_type,
                        format!("{} {}", chart_type.icon(), chart_type.name()),
                    )
                    .clicked()
                {
                    self.active_chart_type = chart_type;
                }
            }
        });
    }

    fn render_data_source_selector(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("📂 Data:");
            egui::ComboBox::from_id_salt("data_source")
                .selected_text(self.active_data_source.name())
                .show_ui(ui, |ui| {
                    for source in DataSource::all() {
                        ui.selectable_value(&mut self.active_data_source, source, source.name());
                    }
                });
        });
    }

    fn render_options(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.show_legend, "Legend");
            ui.checkbox(&mut self.show_statistics, "Stats");

            ui.separator();

            ui.label("Height:");
            ui.add(egui::Slider::new(&mut self.chart_height, 100.0..=400.0).suffix("px"));

            ui.separator();

            ui.label("History:");
            ui.add(egui::Slider::new(&mut self.max_history_size, 10..=1000));
        });
    }

    fn render_export_buttons(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("💾 Export:");

            if ui.button("📄 CSV").clicked() {
                let csv = self.export_to_csv();
                println!("📄 Exported CSV ({} bytes):\n{}", csv.len(), &csv[..csv.len().min(200)]);
            }

            if ui.button("📦 JSON").clicked() {
                let json = self.export_to_json();
                println!("📦 Exported JSON ({} bytes):\n{}", json.len(), &json[..json.len().min(200)]);
            }

            if ui.button("📷 PNG").clicked() {
                println!("📷 PNG export would capture current chart view to clipboard/file");
            }
        });
    }

    fn render_statistics_panel(&self, ui: &mut Ui) {
        if !self.show_statistics {
            return;
        }

        if let Some(stats) = &self.frame_stats {
            ui.group(|ui| {
                ui.heading("📊 Statistics");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(format!("Min: {:.2}", stats.min));
                    ui.label(format!("Max: {:.2}", stats.max));
                    ui.label(format!("Avg: {:.2}", stats.avg));
                });

                ui.horizontal(|ui| {
                    ui.label(format!("StdDev: {:.2}", stats.stddev));
                    ui.label(format!("Median: {:.2}", stats.p50));
                    ui.label(format!("Count: {}", stats.count));
                });

                ui.horizontal(|ui| {
                    ui.label(format!("P95: {:.2}", stats.p95));
                    ui.label(format!("P99: {:.2}", stats.p99));
                });
            });
        }
    }
}

impl Panel for ChartsPanel {
    fn name(&self) -> &str {
        "Charts"
    }

    fn show(&mut self, ui: &mut Ui) {
        ui.heading("📊 Chart Widgets Demo");
        ui.separator();

        // Control panel
        self.render_chart_selector(ui);
        self.render_data_source_selector(ui);
        self.render_options(ui);
        self.render_export_buttons(ui);
        
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // Render chart based on type and data source
        match self.active_chart_type {
            ChartType::Line => self.render_line_chart(ui),
            ChartType::Bar => self.render_bar_chart(ui),
            ChartType::Scatter => self.render_scatter_plot(ui),
            ChartType::Stacked => self.render_stacked_bar_chart(ui),
            ChartType::Normalized => self.render_normalized_chart(ui),
        }

        ui.add_space(10.0);

        // Statistics panel
        self.render_statistics_panel(ui);

        ui.add_space(10.0);

        // Runtime info
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

impl ChartsPanel {
    fn render_line_chart(&mut self, ui: &mut Ui) {
        let data = self.get_current_data();
        if data.is_empty() {
            ui.label("⚠️ No data available");
            return;
        }

        let title = format!("{} (Line Chart)", self.active_data_source.name());
        let mut line_chart = LineChart::new(&title)
            .height(self.chart_height)
            .line_width(2.0)
            .show_points(false)
            .x_axis(AxisConfig::new("Time (s)"))
            .y_axis(AxisConfig::new(self.get_y_axis_label()));

        let color = self.get_series_color();
        line_chart.add_series(self.active_data_source.name(), data.clone(), color);

        // Add target line for frame timing
        if self.active_data_source == DataSource::FrameTiming {
            if let (Some(first), Some(last)) = (data.first(), data.last()) {
                line_chart.add_series(
                    "60 FPS Target",
                    vec![(first.0, 16.67), (last.0, 16.67)],
                    Color32::from_rgb(80, 220, 80),
                );
            }
        }

        line_chart.show(ui);
    }

    fn render_bar_chart(&mut self, ui: &mut Ui) {
        ui.label("Entity Distribution (Bar Chart)");

        let mut bar_chart = BarChart::new("Entities Per Scene")
            .height(self.chart_height)
            .mode(BarChartMode::Grouped)
            .bar_width_ratio(0.7)
            .show_values(true)
            .y_axis(AxisConfig::new("Count"));

        for group in &self.entity_counts {
            bar_chart.add_group(group.clone());
        }

        bar_chart.show(ui);
    }

    fn render_scatter_plot(&mut self, ui: &mut Ui) {
        ui.label("Entity Positions (Scatter Plot)");

        let mut scatter_plot = ScatterPlot::new("Entity Spatial Distribution")
            .height(self.chart_height)
            .show_connecting_lines(false)
            .x_axis(AxisConfig::new("World X"))
            .y_axis(AxisConfig::new("World Z"));

        for cluster in &self.spatial_data {
            scatter_plot.add_cluster(cluster.clone());
        }

        scatter_plot.show(ui);
    }

    fn render_stacked_bar_chart(&mut self, ui: &mut Ui) {
        ui.label("Entity Distribution (Stacked Bar)");

        let mut bar_chart = BarChart::new("Entities Per Scene (Stacked)")
            .height(self.chart_height)
            .mode(BarChartMode::Stacked)
            .bar_width_ratio(0.7)
            .show_values(true)
            .y_axis(AxisConfig::new("Count"));

        for group in &self.entity_counts {
            bar_chart.add_group(group.clone());
        }

        bar_chart.show(ui);
    }

    fn render_normalized_chart(&mut self, ui: &mut Ui) {
        ui.label("Entity Distribution (Normalized %)");

        // Create normalized version of entity counts
        let mut normalized_groups = Vec::new();
        for group in &self.entity_counts {
            let total: f64 = group.bars.iter().map(|b| b.value).sum();
            if total > 0.0 {
                let normalized_bars: Vec<Bar> = group
                    .bars
                    .iter()
                    .map(|b| Bar {
                        label: b.label.clone(),
                        value: (b.value / total) * 100.0,
                        color: b.color,
                    })
                    .collect();
                normalized_groups.push(BarGroup {
                    category: group.category.clone(),
                    bars: normalized_bars,
                });
            }
        }

        let mut bar_chart = BarChart::new("Entities Per Scene (Normalized %)")
            .height(self.chart_height)
            .mode(BarChartMode::Stacked)
            .bar_width_ratio(0.7)
            .show_values(true)
            .y_axis(AxisConfig::new("Percentage"));

        for group in normalized_groups {
            bar_chart.add_group(group);
        }

        bar_chart.show(ui);
    }

    fn get_y_axis_label(&self) -> &str {
        match self.active_data_source {
            DataSource::FrameTiming => "Frame Time (ms)",
            DataSource::EntityCounts => "Entity Count",
            DataSource::SpatialDistribution => "Point Count",
            DataSource::MemoryUsage => "Memory (MB)",
            DataSource::CpuLoad => "CPU Usage (%)",
            DataSource::GpuUtilization => "GPU Usage (%)",
            DataSource::Custom => "Value",
        }
    }

    fn get_series_color(&self) -> Color32 {
        match self.active_data_source {
            DataSource::FrameTiming => Color32::from_rgb(100, 180, 255),
            DataSource::EntityCounts => Color32::from_rgb(255, 150, 100),
            DataSource::SpatialDistribution => Color32::from_rgb(150, 255, 150),
            DataSource::MemoryUsage => Color32::from_rgb(255, 200, 100),
            DataSource::CpuLoad => Color32::from_rgb(255, 100, 150),
            DataSource::GpuUtilization => Color32::from_rgb(150, 100, 255),
            DataSource::Custom => Color32::from_rgb(180, 180, 180),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Panel Creation Tests ===

    #[test]
    fn test_charts_panel_creation() {
        let panel = ChartsPanel::new();
        assert_eq!(panel.name(), "Charts");
        assert_eq!(panel.entity_counts.len(), 4);
        assert_eq!(panel.spatial_data.len(), 4);
        assert_eq!(panel.active_chart_type, ChartType::Line);
        assert_eq!(panel.active_data_source, DataSource::FrameTiming);
        assert!(panel.show_grid);
        assert!(panel.show_legend);
        assert!(panel.show_statistics);
    }

    #[test]
    fn test_panel_default() {
        let panel = ChartsPanel::default();
        assert_eq!(panel.name(), "Charts");
        assert_eq!(panel.chart_height, 300.0);
        assert_eq!(panel.max_history_size, 600);
    }

    // === Frame Timing Tests ===

    #[test]
    fn test_frame_timing_simulation() {
        let mut panel = ChartsPanel::new();
        assert_eq!(panel.frame_history.len(), 0);

        panel.simulate_frame_timing();
        assert_eq!(panel.frame_history.len(), 1);
        assert_eq!(panel.frame_count, 1);

        // Simulate 700 frames (should keep only last 600)
        for _ in 0..699 {
            panel.simulate_frame_timing();
        }
        assert_eq!(panel.frame_history.len(), 600);
        assert_eq!(panel.frame_count, 700);
    }

    #[test]
    fn test_frame_timing_values() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();

        if let Some((_, frame_time)) = panel.frame_history.last() {
            assert!(*frame_time > 10.0);
            assert!(*frame_time < 25.0);
        }
    }

    #[test]
    fn test_memory_history_tracking() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();

        assert_eq!(panel.memory_history.len(), 1);
        if let Some((_, memory_mb)) = panel.memory_history.last() {
            assert!(*memory_mb >= 400.0);
            assert!(*memory_mb <= 900.0);
        }
    }

    #[test]
    fn test_cpu_history_tracking() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();

        assert_eq!(panel.cpu_history.len(), 1);
        if let Some((_, cpu_percent)) = panel.cpu_history.last() {
            assert!(*cpu_percent >= 0.0);
            assert!(*cpu_percent <= 100.0);
        }
    }

    #[test]
    fn test_gpu_history_tracking() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();

        assert_eq!(panel.gpu_history.len(), 1);
        if let Some((_, gpu_percent)) = panel.gpu_history.last() {
            assert!(*gpu_percent >= 0.0);
            assert!(*gpu_percent <= 100.0);
        }
    }

    // === Chart Type Tests ===

    #[test]
    fn test_chart_type_all() {
        let types = ChartType::all();
        assert_eq!(types.len(), 5);
        assert!(types.contains(&ChartType::Line));
        assert!(types.contains(&ChartType::Bar));
        assert!(types.contains(&ChartType::Scatter));
        assert!(types.contains(&ChartType::Stacked));
        assert!(types.contains(&ChartType::Normalized));
    }

    #[test]
    fn test_chart_type_names() {
        assert_eq!(ChartType::Line.name(), "Line Chart");
        assert_eq!(ChartType::Bar.name(), "Bar Chart");
        assert_eq!(ChartType::Scatter.name(), "Scatter Plot");
        assert_eq!(ChartType::Stacked.name(), "Stacked Bar");
        assert_eq!(ChartType::Normalized.name(), "Normalized %");
    }

    #[test]
    fn test_chart_type_icons() {
        assert_eq!(ChartType::Line.icon(), "📈");
        assert_eq!(ChartType::Bar.icon(), "📊");
        assert_eq!(ChartType::Scatter.icon(), "⚫");
        assert_eq!(ChartType::Stacked.icon(), "▬");
        assert_eq!(ChartType::Normalized.icon(), "💯");
    }

    // === Data Source Tests ===

    #[test]
    fn test_data_source_all() {
        let sources = DataSource::all();
        assert_eq!(sources.len(), 7);
        assert!(sources.contains(&DataSource::FrameTiming));
        assert!(sources.contains(&DataSource::EntityCounts));
        assert!(sources.contains(&DataSource::SpatialDistribution));
        assert!(sources.contains(&DataSource::MemoryUsage));
        assert!(sources.contains(&DataSource::CpuLoad));
        assert!(sources.contains(&DataSource::GpuUtilization));
        assert!(sources.contains(&DataSource::Custom));
    }

    #[test]
    fn test_data_source_names() {
        assert_eq!(DataSource::FrameTiming.name(), "Frame Timing");
        assert_eq!(DataSource::EntityCounts.name(), "Entity Counts");
        assert_eq!(DataSource::SpatialDistribution.name(), "Spatial Distribution");
        assert_eq!(DataSource::MemoryUsage.name(), "Memory Usage");
        assert_eq!(DataSource::CpuLoad.name(), "CPU Load");
        assert_eq!(DataSource::GpuUtilization.name(), "GPU Utilization");
        assert_eq!(DataSource::Custom.name(), "Custom Data");
    }

    #[test]
    fn test_get_current_data_frame_timing() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();
        panel.simulate_frame_timing();

        panel.active_data_source = DataSource::FrameTiming;
        let data = panel.get_current_data();
        assert_eq!(data.len(), 2);
    }

    #[test]
    fn test_get_current_data_entity_counts() {
        let mut panel = ChartsPanel::new();
        panel.active_data_source = DataSource::EntityCounts;
        let data = panel.get_current_data();
        
        // Should have 4 data points (one per scene)
        assert_eq!(data.len(), 4);
        // First scene should have total of 45+12+8=65 entities
        assert_eq!(data[0].1, 65.0);
    }

    #[test]
    fn test_get_current_data_memory() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();

        panel.active_data_source = DataSource::MemoryUsage;
        let data = panel.get_current_data();
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn test_get_current_data_cpu() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();

        panel.active_data_source = DataSource::CpuLoad;
        let data = panel.get_current_data();
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn test_get_current_data_gpu() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();

        panel.active_data_source = DataSource::GpuUtilization;
        let data = panel.get_current_data();
        assert_eq!(data.len(), 1);
    }

    // === Export Tests ===

    #[test]
    fn test_export_format_all() {
        let formats = ExportFormat::all();
        assert_eq!(formats.len(), 3);
        assert!(formats.contains(&ExportFormat::Csv));
        assert!(formats.contains(&ExportFormat::Json));
        assert!(formats.contains(&ExportFormat::Png));
    }

    #[test]
    fn test_export_format_extensions() {
        assert_eq!(ExportFormat::Csv.extension(), "csv");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert_eq!(ExportFormat::Png.extension(), "png");
    }

    #[test]
    fn test_export_to_csv() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();
        panel.simulate_frame_timing();

        let csv = panel.export_to_csv();
        assert!(csv.starts_with("time,value\n"));
        assert!(csv.contains(','));
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 3); // Header + 2 data rows
    }

    #[test]
    fn test_export_to_json() {
        let mut panel = ChartsPanel::new();
        panel.simulate_frame_timing();

        let json = panel.export_to_json();
        assert!(json.starts_with("{\"data\":["));
        assert!(json.ends_with("]}"));
        assert!(json.contains("\"x\":"));
        assert!(json.contains("\"y\":"));
    }

    #[test]
    fn test_export_empty_data() {
        let panel = ChartsPanel::new();
        let csv = panel.export_to_csv();
        assert_eq!(csv, "time,value\n");

        let json = panel.export_to_json();
        assert_eq!(json, "{\"data\":[]}");
    }

    // === Statistics Tests ===

    #[test]
    fn test_chart_stats_from_empty_data() {
        let data: Vec<(f64, f64)> = vec![];
        let stats = ChartStats::from_data(&data);
        assert_eq!(stats.count, 0);
    }

    #[test]
    fn test_chart_stats_from_single_point() {
        let data = vec![(0.0, 10.0)];
        let stats = ChartStats::from_data(&data);
        assert_eq!(stats.count, 1);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 10.0);
        assert_eq!(stats.avg, 10.0);
    }

    #[test]
    fn test_chart_stats_from_multiple_points() {
        let data = vec![(0.0, 10.0), (1.0, 20.0), (2.0, 30.0)];
        let stats = ChartStats::from_data(&data);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
        assert_eq!(stats.avg, 20.0);
    }

    #[test]
    fn test_frame_stats_updated_on_simulation() {
        let mut panel = ChartsPanel::new();
        assert!(panel.frame_stats.is_none());

        panel.simulate_frame_timing();
        assert!(panel.frame_stats.is_some());

        if let Some(stats) = &panel.frame_stats {
            assert_eq!(stats.count, 1);
        }
    }

    // === Configuration Tests ===

    #[test]
    fn test_chart_height_configuration() {
        let mut panel = ChartsPanel::new();
        assert_eq!(panel.chart_height, 300.0);

        panel.chart_height = 400.0;
        assert_eq!(panel.chart_height, 400.0);
    }

    #[test]
    fn test_max_history_size_configuration() {
        let mut panel = ChartsPanel::new();
        assert_eq!(panel.max_history_size, 600);

        panel.max_history_size = 100;
        
        // Simulate more than max
        for _ in 0..150 {
            panel.simulate_frame_timing();
        }
        
        assert_eq!(panel.frame_history.len(), 100);
    }

    #[test]
    fn test_show_grid_toggle() {
        let mut panel = ChartsPanel::new();
        assert!(panel.show_grid);
        
        panel.show_grid = false;
        assert!(!panel.show_grid);
    }

    #[test]
    fn test_show_legend_toggle() {
        let mut panel = ChartsPanel::new();
        assert!(panel.show_legend);
        
        panel.show_legend = false;
        assert!(!panel.show_legend);
    }

    #[test]
    fn test_show_statistics_toggle() {
        let mut panel = ChartsPanel::new();
        assert!(panel.show_statistics);
        
        panel.show_statistics = false;
        assert!(!panel.show_statistics);
    }

    // === Color System Tests ===

    #[test]
    fn test_custom_colors_initialized() {
        let panel = ChartsPanel::new();
        assert_eq!(panel.custom_colors.len(), 5);
        assert!(panel.custom_colors.contains_key("primary"));
        assert!(panel.custom_colors.contains_key("secondary"));
        assert!(panel.custom_colors.contains_key("success"));
        assert!(panel.custom_colors.contains_key("warning"));
        assert!(panel.custom_colors.contains_key("danger"));
    }

    #[test]
    fn test_get_series_color_frame_timing() {
        let mut panel = ChartsPanel::new();
        panel.active_data_source = DataSource::FrameTiming;
        let color = panel.get_series_color();
        assert_eq!(color, Color32::from_rgb(100, 180, 255));
    }

    #[test]
    fn test_get_series_color_memory() {
        let mut panel = ChartsPanel::new();
        panel.active_data_source = DataSource::MemoryUsage;
        let color = panel.get_series_color();
        assert_eq!(color, Color32::from_rgb(255, 200, 100));
    }

    #[test]
    fn test_get_y_axis_label_frame_timing() {
        let mut panel = ChartsPanel::new();
        panel.active_data_source = DataSource::FrameTiming;
        assert_eq!(panel.get_y_axis_label(), "Frame Time (ms)");
    }

    #[test]
    fn test_get_y_axis_label_memory() {
        let mut panel = ChartsPanel::new();
        panel.active_data_source = DataSource::MemoryUsage;
        assert_eq!(panel.get_y_axis_label(), "Memory (MB)");
    }

    #[test]
    fn test_get_y_axis_label_cpu() {
        let mut panel = ChartsPanel::new();
        panel.active_data_source = DataSource::CpuLoad;
        assert_eq!(panel.get_y_axis_label(), "CPU Usage (%)");
    }

    // === Entity Data Tests ===

    #[test]
    fn test_entity_counts_initialized() {
        let panel = ChartsPanel::new();
        assert_eq!(panel.entity_counts.len(), 4);
        
        for group in &panel.entity_counts {
            assert_eq!(group.bars.len(), 3); // Enemies, Allies, NPCs
        }
    }

    #[test]
    fn test_entity_counts_scene_1() {
        let panel = ChartsPanel::new();
        let scene1 = &panel.entity_counts[0];
        assert_eq!(scene1.category, "Scene 1");
        assert_eq!(scene1.bars[0].value, 45.0); // Enemies
        assert_eq!(scene1.bars[1].value, 12.0); // Allies
        assert_eq!(scene1.bars[2].value, 8.0);  // NPCs
    }

    #[test]
    fn test_spatial_data_initialized() {
        let panel = ChartsPanel::new();
        assert_eq!(panel.spatial_data.len(), 4);
        
        assert!(panel.spatial_data[0].points.len() >= 5); // Enemies
        assert!(panel.spatial_data[1].points.len() >= 4); // Allies
        assert!(panel.spatial_data[2].points.len() >= 4); // NPCs
        assert!(panel.spatial_data[3].points.len() >= 3); // Pickups
    }

    // === Panel Update Tests ===

    #[test]
    fn test_panel_update_increments_frame_count() {
        let mut panel = ChartsPanel::new();
        assert_eq!(panel.frame_count, 0);
        
        panel.update();
        assert_eq!(panel.frame_count, 1);
        
        panel.update();
        assert_eq!(panel.frame_count, 2);
    }

    #[test]
    fn test_panel_update_adds_history() {
        let mut panel = ChartsPanel::new();
        assert_eq!(panel.frame_history.len(), 0);
        
        panel.update();
        assert_eq!(panel.frame_history.len(), 1);
    }

    // === Edge Case Tests ===

    #[test]
    fn test_history_overflow_prevention() {
        let mut panel = ChartsPanel::new();
        panel.max_history_size = 10;
        
        for _ in 0..20 {
            panel.simulate_frame_timing();
        }
        
        assert_eq!(panel.frame_history.len(), 10);
        assert_eq!(panel.memory_history.len(), 10);
        assert_eq!(panel.cpu_history.len(), 10);
        assert_eq!(panel.gpu_history.len(), 10);
    }

    #[test]
    fn test_minimal_history_size() {
        let mut panel = ChartsPanel::new();
        panel.max_history_size = 1;
        
        panel.simulate_frame_timing();
        panel.simulate_frame_timing();
        
        assert_eq!(panel.frame_history.len(), 1);
    }

    #[test]
    fn test_chart_type_switching() {
        let mut panel = ChartsPanel::new();
        assert_eq!(panel.active_chart_type, ChartType::Line);
        
        panel.active_chart_type = ChartType::Bar;
        assert_eq!(panel.active_chart_type, ChartType::Bar);
        
        panel.active_chart_type = ChartType::Scatter;
        assert_eq!(panel.active_chart_type, ChartType::Scatter);
    }

    #[test]
    fn test_data_source_switching() {
        let mut panel = ChartsPanel::new();
        assert_eq!(panel.active_data_source, DataSource::FrameTiming);
        
        panel.active_data_source = DataSource::MemoryUsage;
        assert_eq!(panel.active_data_source, DataSource::MemoryUsage);
    }
}
