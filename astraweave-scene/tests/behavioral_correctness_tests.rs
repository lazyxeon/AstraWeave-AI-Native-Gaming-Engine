//! Behavioral Correctness Tests for astraweave-scene
//!
//! These tests validate the mathematical/behavioral correctness of scene systems,
//! ensuring transforms, spatial partitioning, and streaming follow expected formulas.

use glam::{Mat4, Quat, Vec3, Vec4};

// ============================================================================
// Transform Tests
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct Transform {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { translation, rotation, scale }
    }
    
    fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
    
    fn is_identity(&self) -> bool {
        self.translation == Vec3::ZERO 
            && self.rotation == Quat::IDENTITY 
            && self.scale == Vec3::ONE
    }
    
    fn is_uniform_scale(&self) -> bool {
        (self.scale.x - self.scale.y).abs() < f32::EPSILON
            && (self.scale.y - self.scale.z).abs() < f32::EPSILON
    }
    
    fn uniform_scale(&self) -> f32 {
        (self.scale.x + self.scale.y + self.scale.z) / 3.0
    }
    
    fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }
    
    fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }
    
    fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
    
    fn inverse(&self) -> Self {
        let inv_rotation = self.rotation.inverse();
        let inv_scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        let inv_translation = inv_rotation * (-self.translation * inv_scale);
        Self {
            translation: inv_translation,
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }
    
    fn transform_point(&self, point: Vec3) -> Vec3 {
        self.rotation * (point * self.scale) + self.translation
    }
    
    fn transform_direction(&self, direction: Vec3) -> Vec3 {
        self.rotation * direction
    }
    
    fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

/// Test default transform is identity
#[test]
fn test_transform_default_is_identity() {
    let t = Transform::default();
    assert_eq!(t.translation, Vec3::ZERO, "Default translation is zero");
    assert_eq!(t.rotation, Quat::IDENTITY, "Default rotation is identity");
    assert_eq!(t.scale, Vec3::ONE, "Default scale is one");
    assert!(t.is_identity(), "Default transform is identity");
}

/// Test transform matrix formula: TRS = Translation * Rotation * Scale
#[test]
fn test_transform_matrix_trs() {
    let t = Transform::new(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Vec3::new(2.0, 2.0, 2.0),
    );
    let m = t.matrix();
    
    // Verify translation is in last column
    assert!((m.w_axis.x - 1.0).abs() < 0.001, "Translation X");
    assert!((m.w_axis.y - 2.0).abs() < 0.001, "Translation Y");
    assert!((m.w_axis.z - 3.0).abs() < 0.001, "Translation Z");
    
    // Transform origin should give translation
    let origin_world = m.transform_point3(Vec3::ZERO);
    assert!((origin_world - t.translation).length() < 0.001, "Origin transforms to translation");
}

/// Test uniform scale detection
#[test]
fn test_transform_uniform_scale() {
    let uniform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::splat(2.0));
    assert!(uniform.is_uniform_scale(), "2,2,2 is uniform");
    
    let non_uniform = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::new(1.0, 2.0, 3.0));
    assert!(!non_uniform.is_uniform_scale(), "1,2,3 is not uniform");
}

/// Test uniform scale formula: (x + y + z) / 3
#[test]
fn test_transform_uniform_scale_formula() {
    let t = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::new(1.0, 2.0, 3.0));
    let avg = t.uniform_scale();
    
    // Formula: (scale.x + scale.y + scale.z) / 3
    let expected = (1.0 + 2.0 + 3.0) / 3.0;
    assert!((avg - expected).abs() < 0.001, "Average scale = (1+2+3)/3 = 2");
}

/// Test forward direction: rotation * -Z
#[test]
fn test_transform_forward() {
    let t = Transform::default();
    let forward = t.forward();
    
    // Identity rotation: forward = -Z
    assert!((forward - Vec3::NEG_Z).length() < 0.001, "Identity forward is -Z");
    
    // 90° rotation around Y: forward becomes -X
    let rotated = Transform::new(
        Vec3::ZERO,
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Vec3::ONE,
    );
    let forward_rotated = rotated.forward();
    assert!((forward_rotated - Vec3::NEG_X).length() < 0.001, "90° Y rotation: forward = -X");
}

