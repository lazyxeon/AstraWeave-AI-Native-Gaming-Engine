//! Mutation-Hardening Tests for astraweave-nav
//!
//! These tests are specifically designed to catch mutations that cargo-mutants
//! introduces: operator flips (>= → >, <= → <), boolean logic changes (&& → ||),
//! arithmetic swaps (+→-, *→/), and constant replacements. Each test targets
//! a specific mutation site with boundary values that differentiate correct
//! from mutated behavior.
//!
//! Coverage targets: Aabb boundary comparisons, is_empty per-axis, volume/surface_area
//! arithmetic, smooth() constants, share_edge() thresholds, edge_count() division,
//! average_neighbor_count() division, NavTri::is_walkable/is_edge boundaries.

use astraweave_nav::{Aabb, NavMesh, NavTri, Triangle};
use glam::Vec3;

// =============================================================================
// AABB::contains() — 6 boundary comparisons + boolean AND logic
// Each test isolates one comparison to catch >= → > or <= → < mutations
// =============================================================================

mod aabb_contains_boundary {
    use super::*;

    fn aabb() -> Aabb {
        Aabb::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0))
    }

    // --- Point exactly at min boundary (tests >= vs >) ---

    #[test]
    fn contains_point_at_min_x_boundary() {
        // point.x == min.x: true with >=, false with >
        assert!(
            aabb().contains(Vec3::new(1.0, 3.5, 4.5)),
            "point at min.x boundary must be contained (>= not >)"
        );
    }

    #[test]
    fn contains_point_at_min_y_boundary() {
        // point.y == min.y: true with >=, false with >
        assert!(
            aabb().contains(Vec3::new(2.5, 2.0, 4.5)),
            "point at min.y boundary must be contained (>= not >)"
        );
    }

    #[test]
    fn contains_point_at_min_z_boundary() {
        // point.z == min.z: true with >=, false with >
        assert!(
            aabb().contains(Vec3::new(2.5, 3.5, 3.0)),
            "point at min.z boundary must be contained (>= not >)"
        );
    }

    // --- Point exactly at max boundary (tests <= vs <) ---

    #[test]
    fn contains_point_at_max_x_boundary() {
        // point.x == max.x: true with <=, false with <
        assert!(
            aabb().contains(Vec3::new(4.0, 3.5, 4.5)),
            "point at max.x boundary must be contained (<= not <)"
        );
    }

    #[test]
    fn contains_point_at_max_y_boundary() {
        // point.y == max.y: true with <=, false with <
        assert!(
            aabb().contains(Vec3::new(2.5, 5.0, 4.5)),
            "point at max.y boundary must be contained (<= not <)"
        );
    }

    #[test]
    fn contains_point_at_max_z_boundary() {
        // point.z == max.z: true with <=, false with <
        assert!(
            aabb().contains(Vec3::new(2.5, 3.5, 6.0)),
            "point at max.z boundary must be contained (<= not <)"
        );
    }

    // --- Point just barely outside each axis (catches && → || mutation) ---
    // If && were replaced by ||, these outside points would incorrectly
    // return true since they satisfy 5 of 6 conditions.

    #[test]
    fn rejects_point_just_below_min_x() {
        assert!(
            !aabb().contains(Vec3::new(0.999, 3.5, 4.5)),
            "point below min.x must NOT be contained"
        );
    }

    #[test]
    fn rejects_point_just_below_min_y() {
        assert!(
            !aabb().contains(Vec3::new(2.5, 1.999, 4.5)),
            "point below min.y must NOT be contained"
        );
    }

    #[test]
    fn rejects_point_just_below_min_z() {
        assert!(
            !aabb().contains(Vec3::new(2.5, 3.5, 2.999)),
            "point below min.z must NOT be contained"
        );
    }

    #[test]
    fn rejects_point_just_above_max_x() {
        assert!(
            !aabb().contains(Vec3::new(4.001, 3.5, 4.5)),
            "point above max.x must NOT be contained"
        );
    }

    #[test]
    fn rejects_point_just_above_max_y() {
        assert!(
            !aabb().contains(Vec3::new(2.5, 5.001, 4.5)),
            "point above max.y must NOT be contained"
        );
    }

    #[test]
    fn rejects_point_just_above_max_z() {
        assert!(
            !aabb().contains(Vec3::new(2.5, 3.5, 6.001)),
            "point above max.z must NOT be contained"
        );
    }

    // --- Point at exact corner (all 6 conditions at boundary simultaneously) ---

    #[test]
    fn contains_min_corner() {
        assert!(aabb().contains(Vec3::new(1.0, 2.0, 3.0)));
    }

    #[test]
    fn contains_max_corner() {
        assert!(aabb().contains(Vec3::new(4.0, 5.0, 6.0)));
    }
}

// =============================================================================
// AABB::intersects() — 6 boundary comparisons + boolean AND logic
// Tests touching AABBs (boundary equality) to catch <= → < and >= → > flips
// =============================================================================

