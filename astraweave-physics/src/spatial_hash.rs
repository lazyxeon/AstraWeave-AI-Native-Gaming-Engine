/*!
# Spatial Hash Grid for Broad-Phase Collision Detection

Implements a grid-based spatial partitioning system to reduce collision detection
from O(n²) to O(n log n) by only testing objects in nearby grid cells.

## Performance

**Baseline (Naive O(n²))**:
- 1000 entities: ~500,000 collision pairs to test
- Frame time: 548.5 µs (17.71% of frame)

**With Spatial Hashing**:
- 1000 entities: ~5,000-10,000 collision pairs (99% reduction)
- Expected frame time: 250-330 µs (8-10% of frame)
- **Improvement**: -40-55% collision detection time

## Usage

```rust
use astraweave_physics::{SpatialHash, AABB};
use glam::Vec3;

// Create grid with 10-unit cells (adjust based on object sizes)
let mut grid = SpatialHash::<u32>::new(10.0);

// Insert objects
let objects = vec![(1, AABB::from_sphere(Vec3::ZERO, 1.0))];
for (id, aabb) in objects {
    grid.insert(id, aabb);
}

// Query potential collisions for an object
let aabb = AABB::from_sphere(Vec3::ZERO, 1.0);
let candidates = grid.query(aabb);

// Only test narrow-phase collision against candidates (not all objects)
for candidate_id in candidates {
    let candidate_aabb = AABB::from_sphere(Vec3::ZERO, 1.0); // Mock
    if aabb.intersects(&candidate_aabb) {
        // Handle collision
    }
}

// Clear grid before next frame
grid.clear();
```

## Cell Size Selection

**Rule of Thumb**: Cell size should be ~2× the average object size

- Too small: Objects span multiple cells, redundant queries
- Too large: Too many objects per cell, approaches O(n²)
- Optimal: Each object touches 1-4 cells

**Example**:
- Character radius: 1 unit → Cell size: 2-4 units
- Small objects (0.5 radius) → Cell size: 1-2 units
- Large objects (5 radius) → Cell size: 10-20 units
*/

use glam::Vec3;

/// Axis-Aligned Bounding Box for collision detection
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// Create AABB from center and half-extents
    pub fn from_center_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Create AABB from center and radius (sphere approximation)
    pub fn from_sphere(center: Vec3, radius: f32) -> Self {
        let half_extents = Vec3::splat(radius);
        Self::from_center_extents(center, half_extents)
    }

    /// Check if two AABBs intersect
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Get center point of AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get half-extents of AABB
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
}

/// Grid cell coordinates (3D integer grid)
type GridCell = (i32, i32, i32);

/// Spatial hash grid for broad-phase collision detection
///
/// Uses a uniform grid to partition 3D space. Objects are inserted into grid cells
/// based on their AABB. Queries return only objects in nearby cells, dramatically
/// reducing the number of collision pairs to test.
///
/// **Phase B Optimization (Action 27)**: Uses FxHashMap for faster hashing.
/// Week 9 testing confirmed FxHashMap provides better performance than SipHash
/// for spatial grids (3.77ms vs 5.61ms with Tracy, 3.82ms without Tracy).
#[derive(Debug)]
pub struct SpatialHash<T> {
    /// Grid cell size (world units)
    cell_size: f32,

    /// Inverse cell size (for faster division-free grid coordinate calculation)
    inv_cell_size: f32,

    /// Grid storage: (x, y, z) → [object IDs]
    /// Uses FxHashMap for faster non-cryptographic hashing
    grid: rustc_hash::FxHashMap<GridCell, Vec<T>>,

    /// Total objects currently in grid
    object_count: usize,
}

impl<T: Copy + Eq + Ord> SpatialHash<T> {
    /// Create new spatial hash with specified cell size
    ///
    /// # Arguments
    /// * `cell_size` - Grid cell size in world units (recommended: 2× average object size)
    ///
    /// # Example
    /// ```
    /// use astraweave_physics::SpatialHash;
    /// let grid = SpatialHash::<u32>::new(10.0); // 10-unit cells
    /// ```
    pub fn new(cell_size: f32) -> Self {
        assert!(cell_size > 0.0, "Cell size must be positive");

        Self {
            cell_size,
            inv_cell_size: 1.0 / cell_size,
            grid: rustc_hash::FxHashMap::default(),
            object_count: 0,
        }
    }

