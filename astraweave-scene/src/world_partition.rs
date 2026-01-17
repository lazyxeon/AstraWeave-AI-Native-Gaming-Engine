//! World Partition System for AstraWeave
//!
//! This module implements a grid-based spatial partitioning system for large open worlds.
//! It enables streaming of scene content based on camera position, keeping memory usage bounded.
//!
//! # Architecture
//!
//! ```text
//! WorldPartition
//! ├── Grid (HashMap<GridCoord, Cell>)
//! │   └── Cell
//! │       ├── Entities (Vec<Entity>)
//! │       ├── Assets (Vec<AssetRef>)
//! │       └── State (Unloaded/Loading/Loaded)
//! └── WorldPartitionManager
//!     ├── Active Cells (based on camera frustum)
//!     ├── LRU Cache (recently unloaded cells)
//!     └── Async Loader (tokio tasks)
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use astraweave_scene::world_partition::{WorldPartition, GridConfig};
//! use glam::Vec3;
//!
//! // Create a world partition with 100m cells
//! let config = GridConfig {
//!     cell_size: 100.0,
//!     world_bounds: (-5000.0, 5000.0, -5000.0, 5000.0), // 10km x 10km
//! };
//! let mut partition = WorldPartition::new(config);
//!
//! // Assign an entity to a cell based on its position
//! let entity_pos = Vec3::new(150.0, 0.0, 250.0);
//! partition.assign_entity_to_cell(entity_id, entity_pos);
//! ```

use astraweave_asset::cell_loader::{CellMetadata, ComponentData as CellComponentData};
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

// Entity is just a u64 ID
pub type Entity = u64;

/// Grid coordinate in 3D space (i32 for signed coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl GridCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    /// Convert world position to grid coordinate
    pub fn from_world_pos(pos: Vec3, cell_size: f32) -> Self {
        Self {
            x: (pos.x / cell_size).floor() as i32,
            y: (pos.y / cell_size).floor() as i32,
            z: (pos.z / cell_size).floor() as i32,
        }
    }

    /// Get world-space center of this cell
    pub fn to_world_center(self, cell_size: f32) -> Vec3 {
        Vec3::new(
            (self.x as f32 + 0.5) * cell_size,
            (self.y as f32 + 0.5) * cell_size,
            (self.z as f32 + 0.5) * cell_size,
        )
    }

    /// Get all neighboring cells (26 neighbors in 3D, or 8 in 2D if y=0)
    pub fn neighbors_3d(self) -> Vec<GridCoord> {
        let mut neighbors = Vec::with_capacity(26);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    neighbors.push(GridCoord::new(self.x + dx, self.y + dy, self.z + dz));
                }
            }
        }
        neighbors
    }

    /// Get 2D neighbors (8 neighbors, ignoring y-axis)
    pub fn neighbors_2d(self) -> Vec<GridCoord> {
        let mut neighbors = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 {
                    continue;
                }
                neighbors.push(GridCoord::new(self.x + dx, self.y, self.z + dz));
            }
        }
        neighbors
    }

    /// Manhattan distance to another cell
    pub fn manhattan_distance(self, other: GridCoord) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }
}

/// Axis-Aligned Bounding Box for spatial queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create AABB from center and half-extents
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Get center point
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get half-extents (size / 2)
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Check if point is inside AABB
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Check if this AABB intersects another
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Get all grid cells that this AABB overlaps
    pub fn overlapping_cells(&self, cell_size: f32) -> Vec<GridCoord> {
        let min_coord = GridCoord::from_world_pos(self.min, cell_size);
        let max_coord = GridCoord::from_world_pos(self.max, cell_size);

        let mut cells = Vec::new();
        for x in min_coord.x..=max_coord.x {
            for y in min_coord.y..=max_coord.y {
                for z in min_coord.z..=max_coord.z {
                    cells.push(GridCoord::new(x, y, z));
                }
            }
        }
        cells
    }
}

/// Camera frustum for culling
#[derive(Debug, Clone)]
pub struct Frustum {
    /// Six frustum planes (left, right, bottom, top, near, far)
    /// Each plane is represented as Vec4(a, b, c, d) where ax + by + cz + d = 0
    pub planes: [Vec4; 6],
}

