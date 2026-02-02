//! Mutation-resistant tests for navigation systems.
//!
//! These tests are designed to catch common mutations like:
//! - Boundary condition changes (< vs <=, > vs >=)
//! - Arithmetic operator swaps (+/-, */, etc.)
//! - Boolean logic inversions
//! - Off-by-one errors
//! - Return value mutations

use crate::{Aabb, NavMesh, NavTri, Triangle};
use glam::Vec3;

// ============================================================================
// Triangle Tests - Geometry Calculations
// ============================================================================

mod triangle_tests {
    use super::*;

    #[test]
    fn test_triangle_center_is_average_of_vertices() {
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 3.0, 0.0),
        );
        let center = tri.center();
        // Center = (0+3+0)/3, (0+0+3)/3, 0 = (1, 1, 0)
        assert!((center.x - 1.0).abs() < 1e-6, "center.x should be 1.0, got {}", center.x);
        assert!((center.y - 1.0).abs() < 1e-6, "center.y should be 1.0, got {}", center.y);
        assert!((center.z).abs() < 1e-6, "center.z should be 0.0, got {}", center.z);
    }

    #[test]
    fn test_triangle_normal_direction() {
        // Right-handed triangle in XY plane
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let normal = tri.normal();
        // Cross product of (1,0,0) x (0,1,0) = (0,0,1)
        assert!(normal.z > 0.0, "Normal should point in +Z direction");
    }

    #[test]
    fn test_triangle_normal_normalized_is_unit_length() {
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(0.0, 5.0, 0.0),
        );
        let normal = tri.normal_normalized();
        let length = normal.length();
        assert!((length - 1.0).abs() < 1e-6, "Normalized normal should be unit length, got {}", length);
    }

    #[test]
    fn test_triangle_area_calculation() {
        // Right triangle with legs 3 and 4
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        let area = tri.area();
        // Area = 0.5 * base * height = 0.5 * 3 * 4 = 6
        assert!((area - 6.0).abs() < 1e-6, "Area should be 6.0, got {}", area);
    }

    #[test]
    fn test_triangle_area_half_of_cross_product() {
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
        );
        let area = tri.area();
        // Area = 0.5 * |cross| = 0.5 * 4 = 2
        assert!((area - 2.0).abs() < 1e-6, "Area should be 2.0 (half cross product), got {}", area);
    }

    #[test]
    fn test_triangle_is_degenerate_when_zero_area() {
        // Collinear points
        let degenerate = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
        );
        assert!(degenerate.is_degenerate(), "Collinear points should be degenerate");

        // Non-degenerate
        let valid = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        assert!(!valid.is_degenerate(), "Valid triangle should not be degenerate");
    }

    #[test]
    fn test_triangle_perimeter_sum_of_edges() {
        // Equilateral triangle with side 1
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.5, 0.866025, 0.0), // Height = sqrt(3)/2
        );
        let perimeter = tri.perimeter();
        // Should be approximately 3.0
        assert!((perimeter - 3.0).abs() < 0.01, "Perimeter should be ~3.0, got {}", perimeter);
    }

    #[test]
    fn test_triangle_edge_lengths_order() {
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        let edges = tri.edge_lengths();
        // [ab, bc, ca] = [3, 5, 4]
        assert!((edges[0] - 3.0).abs() < 1e-6, "Edge AB should be 3.0");
        assert!((edges[1] - 5.0).abs() < 1e-6, "Edge BC should be 5.0 (hypotenuse)");
        assert!((edges[2] - 4.0).abs() < 1e-6, "Edge CA should be 4.0");
    }

    #[test]
    fn test_triangle_min_max_edge_lengths() {
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        assert!((tri.min_edge_length() - 3.0).abs() < 1e-6, "Min edge should be 3.0");
        assert!((tri.max_edge_length() - 5.0).abs() < 1e-6, "Max edge should be 5.0");
    }

    #[test]
    fn test_triangle_vertices_and_from_vertices() {
        let original = Triangle::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let vertices = original.vertices();
        let reconstructed = Triangle::from_vertices(vertices);
        assert_eq!(original.a, reconstructed.a);
        assert_eq!(original.b, reconstructed.b);
        assert_eq!(original.c, reconstructed.c);
    }
}