    /// Convert world position to grid cell coordinates
    #[inline]
    fn world_to_cell(&self, pos: Vec3) -> GridCell {
        (
            (pos.x * self.inv_cell_size).floor() as i32,
            (pos.y * self.inv_cell_size).floor() as i32,
            (pos.z * self.inv_cell_size).floor() as i32,
        )
    }

    /// Get all grid cells that an AABB overlaps
    fn get_overlapping_cells(&self, aabb: &AABB) -> Vec<GridCell> {
        let min_cell = self.world_to_cell(aabb.min);
        let max_cell = self.world_to_cell(aabb.max);

        let mut cells = Vec::new();

        // Iterate over all cells that the AABB spans
        for x in min_cell.0..=max_cell.0 {
            for y in min_cell.1..=max_cell.1 {
                for z in min_cell.2..=max_cell.2 {
                    cells.push((x, y, z));
                }
            }
        }

        cells
    }

    /// Insert object into grid based on its AABB
    ///
    /// Objects that span multiple cells are inserted into all overlapping cells.
    ///
    /// # Arguments
    /// * `id` - Object identifier
    /// * `aabb` - Object's axis-aligned bounding box
    ///
    /// # Example
    /// ```
    /// use astraweave_physics::{SpatialHash, AABB};
    /// use glam::Vec3;
    /// let mut grid = SpatialHash::<u32>::new(10.0);
    /// let entity_id = 1;
    /// let aabb = AABB::from_sphere(Vec3::new(5.0, 0.0, 5.0), 1.0);
    /// grid.insert(entity_id, aabb);
    /// ```
    pub fn insert(&mut self, id: T, aabb: AABB) {
        let cells = self.get_overlapping_cells(&aabb);

        for cell in cells {
            self.grid.entry(cell).or_default().push(id);
        }

        self.object_count += 1;
    }

    /// Query potential collision candidates for an AABB
    ///
    /// Returns all objects in grid cells that overlap the query AABB.
    /// Duplicates are possible if objects span multiple cells.
    ///
    /// # Arguments
    /// * `aabb` - Query bounding box
    ///
    /// # Returns
    /// Vec of object IDs that *might* collide (candidates for narrow-phase testing)
    ///
    /// # Example
    /// ```
    /// use astraweave_physics::{SpatialHash, AABB};
    /// use glam::Vec3;
    /// let mut grid = SpatialHash::<u32>::new(10.0);
    /// let pos = Vec3::ZERO;
    /// let radius = 1.0;
    /// let query_aabb = AABB::from_sphere(pos, radius);
    /// let candidates = grid.query(query_aabb);
    ///
    /// for &candidate_id in &candidates {
    ///     // Perform detailed collision test with candidate
    /// }
    /// ```
    pub fn query(&self, aabb: AABB) -> Vec<T> {
        let cells = self.get_overlapping_cells(&aabb);
        let mut results = Vec::new();

        for cell in cells {
            if let Some(objects) = self.grid.get(&cell) {
                results.extend_from_slice(objects);
            }
        }

        results
    }

    /// Query unique collision candidates (removes duplicates)
    ///
    /// Same as `query()` but deduplicates results. Slightly slower due to sorting,
    /// but useful when duplicate checks are expensive.
    pub fn query_unique(&self, aabb: AABB) -> Vec<T> {
        let mut results = self.query(aabb);
        results.sort_unstable();
        results.dedup();
        results
    }

    /// Clear all objects from grid
    ///
    /// Call this before rebuilding grid each frame (dynamic objects).
    pub fn clear(&mut self) {
        self.grid.clear();
        self.object_count = 0;
    }

    /// Get number of objects in grid
    pub fn object_count(&self) -> usize {
        self.object_count
    }

    /// Get number of occupied grid cells
    pub fn cell_count(&self) -> usize {
        self.grid.len()
    }

    /// Get grid cell size
    pub fn cell_size(&self) -> f32 {
        self.cell_size
    }

    /// Get average objects per cell (density metric)
    pub fn average_cell_density(&self) -> f32 {
        if self.grid.is_empty() {
            0.0
        } else {
            self.object_count as f32 / self.grid.len() as f32
        }
    }

