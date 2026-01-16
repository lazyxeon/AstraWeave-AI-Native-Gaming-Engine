//! Charts showcase tab - demonstrates LineChart, BarChart, and ScatterPlot widgets

use astract::charts::{Bar, BarChart, BarGroup, LineChart, PointCluster, ScatterPlot};
use astract::prelude::egui::*;
use std::f32::consts::PI;

pub struct ChartsTab {
    // Line chart controls
    line_data_type: LineDataType,
    line_point_count: usize,

    // Bar chart data
    bar_values: Vec<f32>,

    // Scatter plot controls
    scatter_cluster_count: usize,

    // Animation
    time: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LineDataType {
    Sine,
    Exponential,
    Random,
}

impl Default for ChartsTab {
    fn default() -> Self {
        Self {
            line_data_type: LineDataType::Sine,
            line_point_count: 50,
            bar_values: vec![65.0, 85.0, 45.0, 95.0, 75.0],
            scatter_cluster_count: 3,
            time: 0.0,
        }
    }
}

impl ChartsTab {
    pub fn show(&mut self, ui: &mut Ui) {
        self.time += ui.input(|i| i.stable_dt);

        ui.heading("📊 Charts Showcase");
        ui.add_space(10.0);

        // Line Chart
        ui.group(|ui| {
            ui.heading("Line Chart");
            ui.label("Visualize time-series and continuous data");

            ui.horizontal(|ui| {
                ui.label("Data type:");
                ui.selectable_value(&mut self.line_data_type, LineDataType::Sine, "Sine Wave");
                ui.selectable_value(
                    &mut self.line_data_type,
                    LineDataType::Exponential,
                    "Exponential",
                );
                ui.selectable_value(
                    &mut self.line_data_type,
                    LineDataType::Random,
                    "Random Walk",
                );
            });

            ui.add(Slider::new(&mut self.line_point_count, 10..=200).text("Points"));

            // Create line chart with correct API
            let mut chart = LineChart::new("Line Chart Demo");

            // Generate data points as (f64, f64) tuples
            let points = self.generate_line_data();

            // Add series using add_series method
            let label = match self.line_data_type {
                LineDataType::Sine => "sin(x)",
                LineDataType::Exponential => "exp(x/10)",
                LineDataType::Random => "Random Walk",
            };

            chart.add_series(label, points, Color32::from_rgb(100, 150, 255));

            // Show chart
            chart.show(ui);
        });

        ui.add_space(20.0);

        // Bar Chart
        ui.group(|ui| {
            ui.heading("Bar Chart");
            ui.label("Compare categorical data with vertical bars");

            ui.horizontal(|ui| {
                ui.label("Adjust values:");
            });

            // Value sliders
            for (i, value) in self.bar_values.iter_mut().enumerate() {
                ui.add(
                    Slider::new(value, 0.0..=100.0)
                        .text(format!("Product {}", (b'A' + i as u8) as char)),
                );
            }

            // Create bar chart with correct API
            let mut chart = BarChart::new("Product Sales");

            // Add bar group
            let group = BarGroup {
                category: "Q1 2025".to_string(),
                bars: vec![
                    Bar {
                        label: "Product A".to_string(),
                        value: self.bar_values[0] as f64,
                        color: Color32::from_rgb(255, 100, 100),
                    },
                    Bar {
                        label: "Product B".to_string(),
                        value: self.bar_values[1] as f64,
                        color: Color32::from_rgb(100, 255, 100),
                    },
                    Bar {
                        label: "Product C".to_string(),
                        value: self.bar_values[2] as f64,
                        color: Color32::from_rgb(100, 100, 255),
                    },
                    Bar {
                        label: "Product D".to_string(),
                        value: self.bar_values[3] as f64,
                        color: Color32::from_rgb(255, 255, 100),
                    },
                    Bar {
                        label: "Product E".to_string(),
                        value: self.bar_values[4] as f64,
                        color: Color32::from_rgb(255, 100, 255),
                    },
                ],
            };

            chart.add_group(group);

            // Show chart
            chart.show(ui);
        });

        ui.add_space(20.0);

        // Scatter Plot
        ui.group(|ui| {
            ui.heading("Scatter Plot");
            ui.label("Visualize relationships between two variables");

            ui.add(Slider::new(&mut self.scatter_cluster_count, 1..=5).text("Clusters"));

            // Create scatter plot
            let mut scatter = ScatterPlot::new("Data Clusters");

            // Generate clustered data
            for i in 0..self.scatter_cluster_count {
                let points = self.generate_scatter_cluster(i);
                let color = match i % 5 {
                    0 => Color32::RED,
                    1 => Color32::GREEN,
                    2 => Color32::BLUE,
                    3 => Color32::YELLOW,
                    _ => Color32::LIGHT_BLUE,
                };

                let cluster = PointCluster::new(format!("Cluster {}", i + 1), points, color);
                scatter.add_cluster(cluster);
            }

            // Show scatter plot
            scatter.show(ui);
        });

        ui.add_space(20.0);

        // Code example
        ui.collapsing("📝 Code Example", |ui| {
            ui.label("Line Chart Usage:");
            ui.code(
                r#"use astract::charts::LineChart;
use egui::Color32;

let mut chart = LineChart::new("Frame Times");

// Points are (f64, f64) tuples
let points = vec![
    (0.0, 2.1),
    (1.0, 2.3),
    (2.0, 2.0),
];

chart.add_series("Physics", points, Color32::GREEN);
chart.show(ui);"#,
            );

            ui.add_space(10.0);
            ui.label("Bar Chart Usage:");
            ui.code(
                r#"use astract::charts::{BarChart, BarGroup, Bar};
use egui::Color32;

let mut chart = BarChart::new("Sales Data");

let group = BarGroup {
    category: "Q1".to_string(),
    bars: vec![
        Bar { label: "A".into(), value: 65.0, color: Color32::RED },
        Bar { label: "B".into(), value: 85.0, color: Color32::GREEN },
    ],
};

chart.add_group(group);
chart.show(ui);"#,
            );
        });
    }

    /// Generate line chart data as (f64, f64) tuples
    fn generate_line_data(&self) -> Vec<(f64, f64)> {
        let count = self.line_point_count;
        match self.line_data_type {
            LineDataType::Sine => (0..count)
                .map(|i| {
                    let x = i as f64 / count as f64 * 4.0 * PI as f64;
                    let y = x.sin();
                    (x, y)
                })
                .collect(),
            LineDataType::Exponential => (0..count)
                .map(|i| {
                    let x = i as f64 / count as f64 * 10.0;
                    let y = (x / 10.0).exp();
                    (x, y)
                })
                .collect(),
            LineDataType::Random => {
                let mut y = 0.0;
                (0..count)
                    .map(|i| {
                        let x = i as f64;
                        // Simple random walk
                        y += ((i * 1234567) % 100) as f64 / 50.0 - 1.0;
                        (x, y)
                    })
                    .collect()
            }
        }
    }

    /// Generate scatter plot cluster data
    fn generate_scatter_cluster(&self, cluster_index: usize) -> Vec<(f64, f64)> {
        let center_x = (cluster_index as f64 * 50.0) % 200.0;
        let center_y = ((cluster_index * 73) % 100) as f64;

        (0..30)
            .map(|i| {
                let angle = i as f64 * 0.5 + cluster_index as f64;
                let radius = ((i * 17 + cluster_index * 31) % 20) as f64 + 5.0;
                let x = center_x + radius * angle.cos();
                let y = center_y + radius * angle.sin();
                (x, y)
            })
            .collect()
    }
}
