//! Line chart widget for time series and trend visualization.

use crate::charts::{
    calculate_nice_bounds, transform_point, AxisConfig, ChartStyle, DataSeries, Point,
};
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// A line chart widget for visualizing time series data.
///
/// # Example
///
/// ```rust,ignore
/// use astract::charts::LineChart;
/// use egui::Color32;
///
/// let mut chart = LineChart::new("Frame Times");
/// chart.add_series("Physics", vec![(0.0, 2.1), (1.0, 2.3), (2.0, 2.0)], Color32::GREEN);
/// chart.add_series("Render", vec![(0.0, 6.4), (1.0, 6.8), (2.0, 6.2)], Color32::BLUE);
///
/// // In your egui UI:
/// chart.show(ui);
/// ```
pub struct LineChart {
    title: String,
    series: Vec<DataSeries>,
    x_axis: AxisConfig,
    y_axis: AxisConfig,
    style: ChartStyle,
    height: f32,
    line_width: f32,
    show_points: bool,
    point_radius: f32,
}

impl LineChart {
    /// Create a new line chart with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            series: Vec::new(),
            x_axis: AxisConfig::new("X"),
            y_axis: AxisConfig::new("Y"),
            style: ChartStyle::default(),
            height: 200.0,
            line_width: 2.0,
            show_points: true,
            point_radius: 3.0,
        }
    }

    /// Set the chart height in pixels.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Set line width in pixels.
    pub fn line_width(mut self, width: f32) -> Self {
        self.line_width = width;
        self
    }

    /// Show/hide data points.
    pub fn show_points(mut self, show: bool) -> Self {
        self.show_points = show;
        self
    }

    /// Set X axis configuration.
    pub fn x_axis(mut self, config: AxisConfig) -> Self {
        self.x_axis = config;
        self
    }

    /// Set Y axis configuration.
    pub fn y_axis(mut self, config: AxisConfig) -> Self {
        self.y_axis = config;
        self
    }

    /// Add a data series to the chart.
    pub fn add_series(&mut self, name: impl Into<String>, points: Vec<Point>, color: Color32) {
        self.series.push(DataSeries::new(name, points, color));
    }

    /// Clear all series.
    pub fn clear(&mut self) {
        self.series.clear();
    }

    /// Calculate the combined bounds of all visible series.
    fn calculate_bounds(&self) -> Option<(Point, Point)> {
        let mut all_min_x = f64::INFINITY;
        let mut all_max_x = f64::NEG_INFINITY;
        let mut all_min_y = f64::INFINITY;
        let mut all_max_y = f64::NEG_INFINITY;
        let mut found_any = false;

        for series in &self.series {
            if !series.visible {
                continue;
            }

            if let Some(((min_x, min_y), (max_x, max_y))) = series.bounds() {
                all_min_x = all_min_x.min(min_x);
                all_max_x = all_max_x.max(max_x);
                all_min_y = all_min_y.min(min_y);
                all_max_y = all_max_y.max(max_y);
                found_any = true;
            }
        }

        if !found_any {
            return None;
        }

        // Apply axis constraints or auto-scale
        let min_x = self.x_axis.min.unwrap_or(all_min_x);
        let max_x = self.x_axis.max.unwrap_or(all_max_x);
        let min_y = self.y_axis.min.unwrap_or(all_min_y);
        let max_y = self.y_axis.max.unwrap_or(all_max_y);

        let (final_min_x, final_max_x) = if self.x_axis.auto_scale {
            calculate_nice_bounds(min_x, max_x)
        } else {
            (min_x, max_x)
        };

        let (final_min_y, final_max_y) = if self.y_axis.auto_scale {
            calculate_nice_bounds(min_y, max_y)
        } else {
            (min_y, max_y)
        };

        Some(((final_min_x, final_min_y), (final_max_x, final_max_y)))
    }

    /// Render the chart.
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        let (response, painter) =
            ui.allocate_painter(Vec2::new(ui.available_width(), self.height), Sense::hover());

        let chart_rect = response.rect;

        // Background
        painter.rect_filled(chart_rect, 2.0, self.style.background_color);

        // Title
        let title_pos = chart_rect.min + Vec2::new(10.0, 5.0);
        painter.text(
            title_pos,
            egui::Align2::LEFT_TOP,
            &self.title,
            egui::FontId::proportional(14.0),
            self.style.text_color,
        );

        // Calculate data area (leave space for axes and legend)
        let margin = 40.0;
        let legend_width = if self.style.show_legend { 120.0 } else { 0.0 };
        let data_rect = Rect::from_min_max(
            chart_rect.min + Vec2::new(margin, 30.0),
            chart_rect.max - Vec2::new(margin + legend_width, margin),
        );

        // Get data bounds
        let Some(data_bounds) = self.calculate_bounds() else {
            painter.text(
                chart_rect.center(),
                egui::Align2::CENTER_CENTER,
                "No data",
                egui::FontId::proportional(12.0),
                self.style.text_color,
            );
            return response;
        };

        let ((_min_x, _min_y), (_max_x, _max_y)) = data_bounds;

        // Draw grid
        if self.style.show_grid {
            self.draw_grid(&painter, data_rect, data_bounds);
        }

        // Draw axes
        if self.style.show_axes {
            self.draw_axes(&painter, data_rect, data_bounds);
        }

        // Draw data series
        for series in &self.series {
            if !series.visible || series.points.is_empty() {
                continue;
            }

            // Draw lines
            let screen_points: Vec<Pos2> = series
                .points
                .iter()
                .map(|&p| transform_point(p, data_bounds, data_rect))
                .collect();

            if screen_points.len() >= 2 {
                painter.add(egui::Shape::line(
                    screen_points.clone(),
                    Stroke::new(self.line_width, series.color),
                ));
            }

            // Draw points
            if self.show_points {
                for pos in screen_points {
                    painter.circle_filled(pos, self.point_radius, series.color);
                }
            }
        }

        // Draw legend
        if self.style.show_legend {
            self.draw_legend(&painter, chart_rect);
        }

        // Axis labels
        let x_label_pos = Pos2::new(data_rect.center().x, chart_rect.max.y - 10.0);
        painter.text(
            x_label_pos,
            egui::Align2::CENTER_BOTTOM,
            &self.x_axis.label,
            egui::FontId::proportional(10.0),
            self.style.text_color,
        );

        let y_label_pos = Pos2::new(chart_rect.min.x + 10.0, data_rect.center().y);
        painter.text(
            y_label_pos,
            egui::Align2::LEFT_CENTER,
            &self.y_axis.label,
            egui::FontId::proportional(10.0),
            self.style.text_color,
        );

        response
    }

    fn draw_grid(&self, painter: &egui::Painter, data_rect: Rect, data_bounds: (Point, Point)) {
        let ((min_x, min_y), (max_x, max_y)) = data_bounds;

        // Vertical grid lines (X axis)
        let num_x_lines = 5;
        for i in 0..=num_x_lines {
            let t = i as f64 / num_x_lines as f64;
            let x = min_x + t * (max_x - min_x);
            let screen_pos = transform_point((x, min_y), data_bounds, data_rect);

            painter.line_segment(
                [
                    Pos2::new(screen_pos.x, data_rect.min.y),
                    Pos2::new(screen_pos.x, data_rect.max.y),
                ],
                Stroke::new(1.0, self.style.grid_color),
            );
        }

        // Horizontal grid lines (Y axis)
        let num_y_lines = 5;
        for i in 0..=num_y_lines {
            let t = i as f64 / num_y_lines as f64;
            let y = min_y + t * (max_y - min_y);
            let screen_pos = transform_point((min_x, y), data_bounds, data_rect);

            painter.line_segment(
                [
                    Pos2::new(data_rect.min.x, screen_pos.y),
                    Pos2::new(data_rect.max.x, screen_pos.y),
                ],
                Stroke::new(1.0, self.style.grid_color),
            );
        }
    }

    fn draw_axes(&self, painter: &egui::Painter, data_rect: Rect, data_bounds: (Point, Point)) {
        let ((min_x, min_y), (max_x, max_y)) = data_bounds;

        // X axis
        painter.line_segment(
            [
                Pos2::new(data_rect.min.x, data_rect.max.y),
                Pos2::new(data_rect.max.x, data_rect.max.y),
            ],
            Stroke::new(1.5, self.style.axis_color),
        );

        // Y axis
        painter.line_segment(
            [
                Pos2::new(data_rect.min.x, data_rect.min.y),
                Pos2::new(data_rect.min.x, data_rect.max.y),
            ],
            Stroke::new(1.5, self.style.axis_color),
        );

        // Tick labels
        let num_ticks = 5;
        for i in 0..=num_ticks {
            let t = i as f64 / num_ticks as f64;

            // X tick
            let x_val = min_x + t * (max_x - min_x);
            let x_screen = transform_point((x_val, min_y), data_bounds, data_rect);
            painter.text(
                Pos2::new(x_screen.x, data_rect.max.y + 5.0),
                egui::Align2::CENTER_TOP,
                format!("{:.1}", x_val),
                egui::FontId::proportional(9.0),
                self.style.text_color,
            );

            // Y tick
            let y_val = min_y + t * (max_y - min_y);
            let y_screen = transform_point((min_x, y_val), data_bounds, data_rect);
            painter.text(
                Pos2::new(data_rect.min.x - 5.0, y_screen.y),
                egui::Align2::RIGHT_CENTER,
                format!("{:.1}", y_val),
                egui::FontId::proportional(9.0),
                self.style.text_color,
            );
        }
    }

    fn draw_legend(&self, painter: &egui::Painter, chart_rect: Rect) {
        let legend_x = chart_rect.max.x - 110.0;
        let mut legend_y = chart_rect.min.y + 40.0;

        for series in &self.series {
            if !series.visible {
                continue;
            }

            // Color box
            let box_rect =
                Rect::from_min_size(Pos2::new(legend_x, legend_y - 6.0), Vec2::new(12.0, 12.0));
            painter.rect_filled(box_rect, 2.0, series.color);
            painter.rect_stroke(
                box_rect,
                2.0,
                (1.0, self.style.text_color),
                egui::StrokeKind::Middle,
            );

            // Series name
            painter.text(
                Pos2::new(legend_x + 18.0, legend_y),
                egui::Align2::LEFT_CENTER,
                &series.name,
                egui::FontId::proportional(10.0),
                self.style.text_color,
            );

            legend_y += 18.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_chart_creation() {
        let mut chart = LineChart::new("Test Chart");
        chart.add_series("Series 1", vec![(0.0, 0.0), (1.0, 1.0)], Color32::RED);

        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].name, "Series 1");
    }

    #[test]
    fn test_bounds_calculation() {
        let mut chart = LineChart::new("Test");
        chart.add_series("A", vec![(0.0, 0.0), (10.0, 20.0)], Color32::RED);
        chart.add_series("B", vec![(5.0, 10.0), (15.0, 30.0)], Color32::BLUE);

        let bounds = chart.calculate_bounds().unwrap();
        assert_eq!(bounds.0 .0, 0.0); // min_x (auto-scaled)
        assert_eq!(bounds.1 .0, 20.0); // max_x (auto-scaled)
    }

    #[test]
    fn test_clear() {
        let mut chart = LineChart::new("Test");
        chart.add_series("A", vec![(0.0, 0.0)], Color32::RED);
        chart.clear();

        assert_eq!(chart.series.len(), 0);
    }
}
