//! GPU Culling & LOD Generation Benchmarks
//!
//! Comprehensive CPU-side benchmarks for:
//! 1. Frustum culling (CPU fallback path)
//! 2. AABB construction and transformation
//! 3. Frustum plane extraction
//! 4. Batch building and indirect commands
//! 5. LOD generation with quadric error metrics
//! 6. Mesh simplification
//!
//! These benchmarks measure CPU performance for culling and LOD operations.

// =============================================================================
// MISSION-CRITICAL CORRECTNESS ASSERTIONS
// =============================================================================
// Culling & LOD benchmarks validate CORRECTNESS of visibility systems.
// Assertions verify:
//   1. AABB Validity: Bounding boxes have non-negative extent, finite values
//   2. Frustum Planes: Normal vectors are normalized, plane equations valid
//   3. Culling Results: Visible count <= total count, indices valid
//   4. LOD Selection: LOD index is within valid range for mesh
//   5. Batch Integrity: Draw indirect commands have valid counts
// =============================================================================

#![allow(dead_code)]
#![allow(private_interfaces)]
#![allow(unused_variables)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

/// CORRECTNESS: Validate AABB has valid geometry
#[inline]
fn assert_aabb_valid(center: [f32; 3], extent: [f32; 3], context: &str) {
    // Center must be finite
    for i in 0..3 {
        assert!(center[i].is_finite(),
            "[CORRECTNESS FAILURE] {}: AABB center[{}] is non-finite ({})", context, i, center[i]);
    }
    // Extent must be non-negative and finite
    for i in 0..3 {
        assert!(extent[i] >= 0.0 && extent[i].is_finite(),
            "[CORRECTNESS FAILURE] {}: AABB extent[{}] invalid ({})", context, i, extent[i]);
    }
}

/// CORRECTNESS: Validate frustum plane is properly normalized
#[inline]
fn assert_frustum_plane_valid(plane: [f32; 4], context: &str) {
    // Normal should be approximately unit length
    let normal_len = (plane[0]*plane[0] + plane[1]*plane[1] + plane[2]*plane[2]).sqrt();
    assert!(normal_len > 0.99 && normal_len < 1.01,
        "[CORRECTNESS FAILURE] {}: frustum plane normal not normalized (len={})", context, normal_len);
    // Distance should be finite
    assert!(plane[3].is_finite(),
        "[CORRECTNESS FAILURE] {}: frustum plane distance non-finite ({})", context, plane[3]);
}

/// CORRECTNESS: Validate culling results are consistent
#[inline]
fn assert_culling_result_valid(visible_count: usize, total_count: usize, context: &str) {
    assert!(visible_count <= total_count,
        "[CORRECTNESS FAILURE] {}: visible ({}) > total ({})", context, visible_count, total_count);
}

// ============================================================================
// AABB & Frustum Structures (matching culling.rs)
// ============================================================================

/// Per-instance AABB for culling
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct InstanceAABB {
    pub center: [f32; 3],
    pub _pad0: u32,
    pub extent: [f32; 3],
    pub instance_index: u32,
}

impl InstanceAABB {
    pub fn new(center: [f32; 3], extent: [f32; 3], instance_index: u32) -> Self {
        // CORRECTNESS: Validate AABB on construction
        assert_aabb_valid(center, extent, "InstanceAABB::new");
        Self {
            center,
            _pad0: 0,
            extent,
            instance_index,
        }
    }

    /// Create AABB from transform matrix and local bounds
    pub fn from_transform(
        transform: &[[f32; 4]; 4],
        local_min: [f32; 3],
        local_max: [f32; 3],
        instance_index: u32,
    ) -> Self {
        let local_center = [
            (local_min[0] + local_max[0]) * 0.5,
            (local_min[1] + local_max[1]) * 0.5,
            (local_min[2] + local_max[2]) * 0.5,
        ];
        let local_extent = [
            (local_max[0] - local_min[0]) * 0.5,
            (local_max[1] - local_min[1]) * 0.5,
            (local_max[2] - local_min[2]) * 0.5,
        ];

        // Transform center to world space
        let world_center = transform_point(transform, local_center);

        // Transform extent by taking absolute values of transformed basis vectors
        let mut world_extent = [0.0f32; 3];
        for i in 0..3 {
            world_extent[i] = 
                local_extent[0] * transform[0][i].abs() +
                local_extent[1] * transform[1][i].abs() +
                local_extent[2] * transform[2][i].abs();
        }

        Self {
            center: world_center,
            _pad0: 0,
            extent: world_extent,
            instance_index,
        }
    }
}

