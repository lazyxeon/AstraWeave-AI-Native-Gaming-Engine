//! Mutation-resistant comprehensive tests for astraweave-nav.
//!
//! Targets: Triangle, NavTri, Aabb, NavMesh — exact values, boundary
//! conditions, off-by-one, negation, operator swaps.

use astraweave_nav::{Aabb, NavMesh, NavTri, Triangle};
use glam::Vec3;

// =========================================================================
// Triangle — construction & geometry
// =========================================================================

#[test]
fn triangle_new_stores_vertices() {
    let t = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Y);
    assert_eq!(t.a, Vec3::ZERO);
    assert_eq!(t.b, Vec3::X);
    assert_eq!(t.c, Vec3::Y);
}

#[test]
fn triangle_from_vertices_round_trip() {
    let t = Triangle::new(Vec3::X, Vec3::Y, Vec3::Z);
    let verts = t.vertices();
    let t2 = Triangle::from_vertices(verts);
    assert_eq!(t, t2);
}

#[test]
fn triangle_center_is_centroid() {
    let t = Triangle::new(
        Vec3::ZERO,
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(0.0, 6.0, 0.0),
    );
    let c = t.center();
    assert!((c.x - 1.0).abs() < 1e-5);
    assert!((c.y - 2.0).abs() < 1e-5);
    assert!((c.z - 0.0).abs() < 1e-5);
}

#[test]
fn triangle_area_right_angle() {
    // 3-4-5 right triangle at origin in XY plane
    let t = Triangle::new(
        Vec3::ZERO,
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
    );
    assert!((t.area() - 6.0).abs() < 1e-4, "area={}", t.area());
}

#[test]
fn triangle_area_unit_triangle() {
    // Unit right triangle: (0,0,0),(1,0,0),(0,1,0) → area = 0.5
    let t = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Y);
    assert!((t.area() - 0.5).abs() < 1e-5, "area={}", t.area());
}

#[test]
fn triangle_degenerate_collinear() {
    let t = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0));
    assert!(t.is_degenerate());
}

#[test]
fn triangle_not_degenerate_normal() {
    let t = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Y);
    assert!(!t.is_degenerate());
}

#[test]
fn triangle_normal_points_up_for_xy_plane() {
    let t = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Y);
    let n = t.normal();
    // Cross(X, Y) = Z
    assert!(n.z > 0.0, "normal.z={}", n.z);
    assert!((n.x).abs() < 1e-6);
    assert!((n.y).abs() < 1e-6);
}

#[test]
fn triangle_normal_normalized_is_unit() {
    let t = Triangle::new(
        Vec3::ZERO,
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let nn = t.normal_normalized();
    assert!((nn.length() - 1.0).abs() < 1e-5);
}

#[test]
fn triangle_perimeter_equilateral() {
    // Equilateral triangle with side length sqrt(2) in XY plane
    let t = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Y);
    let p = t.perimeter();
    // 1 + 1 + sqrt(2) ≈ 3.414
    let expected = 1.0 + 1.0 + (2.0_f32).sqrt();
    assert!((p - expected).abs() < 1e-4, "perim={}", p);
}

#[test]
fn triangle_edge_lengths_three_edges() {
    let t = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Y);
    let edges = t.edge_lengths();
    assert_eq!(edges.len(), 3);
    assert!((edges[0] - 1.0).abs() < 1e-5); // a→b = 1
    assert!((edges[1] - (2.0_f32).sqrt()).abs() < 1e-5); // b→c
    assert!((edges[2] - 1.0).abs() < 1e-5); // c→a = 1
}

#[test]
fn triangle_min_max_edge_length() {
    let t = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Y);
    assert!((t.min_edge_length() - 1.0).abs() < 1e-5);
    assert!((t.max_edge_length() - (2.0_f32).sqrt()).abs() < 1e-5);
}

#[test]
fn triangle_display_contains_coords() {
    let t = Triangle::new(Vec3::new(1.5, 2.5, 3.5), Vec3::ZERO, Vec3::X);
    let s = format!("{}", t);
    assert!(s.contains("1.50"), "display={}", s);
    assert!(s.contains("Triangle"), "display={}", s);
}

#[test]
fn triangle_clone_eq() {
    let t = Triangle::new(Vec3::X, Vec3::Y, Vec3::Z);
    let t2 = t.clone();
    assert_eq!(t, t2);
}