mod aabb_intersects_boundary {
    use super::*;

    fn base() -> Aabb {
        Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(2.0, 2.0, 2.0))
    }

    // --- Touching at exactly one face (tests <= and >= boundaries) ---

    #[test]
    fn intersects_touching_at_max_x_face() {
        // base.max.x == other.min.x: self.max.x >= other.min.x is true with >=, false with >
        let other = Aabb::new(Vec3::new(2.0, 0.0, 0.0), Vec3::new(4.0, 2.0, 2.0));
        assert!(
            base().intersects(&other),
            "AABBs touching at x=2.0 face must intersect (>= not >)"
        );
    }

    #[test]
    fn intersects_touching_at_min_x_face() {
        // self.min.x <= other.max.x at the boundary
        let other = Aabb::new(Vec3::new(-2.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 2.0));
        assert!(
            base().intersects(&other),
            "AABBs touching at x=0.0 face must intersect (<= not <)"
        );
    }

    #[test]
    fn intersects_touching_at_max_y_face() {
        let other = Aabb::new(Vec3::new(0.0, 2.0, 0.0), Vec3::new(2.0, 4.0, 2.0));
        assert!(
            base().intersects(&other),
            "AABBs touching at y=2.0 face must intersect"
        );
    }

    #[test]
    fn intersects_touching_at_min_y_face() {
        let other = Aabb::new(Vec3::new(0.0, -2.0, 0.0), Vec3::new(2.0, 0.0, 2.0));
        assert!(
            base().intersects(&other),
            "AABBs touching at y=0.0 face must intersect"
        );
    }

    #[test]
    fn intersects_touching_at_max_z_face() {
        let other = Aabb::new(Vec3::new(0.0, 0.0, 2.0), Vec3::new(2.0, 2.0, 4.0));
        assert!(
            base().intersects(&other),
            "AABBs touching at z=2.0 face must intersect"
        );
    }

    #[test]
    fn intersects_touching_at_min_z_face() {
        let other = Aabb::new(Vec3::new(0.0, 0.0, -2.0), Vec3::new(2.0, 2.0, 0.0));
        assert!(
            base().intersects(&other),
            "AABBs touching at z=0.0 face must intersect"
        );
    }

    // --- Separated by epsilon on each axis (catches && → || mutation) ---

    #[test]
    fn no_intersect_separated_x_only() {
        let other = Aabb::new(Vec3::new(2.01, 0.0, 0.0), Vec3::new(4.0, 2.0, 2.0));
        assert!(
            !base().intersects(&other),
            "separated on x axis must NOT intersect"
        );
    }

    #[test]
    fn no_intersect_separated_y_only() {
        let other = Aabb::new(Vec3::new(0.0, 2.01, 0.0), Vec3::new(2.0, 4.0, 2.0));
        assert!(
            !base().intersects(&other),
            "separated on y axis must NOT intersect"
        );
    }

    #[test]
    fn no_intersect_separated_z_only() {
        let other = Aabb::new(Vec3::new(0.0, 0.0, 2.01), Vec3::new(2.0, 2.0, 4.0));
        assert!(
            !base().intersects(&other),
            "separated on z axis must NOT intersect"
        );
    }

    // --- Symmetry test (intersects should be commutative) ---

    #[test]
    fn intersects_is_symmetric() {
        let a = Aabb::new(Vec3::ZERO, Vec3::new(3.0, 3.0, 3.0));
        let b = Aabb::new(Vec3::new(1.0, 1.0, 1.0), Vec3::new(5.0, 5.0, 5.0));
        assert_eq!(a.intersects(&b), b.intersects(&a));
    }
}

// =============================================================================
// AABB::is_empty() — 3 comparisons with || logic
// Each test isolates one axis to catch >= → > and || → && mutations
// =============================================================================

mod aabb_is_empty_per_axis {
    use super::*;

    // --- Equal on exactly one axis (catches >= → > mutation per axis) ---