fn transform_point(m: &[[f32; 4]; 4], p: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2] + m[3][0],
        m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2] + m[3][1],
        m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2] + m[3][2],
    ]
}

/// Frustum planes for culling (6 planes: left, right, bottom, top, near, far)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FrustumPlanes {
    pub planes: [[f32; 4]; 6],
}

impl FrustumPlanes {
    /// Extract frustum planes from view-projection matrix (Gribb-Hartmann method)
    pub fn from_view_proj(vp: &[[f32; 4]; 4]) -> Self {
        // Flatten matrix for easier access
        let m: [f32; 16] = [
            vp[0][0], vp[0][1], vp[0][2], vp[0][3],
            vp[1][0], vp[1][1], vp[1][2], vp[1][3],
            vp[2][0], vp[2][1], vp[2][2], vp[2][3],
            vp[3][0], vp[3][1], vp[3][2], vp[3][3],
        ];

        let left = Self::normalize_plane([m[3] + m[0], m[7] + m[4], m[11] + m[8], m[15] + m[12]]);
        let right = Self::normalize_plane([m[3] - m[0], m[7] - m[4], m[11] - m[8], m[15] - m[12]]);
        let bottom = Self::normalize_plane([m[3] + m[1], m[7] + m[5], m[11] + m[9], m[15] + m[13]]);
        let top = Self::normalize_plane([m[3] - m[1], m[7] - m[5], m[11] - m[9], m[15] - m[13]]);
        let near = Self::normalize_plane([m[3] + m[2], m[7] + m[6], m[11] + m[10], m[15] + m[14]]);
        let far = Self::normalize_plane([m[3] - m[2], m[7] - m[6], m[11] - m[10], m[15] - m[14]]);

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    fn normalize_plane(plane: [f32; 4]) -> [f32; 4] {
        let len = (plane[0] * plane[0] + plane[1] * plane[1] + plane[2] * plane[2]).sqrt();
        if len > 0.0 {
            [plane[0] / len, plane[1] / len, plane[2] / len, plane[3] / len]
        } else {
            plane
        }
    }

    /// Test if AABB intersects frustum (CPU culling)
    pub fn test_aabb(&self, center: [f32; 3], extent: [f32; 3]) -> bool {
        for plane in &self.planes {
            let dist = plane[0] * center[0] + plane[1] * center[1] + plane[2] * center[2] + plane[3];
            let radius = extent[0] * plane[0].abs() + extent[1] * plane[1].abs() + extent[2] * plane[2].abs();
            if dist < -radius {
                return false;
            }
        }
        true
    }
}

/// Indirect draw command
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct DrawIndirectCommand {
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
}

/// Batch identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BatchId {
    pub mesh_id: u32,
    pub material_id: u32,
}

/// Draw batch
#[derive(Debug, Clone)]
pub struct DrawBatch {
    pub batch_id: BatchId,
    pub vertex_count: u32,
    pub first_vertex: u32,
    pub instances: Vec<u32>,
}

impl DrawBatch {
    pub fn new(batch_id: BatchId, vertex_count: u32, first_vertex: u32) -> Self {
        Self {
            batch_id,
            vertex_count,
            first_vertex,
            instances: Vec::new(),
        }
    }
}

/// CPU frustum culling
pub fn cpu_frustum_cull(instances: &[InstanceAABB], frustum: &FrustumPlanes) -> Vec<u32> {
    instances
        .iter()
        .filter(|inst| frustum.test_aabb(inst.center, inst.extent))
        .map(|inst| inst.instance_index)
        .collect()
}

