//! Mutation-Resistant Tests for astraweave-nav
//!
//! These tests verify exact computed values (not just structural checks)
//! to ensure they will catch mutations during cargo mutants testing.

use astraweave_nav::{Aabb, NavMesh, NavTri, Triangle};
use glam::Vec3;

// =============================================================================
// TRIANGLE GEOMETRY TESTS
// =============================================================================

mod triangle_geometry_tests {
    use super::*;

    // ---------------------------------------------------------------------
    // center() - Centroid calculation: (a + b + c) / 3.0
    // ---------------------------------------------------------------------

    #[test]
    fn center_of_unit_triangle_at_origin() {
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
        );
        let c = t.center();
        // (0+3+0)/3 = 1, (0+0+3)/3 = 1, (0+0+0)/3 = 0
        assert!((c.x - 1.0).abs() < 1e-6, "center x should be 1.0, got {}", c.x);
        assert!((c.y - 1.0).abs() < 1e-6, "center y should be 1.0, got {}", c.y);
        assert!((c.z - 0.0).abs() < 1e-6, "center z should be 0.0, got {}", c.z);
    }

    #[test]
    fn center_of_offset_triangle() {
        let t = Triangle::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let c = t.center();
        // (1+4+7)/3 = 4, (2+5+8)/3 = 5, (3+6+9)/3 = 6
        assert!((c.x - 4.0).abs() < 1e-6, "center x should be 4.0, got {}", c.x);
        assert!((c.y - 5.0).abs() < 1e-6, "center y should be 5.0, got {}", c.y);
        assert!((c.z - 6.0).abs() < 1e-6, "center z should be 6.0, got {}", c.z);
    }

    #[test]
    fn center_of_negative_coords_triangle() {
        let t = Triangle::new(
            Vec3::new(-3.0, 0.0, 0.0),
            Vec3::new(0.0, -3.0, 0.0),
            Vec3::new(0.0, 0.0, -3.0),
        );
        let c = t.center();
        // (-3+0+0)/3 = -1, (0-3+0)/3 = -1, (0+0-3)/3 = -1
        assert!((c.x - (-1.0)).abs() < 1e-6, "center x should be -1.0, got {}", c.x);
        assert!((c.y - (-1.0)).abs() < 1e-6, "center y should be -1.0, got {}", c.y);
        assert!((c.z - (-1.0)).abs() < 1e-6, "center z should be -1.0, got {}", c.z);
    }

    // ---------------------------------------------------------------------
    // area() - Area = |cross(b-a, c-a)| * 0.5
    // ---------------------------------------------------------------------

    #[test]
    fn area_of_right_triangle_base_3_height_4() {
        // Triangle: origin, (3, 0, 0), (0, 4, 0)
        // Area = 0.5 * base * height = 0.5 * 3 * 4 = 6.0
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        let area = t.area();
        assert!((area - 6.0).abs() < 1e-6, "area should be 6.0, got {}", area);
    }

    #[test]
    fn area_of_unit_right_triangle() {
        // Triangle: origin, (1, 0, 0), (0, 1, 0)
        // Area = 0.5 * 1 * 1 = 0.5
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let area = t.area();
        assert!((area - 0.5).abs() < 1e-6, "area should be 0.5, got {}", area);
    }

    #[test]
    fn area_of_equilateral_triangle_side_2() {
        // Equilateral triangle with side length 2
        // Area = (sqrt(3)/4) * s^2 = (sqrt(3)/4) * 4 = sqrt(3) ≈ 1.732
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(1.0, 3.0_f32.sqrt(), 0.0),
        );
        let area = t.area();
        let expected = 3.0_f32.sqrt(); // sqrt(3) ≈ 1.732
        assert!((area - expected).abs() < 1e-5, "area should be {}, got {}", expected, area);
    }

    #[test]
    fn area_of_degenerate_collinear_points() {
        // All points on a line = area 0
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        );
        let area = t.area();
        assert!(area < 1e-6, "degenerate triangle area should be ~0, got {}", area);
    }

    // ---------------------------------------------------------------------
    // perimeter() - Sum of edge lengths
    // ---------------------------------------------------------------------

    #[test]
    fn perimeter_of_3_4_5_right_triangle() {
        // Right triangle with legs 3 and 4, hypotenuse 5
        // Perimeter = 3 + 4 + 5 = 12
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        let perimeter = t.perimeter();
        assert!((perimeter - 12.0).abs() < 1e-5, "perimeter should be 12.0, got {}", perimeter);
    }

    #[test]
    fn perimeter_of_equilateral_triangle_side_3() {
        // Equilateral triangle with side length 3
        // Perimeter = 3 * 3 = 9
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(1.5, (3.0_f32.powi(2) - 1.5_f32.powi(2)).sqrt(), 0.0),
        );
        let perimeter = t.perimeter();
        assert!((perimeter - 9.0).abs() < 1e-5, "perimeter should be 9.0, got {}", perimeter);
    }

    // ---------------------------------------------------------------------
    // edge_lengths() - Returns [ab, bc, ca]
    // ---------------------------------------------------------------------

    #[test]
    fn edge_lengths_of_3_4_5_triangle() {
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        let edges = t.edge_lengths();
        // ab = distance(a, b) = 3.0
        // bc = distance(b, c) = sqrt(9+16) = 5.0
        // ca = distance(c, a) = 4.0
        assert!((edges[0] - 3.0).abs() < 1e-5, "edge ab should be 3.0, got {}", edges[0]);
        assert!((edges[1] - 5.0).abs() < 1e-5, "edge bc should be 5.0, got {}", edges[1]);
        assert!((edges[2] - 4.0).abs() < 1e-5, "edge ca should be 4.0, got {}", edges[2]);
    }

    #[test]
    fn min_edge_length_is_3() {
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        let min_edge = t.min_edge_length();
        assert!((min_edge - 3.0).abs() < 1e-5, "min edge should be 3.0, got {}", min_edge);
    }

    #[test]
    fn max_edge_length_is_5() {
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        let max_edge = t.max_edge_length();
        assert!((max_edge - 5.0).abs() < 1e-5, "max edge should be 5.0, got {}", max_edge);
    }

    // ---------------------------------------------------------------------
    // normal() - Cross product (b-a) × (c-a)
    // ---------------------------------------------------------------------

    #[test]
    fn normal_of_xy_plane_triangle_points_up() {
        // Triangle in XY plane with CCW winding should have +Z normal
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let n = t.normal();
        // (1,0,0) × (0,1,0) = (0,0,1) with magnitude 1
        assert!(n.z > 0.0, "normal z should be positive, got {}", n.z);
        assert!(n.x.abs() < 1e-6, "normal x should be ~0, got {}", n.x);
        assert!(n.y.abs() < 1e-6, "normal y should be ~0, got {}", n.y);
    }

    #[test]
    fn normal_of_xz_plane_triangle_points_up_y() {
        // Triangle in XZ plane, CCW when viewed from +Y
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        );
        let n = t.normal();
        // (0,0,1) × (1,0,0) = (0,1,0)
        assert!(n.y > 0.0, "normal y should be positive, got {}", n.y);
    }

    #[test]
    fn normal_normalized_has_length_1() {
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(0.0, 5.0, 0.0),
        );
        let n = t.normal_normalized();
        let len = n.length();
        assert!((len - 1.0).abs() < 1e-5, "normalized normal length should be 1.0, got {}", len);
    }

    // ---------------------------------------------------------------------
    // is_degenerate() - Area < 1e-6
    // ---------------------------------------------------------------------

    #[test]
    fn is_degenerate_for_collinear_points() {
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        );
        assert!(t.is_degenerate(), "collinear points should be degenerate");
    }

    #[test]
    fn is_not_degenerate_for_valid_triangle() {
        let t = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        assert!(!t.is_degenerate(), "valid triangle should not be degenerate");
    }
}

