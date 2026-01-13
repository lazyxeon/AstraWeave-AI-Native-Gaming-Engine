#![allow(dead_code)]

use astraweave_terrain::{
    BiomeConfig, BiomeType, ChunkId, Heightmap, TerrainChunk, WorldConfig, WorldGenerator,
};
use glam::Vec3;
use std::collections::HashMap;

pub struct TerrainState {
    generator: Option<WorldGenerator>,
    config: WorldConfig,
    generated_chunks: HashMap<ChunkId, GeneratedChunk>,
    terrain_dirty: bool,
    last_seed: u64,
    last_biome: String,
}

pub struct GeneratedChunk {
    pub chunk: TerrainChunk,
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
    pub world_position: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TerrainVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub biome_id: u32,
    _padding: [u32; 3],
}

impl TerrainVertex {
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2], biome_id: u32) -> Self {
        Self {
            position,
            normal,
            uv,
            biome_id,
            _padding: [0; 3],
        }
    }
}

impl Default for TerrainState {
    fn default() -> Self {
        Self {
            generator: None,
            config: WorldConfig::default(),
            generated_chunks: HashMap::new(),
            terrain_dirty: true,
            last_seed: 0,
            last_biome: String::new(),
        }
    }
}

impl TerrainState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn configure(&mut self, seed: u64, primary_biome: &str) {
        if self.last_seed != seed || self.last_biome != primary_biome {
            self.terrain_dirty = true;
            self.last_seed = seed;
            self.last_biome = primary_biome.to_string();
        }