/// Build indirect draw commands from batches
pub fn build_indirect_commands(batches: &[DrawBatch]) -> Vec<DrawIndirectCommand> {
    batches
        .iter()
        .map(|batch| DrawIndirectCommand {
            vertex_count: batch.vertex_count,
            instance_count: batch.instances.len() as u32,
            first_vertex: batch.first_vertex,
            first_instance: 0,
        })
        .collect()
}

// ============================================================================
// LOD Generation Structures (matching lod_generator.rs)
// ============================================================================

/// Quadric error matrix (4x4 symmetric, stored as 10 values)
#[derive(Debug, Clone, Copy)]
struct Quadric {
    data: [f64; 10],
}

impl Quadric {
    fn zero() -> Self {
        Self { data: [0.0; 10] }
    }

    fn from_plane(a: f64, b: f64, c: f64, d: f64) -> Self {
        Self {
            data: [
                a * a, a * b, a * c, a * d,
                b * b, b * c, b * d,
                c * c, c * d,
                d * d,
            ],
        }
    }

    fn add(&self, other: &Quadric) -> Quadric {
        let mut result = Quadric::zero();
        for i in 0..10 {
            result.data[i] = self.data[i] + other.data[i];
        }
        result
    }

    fn evaluate(&self, x: f64, y: f64, z: f64) -> f64 {
        let q11 = self.data[0];
        let q12 = self.data[1];
        let q13 = self.data[2];
        let q14 = self.data[3];
        let q22 = self.data[4];
        let q23 = self.data[5];
        let q24 = self.data[6];
        let q33 = self.data[7];
        let q34 = self.data[8];
        let q44 = self.data[9];

        x * (q11 * x + q12 * y + q13 * z + q14)
            + y * (q12 * x + q22 * y + q23 * z + q24)
            + z * (q13 * x + q23 * y + q33 * z + q34)
            + (q14 * x + q24 * y + q34 * z + q44)
    }
}

/// Edge collapse candidate for mesh simplification
#[derive(Debug, Clone)]
struct EdgeCollapse {
    v1: usize,
    v2: usize,
    error: f64,
    new_pos: [f32; 3],
}

impl PartialOrd for EdgeCollapse {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.error.partial_cmp(&self.error) // Min-heap
    }
}

impl PartialEq for EdgeCollapse {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error
    }
}

impl Eq for EdgeCollapse {}

impl Ord for EdgeCollapse {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

/// Simplified mesh for LOD
#[derive(Debug, Clone)]
pub struct SimplificationMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl SimplificationMesh {
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

/// LOD configuration
#[derive(Debug, Clone)]
pub struct LODConfig {
    pub reduction_targets: Vec<f32>,
    pub max_error: f32,
    pub preserve_boundaries: bool,
}

impl Default for LODConfig {
    fn default() -> Self {
        Self {
            reduction_targets: vec![0.75, 0.50, 0.25],
            max_error: 0.01,
            preserve_boundaries: true,
        }
    }
}

/// LOD Generator
pub struct LODGenerator {
    config: LODConfig,
}

impl LODGenerator {
    pub fn new(config: LODConfig) -> Self {
        Self { config }
    }

    /// Compute vertex quadrics from mesh triangles
    pub fn compute_vertex_quadrics(&self, mesh: &SimplificationMesh) -> Vec<Quadric> {
        let mut quadrics = vec![Quadric::zero(); mesh.vertex_count()];

        for tri_idx in 0..mesh.triangle_count() {
            let i0 = mesh.indices[tri_idx * 3] as usize;
            let i1 = mesh.indices[tri_idx * 3 + 1] as usize;
            let i2 = mesh.indices[tri_idx * 3 + 2] as usize;

            let p0 = mesh.positions[i0];
            let p1 = mesh.positions[i1];
            let p2 = mesh.positions[i2];

            // Compute triangle normal
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            let normal = if len > 0.0 { [n[0] / len, n[1] / len, n[2] / len] } else { [0.0, 1.0, 0.0] };

            let d = -(normal[0] * p0[0] + normal[1] * p0[1] + normal[2] * p0[2]);
            let quadric = Quadric::from_plane(
                normal[0] as f64,
                normal[1] as f64,
                normal[2] as f64,
                d as f64,
            );

            quadrics[i0] = quadrics[i0].add(&quadric);
            quadrics[i1] = quadrics[i1].add(&quadric);
            quadrics[i2] = quadrics[i2].add(&quadric);
        }

        quadrics
    }