// =============================================================================
// AABB GEOMETRY TESTS
// =============================================================================

mod aabb_geometry_tests {
    use super::*;

    // ---------------------------------------------------------------------
    // center() - (min + max) * 0.5
    // ---------------------------------------------------------------------

    #[test]
    fn center_of_unit_cube() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let c = aabb.center();
        // (0+1)/2 = 0.5 for all axes
        assert!((c.x - 0.5).abs() < 1e-6, "center x should be 0.5, got {}", c.x);
        assert!((c.y - 0.5).abs() < 1e-6, "center y should be 0.5, got {}", c.y);
        assert!((c.z - 0.5).abs() < 1e-6, "center z should be 0.5, got {}", c.z);
    }

    #[test]
    fn center_of_offset_aabb() {
        let aabb = Aabb::new(Vec3::new(2.0, 4.0, 6.0), Vec3::new(4.0, 8.0, 10.0));
        let c = aabb.center();
        // (2+4)/2=3, (4+8)/2=6, (6+10)/2=8
        assert!((c.x - 3.0).abs() < 1e-6, "center x should be 3.0, got {}", c.x);
        assert!((c.y - 6.0).abs() < 1e-6, "center y should be 6.0, got {}", c.y);
        assert!((c.z - 8.0).abs() < 1e-6, "center z should be 8.0, got {}", c.z);
    }

    // ---------------------------------------------------------------------
    // size() - max - min
    // ---------------------------------------------------------------------

    #[test]
    fn size_of_unit_cube_is_one() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let s = aabb.size();
        assert!((s.x - 1.0).abs() < 1e-6, "size x should be 1.0, got {}", s.x);
        assert!((s.y - 1.0).abs() < 1e-6, "size y should be 1.0, got {}", s.y);
        assert!((s.z - 1.0).abs() < 1e-6, "size z should be 1.0, got {}", s.z);
    }

    #[test]
    fn size_of_rectangular_box() {
        let aabb = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 3.0, 4.0));
        let s = aabb.size();
        assert!((s.x - 2.0).abs() < 1e-6, "size x should be 2.0, got {}", s.x);
        assert!((s.y - 3.0).abs() < 1e-6, "size y should be 3.0, got {}", s.y);
        assert!((s.z - 4.0).abs() < 1e-6, "size z should be 4.0, got {}", s.z);
    }

    // ---------------------------------------------------------------------
    // half_extents() - size() * 0.5
    // ---------------------------------------------------------------------

    #[test]
    fn half_extents_of_2x4x6_box() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 4.0, 6.0));
        let he = aabb.half_extents();
        assert!((he.x - 1.0).abs() < 1e-6, "half_extent x should be 1.0, got {}", he.x);
        assert!((he.y - 2.0).abs() < 1e-6, "half_extent y should be 2.0, got {}", he.y);
        assert!((he.z - 3.0).abs() < 1e-6, "half_extent z should be 3.0, got {}", he.z);
    }

    // ---------------------------------------------------------------------
    // volume() - s.x * s.y * s.z
    // ---------------------------------------------------------------------

    #[test]
    fn volume_of_unit_cube_is_1() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let v = aabb.volume();
        assert!((v - 1.0).abs() < 1e-6, "volume should be 1.0, got {}", v);
    }

    #[test]
    fn volume_of_2x3x4_box_is_24() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
        let v = aabb.volume();
        // 2 * 3 * 4 = 24
        assert!((v - 24.0).abs() < 1e-6, "volume should be 24.0, got {}", v);
    }

    #[test]
    fn volume_of_zero_aabb_is_0() {
        let aabb = Aabb::zero();
        let v = aabb.volume();
        assert!(v.abs() < 1e-6, "volume of zero AABB should be ~0, got {}", v);
    }

    // ---------------------------------------------------------------------
    // surface_area() - 2 * (xy + yz + zx)
    // ---------------------------------------------------------------------

    #[test]
    fn surface_area_of_unit_cube_is_6() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let sa = aabb.surface_area();
        // 2 * (1*1 + 1*1 + 1*1) = 2 * 3 = 6
        assert!((sa - 6.0).abs() < 1e-6, "surface area should be 6.0, got {}", sa);
    }

    #[test]
    fn surface_area_of_2x3x4_box_is_52() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
        let sa = aabb.surface_area();
        // 2 * (2*3 + 3*4 + 4*2) = 2 * (6 + 12 + 8) = 2 * 26 = 52
        assert!((sa - 52.0).abs() < 1e-6, "surface area should be 52.0, got {}", sa);
    }

    // ---------------------------------------------------------------------
    // is_empty() - min >= max on any axis
    // ---------------------------------------------------------------------

    #[test]
    fn zero_aabb_is_empty() {
        let aabb = Aabb::zero();
        assert!(aabb.is_empty(), "zero AABB should be empty");
    }

    #[test]
    fn unit_aabb_is_not_empty() {
        let aabb = Aabb::unit();
        assert!(!aabb.is_empty(), "unit AABB should not be empty");
    }

    #[test]
    fn inverted_aabb_is_empty() {
        let aabb = Aabb::new(Vec3::ONE, Vec3::ZERO);
        assert!(aabb.is_empty(), "inverted AABB should be empty");
    }

    // ---------------------------------------------------------------------
    // longest_axis() and shortest_axis()
    // ---------------------------------------------------------------------

    #[test]
    fn longest_axis_of_1x2x3_box_is_3() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0));
        let longest = aabb.longest_axis();
        assert!((longest - 3.0).abs() < 1e-6, "longest axis should be 3.0, got {}", longest);
    }

    #[test]
    fn shortest_axis_of_1x2x3_box_is_1() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0));
        let shortest = aabb.shortest_axis();
        assert!((shortest - 1.0).abs() < 1e-6, "shortest axis should be 1.0, got {}", shortest);
    }

    // ---------------------------------------------------------------------
    // expand() - expands by amount on all sides
    // ---------------------------------------------------------------------

    #[test]
    fn expand_unit_cube_by_1() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let expanded = aabb.expand(1.0);
        // min = (0,0,0) - (1,1,1) = (-1,-1,-1)
        // max = (1,1,1) + (1,1,1) = (2,2,2)
        assert!((expanded.min.x - (-1.0)).abs() < 1e-6);
        assert!((expanded.max.x - 2.0).abs() < 1e-6);
        assert!((expanded.min.y - (-1.0)).abs() < 1e-6);
        assert!((expanded.max.y - 2.0).abs() < 1e-6);
    }

    // ---------------------------------------------------------------------
    // from_center_half_extents()
    // ---------------------------------------------------------------------

    #[test]
    fn from_center_half_extents_creates_correct_aabb() {
        let aabb = Aabb::from_center_half_extents(Vec3::new(5.0, 5.0, 5.0), Vec3::new(1.0, 2.0, 3.0));
        // min = center - half_extents = (4, 3, 2)
        // max = center + half_extents = (6, 7, 8)
        assert!((aabb.min.x - 4.0).abs() < 1e-6);
        assert!((aabb.min.y - 3.0).abs() < 1e-6);
        assert!((aabb.min.z - 2.0).abs() < 1e-6);
        assert!((aabb.max.x - 6.0).abs() < 1e-6);
        assert!((aabb.max.y - 7.0).abs() < 1e-6);
        assert!((aabb.max.z - 8.0).abs() < 1e-6);
    }

    // ---------------------------------------------------------------------
    // contains() and intersects()
    // ---------------------------------------------------------------------

    #[test]
    fn contains_center_point() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        assert!(aabb.contains(Vec3::new(1.0, 1.0, 1.0)));
    }

    #[test]
    fn does_not_contain_outside_point() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        assert!(!aabb.contains(Vec3::new(3.0, 1.0, 1.0)));
    }

    #[test]
    fn intersects_overlapping_aabbs() {
        let a = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        let b = Aabb::new(Vec3::ONE, Vec3::new(3.0, 3.0, 3.0));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn does_not_intersect_separated_aabbs() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(6.0, 6.0, 6.0));
        assert!(!a.intersects(&b));
    }
}