impl Frustum {
    /// Create frustum from view-projection matrix
    pub fn from_view_projection(view_proj: glam::Mat4) -> Self {
        let mut planes = [Vec4::ZERO; 6];

        // Extract frustum planes from view-projection matrix
        // Left plane
        planes[0] = Vec4::new(
            view_proj.x_axis.w + view_proj.x_axis.x,
            view_proj.y_axis.w + view_proj.y_axis.x,
            view_proj.z_axis.w + view_proj.z_axis.x,
            view_proj.w_axis.w + view_proj.w_axis.x,
        );

        // Right plane
        planes[1] = Vec4::new(
            view_proj.x_axis.w - view_proj.x_axis.x,
            view_proj.y_axis.w - view_proj.y_axis.x,
            view_proj.z_axis.w - view_proj.z_axis.x,
            view_proj.w_axis.w - view_proj.w_axis.x,
        );

        // Bottom plane
        planes[2] = Vec4::new(
            view_proj.x_axis.w + view_proj.x_axis.y,
            view_proj.y_axis.w + view_proj.y_axis.y,
            view_proj.z_axis.w + view_proj.z_axis.y,
            view_proj.w_axis.w + view_proj.w_axis.y,
        );

        // Top plane
        planes[3] = Vec4::new(
            view_proj.x_axis.w - view_proj.x_axis.y,
            view_proj.y_axis.w - view_proj.y_axis.y,
            view_proj.z_axis.w - view_proj.z_axis.y,
            view_proj.w_axis.w - view_proj.w_axis.y,
        );

        // Near plane
        planes[4] = Vec4::new(
            view_proj.x_axis.w + view_proj.x_axis.z,
            view_proj.y_axis.w + view_proj.y_axis.z,
            view_proj.z_axis.w + view_proj.z_axis.z,
            view_proj.w_axis.w + view_proj.w_axis.z,
        );

        // Far plane
        planes[5] = Vec4::new(
            view_proj.x_axis.w - view_proj.x_axis.z,
            view_proj.y_axis.w - view_proj.y_axis.z,
            view_proj.z_axis.w - view_proj.z_axis.z,
            view_proj.w_axis.w - view_proj.w_axis.z,
        );

        // Normalize planes
        for plane in &mut planes {
            let length = Vec3::new(plane.x, plane.y, plane.z).length();
            *plane /= length;
        }

        Self { planes }
    }

    /// Test if AABB is inside or intersecting frustum
    pub fn intersects_aabb(&self, aabb: &AABB) -> bool {
        for plane in &self.planes {
            let normal = Vec3::new(plane.x, plane.y, plane.z);
            let d = plane.w;

            // Get positive vertex (furthest point in direction of plane normal)
            let p = Vec3::new(
                if normal.x >= 0.0 {
                    aabb.max.x
                } else {
                    aabb.min.x
                },
                if normal.y >= 0.0 {
                    aabb.max.y
                } else {
                    aabb.min.y
                },
                if normal.z >= 0.0 {
                    aabb.max.z
                } else {
                    aabb.min.z
                },
            );

            // If positive vertex is outside plane, AABB is completely outside
            if normal.dot(p) + d < 0.0 {
                return false;
            }
        }
        true
    }

    /// Get cells within frustum (simplified: use sphere around camera)
    pub fn cells_in_frustum(
        &self,
        camera_pos: Vec3,
        cell_size: f32,
        radius: f32,
    ) -> Vec<GridCoord> {
        let camera_cell = GridCoord::from_world_pos(camera_pos, cell_size);
        let radius_cells = (radius / cell_size).ceil() as i32;

        let mut cells = Vec::new();
        for dx in -radius_cells..=radius_cells {
            for dy in -radius_cells..=radius_cells {
                for dz in -radius_cells..=radius_cells {
                    let coord =
                        GridCoord::new(camera_cell.x + dx, camera_cell.y + dy, camera_cell.z + dz);

                    // Check if cell AABB intersects frustum
                    let cell_center = coord.to_world_center(cell_size);
                    let cell_half_size = Vec3::splat(cell_size * 0.5);
                    let cell_aabb = AABB::from_center_half_extents(cell_center, cell_half_size);

                    if self.intersects_aabb(&cell_aabb) {
                        cells.push(coord);
                    }
                }
            }
        }
        cells
    }
}

