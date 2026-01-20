use glam::Vec3;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Triangle {
    /// Creates a new triangle from three vertices.
    #[must_use]
    pub fn new(a: Vec3, b: Vec3, c: Vec3) -> Self {
        Self { a, b, c }
    }

    /// Calculates the center (centroid) of the triangle.
    #[must_use]
    pub fn center(&self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }

    /// Calculates the normal vector of the triangle (not normalized).
    #[must_use]
    pub fn normal(&self) -> Vec3 {
        (self.b - self.a).cross(self.c - self.a)
    }

    /// Calculates the normalized normal vector of the triangle.
    #[must_use]
    pub fn normal_normalized(&self) -> Vec3 {
        self.normal().normalize_or_zero()
    }

    /// Calculates the area of the triangle.
    #[must_use]
    pub fn area(&self) -> f32 {
        self.normal().length() * 0.5
    }

    /// Returns true if the triangle is degenerate (has zero or near-zero area).
    #[must_use]
    pub fn is_degenerate(&self) -> bool {
        self.area() < 1e-6
    }

    /// Calculates the perimeter of the triangle.
    #[must_use]
    pub fn perimeter(&self) -> f32 {
        self.a.distance(self.b) + self.b.distance(self.c) + self.c.distance(self.a)
    }

    /// Returns the edge lengths as an array [ab, bc, ca].
    #[must_use]
    pub fn edge_lengths(&self) -> [f32; 3] {
        [
            self.a.distance(self.b),
            self.b.distance(self.c),
            self.c.distance(self.a),
        ]
    }

    /// Returns the shortest edge length.
    #[must_use]
    pub fn min_edge_length(&self) -> f32 {
        let edges = self.edge_lengths();
        edges[0].min(edges[1]).min(edges[2])
    }

    /// Returns the longest edge length.
    #[must_use]
    pub fn max_edge_length(&self) -> f32 {
        let edges = self.edge_lengths();
        edges[0].max(edges[1]).max(edges[2])
    }

    /// Returns the vertices as an array.
    #[must_use]
    pub fn vertices(&self) -> [Vec3; 3] {
        [self.a, self.b, self.c]
    }

    /// Creates a triangle from a vertex array.
    #[must_use]
    pub fn from_vertices(vertices: [Vec3; 3]) -> Self {
        Self {
            a: vertices[0],
            b: vertices[1],
            c: vertices[2],
        }
    }
}

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Triangle({:.2}, {:.2}, {:.2})-({:.2}, {:.2}, {:.2})-({:.2}, {:.2}, {:.2})",
            self.a.x, self.a.y, self.a.z,
            self.b.x, self.b.y, self.b.z,
            self.c.x, self.c.y, self.c.z
        )
    }
}

#[derive(Clone, Debug)]
pub struct NavTri {
    pub idx: usize,
    pub verts: [Vec3; 3],
    pub normal: Vec3,
    pub center: Vec3,
    pub neighbors: Vec<usize>,
}

impl NavTri {
    /// Creates a new navigation triangle.
    #[must_use]
    pub fn new(idx: usize, verts: [Vec3; 3], normal: Vec3, center: Vec3) -> Self {
        Self {
            idx,
            verts,
            normal,
            center,
            neighbors: Vec::new(),
        }
    }

    /// Returns the number of neighbors.
    #[must_use]
    pub fn neighbor_count(&self) -> usize {
        self.neighbors.len()
    }

    /// Returns true if this triangle has the specified neighbor.
    #[must_use]
    pub fn has_neighbor(&self, idx: usize) -> bool {
        self.neighbors.contains(&idx)
    }

    /// Returns true if this triangle has no neighbors.
    #[must_use]
    pub fn is_isolated(&self) -> bool {
        self.neighbors.is_empty()
    }

    /// Returns true if this is an edge triangle (has fewer than 3 neighbors).
    #[must_use]
    pub fn is_edge(&self) -> bool {
        self.neighbors.len() < 3
    }

    /// Returns the area of the triangle.
    #[must_use]
    pub fn area(&self) -> f32 {
        let a = self.verts[0];
        let b = self.verts[1];
        let c = self.verts[2];
        (b - a).cross(c - a).length() * 0.5
    }

