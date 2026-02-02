//! Behavioral Correctness Tests for astraweave-nav
//!
//! These tests validate mathematically correct behavior of navigation systems.
//! Focus on geometric calculations, pathfinding correctness, and spatial queries.
//!
//! Coverage targets:
//! - Triangle: Area, normal, centroid calculations
//! - AABB: Contains, intersects, merge operations
//! - NavMesh: Baking, adjacency, pathfinding
//! - A*: Optimal path, heuristic admissibility
//!
//! IMPORTANT: For NavMesh baking, triangles need proper CCW winding to produce 
//! upward (+Y) facing normals. For XZ-plane triangles:
//! - CCW when viewed from +Y: (0,0,0) -> (1,0,0) -> (1,0,1) gives -Y normal (BAD)
//! - CCW when viewed from +Y: (0,0,0) -> (0,0,1) -> (1,0,0) gives +Y normal (GOOD)

use astraweave_nav::{Aabb, NavMesh, NavTri, Triangle};
use glam::Vec3;

// ============================================================================
// TRIANGLE GEOMETRY TESTS
// ============================================================================

/// Test triangle centroid formula: (a + b + c) / 3
#[test]
fn test_triangle_centroid_formula() {
    let tri = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0),
    );

    let center = tri.center();

    // Expected: (0+3+0)/3 = 1, (0+0+3)/3 = 1, (0+0+0)/3 = 0
    assert!(
        (center.x - 1.0).abs() < 0.001,
        "Centroid X should be 1.0"
    );
    assert!(
        (center.y - 1.0).abs() < 0.001,
        "Centroid Y should be 1.0"
    );
    assert!(
        center.z.abs() < 0.001,
        "Centroid Z should be 0.0"
    );
}

/// Test triangle area formula: |cross(b-a, c-a)| / 2
#[test]
fn test_triangle_area_formula() {
    // Right triangle with legs of length 2
    let tri = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
    );

    let area = tri.area();

    // Expected: (base * height) / 2 = (2 * 2) / 2 = 2
    assert!(
        (area - 2.0).abs() < 0.001,
        "Area of right triangle with legs 2 should be 2.0, got {}",
        area
    );
}

/// Test triangle area with unit triangle
#[test]
fn test_triangle_area_unit() {
    let tri = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let area = tri.area();

    // Expected: 0.5
    assert!(
        (area - 0.5).abs() < 0.001,
        "Unit right triangle area should be 0.5"
    );
}

/// Test triangle normal points perpendicular to surface
#[test]
fn test_triangle_normal_perpendicular() {
    // XY plane triangle
    let tri = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    let normal = tri.normal_normalized();

    // Normal should be along Z axis (perpendicular to XY plane)
    assert!(
        normal.x.abs() < 0.001,
        "Normal X should be ~0"
    );
    assert!(
        normal.y.abs() < 0.001,
        "Normal Y should be ~0"
    );
    assert!(
        normal.z.abs() > 0.99,
        "Normal Z should be ~±1"
    );
}

/// Test triangle normal direction follows winding order
#[test]
fn test_triangle_normal_winding_order() {
    // CCW winding -> normal in positive direction
    let tri_ccw = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // CW winding -> normal in negative direction
    let tri_cw = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
    );

    let normal_ccw = tri_ccw.normal_normalized();
    let normal_cw = tri_cw.normal_normalized();

    // Normals should be opposite
    let dot = normal_ccw.dot(normal_cw);
    assert!(
        dot < -0.99,
        "CCW and CW normals should be opposite, dot={}",
        dot
    );
}

/// Test degenerate triangle detection
#[test]
fn test_triangle_degenerate() {
    // Collinear points (zero area)
    let degenerate = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
    );

    assert!(
        degenerate.is_degenerate(),
        "Collinear points should be degenerate"
    );

    // Valid triangle
    let valid = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    assert!(
        !valid.is_degenerate(),
        "Valid triangle should not be degenerate"
    );
}

