//! Debug visualization utilities for water effects
//!
//! Provides visualization helpers for debugging and profiling water systems
//! during development. All debug functionality can be compiled out in release.

use glam::{Vec3, Vec4};

/// Debug visualization settings
#[derive(Debug, Clone, PartialEq)]
pub struct WaterDebugConfig {
    /// Show particle positions as points
    pub show_particles: bool,
    /// Show particle velocities as lines
    pub show_velocities: bool,
    /// Show god ray shafts
    pub show_god_rays: bool,
    /// Show caustic projector bounds
    pub show_caustic_bounds: bool,
    /// Show water surface wireframe
    pub show_surface_grid: bool,
    /// Show foam sources
    pub show_foam_sources: bool,
    /// Show performance overlay
    pub show_stats_overlay: bool,
    /// Particle point size for debug rendering
    pub particle_point_size: f32,
    /// Velocity line scale
    pub velocity_scale: f32,
    /// Debug line width
    pub line_width: f32,
}

impl Default for WaterDebugConfig {
    fn default() -> Self {
        Self {
            show_particles: false,
            show_velocities: false,
            show_god_rays: false,
            show_caustic_bounds: false,
            show_surface_grid: false,
            show_foam_sources: false,
            show_stats_overlay: true,
            particle_point_size: 4.0,
            velocity_scale: 0.5,
            line_width: 1.0,
        }
    }
}

impl WaterDebugConfig {
    /// All visualizations enabled
    pub fn all_enabled() -> Self {
        Self {
            show_particles: true,
            show_velocities: true,
            show_god_rays: true,
            show_caustic_bounds: true,
            show_surface_grid: true,
            show_foam_sources: true,
            show_stats_overlay: true,
            ..Default::default()
        }
    }

    /// Only stats overlay
    pub fn stats_only() -> Self {
        Self {
            show_stats_overlay: true,
            ..Self::default()
        }
    }

    /// Check if any visualization is enabled
    pub fn any_enabled(&self) -> bool {
        self.show_particles
            || self.show_velocities
            || self.show_god_rays
            || self.show_caustic_bounds
            || self.show_surface_grid
            || self.show_foam_sources
            || self.show_stats_overlay
    }
}

/// A debug line for visualization
#[derive(Debug, Clone, Copy)]
pub struct DebugLine {
    /// Start point
    pub start: Vec3,
    /// End point
    pub end: Vec3,
    /// Color (RGBA)
    pub color: Vec4,
}

impl DebugLine {
    /// Create a new debug line
    pub fn new(start: Vec3, end: Vec3, color: Vec4) -> Self {
        Self { start, end, color }
    }

    /// Create a red line
    pub fn red(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(1.0, 0.0, 0.0, 1.0))
    }

    /// Create a green line
    pub fn green(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(0.0, 1.0, 0.0, 1.0))
    }

    /// Create a blue line
    pub fn blue(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(0.0, 0.0, 1.0, 1.0))
    }

    /// Create a yellow line
    pub fn yellow(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(1.0, 1.0, 0.0, 1.0))
    }

    /// Create a cyan line
    pub fn cyan(start: Vec3, end: Vec3) -> Self {
        Self::new(start, end, Vec4::new(0.0, 1.0, 1.0, 1.0))
    }
}

/// A debug point for visualization
#[derive(Debug, Clone, Copy)]
pub struct DebugPoint {
    /// Position
    pub position: Vec3,
    /// Color (RGBA)
    pub color: Vec4,
    /// Size
    pub size: f32,
}

impl DebugPoint {
    /// Create a new debug point
    pub fn new(position: Vec3, color: Vec4, size: f32) -> Self {
        Self { position, color, size }
    }

    /// Create from particle data
    pub fn from_particle(position: Vec3, particle_type: ParticleDebugType) -> Self {
        let (color, size) = match particle_type {
            ParticleDebugType::Foam => (Vec4::new(1.0, 1.0, 1.0, 0.8), 3.0),
            ParticleDebugType::Bubble => (Vec4::new(0.5, 0.8, 1.0, 0.6), 4.0),
            ParticleDebugType::Debris => (Vec4::new(0.4, 0.3, 0.2, 0.7), 2.0),
            ParticleDebugType::WaterfallDroplet => (Vec4::new(0.3, 0.6, 1.0, 0.9), 2.5),
            ParticleDebugType::Mist => (Vec4::new(0.8, 0.9, 1.0, 0.3), 5.0),
        };
        Self { position, color, size }
    }
}

/// Particle type for debug coloring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleDebugType {
    /// Foam particle
    Foam,
    /// Bubble
    Bubble,
    /// Debris/sediment
    Debris,
    /// Waterfall droplet
    WaterfallDroplet,
    /// Mist particle
    Mist,
}