    /// Returns the perimeter of the triangle.
    #[must_use]
    pub fn perimeter(&self) -> f32 {
        let a = self.verts[0];
        let b = self.verts[1];
        let c = self.verts[2];
        a.distance(b) + b.distance(c) + c.distance(a)
    }

    /// Returns the distance from this triangle's center to another point.
    #[must_use]
    pub fn distance_to(&self, point: Vec3) -> f32 {
        self.center.distance(point)
    }

    /// Returns the squared distance from this triangle's center to another point.
    #[must_use]
    pub fn distance_squared_to(&self, point: Vec3) -> f32 {
        self.center.distance_squared(point)
    }

    /// Returns the slope angle in degrees (angle from vertical Y axis).
    #[must_use]
    pub fn slope_degrees(&self) -> f32 {
        let dot = self.normal.dot(Vec3::Y).clamp(-1.0, 1.0);
        dot.acos().to_degrees()
    }

    /// Returns true if the triangle is walkable (normal points upward).
    #[must_use]
    pub fn is_walkable(&self) -> bool {
        self.normal.dot(Vec3::Y) > 0.0
    }
}

impl fmt::Display for NavTri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "NavTri[{}] center=({:.2}, {:.2}, {:.2}), {} neighbors",
            self.idx, self.center.x, self.center.y, self.center.z, self.neighbor_count()
        )
    }
}

/// Axis-aligned bounding box for region invalidation
#[derive(Clone, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// Create a new AABB
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Creates an AABB at the origin with zero size.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ZERO,
        }
    }

    /// Creates a unit AABB from (0,0,0) to (1,1,1).
    #[must_use]
    pub fn unit() -> Self {
        Self {
            min: Vec3::ZERO,
            max: Vec3::ONE,
        }
    }

    /// Creates an AABB centered at a point with given half-extents.
    #[must_use]
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Check if this AABB contains a point
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Check if this AABB intersects with another
    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Merge with another AABB (returns bounding box of both)
    pub fn merge(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Create an AABB from a triangle
    pub fn from_triangle(tri: &Triangle) -> Self {
        let min = Vec3::new(
            tri.a.x.min(tri.b.x).min(tri.c.x),
            tri.a.y.min(tri.b.y).min(tri.c.y),
            tri.a.z.min(tri.b.z).min(tri.c.z),
        );
        let max = Vec3::new(
            tri.a.x.max(tri.b.x).max(tri.c.x),
            tri.a.y.max(tri.b.y).max(tri.c.y),
            tri.a.z.max(tri.b.z).max(tri.c.z),
        );
        Aabb { min, max }
    }

    /// Returns the center point of the AABB.
    #[must_use]
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the size (extent) of the AABB.
    #[must_use]
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Returns the half-extents of the AABB.
    #[must_use]
    pub fn half_extents(&self) -> Vec3 {
        self.size() * 0.5
    }

    /// Returns the volume of the AABB.
    #[must_use]
    pub fn volume(&self) -> f32 {
        let s = self.size();
        s.x * s.y * s.z
    }

    /// Returns the surface area of the AABB.
    #[must_use]
    pub fn surface_area(&self) -> f32 {
        let s = self.size();
        2.0 * (s.x * s.y + s.y * s.z + s.z * s.x)
    }

    /// Returns true if the AABB has zero volume.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y || self.min.z >= self.max.z
    }

    /// Returns the longest axis dimension.
    #[must_use]
    pub fn longest_axis(&self) -> f32 {
        let s = self.size();
        s.x.max(s.y).max(s.z)
    }

    /// Returns the shortest axis dimension.
    #[must_use]
    pub fn shortest_axis(&self) -> f32 {
        let s = self.size();
        s.x.min(s.y).min(s.z)
    }

    /// Expands the AABB by the given amount on all sides.
    #[must_use]
    pub fn expand(&self, amount: f32) -> Self {
        Self {
            min: self.min - Vec3::splat(amount),
            max: self.max + Vec3::splat(amount),
        }
    }

    /// Returns the distance from the AABB center to a point.
    #[must_use]
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.center().distance(point)
    }
}