/// Test triangle perimeter
#[test]
fn test_triangle_perimeter() {
    // Equilateral triangle with side length 1
    let tri = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(0.5, 0.866, 0.0), // Height of equilateral = sqrt(3)/2
    );

    let perimeter = tri.perimeter();

    // Expected: 3 * 1.0 = 3.0 (approximately)
    assert!(
        (perimeter - 3.0).abs() < 0.01,
        "Equilateral triangle perimeter should be ~3.0, got {}",
        perimeter
    );
}

/// Test triangle edge lengths
#[test]
fn test_triangle_edge_lengths() {
    let tri = Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
    );

    let edges = tri.edge_lengths();
    let min_edge = tri.min_edge_length();
    let max_edge = tri.max_edge_length();

    // Edge AB = 3, BC = 5 (hypotenuse), CA = 4
    assert!((edges[0] - 3.0).abs() < 0.001, "AB should be 3.0");
    assert!((edges[1] - 5.0).abs() < 0.001, "BC should be 5.0");
    assert!((edges[2] - 4.0).abs() < 0.001, "CA should be 4.0");
    assert!((min_edge - 3.0).abs() < 0.001, "Min edge should be 3.0");
    assert!((max_edge - 5.0).abs() < 0.001, "Max edge should be 5.0");
}

// ============================================================================
// AABB TESTS
// ============================================================================

/// Test AABB contains point inside
#[test]
fn test_aabb_contains_inside() {
    let aabb = Aabb::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 10.0, 10.0),
    );

    let inside = Vec3::new(5.0, 5.0, 5.0);
    assert!(aabb.contains(inside), "Center point should be inside");

    let corner = Vec3::new(0.0, 0.0, 0.0);
    assert!(aabb.contains(corner), "Corner point should be inside (inclusive)");
}

/// Test AABB contains point outside
#[test]
fn test_aabb_contains_outside() {
    let aabb = Aabb::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 10.0, 10.0),
    );

    let outside = Vec3::new(15.0, 5.0, 5.0);
    assert!(!aabb.contains(outside), "Point outside should not be contained");

    let negative = Vec3::new(-1.0, 5.0, 5.0);
    assert!(!aabb.contains(negative), "Negative point should not be contained");
}

/// Test AABB intersection
#[test]
fn test_aabb_intersects() {
    let a = Aabb::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(10.0, 10.0, 10.0),
    );

    // Overlapping box
    let b = Aabb::new(
        Vec3::new(5.0, 5.0, 5.0),
        Vec3::new(15.0, 15.0, 15.0),
    );
    assert!(a.intersects(&b), "Overlapping boxes should intersect");

    // Touching box (edge contact)
    let c = Aabb::new(
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(20.0, 10.0, 10.0),
    );
    assert!(a.intersects(&c), "Touching boxes should intersect");

    // Separated box
    let d = Aabb::new(
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(30.0, 10.0, 10.0),
    );
    assert!(!a.intersects(&d), "Separated boxes should not intersect");
}

/// Test AABB merge produces bounding box of both
#[test]
fn test_aabb_merge() {
    let a = Aabb::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 5.0, 5.0),
    );

    let b = Aabb::new(
        Vec3::new(3.0, 3.0, 3.0),
        Vec3::new(10.0, 10.0, 10.0),
    );

    let merged = a.merge(&b);

    // Merged should be min(0,3)=0 to max(5,10)=10
    assert_eq!(merged.min.x, 0.0);
    assert_eq!(merged.min.y, 0.0);
    assert_eq!(merged.min.z, 0.0);
    assert_eq!(merged.max.x, 10.0);
    assert_eq!(merged.max.y, 10.0);
    assert_eq!(merged.max.z, 10.0);
}

/// Test AABB center calculation
#[test]
fn test_aabb_center() {
    let aabb = Aabb::new(
        Vec3::new(2.0, 4.0, 6.0),
        Vec3::new(8.0, 10.0, 12.0),
    );

    let center = aabb.center();

    // Expected: (2+8)/2=5, (4+10)/2=7, (6+12)/2=9
    assert!((center.x - 5.0).abs() < 0.001);
    assert!((center.y - 7.0).abs() < 0.001);
    assert!((center.z - 9.0).abs() < 0.001);
}