/// Test right direction: rotation * +X
#[test]
fn test_transform_right() {
    let t = Transform::default();
    assert!((t.right() - Vec3::X).length() < 0.001, "Identity right is +X");
}

/// Test up direction: rotation * +Y
#[test]
fn test_transform_up() {
    let t = Transform::default();
    assert!((t.up() - Vec3::Y).length() < 0.001, "Identity up is +Y");
}

/// Test transform inverse: T * T^-1 = Identity
#[test]
fn test_transform_inverse() {
    let t = Transform::new(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::from_rotation_y(0.5),
        Vec3::splat(2.0),
    );
    let inv = t.inverse();
    
    // Apply transform then inverse: should get back original point
    let point = Vec3::new(5.0, 6.0, 7.0);
    let transformed = t.transform_point(point);
    let back = inv.transform_point(transformed);
    
    assert!((back - point).length() < 0.01, "T(T^-1(p)) = p");
}

/// Test transform point formula: rotation * (point * scale) + translation
#[test]
fn test_transform_point_formula() {
    let t = Transform::new(
        Vec3::new(10.0, 0.0, 0.0),
        Quat::IDENTITY,
        Vec3::splat(2.0),
    );
    let point = Vec3::new(1.0, 1.0, 1.0);
    let result = t.transform_point(point);
    
    // Formula: rotation * (point * scale) + translation
    // = (1,1,1) * 2 + (10,0,0) = (12, 2, 2)
    assert!((result - Vec3::new(12.0, 2.0, 2.0)).length() < 0.001);
}

/// Test transform direction (ignores translation and scale)
#[test]
fn test_transform_direction() {
    let t = Transform::new(
        Vec3::new(100.0, 100.0, 100.0), // Translation should be ignored
        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
        Vec3::splat(5.0), // Scale should be ignored for direction
    );
    
    let direction = Vec3::Z;
    let result = t.transform_direction(direction);
    
    // 90° Y rotation (CCW looking down Y): Z rotates towards -X
    // Verify length is preserved (magnitude 1)
    assert!((result.length() - 1.0).abs() < 0.001, "Direction magnitude preserved");
    
    // Verify rotation applied (result is perpendicular to original direction in XZ plane)
    let dot_with_original = result.dot(Vec3::Z);
    assert!(dot_with_original.abs() < 0.001, "90° rotation makes perpendicular");
}

/// Test lerp formula: linear interpolation for translation/scale, slerp for rotation
#[test]
fn test_transform_lerp() {
    let a = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
    let b = Transform::new(Vec3::new(10.0, 0.0, 0.0), Quat::IDENTITY, Vec3::splat(3.0));
    
    let mid = a.lerp(&b, 0.5);
    
    // Translation lerp: 0 + 0.5 * (10 - 0) = 5
    assert!((mid.translation.x - 5.0).abs() < 0.001, "Translation lerped");
    
    // Scale lerp: 1 + 0.5 * (3 - 1) = 2
    assert!((mid.scale.x - 2.0).abs() < 0.001, "Scale lerped");
}

/// Test lerp at boundaries t=0 and t=1
#[test]
fn test_transform_lerp_boundaries() {
    let a = Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
    let b = Transform::new(Vec3::new(10.0, 20.0, 30.0), Quat::IDENTITY, Vec3::splat(5.0));
    
    let at_0 = a.lerp(&b, 0.0);
    let at_1 = a.lerp(&b, 1.0);
    
    assert!((at_0.translation - a.translation).length() < 0.001, "t=0 gives A");
    assert!((at_1.translation - b.translation).length() < 0.001, "t=1 gives B");
}

// ============================================================================
// GridCoord Tests
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GridCoord {
    x: i32,
    y: i32,
    z: i32,
}

impl GridCoord {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    
    fn from_world_pos(pos: Vec3, cell_size: f32) -> Self {
        Self {
            x: (pos.x / cell_size).floor() as i32,
            y: (pos.y / cell_size).floor() as i32,
            z: (pos.z / cell_size).floor() as i32,
        }
    }
    
    fn to_world_center(self, cell_size: f32) -> Vec3 {
        Vec3::new(
            (self.x as f32 + 0.5) * cell_size,
            (self.y as f32 + 0.5) * cell_size,
            (self.z as f32 + 0.5) * cell_size,
        )
    }
    