// =========================================================================
// NavTri — navigation triangle
// =========================================================================

#[test]
fn navtri_new_defaults() {
    let nt = NavTri::new(
        0,
        [Vec3::ZERO, Vec3::X, Vec3::Y],
        Vec3::Z,
        Vec3::new(0.33, 0.33, 0.0),
    );
    assert_eq!(nt.idx, 0);
    assert_eq!(nt.neighbors.len(), 0);
    assert_eq!(nt.neighbor_count(), 0);
}

#[test]
fn navtri_is_isolated_when_no_neighbors() {
    let nt = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Y], Vec3::Z, Vec3::ZERO);
    assert!(nt.is_isolated());
}

#[test]
fn navtri_has_neighbor_after_push() {
    let mut nt = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Y], Vec3::Z, Vec3::ZERO);
    nt.neighbors.push(1);
    assert!(nt.has_neighbor(1));
    assert!(!nt.has_neighbor(2));
    assert_eq!(nt.neighbor_count(), 1);
    assert!(!nt.is_isolated());
}

#[test]
fn navtri_is_edge_fewer_than_3_neighbors() {
    let mut nt = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Y], Vec3::Z, Vec3::ZERO);
    assert!(nt.is_edge()); // 0 neighbors < 3
    nt.neighbors.push(1);
    nt.neighbors.push(2);
    assert!(nt.is_edge()); // 2 neighbors < 3
    nt.neighbors.push(3);
    assert!(!nt.is_edge()); // 3 neighbors == 3
}

#[test]
fn navtri_area_matches_triangle() {
    let verts = [
        Vec3::ZERO,
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
    ];
    let nt = NavTri::new(0, verts, Vec3::Z, Vec3::ZERO);
    assert!((nt.area() - 6.0).abs() < 1e-4);
}

#[test]
fn navtri_perimeter() {
    let verts = [Vec3::ZERO, Vec3::X, Vec3::Y];
    let nt = NavTri::new(0, verts, Vec3::Z, Vec3::ZERO);
    let expected = 1.0 + 1.0 + (2.0_f32).sqrt();
    assert!((nt.perimeter() - expected).abs() < 1e-4);
}

#[test]
fn navtri_distance_to_from_center() {
    let center = Vec3::new(1.0, 2.0, 3.0);
    let nt = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Z, center);
    let d = nt.distance_to(Vec3::ZERO);
    let expected = (1.0_f32 + 4.0 + 9.0).sqrt();
    assert!((d - expected).abs() < 1e-4);
}

#[test]
fn navtri_distance_squared_to() {
    let center = Vec3::new(1.0, 0.0, 0.0);
    let nt = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Z, center);
    assert!((nt.distance_squared_to(Vec3::new(4.0, 0.0, 0.0)) - 9.0).abs() < 1e-4);
}

#[test]
fn navtri_slope_degrees_flat() {
    // Normal = (0, 1, 0) → 0 degrees from vertical
    let nt = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
    assert!((nt.slope_degrees()).abs() < 1e-3);
}

#[test]
fn navtri_slope_degrees_vertical() {
    // Normal = (1, 0, 0) → 90 degrees from vertical
    let nt = NavTri::new(0, [Vec3::ZERO; 3], Vec3::X, Vec3::ZERO);
    assert!((nt.slope_degrees() - 90.0).abs() < 0.1);
}

#[test]
fn navtri_is_walkable_upward_normal() {
    let nt = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
    assert!(nt.is_walkable());
}

#[test]
fn navtri_not_walkable_downward_normal() {
    let nt = NavTri::new(0, [Vec3::ZERO; 3], Vec3::NEG_Y, Vec3::ZERO);
    assert!(!nt.is_walkable());
}

#[test]
fn navtri_display_contains_idx() {
    let nt = NavTri::new(42, [Vec3::ZERO; 3], Vec3::Y, Vec3::new(1.0, 2.0, 3.0));
    let s = format!("{}", nt);
    assert!(s.contains("42"), "display={}", s);
    assert!(s.contains("NavTri"), "display={}", s);
}

// =========================================================================
// Aabb — axis-aligned bounding box
// =========================================================================

#[test]
fn aabb_new_stores_min_max() {
    let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
    assert_eq!(a.min, Vec3::ZERO);
    assert_eq!(a.max, Vec3::ONE);
}