/// Test AABB size calculation
#[test]
fn test_aabb_size() {
    let aabb = Aabb::new(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(4.0, 7.0, 11.0),
    );

    let size = aabb.size();

    // Expected: 4-1=3, 7-2=5, 11-3=8
    assert!((size.x - 3.0).abs() < 0.001);
    assert!((size.y - 5.0).abs() < 0.001);
    assert!((size.z - 8.0).abs() < 0.001);
}

/// Test AABB volume
#[test]
fn test_aabb_volume() {
    let aabb = Aabb::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 3.0, 4.0),
    );

    let volume = aabb.volume();

    // Expected: 2 * 3 * 4 = 24
    assert!((volume - 24.0).abs() < 0.001);
}

/// Test AABB surface area
#[test]
fn test_aabb_surface_area() {
    let aabb = Aabb::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(2.0, 3.0, 4.0),
    );

    let area = aabb.surface_area();

    // Expected: 2 * (2*3 + 3*4 + 4*2) = 2 * (6 + 12 + 8) = 52
    assert!((area - 52.0).abs() < 0.001);
}

/// Test AABB from triangle
#[test]
fn test_aabb_from_triangle() {
    let tri = Triangle::new(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(5.0, 1.0, 2.0),
        Vec3::new(3.0, 6.0, 4.0),
    );

    let aabb = Aabb::from_triangle(&tri);

    // Min should be (1, 1, 2), Max should be (5, 6, 4)
    assert!((aabb.min.x - 1.0).abs() < 0.001);
    assert!((aabb.min.y - 1.0).abs() < 0.001);
    assert!((aabb.min.z - 2.0).abs() < 0.001);
    assert!((aabb.max.x - 5.0).abs() < 0.001);
    assert!((aabb.max.y - 6.0).abs() < 0.001);
    assert!((aabb.max.z - 4.0).abs() < 0.001);
}

/// Test AABB expand
#[test]
fn test_aabb_expand() {
    let aabb = Aabb::new(
        Vec3::new(5.0, 5.0, 5.0),
        Vec3::new(10.0, 10.0, 10.0),
    );

    let expanded = aabb.expand(2.0);

    // Min should be 5-2=3, Max should be 10+2=12
    assert!((expanded.min.x - 3.0).abs() < 0.001);
    assert!((expanded.max.x - 12.0).abs() < 0.001);
}

/// Test AABB is_empty
#[test]
fn test_aabb_is_empty() {
    let zero = Aabb::zero();
    assert!(zero.is_empty(), "Zero AABB should be empty");

    let inverted = Aabb::new(
        Vec3::new(10.0, 10.0, 10.0),
        Vec3::new(5.0, 5.0, 5.0),
    );
    assert!(inverted.is_empty(), "Inverted AABB should be empty");

    let valid = Aabb::unit();
    assert!(!valid.is_empty(), "Unit AABB should not be empty");
}

/// Test AABB from center and half-extents
#[test]
fn test_aabb_from_center_half_extents() {
    let aabb = Aabb::from_center_half_extents(
        Vec3::new(5.0, 5.0, 5.0),
        Vec3::new(2.0, 2.0, 2.0),
    );

    // Min should be 5-2=3, Max should be 5+2=7
    assert!((aabb.min.x - 3.0).abs() < 0.001);
    assert!((aabb.max.x - 7.0).abs() < 0.001);
    assert!((aabb.center().x - 5.0).abs() < 0.001);
}

// ============================================================================
// NAVTRI TESTS
// ============================================================================