impl fmt::Display for Aabb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AABB[({:.2}, {:.2}, {:.2}) - ({:.2}, {:.2}, {:.2})]",
            self.min.x, self.min.y, self.min.z,
            self.max.x, self.max.y, self.max.z
        )
    }
}

#[derive(Clone, Debug)]
pub struct NavMesh {
    pub tris: Vec<NavTri>,
    pub max_step: f32,
    pub max_slope_deg: f32,
    /// Dirty regions that need rebaking (Phase 10: terrain invalidation)
    dirty_regions: Vec<Aabb>,
    /// Total rebakes performed
    rebake_count: u32,
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
            dirty_regions: Vec::new(),
            rebake_count: 0,
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

    // ========================================================================
    // REGION INVALIDATION - Phase 10: AI-Orchestrated Dynamic Terrain
    // ========================================================================

    /// Mark a region as dirty (needs rebaking)
    ///
    /// Triangles that intersect with this AABB will be marked for rebake.
    pub fn invalidate_region(&mut self, region: Aabb) {
        // Merge overlapping regions to reduce rebake calls
        let mut merged = false;
        for existing in self.dirty_regions.iter_mut() {
            if existing.intersects(&region) {
                *existing = existing.merge(&region);
                merged = true;
                break;
            }
        }
        if !merged {
            self.dirty_regions.push(region);
        }
    }

    /// Check if the NavMesh needs rebaking
    pub fn needs_rebake(&self) -> bool {
        !self.dirty_regions.is_empty()
    }

    /// Get the number of dirty regions
    pub fn dirty_region_count(&self) -> usize {
        self.dirty_regions.len()
    }

    /// Get dirty regions (for debugging/visualization)
    pub fn dirty_regions(&self) -> &[Aabb] {
        &self.dirty_regions
    }

    /// Clear all dirty regions without rebaking
    pub fn clear_dirty_regions(&mut self) {
        self.dirty_regions.clear();
    }

    /// Rebake dirty regions with new triangle data
    ///
    /// This performs a full rebake of the entire mesh using the provided triangles.
    /// For large meshes, consider using partial_rebake for better performance.
    pub fn rebake_dirty_regions(&mut self, tris: &[Triangle]) {
        if self.dirty_regions.is_empty() {
            return;
        }

        // Full rebake (simplest implementation)
        let rebaked = NavMesh::bake(tris, self.max_step, self.max_slope_deg);
        self.tris = rebaked.tris;
        self.dirty_regions.clear();
        self.rebake_count += 1;
    }

    /// Partial rebake - only processes triangles within dirty regions
    ///
    /// More efficient for large meshes with small changes.
    /// Returns the number of triangles that were updated.
    pub fn partial_rebake(&mut self, all_tris: &[Triangle]) -> usize {
        if self.dirty_regions.is_empty() {
            return 0;
        }

        // Find triangles that intersect with dirty regions
        let mut affected_count = 0;
        for tri in all_tris {
            let tri_aabb = Aabb::from_triangle(tri);
            for region in &self.dirty_regions {
                if tri_aabb.intersects(region) {
                    affected_count += 1;
                    break;
                }
            }
        }

        // For now, do a full rebake if any triangles are affected
        // In a production system, you would only rebuild the affected portions
        if affected_count > 0 {
            self.rebake_dirty_regions(all_tris);
        }

        affected_count
    }

    /// Get the total number of rebakes performed
    pub fn rebake_count(&self) -> u32 {
        self.rebake_count
    }

    /// Check if a path crosses any dirty regions
    ///
    /// Useful to determine if a path might be invalid due to terrain changes.
    pub fn path_crosses_dirty_region(&self, path: &[Vec3]) -> bool {
        if self.dirty_regions.is_empty() || path.is_empty() {
            return false;
        }

        for point in path {
            for region in &self.dirty_regions {
                if region.contains(*point) {
                    return true;
                }
            }
        }
        false
    }