/// Asset reference for streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRef {
    pub path: String,
    pub asset_type: AssetType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Audio,
    Other,
}

/// Cell state for streaming
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellState {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
}

/// A single cell in the world partition grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub coord: GridCoord,
    pub state: CellState,
    pub entities: Vec<Entity>,
    pub assets: Vec<AssetRef>,
    pub bounds: AABB,
    pub entity_blueprints: Vec<CellEntityBlueprint>,
    pub metadata: Option<CellMetadata>,
}

impl Cell {
    pub fn new(coord: GridCoord, cell_size: f32) -> Self {
        let center = coord.to_world_center(cell_size);
        let half_size = Vec3::splat(cell_size * 0.5);
        let bounds = AABB::from_center_half_extents(center, half_size);

        Self {
            coord,
            state: CellState::Unloaded,
            entities: Vec::new(),
            assets: Vec::new(),
            bounds,
            entity_blueprints: Vec::new(),
            metadata: None,
        }
    }

    pub fn is_loaded(&self) -> bool {
        self.state == CellState::Loaded
    }

    pub fn is_loading(&self) -> bool {
        self.state == CellState::Loading
    }

    pub fn components_of_type<'a>(
        &'a self,
        component_type: &'a str,
    ) -> impl Iterator<Item = CellComponentView<'a>> + 'a {
        self.entity_blueprints.iter().flat_map(move |entity| {
            entity.components.iter().filter_map(move |component| {
                if component.component_type == component_type {
                    Some(CellComponentView { entity, component })
                } else {
                    None
                }
            })
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellEntityBlueprint {
    pub name: Option<String>,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
    pub components: Vec<CellComponentData>,
}

pub struct CellComponentView<'a> {
    pub entity: &'a CellEntityBlueprint,
    pub component: &'a CellComponentData,
}

/// Grid configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GridConfig {
    /// Size of each cell in world units (default: 100.0 meters)
    pub cell_size: f32,
    /// World bounds (min_x, max_x, min_z, max_z) for 2D grid
    pub world_bounds: (f32, f32, f32, f32),
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            cell_size: 100.0,
            world_bounds: (-5000.0, 5000.0, -5000.0, 5000.0), // 10km x 10km
        }
    }
}

/// World partition grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPartition {
    pub config: GridConfig,
    pub cells: HashMap<GridCoord, Cell>,
}

impl WorldPartition {
    pub fn new(config: GridConfig) -> Self {
        Self {
            config,
            cells: HashMap::new(),
        }
    }

    /// Get or create a cell at the given coordinate
    pub fn get_or_create_cell(&mut self, coord: GridCoord) -> &mut Cell {
        self.cells
            .entry(coord)
            .or_insert_with(|| Cell::new(coord, self.config.cell_size))
    }

    /// Get cell at coordinate (immutable)
    pub fn get_cell(&self, coord: GridCoord) -> Option<&Cell> {
        self.cells.get(&coord)
    }

    /// Get cell at coordinate (mutable)
    pub fn get_cell_mut(&mut self, coord: GridCoord) -> Option<&mut Cell> {
        self.cells.get_mut(&coord)
    }

    /// Assign entity to cell based on position
    pub fn assign_entity_to_cell(&mut self, entity: Entity, position: Vec3) {
        let coord = GridCoord::from_world_pos(position, self.config.cell_size);
        let cell = self.get_or_create_cell(coord);
        if !cell.entities.contains(&entity) {
            cell.entities.push(entity);
        }
    }

    /// Assign entity to cell based on AABB (can span multiple cells)
    pub fn assign_entity_to_cells_by_bounds(&mut self, entity: Entity, bounds: AABB) {
        let cells = bounds.overlapping_cells(self.config.cell_size);
        for coord in cells {
            let cell = self.get_or_create_cell(coord);
            if !cell.entities.contains(&entity) {
                cell.entities.push(entity);
            }
        }
    }

    /// Remove entity from all cells
    pub fn remove_entity(&mut self, entity: Entity) {
        for cell in self.cells.values_mut() {
            cell.entities.retain(|&e| e != entity);
        }
    }