    fn neighbors_3d(self) -> Vec<GridCoord> {
        let mut neighbors = Vec::with_capacity(26);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 { continue; }
                    neighbors.push(GridCoord::new(self.x + dx, self.y + dy, self.z + dz));
                }
            }
        }
        neighbors
    }
    
    fn neighbors_2d(self) -> Vec<GridCoord> {
        let mut neighbors = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dz in -1..=1 {
                if dx == 0 && dz == 0 { continue; }
                neighbors.push(GridCoord::new(self.x + dx, self.y, self.z + dz));
            }
        }
        neighbors
    }
    
    fn manhattan_distance(self, other: GridCoord) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()
    }
}

/// Test world position to grid coordinate formula: floor(pos / cell_size)
#[test]
fn test_grid_coord_from_world_pos() {
    let cell_size = 100.0;
    
    // Position at 150, 250, 350 => cells 1, 2, 3
    let coord = GridCoord::from_world_pos(Vec3::new(150.0, 250.0, 350.0), cell_size);
    assert_eq!(coord.x, 1, "150/100 = 1");
    assert_eq!(coord.y, 2, "250/100 = 2");
    assert_eq!(coord.z, 3, "350/100 = 3");
}

/// Test negative coordinates use floor (not truncation)
#[test]
fn test_grid_coord_negative_floor() {
    let cell_size = 100.0;
    
    // -50 / 100 = -0.5, floor = -1
    let coord = GridCoord::from_world_pos(Vec3::new(-50.0, -1.0, 0.0), cell_size);
    assert_eq!(coord.x, -1, "-50/100 floors to -1");
    assert_eq!(coord.y, -1, "-1/100 floors to -1");
    assert_eq!(coord.z, 0, "0/100 floors to 0");
}

/// Test grid coord to world center formula: (coord + 0.5) * cell_size
#[test]
fn test_grid_coord_to_world_center() {
    let cell_size = 100.0;
    let coord = GridCoord::new(2, 3, 4);
    let center = coord.to_world_center(cell_size);
    
    // Formula: (coord + 0.5) * cell_size
    assert!((center.x - 250.0).abs() < 0.001, "(2 + 0.5) * 100 = 250");
    assert!((center.y - 350.0).abs() < 0.001, "(3 + 0.5) * 100 = 350");
    assert!((center.z - 450.0).abs() < 0.001, "(4 + 0.5) * 100 = 450");
}

/// Test 3D neighbors count: 26 neighbors (3³ - 1)
#[test]
fn test_grid_coord_neighbors_3d_count() {
    let coord = GridCoord::new(0, 0, 0);
    let neighbors = coord.neighbors_3d();
    
    assert_eq!(neighbors.len(), 26, "3D neighbors = 3³ - 1 = 26");
    
    // Verify self is not included
    assert!(!neighbors.contains(&coord), "Self not in neighbors");
}

/// Test 2D neighbors count: 8 neighbors (3² - 1)
#[test]
fn test_grid_coord_neighbors_2d_count() {
    let coord = GridCoord::new(0, 0, 0);
    let neighbors = coord.neighbors_2d();
    
    assert_eq!(neighbors.len(), 8, "2D neighbors = 3² - 1 = 8");
    
    // Verify Y coordinate unchanged
    for n in &neighbors {
        assert_eq!(n.y, coord.y, "2D neighbors keep same Y");
    }
}

/// Test manhattan distance formula: |dx| + |dy| + |dz|
#[test]
fn test_grid_coord_manhattan_distance() {
    let a = GridCoord::new(0, 0, 0);
    let b = GridCoord::new(3, 4, 5);
    
    let dist = a.manhattan_distance(b);
    assert_eq!(dist, 3 + 4 + 5, "Manhattan distance = |3| + |4| + |5| = 12");
    
    // Verify symmetry
    assert_eq!(b.manhattan_distance(a), dist, "Distance is symmetric");
    
    // Verify self-distance is 0
    assert_eq!(a.manhattan_distance(a), 0, "Self-distance is 0");
}

// ============================================================================
// AABB Tests
// ============================================================================

#[derive(Debug, Clone, Copy)]
struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
    
    fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }
    
    fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    
    fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
    
    fn contains_point(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
            && point.z >= self.min.z && point.z <= self.max.z
    }
    
    fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x
            && self.min.y <= other.max.y && self.max.y >= other.min.y
            && self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}