/// Debug draw commands collected during a frame
#[derive(Debug, Default)]
pub struct DebugDrawList {
    /// Lines to draw
    pub lines: Vec<DebugLine>,
    /// Points to draw
    pub points: Vec<DebugPoint>,
    /// Text labels (position, text)
    pub labels: Vec<(Vec3, String)>,
}

impl DebugDrawList {
    /// Create empty draw list
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with capacity hints
    pub fn with_capacity(lines: usize, points: usize) -> Self {
        Self {
            lines: Vec::with_capacity(lines),
            points: Vec::with_capacity(points),
            labels: Vec::new(),
        }
    }

    /// Add a line
    pub fn add_line(&mut self, line: DebugLine) {
        self.lines.push(line);
    }

    /// Add a point
    pub fn add_point(&mut self, point: DebugPoint) {
        self.points.push(point);
    }

    /// Add a text label
    pub fn add_label(&mut self, position: Vec3, text: impl Into<String>) {
        self.labels.push((position, text.into()));
    }

    /// Add an axis-aligned bounding box
    pub fn add_aabb(&mut self, min: Vec3, max: Vec3, color: Vec4) {
        // Bottom face
        self.lines.push(DebugLine::new(Vec3::new(min.x, min.y, min.z), Vec3::new(max.x, min.y, min.z), color));
        self.lines.push(DebugLine::new(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, min.y, max.z), color));
        self.lines.push(DebugLine::new(Vec3::new(max.x, min.y, max.z), Vec3::new(min.x, min.y, max.z), color));
        self.lines.push(DebugLine::new(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, min.y, min.z), color));

        // Top face
        self.lines.push(DebugLine::new(Vec3::new(min.x, max.y, min.z), Vec3::new(max.x, max.y, min.z), color));
        self.lines.push(DebugLine::new(Vec3::new(max.x, max.y, min.z), Vec3::new(max.x, max.y, max.z), color));
        self.lines.push(DebugLine::new(Vec3::new(max.x, max.y, max.z), Vec3::new(min.x, max.y, max.z), color));
        self.lines.push(DebugLine::new(Vec3::new(min.x, max.y, max.z), Vec3::new(min.x, max.y, min.z), color));

        // Vertical edges
        self.lines.push(DebugLine::new(Vec3::new(min.x, min.y, min.z), Vec3::new(min.x, max.y, min.z), color));
        self.lines.push(DebugLine::new(Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, max.y, min.z), color));
        self.lines.push(DebugLine::new(Vec3::new(max.x, min.y, max.z), Vec3::new(max.x, max.y, max.z), color));
        self.lines.push(DebugLine::new(Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, max.y, max.z), color));
    }

    /// Add a grid on the XZ plane
    pub fn add_grid(&mut self, center: Vec3, size: f32, divisions: u32, color: Vec4) {
        let half = size * 0.5;
        let step = size / divisions as f32;

        for i in 0..=divisions {
            let offset = -half + step * i as f32;

            // X-parallel lines
            self.lines.push(DebugLine::new(
                Vec3::new(center.x - half, center.y, center.z + offset),
                Vec3::new(center.x + half, center.y, center.z + offset),
                color,
            ));

            // Z-parallel lines
            self.lines.push(DebugLine::new(
                Vec3::new(center.x + offset, center.y, center.z - half),
                Vec3::new(center.x + offset, center.y, center.z + half),
                color,
            ));
        }
    }

    /// Add a circle on the XZ plane
    pub fn add_circle(&mut self, center: Vec3, radius: f32, segments: u32, color: Vec4) {
        let step = std::f32::consts::TAU / segments as f32;
        
        for i in 0..segments {
            let a1 = step * i as f32;
            let a2 = step * (i + 1) as f32;

            let p1 = center + Vec3::new(a1.cos() * radius, 0.0, a1.sin() * radius);
            let p2 = center + Vec3::new(a2.cos() * radius, 0.0, a2.sin() * radius);

            self.lines.push(DebugLine::new(p1, p2, color));
        }
    }

    /// Add a velocity vector
    pub fn add_velocity(&mut self, position: Vec3, velocity: Vec3, scale: f32) {
        let speed = velocity.length();
        if speed < 0.001 {
            return;
        }

        // Color based on speed (blue = slow, red = fast)
        let t = (speed / 10.0).min(1.0);
        let color = Vec4::new(t, 0.2, 1.0 - t, 0.8);

        self.lines.push(DebugLine::new(position, position + velocity * scale, color));
    }

    /// Clear all debug draws
    pub fn clear(&mut self) {
        self.lines.clear();
        self.points.clear();
        self.labels.clear();
    }

    /// Get total primitive count
    pub fn primitive_count(&self) -> usize {
        self.lines.len() + self.points.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.points.is_empty() && self.labels.is_empty()
    }
}

