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
use astraweave_physics::SpatialHash;

// Create grid with 10-unit cells (adjust based on object sizes)
let mut grid = SpatialHash::new(10.0);

// Insert objects
for (id, aabb) in objects {
    grid.insert(id, aabb);
}

// Query potential collisions for an object
let candidates = grid.query(aabb);

// Only test narrow-phase collision against candidates (not all objects)
for candidate_id in candidates {
    if detailed_collision_test(aabb, candidate_aabb) {
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
    /// let aabb = AABB::from_sphere(Vec3::new(5.0, 0.0, 5.0), 1.0);
    /// grid.insert(entity_id, aabb);
    /// ```
    pub fn insert(&mut self, id: T, aabb: AABB) {
        let cells = self.get_overlapping_cells(&aabb);
        
        for cell in cells {
            self.grid
                .entry(cell)
                .or_insert_with(Vec::new)
                .push(id);
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

    #[test]
    fn test_aabb_intersection() {
        let aabb1 = AABB::from_sphere(Vec3::ZERO, 1.0);
        let aabb2 = AABB::from_sphere(Vec3::new(1.5, 0.0, 0.0), 1.0);
        let aabb3 = AABB::from_sphere(Vec3::new(5.0, 0.0, 0.0), 1.0);

        assert!(aabb1.intersects(&aabb2), "Adjacent AABBs should intersect");
        assert!(!aabb1.intersects(&aabb3), "Distant AABBs should not intersect");
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
        assert!(grid.cell_count() >= 27, "Large object should span multiple cells");
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

        assert_eq!(cell1, cell2, "Points in same cell should have same cell coords");
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
}