// =============================================================================
// NAVTRI TESTS
// =============================================================================

mod navtri_tests {
    use super::*;

    fn create_test_navtri() -> NavTri {
        NavTri::new(
            0,
            [
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(3.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 4.0),
            ],
            Vec3::Y,
            Vec3::new(1.0, 0.0, 4.0 / 3.0),
        )
    }

    #[test]
    fn navtri_area_is_6() {
        let tri = create_test_navtri();
        let area = tri.area();
        // 0.5 * 3 * 4 = 6.0
        assert!((area - 6.0).abs() < 1e-5, "area should be 6.0, got {}", area);
    }

    #[test]
    fn navtri_perimeter_is_12() {
        let tri = create_test_navtri();
        let perimeter = tri.perimeter();
        // 3 + 4 + 5 = 12
        assert!((perimeter - 12.0).abs() < 1e-5, "perimeter should be 12.0, got {}", perimeter);
    }

    #[test]
    fn navtri_distance_to_origin_is_correct() {
        let tri = create_test_navtri();
        let center = Vec3::new(1.0, 0.0, 4.0 / 3.0);
        let dist = tri.distance_to(Vec3::ZERO);
        let expected = center.distance(Vec3::ZERO);
        assert!((dist - expected).abs() < 1e-5, "distance should be {}, got {}", expected, dist);
    }