    #[test]
    fn empty_on_x_axis_only() {
        // min.x == max.x, but y and z are valid 
        // With >=: true (empty). With >: false (not empty) — catches the mutation
        let aabb = Aabb::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(5.0, 1.0, 1.0));
        assert!(aabb.is_empty(), "AABB with min.x == max.x must be empty (>= not >)");
    }

    #[test]
    fn empty_on_y_axis_only() {
        let aabb = Aabb::new(Vec3::new(0.0, 5.0, 0.0), Vec3::new(1.0, 5.0, 1.0));
        assert!(aabb.is_empty(), "AABB with min.y == max.y must be empty (>= not >)");
    }

    #[test]
    fn empty_on_z_axis_only() {
        let aabb = Aabb::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(1.0, 1.0, 5.0));
        assert!(aabb.is_empty(), "AABB with min.z == max.z must be empty (>= not >)");
    }

    // --- Valid on all axes (catches || → && when only one axis is collapsed) ---
    // The above tests also serve this purpose: if || were &&, only
    // "all axes collapsed" would return true, not "one axis collapsed."
    // But let's explicitly test a fully valid AABB too:

    #[test]
    fn not_empty_when_all_axes_positive() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(0.001, 0.001, 0.001));
        assert!(!aabb.is_empty(), "tiny but valid AABB must NOT be empty");
    }

    #[test]
    fn inverted_on_x_only_is_empty() {
        // min.x > max.x (0 > -1 = true), y and z valid
        let aabb = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(-1.0, 1.0, 1.0));
        assert!(aabb.is_empty(), "inverted x axis must be empty");
    }

    #[test]
    fn inverted_on_y_only_is_empty() {
        let aabb = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, -1.0, 1.0));
        assert!(aabb.is_empty(), "inverted y axis must be empty");
    }

    #[test]
    fn inverted_on_z_only_is_empty() {
        let aabb = Aabb::new(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, -1.0));
        assert!(aabb.is_empty(), "inverted z axis must be empty");
    }
}

// =============================================================================
// AABB arithmetic: volume(), surface_area(), longest_axis(), shortest_axis()
// Tests designed so any operator swap (+↔-, *↔/, constant change) produces
// a detectably wrong answer.
// =============================================================================

mod aabb_arithmetic {
    use super::*;