// ============================================================================
// NavTri Tests - Navigation Triangle Properties
// ============================================================================

mod navtri_tests {
    use super::*;

    fn create_test_navtri() -> NavTri {
        let mut tri = NavTri::new(
            0,
            [
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
            Vec3::Y, // Normal pointing up
            Vec3::new(0.33, 0.0, 0.33), // Approximate center
        );
        tri.neighbors = vec![1, 2];
        tri
    }

    #[test]
    fn test_navtri_neighbor_count() {
        let tri = create_test_navtri();
        assert_eq!(tri.neighbor_count(), 2);
    }

    #[test]
    fn test_navtri_has_neighbor() {
        let tri = create_test_navtri();
        assert!(tri.has_neighbor(1), "Should have neighbor 1");
        assert!(tri.has_neighbor(2), "Should have neighbor 2");
        assert!(!tri.has_neighbor(3), "Should not have neighbor 3");
    }

    #[test]
    fn test_navtri_is_isolated() {
        let mut tri = create_test_navtri();
        assert!(!tri.is_isolated(), "Triangle with neighbors should not be isolated");
        
        tri.neighbors.clear();
        assert!(tri.is_isolated(), "Triangle without neighbors should be isolated");
    }

    #[test]
    fn test_navtri_is_edge() {
        let mut tri = create_test_navtri();
        tri.neighbors = vec![1, 2]; // 2 neighbors
        assert!(tri.is_edge(), "Triangle with 2 neighbors is on edge");
        
        tri.neighbors = vec![1, 2, 3]; // 3 neighbors
        assert!(!tri.is_edge(), "Triangle with 3 neighbors is not on edge");
        
        tri.neighbors = vec![1]; // 1 neighbor
        assert!(tri.is_edge(), "Triangle with 1 neighbor is on edge");
    }

    #[test]
    fn test_navtri_area_matches_triangle_formula() {
        let tri = NavTri::new(
            0,
            [
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(3.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 4.0),
            ],
            Vec3::Y,
            Vec3::ZERO,
        );
        let area = tri.area();
        // Area = 0.5 * 3 * 4 = 6
        assert!((area - 6.0).abs() < 1e-6, "Area should be 6.0, got {}", area);
    }

    #[test]
    fn test_navtri_distance_to_point() {
        let tri = NavTri::new(
            0,
            [Vec3::ZERO; 3],
            Vec3::Y,
            Vec3::new(5.0, 0.0, 0.0), // Center at (5, 0, 0)
        );
        let dist = tri.distance_to(Vec3::new(8.0, 0.0, 0.0));
        assert!((dist - 3.0).abs() < 1e-6, "Distance should be 3.0");
    }

    #[test]
    fn test_navtri_distance_squared_to_point() {
        let tri = NavTri::new(
            0,
            [Vec3::ZERO; 3],
            Vec3::Y,
            Vec3::new(0.0, 0.0, 0.0),
        );
        let dist_sq = tri.distance_squared_to(Vec3::new(3.0, 4.0, 0.0));
        // Distance = 5, squared = 25
        assert!((dist_sq - 25.0).abs() < 1e-6, "Distance squared should be 25.0");
    }

    #[test]
    fn test_navtri_slope_degrees() {
        // Flat triangle (normal pointing straight up)
        let flat = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        let slope = flat.slope_degrees();
        assert!((slope - 0.0).abs() < 1e-3, "Flat triangle should have 0 degree slope");

        // 45 degree slope
        let angled = NavTri::new(
            0,
            [Vec3::ZERO; 3],
            Vec3::new(0.707, 0.707, 0.0).normalize(),
            Vec3::ZERO,
        );
        let slope45 = angled.slope_degrees();
        assert!((slope45 - 45.0).abs() < 1.0, "Should be approximately 45 degrees, got {}", slope45);
    }

    #[test]
    fn test_navtri_is_walkable() {
        // Normal pointing up - walkable
        let walkable = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        assert!(walkable.is_walkable(), "Upward-facing triangle should be walkable");

        // Normal pointing down - not walkable
        let ceiling = NavTri::new(0, [Vec3::ZERO; 3], Vec3::NEG_Y, Vec3::ZERO);
        assert!(!ceiling.is_walkable(), "Downward-facing triangle should not be walkable");

        // Vertical wall - not walkable
        let wall = NavTri::new(0, [Vec3::ZERO; 3], Vec3::X, Vec3::ZERO);
        assert!(!wall.is_walkable(), "Vertical wall should not be walkable");
    }
}

// ============================================================================
// AABB Tests - Axis-Aligned Bounding Box
// ============================================================================

mod aabb_tests {
    use super::*;