/// Test NavTri slope calculation
#[test]
fn test_navtri_slope_flat() {
    // Flat triangle (normal = +Y)
    let tri = NavTri::new(
        0,
        [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ],
        Vec3::Y,
        Vec3::new(0.33, 0.0, 0.33),
    );

    let slope = tri.slope_degrees();

    // Flat triangle has 0 degrees slope
    assert!(
        slope.abs() < 1.0,
        "Flat triangle should have ~0 degree slope, got {}",
        slope
    );
}

/// Test NavTri slope for 45 degree surface
#[test]
fn test_navtri_slope_45() {
    // Normal at 45 degrees from vertical
    let normal = Vec3::new(1.0, 1.0, 0.0).normalize();
    let tri = NavTri::new(
        0,
        [Vec3::ZERO, Vec3::X, Vec3::Z],
        normal,
        Vec3::ZERO,
    );

    let slope = tri.slope_degrees();

    // Should be close to 45 degrees
    assert!(
        (slope - 45.0).abs() < 1.0,
        "45-degree normal should give ~45 degree slope, got {}",
        slope
    );
}

/// Test NavTri is_walkable
#[test]
fn test_navtri_is_walkable() {
    // Upward-facing (walkable)
    let walkable = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
    assert!(walkable.is_walkable(), "Upward normal should be walkable");

    // Downward-facing (not walkable)
    let not_walkable = NavTri::new(0, [Vec3::ZERO; 3], Vec3::NEG_Y, Vec3::ZERO);
    assert!(
        !not_walkable.is_walkable(),
        "Downward normal should not be walkable"
    );
}

/// Test NavTri distance calculations
#[test]
fn test_navtri_distance() {
    let tri = NavTri::new(
        0,
        [Vec3::ZERO; 3],
        Vec3::Y,
        Vec3::new(5.0, 0.0, 0.0),
    );

    let point = Vec3::new(8.0, 0.0, 4.0);

    let dist = tri.distance_to(point);
    let dist_sq = tri.distance_squared_to(point);

    // Distance from (5,0,0) to (8,0,4) = sqrt(9 + 16) = 5
    assert!((dist - 5.0).abs() < 0.001, "Distance should be 5.0");
    assert!((dist_sq - 25.0).abs() < 0.001, "Distance squared should be 25.0");
}

/// Test NavTri neighbor tracking
#[test]
fn test_navtri_neighbors() {
    let mut tri = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);

    assert!(tri.is_isolated(), "New triangle should be isolated");
    assert!(tri.is_edge(), "Isolated triangle is also an edge triangle");
    assert_eq!(tri.neighbor_count(), 0);

    tri.neighbors.push(1);
    tri.neighbors.push(2);

    assert!(!tri.is_isolated(), "Triangle with neighbors is not isolated");
    assert!(tri.has_neighbor(1), "Should have neighbor 1");
    assert!(!tri.has_neighbor(3), "Should not have neighbor 3");
    assert!(tri.is_edge(), "< 3 neighbors is still edge");

    tri.neighbors.push(3);
    assert!(!tri.is_edge(), "3 neighbors is not an edge triangle");
}

// ============================================================================
// NAVMESH BAKING TESTS
// ============================================================================

/// Test NavMesh bake creates correct number of triangles
/// Triangles need CCW winding to produce +Y normals for walkability
#[test]
fn test_navmesh_bake_count() {
    // CCW winding when viewed from +Y: gives +Y normal (upward-facing)
    let tris = vec![
        Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),  // Changed order for +Y normal
            Vec3::new(1.0, 0.0, 0.0),
        ),
        Triangle::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
        ),
    ];

    let nav = NavMesh::bake(&tris, 0.4, 60.0);

    assert_eq!(
        nav.triangle_count(),
        2,
        "Should have 2 walkable triangles"
    );
}

/// Test NavMesh filters steep slopes
#[test]
fn test_navmesh_filters_steep() {
    let tris = vec![
        // Flat (0 degree slope) - CCW winding for +Y normal
        Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        ),
        // Steep wall (nearly 90 degree slope) - side-facing normal
        // This triangle is in the XY plane with Z-facing normal
        Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ),
    ];

    let nav = NavMesh::bake(&tris, 0.4, 45.0);

    // Only flat triangle should be included (second has ~90° slope)
    assert_eq!(
        nav.triangle_count(),
        1,
        "Only flat triangle should pass 45° slope filter"
    );
}

