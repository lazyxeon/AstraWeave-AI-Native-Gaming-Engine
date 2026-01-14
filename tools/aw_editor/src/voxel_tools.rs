//! Voxel Editing Tools for AstraWeave Editor
//!
//! This module provides interactive voxel editing capabilities including
//! brushes for adding/removing voxels, undo/redo, and real-time preview.

use astraweave_terrain::{ChunkCoord, MaterialId, Voxel, VoxelGrid};
use glam::Vec3;
use std::collections::VecDeque;

/// Brush shape for voxel editing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushShape {
    /// Spherical brush
    Sphere,
    /// Cubic brush
    Cube,
    /// Cylindrical brush
    Cylinder,
}

/// Brush mode (add or remove voxels)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushMode {
    /// Add voxels (increase density)
    Add,
    /// Remove voxels (decrease density)
    Remove,
    /// Paint material without changing density
    Paint,
}

/// Brush configuration
#[derive(Debug, Clone, Copy)]
pub struct BrushConfig {
    /// Brush shape
    pub shape: BrushShape,
    /// Brush mode
    pub mode: BrushMode,
    /// Brush radius in world units
    pub radius: f32,
    /// Brush strength (0.0 to 1.0)
    pub strength: f32,
    /// Material ID to apply
    pub material: MaterialId,
    /// Smooth edges
    pub smooth: bool,
}

impl Default for BrushConfig {
    fn default() -> Self {
        Self {
            shape: BrushShape::Sphere,
            mode: BrushMode::Add,
            radius: 5.0,
            strength: 1.0,
            material: 1,
            smooth: true,
        }
    }
}

/// A single edit operation for undo/redo
#[derive(Debug, Clone)]
struct EditOperation {
    /// Position of the edit
    position: Vec3,
    /// Affected voxels before the edit
    before: Vec<(Vec3, Voxel)>,
    /// Affected voxels after the edit
    after: Vec<(Vec3, Voxel)>,
}

/// Voxel editor with brush tools and undo/redo
pub struct VoxelEditor {
    /// Current brush configuration
    brush: BrushConfig,
    /// Undo stack
    undo_stack: VecDeque<EditOperation>,
    /// Redo stack
    redo_stack: VecDeque<EditOperation>,
    /// Maximum undo history
    max_undo_history: usize,
    /// Preview mode (don't commit changes)
    preview_mode: bool,
}

impl VoxelEditor {
    /// Create a new voxel editor
    pub fn new() -> Self {
        Self {
            brush: BrushConfig::default(),
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_undo_history: 100,
            preview_mode: false,
        }
    }

    /// Set brush configuration
    pub fn set_brush(&mut self, brush: BrushConfig) {
        self.brush = brush;
    }

    /// Get current brush configuration
    pub fn brush(&self) -> &BrushConfig {
        &self.brush
    }

    /// Apply brush at world position
    pub fn apply_brush(&mut self, grid: &mut VoxelGrid, position: Vec3) -> Vec<ChunkCoord> {
        let mut operation = EditOperation {
            position,
            before: Vec::new(),
            after: Vec::new(),
        };

        let mut affected_chunks = Vec::new();

        // Get voxels in brush radius
        let voxels = self.get_voxels_in_brush(position);

        for voxel_pos in voxels {
            // Store before state
            if let Some(voxel) = grid.get_voxel(voxel_pos) {
                operation.before.push((voxel_pos, voxel));
            } else {
                operation.before.push((voxel_pos, Voxel::default()));
            }

            // Apply brush effect
            let new_voxel = self.apply_brush_to_voxel(
                grid.get_voxel(voxel_pos).unwrap_or_default(),
                voxel_pos,
                position,
            );

            // Set new voxel
            grid.set_voxel(voxel_pos, new_voxel);
            operation.after.push((voxel_pos, new_voxel));

            // Track affected chunk
            let chunk_coord = ChunkCoord::from_world_pos(voxel_pos);
            if !affected_chunks.contains(&chunk_coord) {
                affected_chunks.push(chunk_coord);
            }
        }

        // Add to undo stack if not in preview mode
        if !self.preview_mode {
            self.undo_stack.push_back(operation);
            if self.undo_stack.len() > self.max_undo_history {
                self.undo_stack.pop_front();
            }
            self.redo_stack.clear();
        }

        affected_chunks
    }