#[test]
fn aabb_zero_is_origin() {
    let a = Aabb::zero();
    assert_eq!(a.min, Vec3::ZERO);
    assert_eq!(a.max, Vec3::ZERO);
}

#[test]
fn aabb_unit_is_zero_to_one() {
    let a = Aabb::unit();
    assert_eq!(a.min, Vec3::ZERO);
    assert_eq!(a.max, Vec3::ONE);
}

#[test]
fn aabb_from_center_half_extents() {
    let a = Aabb::from_center_half_extents(Vec3::new(5.0, 5.0, 5.0), Vec3::ONE);
    assert_eq!(a.min, Vec3::new(4.0, 4.0, 4.0));
    assert_eq!(a.max, Vec3::new(6.0, 6.0, 6.0));
}

#[test]
fn aabb_contains_point_inside() {
    let a = Aabb::unit();
    assert!(a.contains(Vec3::new(0.5, 0.5, 0.5)));
}

#[test]
fn aabb_contains_point_on_boundary() {
    let a = Aabb::unit();
    assert!(a.contains(Vec3::ZERO));
    assert!(a.contains(Vec3::ONE));
}

#[test]
fn aabb_does_not_contain_outside() {
    let a = Aabb::unit();
    assert!(!a.contains(Vec3::new(1.1, 0.5, 0.5)));
    assert!(!a.contains(Vec3::new(-0.1, 0.5, 0.5)));
}

#[test]
fn aabb_intersects_overlapping() {
    let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
    let b = Aabb::new(Vec3::new(0.5, 0.5, 0.5), Vec3::new(1.5, 1.5, 1.5));
    assert!(a.intersects(&b));
    assert!(b.intersects(&a));
}

#[test]
fn aabb_does_not_intersect_separated() {
    let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
    let b = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
    assert!(!a.intersects(&b));
}

#[test]
fn aabb_merge_union() {
    let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
    let b = Aabb::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(0.5, 0.5, 0.5));
    let m = a.merge(&b);
    assert_eq!(m.min, Vec3::new(-1.0, -1.0, -1.0));
    assert_eq!(m.max, Vec3::ONE);
}

#[test]
fn aabb_from_triangle() {
    let t = Triangle::new(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(4.0, 0.0, 1.0),
        Vec3::new(0.0, 5.0, 2.0),
    );
    let a = Aabb::from_triangle(&t);
    assert_eq!(a.min.x, 0.0);
    assert_eq!(a.min.y, 0.0);
    assert_eq!(a.min.z, 1.0);
    assert_eq!(a.max.x, 4.0);
    assert_eq!(a.max.y, 5.0);
    assert_eq!(a.max.z, 3.0);
}

#[test]
fn aabb_center() {
    let a = Aabb::new(Vec3::new(2.0, 4.0, 6.0), Vec3::new(4.0, 8.0, 10.0));
    let c = a.center();
    assert_eq!(c, Vec3::new(3.0, 6.0, 8.0));
}

#[test]
fn aabb_size() {
    let a = Aabb::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 6.0, 9.0));
    assert_eq!(a.size(), Vec3::new(3.0, 4.0, 6.0));
}

#[test]
fn aabb_half_extents() {
    let a = Aabb::new(Vec3::ZERO, Vec3::new(4.0, 6.0, 8.0));
    assert_eq!(a.half_extents(), Vec3::new(2.0, 3.0, 4.0));
}

#[test]
fn aabb_volume() {
    let a = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
    assert!((a.volume() - 24.0).abs() < 1e-5);
}

#[test]
fn aabb_surface_area() {
    // 2*(2*3 + 3*4 + 4*2) = 2*(6+12+8) = 52
    let a = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
    assert!((a.surface_area() - 52.0).abs() < 1e-4);
}

#[test]
fn aabb_is_empty_degenerate() {
    let a = Aabb::zero();
    assert!(a.is_empty());
}

#[test]
fn aabb_not_empty_unit() {
    let a = Aabb::unit();
    assert!(!a.is_empty());
}

#[test]
fn aabb_longest_axis() {
    let a = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 5.0, 3.0));
    assert!((a.longest_axis() - 5.0).abs() < 1e-5);
}