/// Test NavMesh adjacency detection
#[test]
fn test_navmesh_adjacency() {
    // Two triangles sharing an edge, both with +Y normals (CCW winding)
    let tris = vec![
        Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        ),
        Triangle::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
        ),
    ];

    let nav = NavMesh::bake(&tris, 0.4, 60.0);

    assert_eq!(nav.triangle_count(), 2, "Should have 2 triangles");
    assert_eq!(nav.edge_count(), 1, "Should have 1 shared edge");
    assert!(
        nav.tris[0].has_neighbor(1),
        "Triangle 0 should have neighbor 1"
    );
    assert!(
        nav.tris[1].has_neighbor(0),
        "Triangle 1 should have neighbor 0"
    );
}

/// Test NavMesh bounds calculation
#[test]
fn test_navmesh_bounds() {
    // Single large triangle with +Y normal
    let tris = vec![
        Triangle::new(
            Vec3::new(-5.0, 0.0, -5.0),
            Vec3::new(-5.0, 0.0, 5.0),  // CCW for +Y normal
            Vec3::new(5.0, 0.0, -5.0),
        ),
    ];

    let nav = NavMesh::bake(&tris, 0.4, 60.0);
    
    assert_eq!(nav.triangle_count(), 1, "Should have 1 triangle");
    let bounds = nav.bounds().expect("Should have bounds");

    assert!((bounds.min.x - (-5.0)).abs() < 0.001);
    assert!((bounds.max.x - 5.0).abs() < 0.001);
    assert!((bounds.min.z - (-5.0)).abs() < 0.001);
    assert!((bounds.max.z - 5.0).abs() < 0.001);
}

/// Test NavMesh total area
#[test]
fn test_navmesh_total_area() {
    // Two triangles forming a 1x1 square, both with +Y normals
    let tris = vec![
        Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        ),
        Triangle::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
        ),
    ];

    let nav = NavMesh::bake(&tris, 0.4, 60.0);
    let total_area = nav.total_area();

    // Each triangle has area 0.5, total = 1.0
    assert!(
        (total_area - 1.0).abs() < 0.01,
        "Total area should be ~1.0, got {}",
        total_area
    );
}

// ============================================================================
// PATHFINDING TESTS
// ============================================================================

/// Test pathfinding finds direct path
#[test]
fn test_pathfinding_direct() {
    // Create a strip of 3 connected triangles with +Y normals (CCW winding)
    let tris = vec![
        Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        ),
        Triangle::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
        ),
        Triangle::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(2.0, 0.0, 0.5),
        ),
    ];

    let nav = NavMesh::bake(&tris, 0.4, 60.0);

    assert_eq!(nav.triangle_count(), 3, "Should have 3 walkable triangles");

    let start = Vec3::new(0.25, 0.0, 0.25);
    let goal = Vec3::new(1.5, 0.0, 0.5);

    let path = nav.find_path(start, goal);

    assert!(!path.is_empty(), "Should find a path");
    assert!(path.len() >= 2, "Path should have at least start and goal");

    // First point should be near start, last near goal
    assert!(
        (path.first().unwrap().x - start.x).abs() < 0.5,
        "Path should start near start point"
    );
    assert!(
        (path.last().unwrap().x - goal.x).abs() < 0.5,
        "Path should end near goal point"
    );
}

