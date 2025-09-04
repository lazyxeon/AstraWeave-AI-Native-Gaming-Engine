use glam::Vec3;
use astraweave_nav::{NavMesh, Triangle};

#[derive(Clone, Debug)]
pub struct Portal { pub a: Vec3, pub b: Vec3, pub left_tri: usize, pub right_tri: usize }

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
            if j < i { continue; }
            let a = shared_edge(&t.verts, &nav.tris[j].verts, eps);
            if let Some((p0, p1)) = a {
                let pid = portals.len();
                portals.push(Portal { a: p0, b: p1, left_tri: i, right_tri: j });
                tri_to_portals[i].push(pid);
                tri_to_portals[j].push(pid);
            }
        }
    }
    PortalGraph { portals, tri_to_portals }
}

fn shared_edge(a: &[Vec3;3], b: &[Vec3;3], eps: f32) -> Option<(Vec3,Vec3)> {
    let mut shared = vec![];
    for va in a { for vb in b { if va.distance(*vb) <= eps { shared.push(*va) } } }
    if shared.len() >= 2 { Some((shared[0], shared[1])) } else { None }
}

/// Funnel / string‑pull through portals from start to goal.
/// Path indexed by triangles. Returns refined waypoints.
pub fn string_pull(nav: &NavMesh, pg: &PortalGraph, tri_path: &[usize], start: Vec3, goal: Vec3) -> Vec<Vec3> {
    if tri_path.len() < 2 { return vec![start, goal]; }
    let mut way = vec![start];
    let mut apex = start;
    let mut left = start;
    let mut right = start;
    let mut left_idx = 0usize;
    let mut right_idx = 0usize;

    // Build ordered portal edges between triangles on path
    let mut edges: Vec<(Vec3,Vec3)> = vec![];
    for w in tri_path.windows(2) {
        let t0 = w[0]; let t1 = w[1];
        // find portal shared by t0/t1
        if let Some(pid) = pg.tri_to_portals[t0].iter().find(|pid| {
            let p = &pg.portals[***pid];
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
            left_idx = i;
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
            left = apex; right = apex;
            continue;
        }
        i += 1;
    }
    way.push(goal);
    way
}

fn triangle_area2(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let ab = b - a; let ac = c - a;
    (ab.x * ac.z - ab.z * ac.x) // 2D area on XZ plane
}