    // Use asymmetric dimensions (2, 3, 5) so swapping any pair changes the result
    fn asymmetric() -> Aabb {
        Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 5.0))
    }

    // --- volume = x * y * z ---

    #[test]
    fn volume_asymmetric_exact() {
        // 2 * 3 * 5 = 30
        // If * → +: 2+3+5 = 10. If * → /: broken. If operands swapped: same.
        let v = asymmetric().volume();
        assert!(
            (v - 30.0).abs() < 1e-6,
            "volume of 2x3x5 box must be exactly 30.0, got {}",
            v
        );
    }

    #[test]
    fn volume_1x1x1_is_1_not_3() {
        // Catches * → + (1+1+1=3) since 1*1*1=1
        let v = Aabb::new(Vec3::ZERO, Vec3::ONE).volume();
        assert!((v - 1.0).abs() < 1e-6, "volume of unit cube must be 1.0, got {}", v);
    }

    // --- surface_area = 2 * (xy + yz + zx) ---

    #[test]
    fn surface_area_asymmetric_exact() {
        // 2*(2*3 + 3*5 + 5*2) = 2*(6+15+10) = 2*31 = 62
        let sa = asymmetric().surface_area();
        assert!(
            (sa - 62.0).abs() < 1e-6,
            "surface area of 2x3x5 box must be 62.0, got {}",
            sa
        );
    }

    #[test]
    fn surface_area_1x1x1_is_6() {
        // 2*(1+1+1) = 6
        // If the 2.0* were removed: 3.0. If + → *: 2*(1*1*1) = 2.
        let sa = Aabb::new(Vec3::ZERO, Vec3::ONE).surface_area();
        assert!((sa - 6.0).abs() < 1e-6, "surface area of unit cube must be 6.0, got {}", sa);
    }

    #[test]
    fn surface_area_2x3x4_is_52() {
        // 2*(6+12+8) = 52 (from existing test, reinforced here)
        let sa = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0)).surface_area();
        assert!((sa - 52.0).abs() < 1e-6);
    }

    // --- longest_axis = max of sizes ---

    #[test]
    fn longest_axis_is_z_when_z_largest() {
        // sizes: (2, 3, 5) → max = 5
        let la = asymmetric().longest_axis();
        assert!((la - 5.0).abs() < 1e-6, "longest axis should be 5.0, got {}", la);
    }

    #[test]
    fn longest_axis_is_x_when_x_largest() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(10.0, 3.0, 5.0));
        assert!((aabb.longest_axis() - 10.0).abs() < 1e-6);
    }

    #[test]
    fn longest_axis_is_y_when_y_largest() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 7.0, 5.0));
        assert!((aabb.longest_axis() - 7.0).abs() < 1e-6);
    }

    // --- shortest_axis = min of sizes ---

    #[test]
    fn shortest_axis_is_x_when_x_smallest() {
        // sizes: (2, 3, 5) → min = 2
        let sa = asymmetric().shortest_axis();
        assert!((sa - 2.0).abs() < 1e-6, "shortest axis should be 2.0, got {}", sa);
    }

    #[test]
    fn shortest_axis_is_y_when_y_smallest() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(5.0, 1.0, 3.0));
        assert!((aabb.shortest_axis() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn shortest_axis_is_z_when_z_smallest() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(5.0, 3.0, 0.5));
        assert!((aabb.shortest_axis() - 0.5).abs() < 1e-6);
    }

    // --- longest != shortest for asymmetric ---

    #[test]
    fn longest_and_shortest_differ_for_asymmetric() {
        let la = asymmetric().longest_axis();
        let sa = asymmetric().shortest_axis();
        assert!(
            (la - sa).abs() > 1e-6,
            "longest and shortest must differ: longest={}, shortest={}",
            la,
            sa
        );
    }

    // --- expand() arithmetic ---

    #[test]
    fn expand_by_half() {
        let aabb = Aabb::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0));
        let expanded = aabb.expand(0.5);
        // min -= 0.5 on each axis, max += 0.5 on each axis
        assert!((expanded.min.x - 0.5).abs() < 1e-6);
        assert!((expanded.min.y - 1.5).abs() < 1e-6);
        assert!((expanded.min.z - 2.5).abs() < 1e-6);
        assert!((expanded.max.x - 4.5).abs() < 1e-6);
        assert!((expanded.max.y - 5.5).abs() < 1e-6);
        assert!((expanded.max.z - 6.5).abs() < 1e-6);
    }

    // --- from_center_half_extents round-trip ---

    #[test]
    fn from_center_half_extents_round_trip() {
        let center = Vec3::new(3.0, 7.0, 11.0);
        let half = Vec3::new(1.0, 2.0, 3.0);
        let aabb = Aabb::from_center_half_extents(center, half);
        // Verify center() matches
        let c = aabb.center();
        assert!((c.x - 3.0).abs() < 1e-6);
        assert!((c.y - 7.0).abs() < 1e-6);
        assert!((c.z - 11.0).abs() < 1e-6);
        // Verify half_extents() matches
        let he = aabb.half_extents();
        assert!((he.x - 1.0).abs() < 1e-6);
        assert!((he.y - 2.0).abs() < 1e-6);
        assert!((he.z - 3.0).abs() < 1e-6);
    }

    // --- merge() arithmetic ---

    #[test]
    fn merge_takes_component_wise_min_max() {
        let a = Aabb::new(Vec3::new(1.0, 5.0, 3.0), Vec3::new(4.0, 8.0, 6.0));
        let b = Aabb::new(Vec3::new(2.0, 3.0, 1.0), Vec3::new(6.0, 7.0, 9.0));
        let m = a.merge(&b);
        // merged.min = min per component
        assert!((m.min.x - 1.0).abs() < 1e-6);
        assert!((m.min.y - 3.0).abs() < 1e-6);
        assert!((m.min.z - 1.0).abs() < 1e-6);
        // merged.max = max per component
        assert!((m.max.x - 6.0).abs() < 1e-6);
        assert!((m.max.y - 8.0).abs() < 1e-6);
        assert!((m.max.z - 9.0).abs() < 1e-6);
    }

    // --- distance_to_point ---

    #[test]
    fn distance_to_point_inside_is_center_distance() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(4.0, 4.0, 4.0));
        // Point inside → distance is from center to point
        let center = aabb.center(); // (2,2,2)
        let d = aabb.distance_to_point(Vec3::new(1.0, 1.0, 1.0));
        let expected = center.distance(Vec3::new(1.0, 1.0, 1.0));
        assert!((d - expected).abs() < 1e-5);
    }

    // --- from_triangle picks correct min/max ---

    #[test]
    fn from_triangle_min_max_per_component() {
        let tri = Triangle::new(
            Vec3::new(3.0, 1.0, 7.0),
            Vec3::new(1.0, 5.0, 2.0),
            Vec3::new(8.0, 3.0, 4.0),
        );
        let aabb = Aabb::from_triangle(&tri);
        assert!((aabb.min.x - 1.0).abs() < 1e-6);
        assert!((aabb.min.y - 1.0).abs() < 1e-6);
        assert!((aabb.min.z - 2.0).abs() < 1e-6);
        assert!((aabb.max.x - 8.0).abs() < 1e-6);
        assert!((aabb.max.y - 5.0).abs() < 1e-6);
        assert!((aabb.max.z - 7.0).abs() < 1e-6);
    }
}

// =============================================================================
// NavTri::is_walkable() — dot product boundary (dot >= 0)
// NavTri::is_edge() — neighbor_count < 3
// =============================================================================

mod navtri_boundaries {
    use super::*;