/// Test pathfinding returns empty for unreachable goal
#[test]
fn test_pathfinding_unreachable() {
    // Two disconnected triangles with +Y normals (CCW winding)
    let tris = vec![
        Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        ),
        Triangle::new(
            Vec3::new(10.0, 0.0, 0.0), // Far away, not connected
            Vec3::new(10.0, 0.0, 1.0),
            Vec3::new(11.0, 0.0, 0.0),
        ),
    ];

    let nav = NavMesh::bake(&tris, 0.4, 60.0);

    assert_eq!(nav.triangle_count(), 2, "Should have 2 triangles");

    // The triangles should have no shared neighbors (they're far apart)
    assert!(
        nav.tris[0].neighbors.is_empty(),
        "First triangle should have no neighbors"
    );
    assert!(
        nav.tris[1].neighbors.is_empty(),
        "Second triangle should have no neighbors"
    );

    let start = Vec3::new(0.25, 0.0, 0.25);
    let goal = Vec3::new(10.25, 0.0, 0.25);

    let path = nav.find_path(start, goal);

    // A* should fail to find a path since triangles are not connected
    // Path should be empty for truly unreachable goals
    assert!(
        path.is_empty(),
        "Path to unreachable goal should be empty, got {} points",
        path.len()
    );
}

/// Test pathfinding on empty mesh
#[test]
fn test_pathfinding_empty_mesh() {
    let nav = NavMesh::bake(&[], 0.4, 60.0);

    let path = nav.find_path(Vec3::ZERO, Vec3::new(10.0, 0.0, 10.0));

    assert!(path.is_empty(), "Empty mesh should return empty path");
}

// ============================================================================
// REGION INVALIDATION TESTS
// ============================================================================

/// Test dirty region marking
#[test]
fn test_navmesh_dirty_regions() {
    // +Y normal triangle (CCW winding)
    let tris = vec![Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
    )];

    let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

    assert!(!nav.needs_rebake(), "Fresh mesh should not need rebake");
    assert_eq!(nav.dirty_region_count(), 0);

    // Mark a region as dirty
    nav.invalidate_region(Aabb::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 5.0, 5.0),
    ));

    assert!(nav.needs_rebake(), "Should need rebake after invalidation");
    assert_eq!(nav.dirty_region_count(), 1);
}

/// Test dirty region merging
#[test]
fn test_navmesh_dirty_region_merge() {
    // +Y normal triangle (CCW winding)
    let tris = vec![Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::new(10.0, 0.0, 0.0),
    )];

    let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

    // Add overlapping regions
    nav.invalidate_region(Aabb::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(5.0, 5.0, 5.0),
    ));
    nav.invalidate_region(Aabb::new(
        Vec3::new(3.0, 3.0, 3.0), // Overlaps with first
        Vec3::new(8.0, 8.0, 8.0),
    ));

    // Should merge into one region
    assert_eq!(
        nav.dirty_region_count(),
        1,
        "Overlapping regions should merge"
    );
}

/// Test rebake clears dirty regions
#[test]
fn test_navmesh_rebake_clears_dirty() {
    // +Y normal triangle (CCW winding)
    let tris = vec![Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
    )];

    let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

    nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::splat(5.0)));
    assert!(nav.needs_rebake());

    nav.rebake_dirty_regions(&tris);

    assert!(!nav.needs_rebake(), "Rebake should clear dirty regions");
    assert_eq!(nav.rebake_count(), 1, "Rebake count should increment");
}

/// Test path crosses dirty region detection
#[test]
fn test_path_crosses_dirty_region() {
    // +Y normal triangle (CCW winding)
    let tris = vec![Triangle::new(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 10.0),
        Vec3::new(10.0, 0.0, 5.0),
    )];

    let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

    let path = vec![
        Vec3::new(1.0, 0.0, 1.0),
        Vec3::new(5.0, 0.0, 5.0),
        Vec3::new(8.0, 0.0, 2.0),
    ];

    assert!(
        !nav.path_crosses_dirty_region(&path),
        "Clean mesh should not cross dirty region"
    );

    // Mark region that intersects with path
    nav.invalidate_region(Aabb::new(
        Vec3::new(4.0, -1.0, 4.0),
        Vec3::new(6.0, 1.0, 6.0),
    ));

    assert!(
        nav.path_crosses_dirty_region(&path),
        "Path through dirty region should be detected"
    );
}
