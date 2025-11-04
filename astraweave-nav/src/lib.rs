use glam::Vec3;

#[derive(Clone, Debug)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

#[derive(Clone, Debug)]
pub struct NavTri {
    pub idx: usize,
    pub verts: [Vec3; 3],
    pub normal: Vec3,
    pub center: Vec3,
    pub neighbors: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct NavMesh {
    pub tris: Vec<NavTri>,
    pub max_step: f32,
    pub max_slope_deg: f32,
}

impl NavMesh {
    pub fn bake(tris: &[Triangle], max_step: f32, max_slope_deg: f32) -> Self {
        let mut ntris: Vec<NavTri> = tris
            .iter()
            .enumerate()
            .filter_map(|(i, t)| {
                let n = (t.b - t.a).cross(t.c - t.a).normalize_or_zero();

                // Handle degenerate triangles (zero normal)
                if n.length_squared() < 1e-6 {
                    return None;
                }

                // Calculate angle from vertical (Y-axis)
                // Accept only upward-facing triangles (dot product >= 0)
                let dot = n.dot(Vec3::Y).clamp(-1.0, 1.0);
                if dot < 0.0 {
                    // Downward-facing triangle, filter it
                    return None;
                }

                let angle_from_vertical = dot.acos().to_degrees();

                // Accept triangles where angle from vertical <= max_slope
                // (0° = flat horizontal, 90° = vertical wall)
                let slope_ok = angle_from_vertical <= max_slope_deg;

                if !slope_ok {
                    return None;
                }
                let center = (t.a + t.b + t.c) / 3.0;
                Some(NavTri {
                    idx: i,
                    verts: [t.a, t.b, t.c],
                    normal: n,
                    center,
                    neighbors: vec![],
                })
            })
            .collect();

        // Build adjacency by shared edge (position‑based, epsilon)
        let eps = 1e-3;
        for i in 0..ntris.len() {
            for j in i + 1..ntris.len() {
                if share_edge(&ntris[i], &ntris[j], eps) {
                    ntris[i].neighbors.push(j);
                    ntris[j].neighbors.push(i);
                }
            }
        }

        Self {
            tris: ntris,
            max_step,
            max_slope_deg,
        }
    }

    pub fn find_path(&self, start: Vec3, goal: Vec3) -> Vec<Vec3> {
        let s = closest_tri(&self.tris, start);
        let g = closest_tri(&self.tris, goal);
        if s.is_none() || g.is_none() {
            return vec![];
        }
        let (s, g) = (
            s.expect("BUG: start triangle should be Some after is_none check"),
            g.expect("BUG: goal triangle should be Some after is_none check"),
        );
        let idx_path = astar_tri(&self.tris, s, g);
        if idx_path.is_empty() {
            return vec![];
        }

        // seed with start and goal
        let mut pts = vec![start];
        for ti in idx_path
            .iter()
            .skip(1)
            .take(idx_path.len().saturating_sub(2))
        {
            pts.push(self.tris[*ti].center);
        }
        pts.push(goal);

        // optional: simple smoothing
        smooth(&mut pts, &self.tris);

        pts
    }
}

fn share_edge(a: &NavTri, b: &NavTri, eps: f32) -> bool {
    let mut shared = 0;
    for va in a.verts {
        for vb in b.verts {
            if va.distance(vb) <= eps {
                shared += 1;
            }
        }
    }
    shared >= 2
}

fn closest_tri(tris: &[NavTri], p: Vec3) -> Option<usize> {
    tris.iter()
        .enumerate()
        .min_by(|(_, x), (_, y)| {
            x.center
                .distance_squared(p)
                .total_cmp(&y.center.distance_squared(p))
        })
        .map(|(i, _)| i)
}

fn astar_tri(tris: &[NavTri], start: usize, goal: usize) -> Vec<usize> {
    use std::cmp::Ordering;
    use std::collections::{BinaryHeap, HashMap};

    #[derive(Copy, Clone, PartialEq)]
    struct Node {
        f: f32,
        i: usize,
    }
    impl Eq for Node {}
    impl Ord for Node {
        fn cmp(&self, o: &Self) -> Ordering {
            o.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
        }
    }
    impl PartialOrd for Node {
        fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
            Some(self.cmp(o))
        }
    }