        self.config.seed = seed;
        self.config.biomes = Self::biomes_for_primary(primary_biome);
    }

    fn biomes_for_primary(primary: &str) -> Vec<BiomeConfig> {
        let primary_type = primary
            .parse::<BiomeType>()
            .unwrap_or(BiomeType::Grassland);

        let mut biomes = vec![Self::biome_config_for_type(primary_type)];
        
        for bt in BiomeType::all() {
            if *bt != primary_type {
                biomes.push(Self::biome_config_for_type(*bt));
            }
        }
        biomes
    }

    fn biome_config_for_type(bt: BiomeType) -> BiomeConfig {
        match bt {
            BiomeType::Grassland => BiomeConfig::grassland(),
            BiomeType::Desert => BiomeConfig::desert(),
            BiomeType::Forest => BiomeConfig::forest(),
            BiomeType::Mountain => BiomeConfig::mountain(),
            BiomeType::Tundra => BiomeConfig::tundra(),
            BiomeType::Swamp => BiomeConfig::swamp(),
            BiomeType::Beach => BiomeConfig::beach(),
            BiomeType::River => BiomeConfig::river(),
        }
    }

    pub fn generate_terrain(&mut self, chunk_radius: i32) -> anyhow::Result<usize> {
        self.generator = Some(WorldGenerator::new(self.config.clone()));
        self.generated_chunks.clear();

        let generator = self.generator.as_mut().ok_or_else(|| {
            anyhow::anyhow!("Generator not initialized")
        })?;

        let chunk_size = self.config.chunk_size;
        let mut count = 0;

        for x in -chunk_radius..=chunk_radius {
            for z in -chunk_radius..=chunk_radius {
                let chunk_id = ChunkId { x, z };
                
                let (chunk, _scatter) = generator.generate_chunk_with_scatter(chunk_id)?;
                
                let world_pos = chunk_id.to_world_pos(chunk_size);
                let world_offset = Vec3::new(world_pos.x, 0.0, world_pos.y);
                
                let (vertices, indices) = Self::generate_heightmap_mesh(
                    chunk.heightmap(),
                    chunk.biome_map(),
                    chunk_size,
                    world_offset,
                );

                self.generated_chunks.insert(chunk_id, GeneratedChunk {
                    chunk,
                    vertices,
                    indices,
                    world_position: world_offset,
                });

                count += 1;
            }
        }

        self.terrain_dirty = false;
        Ok(count)
    }

    fn generate_heightmap_mesh(
        heightmap: &Heightmap,
        biome_map: &[BiomeType],
        chunk_size: f32,
        world_offset: Vec3,
    ) -> (Vec<TerrainVertex>, Vec<u32>) {
        let resolution = heightmap.resolution() as usize;
        let cell_size = chunk_size / (resolution - 1) as f32;
        
        let mut vertices = Vec::with_capacity(resolution * resolution);
        let mut indices = Vec::with_capacity((resolution - 1) * (resolution - 1) * 6);

        for z in 0..resolution {
            for x in 0..resolution {
                let height = heightmap.get_height(x as u32, z as u32);
                
                let world_x = world_offset.x + x as f32 * cell_size;
                let world_z = world_offset.z + z as f32 * cell_size;
                
                let normal = Self::calculate_normal(heightmap, x, z, cell_size);
                
                let biome_idx = z * resolution + x;
                let biome_id = biome_map.get(biome_idx)
                    .map(|b| Self::biome_to_id(*b))
                    .unwrap_or(0);

                vertices.push(TerrainVertex::new(
                    [world_x, height, world_z],
                    [normal.x, normal.y, normal.z],
                    [x as f32 / resolution as f32, z as f32 / resolution as f32],
                    biome_id,
                ));
            }
        }

        for z in 0..(resolution - 1) {
            for x in 0..(resolution - 1) {
                let top_left = (z * resolution + x) as u32;
                let top_right = top_left + 1;
                let bottom_left = ((z + 1) * resolution + x) as u32;
                let bottom_right = bottom_left + 1;

                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        (vertices, indices)
    }

    fn calculate_normal(heightmap: &Heightmap, x: usize, z: usize, cell_size: f32) -> Vec3 {
        let resolution = heightmap.resolution() as usize;
        
        let h_center = heightmap.get_height(x as u32, z as u32);
        let h_left = if x > 0 { heightmap.get_height((x - 1) as u32, z as u32) } else { h_center };
        let h_right = if x < resolution - 1 { heightmap.get_height((x + 1) as u32, z as u32) } else { h_center };
        let h_up = if z > 0 { heightmap.get_height(x as u32, (z - 1) as u32) } else { h_center };
        let h_down = if z < resolution - 1 { heightmap.get_height(x as u32, (z + 1) as u32) } else { h_center };

        let dx = (h_right - h_left) / (2.0 * cell_size);
        let dz = (h_down - h_up) / (2.0 * cell_size);

        Vec3::new(-dx, 1.0, -dz).normalize()
    }

    fn biome_to_id(biome: BiomeType) -> u32 {
        match biome {
            BiomeType::Grassland => 0,
            BiomeType::Desert => 1,
            BiomeType::Forest => 2,
            BiomeType::Mountain => 3,
            BiomeType::Tundra => 4,
            BiomeType::Swamp => 5,
            BiomeType::Beach => 6,
            BiomeType::River => 7,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.terrain_dirty
    }

    pub fn chunk_count(&self) -> usize {
        self.generated_chunks.len()
    }

    pub fn get_all_vertices(&self) -> Vec<TerrainVertex> {
        let mut all_vertices = Vec::new();
        for gen_chunk in self.generated_chunks.values() {
            all_vertices.extend_from_slice(&gen_chunk.vertices);
        }
        all_vertices
    }

    pub fn get_all_indices(&self, vertex_offset: u32) -> Vec<u32> {
        let mut all_indices = Vec::new();
        let mut current_offset = vertex_offset;
        
        for gen_chunk in self.generated_chunks.values() {
            for &idx in &gen_chunk.indices {
                all_indices.push(idx + current_offset);
            }
            current_offset += gen_chunk.vertices.len() as u32;
        }
        all_indices
    }

    pub fn get_height_at(&self, world_x: f32, world_z: f32) -> Option<f32> {
        let chunk_size = self.config.chunk_size;
        let chunk_x = (world_x / chunk_size).floor() as i32;
        let chunk_z = (world_z / chunk_size).floor() as i32;
        let chunk_id = ChunkId { x: chunk_x, z: chunk_z };

        if let Some(gen_chunk) = self.generated_chunks.get(&chunk_id) {
            let world_pos = Vec3::new(world_x, 0.0, world_z);
            gen_chunk.chunk.get_height_at_world_pos(world_pos, chunk_size)
        } else {
            None
        }
    }

    pub fn seed(&self) -> u64 {
        self.config.seed
    }

    pub fn primary_biome(&self) -> &str {
        if let Some(first) = self.config.biomes.first() {
            first.biome_type.as_str()
        } else {
            "grassland"
        }
    }

    pub fn chunks(&self) -> impl Iterator<Item = (&ChunkId, &GeneratedChunk)> {
        self.generated_chunks.iter()
    }

    pub fn get_gpu_chunks(&self) -> Vec<(Vec<TerrainVertex>, Vec<u32>)> {
        self.generated_chunks
            .values()
            .map(|chunk| (chunk.vertices.clone(), chunk.indices.clone()))
            .collect()
    }
}

impl TerrainVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TerrainVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

pub fn biome_display_name(biome_str: &str) -> &'static str {
    match biome_str {
        "grassland" => "Grassland",
        "desert" => "Desert", 
        "forest" => "Forest",
        "mountain" => "Mountain",
        "tundra" => "Tundra",
        "swamp" => "Swamp",
        "beach" => "Beach",
        "river" => "River",
        "temperate_forest" => "Forest",
        _ => "Unknown",
    }
}

pub fn all_biome_options() -> &'static [(&'static str, &'static str)] {
    &[
        ("grassland", "Grassland"),
        ("desert", "Desert"),
        ("forest", "Forest"),
        ("mountain", "Mountain"),
        ("tundra", "Tundra"),
        ("swamp", "Swamp"),
        ("beach", "Beach"),
        ("river", "River"),
    ]
}
