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
                let slope_ok = n.dot(Vec3::Y).acos().to_degrees() <= max_slope_deg;
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
        let (s, g) = (s.unwrap(), g.unwrap());
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

fn smooth(pts: &mut Vec<Vec3>, _tris: &[NavTri]) {
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
