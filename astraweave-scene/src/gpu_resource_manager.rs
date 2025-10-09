//! GPU Resource Lifecycle Management for World Partition
//!
//! This module manages wgpu resources (buffers, textures) per cell,
//! enforcing memory budgets and handling upload/unload lifecycle.

use crate::world_partition::GridCoord;
use anyhow::Result;
use std::collections::HashMap;
use wgpu::{Buffer, Device, Queue, Texture};

/// Handle types for GPU resources
pub type AssetId = u64;

/// GPU resources for a single cell
pub struct CellGpuResources {
    /// Cell coordinate for debugging
    pub coord: GridCoord,
    /// Vertex buffers mapped by asset ID
    pub vertex_buffers: HashMap<AssetId, Buffer>,
    /// Index buffers mapped by asset ID
    pub index_buffers: HashMap<AssetId, Buffer>,
    /// Textures mapped by asset ID
    pub textures: HashMap<AssetId, Texture>,
    /// Current memory usage in bytes
    pub memory_usage: usize,
}

impl CellGpuResources {
    pub fn new(coord: GridCoord) -> Self {
        Self {
            coord,
            vertex_buffers: HashMap::new(),
            index_buffers: HashMap::new(),
            textures: HashMap::new(),
            memory_usage: 0,
        }
    }

    /// Upload a mesh's vertex buffer
    pub fn upload_vertex_buffer(
        &mut self,
        device: &Device,
        _queue: &Queue,
        asset_id: AssetId,
        vertices: &[u8],
    ) -> Result<()> {
        use wgpu::util::DeviceExt;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("cell-{:?}-mesh-{}-vertices", self.coord, asset_id)),
            contents: vertices,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let buffer_size = vertices.len();
        self.vertex_buffers.insert(asset_id, vertex_buffer);
        self.memory_usage += buffer_size;

        Ok(())
    }

    /// Upload a mesh's index buffer
    pub fn upload_index_buffer(
        &mut self,
        device: &Device,
        _queue: &Queue,
        asset_id: AssetId,
        indices: &[u8],
    ) -> Result<()> {
        use wgpu::util::DeviceExt;

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("cell-{:?}-mesh-{}-indices", self.coord, asset_id)),
            contents: indices,
            usage: wgpu::BufferUsages::INDEX,
        });

        let buffer_size = indices.len();
        self.index_buffers.insert(asset_id, index_buffer);
        self.memory_usage += buffer_size;

        Ok(())
    }

    /// Upload a texture
    pub fn upload_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        asset_id: AssetId,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<()> {
        // Create texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&format!("cell-{:?}-texture-{}", self.coord, asset_id)),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload data
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let texture_size = (width * height * 4) as usize; // RGBA8
        self.textures.insert(asset_id, texture);
        self.memory_usage += texture_size;

        Ok(())
    }

    /// Unload all GPU resources for this cell
    pub fn unload_all(&mut self) {
        // Dropping the buffers and textures will release GPU memory
        self.vertex_buffers.clear();
        self.index_buffers.clear();
        self.textures.clear();
        self.memory_usage = 0;
    }

    /// Get a vertex buffer by asset ID
    pub fn get_vertex_buffer(&self, asset_id: AssetId) -> Option<&Buffer> {
        self.vertex_buffers.get(&asset_id)
    }

    /// Get an index buffer by asset ID
    pub fn get_index_buffer(&self, asset_id: AssetId) -> Option<&Buffer> {
        self.index_buffers.get(&asset_id)
    }

    /// Get a texture by asset ID
    pub fn get_texture(&self, asset_id: AssetId) -> Option<&Texture> {
        self.textures.get(&asset_id)
    }
}

/// GPU resource budget manager
pub struct GpuResourceBudget {
    /// Maximum memory in bytes (default: 500MB)
    pub max_memory_bytes: usize,
    /// Current memory usage across all cells
    pub current_usage: usize,
    /// Resources per cell
    pub cells: HashMap<GridCoord, CellGpuResources>,
}

impl GpuResourceBudget {
    pub fn new(max_memory_bytes: usize) -> Self {
        Self {
            max_memory_bytes,
            current_usage: 0,
            cells: HashMap::new(),
        }
    }

    /// Create with default 500MB budget
    pub fn with_default_budget() -> Self {
        Self::new(500 * 1024 * 1024) // 500MB
    }

    /// Check if allocation would exceed budget
    pub fn can_allocate(&self, bytes: usize) -> bool {
        self.current_usage + bytes <= self.max_memory_bytes
    }

    /// Get or create cell resources
    pub fn get_or_create_cell(&mut self, coord: GridCoord) -> &mut CellGpuResources {
        self.cells
            .entry(coord)
            .or_insert_with(|| CellGpuResources::new(coord))
    }

    /// Unload a specific cell's resources
    pub fn unload_cell(&mut self, coord: GridCoord) {
        if let Some(cell) = self.cells.get_mut(&coord) {
            self.current_usage = self.current_usage.saturating_sub(cell.memory_usage);
            cell.unload_all();
        }
        self.cells.remove(&coord);
    }