    /// Debug: Get grid statistics
    pub fn stats(&self) -> SpatialHashStats {
        let mut max_objects_per_cell = 0;
        let mut total_objects_in_cells = 0;

        for objects in self.grid.values() {
            max_objects_per_cell = max_objects_per_cell.max(objects.len());
            total_objects_in_cells += objects.len();
        }

        SpatialHashStats {
            object_count: self.object_count,
            cell_count: self.grid.len(),
            max_objects_per_cell,
            average_objects_per_cell: if self.grid.is_empty() {
                0.0
            } else {
                total_objects_in_cells as f32 / self.grid.len() as f32
            },
            cell_size: self.cell_size,
        }
    }
}

/// Spatial hash statistics for debugging/profiling
#[derive(Debug, Clone)]
pub struct SpatialHashStats {
    pub object_count: usize,
    pub cell_count: usize,
    pub max_objects_per_cell: usize,
    pub average_objects_per_cell: f32,
    pub cell_size: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // BASIC AABB TESTS (Original)
    // ============================================================================

    #[test]
    fn test_aabb_intersection() {
        let aabb1 = AABB::from_sphere(Vec3::ZERO, 1.0);
        let aabb2 = AABB::from_sphere(Vec3::new(1.5, 0.0, 0.0), 1.0);
        let aabb3 = AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 1.0);