    #[test]
    fn test_aabb_zero_is_at_origin() {
        let aabb = Aabb::zero();
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ZERO);
    }

    #[test]
    fn test_aabb_unit_is_0_to_1() {
        let aabb = Aabb::unit();
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_aabb_from_center_half_extents() {
        let aabb = Aabb::from_center_half_extents(Vec3::new(5.0, 5.0, 5.0), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.min, Vec3::new(4.0, 3.0, 2.0));
        assert_eq!(aabb.max, Vec3::new(6.0, 7.0, 8.0));
    }

    #[test]
    fn test_aabb_contains_point_inside() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains(Vec3::new(0.5, 0.5, 0.5)), "Center point should be inside");
    }

    #[test]
    fn test_aabb_contains_point_on_boundary() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains(Vec3::ZERO), "Min corner should be inside (inclusive)");
        assert!(aabb.contains(Vec3::ONE), "Max corner should be inside (inclusive)");
    }

    #[test]
    fn test_aabb_does_not_contain_point_outside() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(!aabb.contains(Vec3::new(1.1, 0.5, 0.5)), "Point outside X should not be contained");
        assert!(!aabb.contains(Vec3::new(0.5, -0.1, 0.5)), "Point outside Y should not be contained");
        assert!(!aabb.contains(Vec3::new(0.5, 0.5, 1.1)), "Point outside Z should not be contained");
    }

    #[test]
    fn test_aabb_intersects_overlapping() {
        let a = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        let b = Aabb::new(Vec3::ONE, Vec3::new(3.0, 3.0, 3.0));
        assert!(a.intersects(&b), "Overlapping AABBs should intersect");
        assert!(b.intersects(&a), "Intersection should be symmetric");
    }

    #[test]
    fn test_aabb_intersects_touching() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::ONE, Vec3::new(2.0, 2.0, 2.0));
        assert!(a.intersects(&b), "Touching AABBs should intersect");
    }

    #[test]
    fn test_aabb_does_not_intersect_separated() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0));
        assert!(!a.intersects(&b), "Separated AABBs should not intersect");
    }

    #[test]
    fn test_aabb_merge_creates_bounding_box() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
        let merged = a.merge(&b);
        assert_eq!(merged.min, Vec3::ZERO);
        assert_eq!(merged.max, Vec3::new(3.0, 3.0, 3.0));
    }

    #[test]
    fn test_aabb_merge_is_commutative() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
        let merged_ab = a.merge(&b);
        let merged_ba = b.merge(&a);
        assert_eq!(merged_ab.min, merged_ba.min);
        assert_eq!(merged_ab.max, merged_ba.max);
    }

    #[test]
    fn test_aabb_from_triangle() {
        let tri = Triangle::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 1.0, 2.0),
            Vec3::new(2.0, 5.0, 1.0),
        );
        let aabb = Aabb::from_triangle(&tri);
        assert_eq!(aabb.min, Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(aabb.max, Vec3::new(4.0, 5.0, 3.0));
    }

    #[test]
    fn test_aabb_center() {
        let aabb = Aabb::new(Vec3::new(2.0, 4.0, 6.0), Vec3::new(4.0, 8.0, 10.0));
        let center = aabb.center();
        assert_eq!(center, Vec3::new(3.0, 6.0, 8.0));
    }
}

// ============================================================================
// Behavioral Correctness Tests - Navigation Invariants
// ============================================================================

mod behavioral_tests {
    use super::*;

    #[test]
    fn test_triangle_area_is_non_negative() {
        // Any triangle should have non-negative area
        for _ in 0..100 {
            let tri = Triangle::new(
                Vec3::new(rand_f32(), rand_f32(), rand_f32()),
                Vec3::new(rand_f32(), rand_f32(), rand_f32()),
                Vec3::new(rand_f32(), rand_f32(), rand_f32()),
            );
            assert!(tri.area() >= 0.0, "Triangle area must be non-negative");
        }
    }