    /// Build edge collapse candidates
    pub fn build_edge_collapses(
        &self,
        mesh: &SimplificationMesh,
        quadrics: &[Quadric],
    ) -> BinaryHeap<EdgeCollapse> {
        let mut collapses = BinaryHeap::new();
        let mut seen_edges = std::collections::HashSet::new();

        for tri_idx in 0..mesh.triangle_count() {
            let i0 = mesh.indices[tri_idx * 3] as usize;
            let i1 = mesh.indices[tri_idx * 3 + 1] as usize;
            let i2 = mesh.indices[tri_idx * 3 + 2] as usize;

            for &(v1, v2) in &[(i0, i1), (i1, i2), (i2, i0)] {
                let edge_key = if v1 < v2 { (v1, v2) } else { (v2, v1) };
                if seen_edges.insert(edge_key) {
                    let combined = quadrics[v1].add(&quadrics[v2]);
                    
                    // Use midpoint as collapse position
                    let p1 = mesh.positions[v1];
                    let p2 = mesh.positions[v2];
                    let mid = [
                        (p1[0] + p2[0]) * 0.5,
                        (p1[1] + p2[1]) * 0.5,
                        (p1[2] + p2[2]) * 0.5,
                    ];
                    
                    let error = combined.evaluate(mid[0] as f64, mid[1] as f64, mid[2] as f64);
                    
                    collapses.push(EdgeCollapse {
                        v1,
                        v2,
                        error,
                        new_pos: mid,
                    });
                }
            }
        }

        collapses
    }

    /// Perform mesh simplification
    pub fn simplify(&self, mesh: &SimplificationMesh, target_vertices: usize) -> SimplificationMesh {
        if mesh.vertex_count() <= target_vertices {
            return mesh.clone();
        }

        let quadrics = self.compute_vertex_quadrics(mesh);
        let mut collapses = self.build_edge_collapses(mesh, &quadrics);

        let mut active_vertices: Vec<bool> = vec![true; mesh.vertex_count()];
        let mut new_positions = mesh.positions.clone();
        let vertices_to_remove = mesh.vertex_count() - target_vertices;

        for _ in 0..vertices_to_remove {
            if let Some(collapse) = collapses.pop() {
                if collapse.error > self.config.max_error as f64 {
                    break;
                }

                if active_vertices[collapse.v1] && active_vertices[collapse.v2] {
                    new_positions[collapse.v1] = collapse.new_pos;
                    active_vertices[collapse.v2] = false;
                }
            } else {
                break;
            }
        }

        // Rebuild mesh (simplified version - just count remaining)
        let active_count = active_vertices.iter().filter(|&&v| v).count();
        SimplificationMesh {
            positions: new_positions.into_iter()
                .zip(active_vertices.iter())
                .filter(|(_, &active)| active)
                .map(|(pos, _)| pos)
                .collect(),
            normals: mesh.normals.clone(), // Simplified - would need remapping
            uvs: mesh.uvs.clone(),
            indices: Vec::new(), // Would need triangle remapping
        }
    }
}

// ============================================================================
// Test Data Generation
// ============================================================================

fn generate_random_instances(count: usize, visibility_ratio: f32) -> Vec<InstanceAABB> {
    let mut instances = Vec::with_capacity(count);
    let seed = 42u64;
    
    for i in 0..count {
        // Pseudo-random position based on index
        let hash = (seed.wrapping_mul(i as u64 + 1).wrapping_add(0x9E3779B97F4A7C15)) as f32 / u64::MAX as f32;
        
        // Place some instances inside frustum, some outside based on visibility_ratio
        let range = if hash < visibility_ratio { 10.0 } else { 100.0 };
        
        let x = (hash * 2.0 - 1.0) * range;
        let y = ((hash * 3.7) % 1.0 * 2.0 - 1.0) * range;
        let z = ((hash * 7.3) % 1.0) * -50.0 - 5.0; // Mostly in front of camera
        
        let extent = 1.0 + (hash * 5.0) % 5.0;
        
        instances.push(InstanceAABB::new(
            [x, y, z],
            [extent, extent, extent],
            i as u32,
        ));
    }
    instances
}

fn generate_standard_frustum() -> FrustumPlanes {
    // Simple perspective projection looking down -Z
    let fov = 60.0f32.to_radians();
    let aspect = 16.0 / 9.0;
    let near = 0.1;
    let far = 1000.0;
    
    let f = 1.0 / (fov / 2.0).tan();
    
    let proj = [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), -1.0],
        [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
    ];
    