    /// Get all voxel positions within brush radius
    fn get_voxels_in_brush(&self, center: Vec3) -> Vec<Vec3> {
        let mut voxels = Vec::new();
        let radius_int = self.brush.radius.ceil() as i32;

        for x in -radius_int..=radius_int {
            for y in -radius_int..=radius_int {
                for z in -radius_int..=radius_int {
                    let offset = Vec3::new(x as f32, y as f32, z as f32);
                    let pos = center + offset;

                    if self.is_in_brush(pos, center) {
                        voxels.push(pos);
                    }
                }
            }
        }

        voxels
    }

    /// Check if position is within brush shape
    fn is_in_brush(&self, pos: Vec3, center: Vec3) -> bool {
        let offset = pos - center;

        match self.brush.shape {
            BrushShape::Sphere => offset.length() <= self.brush.radius,
            BrushShape::Cube => {
                offset.x.abs() <= self.brush.radius
                    && offset.y.abs() <= self.brush.radius
                    && offset.z.abs() <= self.brush.radius
            }
            BrushShape::Cylinder => {
                let horizontal_dist = (offset.x * offset.x + offset.z * offset.z).sqrt();
                horizontal_dist <= self.brush.radius && offset.y.abs() <= self.brush.radius
            }
        }
    }

    /// Apply brush effect to a single voxel
    fn apply_brush_to_voxel(&self, mut voxel: Voxel, pos: Vec3, center: Vec3) -> Voxel {
        let distance = (pos - center).length();
        let falloff = if self.brush.smooth {
            (1.0 - distance / self.brush.radius).max(0.0)
        } else {
            1.0
        };

        let strength = self.brush.strength * falloff;

        match self.brush.mode {
            BrushMode::Add => {
                voxel.density = (voxel.density + strength).min(1.0);
                if voxel.density > 0.5 {
                    voxel.material = self.brush.material;
                }
            }
            BrushMode::Remove => {
                voxel.density = (voxel.density - strength).max(0.0);
            }
            BrushMode::Paint => {
                if voxel.density > 0.5 {
                    voxel.material = self.brush.material;
                }
            }
        }

        voxel
    }

    /// Undo last operation
    pub fn undo(&mut self, grid: &mut VoxelGrid) -> Option<Vec<ChunkCoord>> {
        if let Some(operation) = self.undo_stack.pop_back() {
            let mut affected_chunks = Vec::new();

            // Restore before state
            for (pos, voxel) in &operation.before {
                grid.set_voxel(*pos, *voxel);
                let chunk_coord = ChunkCoord::from_world_pos(*pos);
                if !affected_chunks.contains(&chunk_coord) {
                    affected_chunks.push(chunk_coord);
                }
            }

            // Add to redo stack
            self.redo_stack.push_back(operation);

            Some(affected_chunks)
        } else {
            None
        }
    }

