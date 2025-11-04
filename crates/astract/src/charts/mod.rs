//! Chart widgets for data visualization in Astract.
//!
//! This module provides production-ready chart widgets for visualizing
//! game engine metrics, debug data, and UI dashboards.
//!
//! # Supported Chart Types
//!
//! - **LineChart**: Time series and trend data
//! - **BarChart**: Categorical and grouped data
//! - **ScatterPlot**: 2D point distributions and clustering
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use astract::charts::LineChart;
//! use egui::Color32;
//!
//! let mut chart = LineChart::new("Frame Times");
//! chart.add_series("Physics", vec![(0.0, 2.1), (1.0, 2.3), (2.0, 2.0)], Color32::GREEN);
//! chart.add_series("Render", vec![(0.0, 6.4), (1.0, 6.8), (2.0, 6.2)], Color32::BLUE);
//!
//! // In your egui UI:
//! // chart.show(ui);
//! ```

mod bar_chart;
mod line_chart;
mod scatter_plot;

pub use bar_chart::{Bar, BarChart, BarChartMode, BarGroup};
pub use line_chart::LineChart;
pub use scatter_plot::{PointCluster, PointShape, ScatterPlot};

use egui::{Color32, Pos2, Rect};

/// A data point in 2D space.
pub type Point = (f64, f64);

/// A series of data points with a name and color.
#[derive(Clone, Debug)]
pub struct DataSeries {
    pub name: String,
    pub points: Vec<Point>,
    pub color: Color32,
    pub visible: bool,
}

impl DataSeries {
    /// Create a new data series.
    pub fn new(name: impl Into<String>, points: Vec<Point>, color: Color32) -> Self {
        Self {
            name: name.into(),
            points,
            color,
            visible: true,
        }
    }

    /// Get the bounding box of this series.
    pub fn bounds(&self) -> Option<(Point, Point)> {
        if self.points.is_empty() {
            return None;
        }

        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for &(x, y) in &self.points {
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        Some(((min_x, min_y), (max_x, max_y)))
    }
}

/// Chart styling options.
#[derive(Clone, Debug)]
pub struct ChartStyle {
    pub background_color: Color32,
    pub grid_color: Color32,
    pub axis_color: Color32,
    pub text_color: Color32,
    pub grid_spacing: f32,
    pub show_grid: bool,
    pub show_axes: bool,
    pub show_legend: bool,
}

impl Default for ChartStyle {
    fn default() -> Self {
        Self {
            background_color: Color32::from_gray(20),
            grid_color: Color32::from_gray(40),
            axis_color: Color32::from_gray(160),
            text_color: Color32::from_gray(200),
            grid_spacing: 50.0,
            show_grid: true,
            show_axes: true,
            show_legend: true,
        }
    }
}

/// Axis configuration for charts.
#[derive(Clone, Debug)]
pub struct AxisConfig {
    pub label: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub auto_scale: bool,
}

impl Default for AxisConfig {
    fn default() -> Self {
        Self {
            label: String::new(),
            min: None,
            max: None,
            auto_scale: true,
        }
    }
}

impl AxisConfig {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Default::default()
        }
    }

    pub fn with_range(mut self, min: f64, max: f64) -> Self {
        self.min = Some(min);
        self.max = Some(max);
        self.auto_scale = false;
        self
    }
}

/// Calculate nice bounds for a range of values.
pub fn calculate_nice_bounds(min: f64, max: f64) -> (f64, f64) {
    if (max - min).abs() < f64::EPSILON {
        return (min - 1.0, max + 1.0);
    }

    let range = max - min;
    let magnitude = 10_f64.powf(range.log10().floor());
    let normalized_range = range / magnitude;

    let nice_range = if normalized_range <= 1.0 {
        1.0
    } else if normalized_range <= 2.0 {
        2.0
    } else if normalized_range <= 5.0 {
        5.0
    } else {
        10.0
    };

    let nice_range_actual = nice_range * magnitude;
    let nice_min = (min / magnitude).floor() * magnitude;
    let nice_max = nice_min + nice_range_actual;

    (nice_min, nice_max)
}

/// Transform a data point to screen coordinates.
pub fn transform_point(point: Point, data_bounds: (Point, Point), screen_rect: Rect) -> Pos2 {
    let ((min_x, min_y), (max_x, max_y)) = data_bounds;
    let (x, y) = point;

    let x_range = max_x - min_x;
    let y_range = max_y - min_y;

    let x_norm = if x_range > f64::EPSILON {
        ((x - min_x) / x_range) as f32
    } else {
        0.5
    };

    let y_norm = if y_range > f64::EPSILON {
        ((y - min_y) / y_range) as f32
    } else {
        0.5
    };

    let screen_x = screen_rect.min.x + x_norm * screen_rect.width();
    let screen_y = screen_rect.max.y - y_norm * screen_rect.height(); // Flip Y

    Pos2::new(screen_x, screen_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_series_bounds() {
        let series = DataSeries::new(
            "test",
            vec![(0.0, 0.0), (10.0, 20.0), (5.0, 10.0)],
            Color32::RED,
        );

        let bounds = series.bounds().unwrap();
        assert_eq!(bounds, ((0.0, 0.0), (10.0, 20.0)));
    }

    #[test]
    fn test_data_series_empty_bounds() {
        let series = DataSeries::new("empty", vec![], Color32::RED);
        assert!(series.bounds().is_none());
    }

    #[test]
    fn test_nice_bounds() {
        let (min, max) = calculate_nice_bounds(1.3, 9.7);
        // Range 8.4, magnitude 1.0, nice_range 10.0
        // nice_min = floor(1.3) * 1.0 = 1.0
        // nice_max = 1.0 + 10.0 = 11.0
        assert_eq!(min, 1.0);
        assert_eq!(max, 11.0);

        let (min, max) = calculate_nice_bounds(143.0, 187.0);
        // Range 44, magnitude 10, nice_range 50
        // nice_min = floor(143/10) * 10 = 140
        // nice_max = 140 + 50 = 190
        assert_eq!(min, 140.0);
        assert_eq!(max, 190.0);
    }

    #[test]
    fn test_transform_point() {
        let data_bounds = ((0.0, 0.0), (100.0, 100.0));
        let screen_rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(200.0, 200.0));

        let screen_pos = transform_point((50.0, 50.0), data_bounds, screen_rect);

        // (50, 50) in data should map to (100, 100) in screen
        // But Y is flipped, so it's (100, 100) in screen coords
        assert!((screen_pos.x - 100.0).abs() < 0.1);
        assert!((screen_pos.y - 100.0).abs() < 0.1);
    }
}