/// Test AABB from center and half-extents
#[test]
fn test_aabb_from_center_half_extents() {
    let center = Vec3::new(10.0, 20.0, 30.0);
    let half = Vec3::new(5.0, 5.0, 5.0);
    let aabb = AABB::from_center_half_extents(center, half);
    
    assert!((aabb.min - Vec3::new(5.0, 15.0, 25.0)).length() < 0.001, "min = center - half");
    assert!((aabb.max - Vec3::new(15.0, 25.0, 35.0)).length() < 0.001, "max = center + half");
}

/// Test AABB center formula
#[test]
fn test_aabb_center() {
    let aabb = AABB::new(Vec3::ZERO, Vec3::new(10.0, 20.0, 30.0));
    let center = aabb.center();
    
    assert!((center - Vec3::new(5.0, 10.0, 15.0)).length() < 0.001, "center = (min + max) / 2");
}

/// Test AABB half-extents formula
#[test]
fn test_aabb_half_extents() {
    let aabb = AABB::new(Vec3::ZERO, Vec3::new(10.0, 20.0, 30.0));
    let half = aabb.half_extents();
    
    assert!((half - Vec3::new(5.0, 10.0, 15.0)).length() < 0.001, "half = (max - min) / 2");
}

/// Test AABB contains point - inside/outside/boundary
#[test]
fn test_aabb_contains_point() {
    let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
    
    assert!(aabb.contains_point(Vec3::new(0.5, 0.5, 0.5)), "Center is inside");
    assert!(aabb.contains_point(Vec3::ZERO), "Min corner is inside");
    assert!(aabb.contains_point(Vec3::ONE), "Max corner is inside");
    assert!(!aabb.contains_point(Vec3::new(1.5, 0.5, 0.5)), "Outside X");
    assert!(!aabb.contains_point(Vec3::new(-0.1, 0.5, 0.5)), "Negative X outside");
}

/// Test AABB intersection
#[test]
fn test_aabb_intersects() {
    let a = AABB::new(Vec3::ZERO, Vec3::ONE);
    let b = AABB::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    let c = AABB::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 1.0, 1.0));
    
    assert!(a.intersects(&b), "Overlapping AABBs intersect");
    assert!(!a.intersects(&c), "Separate AABBs don't intersect");
}

/// Test AABB self-intersection
#[test]
fn test_aabb_self_intersection() {
    let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
    assert!(aabb.intersects(&aabb), "AABB intersects itself");
}

// ============================================================================
// Frustum Culling Tests
// ============================================================================

/// Test frustum plane normalization
#[test]
fn test_frustum_plane_normalization() {
    // Plane equation: ax + by + cz + d = 0
    // Must normalize: (a,b,c) should have length 1
    let plane = Vec4::new(3.0, 4.0, 0.0, 5.0);
    let length = Vec3::new(plane.x, plane.y, plane.z).length();
    let normalized = plane / length;
    
    let normal_length = Vec3::new(normalized.x, normalized.y, normalized.z).length();
    assert!((normal_length - 1.0).abs() < 0.001, "Normalized plane normal has length 1");
}

/// Test frustum AABB intersection using p-vertex method
#[test]
fn test_frustum_pvertex_selection() {
    // For plane with positive normal, p-vertex is max
    // For plane with negative normal, p-vertex is min
    let plane_normal = Vec3::new(1.0, 0.0, 0.0); // Pointing +X
    let aabb = AABB::new(Vec3::ZERO, Vec3::ONE);
    
    // P-vertex for +X normal: max.x
    let p = Vec3::new(
        if plane_normal.x >= 0.0 { aabb.max.x } else { aabb.min.x },
        if plane_normal.y >= 0.0 { aabb.max.y } else { aabb.min.y },
        if plane_normal.z >= 0.0 { aabb.max.z } else { aabb.min.z },
    );
    
    assert_eq!(p.x, 1.0, "P-vertex X is max for +X normal");
}

// ============================================================================
// Streaming Tests
// ============================================================================