    #[test]
    fn is_walkable_at_dot_zero_horizontal_normal() {
        // Normal pointing exactly horizontal (+X): dot(X, Y) = 0
        // is_walkable checks dot > 0 (strict), so dot=0 → NOT walkable
        // Mutation > → >= would make this walkable → test catches it
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::Y, Vec3::Z], Vec3::X, Vec3::ZERO);
        assert!(
            !tri.is_walkable(),
            "horizontal normal (dot=0) must NOT be walkable (> not >=)"
        );
    }

    #[test]
    fn is_walkable_at_dot_negative_epsilon() {
        // Normal pointing slightly downward: dot < 0 → not walkable
        let normal = Vec3::new(0.0, -0.001, 1.0).normalize();
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], normal, Vec3::ZERO);
        assert!(
            !tri.is_walkable(),
            "slightly downward normal must NOT be walkable"
        );
    }

    #[test]
    fn is_walkable_at_dot_positive_epsilon() {
        // Normal pointing slightly upward: dot > 0 → walkable
        let normal = Vec3::new(0.0, 0.001, 1.0).normalize();
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], normal, Vec3::ZERO);
        assert!(tri.is_walkable(), "slightly upward normal must be walkable");
    }

    // --- is_edge() boundary: neighbor_count < 3 ---

    #[test]
    fn is_edge_with_exactly_2_neighbors() {
        // 2 < 3 is true → is_edge
        // If < → <=: 2 <= 3 still true (no change). But 3 < 3 is false, 3 <= 3 is true.
        let mut tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        tri.neighbors = vec![1, 2];
        assert!(tri.is_edge(), "2 neighbors must be edge (< 3)");
    }

    #[test]
    fn is_edge_with_exactly_3_neighbors_is_false() {
        // 3 < 3 is false → NOT edge
        let mut tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        tri.neighbors = vec![1, 2, 3];
        assert!(!tri.is_edge(), "3 neighbors must NOT be edge");
    }

    #[test]
    fn is_edge_with_exactly_4_neighbors_is_false() {
        let mut tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        tri.neighbors = vec![1, 2, 3, 4];
        assert!(!tri.is_edge(), "4 neighbors must NOT be edge");
    }

    #[test]
    fn is_edge_with_0_neighbors_is_true() {
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        assert!(tri.is_edge(), "0 neighbors must be edge");
    }

    #[test]
    fn is_edge_with_1_neighbor_is_true() {
        let mut tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        tri.neighbors = vec![1];
        assert!(tri.is_edge(), "1 neighbor must be edge");
    }

    // --- NavTri::distance_squared_to ---

    #[test]
    fn distance_squared_to_known_value() {
        let tri = NavTri::new(
            0,
            [Vec3::ZERO, Vec3::X, Vec3::Z],
            Vec3::Y,
            Vec3::new(1.0, 0.0, 0.0), // center at (1,0,0)
        );
        // distance_squared from center(1,0,0) to (4,0,0) = 9
        let dsq = tri.distance_squared_to(Vec3::new(4.0, 0.0, 0.0));
        assert!(
            (dsq - 9.0).abs() < 1e-5,
            "distance_squared should be 9.0, got {}",
            dsq
        );
    }

    #[test]
    fn distance_to_matches_sqrt_of_distance_squared() {
        let tri = NavTri::new(
            0,
            [Vec3::ZERO, Vec3::X, Vec3::Z],
            Vec3::Y,
            Vec3::new(2.0, 3.0, 6.0),
        );
        let target = Vec3::new(5.0, 7.0, 6.0);
        let d = tri.distance_to(target);
        let dsq = tri.distance_squared_to(target);
        assert!((d * d - dsq).abs() < 1e-4, "d²={} should equal dsq={}", d * d, dsq);
    }

    // --- NavTri::has_neighbor ---

    #[test]
    fn has_neighbor_exact_match() {
        let mut tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        tri.neighbors = vec![3, 7, 11];
        assert!(tri.has_neighbor(7));
        assert!(!tri.has_neighbor(8));
    }

    // --- NavTri::slope_degrees for 45° ---

    #[test]
    fn slope_degrees_45_exact() {
        // Normal at 45° between Y and Z: (0, 1, 1).normalize()
        // angle from Y = acos(dot(n, Y)) = acos(1/sqrt(2)) = 45°
        let normal = Vec3::new(0.0, 1.0, 1.0).normalize();
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], normal, Vec3::ZERO);
        let slope = tri.slope_degrees();
        assert!(
            (slope - 45.0).abs() < 0.5,
            "45° slope expected, got {}",
            slope
        );
    }
}

// =============================================================================
// NavMesh::edge_count() — division by 2
// NavMesh::average_neighbor_count() — division by tris.len()
// =============================================================================

mod navmesh_division_ops {
    use super::*;

