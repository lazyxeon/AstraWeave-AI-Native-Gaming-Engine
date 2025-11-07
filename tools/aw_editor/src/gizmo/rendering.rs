//! Gizmo rendering (3D arrows, circles, cubes).
//!
//! Supports:
//! - Translation arrows (RGB = XYZ, 3D line geometry)
//! - Rotation circles (3 axis circles, torus geometry)
//! - Scale cubes (3D boxes on axes)
//! - Depth testing (proper Z-ordering)
//! - Billboarded labels ("X", "Y", "Z")
//! - Color coding (Red = X, Green = Y, Blue = Z)

use super::{AxisConstraint, GizmoMode};
use glam::{Mat4, Quat, Vec3};

/// RGB color constants for axis visualization
pub const COLOR_X: [f32; 3] = [1.0, 0.0, 0.0]; // Red
pub const COLOR_Y: [f32; 3] = [0.0, 1.0, 0.0]; // Green
pub const COLOR_Z: [f32; 3] = [0.0, 0.0, 1.0]; // Blue
pub const COLOR_HIGHLIGHT: [f32; 3] = [1.0, 1.0, 0.0]; // Yellow (hover/selected)
pub const COLOR_GRAY: [f32; 3] = [0.5, 0.5, 0.5]; // Gray (inactive)

/// Gizmo rendering parameters
#[derive(Clone, Debug)]
pub struct GizmoRenderParams {
    /// Gizmo world position (center)
    pub position: Vec3,

    /// Gizmo orientation
    pub rotation: Quat,

    /// Scale factor (scales gizmo size)
    pub scale: f32,

    /// Camera position (for billboarding)
    pub camera_pos: Vec3,

    /// View-projection matrix
    pub view_proj: Mat4,

    /// Current gizmo mode
    pub mode: GizmoMode,

    /// Active constraint (highlights specific axis)
    pub constraint: AxisConstraint,

    /// Hovered axis (for hover feedback)
    pub hovered_axis: Option<AxisConstraint>,
}

/// Gizmo geometry generator
pub struct GizmoRenderer;

impl GizmoRenderer {
    /// Generate translation arrow geometry (single axis)
    ///
    /// # Returns
    /// Vertex positions for a 3D arrow (cone + cylinder)
    ///
    /// # Algorithm
    /// 1. Cylinder shaft (origin → tip - cone_height)
    /// 2. Cone head (tip - cone_height → tip)
    /// 3. Total length: `length`, cone: 20% of length
    pub fn generate_arrow(axis: Vec3, length: f32) -> Vec<Vec3> {
        let cone_height = length * 0.2;
        let shaft_length = length - cone_height;
        let shaft_radius = length * 0.02; // 2% of length
        let _cone_radius = shaft_radius * 2.5;

        let mut vertices = Vec::new();

        // Shaft (simplified as line for now)
        vertices.push(Vec3::ZERO);
        vertices.push(axis * shaft_length);

        // Cone (simplified as line to tip)
        vertices.push(axis * shaft_length);
        vertices.push(axis * length);

        vertices
    }

    /// Generate rotation circle geometry (single axis)
    ///
    /// # Returns
    /// Vertex positions for a torus/circle
    ///
    /// # Algorithm
    /// Generate circle perpendicular to axis with `segments` points
    pub fn generate_circle(axis: Vec3, radius: f32, segments: usize) -> Vec<Vec3> {
        let mut vertices = Vec::new();

        // Find two perpendicular vectors to axis
        let perpendicular = if axis.x.abs() < 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };

        let u = axis.cross(perpendicular).normalize();
        let v = axis.cross(u).normalize();