    #[test]
    fn test_triangle_perimeter_greater_than_any_edge() {
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(0.0, 4.0, 0.0),
        );
        let perimeter = tri.perimeter();
        let edges = tri.edge_lengths();
        for edge in edges {
            assert!(perimeter > edge, "Perimeter must be greater than any single edge");
        }
    }

    #[test]
    fn test_triangle_inequality_holds() {
        // Triangle inequality: sum of any two sides > third side
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(3.0, 0.0, 0.0),
            Vec3::new(1.0, 2.0, 0.0),
        );
        let [a, b, c] = tri.edge_lengths();
        assert!(a + b > c, "Triangle inequality: a + b > c");
        assert!(b + c > a, "Triangle inequality: b + c > a");
        assert!(a + c > b, "Triangle inequality: a + c > b");
    }

    #[test]
    fn test_navtri_slope_range() {
        // Slope should be between 0 and 180 degrees
        for i in 0..10 {
            let angle = i as f32 * 18.0; // 0, 18, 36, ... 162
            let rad = angle.to_radians();
            let normal = Vec3::new(rad.sin(), rad.cos(), 0.0).normalize();
            let tri = NavTri::new(0, [Vec3::ZERO; 3], normal, Vec3::ZERO);
            let slope = tri.slope_degrees();
            assert!(slope >= 0.0 && slope <= 180.0, "Slope should be 0-180, got {}", slope);
        }
    }

    #[test]
    fn test_aabb_contains_is_reflexive() {
        // A point on AABB boundary should be contained
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.contains(aabb.min), "AABB should contain its min corner");
        assert!(aabb.contains(aabb.max), "AABB should contain its max corner");
    }

    #[test]
    fn test_aabb_intersects_is_reflexive() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(aabb.intersects(&aabb), "AABB should intersect with itself");
    }

    #[test]
    fn test_aabb_merged_contains_originals() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::new(2.0, 2.0, 2.0), Vec3::new(3.0, 3.0, 3.0));
        let merged = a.merge(&b);
        
        // Merged should contain all corners of original AABBs
        assert!(merged.contains(a.min), "Merged should contain a.min");
        assert!(merged.contains(a.max), "Merged should contain a.max");
        assert!(merged.contains(b.min), "Merged should contain b.min");
        assert!(merged.contains(b.max), "Merged should contain b.max");
    }

    fn rand_f32() -> f32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        ((t.as_nanos() % 1000) as f32 / 1000.0) * 100.0 - 50.0
    }
}

// ============================================================================
// BOUNDARY CONDITION TESTS
// Catches mutations: < vs <=, > vs >=, off-by-one errors
// ============================================================================

mod boundary_condition_tests {
    use super::*;

    // --- Triangle Edge Boundaries ---
    
    #[test]
    fn test_triangle_degenerate_boundary_at_threshold() {
        // Area exactly at threshold 1e-6
        let small = Triangle::new(
            Vec3::ZERO,
            Vec3::new(1e-3, 0.0, 0.0),  // Very small triangle, area ~ 0
            Vec3::new(0.0, 1e-3, 0.0),
        );
        // Area = 0.5 * 1e-3 * 1e-3 = 5e-7 < 1e-6
        assert!(small.is_degenerate(), "Area below threshold should be degenerate");
        
        // Just above threshold
        let bigger = Triangle::new(
            Vec3::ZERO,
            Vec3::new(0.002, 0.0, 0.0),
            Vec3::new(0.0, 0.002, 0.0),
        );
        // Area = 0.5 * 0.002 * 0.002 = 2e-6 > 1e-6
        assert!(!bigger.is_degenerate(), "Area above threshold should not be degenerate");
    }