    #[test]
    fn navtri_slope_degrees_for_flat_is_0() {
        // Normal pointing straight up (+Y)
        let tri = NavTri::new(
            0,
            [Vec3::ZERO, Vec3::X, Vec3::Z],
            Vec3::Y,
            Vec3::ZERO,
        );
        let slope = tri.slope_degrees();
        assert!(slope.abs() < 1e-3, "flat triangle slope should be ~0, got {}", slope);
    }

    #[test]
    fn navtri_slope_degrees_for_vertical_is_90() {
        // Normal pointing horizontal (+X)
        let tri = NavTri::new(
            0,
            [Vec3::ZERO, Vec3::Y, Vec3::Z],
            Vec3::X,
            Vec3::ZERO,
        );
        let slope = tri.slope_degrees();
        assert!((slope - 90.0).abs() < 1e-3, "vertical slope should be ~90, got {}", slope);
    }

    #[test]
    fn navtri_is_walkable_for_upward_normal() {
        let tri = NavTri::new(
            0,
            [Vec3::ZERO, Vec3::X, Vec3::Z],
            Vec3::Y,
            Vec3::ZERO,
        );
        assert!(tri.is_walkable(), "triangle with +Y normal should be walkable");
    }

    #[test]
    fn navtri_is_not_walkable_for_downward_normal() {
        let tri = NavTri::new(
            0,
            [Vec3::ZERO, Vec3::X, Vec3::Z],
            -Vec3::Y,
            Vec3::ZERO,
        );
        assert!(!tri.is_walkable(), "triangle with -Y normal should not be walkable");
    }

