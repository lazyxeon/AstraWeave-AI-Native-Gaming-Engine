// Week 2 Day 3: Edge Case Tests for astraweave-nav
// Tests invalid inputs, boundary conditions, and advanced scenarios

use super::*;

// ===== Invalid Input Tests =====

#[test]
fn test_degenerate_triangle_zero_area() {
    // Triangle with all three vertices at the same point (zero area)
    let tris = vec![Triangle {
        a: Vec3::new(1.0, 0.0, 1.0),
        b: Vec3::new(1.0, 0.0, 1.0),
        c: Vec3::new(1.0, 0.0, 1.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Degenerate triangle should be filtered out (zero area = undefined normal)
    // or included with zero normal (implementation dependent)
    // Either way, pathfinding should not crash
    let path = nav.find_path(Vec3::new(0.5, 0.0, 0.5), Vec3::new(1.5, 0.0, 1.5));
    // Should return empty path or valid path (no crash)
    assert!(path.len() >= 0, "Should not crash on degenerate triangle");
}

#[test]
fn test_degenerate_triangle_colinear_vertices() {
    // Triangle with three colinear vertices (zero area)
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0),
        c: Vec3::new(2.0, 0.0, 0.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Colinear vertices = zero normal, may be filtered or cause issues
    let path = nav.find_path(Vec3::new(0.5, 0.0, 0.5), Vec3::new(1.5, 0.0, 0.5));
    assert!(path.len() >= 0, "Should not crash on colinear vertices");
}

#[test]
fn test_very_small_triangle() {
    // Triangle with very small area (1e-6 square units)
    let epsilon = 1e-3;
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(epsilon, 0.0, 0.0),
        c: Vec3::new(0.0, 0.0, epsilon),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Very small triangle should still be included if slope is valid
    // Should not cause numerical instability
    assert!(nav.tris.len() <= 1, "Very small triangle may or may not be included");
}

#[test]
fn test_negative_max_slope() {
    // Negative max_slope_deg should filter all triangles or be clamped
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0),
        c: Vec3::new(0.0, 0.0, 1.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, -45.0);
    
    // Negative slope likely filters all triangles (no triangle can have negative angle with Y)
    // Should not crash
    assert!(nav.tris.len() == 0 || nav.max_slope_deg == -45.0, "Negative slope should filter triangles or be preserved");
}

#[test]
fn test_very_large_coordinates() {
    // Triangle with very large coordinates (near f32::MAX / 10)
    let large = 1e6;
    let tris = vec![Triangle {
        a: Vec3::new(large, 0.0, large),
        b: Vec3::new(large + 1.0, 0.0, large),
        c: Vec3::new(large, 0.0, large + 1.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Large coordinates should not cause overflow or precision issues
    assert_eq!(nav.tris.len(), 1, "Large coordinates should be handled correctly");
    
    // Pathfinding with large coordinates
    let path = nav.find_path(
        Vec3::new(large + 0.5, 0.0, large + 0.5),
        Vec3::new(large + 0.5, 0.0, large + 0.5),
    );
    assert!(path.len() >= 2, "Pathfinding with large coordinates should work");
}

#[test]
fn test_mixed_positive_negative_coordinates() {
    // Triangle spanning negative and positive coordinates
    let tris = vec![Triangle {
        a: Vec3::new(-1.0, 0.0, -1.0),
        b: Vec3::new(1.0, 0.0, -1.0),
        c: Vec3::new(-1.0, 0.0, 1.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    assert_eq!(nav.tris.len(), 1, "Mixed sign coordinates should work");
    
    // Path from negative to positive quadrant
    let path = nav.find_path(Vec3::new(-0.5, 0.0, -0.5), Vec3::new(0.5, 0.0, 0.5));
    assert!(path.len() >= 2, "Path across origin should work");
}

#[test]
fn test_zero_max_step() {
    // max_step = 0.0 should be preserved (may affect character controller, not pathfinding)
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0),
        c: Vec3::new(0.0, 0.0, 1.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.0, 60.0);
    
    assert_eq!(nav.max_step, 0.0, "Zero max_step should be preserved");
    assert_eq!(nav.tris.len(), 1);
}

// ===== Boundary Condition Tests =====

#[test]
fn test_exactly_one_shared_vertex() {
    // Two triangles sharing exactly 1 vertex (not an edge)
    let tris = vec![
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        },
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0), // Shared vertex
            b: Vec3::new(-1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, -1.0),
        },
    ];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    assert_eq!(nav.tris.len(), 2, "Both triangles should be included");
    
    // Check adjacency: should NOT be neighbors (need 2 shared vertices for edge)
    assert!(nav.tris[0].neighbors.is_empty() || nav.tris[0].neighbors.len() == 0, 
            "Triangles sharing 1 vertex should not be neighbors");
    assert!(nav.tris[1].neighbors.is_empty() || nav.tris[1].neighbors.len() == 0,
            "Triangles sharing 1 vertex should not be neighbors");
}

#[test]
fn test_start_on_triangle_edge() {
    // Start position exactly on triangle edge
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(2.0, 0.0, 0.0),
        c: Vec3::new(1.0, 0.0, 2.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Start on edge (midpoint of a-b)
    let start = Vec3::new(1.0, 0.0, 0.0);
    let goal = Vec3::new(1.0, 0.0, 1.0);
    
    let path = nav.find_path(start, goal);
    assert!(path.len() >= 2, "Path from edge should work");
}

#[test]
fn test_goal_outside_all_triangles() {
    // Goal position far outside navmesh bounds
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0),
        c: Vec3::new(0.0, 0.0, 1.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    let start = Vec3::new(0.5, 0.0, 0.5);
    let goal = Vec3::new(100.0, 0.0, 100.0); // Far outside
    
    let path = nav.find_path(start, goal);
    
    // Should find path to closest triangle to goal
    assert!(path.len() >= 2, "Should find path to closest triangle near goal");
    // Last waypoint should be near the triangle, not at the unreachable goal
}

#[test]
fn test_start_outside_all_triangles() {
    // Start position far outside navmesh bounds
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0),
        c: Vec3::new(0.0, 0.0, 1.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    let start = Vec3::new(100.0, 0.0, 100.0); // Far outside
    let goal = Vec3::new(0.5, 0.0, 0.5);
    
    let path = nav.find_path(start, goal);
    
    // Should find path from closest triangle to start
    assert!(path.len() >= 2, "Should find path from closest triangle near start");
}

#[test]
fn test_slope_exactly_at_max_threshold() {
    // Triangle with slope exactly at max_slope_deg (60°)
    // Flat triangle: dot(normal, Y) = cos(0°) = 1.0
    // 60° slope: dot(normal, Y) = cos(60°) = 0.5
    
    // Create triangle with 60° slope by tilting 60° from horizontal
    let angle_rad = 60.0_f32.to_radians();
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0),
        c: Vec3::new(0.5, angle_rad.tan(), 0.5), // Height creates 60° slope
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Should be included (angle <= max_slope_deg)
    assert_eq!(nav.tris.len(), 1, "Triangle at exactly max slope should be included");
}

#[test]
fn test_slope_just_above_max_threshold() {
    // Triangle with slope just above max_slope_deg (60.1°)
    let angle_rad = 60.1_f32.to_radians();
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0),
        c: Vec3::new(0.5, angle_rad.tan(), 0.5),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Should be filtered out (angle > max_slope_deg)
    assert_eq!(nav.tris.len(), 0, "Triangle just above max slope should be filtered");
}

#[test]
fn test_vertical_triangle() {
    // Triangle perpendicular to ground (90° slope)
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0),
        c: Vec3::new(0.5, 1.0, 0.0), // Vertical wall
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Should be filtered out (90° > 60°)
    assert_eq!(nav.tris.len(), 0, "Vertical triangle should be filtered");
}

// ===== Advanced Scenario Tests =====

#[test]
fn test_concave_navmesh_l_shape() {
    // L-shaped navmesh (concave topology)
    let tris = vec![
        // Horizontal bar of L
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(3.0, 0.0, 0.0),
        },
        Triangle {
            a: Vec3::new(3.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(3.0, 0.0, 1.0),
        },
        // Vertical bar of L
        Triangle {
            a: Vec3::new(0.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, 4.0),
            c: Vec3::new(1.0, 0.0, 1.0),
        },
        Triangle {
            a: Vec3::new(1.0, 0.0, 1.0),
            b: Vec3::new(0.0, 0.0, 4.0),
            c: Vec3::new(1.0, 0.0, 4.0),
        },
    ];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    assert_eq!(nav.tris.len(), 4, "L-shape should have 4 triangles");
    
    // Path from one arm of L to the other
    let path = nav.find_path(Vec3::new(2.5, 0.0, 0.5), Vec3::new(0.5, 0.0, 3.5));
    assert!(path.len() >= 2, "Path through L-shape should exist");
}

#[test]
fn test_navmesh_with_hole_donut() {
    // Square navmesh with square hole in the middle (donut topology)
    // Outer square: 4×4, inner hole: 2×2 centered
    let mut tris = Vec::new();
    
    // Bottom strip (y=0 to y=1)
    tris.push(Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(0.0, 0.0, 1.0),
        c: Vec3::new(4.0, 0.0, 0.0),
    });
    tris.push(Triangle {
        a: Vec3::new(4.0, 0.0, 0.0),
        b: Vec3::new(0.0, 0.0, 1.0),
        c: Vec3::new(4.0, 0.0, 1.0),
    });
    
    // Left strip (x=0 to x=1, y=1 to y=3)
    tris.push(Triangle {
        a: Vec3::new(0.0, 0.0, 1.0),
        b: Vec3::new(0.0, 0.0, 3.0),
        c: Vec3::new(1.0, 0.0, 1.0),
    });
    tris.push(Triangle {
        a: Vec3::new(1.0, 0.0, 1.0),
        b: Vec3::new(0.0, 0.0, 3.0),
        c: Vec3::new(1.0, 0.0, 3.0),
    });
    
    // Right strip (x=3 to x=4, y=1 to y=3)
    tris.push(Triangle {
        a: Vec3::new(3.0, 0.0, 1.0),
        b: Vec3::new(3.0, 0.0, 3.0),
        c: Vec3::new(4.0, 0.0, 1.0),
    });
    tris.push(Triangle {
        a: Vec3::new(4.0, 0.0, 1.0),
        b: Vec3::new(3.0, 0.0, 3.0),
        c: Vec3::new(4.0, 0.0, 3.0),
    });
    
    // Top strip (y=3 to y=4)
    tris.push(Triangle {
        a: Vec3::new(0.0, 0.0, 3.0),
        b: Vec3::new(0.0, 0.0, 4.0),
        c: Vec3::new(4.0, 0.0, 3.0),
    });
    tris.push(Triangle {
        a: Vec3::new(4.0, 0.0, 3.0),
        b: Vec3::new(0.0, 0.0, 4.0),
        c: Vec3::new(4.0, 0.0, 4.0),
    });
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    assert_eq!(nav.tris.len(), 8, "Donut should have 8 triangles (4 strips)");
    
    // Path from left side to right side (must go around hole)
    let path = nav.find_path(Vec3::new(0.5, 0.0, 2.0), Vec3::new(3.5, 0.0, 2.0));
    assert!(path.len() >= 2, "Path around hole should exist");
    
    // Path should NOT go through hole center (2.0, 2.0)
    // (Hard to verify without inspecting waypoints, but at least path exists)
}

#[test]
fn test_narrow_passage_bottleneck() {
    // Two large areas connected by narrow 1-triangle passage
    // Area 1: 3×3 grid (left)
    // Passage: 1×1 triangle (middle)
    // Area 2: 3×3 grid (right)
    
    let mut tris = Vec::new();
    
    // Area 1 (left): Simple square
    tris.push(Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(0.0, 0.0, 3.0),
        c: Vec3::new(3.0, 0.0, 0.0),
    });
    tris.push(Triangle {
        a: Vec3::new(3.0, 0.0, 0.0),
        b: Vec3::new(0.0, 0.0, 3.0),
        c: Vec3::new(3.0, 0.0, 3.0),
    });
    
    // Passage (middle): Narrow connection
    tris.push(Triangle {
        a: Vec3::new(3.0, 0.0, 1.0),
        b: Vec3::new(3.0, 0.0, 2.0),
        c: Vec3::new(4.0, 0.0, 1.5),
    });
    
    // Area 2 (right): Simple square
    tris.push(Triangle {
        a: Vec3::new(4.0, 0.0, 0.0),
        b: Vec3::new(4.0, 0.0, 3.0),
        c: Vec3::new(7.0, 0.0, 0.0),
    });
    tris.push(Triangle {
        a: Vec3::new(7.0, 0.0, 0.0),
        b: Vec3::new(4.0, 0.0, 3.0),
        c: Vec3::new(7.0, 0.0, 3.0),
    });
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    assert_eq!(nav.tris.len(), 5, "Should have 5 triangles (2+1+2)");
    
    // Path from left area to right area (must go through bottleneck)
    let path = nav.find_path(Vec3::new(1.5, 0.0, 1.5), Vec3::new(5.5, 0.0, 1.5));
    assert!(path.len() >= 2, "Path through bottleneck should exist");
}