        // Generate circle points
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos();
            let y = angle.sin();
            let point = u * x * radius + v * y * radius;
            vertices.push(point);
        }

        vertices
    }

    /// Generate scale cube geometry (single axis)
    ///
    /// # Returns
    /// Vertex positions for a cube at the end of an axis
    ///
    /// # Algorithm
    /// Place cube at `axis * offset` with size `size`
    pub fn generate_scale_cube(axis: Vec3, offset: f32, size: f32) -> Vec<Vec3> {
        let center = axis * offset;
        let half_size = size * 0.5;

        // 8 cube vertices (simplified)
        vec![
            center + Vec3::new(-half_size, -half_size, -half_size),
            center + Vec3::new(half_size, -half_size, -half_size),
            center + Vec3::new(half_size, half_size, -half_size),
            center + Vec3::new(-half_size, half_size, -half_size),
            center + Vec3::new(-half_size, -half_size, half_size),
            center + Vec3::new(half_size, -half_size, half_size),
            center + Vec3::new(half_size, half_size, half_size),
            center + Vec3::new(-half_size, half_size, half_size),
        ]
    }

    /// Render translation gizmo (3 arrows for X/Y/Z)
    ///
    /// # Returns
    /// List of (vertices, color, highlighted) tuples for each axis
    pub fn render_translation(params: &GizmoRenderParams) -> Vec<(Vec<Vec3>, [f32; 3], bool)> {
        let length = params.scale;
        let mut geometries = Vec::new();

        // X-axis arrow (Red)
        let x_highlighted = matches!(params.constraint, AxisConstraint::X)
            || matches!(params.hovered_axis, Some(AxisConstraint::X));
        let x_color = if x_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_X
        };
        geometries.push((
            Self::generate_arrow(Vec3::X, length),
            x_color,
            x_highlighted,
        ));

        // Y-axis arrow (Green)
        let y_highlighted = matches!(params.constraint, AxisConstraint::Y)
            || matches!(params.hovered_axis, Some(AxisConstraint::Y));
        let y_color = if y_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_Y
        };
        geometries.push((
            Self::generate_arrow(Vec3::Y, length),
            y_color,
            y_highlighted,
        ));

        // Z-axis arrow (Blue)
        let z_highlighted = matches!(params.constraint, AxisConstraint::Z)
            || matches!(params.hovered_axis, Some(AxisConstraint::Z));
        let z_color = if z_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_Z
        };
        geometries.push((
            Self::generate_arrow(Vec3::Z, length),
            z_color,
            z_highlighted,
        ));

        geometries
    }

    /// Render rotation gizmo (3 circles for X/Y/Z)
    ///
    /// # Returns
    /// List of (vertices, color, highlighted) tuples for each axis
    pub fn render_rotation(params: &GizmoRenderParams) -> Vec<(Vec<Vec3>, [f32; 3], bool)> {
        let radius = params.scale;
        let segments = 64; // Circle smoothness
        let mut geometries = Vec::new();

        // X-axis circle (Red, YZ plane)
        let x_highlighted = matches!(params.constraint, AxisConstraint::X)
            || matches!(params.hovered_axis, Some(AxisConstraint::X));
        let x_color = if x_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_X
        };
        geometries.push((
            Self::generate_circle(Vec3::X, radius, segments),
            x_color,
            x_highlighted,
        ));

        // Y-axis circle (Green, XZ plane)
        let y_highlighted = matches!(params.constraint, AxisConstraint::Y)
            || matches!(params.hovered_axis, Some(AxisConstraint::Y));
        let y_color = if y_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_Y
        };
        geometries.push((
            Self::generate_circle(Vec3::Y, radius, segments),
            y_color,
            y_highlighted,
        ));

        // Z-axis circle (Blue, XY plane)
        let z_highlighted = matches!(params.constraint, AxisConstraint::Z)
            || matches!(params.hovered_axis, Some(AxisConstraint::Z));
        let z_color = if z_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_Z
        };
        geometries.push((
            Self::generate_circle(Vec3::Z, radius, segments),
            z_color,
            z_highlighted,
        ));

        geometries
    }

    /// Render scale gizmo (3 cubes for X/Y/Z)
    ///
    /// # Returns
    /// List of (vertices, color, highlighted) tuples for each axis
    pub fn render_scale(params: &GizmoRenderParams) -> Vec<(Vec<Vec3>, [f32; 3], bool)> {
        let offset = params.scale * 0.8; // Cube position along axis
        let size = params.scale * 0.15; // Cube size
        let mut geometries = Vec::new();

        // X-axis cube (Red)
        let x_highlighted = matches!(params.constraint, AxisConstraint::X)
            || matches!(params.hovered_axis, Some(AxisConstraint::X));
        let x_color = if x_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_X
        };
        geometries.push((
            Self::generate_scale_cube(Vec3::X, offset, size),
            x_color,
            x_highlighted,
        ));

        // Y-axis cube (Green)
        let y_highlighted = matches!(params.constraint, AxisConstraint::Y)
            || matches!(params.hovered_axis, Some(AxisConstraint::Y));
        let y_color = if y_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_Y
        };
        geometries.push((
            Self::generate_scale_cube(Vec3::Y, offset, size),
            y_color,
            y_highlighted,
        ));

        // Z-axis cube (Blue)
        let z_highlighted = matches!(params.constraint, AxisConstraint::Z)
            || matches!(params.hovered_axis, Some(AxisConstraint::Z));
        let z_color = if z_highlighted {
            COLOR_HIGHLIGHT
        } else {
            COLOR_Z
        };
        geometries.push((
            Self::generate_scale_cube(Vec3::Z, offset, size),
            z_color,
            z_highlighted,
        ));

        geometries
    }

    /// Transform vertices from local space to world space
    ///
    /// # Algorithm
    /// `world_pos = position + rotation * (vertex * scale)`
    pub fn transform_vertices(vertices: &[Vec3], position: Vec3, rotation: Quat) -> Vec<Vec3> {
        vertices.iter().map(|&v| position + rotation * v).collect()
    }

    /// Generate billboarded label position
    ///
    /// # Returns
    /// Screen-space position for axis label ("X", "Y", "Z")
    ///
    /// # Algorithm
    /// Place label at end of axis, always facing camera
    pub fn generate_label_position(axis: Vec3, params: &GizmoRenderParams) -> Vec3 {
        let offset = params.scale * 1.1; // Slightly beyond gizmo
        params.position + params.rotation * (axis * offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_generation() {
        let vertices = GizmoRenderer::generate_arrow(Vec3::X, 1.0);

        // Should have 4 vertices (2 for shaft, 2 for cone)
        assert_eq!(vertices.len(), 4);

        // First vertex at origin
        assert_eq!(vertices[0], Vec3::ZERO);

        // Last vertex at tip
        assert!((vertices[3] - Vec3::X).length() < 0.001);
    }

    #[test]
    fn test_circle_generation() {
        let vertices = GizmoRenderer::generate_circle(Vec3::Y, 1.0, 32);

        // Should have 33 vertices (32 + 1 to close loop)
        assert_eq!(vertices.len(), 33);

        // All vertices should be ~1.0 units from origin
        for v in &vertices {
            let distance = v.length();
            assert!((distance - 1.0).abs() < 0.01, "Distance: {}", distance);
        }
    }

    #[test]
    fn test_cube_generation() {
        let vertices = GizmoRenderer::generate_scale_cube(Vec3::Z, 1.0, 0.2);

        // Should have 8 vertices (cube corners)
        assert_eq!(vertices.len(), 8);

        // Center should be at (0, 0, 1)
        let center = vertices.iter().fold(Vec3::ZERO, |acc, &v| acc + v) / 8.0;
        assert!((center - Vec3::new(0.0, 0.0, 1.0)).length() < 0.001);
    }

    #[test]
    fn test_translation_render() {
        let params = GizmoRenderParams {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            camera_pos: Vec3::new(0.0, 0.0, 5.0),
            view_proj: Mat4::IDENTITY,
            mode: GizmoMode::Translate {
                constraint: AxisConstraint::None,
            },
            constraint: AxisConstraint::None,
            hovered_axis: None,
        };

        let geometries = GizmoRenderer::render_translation(&params);

        // Should have 3 arrows (X, Y, Z)
        assert_eq!(geometries.len(), 3);

        // Check colors (RGB for XYZ)
        assert_eq!(geometries[0].1, COLOR_X); // Red
        assert_eq!(geometries[1].1, COLOR_Y); // Green
        assert_eq!(geometries[2].1, COLOR_Z); // Blue
    }

    #[test]
    fn test_rotation_render() {
        let params = GizmoRenderParams {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            camera_pos: Vec3::new(0.0, 0.0, 5.0),
            view_proj: Mat4::IDENTITY,
            mode: GizmoMode::Rotate {
                constraint: AxisConstraint::None,
            },
            constraint: AxisConstraint::None,
            hovered_axis: None,
        };

        let geometries = GizmoRenderer::render_rotation(&params);

        // Should have 3 circles (X, Y, Z)
        assert_eq!(geometries.len(), 3);

        // Check colors
        assert_eq!(geometries[0].1, COLOR_X);
        assert_eq!(geometries[1].1, COLOR_Y);
        assert_eq!(geometries[2].1, COLOR_Z);
    }

    #[test]
    fn test_scale_render() {
        let params = GizmoRenderParams {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            camera_pos: Vec3::new(0.0, 0.0, 5.0),
            view_proj: Mat4::IDENTITY,
            mode: GizmoMode::Scale {
                constraint: AxisConstraint::None,
                uniform: false,
            },
            constraint: AxisConstraint::None,
            hovered_axis: None,
        };

        let geometries = GizmoRenderer::render_scale(&params);

        // Should have 3 cubes (X, Y, Z)
        assert_eq!(geometries.len(), 3);

        // Check colors
        assert_eq!(geometries[0].1, COLOR_X);
        assert_eq!(geometries[1].1, COLOR_Y);
        assert_eq!(geometries[2].1, COLOR_Z);
    }

    #[test]
    fn test_highlight_on_constraint() {
        let params = GizmoRenderParams {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: 1.0,
            camera_pos: Vec3::new(0.0, 0.0, 5.0),
            view_proj: Mat4::IDENTITY,
            mode: GizmoMode::Translate {
                constraint: AxisConstraint::X,
            },
            constraint: AxisConstraint::X, // X-axis constrained
            hovered_axis: None,
        };

        let geometries = GizmoRenderer::render_translation(&params);

        // X-axis should be highlighted (yellow)
        assert_eq!(geometries[0].1, COLOR_HIGHLIGHT);
        assert!(geometries[0].2); // highlighted flag

        // Y and Z should be normal colors
        assert_eq!(geometries[1].1, COLOR_Y);
        assert_eq!(geometries[2].1, COLOR_Z);
    }

    #[test]
    fn test_transform_vertices() {
        let vertices = vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];

        let position = Vec3::new(5.0, 5.0, 5.0);
        let rotation = Quat::IDENTITY;

        let transformed = GizmoRenderer::transform_vertices(&vertices, position, rotation);

        // Should be offset by position
        assert_eq!(transformed[0], Vec3::new(6.0, 5.0, 5.0));
        assert_eq!(transformed[1], Vec3::new(5.0, 6.0, 5.0));
    }
}