    #[test]
    fn navtri_neighbor_count_initially_zero() {
        let tri = create_test_navtri();
        assert_eq!(tri.neighbor_count(), 0);
    }

    #[test]
    fn navtri_is_isolated_initially() {
        let tri = create_test_navtri();
        assert!(tri.is_isolated());
    }

    #[test]
    fn navtri_is_edge_with_less_than_3_neighbors() {
        let mut tri = create_test_navtri();
        tri.neighbors.push(1);
        tri.neighbors.push(2);
        assert!(tri.is_edge(), "triangle with 2 neighbors should be edge");
    }

    #[test]
    fn navtri_is_not_edge_with_3_neighbors() {
        let mut tri = create_test_navtri();
        tri.neighbors.push(1);
        tri.neighbors.push(2);
        tri.neighbors.push(3);
        assert!(!tri.is_edge(), "triangle with 3 neighbors should not be edge");
    }
}

// =============================================================================
// NAVMESH STATISTICS TESTS
// =============================================================================

mod navmesh_stats_tests {
    use super::*;

    fn create_two_connected_triangles() -> Vec<Triangle> {
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
    fn triangle_count_is_2() {
        let nav = NavMesh::bake(&create_two_connected_triangles(), 0.4, 60.0);
        assert_eq!(nav.triangle_count(), 2);
    }

    #[test]
    fn edge_count_is_1() {
        let nav = NavMesh::bake(&create_two_connected_triangles(), 0.4, 60.0);
        // 2 triangles, each has 1 neighbor, so 2 total neighbors / 2 = 1 edge
        assert_eq!(nav.edge_count(), 1);
    }

    #[test]
    fn average_neighbor_count_is_1() {
        let nav = NavMesh::bake(&create_two_connected_triangles(), 0.4, 60.0);
        // Each triangle has 1 neighbor, so average = 2/2 = 1.0
        let avg = nav.average_neighbor_count();
        assert!((avg - 1.0).abs() < 1e-6, "average neighbor count should be 1.0, got {}", avg);
    }

    #[test]
    fn isolated_count_is_0() {
        let nav = NavMesh::bake(&create_two_connected_triangles(), 0.4, 60.0);
        assert_eq!(nav.isolated_count(), 0);
    }

    #[test]
    fn total_area_is_correct() {
        let nav = NavMesh::bake(&create_two_connected_triangles(), 0.4, 60.0);
        // Each triangle has area 0.5 (1x1 right triangle), so total = 1.0
        let area = nav.total_area();
        assert!((area - 1.0).abs() < 1e-5, "total area should be 1.0, got {}", area);
    }

    #[test]
    fn bounds_is_correct() {
        let nav = NavMesh::bake(&create_two_connected_triangles(), 0.4, 60.0);
        let bounds = nav.bounds().unwrap();
        // All triangles span (0,0,0) to (1,0,1)
        assert!((bounds.min.x - 0.0).abs() < 1e-5);
        assert!((bounds.max.x - 1.0).abs() < 1e-5);
        assert!((bounds.min.z - 0.0).abs() < 1e-5);
        assert!((bounds.max.z - 1.0).abs() < 1e-5);
    }

    #[test]
    fn empty_navmesh_has_no_bounds() {
        let nav = NavMesh::bake(&[], 0.4, 60.0);
        assert!(nav.bounds().is_none());
    }

    #[test]
    fn is_empty_for_empty_navmesh() {
        let nav = NavMesh::bake(&[], 0.4, 60.0);
        assert!(nav.is_empty());
    }

    #[test]
    fn is_not_empty_for_valid_navmesh() {
        let nav = NavMesh::bake(&create_two_connected_triangles(), 0.4, 60.0);
        assert!(!nav.is_empty());
    }
}

// =============================================================================
// REGION INVALIDATION TESTS
// =============================================================================

mod region_invalidation_tests {
    use super::*;