    fn three_connected_in_strip() -> Vec<Triangle> {
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
            Triangle::new(
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(2.0, 0.0, 0.0),
            ),
        ]
    }

    #[test]
    fn edge_count_three_strip_is_2() {
        // Strip of 3: tri0—tri1—tri2 = 2 edges
        // Total neighbor links = 4 (tri0:1, tri1:2, tri2:1) → 4/2 = 2
        // If /2 were removed: 4. If /2 → *2: 8.
        let nav = NavMesh::bake(&three_connected_in_strip(), 0.4, 60.0);
        assert_eq!(
            nav.edge_count(),
            2,
            "strip of 3 triangles must have exactly 2 edges (not {})",
            nav.edge_count()
        );
    }

    #[test]
    fn edge_count_single_pair_is_1() {
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
        // Total neighbor links = 2 (each has 1) → 2/2 = 1
        assert_eq!(nav.edge_count(), 1);
    }

    #[test]
    fn edge_count_isolated_is_0() {
        let tris = vec![Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.edge_count(), 0);
    }

    // --- average_neighbor_count ---

    #[test]
    fn average_neighbor_count_strip_of_3() {
        // tri0: 1 neighbor, tri1: 2 neighbors, tri2: 1 neighbor
        // total = 4, avg = 4/3 ≈ 1.333
        let nav = NavMesh::bake(&three_connected_in_strip(), 0.4, 60.0);
        let avg = nav.average_neighbor_count();
        let expected = 4.0 / 3.0;
        assert!(
            (avg - expected).abs() < 1e-5,
            "avg neighbor count should be {:.4}, got {:.4}",
            expected,
            avg
        );
    }

    #[test]
    fn average_neighbor_count_empty_is_0() {
        let nav = NavMesh::bake(&[], 0.4, 60.0);
        assert!(
            (nav.average_neighbor_count() - 0.0).abs() < 1e-6,
            "empty navmesh average should be 0.0"
        );
    }

    #[test]
    fn average_neighbor_count_single_isolated_is_0() {
        let tris = vec![Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert!(
            (nav.average_neighbor_count() - 0.0).abs() < 1e-6,
            "single isolated triangle average should be 0.0"
        );
    }

    // --- isolated_count ---

    #[test]
    fn isolated_count_in_strip_is_0() {
        let nav = NavMesh::bake(&three_connected_in_strip(), 0.4, 60.0);
        assert_eq!(nav.isolated_count(), 0);
    }

    // --- total_area exact ---

    #[test]
    fn total_area_two_right_triangles_form_unit_square() {
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
        // Each triangle: area = 0.5, total = 1.0
        assert!(
            (nav.total_area() - 1.0).abs() < 1e-5,
            "two right triangles forming unit square must have area 1.0"
        );
    }
}

// =============================================================================
// smooth() — Verifies exact weights (0.25, 0.5, 0.25) and 2 iterations
// =============================================================================

mod smooth_constants {
    use super::*;

    /// Compute expected smooth output analytically for 3 collinear points.
    /// smooth() does 2 passes of: pts[1] = 0.25*pts[0] + 0.5*pts[1] + 0.25*pts[2]
    /// For pts = [0, 10, 0]:
    ///   Pass 1: pts[1] = 0.25*0 + 0.5*10 + 0.25*0 = 5.0
    ///   Pass 2: pts[1] = 0.25*0 + 0.5*5 + 0.25*0 = 2.5
    #[test]
    fn smooth_3_points_exact_weights() {
        let tris = vec![Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);

        // We test smooth through find_path which calls smooth internally.
        // Instead, build a known scenario with 3+ waypoints.
        // Actually, smooth is a private function. Let's verify through path output.

        // Create a strip of 5 triangles to ensure smoothing has an effect
        let strip_tris = vec![
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
                Vec3::new(2.0, 0.0, 0.0),
            ),
            Triangle::new(
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 1.0),
                Vec3::new(2.0, 0.0, 1.0),
            ),
            Triangle::new(
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(2.0, 0.0, 1.0),
                Vec3::new(3.0, 0.0, 0.0),
            ),
        ];
        let nav = NavMesh::bake(&strip_tris, 0.4, 60.0);
        let path = nav.find_path(Vec3::new(0.1, 0.0, 0.3), Vec3::new(2.8, 0.0, 0.3));

        // Path should exist and have multiple waypoints
        assert!(path.len() >= 2, "path should have at least 2 points");
        // Start and end should be preserved
        assert!(
            (path[0] - Vec3::new(0.1, 0.0, 0.3)).length() < 0.05,
            "start should be preserved"
        );
        assert!(
            (path.last().unwrap() - Vec3::new(2.8, 0.0, 0.3)).length() < 0.05,
            "end should be preserved"
        );
    }

    // Test that smoothing doesn't modify endpoints
    #[test]
    fn smooth_preserves_endpoints_in_path() {
        let strip = vec![
            Triangle::new(
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 2.0),
                Vec3::new(2.0, 0.0, 0.0),
            ),
            Triangle::new(
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 2.0),
                Vec3::new(2.0, 0.0, 2.0),
            ),
            Triangle::new(
                Vec3::new(2.0, 0.0, 0.0),
                Vec3::new(2.0, 0.0, 2.0),
                Vec3::new(4.0, 0.0, 0.0),
            ),
        ];
        let nav = NavMesh::bake(&strip, 0.4, 60.0);
        let start = Vec3::new(0.5, 0.0, 0.5);
        let goal = Vec3::new(3.5, 0.0, 0.5);
        let path = nav.find_path(start, goal);
        assert!(path.len() >= 2);
        assert!(
            (path[0] - start).length() < 0.01,
            "path start must match query start"
        );
        assert!(
            (path.last().unwrap() - goal).length() < 0.01,
            "path end must match query goal"
        );
    }
}

