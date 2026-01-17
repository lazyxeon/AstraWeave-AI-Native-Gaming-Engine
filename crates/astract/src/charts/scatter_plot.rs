//! Scatter plot widget for 2D point distribution visualization.

use crate::charts::{calculate_nice_bounds, transform_point, AxisConfig, ChartStyle, Point};
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Point shape for scatter plot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PointShape {
    Circle,
    Square,
    Triangle,
    Diamond,
}

/// A cluster of points with shared styling.
#[derive(Clone, Debug)]
pub struct PointCluster {
    pub name: String,
    pub points: Vec<Point>,
    pub color: Color32,
    pub shape: PointShape,
    pub size: f32,
    pub visible: bool,
}

impl PointCluster {
    /// Create a new point cluster.
    pub fn new(name: impl Into<String>, points: Vec<Point>, color: Color32) -> Self {
        Self {
            name: name.into(),
            points,
            color,
            shape: PointShape::Circle,
            size: 4.0,
            visible: true,
        }
    }

    /// Set point shape.
    pub fn shape(mut self, shape: PointShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set point size in pixels.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Get the bounding box of this cluster.
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

/// A scatter plot widget for visualizing 2D point distributions.
///
/// # Example
///
/// ```rust,ignore
/// use astract::charts::{ScatterPlot, PointCluster};
/// use egui::Color32;
///
/// let mut chart = ScatterPlot::new("Entity Positions");
/// chart.add_cluster(PointCluster::new(
///     "Enemies",
///     vec![(10.0, 15.0), (12.0, 18.0), (11.0, 16.0)],
///     Color32::RED,
/// ));
/// chart.add_cluster(PointCluster::new(
///     "Allies",
///     vec![(5.0, 8.0), (6.0, 9.0), (5.5, 7.5)],
///     Color32::GREEN,
/// ));
///
/// // In your egui UI:
/// chart.show(ui);
/// ```
pub struct ScatterPlot {
    title: String,
    clusters: Vec<PointCluster>,
    x_axis: AxisConfig,
    y_axis: AxisConfig,
    style: ChartStyle,
    height: f32,
    show_connecting_lines: bool,
}

impl ScatterPlot {
    /// Create a new scatter plot with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            clusters: Vec::new(),
            x_axis: AxisConfig::new("X"),
            y_axis: AxisConfig::new("Y"),
            style: ChartStyle::default(),
            height: 200.0,
            show_connecting_lines: false,
        }
    }

    /// Set the chart height in pixels.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Show/hide connecting lines between points in each cluster.
    pub fn show_connecting_lines(mut self, show: bool) -> Self {
        self.show_connecting_lines = show;
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

    /// Add a point cluster to the plot.
    pub fn add_cluster(&mut self, cluster: PointCluster) {
        self.clusters.push(cluster);
    }

    /// Clear all clusters.
    pub fn clear(&mut self) {
        self.clusters.clear();
    }

    /// Calculate the combined bounds of all visible clusters.
    fn calculate_bounds(&self) -> Option<(Point, Point)> {
        let mut all_min_x = f64::INFINITY;
        let mut all_max_x = f64::NEG_INFINITY;
        let mut all_min_y = f64::INFINITY;
        let mut all_max_y = f64::NEG_INFINITY;
        let mut found_any = false;

        for cluster in &self.clusters {
            if !cluster.visible {
                continue;
            }

            if let Some(((min_x, min_y), (max_x, max_y))) = cluster.bounds() {
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

        // Calculate data area
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

        // Draw grid
        if self.style.show_grid {
            self.draw_grid(&painter, data_rect, data_bounds);
        }

        // Draw axes
        if self.style.show_axes {
            self.draw_axes(&painter, data_rect, data_bounds);
        }

        // Draw clusters
        for cluster in &self.clusters {
            if !cluster.visible || cluster.points.is_empty() {
                continue;
            }

            let screen_points: Vec<Pos2> = cluster
                .points
                .iter()
                .map(|&p| transform_point(p, data_bounds, data_rect))
                .collect();

            // Draw connecting lines (optional)
            if self.show_connecting_lines && screen_points.len() >= 2 {
                painter.add(egui::Shape::line(
                    screen_points.clone(),
                    Stroke::new(1.0, cluster.color.gamma_multiply(0.5)),
                ));
            }

            // Draw points
            for pos in screen_points {
                self.draw_point(&painter, pos, cluster.shape, cluster.size, cluster.color);
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

    fn draw_point(
        &self,
        painter: &egui::Painter,
        pos: Pos2,
        shape: PointShape,
        size: f32,
        color: Color32,
    ) {
        match shape {
            PointShape::Circle => {
                painter.circle_filled(pos, size, color);
                painter.circle_stroke(pos, size, Stroke::new(1.0, Color32::BLACK));
            }
            PointShape::Square => {
                let rect = Rect::from_center_size(pos, Vec2::splat(size * 2.0));
                painter.rect_filled(rect, 0.0, color);
                painter.rect_stroke(rect, 0.0, (1.0, Color32::BLACK), egui::StrokeKind::Middle);
            }
            PointShape::Triangle => {
                let h = size * 1.5;
                let points = [
                    pos + Vec2::new(0.0, -h),
                    pos + Vec2::new(-h, h),
                    pos + Vec2::new(h, h),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points.to_vec(),
                    color,
                    Stroke::new(1.0, Color32::BLACK),
                ));
            }
            PointShape::Diamond => {
                let s = size * 1.2;
                let points = [
                    pos + Vec2::new(0.0, -s),
                    pos + Vec2::new(s, 0.0),
                    pos + Vec2::new(0.0, s),
                    pos + Vec2::new(-s, 0.0),
                ];
                painter.add(egui::Shape::convex_polygon(
                    points.to_vec(),
                    color,
                    Stroke::new(1.0, Color32::BLACK),
                ));
            }
        }
    }

    fn draw_grid(&self, painter: &egui::Painter, data_rect: Rect, data_bounds: (Point, Point)) {
        let ((min_x, min_y), (max_x, max_y)) = data_bounds;

        // Vertical grid lines
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

        // Horizontal grid lines
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

        for cluster in &self.clusters {
            if !cluster.visible {
                continue;
            }

            // Shape indicator
            let shape_pos = Pos2::new(legend_x + 6.0, legend_y);
            self.draw_point(painter, shape_pos, cluster.shape, 4.0, cluster.color);

            // Cluster name
            painter.text(
                Pos2::new(legend_x + 18.0, legend_y),
                egui::Align2::LEFT_CENTER,
                &cluster.name,
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
    fn test_scatter_plot_creation() {
        let mut plot = ScatterPlot::new("Test Plot");
        plot.add_cluster(PointCluster::new(
            "Cluster 1",
            vec![(0.0, 0.0), (1.0, 1.0)],
            Color32::RED,
        ));

        assert_eq!(plot.clusters.len(), 1);
    }

    #[test]
    fn test_cluster_bounds() {
        let cluster = PointCluster::new(
            "Test",
            vec![(0.0, 0.0), (10.0, 20.0), (5.0, 10.0)],
            Color32::RED,
        );

        let bounds = cluster.bounds().unwrap();
        assert_eq!(bounds, ((0.0, 0.0), (10.0, 20.0)));
    }

    #[test]
    fn test_point_shapes() {
        let cluster = PointCluster::new("Test", vec![], Color32::RED)
            .shape(PointShape::Square)
            .size(5.0);

        assert_eq!(cluster.shape, PointShape::Square);
        assert_eq!(cluster.size, 5.0);
    }
}