/// Format statistics for display
pub struct StatsFormatter;

impl StatsFormatter {
    /// Format time in microseconds to appropriate unit
    pub fn format_time_us(us: u64) -> String {
        if us >= 1_000_000 {
            format!("{:.2} s", us as f64 / 1_000_000.0)
        } else if us >= 1_000 {
            format!("{:.2} ms", us as f64 / 1_000.0)
        } else {
            format!("{} µs", us)
        }
    }

    /// Format particle count with commas
    pub fn format_count(count: usize) -> String {
        if count >= 1_000_000 {
            format!("{:.2}M", count as f64 / 1_000_000.0)
        } else if count >= 1_000 {
            format!("{:.1}K", count as f64 / 1_000.0)
        } else {
            count.to_string()
        }
    }

    /// Format percentage
    pub fn format_percent(value: f32) -> String {
        format!("{:.1}%", value)
    }

    /// Format stats as multi-line string
    pub fn format_stats(stats: &crate::water_effects::WaterEffectsStats) -> String {
        let mut lines = Vec::new();
        
        lines.push("═══ Water Effects ═══".to_string());
        lines.push(format!("Frame: {}", stats.frame));
        lines.push(format!("Total: {}", Self::format_time_us(stats.total_update_us)));
        
        if stats.budget_exceeded {
            lines.push("⚠ BUDGET EXCEEDED".to_string());
        }
        
        lines.push(String::new());
        lines.push("─── Timing ───".to_string());
        
        for (name, pct) in stats.breakdown_percentages() {
            lines.push(format!("  {}: {}", name, Self::format_percent(pct)));
        }
        
        lines.push(String::new());
        lines.push("─── Particles ───".to_string());
        lines.push(format!("  Total: {}", Self::format_count(stats.total_particles)));
        lines.push(format!("  Foam: {}", Self::format_count(stats.foam_particles)));
        lines.push(format!("  Underwater: {}", Self::format_count(stats.underwater_particle_count)));
        lines.push(format!("  Waterfall: {}", Self::format_count(stats.waterfall_particles)));
        lines.push(format!("  God Rays: {}", stats.god_ray_shafts));
        
        lines.join("\n")
    }
}

/// GPU-compatible debug vertex for line/point rendering
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct DebugVertex {
    /// Position
    pub position: [f32; 3],
    /// Color RGBA
    pub color: [f32; 4],
}

impl DebugVertex {
    /// Create from debug line start
    pub fn from_line_start(line: &DebugLine) -> Self {
        Self {
            position: [line.start.x, line.start.y, line.start.z],
            color: [line.color.x, line.color.y, line.color.z, line.color.w],
        }
    }

    /// Create from debug line end
    pub fn from_line_end(line: &DebugLine) -> Self {
        Self {
            position: [line.end.x, line.end.y, line.end.z],
            color: [line.color.x, line.color.y, line.color.z, line.color.w],
        }
    }

    /// Create from debug point
    pub fn from_point(point: &DebugPoint) -> Self {
        Self {
            position: [point.position.x, point.position.y, point.position.z],
            color: [point.color.x, point.color.y, point.color.z, point.color.w],
        }
    }
}

unsafe impl bytemuck::Pod for DebugVertex {}
unsafe impl bytemuck::Zeroable for DebugVertex {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_config_default() {
        let config = WaterDebugConfig::default();
        assert!(!config.show_particles);
        assert!(config.show_stats_overlay);
    }

    #[test]
    fn test_debug_config_all_enabled() {
        let config = WaterDebugConfig::all_enabled();
        assert!(config.show_particles);
        assert!(config.show_velocities);
        assert!(config.show_god_rays);
        assert!(config.any_enabled());
    }

    #[test]
    fn test_debug_line_creation() {
        let line = DebugLine::red(Vec3::ZERO, Vec3::ONE);
        assert_eq!(line.color.x, 1.0);
        assert_eq!(line.color.y, 0.0);
    }

    #[test]
    fn test_debug_line_colors() {
        let red = DebugLine::red(Vec3::ZERO, Vec3::ONE);
        let green = DebugLine::green(Vec3::ZERO, Vec3::ONE);
        let blue = DebugLine::blue(Vec3::ZERO, Vec3::ONE);
        let yellow = DebugLine::yellow(Vec3::ZERO, Vec3::ONE);
        let cyan = DebugLine::cyan(Vec3::ZERO, Vec3::ONE);

        assert_eq!(red.color.x, 1.0);
        assert_eq!(green.color.y, 1.0);
        assert_eq!(blue.color.z, 1.0);
        assert_eq!(yellow.color.x, 1.0);
        assert_eq!(cyan.color.y, 1.0);
    }

