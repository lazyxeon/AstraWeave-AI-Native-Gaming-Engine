use astraweave_nav::NavMesh;
use glam::Vec3;

#[derive(Clone, Debug)]
pub struct Portal {
    pub a: Vec3,
    pub b: Vec3,
    pub left_tri: usize,
    pub right_tri: usize,
}

#[derive(Clone, Debug)]
pub struct PortalGraph {
    pub portals: Vec<Portal>,
    pub tri_to_portals: Vec<Vec<usize>>,
}

pub fn build_portals(nav: &NavMesh) -> PortalGraph {
    let mut portals = vec![];
    let mut tri_to_portals = vec![vec![]; nav.tris.len()];
    let eps = 1e-3;

    for (i, t) in nav.tris.iter().enumerate() {
        for &j in &t.neighbors {
            if j < i {
                continue;
            }
            let a = shared_edge(&t.verts, &nav.tris[j].verts, eps);
            if let Some((p0, p1)) = a {
                let pid = portals.len();
                portals.push(Portal {
                    a: p0,
                    b: p1,
                    left_tri: i,
                    right_tri: j,
                });
                tri_to_portals[i].push(pid);
                tri_to_portals[j].push(pid);
            }
        }
    }
    PortalGraph {
        portals,
        tri_to_portals,
    }
}

fn shared_edge(a: &[Vec3; 3], b: &[Vec3; 3], eps: f32) -> Option<(Vec3, Vec3)> {
    let mut shared = vec![];
    for va in a {
        for vb in b {
            if va.distance(*vb) <= eps {
                shared.push(*va)
            }
        }
    }
    if shared.len() >= 2 {
        Some((shared[0], shared[1]))
    } else {
        None
    }
}

/// Funnel / string‑pull through portals from start to goal.
/// Path indexed by triangles. Returns refined waypoints.
pub fn string_pull(
    _nav: &NavMesh,
    pg: &PortalGraph,
    tri_path: &[usize],
    start: Vec3,
    goal: Vec3,
) -> Vec<Vec3> {
    if tri_path.len() < 2 {
        return vec![start, goal];
    }
    let mut way = vec![start];
    let mut apex = start;
    let mut left = start;
    let mut right = start;
    let mut _left_idx = 0usize;
    let mut right_idx = 0usize;

    // Build ordered portal edges between triangles on path
    let mut edges: Vec<(Vec3, Vec3)> = vec![];
    for w in tri_path.windows(2) {
        let t0 = w[0];
        let t1 = w[1];
        // find portal shared by t0/t1
        if let Some(pid) = pg.tri_to_portals[t0].iter().find(|pid| {
            let p = &pg.portals[**pid];
            (p.left_tri == t0 && p.right_tri == t1) || (p.left_tri == t1 && p.right_tri == t0)
        }) {
            let p = &pg.portals[*pid];
            edges.push((p.a, p.b));
        }
    }
    // add final goal as zero‑width portal
    edges.push((goal, goal));

    let mut i = 0;
    while i < edges.len() {
        let (new_left, new_right) = edges[i];

        // try tighten left edge
        if triangle_area2(apex, left, new_left) >= 0.0 {
            left = new_left;
            _left_idx = i;
        }
        // try tighten right edge
        if triangle_area2(apex, right, new_right) <= 0.0 {
            right = new_right;
            right_idx = i;
        }

        // check crossing
        if triangle_area2(apex, left, right) < 0.0 {
            // advance apex to right
            way.push(right);
            apex = right;
            i = right_idx + 1;
            left = apex;
            right = apex;
            continue;
        }
        i += 1;
    }
    way.push(goal);
    way
}

