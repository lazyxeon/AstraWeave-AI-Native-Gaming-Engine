use glam::{Vec2, Vec3, Vec4};
use bytemuck::{Pod, Zeroable};

// Canonical vertex layout for engine meshes: P/N/T/UV
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct MeshVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4], // xyz=tangent, w=handedness
    pub uv: [f32; 2],
}

impl MeshVertex {
    pub fn new(position: Vec3, normal: Vec3, tangent: Vec4, uv: Vec2) -> Self {
        Self { position: position.to_array(), normal: normal.to_array(), tangent: tangent.to_array(), uv: uv.to_array() }
    }
    pub fn from_arrays(position: [f32;3], normal: [f32;3], tangent: [f32;4], uv: [f32;2]) -> Self {
        Self { position, normal, tangent, uv }
    }
    pub const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x4, // tangent
        3 => Float32x2, // uv
    ];
}

pub struct MeshVertexLayout;
impl MeshVertexLayout {
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &MeshVertex::ATTRIBS,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CpuMesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

impl CpuMesh {
    pub fn aabb(&self) -> Option<(Vec3, Vec3)> {
        if self.vertices.is_empty() { return None; }
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for v in &self.vertices {
            let p = Vec3::from_array(v.position);
            min = min.min(p);
            max = max.max(p);
        }
        Some((min, max))
    }
}

// Public tangent generation utility (MikkTSpace-like approximation)
pub fn compute_tangents(mesh: &mut CpuMesh) {
    if mesh.indices.len() % 3 != 0 { return; }
    let v = &mut mesh.vertices;
    let idx = &mesh.indices;
    let mut tan1: Vec<Vec3> = vec![Vec3::ZERO; v.len()];
    let mut tan2: Vec<Vec3> = vec![Vec3::ZERO; v.len()];
    for tri in idx.chunks_exact(3) {
        let (i0,i1,i2) = (tri[0] as usize, tri[1] as usize, tri[2] as usize);
        let p0 = Vec3::from_array(v[i0].position);
        let p1 = Vec3::from_array(v[i1].position);
        let p2 = Vec3::from_array(v[i2].position);
        let uv0 = Vec2::from_array(v[i0].uv);
        let uv1 = Vec2::from_array(v[i1].uv);
        let uv2 = Vec2::from_array(v[i2].uv);
        let dp1 = p1 - p0; let dp2 = p2 - p0;
        let duv1 = uv1 - uv0; let duv2 = uv2 - uv0;
        let r = 1.0 / (duv1.x * duv2.y - duv1.y * duv2.x).max(1e-8);
        let sdir = (dp1 * duv2.y - dp2 * duv1.y) * r;
        let tdir = (dp2 * duv1.x - dp1 * duv2.x) * r;
        tan1[i0] += sdir; tan1[i1] += sdir; tan1[i2] += sdir;
        tan2[i0] += tdir; tan2[i1] += tdir; tan2[i2] += tdir;
    }
    for i in 0..v.len() {
        let n = Vec3::from_array(v[i].normal).normalize_or_zero();
        let t = tan1[i];
        let tangent = (t - n * n.dot(t)).normalize_or_zero();
        let w = if n.cross(t).dot(tan2[i]) < 0.0 { -1.0 } else { 1.0 };
        v[i].tangent = [tangent.x, tangent.y, tangent.z, w];
    }
}