    let mut open = BinaryHeap::new();
    let mut came: HashMap<usize, usize> = HashMap::new();
    let mut gscore: HashMap<usize, f32> = HashMap::new();

    open.push(Node { f: 0.0, i: start });
    gscore.insert(start, 0.0);

    while let Some(Node { i, .. }) = open.pop() {
        if i == goal {
            break;
        }
        let gi = *gscore.get(&i).unwrap_or(&f32::INFINITY);
        for &nb in &tris[i].neighbors {
            let cost = tris[i].center.distance(tris[nb].center);
            let ng = gi + cost;
            if ng < *gscore.get(&nb).unwrap_or(&f32::INFINITY) {
                came.insert(nb, i);
                gscore.insert(nb, ng);
                let f = ng + tris[nb].center.distance(tris[goal].center);
                open.push(Node { f, i: nb });
            }
        }
    }

    // reconstruct
    let mut path = vec![];
    let mut cur = goal;
    path.push(cur);
    while let Some(&prev) = came.get(&cur) {
        cur = prev;
        path.push(cur);
        if cur == start {
            break;
        }
    }
    path.reverse();
    if path.first().copied() != Some(start) {
        return vec![];
    }
    path
}

fn smooth(pts: &mut [Vec3], _tris: &[NavTri]) {
    if pts.len() < 3 {
        return;
    }
    for _ in 0..2 {
        for i in 1..pts.len() - 1 {
            let a = pts[i - 1];
            let b = pts[i + 1];
            pts[i] = a * 0.25 + pts[i] * 0.5 + b * 0.25;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== NavMesh Baking Tests =====

    #[test]
    fn test_navmesh_bake_empty() {
        let nav = NavMesh::bake(&[], 0.4, 60.0);
        assert_eq!(nav.tris.len(), 0);
        assert_eq!(nav.max_step, 0.4);
        assert_eq!(nav.max_slope_deg, 60.0);
    }

    #[test]
    fn test_navmesh_bake_single_triangle() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0), // CCW winding for +Y normal
            c: Vec3::new(1.0, 0.0, 0.0),
        }];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.tris.len(), 1);
        assert_eq!(nav.tris[0].idx, 0);
        assert_eq!(nav.tris[0].neighbors.len(), 0);
    }

    #[test]
    fn test_navmesh_bake_filters_steep_slopes() {
        let tris = vec![
            // Flat triangle (normal = +Y, slope 0°)
            Triangle {
                a: Vec3::new(0.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 1.0),
                c: Vec3::new(1.0, 0.0, 0.0),
            },
            // Steep triangle (normal pointing away from +Y, slope > 45°)
            Triangle {
                a: Vec3::new(2.0, 0.0, 0.0),
                b: Vec3::new(2.0, 1.0, 0.0),
                c: Vec3::new(2.0, 0.0, 1.0),
            },
        ];
        let nav = NavMesh::bake(&tris, 0.4, 45.0); // max_slope_deg=45°
        assert_eq!(nav.tris.len(), 1); // Only flat triangle included
        assert_eq!(nav.tris[0].idx, 0);
    }

    #[test]
    fn test_navmesh_bake_adjacency_two_triangles() {
        let tris = vec![
            Triangle {
                a: Vec3::new(0.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 1.0),
                c: Vec3::new(1.0, 0.0, 0.0),
            },
            Triangle {
                a: Vec3::new(1.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 1.0),
                c: Vec3::new(1.0, 0.0, 1.0),
            },
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.tris.len(), 2);
        assert_eq!(nav.tris[0].neighbors, vec![1]);
        assert_eq!(nav.tris[1].neighbors, vec![0]);
    }

    #[test]
    fn test_navmesh_bake_center_calculation() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 3.0),
            c: Vec3::new(3.0, 0.0, 0.0),
        }];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.tris.len(), 1); // Verify triangle was included
        let center = nav.tris[0].center;
        assert!((center.x - 1.0).abs() < 1e-5);
        assert!((center.y - 0.0).abs() < 1e-5);
        assert!((center.z - 1.0).abs() < 1e-5);
    }

    // ===== Pathfinding Tests =====

    #[test]
    fn path_exists_simple_strip() {
        // Two triangles forming a square on XZ plane
        let tris = vec![
            // Winding chosen so normals point +Y
            Triangle {
                a: Vec3::new(0.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 1.0),
                c: Vec3::new(1.0, 0.0, 0.0),
            },
            Triangle {
                a: Vec3::new(1.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 1.0),
                c: Vec3::new(1.0, 0.0, 1.0),
            },
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        // Use start and goal in same triangle for deterministic minimal path
        let path = nav.find_path(Vec3::new(0.1, 0.0, 0.1), Vec3::new(0.45, 0.0, 0.45));
        assert!(
            path.len() >= 2,
            "expected at least start+goal, got {:?}",
            path
        );
        assert!((path.first().unwrap().x - 0.1).abs() < 1e-3);
        assert!((path.last().unwrap().x - 0.45).abs() < 1e-3);
    }

    #[test]
    fn test_find_path_empty_navmesh() {
        let nav = NavMesh::bake(&[], 0.4, 60.0);
        let path = nav.find_path(Vec3::ZERO, Vec3::ONE);
        assert_eq!(path.len(), 0);
    }

    #[test]
    fn test_find_path_same_triangle() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 10.0),
            c: Vec3::new(10.0, 0.0, 0.0),
        }];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.tris.len(), 1); // Verify triangle was included
        let path = nav.find_path(Vec3::new(1.0, 0.0, 1.0), Vec3::new(2.0, 0.0, 2.0));
        assert!(path.len() >= 2);
        assert!((path[0] - Vec3::new(1.0, 0.0, 1.0)).length() < 0.1);
        assert!((path.last().unwrap() - Vec3::new(2.0, 0.0, 2.0)).length() < 0.1);
    }

    #[test]
    fn test_find_path_across_triangles() {
        // Three connected triangles in a line
        let tris = vec![
            Triangle {
                a: Vec3::new(0.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 1.0),
                c: Vec3::new(1.0, 0.0, 0.0),
            },
            Triangle {
                a: Vec3::new(1.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 1.0),
                c: Vec3::new(1.0, 0.0, 1.0),
            },
            Triangle {
                a: Vec3::new(1.0, 0.0, 0.0),
                b: Vec3::new(1.0, 0.0, 1.0),
                c: Vec3::new(2.0, 0.0, 0.0),
            },
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert!(nav.tris.len() >= 2); // At least 2 triangles connected
        let path = nav.find_path(Vec3::new(0.2, 0.0, 0.2), Vec3::new(1.8, 0.0, 0.2));
        assert!(path.len() >= 2);
        // Path should go from first to last triangle
        assert!((path[0].x - 0.2).abs() < 0.1);
        assert!((path.last().unwrap().x - 1.8).abs() < 0.1);
    }

    #[test]
    fn test_find_path_no_connection() {
        // Two disconnected triangles
        let tris = vec![
            Triangle {
                a: Vec3::new(0.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 1.0),
                c: Vec3::new(1.0, 0.0, 0.0),
            },
            Triangle {
                a: Vec3::new(10.0, 0.0, 0.0),
                b: Vec3::new(10.0, 0.0, 1.0),
                c: Vec3::new(11.0, 0.0, 0.0),
            },
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.tris.len(), 2);
        assert_eq!(nav.tris[0].neighbors.len(), 0);
        assert_eq!(nav.tris[1].neighbors.len(), 0);

        let path = nav.find_path(Vec3::new(0.5, 0.0, 0.5), Vec3::new(10.5, 0.0, 0.5));
        assert_eq!(path.len(), 0); // No path possible
    }

    // ===== Helper Function Tests =====

    #[test]
    fn test_share_edge_true() {
        let tri_a = NavTri {
            idx: 0,
            verts: [
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
            normal: Vec3::Y,
            center: Vec3::ZERO,
            neighbors: vec![],
        };
        let tri_b = NavTri {
            idx: 1,
            verts: [
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                Vec3::new(1.0, 0.0, 1.0),
            ],
            normal: Vec3::Y,
            center: Vec3::ZERO,
            neighbors: vec![],
        };
        assert!(share_edge(&tri_a, &tri_b, 1e-3));
    }

    #[test]
    fn test_share_edge_false() {
        let tri_a = NavTri {
            idx: 0,
            verts: [
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
            normal: Vec3::Y,
            center: Vec3::ZERO,
            neighbors: vec![],
        };
        let tri_b = NavTri {
            idx: 1,
            verts: [
                Vec3::new(10.0, 0.0, 0.0),
                Vec3::new(11.0, 0.0, 0.0),
                Vec3::new(10.0, 0.0, 1.0),
            ],
            normal: Vec3::Y,
            center: Vec3::ZERO,
            neighbors: vec![],
        };
        assert!(!share_edge(&tri_a, &tri_b, 1e-3));
    }

    #[test]
    fn test_share_edge_epsilon_boundary() {
        let tri_a = NavTri {
            idx: 0,
            verts: [
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
            ],
            normal: Vec3::Y,
            center: Vec3::ZERO,
            neighbors: vec![],
        };
        let tri_b = NavTri {
            idx: 1,
            verts: [
                Vec3::new(1.0005, 0.0, 0.0), // Within 1e-3 epsilon
                Vec3::new(0.0005, 0.0, 1.0), // Within 1e-3 epsilon
                Vec3::new(2.0, 0.0, 0.0),
            ],
            normal: Vec3::Y,
            center: Vec3::ZERO,
            neighbors: vec![],
        };
        assert!(share_edge(&tri_a, &tri_b, 1e-3));
    }

    #[test]
    fn test_closest_tri_empty() {
        let result = closest_tri(&[], Vec3::ZERO);
        assert_eq!(result, None);
    }

    #[test]
    fn test_closest_tri_single() {
        let tris = vec![NavTri {
            idx: 0,
            verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
            normal: Vec3::Y,
            center: Vec3::new(0.5, 0.0, 0.5),
            neighbors: vec![],
        }];
        let result = closest_tri(&tris, Vec3::new(0.6, 0.0, 0.6));
        assert_eq!(result, Some(0));
    }

    #[test]
    fn test_closest_tri_multiple() {
        let tris = vec![
            NavTri {
                idx: 0,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(0.5, 0.0, 0.5),
                neighbors: vec![],
            },
            NavTri {
                idx: 1,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(10.0, 0.0, 10.0),
                neighbors: vec![],
            },
            NavTri {
                idx: 2,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(5.0, 0.0, 5.0),
                neighbors: vec![],
            },
        ];
        // Query point closest to tri 0
        let result = closest_tri(&tris, Vec3::new(0.4, 0.0, 0.4));
        assert_eq!(result, Some(0));

        // Query point closest to tri 1
        let result = closest_tri(&tris, Vec3::new(10.1, 0.0, 10.1));
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_astar_tri_same_start_goal() {
        let tris = vec![NavTri {
            idx: 0,
            verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
            normal: Vec3::Y,
            center: Vec3::new(0.5, 0.0, 0.5),
            neighbors: vec![],
        }];
        let path = astar_tri(&tris, 0, 0);
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn test_astar_tri_simple_path() {
        let tris = vec![
            NavTri {
                idx: 0,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(0.5, 0.0, 0.5),
                neighbors: vec![1],
            },
            NavTri {
                idx: 1,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(1.5, 0.0, 0.5),
                neighbors: vec![0, 2],
            },
            NavTri {
                idx: 2,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(2.5, 0.0, 0.5),
                neighbors: vec![1],
            },
        ];
        let path = astar_tri(&tris, 0, 2);
        assert_eq!(path, vec![0, 1, 2]);
    }

    #[test]
    fn test_astar_tri_no_path() {
        let tris = vec![
            NavTri {
                idx: 0,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(0.5, 0.0, 0.5),
                neighbors: vec![],
            },
            NavTri {
                idx: 1,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(10.5, 0.0, 0.5),
                neighbors: vec![],
            },
        ];
        let path = astar_tri(&tris, 0, 1);
        assert_eq!(path.len(), 0); // No connection
    }

    #[test]
    fn test_astar_tri_branching_path() {
        // Diamond graph: 0 → [1, 2] → 3
        let tris = vec![
            NavTri {
                idx: 0,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(0.0, 0.0, 0.0),
                neighbors: vec![1, 2],
            },
            NavTri {
                idx: 1,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(1.0, 0.0, 0.0),
                neighbors: vec![0, 3],
            },
            NavTri {
                idx: 2,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(1.0, 0.0, 1.0),
                neighbors: vec![0, 3],
            },
            NavTri {
                idx: 3,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(2.0, 0.0, 0.5),
                neighbors: vec![1, 2],
            },
        ];
        let path = astar_tri(&tris, 0, 3);
        assert!(path.len() == 3); // 0 → (1 or 2) → 3
        assert_eq!(path[0], 0);
        assert_eq!(path[2], 3);
        assert!(path[1] == 1 || path[1] == 2); // Either route valid
    }

    // ===== Smoothing Tests =====

    #[test]
    fn test_smooth_empty() {
        let mut pts = vec![];
        smooth(&mut pts, &[]);
        assert_eq!(pts.len(), 0);
    }

    #[test]
    fn test_smooth_two_points() {
        let mut pts = vec![Vec3::ZERO, Vec3::ONE];
        smooth(&mut pts, &[]);
        assert_eq!(pts.len(), 2);
        // Endpoints unchanged
        assert_eq!(pts[0], Vec3::ZERO);
        assert_eq!(pts[1], Vec3::ONE);
    }

    #[test]
    fn test_smooth_three_points() {
        let mut pts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
        ];
        smooth(&mut pts, &[]);

        // Endpoints unchanged
        assert_eq!(pts[0], Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(pts[2], Vec3::new(10.0, 0.0, 0.0));

        // Middle point smoothed (weighted average with neighbors)
        // Formula: pts[1] = 0.25 * pts[0] + 0.5 * pts[1] + 0.25 * pts[2]
        // After 2 iterations of smoothing, middle point moves toward line
        // But with weight 0.5 on current position, it stays close to 5.0
        assert!((pts[1].x - 5.0).abs() < 1.0); // Relaxed tolerance
    }

    // ===== Integration Tests =====

    #[test]
    fn test_full_pipeline_bake_and_path() {
        // Create a simple 2x2 square navmesh (2 triangles sharing an edge)
        let tris = vec![
            Triangle {
                a: Vec3::new(0.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 2.0),
                c: Vec3::new(2.0, 0.0, 0.0),
            },
            Triangle {
                a: Vec3::new(2.0, 0.0, 0.0),
                b: Vec3::new(0.0, 0.0, 2.0),
                c: Vec3::new(2.0, 0.0, 2.0),
            },
        ];

        let nav = NavMesh::bake(&tris, 0.5, 70.0);
        assert_eq!(nav.tris.len(), 2);
        assert!(nav.tris[0].neighbors.len() > 0 || nav.tris[1].neighbors.len() > 0); // Connected

        // Path across the square
        let path = nav.find_path(Vec3::new(0.5, 0.0, 0.5), Vec3::new(1.5, 0.0, 1.5));
        assert!(path.len() >= 2);
        assert!((path[0] - Vec3::new(0.5, 0.0, 0.5)).length() < 0.2);
        assert!((path.last().unwrap() - Vec3::new(1.5, 0.0, 1.5)).length() < 0.2);
    }

    #[test]
    fn test_navmesh_with_max_step_parameter() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        }];
        let nav = NavMesh::bake(&tris, 0.8, 60.0);
        assert_eq!(nav.max_step, 0.8);
    }

    #[test]
    fn test_navmesh_with_max_slope_parameter() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(1.0, 0.0, 0.0),
            c: Vec3::new(0.0, 0.0, 1.0),
        }];
        let nav = NavMesh::bake(&tris, 0.4, 30.0);
        assert_eq!(nav.max_slope_deg, 30.0);
    }
}

// Week 2 Day 2: Stress Tests
#[cfg(test)]
#[path = "stress_tests.rs"]
mod stress_tests;

// Week 2 Day 3: Edge Case Tests
#[cfg(test)]
#[path = "edge_case_tests.rs"]
mod edge_case_tests;