    /// Returns the number of triangles in the NavMesh.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.tris.len()
    }

    /// Returns true if the NavMesh is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tris.is_empty()
    }

    /// Returns the total number of edges (neighbor connections).
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.tris.iter().map(|t| t.neighbors.len()).sum::<usize>() / 2
    }

    /// Returns the average number of neighbors per triangle.
    #[must_use]
    pub fn average_neighbor_count(&self) -> f32 {
        if self.tris.is_empty() {
            return 0.0;
        }
        let total: usize = self.tris.iter().map(|t| t.neighbors.len()).sum();
        total as f32 / self.tris.len() as f32
    }

    /// Returns the number of isolated triangles (no neighbors).
    #[must_use]
    pub fn isolated_count(&self) -> usize {
        self.tris.iter().filter(|t| t.neighbors.is_empty()).count()
    }

    /// Returns the total surface area of all triangles.
    #[must_use]
    pub fn total_area(&self) -> f32 {
        self.tris.iter().map(|t| t.area()).sum()
    }

    /// Returns the bounding box of the entire NavMesh.
    #[must_use]
    pub fn bounds(&self) -> Option<Aabb> {
        if self.tris.is_empty() {
            return None;
        }

        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);

        for tri in &self.tris {
            for v in &tri.verts {
                min = min.min(*v);
                max = max.max(*v);
            }
        }

        Some(Aabb::new(min, max))
    }

    /// Gets a triangle by index.
    #[must_use]
    pub fn get_triangle(&self, idx: usize) -> Option<&NavTri> {
        self.tris.get(idx)
    }

    /// Returns a brief summary of the NavMesh.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "NavMesh: {} triangles, {} edges, max_step={:.2}, max_slope={:.1}°",
            self.triangle_count(),
            self.edge_count(),
            self.max_step,
            self.max_slope_deg
        )
    }
}

impl fmt::Display for NavMesh {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
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
        assert!(!nav.tris[0].neighbors.is_empty() || !nav.tris[1].neighbors.is_empty()); // Connected

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

    // ===== AABB Tests =====