    #[test]
    fn test_triangle_min_max_edge_when_all_equal() {
        // Equilateral triangle - all edges equal
        let side = 2.0_f32;
        let height = side * (3.0_f32.sqrt() / 2.0);
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(side, 0.0, 0.0),
            Vec3::new(side / 2.0, height, 0.0),
        );
        let min_edge = tri.min_edge_length();
        let max_edge = tri.max_edge_length();
        // All edges should be approximately equal
        assert!((max_edge - min_edge).abs() < 0.01, "Equal edges should give min==max");
    }

    // --- AABB Boundary Tests ---
    
    #[test]
    fn test_aabb_contains_point_at_exact_boundary() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(10.0, 10.0, 10.0));
        
        // Exactly at min boundary
        assert!(aabb.contains(Vec3::ZERO), "Should contain point at min boundary");
        
        // Exactly at max boundary
        assert!(aabb.contains(Vec3::new(10.0, 10.0, 10.0)), "Should contain point at max boundary");
        
        // Just inside (epsilon below max)
        assert!(aabb.contains(Vec3::new(9.999, 9.999, 9.999)), "Should contain point just inside");
        
        // Just outside
        assert!(!aabb.contains(Vec3::new(10.001, 5.0, 5.0)), "Should not contain point just outside");
    }

    #[test]
    fn test_aabb_intersects_at_exact_boundary() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::ONE, Vec3::splat(2.0));
        
        // Touching at corner - should intersect with <= semantics
        assert!(a.intersects(&b), "Touching AABBs should intersect");
        
        // Separated by small gap - should not intersect
        let c = Aabb::new(Vec3::splat(1.001), Vec3::splat(2.0));
        assert!(!a.intersects(&c), "Separated AABBs should not intersect");
    }

    #[test]
    fn test_aabb_is_empty_boundary_conditions() {
        // Zero volume on one axis
        let flat_x = Aabb::new(Vec3::ZERO, Vec3::new(0.0, 1.0, 1.0));
        assert!(flat_x.is_empty(), "Zero X extent should be empty");
        
        let flat_y = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 0.0, 1.0));
        assert!(flat_y.is_empty(), "Zero Y extent should be empty");
        
        let flat_z = Aabb::new(Vec3::ZERO, Vec3::new(1.0, 1.0, 0.0));
        assert!(flat_z.is_empty(), "Zero Z extent should be empty");
        
        // All positive extents
        let valid = Aabb::new(Vec3::ZERO, Vec3::new(0.001, 0.001, 0.001));
        assert!(!valid.is_empty(), "Tiny positive volume should not be empty");
    }

    // --- NavTri Boundary Tests ---
    
    #[test]
    fn test_navtri_neighbor_count_boundary() {
        let mut tri = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        
        // Zero neighbors
        assert_eq!(tri.neighbor_count(), 0, "Empty neighbors should be 0");
        assert!(tri.is_isolated(), "Empty neighbors means isolated");
        
        // Add exactly one neighbor
        tri.neighbors.push(1);
        assert_eq!(tri.neighbor_count(), 1, "One neighbor should be 1");
        assert!(!tri.is_isolated(), "With neighbor, not isolated");
        assert!(tri.is_edge(), "1 neighbor means edge triangle");
        
        // Add to exactly 3 neighbors
        tri.neighbors.push(2);
        tri.neighbors.push(3);
        assert_eq!(tri.neighbor_count(), 3, "Three neighbors should be 3");
        assert!(!tri.is_edge(), "3 neighbors means not edge");
    }

    #[test]
    fn test_navtri_is_edge_boundary_at_three_neighbors() {
        let mut tri = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        
        // 2 neighbors - is edge
        tri.neighbors = vec![1, 2];
        assert!(tri.is_edge(), "2 neighbors: is_edge should be true");
        
        // 3 neighbors - not edge (boundary case)
        tri.neighbors = vec![1, 2, 3];
        assert!(!tri.is_edge(), "3 neighbors: is_edge should be false");
        
        // 4 neighbors - not edge
        tri.neighbors = vec![1, 2, 3, 4];
        assert!(!tri.is_edge(), "4 neighbors: is_edge should be false");
    }

    // --- NavMesh Dirty Region Boundaries ---
    
    #[test]
    fn test_navmesh_dirty_region_count_zero_boundary() {
        let mesh = NavMesh::bake(&[], 0.5, 45.0);
        assert_eq!(mesh.dirty_region_count(), 0, "Fresh mesh has 0 dirty regions");
        assert!(!mesh.needs_rebake(), "Fresh mesh does not need rebake");
    }

    #[test]
    fn test_navmesh_path_crosses_dirty_with_empty_path() {
        let mut mesh = NavMesh::bake(&[], 0.5, 45.0);
        mesh.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        
        // Empty path - should not cross any region
        let empty_path: Vec<Vec3> = vec![];
        assert!(!mesh.path_crosses_dirty_region(&empty_path), "Empty path should not cross dirty region");
    }

    #[test]
    fn test_navtri_slope_degrees_boundary_values() {
        // Slope = 0 (flat)
        let flat = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        assert!((flat.slope_degrees() - 0.0).abs() < 0.01, "Flat should be 0 degrees");
        
        // Slope = 90 (vertical wall)
        let vertical = NavTri::new(0, [Vec3::ZERO; 3], Vec3::X, Vec3::ZERO);
        assert!((vertical.slope_degrees() - 90.0).abs() < 0.01, "Vertical should be 90 degrees");
        
        // Slope = 180 (ceiling)
        let ceiling = NavTri::new(0, [Vec3::ZERO; 3], Vec3::NEG_Y, Vec3::ZERO);
        assert!((ceiling.slope_degrees() - 180.0).abs() < 0.01, "Ceiling should be 180 degrees");
    }
}