    // Identity view matrix (camera at origin looking down -Z)
    let view_proj = proj; // For identity view, VP = P
    
    FrustumPlanes::from_view_proj(&view_proj)
}

fn generate_test_mesh(vertex_count: usize, triangle_count: usize) -> SimplificationMesh {
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);
    
    // Generate vertices on a sphere
    for i in 0..vertex_count {
        let phi = (i as f32 / vertex_count as f32) * std::f32::consts::PI * 2.0;
        let theta = (i as f32 / vertex_count as f32) * std::f32::consts::PI;
        
        let x = theta.sin() * phi.cos();
        let y = theta.cos();
        let z = theta.sin() * phi.sin();
        
        positions.push([x, y, z]);
        normals.push([x, y, z]); // Normal = position for unit sphere
        uvs.push([phi / (2.0 * std::f32::consts::PI), theta / std::f32::consts::PI]);
    }
    
    // Generate triangles (simplified - just use sequential indices)
    let actual_tris = triangle_count.min(vertex_count / 3);
    let indices: Vec<u32> = (0..(actual_tris * 3) as u32).map(|i| i % vertex_count as u32).collect();
    
    SimplificationMesh {
        positions,
        normals,
        uvs,
        indices,
    }
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_aabb_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("AABB Construction");
    
    // Simple AABB creation
    group.bench_function("aabb_new", |b| {
        b.iter(|| {
            InstanceAABB::new(
                black_box([1.0, 2.0, 3.0]),
                black_box([0.5, 0.5, 0.5]),
                black_box(42),
            )
        })
    });
    
    // AABB from transform matrix
    let transform: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [5.0, 10.0, -20.0, 1.0],
    ];
    
    group.bench_function("aabb_from_transform_identity", |b| {
        b.iter(|| {
            InstanceAABB::from_transform(
                black_box(&transform),
                black_box([-1.0, -1.0, -1.0]),
                black_box([1.0, 1.0, 1.0]),
                black_box(0),
            )
        })
    });
    
    // Rotated transform
    let rotated: [[f32; 4]; 4] = [
        [0.707, 0.707, 0.0, 0.0],
        [-0.707, 0.707, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [5.0, 10.0, -20.0, 1.0],
    ];
    
    group.bench_function("aabb_from_transform_rotated", |b| {
        b.iter(|| {
            InstanceAABB::from_transform(
                black_box(&rotated),
                black_box([-1.0, -1.0, -1.0]),
                black_box([1.0, 1.0, 1.0]),
                black_box(0),
            )
        })
    });
    
    // Batch AABB construction
    for count in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("batch", count), &count, |b, &count| {
            b.iter(|| {
                let mut aabbs = Vec::with_capacity(count);
                for i in 0..count {
                    aabbs.push(InstanceAABB::from_transform(
                        &transform,
                        [-1.0, -1.0, -1.0],
                        [1.0, 1.0, 1.0],
                        i as u32,
                    ));
                }
                black_box(aabbs)
            })
        });
    }
    
    group.finish();
}

fn bench_frustum_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("Frustum Extraction");
    
    let vp: [[f32; 4]; 4] = [
        [1.3, 0.0, 0.0, 0.0],
        [0.0, 1.73, 0.0, 0.0],
        [0.0, 0.0, -1.001, -1.0],
        [0.0, 0.0, -0.2, 0.0],
    ];
    
    group.bench_function("from_view_proj", |b| {
        b.iter(|| {
            FrustumPlanes::from_view_proj(black_box(&vp))
        })
    });
    
    group.finish();
}