    #[test]
    fn test_aabb_new() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_aabb_contains() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::splat(10.0));
        assert!(aabb.contains(Vec3::splat(5.0)));
        assert!(!aabb.contains(Vec3::splat(15.0)));
        assert!(aabb.contains(Vec3::ZERO)); // Edge case: boundary
    }

    #[test]
    fn test_aabb_intersects() {
        let aabb1 = Aabb::new(Vec3::ZERO, Vec3::splat(10.0));
        let aabb2 = Aabb::new(Vec3::splat(5.0), Vec3::splat(15.0));
        let aabb3 = Aabb::new(Vec3::splat(20.0), Vec3::splat(30.0));

        assert!(aabb1.intersects(&aabb2));
        assert!(!aabb1.intersects(&aabb3));
    }

    #[test]
    fn test_aabb_merge() {
        let aabb1 = Aabb::new(Vec3::ZERO, Vec3::splat(5.0));
        let aabb2 = Aabb::new(Vec3::splat(3.0), Vec3::splat(10.0));
        let merged = aabb1.merge(&aabb2);

        assert_eq!(merged.min, Vec3::ZERO);
        assert_eq!(merged.max, Vec3::splat(10.0));
    }

    #[test]
    fn test_aabb_from_triangle() {
        let tri = Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(5.0, 2.0, 1.0),
            c: Vec3::new(3.0, 4.0, 6.0),
        };
        let aabb = Aabb::from_triangle(&tri);

        assert_eq!(aabb.min, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(aabb.max, Vec3::new(5.0, 4.0, 6.0));
    }

    // ===== Region Invalidation Tests =====

    #[test]
    fn test_navmesh_invalidate_region() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        }];
        let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

        assert!(!nav.needs_rebake());
        assert_eq!(nav.dirty_region_count(), 0);

        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::splat(5.0)));

        assert!(nav.needs_rebake());
        assert_eq!(nav.dirty_region_count(), 1);
    }

    #[test]
    fn test_navmesh_invalidate_region_merge() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        }];
        let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

        // Add overlapping regions - should merge
        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::splat(5.0)));
        nav.invalidate_region(Aabb::new(Vec3::splat(3.0), Vec3::splat(8.0)));

        // Should have merged into 1 region
        assert_eq!(nav.dirty_region_count(), 1);

        // Add non-overlapping region
        nav.invalidate_region(Aabb::new(Vec3::splat(20.0), Vec3::splat(25.0)));
        assert_eq!(nav.dirty_region_count(), 2);
    }

    #[test]
    fn test_navmesh_rebake_dirty_regions() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        }];
        let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::splat(5.0)));
        assert!(nav.needs_rebake());

        nav.rebake_dirty_regions(&tris);

        assert!(!nav.needs_rebake());
        assert_eq!(nav.rebake_count(), 1);
    }

    #[test]
    fn test_navmesh_path_crosses_dirty_region() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        }];
        let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

        let path = vec![Vec3::ZERO, Vec3::new(0.5, 0.0, 0.5), Vec3::ONE];

        // No dirty regions
        assert!(!nav.path_crosses_dirty_region(&path));

        // Add dirty region that path crosses
        nav.invalidate_region(Aabb::new(
            Vec3::new(0.4, -1.0, 0.4),
            Vec3::new(0.6, 1.0, 0.6),
        ));
        assert!(nav.path_crosses_dirty_region(&path));
    }

    #[test]
    fn test_navmesh_clear_dirty_regions() {
        let tris = vec![Triangle {
            a: Vec3::new(0.0, 0.0, 0.0),
            b: Vec3::new(0.0, 0.0, 1.0),
            c: Vec3::new(1.0, 0.0, 0.0),
        }];
        let mut nav = NavMesh::bake(&tris, 0.4, 60.0);

        nav.invalidate_region(Aabb::new(Vec3::ZERO, Vec3::splat(5.0)));
        assert!(nav.needs_rebake());

        nav.clear_dirty_regions();
        assert!(!nav.needs_rebake());
        assert_eq!(nav.rebake_count(), 0); // Not rebaked, just cleared
    }

    // ===== Triangle Helper Tests =====

    #[test]
    fn test_triangle_new() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Z);
        assert_eq!(tri.a, Vec3::ZERO);
        assert_eq!(tri.b, Vec3::X);
        assert_eq!(tri.c, Vec3::Z);
    }

    #[test]
    fn test_triangle_center() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::new(3.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 3.0));
        let center = tri.center();
        assert!((center.x - 1.0).abs() < 1e-5);
        assert!((center.z - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_normal() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Z);
        let normal = tri.normal_normalized();
        assert!((normal.y - (-1.0)).abs() < 1e-5 || (normal.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_area() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 2.0));
        assert!((tri.area() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_is_degenerate() {
        let degenerate = Triangle::new(Vec3::ZERO, Vec3::ZERO, Vec3::ZERO);
        assert!(degenerate.is_degenerate());

        let valid = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Z);
        assert!(!valid.is_degenerate());
    }

    #[test]
    fn test_triangle_perimeter() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Z);
        let perimeter = tri.perimeter();
        assert!((perimeter - (1.0 + 1.0 + 2.0_f32.sqrt())).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_edge_lengths() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Z);
        let edges = tri.edge_lengths();
        assert!((edges[0] - 1.0).abs() < 1e-5); // a to b
        assert!((edges[1] - 2.0_f32.sqrt()).abs() < 1e-5); // b to c
        assert!((edges[2] - 1.0).abs() < 1e-5); // c to a
    }

    #[test]
    fn test_triangle_min_max_edge() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Z);
        assert!((tri.min_edge_length() - 1.0).abs() < 1e-5);
        assert!((tri.max_edge_length() - 2.0_f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn test_triangle_vertices() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Z);
        let verts = tri.vertices();
        assert_eq!(verts[0], Vec3::ZERO);
        assert_eq!(verts[1], Vec3::X);
        assert_eq!(verts[2], Vec3::Z);
    }

    #[test]
    fn test_triangle_from_vertices() {
        let tri = Triangle::from_vertices([Vec3::ZERO, Vec3::X, Vec3::Z]);
        assert_eq!(tri.a, Vec3::ZERO);
        assert_eq!(tri.b, Vec3::X);
        assert_eq!(tri.c, Vec3::Z);
    }

    #[test]
    fn test_triangle_display() {
        let tri = Triangle::new(Vec3::ZERO, Vec3::X, Vec3::Z);
        let display = format!("{}", tri);
        assert!(display.contains("Triangle"));
    }

    // ===== NavTri Helper Tests =====

    #[test]
    fn test_navtri_new() {
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::new(0.33, 0.0, 0.33));
        assert_eq!(tri.idx, 0);
        assert!(tri.neighbors.is_empty());
    }

    #[test]
    fn test_navtri_neighbor_count() {
        let mut tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        assert_eq!(tri.neighbor_count(), 0);
        tri.neighbors.push(1);
        tri.neighbors.push(2);
        assert_eq!(tri.neighbor_count(), 2);
    }

    #[test]
    fn test_navtri_has_neighbor() {
        let mut tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        tri.neighbors.push(5);
        assert!(tri.has_neighbor(5));
        assert!(!tri.has_neighbor(3));
    }

    #[test]
    fn test_navtri_is_isolated() {
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        assert!(tri.is_isolated());

        let mut connected = NavTri::new(1, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        connected.neighbors.push(0);
        assert!(!connected.is_isolated());
    }

    #[test]
    fn test_navtri_is_edge() {
        let mut tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        assert!(tri.is_edge()); // 0 neighbors < 3

        tri.neighbors = vec![1, 2];
        assert!(tri.is_edge()); // 2 neighbors < 3

        tri.neighbors.push(3);
        assert!(!tri.is_edge()); // 3 neighbors = 3
    }

    #[test]
    fn test_navtri_area() {
        let tri = NavTri::new(
            0,
            [Vec3::ZERO, Vec3::new(2.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 2.0)],
            Vec3::Y,
            Vec3::ZERO,
        );
        assert!((tri.area() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_navtri_perimeter() {
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        let expected = 1.0 + 2.0_f32.sqrt() + 1.0;
        assert!((tri.perimeter() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_navtri_distance_to() {
        let tri = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        assert!((tri.distance_to(Vec3::ONE) - 3.0_f32.sqrt()).abs() < 1e-5);
    }

    #[test]
    fn test_navtri_slope_degrees() {
        // Flat triangle (normal = Y)
        let flat = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        assert!(flat.slope_degrees() < 1e-3);

        // 45 degree slope
        let slope45 = NavTri::new(
            0,
            [Vec3::ZERO, Vec3::X, Vec3::Z],
            Vec3::new(0.0, 1.0, 1.0).normalize(),
            Vec3::ZERO,
        );
        assert!((slope45.slope_degrees() - 45.0).abs() < 1.0);
    }

    #[test]
    fn test_navtri_is_walkable() {
        let walkable = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::ZERO);
        assert!(walkable.is_walkable());

        let not_walkable = NavTri::new(0, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::NEG_Y, Vec3::ZERO);
        assert!(!not_walkable.is_walkable());
    }

    #[test]
    fn test_navtri_display() {
        let tri = NavTri::new(42, [Vec3::ZERO, Vec3::X, Vec3::Z], Vec3::Y, Vec3::new(0.5, 0.0, 0.5));
        let display = format!("{}", tri);
        assert!(display.contains("NavTri[42]"));
        assert!(display.contains("neighbors"));
    }

    // ===== Aabb Helper Tests =====

    #[test]
    fn test_aabb_zero() {
        let aabb = Aabb::zero();
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ZERO);
    }

    #[test]
    fn test_aabb_unit() {
        let aabb = Aabb::unit();
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ONE);
    }

    #[test]
    fn test_aabb_from_center_half_extents() {
        let aabb = Aabb::from_center_half_extents(Vec3::new(5.0, 5.0, 5.0), Vec3::splat(2.0));
        assert_eq!(aabb.min, Vec3::splat(3.0));
        assert_eq!(aabb.max, Vec3::splat(7.0));
    }

    #[test]
    fn test_aabb_center() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(4.0, 6.0, 8.0));
        let center = aabb.center();
        assert_eq!(center, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_aabb_size() {
        let aabb = Aabb::new(Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 7.0, 9.0));
        let size = aabb.size();
        assert_eq!(size, Vec3::new(3.0, 5.0, 6.0));
    }

    #[test]
    fn test_aabb_half_extents() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(4.0, 6.0, 8.0));
        let half = aabb.half_extents();
        assert_eq!(half, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_aabb_volume() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
        assert!((aabb.volume() - 24.0).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_surface_area() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 3.0, 4.0));
        // 2 * (2*3 + 3*4 + 4*2) = 2 * (6 + 12 + 8) = 52
        assert!((aabb.surface_area() - 52.0).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_is_empty() {
        let empty = Aabb::new(Vec3::splat(5.0), Vec3::splat(5.0));
        assert!(empty.is_empty());

        let non_empty = Aabb::new(Vec3::ZERO, Vec3::ONE);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_aabb_longest_axis() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 5.0, 3.0));
        assert!((aabb.longest_axis() - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_shortest_axis() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 5.0, 3.0));
        assert!((aabb.shortest_axis() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_expand() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let expanded = aabb.expand(0.5);
        assert_eq!(expanded.min, Vec3::splat(-0.5));
        assert_eq!(expanded.max, Vec3::splat(1.5));
    }

    #[test]
    fn test_aabb_distance_to_point() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0));
        let center = aabb.center();
        let distance = aabb.distance_to_point(Vec3::new(5.0, 1.0, 1.0));
        assert!((distance - center.distance(Vec3::new(5.0, 1.0, 1.0))).abs() < 1e-5);
    }

    #[test]
    fn test_aabb_display() {
        let aabb = Aabb::new(Vec3::ZERO, Vec3::ONE);
        let display = format!("{}", aabb);
        assert!(display.contains("AABB"));
    }

    // ===== NavMesh Helper Tests =====

    #[test]
    fn test_navmesh_triangle_count() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
            Triangle::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.triangle_count(), 2);
    }

    #[test]
    fn test_navmesh_is_empty() {
        let empty = NavMesh::bake(&[], 0.4, 60.0);
        assert!(empty.is_empty());

        let non_empty = NavMesh::bake(&[Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0))], 0.4, 60.0);
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_navmesh_edge_count() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
            Triangle::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.edge_count(), 1); // One shared edge
    }

    #[test]
    fn test_navmesh_average_neighbor_count() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
            Triangle::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert!((nav.average_neighbor_count() - 1.0).abs() < 1e-5); // Each has 1 neighbor
    }

    #[test]
    fn test_navmesh_isolated_count() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
            Triangle::new(Vec3::new(10.0, 0.0, 0.0), Vec3::new(10.0, 0.0, 1.0), Vec3::new(11.0, 0.0, 0.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert_eq!(nav.isolated_count(), 2); // Both isolated
    }

    #[test]
    fn test_navmesh_total_area() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 2.0), Vec3::new(2.0, 0.0, 0.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert!((nav.total_area() - 2.0).abs() < 1e-5);
    }

    #[test]
    fn test_navmesh_bounds() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
            Triangle::new(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        let bounds = nav.bounds().unwrap();
        assert_eq!(bounds.min, Vec3::ZERO);
        assert_eq!(bounds.max, Vec3::new(1.0, 0.0, 1.0));
    }

    #[test]
    fn test_navmesh_bounds_empty() {
        let nav = NavMesh::bake(&[], 0.4, 60.0);
        assert!(nav.bounds().is_none());
    }

    #[test]
    fn test_navmesh_get_triangle() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        assert!(nav.get_triangle(0).is_some());
        assert!(nav.get_triangle(99).is_none());
    }

    #[test]
    fn test_navmesh_summary() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        let summary = nav.summary();
        assert!(summary.contains("NavMesh"));
        assert!(summary.contains("triangle"));
    }

    #[test]
    fn test_navmesh_display() {
        let tris = vec![
            Triangle::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)),
        ];
        let nav = NavMesh::bake(&tris, 0.4, 60.0);
        let display = format!("{}", nav);
        assert!(display.contains("NavMesh"));
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