    /// Redo last undone operation
    pub fn redo(&mut self, grid: &mut VoxelGrid) -> Option<Vec<ChunkCoord>> {
        if let Some(operation) = self.redo_stack.pop_back() {
            let mut affected_chunks = Vec::new();

            // Restore after state
            for (pos, voxel) in &operation.after {
                grid.set_voxel(*pos, *voxel);
                let chunk_coord = ChunkCoord::from_world_pos(*pos);
                if !affected_chunks.contains(&chunk_coord) {
                    affected_chunks.push(chunk_coord);
                }
            }

            // Add back to undo stack
            self.undo_stack.push_back(operation);

            Some(affected_chunks)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear undo/redo history
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Enable/disable preview mode
    pub fn set_preview_mode(&mut self, enabled: bool) {
        self.preview_mode = enabled;
    }

    /// Check if in preview mode
    pub fn is_preview_mode(&self) -> bool {
        self.preview_mode
    }
}

impl Default for VoxelEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// Voxel raycasting for editor interaction
pub struct VoxelRaycaster {
    /// Maximum ray distance
    max_distance: f32,
    /// Step size for ray marching
    step_size: f32,
}

impl VoxelRaycaster {
    /// Create a new voxel raycaster
    pub fn new(max_distance: f32) -> Self {
        Self {
            max_distance,
            step_size: 0.5,
        }
    }

    /// Cast a ray and find the first solid voxel
    pub fn raycast(&self, grid: &VoxelGrid, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        let direction = direction.normalize();
        let mut current_pos = origin;
        let mut distance = 0.0;

        while distance < self.max_distance {
            if let Some(voxel) = grid.get_voxel(current_pos) {
                if voxel.is_solid() {
                    return Some(current_pos);
                }
            }

            current_pos += direction * self.step_size;
            distance += self.step_size;
        }

        None
    }

    /// Cast a ray and find the first empty voxel adjacent to a solid voxel
    pub fn raycast_surface(&self, grid: &VoxelGrid, origin: Vec3, direction: Vec3) -> Option<Vec3> {
        let direction = direction.normalize();
        let mut current_pos = origin;
        let mut distance = 0.0;
        let mut last_pos = origin;

        while distance < self.max_distance {
            if let Some(voxel) = grid.get_voxel(current_pos) {
                if voxel.is_solid() {
                    // Return the last empty position before hitting solid
                    return Some(last_pos);
                }
            }

            last_pos = current_pos;
            current_pos += direction * self.step_size;
            distance += self.step_size;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brush_config_default() {
        let brush = BrushConfig::default();
        assert_eq!(brush.shape, BrushShape::Sphere);
        assert_eq!(brush.mode, BrushMode::Add);
        assert_eq!(brush.radius, 5.0);
    }

    #[test]
    fn test_voxel_editor_creation() {
        let editor = VoxelEditor::new();
        assert!(!editor.can_undo());
        assert!(!editor.can_redo());
    }

    #[test]
    fn test_brush_sphere_shape() {
        let editor = VoxelEditor::new();
        let center = Vec3::new(0.0, 0.0, 0.0);

        assert!(editor.is_in_brush(Vec3::new(0.0, 0.0, 0.0), center));
        assert!(editor.is_in_brush(Vec3::new(3.0, 0.0, 0.0), center));
        assert!(!editor.is_in_brush(Vec3::new(10.0, 0.0, 0.0), center));
    }

    #[test]
    fn test_undo_redo() {
        let mut editor = VoxelEditor::new();
        let mut grid = VoxelGrid::new();

        let pos = Vec3::new(10.0, 10.0, 10.0);
        editor.apply_brush(&mut grid, pos);

        assert!(editor.can_undo());
        assert!(!editor.can_redo());

        editor.undo(&mut grid);
        assert!(!editor.can_undo());
        assert!(editor.can_redo());

        editor.redo(&mut grid);
        assert!(editor.can_undo());
        assert!(!editor.can_redo());
    }

    #[test]
    fn test_raycaster() {
        let raycaster = VoxelRaycaster::new(100.0);
        let mut grid = VoxelGrid::new();

        // Add a solid voxel
        grid.set_voxel(Vec3::new(10.0, 0.0, 0.0), Voxel::new(1.0, 1));

        let origin = Vec3::new(0.0, 0.0, 0.0);
        let direction = Vec3::new(1.0, 0.0, 0.0);

        let hit = raycaster.raycast(&grid, origin, direction);
        assert!(hit.is_some());
    }
}