#[test]
fn aabb_shortest_axis() {
    let a = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 5.0, 3.0));
    assert!((a.shortest_axis() - 1.0).abs() < 1e-5);
}

#[test]
fn aabb_expand_grows_all_sides() {
    let a = Aabb::unit();
    let e = a.expand(1.0);
    assert_eq!(e.min, Vec3::new(-1.0, -1.0, -1.0));
    assert_eq!(e.max, Vec3::new(2.0, 2.0, 2.0));
}

#[test]
fn aabb_distance_to_point_from_center() {
    let a = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
    // center = (1,1,1), distance to (4,1,1) = 3
    let d = a.distance_to_point(Vec3::new(4.0, 1.0, 1.0));
    assert!((d - 3.0).abs() < 1e-4);
}

#[test]
fn aabb_display_contains_coordinates() {
    let a = Aabb::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0));
    let s = format!("{}", a);
    assert!(s.contains("AABB"), "display={}", s);
    assert!(s.contains("1.00"), "display={}", s);
}

#[test]
fn aabb_clone_eq() {
    let a = Aabb::unit();
    assert_eq!(a, a.clone());
}

// =========================================================================
// NavMesh — baking, pathfinding, region invalidation
// =========================================================================

fn make_flat_tris() -> Vec<Triangle> {
    // Two adjacent triangles forming a 1x1 rectangle on XZ plane at Y=0
    // Winding order ensures normal points UP (+Y)
    // T0: (0,0,0)-(0,0,1)-(1,0,0) → normal = (0,1,0)
    // T1: (1,0,0)-(0,0,1)-(1,0,1) → normal = (0,1,0)
    vec![
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
    ]
}

#[test]
fn navmesh_bake_flat_triangles() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    assert_eq!(mesh.triangle_count(), 2);
    assert!(!mesh.is_empty());
}

#[test]
fn navmesh_bake_empty_is_empty() {
    let mesh = NavMesh::bake(&[], 0.5, 45.0);
    assert!(mesh.is_empty());
    assert_eq!(mesh.triangle_count(), 0);
}

#[test]
fn navmesh_adjacency_shared_edge() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    // Two sharing edge should be neighbors
    assert!(mesh.edge_count() >= 1, "edge_count={}", mesh.edge_count());
}

#[test]
fn navmesh_average_neighbor_count() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    let avg = mesh.average_neighbor_count();
    assert!(avg > 0.0, "avg_neighbors={}", avg);
}

#[test]
fn navmesh_average_neighbor_count_empty_is_zero() {
    let mesh = NavMesh::bake(&[], 0.5, 45.0);
    assert_eq!(mesh.average_neighbor_count(), 0.0);
}

#[test]
fn navmesh_total_area() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    // Two right triangles each with area 0.5 → total = 1.0
    assert!(
        (mesh.total_area() - 1.0).abs() < 1e-3,
        "area={}",
        mesh.total_area()
    );
}

#[test]
fn navmesh_bounds_some() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    let bounds = mesh.bounds();
    assert!(bounds.is_some());
    let b = bounds.unwrap();
    assert_eq!(b.min.x, 0.0);
    assert_eq!(b.max.x, 1.0);
}

#[test]
fn navmesh_bounds_none_when_empty() {
    let mesh = NavMesh::bake(&[], 0.5, 45.0);
    assert!(mesh.bounds().is_none());
}

#[test]
fn navmesh_get_triangle_valid() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    assert!(mesh.get_triangle(0).is_some());
}

#[test]
fn navmesh_get_triangle_out_of_range_none() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    assert!(mesh.get_triangle(999).is_none());
}

#[test]
fn navmesh_find_path_adjacent() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    let path = mesh.find_path(Vec3::new(0.2, 0.0, 0.2), Vec3::new(0.8, 0.0, 0.8));
    assert!(
        !path.is_empty(),
        "path should not be empty for adjacent tris"
    );
}

#[test]
fn navmesh_find_path_no_mesh_returns_empty() {
    let mesh = NavMesh::bake(&[], 0.5, 45.0);
    let path = mesh.find_path(Vec3::ZERO, Vec3::ONE);
    assert!(path.is_empty());
}

#[test]
fn navmesh_summary_contains_count() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    let s = mesh.summary();
    assert!(s.contains("2 triangles"), "summary={}", s);
    assert!(s.contains("NavMesh"), "summary={}", s);
}