    fn create_single_triangle() -> Vec<Triangle> {
        vec![Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )]
    }

    #[test]
    fn needs_rebake_is_false_initially() {
        let nav = NavMesh::bake(&create_single_triangle(), 0.4, 60.0);
        assert!(!nav.needs_rebake());
    }

    #[test]
    fn dirty_region_count_is_0_initially() {
        let nav = NavMesh::bake(&create_single_triangle(), 0.4, 60.0);
        assert_eq!(nav.dirty_region_count(), 0);
    }

    #[test]
    fn invalidate_region_adds_dirty_region() {
        let mut nav = NavMesh::bake(&create_single_triangle(), 0.4, 60.0);
        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        assert!(nav.needs_rebake());
        assert_eq!(nav.dirty_region_count(), 1);
    }

    #[test]
    fn clear_dirty_regions_clears_all() {
        let mut nav = NavMesh::bake(&create_single_triangle(), 0.4, 60.0);
        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        nav.clear_dirty_regions();
        assert!(!nav.needs_rebake());
        assert_eq!(nav.dirty_region_count(), 0);
    }

    #[test]
    fn rebake_count_initially_zero() {
        let nav = NavMesh::bake(&create_single_triangle(), 0.4, 60.0);
        assert_eq!(nav.rebake_count(), 0);
    }

    #[test]
    fn rebake_dirty_regions_increments_count() {
        let mut nav = NavMesh::bake(&create_single_triangle(), 0.4, 60.0);
        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        nav.rebake_dirty_regions(&create_single_triangle());
        assert_eq!(nav.rebake_count(), 1);
        assert!(!nav.needs_rebake()); // Should be clean after rebake
    }
}