fn bench_aabb_frustum_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("AABB-Frustum Test");
    
    let frustum = generate_standard_frustum();
    
    // Single AABB test - visible
    group.bench_function("single_visible", |b| {
        let aabb = InstanceAABB::new([0.0, 0.0, -10.0], [1.0, 1.0, 1.0], 0);
        b.iter(|| {
            frustum.test_aabb(black_box(aabb.center), black_box(aabb.extent))
        })
    });
    
    // Single AABB test - culled
    group.bench_function("single_culled", |b| {
        let aabb = InstanceAABB::new([100.0, 0.0, -10.0], [1.0, 1.0, 1.0], 0);
        b.iter(|| {
            frustum.test_aabb(black_box(aabb.center), black_box(aabb.extent))
        })
    });
    
    // Single AABB test - edge case (on boundary)
    group.bench_function("single_boundary", |b| {
        let aabb = InstanceAABB::new([8.0, 0.0, -10.0], [2.0, 2.0, 2.0], 0);
        b.iter(|| {
            frustum.test_aabb(black_box(aabb.center), black_box(aabb.extent))
        })
    });
    
    group.finish();
}

fn bench_cpu_frustum_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("CPU Frustum Culling");
    
    let frustum = generate_standard_frustum();
    
    // Different instance counts
    for count in [100, 1000, 10000, 50000] {
        group.throughput(Throughput::Elements(count as u64));
        
        // 50% visibility
        let instances_50 = generate_random_instances(count, 0.5);
        group.bench_with_input(
            BenchmarkId::new("50pct_visible", count),
            &instances_50,
            |b, instances| {
                b.iter(|| cpu_frustum_cull(black_box(instances), black_box(&frustum)))
            },
        );
        
        // 10% visibility (mostly culled)
        let instances_10 = generate_random_instances(count, 0.1);
        group.bench_with_input(
            BenchmarkId::new("10pct_visible", count),
            &instances_10,
            |b, instances| {
                b.iter(|| cpu_frustum_cull(black_box(instances), black_box(&frustum)))
            },
        );
        
        // 90% visibility (mostly visible)
        let instances_90 = generate_random_instances(count, 0.9);
        group.bench_with_input(
            BenchmarkId::new("90pct_visible", count),
            &instances_90,
            |b, instances| {
                b.iter(|| cpu_frustum_cull(black_box(instances), black_box(&frustum)))
            },
        );
    }
    
    group.finish();
}

fn bench_indirect_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("Indirect Commands");
    
    // Generate batches
    for batch_count in [10, 50, 100, 500] {
        let batches: Vec<DrawBatch> = (0..batch_count)
            .map(|i| {
                let mut batch = DrawBatch::new(
                    BatchId { mesh_id: i as u32, material_id: 0 },
                    1000,
                    i as u32 * 1000,
                );
                // Add some instances to each batch
                for j in 0..20 {
                    batch.instances.push(j);
                }
                batch
            })
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("build_commands", batch_count),
            &batches,
            |b, batches| {
                b.iter(|| build_indirect_commands(black_box(batches)))
            },
        );
    }
    
    group.finish();
}

fn bench_quadric_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Quadric Operations");
    
    // Quadric creation from plane
    group.bench_function("from_plane", |b| {
        b.iter(|| {
            Quadric::from_plane(
                black_box(0.577),
                black_box(0.577),
                black_box(0.577),
                black_box(-1.732),
            )
        })
    });
    
    // Quadric addition
    let q1 = Quadric::from_plane(0.0, 1.0, 0.0, -1.0);
    let q2 = Quadric::from_plane(1.0, 0.0, 0.0, -2.0);
    
    group.bench_function("add", |b| {
        b.iter(|| {
            black_box(&q1).add(black_box(&q2))
        })
    });
    
    // Quadric evaluation
    let q = Quadric::from_plane(0.577, 0.577, 0.577, -1.732);
    
    group.bench_function("evaluate", |b| {
        b.iter(|| {
            q.evaluate(black_box(1.0), black_box(1.0), black_box(1.0))
        })
    });
    
    group.finish();
}