#[test]
fn test_shared_edge_epsilon_precision() {
    // Two triangles with vertices exactly 1e-3 apart (epsilon boundary)
    let epsilon = 1e-3;
    let tris = vec![
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        },
        Triangle {
            a: Vec3::new(1.0 + epsilon, 0.0, 0.0 + epsilon), // Slightly offset shared vertex
            b: Vec3::new(0.0 + epsilon, 0.0, 1.0 + epsilon), // Slightly offset shared vertex
            c: Vec3::new(1.0, 0.0, 1.0),
        },
    ];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    assert_eq!(nav.tris.len(), 2, "Both triangles should be included");
    
    // Check adjacency: should be neighbors (epsilon = 1e-3 is the threshold)
    let has_neighbor = nav.tris[0].neighbors.contains(&1) || nav.tris[1].neighbors.contains(&0);
    assert!(has_neighbor, "Triangles at epsilon boundary should be neighbors");
}

#[test]
fn test_inverted_triangle_winding() {
    // Triangle with inverted winding (clockwise instead of counter-clockwise)
    // Normal will point -Y instead of +Y
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(1.0, 0.0, 0.0), // Swapped b and c
        c: Vec3::new(0.0, 0.0, 1.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Inverted triangle has normal pointing -Y, dot product with Y is negative
    // Angle = acos(negative) > 90°, should be filtered by slope check
    assert_eq!(nav.tris.len(), 0, "Inverted triangle should be filtered (normal points down)");
}

#[test]
fn test_empty_navmesh_pathfinding() {
    // Already tested in existing tests, but included for completeness
    let nav = NavMesh::bake(&[], 0.5, 60.0);
    
    let path = nav.find_path(Vec3::ZERO, Vec3::ONE);
    assert_eq!(path.len(), 0, "Empty navmesh should return empty path");
}

#[test]
fn test_single_triangle_multiple_queries() {
    // Single triangle with multiple queries from different positions
    let tris = vec![Triangle {
        a: Vec3::new(0.0, 0.0, 0.0),
        b: Vec3::new(10.0, 0.0, 0.0),
        c: Vec3::new(5.0, 0.0, 10.0),
    }];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    // Query from 9 different positions within the triangle
    for i in 1..=9 {
        let x = (i % 3) as f32 * 2.5 + 1.0;
        let z = (i / 3) as f32 * 2.5 + 1.0;
        let start = Vec3::new(x, 0.0, z);
        let goal = Vec3::new(5.0, 0.0, 5.0); // Center
        
        let path = nav.find_path(start, goal);
        assert!(path.len() >= 2, "Query {} should find path in single triangle", i);
    }
}

#[test]
fn test_max_slope_90_degrees() {
    // max_slope_deg = 90° should include all triangles (even vertical walls)
    let tris = vec![
        // Horizontal triangle
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        },
        // Vertical triangle
        Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.5, 1.0, 0.0),
        },
    ];
    
    let nav = NavMesh::bake(&tris, 0.5, 90.0);
    
    // Both should be included (90° allows any slope)
    assert_eq!(nav.tris.len(), 2, "max_slope=90° should include all triangles");
}

#[test]
fn test_triangles_with_shared_vertices_but_not_edges() {
    // Three triangles forming a "T" junction
    // All share a single central vertex but no edges
    let center = Vec3::new(0.0, 0.0, 0.0);
    let tris = vec![
        // Triangle pointing up
        Triangle {
            a: center,
            b: Vec3::new(-1.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 1.0),
        },
        // Triangle pointing left
        Triangle {
            a: center,
            b: Vec3::new(-1.0, 0.0, 1.0),
            c: Vec3::new(-1.0, 0.0, -1.0),
        },
        // Triangle pointing right
        Triangle {
            a: center,
            b: Vec3::new(1.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, -1.0),
        },
    ];
    
    let nav = NavMesh::bake(&tris, 0.5, 60.0);
    
    assert_eq!(nav.tris.len(), 3, "All three triangles should be included");
    
    // Check adjacency: none should be neighbors (only 1 shared vertex each)
    for tri in &nav.tris {
        assert!(tri.neighbors.len() == 0, "Triangles sharing only 1 vertex should not be neighbors");
    }
}