        assert!(aabb1.intersects(&aabb2), "Adjacent AABBs should intersect");
        assert!(
            !aabb1.intersects(&aabb3),
            "Distant AABBs should not intersect"
        );
    }

    #[test]
    fn test_spatial_hash_insertion() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        let aabb = AABB::from_sphere(Vec3::new(5.0, 0.0, 5.0), 1.0);
        grid.insert(1, aabb);

        assert_eq!(grid.object_count(), 1);
        assert!(grid.cell_count() > 0);
    }

    #[test]
    fn test_spatial_hash_query() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Insert objects in different cells
        grid.insert(1, AABB::from_sphere(Vec3::new(5.0, 0.0, 5.0), 1.0));
        grid.insert(2, AABB::from_sphere(Vec3::new(15.0, 0.0, 5.0), 1.0));
        grid.insert(3, AABB::from_sphere(Vec3::new(25.0, 0.0, 5.0), 1.0));

        // Query near object 1
        let results = grid.query(AABB::from_sphere(Vec3::new(5.0, 0.0, 5.0), 1.0));

        // Should find object 1, but not objects 2 or 3 (in different cells)
        assert!(results.contains(&1));
        // Note: Might find 2 if cells are adjacent, but definitely not 3
    }

    #[test]
    fn test_spatial_hash_clear() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        grid.insert(1, AABB::from_sphere(Vec3::ZERO, 1.0));
        assert_eq!(grid.object_count(), 1);

        grid.clear();
        assert_eq!(grid.object_count(), 0);
        assert_eq!(grid.cell_count(), 0);
    }

    #[test]
    fn test_multi_cell_spanning() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Large AABB spanning multiple cells
        let large_aabb = AABB {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(25.0, 25.0, 25.0),
        };

        grid.insert(1, large_aabb);

        // Object should be in multiple cells (3×3×3 = 27 cells)
        assert!(
            grid.cell_count() >= 27,
            "Large object should span multiple cells"
        );
    }

    #[test]
    fn test_query_unique_deduplication() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Insert object spanning multiple cells
        let large_aabb = AABB {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(15.0, 0.0, 0.0),
        };

        grid.insert(1, large_aabb);

        // Query overlapping multiple cells
        let query_aabb = AABB {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(15.0, 0.0, 0.0),
        };

        let results = grid.query(query_aabb);
        let unique_results = grid.query_unique(query_aabb);

        // query() may return duplicates, query_unique() should not
        assert!(unique_results.len() <= results.len());
        assert_eq!(unique_results.len(), 1, "Should find object 1 exactly once");
    }

    #[test]
    fn test_cell_size_calculation() {
        let grid = SpatialHash::<u32>::new(5.0);

        let cell1 = grid.world_to_cell(Vec3::new(0.0, 0.0, 0.0));
        let cell2 = grid.world_to_cell(Vec3::new(4.9, 0.0, 0.0));
        let cell3 = grid.world_to_cell(Vec3::new(5.1, 0.0, 0.0));

        assert_eq!(
            cell1, cell2,
            "Points in same cell should have same cell coords"
        );
        assert_ne!(cell1, cell3, "Points in different cells should differ");
    }

    #[test]
    fn test_stats() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Insert 3 objects in same cell
        for i in 0..3 {
            grid.insert(i, AABB::from_sphere(Vec3::new(5.0, 0.0, 5.0), 0.5));
        }

        let stats = grid.stats();
        assert_eq!(stats.object_count, 3);
        assert!(stats.average_objects_per_cell >= 3.0);
    }

    // ============================================================================
    // STRESS TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_stress_1000_entities() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Insert 1000 entities spread across space
        for i in 0..1000 {
            let x = (i % 100) as f32 * 5.0;
            let y = ((i / 100) % 10) as f32 * 5.0;
            let z = (i / 1000) as f32 * 5.0;
            grid.insert(i, AABB::from_sphere(Vec3::new(x, y, z), 1.0));
        }

        assert_eq!(grid.object_count(), 1000);
        
        // Verify queries return reasonable results
        let results = grid.query(AABB::from_sphere(Vec3::new(25.0, 25.0, 0.0), 5.0));
        assert!(!results.is_empty(), "Query should find nearby entities");
    }

    #[test]
    fn test_stress_10000_entities() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Insert 10000 entities spread across space
        for i in 0..10000 {
            let x = (i % 100) as f32 * 2.0;
            let y = ((i / 100) % 100) as f32 * 2.0;
            let z = (i / 10000) as f32 * 2.0;
            grid.insert(i, AABB::from_sphere(Vec3::new(x, y, z), 0.5));
        }

        assert_eq!(grid.object_count(), 10000);
        
        let stats = grid.stats();
        // With good distribution, average should be reasonable
        assert!(stats.average_objects_per_cell < 50.0, 
            "Cell density should stay manageable: {}", stats.average_objects_per_cell);
    }

    #[test]
    fn test_stress_clustered_entities() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // All 500 entities in same cell
        for i in 0..500 {
            grid.insert(i, AABB::from_sphere(Vec3::new(5.0, 5.0, 5.0), 0.1));
        }

        let stats = grid.stats();
        assert_eq!(stats.max_objects_per_cell, 500);
        
        // Query should find all entities
        let results = grid.query(AABB::from_sphere(Vec3::new(5.0, 5.0, 5.0), 0.5));
        assert_eq!(results.len(), 500);
    }

    #[test]
    fn test_stress_query_performance_linear() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Insert 1000 scattered entities
        for i in 0..1000 {
            let x = (i % 100) as f32 * 10.0;
            let y = (i / 100) as f32 * 10.0;
            grid.insert(i, AABB::from_sphere(Vec3::new(x, y, 0.0), 1.0));
        }

        // Query a small area - should NOT return all 1000
        let results = grid.query(AABB::from_sphere(Vec3::new(50.0, 50.0, 0.0), 5.0));
        
        // With O(n log n), we should only get nearby entities
        assert!(results.len() < 100, 
            "Query should be localized, got {} results", results.len());
    }

    // ============================================================================
    // EDGE CASES (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_edge_case_zero_size_aabb() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Zero-size AABB (point)
        let point_aabb = AABB {
            min: Vec3::new(5.0, 5.0, 5.0),
            max: Vec3::new(5.0, 5.0, 5.0),
        };

        grid.insert(1, point_aabb);
        assert_eq!(grid.object_count(), 1);

        let results = grid.query(point_aabb);
        assert!(results.contains(&1));
    }

    #[test]
    fn test_edge_case_negative_coordinates() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Entities in negative space
        grid.insert(1, AABB::from_sphere(Vec3::new(-50.0, -50.0, -50.0), 1.0));
        grid.insert(2, AABB::from_sphere(Vec3::new(-5.0, -5.0, -5.0), 1.0));
        grid.insert(3, AABB::from_sphere(Vec3::new(5.0, 5.0, 5.0), 1.0));

        assert_eq!(grid.object_count(), 3);

        // Query in negative space
        let results = grid.query(AABB::from_sphere(Vec3::new(-50.0, -50.0, -50.0), 5.0));
        assert!(results.contains(&1));
        assert!(!results.contains(&3));
    }

    #[test]
    fn test_edge_case_cell_boundary_exact() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Entity exactly on cell boundary
        grid.insert(1, AABB::from_sphere(Vec3::new(10.0, 10.0, 10.0), 0.5));

        // Query from adjacent cell
        let results = grid.query(AABB::from_sphere(Vec3::new(9.0, 9.0, 9.0), 2.0));
        
        // Should find entity spanning boundary
        assert!(results.contains(&1), "Should find entity on boundary");
    }

    #[test]
    fn test_edge_case_entity_spanning_boundary() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Entity spanning cell boundary
        let boundary_aabb = AABB {
            min: Vec3::new(8.0, 8.0, 8.0),
            max: Vec3::new(12.0, 12.0, 12.0),
        };
        grid.insert(1, boundary_aabb);

        // Should be in multiple cells
        assert!(grid.cell_count() >= 8, 
            "Boundary-spanning entity should be in multiple cells: {}", grid.cell_count());
    }

    #[test]
    fn test_edge_case_very_large_aabb() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Massive AABB spanning 100 cells in each dimension
        let huge_aabb = AABB {
            min: Vec3::new(-500.0, -500.0, -500.0),
            max: Vec3::new(500.0, 500.0, 500.0),
        };
        grid.insert(1, huge_aabb);

        // Should span 100×100×100 = 1,000,000 cells
        let expected_cells = 100 * 100 * 100;
        assert!(grid.cell_count() >= expected_cells - 1, 
            "Huge AABB should span many cells: {} vs expected {}", 
            grid.cell_count(), expected_cells);
    }

    #[test]
    fn test_edge_case_very_small_cell_size() {
        let mut grid = SpatialHash::<u32>::new(0.1); // Very small cells

        grid.insert(1, AABB::from_sphere(Vec3::new(5.0, 5.0, 5.0), 1.0));

        // 2-unit diameter sphere should span 20×20×20 = 8000 cells
        let stats = grid.stats();
        assert!(stats.cell_count >= 8000, 
            "Small cells should result in many cells: {}", stats.cell_count);
    }

    #[test]
    fn test_edge_case_very_large_cell_size() {
        let mut grid = SpatialHash::<u32>::new(1000.0); // Very large cells

        // Insert many entities - all should fit in few cells
        for i in 0..100 {
            let x = (i % 10) as f32 * 10.0;
            let y = (i / 10) as f32 * 10.0;
            grid.insert(i, AABB::from_sphere(Vec3::new(x, y, 0.0), 1.0));
        }

        // With large cells, all entities should be in very few cells
        assert!(grid.cell_count() <= 8, 
            "Large cells should contain entities in few cells: {}", grid.cell_count());
    }

    // ============================================================================
    // CELL BOUNDARY TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_cell_boundary_query_finds_adjacent() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Entity just inside cell (0,0,0)
        grid.insert(1, AABB::from_sphere(Vec3::new(9.5, 9.5, 9.5), 1.0));
        
        // Entity just inside cell (1,1,1)
        grid.insert(2, AABB::from_sphere(Vec3::new(10.5, 10.5, 10.5), 1.0));

        // Query spanning the boundary
        let results = grid.query_unique(AABB {
            min: Vec3::new(8.0, 8.0, 8.0),
            max: Vec3::new(12.0, 12.0, 12.0),
        });

        assert!(results.contains(&1), "Should find entity 1 near boundary");
        assert!(results.contains(&2), "Should find entity 2 near boundary");
    }

    #[test]
    fn test_cell_boundary_entity_at_origin() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Entity exactly at origin (spans negative and positive cells)
        grid.insert(1, AABB::from_sphere(Vec3::ZERO, 1.0));

        // Check it's in multiple cells around origin
        assert!(grid.cell_count() >= 8, 
            "Origin entity should span 8 cells: {}", grid.cell_count());
    }

    #[test]
    fn test_cell_boundary_negative_positive_transition() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Entity at negative cell
        grid.insert(1, AABB::from_sphere(Vec3::new(-5.0, 0.0, 0.0), 1.0));
        
        // Entity at positive cell
        grid.insert(2, AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 1.0));

        // Query spanning both cells
        let results = grid.query_unique(AABB {
            min: Vec3::new(-10.0, -1.0, -1.0),
            max: Vec3::new(10.0, 1.0, 1.0),
        });

        assert!(results.contains(&1), "Should find negative cell entity");
        assert!(results.contains(&2), "Should find positive cell entity");
    }

    // ============================================================================
    // AABB HELPER TESTS (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_aabb_center() {
        let aabb = AABB {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(10.0, 20.0, 30.0),
        };

        let center = aabb.center();
        assert!((center.x - 5.0).abs() < 0.001);
        assert!((center.y - 10.0).abs() < 0.001);
        assert!((center.z - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_half_extents() {
        let aabb = AABB {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(10.0, 20.0, 30.0),
        };

        let half_extents = aabb.half_extents();
        assert!((half_extents.x - 5.0).abs() < 0.001);
        assert!((half_extents.y - 10.0).abs() < 0.001);
        assert!((half_extents.z - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_from_center_extents() {
        let aabb = AABB::from_center_extents(Vec3::new(5.0, 10.0, 15.0), Vec3::new(5.0, 10.0, 15.0));

        assert!((aabb.min.x - 0.0).abs() < 0.001);
        assert!((aabb.min.y - 0.0).abs() < 0.001);
        assert!((aabb.min.z - 0.0).abs() < 0.001);
        assert!((aabb.max.x - 10.0).abs() < 0.001);
        assert!((aabb.max.y - 20.0).abs() < 0.001);
        assert!((aabb.max.z - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_aabb_intersection_touching() {
        // AABBs touching exactly at edge
        let aabb1 = AABB {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(10.0, 10.0, 10.0),
        };
        let aabb2 = AABB {
            min: Vec3::new(10.0, 0.0, 0.0),
            max: Vec3::new(20.0, 10.0, 10.0),
        };

        assert!(aabb1.intersects(&aabb2), "Edge-touching AABBs should intersect");
    }

    #[test]
    fn test_aabb_intersection_corner() {
        // AABBs touching at corner
        let aabb1 = AABB {
            min: Vec3::new(0.0, 0.0, 0.0),
            max: Vec3::new(10.0, 10.0, 10.0),
        };
        let aabb2 = AABB {
            min: Vec3::new(10.0, 10.0, 10.0),
            max: Vec3::new(20.0, 20.0, 20.0),
        };

        assert!(aabb1.intersects(&aabb2), "Corner-touching AABBs should intersect");
    }

    #[test]
    fn test_aabb_self_intersection() {
        let aabb = AABB::from_sphere(Vec3::ZERO, 5.0);
        assert!(aabb.intersects(&aabb), "AABB should intersect itself");
    }

    // ============================================================================
    // COLLISION PAIR REDUCTION VALIDATION (Phase 8.8 - New)
    // ============================================================================

    #[test]
    fn test_collision_pair_reduction() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Insert 100 entities scattered across 100×100 unit space
        for i in 0..100 {
            let x = (i % 10) as f32 * 10.0;
            let y = (i / 10) as f32 * 10.0;
            grid.insert(i, AABB::from_sphere(Vec3::new(x, y, 0.0), 1.0));
        }

        // Count collision pairs using grid vs naive O(n²)
        let mut grid_pairs = 0;
        for i in 0..100u32 {
            let x = (i % 10) as f32 * 10.0;
            let y = (i / 10) as f32 * 10.0;
            let candidates = grid.query(AABB::from_sphere(Vec3::new(x, y, 0.0), 1.0));
            grid_pairs += candidates.len();
        }

        let naive_pairs = 100 * 99; // O(n²) pairs to test

        // Grid should significantly reduce pairs (at least 2× improvement)
        assert!(grid_pairs < naive_pairs / 2, 
            "Grid should reduce pairs: {} vs naive {}", grid_pairs, naive_pairs);
    }

    #[test]
    fn test_empty_grid_query() {
        let grid = SpatialHash::<u32>::new(10.0);
        let results = grid.query(AABB::from_sphere(Vec3::ZERO, 5.0));
        assert!(results.is_empty(), "Empty grid should return empty query");
    }

    #[test]
    fn test_multiple_insertions_same_id() {
        let mut grid = SpatialHash::<u32>::new(10.0);

        // Insert same ID multiple times
        grid.insert(1, AABB::from_sphere(Vec3::new(0.0, 0.0, 0.0), 1.0));
        grid.insert(1, AABB::from_sphere(Vec3::new(20.0, 20.0, 20.0), 1.0));

        // Object count is 2 (we track insertions, not unique IDs)
        assert_eq!(grid.object_count(), 2);

        // Both positions should be queryable
        let results1 = grid.query(AABB::from_sphere(Vec3::ZERO, 2.0));
        let results2 = grid.query(AABB::from_sphere(Vec3::new(20.0, 20.0, 20.0), 2.0));
        
        assert!(results1.contains(&1));
        assert!(results2.contains(&1));
    }
}