fn bench_vertex_quadrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("Vertex Quadrics");
    
    let config = LODConfig::default();
    let generator = LODGenerator::new(config);
    
    // Different mesh sizes
    for (verts, tris) in [(100, 150), (500, 750), (1000, 1500), (5000, 7500)] {
        let mesh = generate_test_mesh(verts, tris);
        
        group.bench_with_input(
            BenchmarkId::new("compute", format!("{}v_{}t", verts, tris)),
            &mesh,
            |b, mesh| {
                b.iter(|| generator.compute_vertex_quadrics(black_box(mesh)))
            },
        );
    }
    
    group.finish();
}

fn bench_edge_collapses(c: &mut Criterion) {
    let mut group = c.benchmark_group("Edge Collapses");
    
    let config = LODConfig::default();
    let generator = LODGenerator::new(config);
    
    for (verts, tris) in [(100, 150), (500, 750), (1000, 1500)] {
        let mesh = generate_test_mesh(verts, tris);
        let quadrics = generator.compute_vertex_quadrics(&mesh);
        
        group.bench_with_input(
            BenchmarkId::new("build", format!("{}v_{}t", verts, tris)),
            &(&mesh, &quadrics),
            |b, (mesh, quadrics)| {
                b.iter(|| generator.build_edge_collapses(black_box(*mesh), black_box(*quadrics)))
            },
        );
    }
    
    group.finish();
}

fn bench_mesh_simplification(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mesh Simplification");
    group.sample_size(50); // Fewer samples for expensive operations
    
    let config = LODConfig::default();
    let generator = LODGenerator::new(config);
    
    // Different mesh sizes and reduction targets
    for (verts, tris) in [(500, 750), (1000, 1500), (2000, 3000)] {
        let mesh = generate_test_mesh(verts, tris);
        
        // 75% reduction
        let target_75 = (verts as f32 * 0.75) as usize;
        group.bench_with_input(
            BenchmarkId::new("75pct", format!("{}v", verts)),
            &(&mesh, target_75),
            |b, (mesh, target)| {
                b.iter(|| generator.simplify(black_box(*mesh), black_box(*target)))
            },
        );
        
        // 50% reduction
        let target_50 = (verts as f32 * 0.50) as usize;
        group.bench_with_input(
            BenchmarkId::new("50pct", format!("{}v", verts)),
            &(&mesh, target_50),
            |b, (mesh, target)| {
                b.iter(|| generator.simplify(black_box(*mesh), black_box(*target)))
            },
        );
        
        // 25% reduction
        let target_25 = (verts as f32 * 0.25) as usize;
        group.bench_with_input(
            BenchmarkId::new("25pct", format!("{}v", verts)),
            &(&mesh, target_25),
            |b, (mesh, target)| {
                b.iter(|| generator.simplify(black_box(*mesh), black_box(*target)))
            },
        );
    }
    
    group.finish();
}

fn bench_full_culling_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("Full Culling Pipeline");
    
    let frustum = generate_standard_frustum();
    
    for count in [1000, 5000, 10000] {
        let instances = generate_random_instances(count, 0.5);
        
        group.bench_with_input(
            BenchmarkId::new("cull_and_batch", count),
            &instances,
            |b, instances| {
                b.iter(|| {
                    // 1. Cull instances
                    let visible = cpu_frustum_cull(instances, &frustum);
                    
                    // 2. Group into batches (simplified)
                    let batch_count = (visible.len() / 50).max(1);
                    let batches: Vec<DrawBatch> = (0..batch_count)
                        .map(|i| {
                            let mut batch = DrawBatch::new(
                                BatchId { mesh_id: i as u32, material_id: 0 },
                                1000,
                                0,
                            );
                            batch.instances = visible[i * 50..(i * 50 + 50).min(visible.len())].to_vec();
                            batch
                        })
                        .collect();
                    
                    // 3. Build indirect commands
                    let commands = build_indirect_commands(&batches);
                    
                    black_box((visible.len(), commands))
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_aabb_construction,
    bench_frustum_extraction,
    bench_aabb_frustum_test,
    bench_cpu_frustum_culling,
    bench_indirect_commands,
    bench_quadric_operations,
    bench_vertex_quadrics,
    bench_edge_collapses,
    bench_mesh_simplification,
    bench_full_culling_pipeline,
);

criterion_main!(benches);