    /// Get all loaded cells
    pub fn loaded_cells(&self) -> Vec<GridCoord> {
        self.cells
            .iter()
            .filter(|(_, cell)| cell.is_loaded())
            .map(|(coord, _)| *coord)
            .collect()
    }

    /// Get all cells within radius of a point
    pub fn cells_in_radius(&self, center: Vec3, radius: f32) -> Vec<GridCoord> {
        let center_coord = GridCoord::from_world_pos(center, self.config.cell_size);
        let radius_cells = (radius / self.config.cell_size).ceil() as i32;

        let mut cells = Vec::new();
        for dx in -radius_cells..=radius_cells {
            for dz in -radius_cells..=radius_cells {
                let coord = GridCoord::new(center_coord.x + dx, 0, center_coord.z + dz);
                let cell_center = coord.to_world_center(self.config.cell_size);
                let distance = (cell_center - center).length();
                if distance <= radius {
                    cells.push(coord);
                }
            }
        }
        cells
    }

    /// Get memory usage estimate in bytes
    pub fn memory_usage_estimate(&self) -> usize {
        let mut total = 0;
        for cell in self.cells.values() {
            total += std::mem::size_of::<Cell>();
            total += cell.entities.len() * std::mem::size_of::<u64>();
            total += cell.assets.len() * std::mem::size_of::<AssetRef>();
        }
        total
    }
}

/// LRU cache for recently unloaded cells
#[derive(Debug)]
pub struct LRUCache {
    capacity: usize,
    queue: VecDeque<GridCoord>,
}