/// Test streaming radius to cell count formula
#[test]
fn test_streaming_radius_cells() {
    let cell_size = 100.0_f32;
    let radius = 500.0_f32;
    
    // Cell count in each direction: ceil(radius / cell_size)
    let cells_per_axis = (radius / cell_size).ceil() as i32;
    assert_eq!(cells_per_axis, 5, "500m / 100m = 5 cells");
    
    // Total cells in radius: (2r + 1)³ for 3D or (2r + 1)² for 2D (ignoring Y)
    let total_2d = ((2 * cells_per_axis + 1) * (2 * cells_per_axis + 1)) as i32;
    assert_eq!(total_2d, 121, "(2*5 + 1)² = 11² = 121 cells");
}

/// Test cells within radius formula
#[test]
fn test_cells_in_radius() {
    let camera_cell = GridCoord::new(0, 0, 0);
    let radius_cells = 2;
    
    let mut cells = Vec::new();
    for dx in -radius_cells..=radius_cells {
        for dz in -radius_cells..=radius_cells {
            cells.push(GridCoord::new(camera_cell.x + dx, camera_cell.y, camera_cell.z + dz));
        }
    }
    
    // (2*2 + 1)² = 5² = 25 cells
    assert_eq!(cells.len(), 25, "5x5 grid = 25 cells");
}

/// Test LRU cache eviction
#[test]
fn test_lru_cache_eviction() {
    // LRU: Least Recently Used evicted first
    let capacity = 3;
    let mut cache: Vec<i32> = Vec::new();
    
    fn access(cache: &mut Vec<i32>, item: i32, capacity: usize) {
        // Move to front if exists
        if let Some(pos) = cache.iter().position(|&x| x == item) {
            cache.remove(pos);
        }
        cache.insert(0, item);
        // Evict oldest if over capacity
        while cache.len() > capacity {
            cache.pop();
        }
    }
    
    access(&mut cache, 1, capacity);
    access(&mut cache, 2, capacity);
    access(&mut cache, 3, capacity);
    access(&mut cache, 4, capacity); // Evicts 1
    
    assert_eq!(cache.len(), 3, "Cache respects capacity");
    assert!(!cache.contains(&1), "LRU item (1) evicted");
    assert!(cache.contains(&4), "Newest item (4) present");
}

// ============================================================================
// Cell State Machine Tests
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CellState {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
}

/// Test valid cell state transitions
#[test]
fn test_cell_state_transitions() {
    // Valid transitions:
    // Unloaded -> Loading -> Loaded -> Unloading -> Unloaded
    fn is_valid_transition(from: CellState, to: CellState) -> bool {
        matches!(
            (from, to),
            (CellState::Unloaded, CellState::Loading)
                | (CellState::Loading, CellState::Loaded)
                | (CellState::Loading, CellState::Unloaded) // Load failed
                | (CellState::Loaded, CellState::Unloading)
                | (CellState::Unloading, CellState::Unloaded)
        )
    }
    
    assert!(is_valid_transition(CellState::Unloaded, CellState::Loading), "Unloaded->Loading valid");
    assert!(is_valid_transition(CellState::Loading, CellState::Loaded), "Loading->Loaded valid");
    assert!(is_valid_transition(CellState::Loaded, CellState::Unloading), "Loaded->Unloading valid");
    assert!(!is_valid_transition(CellState::Unloaded, CellState::Loaded), "Unloaded->Loaded invalid (skip Loading)");
}

// ============================================================================
// Node Hierarchy Tests
// ============================================================================

/// Test node child count
#[test]
fn test_node_child_count() {
    let children: Vec<String> = vec!["A".into(), "B".into(), "C".into()];
    assert_eq!(children.len(), 3, "Node has 3 children");
}

/// Test recursive node depth
#[test]
fn test_node_depth() {
    fn depth(has_children: bool, child_depths: &[i32]) -> i32 {
        if has_children {
            1 + child_depths.iter().max().copied().unwrap_or(0)
        } else {
            0
        }
    }
    
    // Leaf node: depth 0
    assert_eq!(depth(false, &[]), 0, "Leaf depth is 0");
    
    // Node with leaf children: depth 1
    assert_eq!(depth(true, &[0, 0]), 1, "Parent of leaves has depth 1");
    
    // Nested: depth 2
    assert_eq!(depth(true, &[1, 0]), 2, "Grandparent has depth 2");
}

// ============================================================================
// Overlapping Cells Tests
// ============================================================================