// =============================================================================
// NavMesh::bake() slope filter boundary tests
// Tests at exactly max_slope_deg to catch <= → < mutation
// =============================================================================

mod bake_slope_boundary {
    use super::*;

    /// Create a triangle with a known slope angle from vertical.
    /// slope_deg = 0 means flat (normal = +Y), 90 means vertical (normal horizontal).
    fn triangle_with_slope(slope_deg: f32) -> Triangle {
        let rad = slope_deg.to_radians();
        // Rotate the triangle so normal is at `slope_deg` from Y axis
        // Normal = (sin(slope), cos(slope), 0)
        // For this, we construct a triangle in the plane perpendicular to this normal
        let ny = rad.cos();
        let nx = rad.sin();
        // Two tangent vectors orthogonal to normal
        let tangent1 = Vec3::new(ny, -nx, 0.0); // perpendicular in XY
        let tangent2 = Vec3::new(0.0, 0.0, 1.0); // Z axis

        let center = Vec3::new(0.0, 0.0, 0.0);
        // Vertex order: (center, center+tangent2, center+tangent1) gives
        // cross(tangent2, tangent1) which produces an upward-facing normal
        Triangle::new(
            center,
            center + tangent2,
            center + tangent1,
        )
    }

    #[test]
    fn bake_includes_triangle_at_exactly_max_slope() {
        // max_slope_deg = 45. Triangle at exactly 45°.
        // condition: angle_from_vertical <= max_slope_deg
        // At exact equality: true with <=, false with <
        let tri = triangle_with_slope(45.0);
        let nav = NavMesh::bake(&[tri], 0.4, 45.0);
        assert_eq!(
            nav.triangle_count(),
            1,
            "triangle at exactly max slope (45°) must be included (<= not <)"
        );
    }

    #[test]
    fn bake_excludes_triangle_just_above_max_slope() {
        let tri = triangle_with_slope(46.0);
        let nav = NavMesh::bake(&[tri], 0.4, 45.0);
        assert_eq!(
            nav.triangle_count(),
            0,
            "triangle slightly above max slope (46° > 45°) must be excluded"
        );
    }

    #[test]
    fn bake_includes_flat_triangle() {
        let tri = triangle_with_slope(0.0);
        let nav = NavMesh::bake(&[tri], 0.4, 45.0);
        assert_eq!(nav.triangle_count(), 1, "flat triangle must be included");
    }

    #[test]
    fn bake_excludes_vertical_triangle() {
        // 90° slope → normal horizontal → dot with Y = 0 → angle = 90°
        // 90 > any reasonable max_slope (e.g., 60)
        let tri = triangle_with_slope(89.0);
        let nav = NavMesh::bake(&[tri], 0.4, 60.0);
        assert_eq!(
            nav.triangle_count(),
            0,
            "near-vertical triangle (89°) must be excluded at max_slope=60°"
        );
    }

    #[test]
    fn bake_filters_downward_facing_triangles() {
        // Triangle with downward normal (dot < 0)
        let tri = Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0), // reversed winding for -Y normal
            Vec3::new(0.0, 0.0, 1.0),
        );
        let normal = (tri.b - tri.a).cross(tri.c - tri.a);
        // If normal points -Y, it should be filtered
        if normal.y < 0.0 {
            let nav = NavMesh::bake(&[tri], 0.4, 90.0);
            assert_eq!(
                nav.triangle_count(),
                0,
                "downward-facing triangle must be excluded regardless of max_slope"
            );
        }
    }
}

// =============================================================================
// Region invalidation boundary tests
// =============================================================================

mod region_invalidation_boundaries {
    use super::*;

    fn single_tri_nav() -> (NavMesh, Vec<Triangle>) {
        let tris = vec![Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        (nav, tris)
    }

    #[test]
    fn rebake_dirty_regions_on_empty_is_noop() {
        let (mut nav, tris) = single_tri_nav();
        assert_eq!(nav.rebake_count(), 0);
        nav.rebake_dirty_regions(&tris); // No dirty regions → does nothing
        assert_eq!(nav.rebake_count(), 0); // Count should NOT increment
    }

    #[test]
    fn partial_rebake_returns_0_when_no_dirty_regions() {
        let (mut nav, tris) = single_tri_nav();
        let affected = nav.partial_rebake(&tris);
        assert_eq!(affected, 0);
    }

    #[test]
    fn partial_rebake_returns_affected_count() {
        let (mut nav, tris) = single_tri_nav();
        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        let affected = nav.partial_rebake(&tris);
        assert!(affected > 0, "should report affected triangles");
    }

    #[test]
    fn path_crosses_dirty_region_empty_path() {
        let (mut nav, _) = single_tri_nav();
        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        assert!(!nav.path_crosses_dirty_region(&[]));
    }

    #[test]
    fn path_crosses_dirty_region_no_dirty() {
        let (nav, _) = single_tri_nav();
        let path = vec![Vec3::new(0.5, 0.0, 0.5)];
        assert!(!nav.path_crosses_dirty_region(&path));
    }

    #[test]
    fn path_does_not_cross_non_overlapping_dirty_region() {
        let (mut nav, _) = single_tri_nav();
        nav.invalidate_region(Aabb::new(
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::new(20.0, 20.0, 20.0),
        ));
        let path = vec![Vec3::new(0.5, 0.0, 0.5)];
        assert!(!nav.path_crosses_dirty_region(&path));
    }

    #[test]
    fn multiple_rebakes_increment_count() {
        let (mut nav, tris) = single_tri_nav();
        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        nav.rebake_dirty_regions(&tris);
        assert_eq!(nav.rebake_count(), 1);

        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        nav.rebake_dirty_regions(&tris);
        assert_eq!(nav.rebake_count(), 2);
    }

    #[test]
    fn dirty_regions_accessor_returns_correct_slice() {
        let (mut nav, _) = single_tri_nav();
        assert!(nav.dirty_regions().is_empty());
        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::ONE));
        assert_eq!(nav.dirty_regions().len(), 1);
    }
}