    #[test]
    fn test_debug_point_from_particle() {
        let foam = DebugPoint::from_particle(Vec3::ZERO, ParticleDebugType::Foam);
        let bubble = DebugPoint::from_particle(Vec3::ZERO, ParticleDebugType::Bubble);

        // Foam should be white-ish
        assert!(foam.color.x > 0.9);
        // Bubble should be blue-ish
        assert!(bubble.color.z > 0.5);
    }

    #[test]
    fn test_draw_list_creation() {
        let list = DebugDrawList::new();
        assert!(list.is_empty());
        assert_eq!(list.primitive_count(), 0);
    }

    #[test]
    fn test_draw_list_add_primitives() {
        let mut list = DebugDrawList::new();

        list.add_line(DebugLine::red(Vec3::ZERO, Vec3::ONE));
        list.add_point(DebugPoint::new(Vec3::ZERO, Vec4::ONE, 1.0));
        list.add_label(Vec3::ZERO, "test");

        assert_eq!(list.lines.len(), 1);
        assert_eq!(list.points.len(), 1);
        assert_eq!(list.labels.len(), 1);
        assert_eq!(list.primitive_count(), 2);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_draw_list_add_aabb() {
        let mut list = DebugDrawList::new();

        list.add_aabb(Vec3::ZERO, Vec3::ONE, Vec4::ONE);

        // AABB has 12 edges
        assert_eq!(list.lines.len(), 12);
    }

    #[test]
    fn test_draw_list_add_grid() {
        let mut list = DebugDrawList::new();

        list.add_grid(Vec3::ZERO, 10.0, 5, Vec4::ONE);

        // 5 divisions = 6 lines each direction = 12 lines
        assert_eq!(list.lines.len(), 12);
    }

    #[test]
    fn test_draw_list_add_circle() {
        let mut list = DebugDrawList::new();

        list.add_circle(Vec3::ZERO, 5.0, 16, Vec4::ONE);

        assert_eq!(list.lines.len(), 16);
    }

    #[test]
    fn test_draw_list_add_velocity() {
        let mut list = DebugDrawList::new();

        // Zero velocity should not add line
        list.add_velocity(Vec3::ZERO, Vec3::ZERO, 1.0);
        assert_eq!(list.lines.len(), 0);

        // Non-zero velocity should add line
        list.add_velocity(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), 1.0);
        assert_eq!(list.lines.len(), 1);
    }

    #[test]
    fn test_draw_list_clear() {
        let mut list = DebugDrawList::new();

        list.add_line(DebugLine::red(Vec3::ZERO, Vec3::ONE));
        list.add_point(DebugPoint::new(Vec3::ZERO, Vec4::ONE, 1.0));

        list.clear();

        assert!(list.is_empty());
    }

    #[test]
    fn test_stats_formatter_time() {
        assert_eq!(StatsFormatter::format_time_us(500), "500 µs");
        assert_eq!(StatsFormatter::format_time_us(1500), "1.50 ms");
        assert_eq!(StatsFormatter::format_time_us(1_500_000), "1.50 s");
    }

    #[test]
    fn test_stats_formatter_count() {
        assert_eq!(StatsFormatter::format_count(500), "500");
        assert_eq!(StatsFormatter::format_count(1500), "1.5K");
        assert_eq!(StatsFormatter::format_count(1_500_000), "1.50M");
    }

    #[test]
    fn test_stats_formatter_percent() {
        assert_eq!(StatsFormatter::format_percent(50.0), "50.0%");
        assert_eq!(StatsFormatter::format_percent(33.33), "33.3%");
    }

    #[test]
    fn test_debug_vertex_from_line() {
        let line = DebugLine::red(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0));

        let start = DebugVertex::from_line_start(&line);
        let end = DebugVertex::from_line_end(&line);

        assert_eq!(start.position, [1.0, 2.0, 3.0]);
        assert_eq!(end.position, [4.0, 5.0, 6.0]);
        assert_eq!(start.color[0], 1.0); // Red
    }

    #[test]
    fn test_debug_vertex_from_point() {
        let point = DebugPoint::new(Vec3::new(1.0, 2.0, 3.0), Vec4::new(0.5, 0.6, 0.7, 0.8), 2.0);

        let vertex = DebugVertex::from_point(&point);

        assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(vertex.color, [0.5, 0.6, 0.7, 0.8]);
    }

    #[test]
    fn test_debug_vertex_size() {
        // Should be 28 bytes (3 floats position + 4 floats color)
        assert_eq!(std::mem::size_of::<DebugVertex>(), 28);
    }

    #[test]
    fn test_with_capacity() {
        let list = DebugDrawList::with_capacity(100, 50);
        assert!(list.lines.capacity() >= 100);
        assert!(list.points.capacity() >= 50);
    }
}