impl LRUCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            queue: VecDeque::with_capacity(capacity),
        }
    }

    /// Mark cell as recently used
    pub fn touch(&mut self, coord: GridCoord) {
        // Remove if already in cache
        if let Some(pos) = self.queue.iter().position(|&c| c == coord) {
            self.queue.remove(pos);
        }
        // Add to front
        self.queue.push_front(coord);
        // Evict oldest if over capacity
        if self.queue.len() > self.capacity {
            self.queue.pop_back();
        }
    }

    /// Check if cell is in cache
    pub fn contains(&self, coord: GridCoord) -> bool {
        self.queue.contains(&coord)
    }

    /// Get least recently used cell
    pub fn lru(&self) -> Option<GridCoord> {
        self.queue.back().copied()
    }

    /// Remove cell from cache
    pub fn remove(&mut self, coord: GridCoord) {
        if let Some(pos) = self.queue.iter().position(|&c| c == coord) {
            self.queue.remove(pos);
        }
    }

    /// Get number of cells in cache
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};

    // ===== GridCoord Tests =====

    #[test]
    fn test_grid_coord_new() {
        let coord = GridCoord::new(1, 2, 3);
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 2);
        assert_eq!(coord.z, 3);
    }

    #[test]
    fn test_grid_coord_from_world_pos() {
        let coord = GridCoord::from_world_pos(Vec3::new(150.0, 50.0, 250.0), 100.0);
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 0);
        assert_eq!(coord.z, 2);
    }

    #[test]
    fn test_grid_coord_from_world_pos_negative() {
        let coord = GridCoord::from_world_pos(Vec3::new(-150.0, 0.0, -50.0), 100.0);
        assert_eq!(coord.x, -2);
        assert_eq!(coord.z, -1);
    }

    #[test]
    fn test_grid_coord_from_world_pos_origin() {
        let coord = GridCoord::from_world_pos(Vec3::ZERO, 100.0);
        assert_eq!(coord.x, 0);
        assert_eq!(coord.y, 0);
        assert_eq!(coord.z, 0);
    }

    #[test]
    fn test_grid_coord_to_world_center() {
        let coord = GridCoord::new(1, 0, 2);
        let center = coord.to_world_center(100.0);
        
        // Center of cell (1, 0, 2) with cell_size 100 should be (150, 50, 250)
        assert!((center - Vec3::new(150.0, 50.0, 250.0)).length() < 0.0001);
    }

    #[test]
    fn test_grid_coord_neighbors_3d() {
        let coord = GridCoord::new(0, 0, 0);
        let neighbors = coord.neighbors_3d();
        
        assert_eq!(neighbors.len(), 26); // 3x3x3 - 1 (self)
        
        // Should not include origin
        assert!(!neighbors.contains(&GridCoord::new(0, 0, 0)));
        
        // Should include all direct neighbors
        assert!(neighbors.contains(&GridCoord::new(1, 0, 0)));
        assert!(neighbors.contains(&GridCoord::new(-1, 0, 0)));
        assert!(neighbors.contains(&GridCoord::new(0, 1, 0)));
        assert!(neighbors.contains(&GridCoord::new(0, -1, 0)));
        assert!(neighbors.contains(&GridCoord::new(0, 0, 1)));
        assert!(neighbors.contains(&GridCoord::new(0, 0, -1)));
    }

    #[test]
    fn test_grid_coord_neighbors_2d() {
        let coord = GridCoord::new(5, 0, 5);
        let neighbors = coord.neighbors_2d();
        
        assert_eq!(neighbors.len(), 8); // 3x3 - 1 (self)
        
        // Y should be preserved
        for n in &neighbors {
            assert_eq!(n.y, 0);
        }
        
        // Should not include self
        assert!(!neighbors.contains(&GridCoord::new(5, 0, 5)));
        
        // Should include diagonal neighbors
        assert!(neighbors.contains(&GridCoord::new(4, 0, 4)));
        assert!(neighbors.contains(&GridCoord::new(6, 0, 6)));
    }

    #[test]
    fn test_grid_coord_manhattan_distance() {
        let a = GridCoord::new(0, 0, 0);
        let b = GridCoord::new(3, 4, 5);
        
        assert_eq!(a.manhattan_distance(b), 12); // 3 + 4 + 5
        assert_eq!(b.manhattan_distance(a), 12); // Symmetric
    }

    #[test]
    fn test_grid_coord_manhattan_distance_same() {
        let a = GridCoord::new(5, 5, 5);
        assert_eq!(a.manhattan_distance(a), 0);
    }

    #[test]
    fn test_grid_coord_equality() {
        let a = GridCoord::new(1, 2, 3);
        let b = GridCoord::new(1, 2, 3);
        let c = GridCoord::new(1, 2, 4);
        
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_grid_coord_hash() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        set.insert(GridCoord::new(1, 2, 3));
        set.insert(GridCoord::new(1, 2, 3)); // Duplicate
        set.insert(GridCoord::new(4, 5, 6));
        
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_grid_coord_serialization() {
        let coord = GridCoord::new(10, 20, 30);
        let json = serde_json::to_string(&coord).unwrap();
        let deserialized: GridCoord = serde_json::from_str(&json).unwrap();
        
        assert_eq!(coord, deserialized);
    }

    // ===== AABB Tests =====

    #[test]
    fn test_aabb_new() {
        let aabb = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        assert_eq!(aabb.min, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(aabb.max, Vec3::new(10.0, 10.0, 10.0));
    }

    #[test]
    fn test_aabb_from_center_half_extents() {
        let aabb = AABB::from_center_half_extents(
            Vec3::new(5.0, 5.0, 5.0),
            Vec3::new(5.0, 5.0, 5.0),
        );
        
        assert!((aabb.min - Vec3::new(0.0, 0.0, 0.0)).length() < 0.0001);
        assert!((aabb.max - Vec3::new(10.0, 10.0, 10.0)).length() < 0.0001);
    }

    #[test]
    fn test_aabb_center() {
        let aabb = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 20.0, 30.0));
        let center = aabb.center();
        
        assert!((center - Vec3::new(5.0, 10.0, 15.0)).length() < 0.0001);
    }

    #[test]
    fn test_aabb_half_extents() {
        let aabb = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 20.0, 30.0));
        let half = aabb.half_extents();
        
        assert!((half - Vec3::new(5.0, 10.0, 15.0)).length() < 0.0001);
    }

    #[test]
    fn test_aabb_contains_point_inside() {
        let aabb = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        
        assert!(aabb.contains_point(Vec3::new(5.0, 5.0, 5.0)));
        assert!(aabb.contains_point(Vec3::new(0.0, 0.0, 0.0))); // On boundary
        assert!(aabb.contains_point(Vec3::new(10.0, 10.0, 10.0))); // On boundary
    }

    #[test]
    fn test_aabb_contains_point_outside() {
        let aabb = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        
        assert!(!aabb.contains_point(Vec3::new(-1.0, 5.0, 5.0)));
        assert!(!aabb.contains_point(Vec3::new(11.0, 5.0, 5.0)));
        assert!(!aabb.contains_point(Vec3::new(5.0, -1.0, 5.0)));
        assert!(!aabb.contains_point(Vec3::new(5.0, 11.0, 5.0)));
        assert!(!aabb.contains_point(Vec3::new(5.0, 5.0, -1.0)));
        assert!(!aabb.contains_point(Vec3::new(5.0, 5.0, 11.0)));
    }

    #[test]
    fn test_aabb_intersects_overlapping() {
        let a = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        let b = AABB::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(15.0, 15.0, 15.0));
        
        assert!(a.intersects(&b));
        assert!(b.intersects(&a)); // Symmetric
    }

    #[test]
    fn test_aabb_intersects_touching() {
        let a = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        let b = AABB::new(Vec3::new(10.0, 0.0, 0.0), Vec3::new(20.0, 10.0, 10.0));
        
        assert!(a.intersects(&b)); // Touching at edge
    }

    #[test]
    fn test_aabb_intersects_separate() {
        let a = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 10.0, 10.0));
        let b = AABB::new(Vec3::new(20.0, 20.0, 20.0), Vec3::new(30.0, 30.0, 30.0));
        
        assert!(!a.intersects(&b));
        assert!(!b.intersects(&a));
    }

    #[test]
    fn test_aabb_intersects_contained() {
        let outer = AABB::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(100.0, 100.0, 100.0));
        let inner = AABB::new(Vec3::new(25.0, 25.0, 25.0), Vec3::new(75.0, 75.0, 75.0));
        
        assert!(outer.intersects(&inner));
        assert!(inner.intersects(&outer));
    }

    #[test]
    fn test_aabb_overlapping_cells_single() {
        let aabb = AABB::new(Vec3::new(25.0, 25.0, 25.0), Vec3::new(75.0, 75.0, 75.0));
        let cells = aabb.overlapping_cells(100.0);
        
        // Should only overlap one cell (0, 0, 0) since AABB is entirely within
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0], GridCoord::new(0, 0, 0));
    }

    #[test]
    fn test_aabb_overlapping_cells_multiple() {
        // AABB crossing the boundary between cell (0,0,0) and (1,0,0)
        let aabb = AABB::new(Vec3::new(50.0, 0.0, 0.0), Vec3::new(150.0, 50.0, 50.0));
        let cells = aabb.overlapping_cells(100.0);
        
        // Should overlap cells along X axis: (0,0,0) and (1,0,0)
        // The AABB is contained within Y=[0,50] and Z=[0,50] so only 1 cell in those dimensions
        // Total: 2 cells in X × 1 in Y × 1 in Z = 2 cells
        assert_eq!(cells.len(), 2);
        assert!(cells.contains(&GridCoord::new(0, 0, 0)));
        assert!(cells.contains(&GridCoord::new(1, 0, 0)));
    }

    #[test]
    fn test_aabb_serialization() {
        let aabb = AABB::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0));
        let json = serde_json::to_string(&aabb).unwrap();
        let deserialized: AABB = serde_json::from_str(&json).unwrap();
        
        assert!((deserialized.min - aabb.min).length() < 0.0001);
        assert!((deserialized.max - aabb.max).length() < 0.0001);
    }

    // ===== Frustum Tests =====

    #[test]
    fn test_frustum_from_view_projection() {
        // Create a simple orthographic projection for testing
        let view_proj = Mat4::orthographic_rh(-10.0, 10.0, -10.0, 10.0, 0.1, 100.0);
        let frustum = Frustum::from_view_projection(view_proj);
        
        // All 6 planes should be normalized (length ≈ 1)
        for plane in &frustum.planes {
            let normal_length = Vec3::new(plane.x, plane.y, plane.z).length();
            assert!((normal_length - 1.0).abs() < 0.0001, "Plane normal should be normalized");
        }
    }

    #[test]
    fn test_frustum_intersects_aabb_inside() {
        let view_proj = Mat4::orthographic_rh(-100.0, 100.0, -100.0, 100.0, 0.1, 1000.0);
        let frustum = Frustum::from_view_projection(view_proj);
        
        // Small box at origin should be inside frustum
        let aabb = AABB::new(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0));
        assert!(frustum.intersects_aabb(&aabb));
    }

    #[test]
    fn test_frustum_cells_in_frustum() {
        let view_proj = Mat4::orthographic_rh(-100.0, 100.0, -100.0, 100.0, 0.1, 1000.0);
        let frustum = Frustum::from_view_projection(view_proj);
        
        let cells = frustum.cells_in_frustum(Vec3::ZERO, 50.0, 100.0);
        
        // Should return some cells near origin
        assert!(!cells.is_empty());
        // Center cell should be included
        assert!(cells.contains(&GridCoord::new(0, 0, 0)));
    }

    // ===== AssetRef & AssetType Tests =====

    #[test]
    fn test_asset_ref_creation() {
        let asset = AssetRef {
            path: "meshes/rock.glb".to_string(),
            asset_type: AssetType::Mesh,
        };
        
        assert_eq!(asset.path, "meshes/rock.glb");
        assert_eq!(asset.asset_type, AssetType::Mesh);
    }

    #[test]
    fn test_asset_type_variants() {
        assert_ne!(AssetType::Mesh, AssetType::Texture);
        assert_ne!(AssetType::Material, AssetType::Audio);
        assert_eq!(AssetType::Mesh, AssetType::Mesh);
    }

    #[test]
    fn test_asset_ref_serialization() {
        let asset = AssetRef {
            path: "test/path.glb".to_string(),
            asset_type: AssetType::Mesh,
        };
        
        let json = serde_json::to_string(&asset).unwrap();
        let deserialized: AssetRef = serde_json::from_str(&json).unwrap();
        
        assert_eq!(asset.path, deserialized.path);
        assert_eq!(asset.asset_type, deserialized.asset_type);
    }

    // ===== CellState Tests =====

    #[test]
    fn test_cell_state_variants() {
        assert_eq!(CellState::Unloaded, CellState::Unloaded);
        assert_ne!(CellState::Loading, CellState::Loaded);
        assert_ne!(CellState::Unloading, CellState::Unloaded);
    }

    #[test]
    fn test_cell_state_serialization() {
        let states = vec![
            CellState::Unloaded,
            CellState::Loading,
            CellState::Loaded,
            CellState::Unloading,
        ];
        
        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: CellState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, deserialized);
        }
    }

    // ===== GridConfig Tests =====

    #[test]
    fn test_grid_config_creation() {
        let config = GridConfig {
            cell_size: 100.0,
            world_bounds: (-5000.0, 5000.0, -5000.0, 5000.0),
        };
        
        assert_eq!(config.cell_size, 100.0);
        assert_eq!(config.world_bounds.0, -5000.0);
        assert_eq!(config.world_bounds.1, 5000.0);
    }

    // ===== WorldPartition Tests =====

    #[test]
    fn test_world_partition_new() {
        let config = GridConfig {
            cell_size: 100.0,
            world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
        };
        let partition = WorldPartition::new(config);
        
        assert_eq!(partition.config.cell_size, 100.0);
        assert!(partition.cells.is_empty());
    }

    #[test]
    fn test_world_partition_get_or_create_cell() {
        let config = GridConfig {
            cell_size: 100.0,
            world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
        };
        let mut partition = WorldPartition::new(config);
        
        let coord = GridCoord::new(0, 0, 0);
        let cell = partition.get_or_create_cell(coord);
        
        assert_eq!(cell.coord, coord);
        assert_eq!(cell.state, CellState::Unloaded);
        assert!(cell.entities.is_empty());
    }

    #[test]
    fn test_world_partition_assign_entity_to_cell() {
        let config = GridConfig {
            cell_size: 100.0,
            world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
        };
        let mut partition = WorldPartition::new(config);
        
        let entity_id = 42;
        let pos = Vec3::new(150.0, 50.0, 250.0);
        
        partition.assign_entity_to_cell(entity_id, pos);
        
        let coord = GridCoord::from_world_pos(pos, 100.0);
        let cell = partition.get_cell(coord).unwrap();
        
        assert!(cell.entities.contains(&entity_id));
    }

    #[test]
    fn test_world_partition_get_cell() {
        let config = GridConfig {
            cell_size: 100.0,
            world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
        };
        let mut partition = WorldPartition::new(config);
        
        // Non-existent cell should return None
        assert!(partition.get_cell(GridCoord::new(99, 99, 99)).is_none());
        
        // Create cell and verify it exists
        let coord = GridCoord::new(1, 2, 3);
        partition.get_or_create_cell(coord);
        assert!(partition.get_cell(coord).is_some());
    }

    #[test]
    fn test_world_partition_cells_in_radius() {
        let config = GridConfig {
            cell_size: 100.0,
            world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
        };
        let partition = WorldPartition::new(config);
        
        let center = Vec3::new(150.0, 50.0, 150.0);
        let cells = partition.cells_in_radius(center, 150.0); // 1.5 cell radius
        
        // Should include center cell and some neighbors
        assert!(!cells.is_empty());
        
        // Center cell should be included
        let center_coord = GridCoord::from_world_pos(center, 100.0);
        assert!(cells.contains(&center_coord));
    }

    #[test]
    fn test_world_partition_memory_usage_estimate() {
        let config = GridConfig {
            cell_size: 100.0,
            world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
        };
        let mut partition = WorldPartition::new(config);
        
        // Empty partition
        let empty_mem = partition.memory_usage_estimate();
        
        // Add a cell
        partition.get_or_create_cell(GridCoord::new(0, 0, 0));
        let with_cell_mem = partition.memory_usage_estimate();
        
        assert!(with_cell_mem > empty_mem);
    }

    // ===== LRUCache Tests =====

    #[test]
    fn test_lru_cache_new() {
        let cache = LRUCache::new(10);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_lru_cache_touch() {
        let mut cache = LRUCache::new(5);
        
        cache.touch(GridCoord::new(1, 0, 0));
        cache.touch(GridCoord::new(2, 0, 0));
        
        assert_eq!(cache.len(), 2);
        assert!(cache.contains(GridCoord::new(1, 0, 0)));
        assert!(cache.contains(GridCoord::new(2, 0, 0)));
    }

    #[test]
    fn test_lru_cache_touch_reorder() {
        let mut cache = LRUCache::new(5);
        
        cache.touch(GridCoord::new(1, 0, 0));
        cache.touch(GridCoord::new(2, 0, 0));
        cache.touch(GridCoord::new(1, 0, 0)); // Touch 1 again
        
        // LRU should be 2 now (1 was more recently used)
        assert_eq!(cache.lru(), Some(GridCoord::new(2, 0, 0)));
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LRUCache::new(3);
        
        cache.touch(GridCoord::new(1, 0, 0));
        cache.touch(GridCoord::new(2, 0, 0));
        cache.touch(GridCoord::new(3, 0, 0));
        cache.touch(GridCoord::new(4, 0, 0)); // Should evict 1
        
        assert_eq!(cache.len(), 3);
        assert!(!cache.contains(GridCoord::new(1, 0, 0))); // Evicted
        assert!(cache.contains(GridCoord::new(2, 0, 0)));
        assert!(cache.contains(GridCoord::new(3, 0, 0)));
        assert!(cache.contains(GridCoord::new(4, 0, 0)));
    }

    #[test]
    fn test_lru_cache_remove() {
        let mut cache = LRUCache::new(5);
        
        cache.touch(GridCoord::new(1, 0, 0));
        cache.touch(GridCoord::new(2, 0, 0));
        
        cache.remove(GridCoord::new(1, 0, 0));
        
        assert_eq!(cache.len(), 1);
        assert!(!cache.contains(GridCoord::new(1, 0, 0)));
        assert!(cache.contains(GridCoord::new(2, 0, 0)));
    }

    #[test]
    fn test_lru_cache_remove_nonexistent() {
        let mut cache = LRUCache::new(5);
        cache.touch(GridCoord::new(1, 0, 0));
        
        // Removing nonexistent should not panic
        cache.remove(GridCoord::new(99, 99, 99));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_lru_cache_lru_empty() {
        let cache = LRUCache::new(5);
        assert_eq!(cache.lru(), None);
    }

    #[test]
    fn test_lru_cache_lru_single() {
        let mut cache = LRUCache::new(5);
        cache.touch(GridCoord::new(42, 0, 0));
        
        assert_eq!(cache.lru(), Some(GridCoord::new(42, 0, 0)));
    }
}