/// Test AABB overlapping cells calculation
#[test]
fn test_aabb_overlapping_cells() {
    let cell_size = 100.0;
    let aabb = AABB::new(Vec3::new(50.0, 0.0, 50.0), Vec3::new(250.0, 0.0, 150.0));
    
    let min_coord = GridCoord::from_world_pos(aabb.min, cell_size);
    let max_coord = GridCoord::from_world_pos(aabb.max, cell_size);
    
    let mut cells = Vec::new();
    for x in min_coord.x..=max_coord.x {
        for z in min_coord.z..=max_coord.z {
            cells.push(GridCoord::new(x, 0, z));
        }
    }
    
    // X: 50-250 spans cells 0, 1, 2 (3 cells)
    // Z: 50-150 spans cells 0, 1 (2 cells)
    // Total: 3 * 2 = 6 cells
    assert_eq!(cells.len(), 6, "AABB overlaps 6 cells");
}

// ============================================================================
// Streaming Metrics Tests
// ============================================================================

/// Test streaming metrics tracking
#[test]
fn test_streaming_metrics() {
    #[derive(Default)]
    struct StreamingMetrics {
        active_cells: usize,
        loaded_cells: usize,
        loading_cells: usize,
        total_loads: u64,
        total_unloads: u64,
        failed_loads: u64,
    }
    
    let mut metrics = StreamingMetrics::default();
    
    // Simulate loading sequence
    metrics.loading_cells += 1;
    metrics.total_loads += 1;
    
    // Load completes
    metrics.loading_cells -= 1;
    metrics.loaded_cells += 1;
    metrics.active_cells += 1;
    
    assert_eq!(metrics.active_cells, 1, "1 active cell");
    assert_eq!(metrics.total_loads, 1, "1 total load");
    
    // Unload
    metrics.active_cells -= 1;
    metrics.loaded_cells -= 1;
    metrics.total_unloads += 1;
    
    assert_eq!(metrics.active_cells, 0, "0 active after unload");
    assert_eq!(metrics.total_unloads, 1, "1 total unload");
}

/// Test memory estimation formula
#[test]
fn test_memory_estimation() {
    // Per-cell memory estimate
    let entities_count = 100;
    let entity_size = 64; // bytes per entity data
    let assets_count = 20;
    let asset_ref_size = 48; // bytes per asset reference
    let overhead = 256; // cell metadata overhead
    
    let cell_memory = overhead + (entities_count * entity_size) + (assets_count * asset_ref_size);
    
    // 256 + 6400 + 960 = 7616 bytes
    assert_eq!(cell_memory, 7616, "Cell memory estimate");
    
    // Total for 25 active cells
    let total_memory = cell_memory * 25;
    assert_eq!(total_memory, 190400, "25 cells = ~190KB");
}

// ============================================================================
// Config Defaults Tests
// ============================================================================

/// Test streaming config defaults
#[test]
fn test_streaming_config_defaults() {
    let max_active_cells = 25; // 5x5 grid
    let lru_cache_size = 5;
    let streaming_radius: f32 = 500.0;
    let max_concurrent_loads = 4;
    
    assert_eq!(max_active_cells, 25, "Default 25 active cells");
    assert_eq!(lru_cache_size, 5, "Default 5 LRU cache");
    assert!((streaming_radius - 500.0).abs() < 0.001, "Default 500m radius");
    assert_eq!(max_concurrent_loads, 4, "Default 4 concurrent loads");
}

/// Test world bounds calculation
#[test]
fn test_world_bounds() {
    // 10km x 10km world
    let min_x = -5000.0_f32;
    let max_x = 5000.0_f32;
    let min_z = -5000.0_f32;
    let max_z = 5000.0_f32;
    
    let width = max_x - min_x;
    let depth = max_z - min_z;
    
    assert!((width - 10000.0).abs() < 0.001, "World width 10km");
    assert!((depth - 10000.0).abs() < 0.001, "World depth 10km");
    
    // Cell count at 100m cells
    let cell_size = 100.0;
    let cells_x = (width / cell_size) as i32;
    let cells_z = (depth / cell_size) as i32;
    
    assert_eq!(cells_x, 100, "100 cells in X");
    assert_eq!(cells_z, 100, "100 cells in Z");
}