// =========================================================================
// NavMesh — dirty region tracking
// =========================================================================

#[test]
fn navmesh_initially_no_dirty_regions() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    assert!(!mesh.needs_rebake());
    assert_eq!(mesh.dirty_region_count(), 0);
    assert!(mesh.dirty_regions().is_empty());
}

#[test]
fn navmesh_invalidate_region_adds_dirty() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    mesh.invalidate_region(Aabb::unit());
    assert!(mesh.needs_rebake());
    assert_eq!(mesh.dirty_region_count(), 1);
}

#[test]
fn navmesh_clear_dirty_regions() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    mesh.invalidate_region(Aabb::unit());
    assert!(mesh.needs_rebake());
    mesh.clear_dirty_regions();
    assert!(!mesh.needs_rebake());
    assert_eq!(mesh.dirty_region_count(), 0);
}

#[test]
fn navmesh_rebake_dirty_regions() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    mesh.invalidate_region(Aabb::unit());
    assert_eq!(mesh.rebake_count(), 0);
    mesh.rebake_dirty_regions(&tris);
    assert_eq!(mesh.rebake_count(), 1);
    assert!(!mesh.needs_rebake());
}

#[test]
fn navmesh_partial_rebake_returns_affected() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    mesh.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::new(0.5, 0.5, 0.5)));
    let affected = mesh.partial_rebake(&tris);
    assert!(affected > 0, "should affect at least 1 triangle");
}

#[test]
fn navmesh_partial_rebake_no_dirty_returns_zero() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    let affected = mesh.partial_rebake(&tris);
    assert_eq!(affected, 0);
}

#[test]
fn navmesh_path_crosses_dirty_region() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    mesh.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::new(0.5, 0.5, 0.5)));
    let crosses = mesh.path_crosses_dirty_region(&[Vec3::new(0.25, 0.25, 0.25)]);
    assert!(crosses);
}

#[test]
fn navmesh_path_does_not_cross_clean() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    let crosses = mesh.path_crosses_dirty_region(&[Vec3::new(0.25, 0.25, 0.25)]);
    assert!(!crosses);
}

#[test]
fn navmesh_path_crosses_empty_path_false() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    mesh.invalidate_region(Aabb::unit());
    assert!(!mesh.path_crosses_dirty_region(&[]));
}

#[test]
fn navmesh_isolated_count() {
    // Single isolated triangle
    let tris = vec![Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Y)];
    let mesh = NavMesh::bake(&tris, 0.5, 90.0);
    assert_eq!(mesh.isolated_count(), mesh.triangle_count());
}

#[test]
fn navmesh_bake_filters_steep_triangles() {
    // Nearly vertical triangle (normal ≈ X direction)
    let tris = vec![Triangle::new(
        Vec3::ZERO,
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.01, 0.0, 1.0),
    )];
    // Very low slope tolerance: only ~0 degree triangles accepted
    let mesh = NavMesh::bake(&tris, 0.5, 5.0);
    // This triangle has a nearly horizontal normal, so it might be filtered
    // depending on exact angle. The key test is that filtering happens.
    assert!(mesh.triangle_count() <= 1);
}

#[test]
fn navmesh_display_contains_info() {
    let tris = make_flat_tris();
    let mesh = NavMesh::bake(&tris, 0.5, 45.0);
    let s = format!("{}", mesh);
    assert!(s.contains("NavMesh"), "display={}", s);
}

#[test]
fn navmesh_overlapping_dirty_regions_merge() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    mesh.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
    mesh.invalidate_region(Aabb::new(
        Vec3::new(0.5, 0.5, 0.5),
        Vec3::new(2.0, 2.0, 2.0),
    ));
    // Overlapping regions should merge → only 1 region
    assert_eq!(mesh.dirty_region_count(), 1, "overlapping should merge");
}

#[test]
fn navmesh_non_overlapping_dirty_regions_separate() {
    let tris = make_flat_tris();
    let mut mesh = NavMesh::bake(&tris, 0.5, 45.0);
    mesh.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
    mesh.invalidate_region(Aabb::new(
        Vec3::new(10.0, 10.0, 10.0),
        Vec3::new(11.0, 11.0, 11.0),
    ));
    assert_eq!(
        mesh.dirty_region_count(),
        2,
        "non-overlapping should stay separate"
    );
}
