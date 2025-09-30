use std::collections::HashMap;
use anyhow::Result;
use glam::Vec3;
use wgpu::{Buffer, Device, Queue, BufferUsages};
use wgpu::util::DeviceExt;

use crate::mesh::CpuMesh;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MeshKey(pub String);

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub u32);

pub struct MeshRegistry {
    next_id: u32,
    map: HashMap<MeshKey, MeshHandle>,
    uploads: HashMap<MeshHandle, GpuMesh>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::MeshVertex;
    use glam::{Vec2, Vec3, Vec4};

    #[test]
    fn cpu_mesh_aabb() {
        let m = CpuMesh {
            vertices: vec![
                MeshVertex::new(Vec3::new(-1.0, 0.0, 2.0), Vec3::Y, Vec4::new(1.0,0.0,0.0,1.0), Vec2::ZERO),
                MeshVertex::new(Vec3::new(3.0, -2.0, -1.0), Vec3::Y, Vec4::new(1.0,0.0,0.0,1.0), Vec2::ZERO),
            ],
            indices: vec![0,1,1],
        };
        let (min, max) = m.aabb().unwrap();
        assert_eq!(min, Vec3::new(-1.0, -2.0, -1.0));
        assert_eq!(max, Vec3::new(3.0, 0.0, 2.0));
    }
}

pub struct GpuMesh {
    // Full interleaved vertex buffer (MeshVertex layout)
    pub vertex_full: Buffer,
    // Position-only vertex buffer (Float32x3 stride) for pipelines that only consume positions
    pub vertex_pos: Buffer,
    pub index: Buffer,
    pub index_count: u32,
    pub aabb: Option<(Vec3, Vec3)>,
}

impl MeshRegistry {
    pub fn new() -> Self {
        Self { next_id: 1, map: HashMap::new(), uploads: HashMap::new() }
    }

    pub fn get(&self, key: &MeshKey) -> Option<MeshHandle> { self.map.get(key).copied() }

    pub fn fetch_or_upload(&mut self, device: &Device, _queue: &Queue, key: MeshKey, mesh: &CpuMesh) -> Result<MeshHandle> {
        if let Some(h) = self.map.get(&key).copied() { return Ok(h); }
        let handle = MeshHandle(self.next_id); self.next_id += 1;

        let vertex_full = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("mesh-vertex-full-{}", handle.0)),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        let positions: Vec<[f32;3]> = mesh.vertices.iter().map(|v| v.position).collect();
        let vertex_pos = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("mesh-vertex-pos-{}", handle.0)),
            contents: bytemuck::cast_slice(&positions),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        let index = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("mesh-index-{}", handle.0)),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });
        let gpu = GpuMesh { vertex_full, vertex_pos, index, index_count: mesh.indices.len() as u32, aabb: mesh.aabb() };
        self.map.insert(key, handle);
        self.uploads.insert(handle, gpu);
        Ok(handle)
    }

    pub fn get_gpu(&self, handle: MeshHandle) -> Option<&GpuMesh> { self.uploads.get(&handle) }
}