// ============================================================================
// COMPARISON OPERATOR TESTS
// Catches mutations: == vs !=, < vs >, wrong enum comparisons
// ============================================================================

mod comparison_operator_tests {
    use super::*;

    // --- Triangle Comparison Tests ---
    
    #[test]
    fn test_triangle_equality_depends_on_all_vertices() {
        let t1 = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let t2 = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        let t3 = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),  // Different vertex c
        );
        
        assert_eq!(t1, t2, "Same vertices should be equal");
        assert_ne!(t1, t3, "Different vertices should not be equal");
    }

    #[test]
    fn test_aabb_equality_depends_on_both_corners() {
        let a1 = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let a2 = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let a3 = Aabb::new(Vec3::ZERO, Vec3::splat(2.0));
        let a4 = Aabb::new(Vec3::splat(-1.0), Vec3::ONE);
        
        assert_eq!(a1, a2, "Same corners should be equal");
        assert_ne!(a1, a3, "Different max should not be equal");
        assert_ne!(a1, a4, "Different min should not be equal");
    }

    // --- NavTri Boolean Method Comparisons ---
    
    #[test]
    fn test_navtri_has_neighbor_exact_match() {
        let mut tri = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        tri.neighbors = vec![1, 5, 10];
        
        // Exact match
        assert!(tri.has_neighbor(1), "Should find neighbor 1");
        assert!(tri.has_neighbor(5), "Should find neighbor 5");
        assert!(tri.has_neighbor(10), "Should find neighbor 10");
        
        // Not in list
        assert!(!tri.has_neighbor(0), "Should not find neighbor 0");
        assert!(!tri.has_neighbor(2), "Should not find neighbor 2");
        assert!(!tri.has_neighbor(100), "Should not find neighbor 100");
    }

    #[test]
    fn test_navtri_is_walkable_threshold_comparison() {
        // dot > 0.0 - walkable
        let up = NavTri::new(0, [Vec3::ZERO; 3], Vec3::new(0.0, 0.001, 0.0).normalize(), Vec3::ZERO);
        assert!(up.is_walkable(), "Slightly upward normal should be walkable");
        
        // dot < 0.0 - not walkable
        let down = NavTri::new(0, [Vec3::ZERO; 3], Vec3::new(0.0, -0.001, 0.0).normalize(), Vec3::ZERO);
        assert!(!down.is_walkable(), "Slightly downward normal should not be walkable");
        
        // dot = 0.0 - vertical (not walkable with > comparison)
        let vertical = NavTri::new(0, [Vec3::ZERO; 3], Vec3::X, Vec3::ZERO);
        assert!(!vertical.is_walkable(), "Vertical wall should not be walkable");
    }

    // --- AABB Axis Comparisons ---
    
    #[test]
    fn test_aabb_longest_vs_shortest_axis_distinct() {
        let aabb = Aabb::new(
            Vec3::ZERO,
            Vec3::new(10.0, 5.0, 2.0),  // X longest, Z shortest
        );
        
        assert!((aabb.longest_axis() - 10.0).abs() < 1e-6, "Longest should be 10");
        assert!((aabb.shortest_axis() - 2.0).abs() < 1e-6, "Shortest should be 2");
        assert!(aabb.longest_axis() > aabb.shortest_axis(), "Longest > Shortest");
    }

    #[test]
    fn test_aabb_contains_vs_intersects_different_semantics() {
        let large = Aabb::new(Vec3::ZERO, Vec3::splat(10.0));
        let small_inside = Aabb::new(Vec3::splat(2.0), Vec3::splat(5.0));
        let touching = Aabb::new(Vec3::splat(10.0), Vec3::splat(12.0));
        
        // Intersects is AABB-to-AABB
        assert!(large.intersects(&small_inside), "Large should intersect small inside");
        assert!(large.intersects(&touching), "Should intersect at boundary");
        
        // Contains is AABB-to-point (different method signature)
        assert!(large.contains(Vec3::splat(5.0)), "Should contain center point");
        assert!(!large.contains(Vec3::splat(11.0)), "Should not contain outside point");
    }

    // --- Triangle min/max edge comparisons ---
    
    #[test]
    fn test_triangle_min_max_edge_distinct_values() {
        let tri = Triangle::new(
            Vec3::ZERO,
            Vec3::new(10.0, 0.0, 0.0),  // Edge length 10
            Vec3::new(3.0, 1.0, 0.0),   // Short edges ~3.16 and ~7.07
        );
        
        let min_e = tri.min_edge_length();
        let max_e = tri.max_edge_length();
        
        assert!(max_e > min_e, "Max edge should be greater than min edge");
        assert!((max_e - 10.0).abs() < 0.01, "Max edge should be 10");
    }

    // --- NavMesh dirty region comparisons ---
    
    #[test]
    fn test_navmesh_needs_rebake_vs_dirty_count() {
        let mut mesh = NavMesh::bake(&[], 0.5, 45.0);
        
        // Initially neither
        assert!(!mesh.needs_rebake(), "Fresh mesh needs no rebake");
        assert_eq!(mesh.dirty_region_count(), 0, "Fresh mesh has no dirty regions");
        
        // After invalidation, both should indicate dirty
        mesh.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        assert!(mesh.needs_rebake(), "Should need rebake after invalidation");
        assert!(mesh.dirty_region_count() > 0, "Should have dirty regions after invalidation");
    }
}

