// Test helper to detect triangles with incorrect winding order
// This analyzes all edge case tests and reports which triangles have downward-facing normals

use astraweave_nav::Triangle;
use glam::Vec3;

#[test]
fn detect_downward_triangles() {
    println!("\n=== WINDING ORDER ANALYSIS ===\n");

    // Test: test_mixed_positive_negative_coordinates
    println!("test_mixed_positive_negative_coordinates:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(-1.0, 0.0, -1.0),
            b: Vec3::new(1.0, 0.0, -1.0),
            c: Vec3::new(-1.0, 0.0, 1.0),
        },
    );

    // Test: test_very_large_coordinates
    let large = 1e6;
    println!("\ntest_very_large_coordinates:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(large, 0.0, large),
            b: Vec3::new(large + 1.0, 0.0, large),
            c: Vec3::new(large, 0.0, large + 1.0),
        },
    );

    // Test: test_zero_max_step
    println!("\ntest_zero_max_step:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        },
    );

    // Test: test_exactly_one_shared_vertex
    println!("\ntest_exactly_one_shared_vertex:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Triangle 2",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(-1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, -1.0),
        },
    );

    // Test: test_start_on_triangle_edge
    println!("\ntest_start_on_triangle_edge:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 2.0),
            c: Vec3::new(2.0, 0.0, 0.0),
        },
    );

    // Test: test_goal_outside_all_triangles
    println!("\ntest_goal_outside_all_triangles:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        },
    );

    // Test: test_start_outside_all_triangles
    println!("\ntest_start_outside_all_triangles:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        },
    );

    // Test: test_single_triangle_multiple_queries
    println!("\ntest_single_triangle_multiple_queries:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(5.0, 0.0, 10.0),
            c: Vec3::new(10.0, 0.0, 0.0),
        },
    );

    // Test: test_max_slope_90_degrees
    println!("\ntest_max_slope_90_degrees:");
    check_triangle(
        "Triangle 1 (horizontal)",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Triangle 2 (vertical)",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.5, 1.0, 0.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        },
    );

    // Test: test_triangles_with_shared_vertices_but_not_edges
    println!("\ntest_triangles_with_shared_vertices_but_not_edges:");
    let center = Vec3::new(0.0, 0.0, 0.0);
    check_triangle(
        "Triangle 1 (up)",
        Triangle {
            a: center,
            b: Vec3::new(-1.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Triangle 2 (left)",
        Triangle {
            a: center,
            b: Vec3::new(-1.0, 0.0, -1.0),
            c: Vec3::new(-1.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Triangle 3 (right)",
        Triangle {
            a: center,
            b: Vec3::new(1.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, -1.0),
        },
    );

    // Test: test_concave_navmesh_l_shape
    println!("\ntest_concave_navmesh_l_shape:");
    check_triangle(
        "Horizontal bar 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(3.0, 0.0, 0.0),
        },
    );
    check_triangle(
        "Horizontal bar 2",
        Triangle {
            a: Vec3::new(3.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(3.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Vertical bar 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, 4.0),
            c: Vec3::new(1.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Vertical bar 2",
        Triangle {
            a: Vec3::new(1.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, 4.0),
            c: Vec3::new(1.0, 0.0, 4.0),
        },
    );

    // Test: test_navmesh_with_hole_donut
    println!("\ntest_navmesh_with_hole_donut:");
    check_triangle(
        "Bottom strip 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(4.0, 0.0, 0.0),
        },
    );
    check_triangle(
        "Bottom strip 2",
        Triangle {
            a: Vec3::new(4.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(4.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Left strip 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, 3.0),
            c: Vec3::new(1.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Left strip 2",
        Triangle {
            a: Vec3::new(1.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, 3.0),
            c: Vec3::new(1.0, 0.0, 3.0),
        },
    );
    check_triangle(
        "Right strip 1",
        Triangle {
            a: Vec3::new(3.0, 0.0, 1.0),
            b: Vec3::new(3.0, 0.0, 3.0),
            c: Vec3::new(4.0, 0.0, 1.0),
        },
    );
    check_triangle(
        "Right strip 2",
        Triangle {
            a: Vec3::new(4.0, 0.0, 1.0),
            b: Vec3::new(3.0, 0.0, 3.0),
            c: Vec3::new(4.0, 0.0, 3.0),
        },
    );
    check_triangle(
        "Top strip 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 3.0),
            b: Vec3::new(0.0, 0.0, 4.0),
            c: Vec3::new(4.0, 0.0, 3.0),
        },
    );
    check_triangle(
        "Top strip 2",
        Triangle {
            a: Vec3::new(4.0, 0.0, 3.0),
            b: Vec3::new(0.0, 0.0, 4.0),
            c: Vec3::new(4.0, 0.0, 4.0),
        },
    );

    // Test: test_narrow_passage_bottleneck
    println!("\ntest_narrow_passage_bottleneck:");
    check_triangle(
        "Area 1 triangle 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 3.0),
            c: Vec3::new(3.0, 0.0, 0.0),
        },
    );
    check_triangle(
        "Area 1 triangle 2",
        Triangle {
            a: Vec3::new(3.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 3.0),
            c: Vec3::new(3.0, 0.0, 3.0),
        },
    );
    check_triangle(
        "Passage",
        Triangle {
            a: Vec3::new(3.0, 0.0, 1.0),
            b: Vec3::new(3.0, 0.0, 2.0),
            c: Vec3::new(4.0, 0.0, 1.5),
        },
    );
    check_triangle(
        "Area 2 triangle 1",
        Triangle {
            a: Vec3::new(4.0, 0.0, 0.0),
            b: Vec3::new(4.0, 0.0, 3.0),
            c: Vec3::new(7.0, 0.0, 0.0),
        },
    );
    check_triangle(
        "Area 2 triangle 2",
        Triangle {
            a: Vec3::new(7.0, 0.0, 0.0),
            b: Vec3::new(4.0, 0.0, 3.0),
            c: Vec3::new(7.0, 0.0, 3.0),
        },
    );

    // Test: test_shared_edge_epsilon_precision
    let epsilon = 1e-3;
    println!("\ntest_shared_edge_epsilon_precision:");
    check_triangle(
        "Triangle 1",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        },
    );
    check_triangle(
        "Triangle 2",
        Triangle {
            a: Vec3::new(1.0 + epsilon, 0.0, 0.0 + epsilon),
            b: Vec3::new(0.0 + epsilon, 0.0, 1.0 + epsilon),
            c: Vec3::new(1.0, 0.0, 1.0),
        },
    );

    // Test: test_slope_exactly_at_max_threshold
    println!("\ntest_slope_exactly_at_max_threshold:");
    let angle_rad = 60.0_f32.to_radians();
    check_triangle(
        "60° slope triangle",
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.5, angle_rad.tan(), 0.5),
        },
    );

    println!("\n=== END ANALYSIS ===\n");

    // This test is primarily for reporting; assert something non-constant to satisfy clippy.
    assert!(angle_rad.is_finite());
}

fn check_triangle(name: &str, tri: Triangle) {
    let edge1 = tri.b - tri.a;
    let edge2 = tri.c - tri.a;
    let normal = edge1.cross(edge2);
    let normalized = normal.normalize_or_zero();

    let dot = normalized.dot(Vec3::Y);
    let angle = if normalized.length_squared() > 1e-6 {
        dot.clamp(-1.0, 1.0).acos().to_degrees()
    } else {
        f32::NAN
    };

    let status = if normalized.length_squared() < 1e-6 {
        "❌ DEGENERATE (zero normal)".to_string()
    } else if dot < 0.0 {
        "❌ DOWNWARD (needs b/c swap)".to_string()
    } else if angle.is_nan() {
        "❌ NAN angle".to_string()
    } else {
        format!("✅ UPWARD ({}°)", angle.round())
    };

    println!(
        "  {} - {} | normal: ({:.2}, {:.2}, {:.2})",
        name, status, normalized.x, normalized.y, normalized.z
    );

    if dot < 0.0 && normalized.length_squared() >= 1e-6 {
        println!("    FIX: Swap b and c vertices");
    }
}