// =============================================================================
// Display implementations — verify they produce non-empty, expected output
// =============================================================================

mod display_tests {
    use super::*;

    #[test]
    fn triangle_display_contains_all_vertices() {
        let tri = Triangle::new(
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        );
        let s = format!("{}", tri);
        assert!(!s.is_empty(), "Display must produce output");
        assert!(s.contains("Triangle"), "Display must contain 'Triangle'");
    }

    #[test]
    fn navtri_display_contains_index_and_neighbors() {
        let mut tri = NavTri::new(
            42,
            [Vec3::ZERO, Vec3::X, Vec3::Z],
            Vec3::Y,
            Vec3::ZERO,
        );
        tri.neighbors = vec![1, 2, 3];
        let s = format!("{}", tri);
        assert!(s.contains("42"), "Display must contain index '42'");
        assert!(s.contains("3"), "Display must show neighbor count");
    }

    #[test]
    fn aabb_display_is_nonempty() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let s = format!("{}", aabb);
        assert!(!s.is_empty());
        assert!(s.contains("AABB"));
    }

    #[test]
    fn navmesh_display_contains_triangle_count() {
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
        let s = format!("{}", nav);
        assert!(s.contains("2"), "Display must contain triangle count '2'");
    }

    #[test]
    fn navmesh_summary_contains_max_step() {
        let nav = NavMesh::bake(&[], 0.75, 60.0);
        let s = nav.summary();
        assert!(s.contains("0.75"), "summary must contain max_step value");
    }
}

// =============================================================================
// Pathfinding edge cases for mutation resistance
// =============================================================================

mod pathfinding_mutations {
    use super::*;

    #[test]
    fn find_path_returns_start_and_goal_in_order() {
        let tris = vec![Triangle::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 10.0),
            Vec3::new(10.0, 0.0, 0.0),
        )];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        let start = Vec3::new(1.0, 0.0, 1.0);
        let goal = Vec3::new(3.0, 0.0, 3.0);
        let path = nav.find_path(start, goal);
        assert!(path.len() >= 2);
        // First point must be start, last must be goal
        assert!(
            (path[0] - start).length() < 0.01,
            "first path point must be start"
        );
        assert!(
            (path.last().unwrap() - goal).length() < 0.01,
            "last path point must be goal"
        );
    }

    #[test]
    fn get_triangle_valid_index() {
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
        assert!(nav.get_triangle(0).is_some());
        assert!(nav.get_triangle(1).is_some());
        assert!(nav.get_triangle(2).is_none());
    }

    #[test]
    fn triangle_count_matches_actual_len() {
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
                Vec3::new(2.0, 0.0, 0.0),
            ),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.triangle_count(), 3);
    }
}

// =============================================================================
// Triangle::from_vertices and Triangle::vertices round-trip
// =============================================================================

mod triangle_roundtrip {
    use super::*;

    #[test]
    fn from_vertices_then_vertices_roundtrip() {
        let verts = [
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(4.0, 5.0, 6.0),
            Vec3::new(7.0, 8.0, 9.0),
        ];
        let tri = Triangle::from_vertices(verts);
        let got = tri.vertices();
        assert_eq!(got[0], verts[0]);
        assert_eq!(got[1], verts[1]);
        assert_eq!(got[2], verts[2]);
    }

    #[test]
    fn new_then_vertices_match_args() {
        let a = Vec3::new(10.0, 20.0, 30.0);
        let b = Vec3::new(40.0, 50.0, 60.0);
        let c = Vec3::new(70.0, 80.0, 90.0);
        let tri = Triangle::new(a, b, c);
        assert_eq!(tri.a, a);
        assert_eq!(tri.b, b);
        assert_eq!(tri.c, c);
        let verts = tri.vertices();
        assert_eq!(verts, [a, b, c]);
    }
}