// ============================================================================
// BOOLEAN RETURN PATH TESTS
// Catches mutations: return true vs false, logic inversions, early returns
// ============================================================================

mod boolean_return_path_tests {
    use super::*;

    // --- Triangle Boolean Methods ---
    
    #[test]
    fn test_triangle_is_degenerate_returns_correct_boolean() {
        // Degenerate case - collinear points
        let degenerate = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),  // All on X axis
        );
        assert_eq!(degenerate.is_degenerate(), true, "Collinear should return true");
        
        // Non-degenerate case
        let valid = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        );
        assert_eq!(valid.is_degenerate(), false, "Valid triangle should return false");
    }

    // --- AABB Boolean Methods ---
    
    #[test]
    fn test_aabb_is_empty_returns_correct_boolean() {
        // Empty case
        let empty = Aabb::new(Vec3::ZERO, Vec3::ZERO);
        assert_eq!(empty.is_empty(), true, "Zero volume should return true");
        
        // Non-empty case
        let valid = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(valid.is_empty(), false, "Positive volume should return false");
    }

    #[test]
    fn test_aabb_contains_returns_correct_boolean() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        
        // Inside - true
        assert_eq!(aabb.contains(Vec3::splat(0.5)), true, "Inside point should return true");
        
        // Outside - false
        assert_eq!(aabb.contains(Vec3::splat(2.0)), false, "Outside point should return false");
    }

    #[test]
    fn test_aabb_intersects_returns_correct_boolean() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let b = Aabb::new(Vec3::splat(0.5), Vec3::splat(1.5));
        let c = Aabb::new(Vec3::splat(5.0), Vec3::splat(6.0));
        
        // Overlapping - true
        assert_eq!(a.intersects(&b), true, "Overlapping should return true");
        
        // Separated - false
        assert_eq!(a.intersects(&c), false, "Separated should return false");
    }

    // --- NavTri Boolean Methods ---
    
    #[test]
    fn test_navtri_is_isolated_returns_correct_boolean() {
        let mut tri = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        
        // Isolated (no neighbors)
        assert_eq!(tri.is_isolated(), true, "No neighbors should return true");
        
        // Not isolated
        tri.neighbors.push(1);
        assert_eq!(tri.is_isolated(), false, "With neighbors should return false");
    }

    #[test]
    fn test_navtri_is_edge_returns_correct_boolean() {
        let mut tri = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        
        // Edge case (less than 3 neighbors)
        tri.neighbors = vec![1, 2];
        assert_eq!(tri.is_edge(), true, "< 3 neighbors should return true");
        
        // Not edge (3 or more neighbors)
        tri.neighbors = vec![1, 2, 3];
        assert_eq!(tri.is_edge(), false, ">= 3 neighbors should return false");
    }

    #[test]
    fn test_navtri_has_neighbor_returns_correct_boolean() {
        let mut tri = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        tri.neighbors = vec![5, 10, 15];
        
        // Has neighbor - true
        assert_eq!(tri.has_neighbor(5), true, "Present neighbor should return true");
        assert_eq!(tri.has_neighbor(10), true, "Present neighbor should return true");
        
        // Doesn't have neighbor - false
        assert_eq!(tri.has_neighbor(0), false, "Absent neighbor should return false");
        assert_eq!(tri.has_neighbor(99), false, "Absent neighbor should return false");
    }

    #[test]
    fn test_navtri_is_walkable_returns_correct_boolean() {
        // Walkable (upward facing)
        let walkable = NavTri::new(0, [Vec3::ZERO; 3], Vec3::Y, Vec3::ZERO);
        assert_eq!(walkable.is_walkable(), true, "Upward normal should return true");
        
        // Not walkable (downward facing)
        let not_walkable = NavTri::new(0, [Vec3::ZERO; 3], Vec3::NEG_Y, Vec3::ZERO);
        assert_eq!(not_walkable.is_walkable(), false, "Downward normal should return false");
    }

    // --- NavMesh Boolean Methods ---
    
    #[test]
    fn test_navmesh_needs_rebake_returns_correct_boolean() {
        let mut mesh = NavMesh::bake(&[], 0.5, 45.0);
        
        // Clean - false
        assert_eq!(mesh.needs_rebake(), false, "Clean mesh should return false");
        
        // Dirty - true
        mesh.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        assert_eq!(mesh.needs_rebake(), true, "Dirty mesh should return true");
    }

    #[test]
    fn test_navmesh_path_crosses_dirty_region_returns_correct_boolean() {
        let mut mesh = NavMesh::bake(&[], 0.5, 45.0);
        mesh.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        
        // Path crosses dirty - true
        let crossing_path = vec![Vec3::splat(0.5)];
        assert_eq!(
            mesh.path_crosses_dirty_region(&crossing_path),
            true,
            "Path inside dirty region should return true"
        );
        
        // Path avoids dirty - false
        let avoiding_path = vec![Vec3::splat(5.0)];
        assert_eq!(
            mesh.path_crosses_dirty_region(&avoiding_path),
            false,
            "Path outside dirty region should return false"
        );
    }

    // --- Early Return Path Tests ---
    
    #[test]
    fn test_navmesh_path_crosses_empty_regions_early_return() {
        let mesh = NavMesh::bake(&[], 0.5, 45.0);
        
        // No dirty regions - should return false early
        let path = vec![Vec3::ZERO, Vec3::ONE];
        assert!(!mesh.path_crosses_dirty_region(&path), "No dirty regions should return false");
    }

    #[test]
    fn test_navmesh_rebake_empty_does_nothing() {
        let mut mesh = NavMesh::bake(&[], 0.5, 45.0);
        let initial_count = mesh.rebake_count();
        
        // Rebake with no dirty regions should not increment counter
        mesh.rebake_dirty_regions(&[]);
        assert_eq!(mesh.rebake_count(), initial_count, "No dirty regions should not trigger rebake");
    }
}