    /// Enforce budget by unloading furthest cells
    pub fn enforce_budget(&mut self, camera_pos: glam::Vec3, cell_size: f32) {
        while self.current_usage > self.max_memory_bytes {
            // Find furthest cell from camera
            let furthest_coord = self.find_furthest_cell(camera_pos, cell_size);

            if let Some(coord) = furthest_coord {
                self.unload_cell(coord);
            } else {
                // No more cells to unload
                break;
            }
        }
    }

    /// Find the cell furthest from camera
    fn find_furthest_cell(&self, camera_pos: glam::Vec3, cell_size: f32) -> Option<GridCoord> {
        let mut furthest: Option<(GridCoord, f32)> = None;

        for coord in self.cells.keys() {
            let cell_center = coord.to_world_center(cell_size);
            let distance = (cell_center - camera_pos).length();

            match furthest {
                None => furthest = Some((*coord, distance)),
                Some((_, max_dist)) if distance > max_dist => furthest = Some((*coord, distance)),
                _ => {}
            }
        }

        furthest.map(|(coord, _)| coord)
    }

    /// Update current usage tracking
    pub fn update_usage(&mut self) {
        self.current_usage = self.cells.values().map(|c| c.memory_usage).sum();
    }

    /// Get memory usage statistics
    pub fn stats(&self) -> GpuMemoryStats {
        GpuMemoryStats {
            total_allocated: self.current_usage,
            max_budget: self.max_memory_bytes,
            active_cells: self.cells.len(),
            utilization: (self.current_usage as f32 / self.max_memory_bytes as f32) * 100.0,
        }
    }
}

/// GPU memory statistics
#[derive(Debug, Clone, Copy)]
pub struct GpuMemoryStats {
    pub total_allocated: usize,
    pub max_budget: usize,
    pub active_cells: usize,
    pub utilization: f32, // Percentage
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_gpu_resources_creation() {
        let coord = GridCoord::new(0, 0, 0);
        let resources = CellGpuResources::new(coord);

        assert_eq!(resources.coord, coord);
        assert_eq!(resources.memory_usage, 0);
        assert_eq!(resources.vertex_buffers.len(), 0);
    }

    #[test]
    fn test_budget_creation() {
        let budget = GpuResourceBudget::with_default_budget();
        assert_eq!(budget.max_memory_bytes, 500 * 1024 * 1024);
        assert_eq!(budget.current_usage, 0);
    }

    #[test]
    fn test_can_allocate() {
        let budget = GpuResourceBudget::new(1000);
        assert!(budget.can_allocate(500));
        assert!(budget.can_allocate(1000));
        assert!(!budget.can_allocate(1001));
    }

    #[test]
    fn test_unload_cell() {
        let mut budget = GpuResourceBudget::new(1000);
        let coord = GridCoord::new(0, 0, 0);

        // Create cell with simulated memory usage
        {
            let cell = budget.get_or_create_cell(coord);
            cell.memory_usage = 500;
        }

        budget.current_usage = 500;

        budget.unload_cell(coord);
        assert_eq!(budget.current_usage, 0);
        assert!(!budget.cells.contains_key(&coord));
    }

    #[test]
    fn test_find_furthest_cell() {
        let mut budget = GpuResourceBudget::new(10000);
        let cell_size = 100.0;

        // Create cells at different positions
        budget.get_or_create_cell(GridCoord::new(0, 0, 0)); // Center
        budget.get_or_create_cell(GridCoord::new(10, 0, 0)); // Far on X
        budget.get_or_create_cell(GridCoord::new(0, 0, 10)); // Far on Z

        let camera_pos = glam::Vec3::ZERO;
        let furthest = budget.find_furthest_cell(camera_pos, cell_size);

        // Should find one of the far cells (both are equidistant)
        assert!(furthest.is_some());
        let coord = furthest.unwrap();
        assert!(coord == GridCoord::new(10, 0, 0) || coord == GridCoord::new(0, 0, 10));
    }

    #[test]
    fn test_enforce_budget() {
        let mut budget = GpuResourceBudget::new(1000);
        let cell_size = 100.0;

        // Create cells exceeding budget
        let coords = [
            GridCoord::new(0, 0, 0),
            GridCoord::new(1, 0, 0),
            GridCoord::new(2, 0, 0),
        ];

        for coord in &coords {
            let cell = budget.get_or_create_cell(*coord);
            cell.memory_usage = 500;
        }

        budget.current_usage = 1500; // Exceeds budget

        let camera_pos = glam::Vec3::new(50.0, 0.0, 50.0); // Near cell (0,0,0)
        budget.enforce_budget(camera_pos, cell_size);

        // Budget should be enforced
        assert!(budget.current_usage <= budget.max_memory_bytes);
        assert!(budget.cells.len() < 3); // At least one cell unloaded
    }

    #[test]
    fn test_stats() {
        let mut budget = GpuResourceBudget::new(1000);
        let coord = GridCoord::new(0, 0, 0);

        {
            let cell = budget.get_or_create_cell(coord);
            cell.memory_usage = 500;
        }

        budget.current_usage = 500;

        let stats = budget.stats();
        assert_eq!(stats.total_allocated, 500);
        assert_eq!(stats.max_budget, 1000);
        assert_eq!(stats.active_cells, 1);
        assert_eq!(stats.utilization, 50.0);
    }
}