fn triangle_area2(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let ab = b - a;
    let ac = c - a;
    ab.x * ac.z - ab.z * ac.x // 2D area on XZ plane
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_nav::NavTri;

    fn create_simple_navmesh() -> NavMesh {
        // Create 2 adjacent triangles for testing
        NavMesh {
            tris: vec![
                NavTri {
                    idx: 0,
                    verts: [
                        Vec3::new(0.0, 0.0, 0.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(0.5, 0.0, 1.0),
                    ],
                    normal: Vec3::Y,
                    center: Vec3::new(0.5, 0.0, 0.333),
                    neighbors: vec![1],
                },
                NavTri {
                    idx: 1,
                    verts: [
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec3::new(1.5, 0.0, 1.0),
                        Vec3::new(0.5, 0.0, 1.0),
                    ],
                    normal: Vec3::Y,
                    center: Vec3::new(1.0, 0.0, 0.666),
                    neighbors: vec![0],
                },
            ],
            max_step: 0.5,
            max_slope_deg: 45.0,
        }
    }

    #[test]
    fn test_shared_edge_finds_common_vertices() {
        let a = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.5, 0.0, 1.0),
        ];
        let b = [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.5, 0.0, 1.0),
            Vec3::new(0.5, 0.0, 1.0),
        ];

        let edge = shared_edge(&a, &b, 1e-3);
        assert!(edge.is_some());
        let (p0, p1) = edge.unwrap();
        
        // Should find the two shared vertices
        let shared_verts = vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.5, 0.0, 1.0)];
        assert!(shared_verts.contains(&p0) || shared_verts.contains(&p1));
    }

    #[test]
    fn test_shared_edge_none_for_disjoint_triangles() {
        let a = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.5, 0.0, 1.0),
        ];
        let b = [
            Vec3::new(5.0, 0.0, 5.0),
            Vec3::new(6.0, 0.0, 5.0),
            Vec3::new(5.5, 0.0, 6.0),
        ];

        let edge = shared_edge(&a, &b, 1e-3);
        assert!(edge.is_none());
    }

    #[test]
    fn test_build_portals_creates_portal_between_neighbors() {
        let nav = create_simple_navmesh();
        let pg = build_portals(&nav);

        // Should create 1 portal between triangles 0 and 1
        assert_eq!(pg.portals.len(), 1);
        assert_eq!(pg.portals[0].left_tri, 0);
        assert_eq!(pg.portals[0].right_tri, 1);
    }

    #[test]
    fn test_build_portals_tri_to_portals_mapping() {
        let nav = create_simple_navmesh();
        let pg = build_portals(&nav);

        // Both triangles should reference portal 0
        assert_eq!(pg.tri_to_portals.len(), 2);
        assert_eq!(pg.tri_to_portals[0], vec![0]);
        assert_eq!(pg.tri_to_portals[1], vec![0]);
    }

    #[test]
    fn test_build_portals_empty_navmesh() {
        let nav = NavMesh {
            tris: vec![],
            max_step: 0.5,
            max_slope_deg: 45.0,
        };
        let pg = build_portals(&nav);

        assert_eq!(pg.portals.len(), 0);
        assert_eq!(pg.tri_to_portals.len(), 0);
    }

    #[test]
    fn test_build_portals_single_triangle_no_portals() {
        let nav = NavMesh {
            tris: vec![NavTri {
                idx: 0,
                verts: [Vec3::ZERO, Vec3::X, Vec3::Z],
                normal: Vec3::Y,
                center: Vec3::new(0.333, 0.0, 0.333),
                neighbors: vec![],
            }],
            max_step: 0.5,
            max_slope_deg: 45.0,
        };
        let pg = build_portals(&nav);

        assert_eq!(pg.portals.len(), 0);
        assert_eq!(pg.tri_to_portals.len(), 1);
        assert_eq!(pg.tri_to_portals[0].len(), 0);
    }

    #[test]
    fn test_triangle_area2_positive() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        let c = Vec3::new(0.0, 0.0, 1.0);

        let area = triangle_area2(a, b, c);
        assert!(area > 0.0); // Counter-clockwise on XZ plane
    }

    #[test]
    fn test_triangle_area2_negative() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(0.0, 0.0, 1.0);
        let c = Vec3::new(1.0, 0.0, 0.0);

        let area = triangle_area2(a, b, c);
        assert!(area < 0.0); // Clockwise on XZ plane
    }

    #[test]
    fn test_triangle_area2_collinear() {
        let a = Vec3::new(0.0, 0.0, 0.0);
        let b = Vec3::new(1.0, 0.0, 0.0);
        let c = Vec3::new(2.0, 0.0, 0.0);

        let area = triangle_area2(a, b, c);
        assert_eq!(area, 0.0); // Collinear points
    }

    #[test]
    fn test_string_pull_short_path() {
        let nav = create_simple_navmesh();
        let pg = build_portals(&nav);

        let start = Vec3::new(0.2, 0.0, 0.2);
        let goal = Vec3::new(1.2, 0.0, 0.8);
        let tri_path = vec![0, 1];

        let waypoints = string_pull(&nav, &pg, &tri_path, start, goal);

        // Should return at least start and goal
        assert!(waypoints.len() >= 2);
        assert_eq!(waypoints[0], start);
        assert_eq!(*waypoints.last().unwrap(), goal);
    }

    #[test]
    fn test_string_pull_single_triangle() {
        let nav = create_simple_navmesh();
        let pg = build_portals(&nav);

        let start = Vec3::new(0.2, 0.0, 0.2);
        let goal = Vec3::new(0.4, 0.0, 0.4);
        let tri_path = vec![0];

        let waypoints = string_pull(&nav, &pg, &tri_path, start, goal);

        // Single triangle path should just return start/goal
        assert_eq!(waypoints.len(), 2);
        assert_eq!(waypoints[0], start);
        assert_eq!(waypoints[1], goal);
    }

    #[test]
    fn test_string_pull_empty_path() {
        let nav = create_simple_navmesh();
        let pg = build_portals(&nav);

        let start = Vec3::ZERO;
        let goal = Vec3::X;
        let tri_path: Vec<usize> = vec![];

        let waypoints = string_pull(&nav, &pg, &tri_path, start, goal);

        // Empty path should return start/goal
        assert_eq!(waypoints.len(), 2);
        assert_eq!(waypoints[0], start);
        assert_eq!(waypoints[1], goal);
    }

    #[test]
    fn test_portal_graph_multiple_triangles() {
        // Create 3 triangles in a row
        let nav = NavMesh {
            tris: vec![
                NavTri {
                    idx: 0,
                    verts: [Vec3::ZERO, Vec3::X, Vec3::new(0.5, 0.0, 1.0)],
                    normal: Vec3::Y,
                    center: Vec3::new(0.5, 0.0, 0.333),
                    neighbors: vec![1],
                },
                NavTri {
                    idx: 1,
                    verts: [Vec3::X, Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.5, 0.0, 1.0)],
                    normal: Vec3::Y,
                    center: Vec3::new(1.167, 0.0, 0.333),
                    neighbors: vec![0, 2],
                },
                NavTri {
                    idx: 2,
                    verts: [Vec3::new(2.0, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0), Vec3::new(0.5, 0.0, 1.0)],
                    normal: Vec3::Y,
                    center: Vec3::new(1.833, 0.0, 0.333),
                    neighbors: vec![1],
                },
            ],
            max_step: 0.5,
            max_slope_deg: 45.0,
        };
        let pg = build_portals(&nav);

        // Should create 2 portals (0-1 and 1-2)
        assert_eq!(pg.portals.len(), 2);
        
        // Middle triangle should have 2 portals
        assert_eq!(pg.tri_to_portals[1].len(), 2);
        
        // Edge triangles should have 1 portal each
        assert_eq!(pg.tri_to_portals[0].len(), 1);
        assert_eq!(pg.tri_to_portals[2].len(), 1);
    }
}


