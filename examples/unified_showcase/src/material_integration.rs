use std::{path::PathBuf, time::Instant};

use anyhow::Result;
use astraweave_render::{ArrayLayout, MaterialGpuArrays, MaterialLoadStats, MaterialManager};

pub struct MaterialPackRuntime {
    pub gpu: MaterialGpuArrays,
    pub stats: MaterialLoadStats,
    pub last_loaded: Instant,
    pub bind_group: wgpu::BindGroup,
}

pub struct MaterialIntegrator {
    manager: MaterialManager,
    pub bgl: wgpu::BindGroupLayout,
    current_biome: Option<String>,
    cache: std::collections::HashMap<String, MaterialPackRuntime>,
}

impl MaterialIntegrator {
    pub fn new(device: &wgpu::Device) -> Self {
        // Expectation: shaders use layout 0: albedo array, 1: albedo sampler, 2: normal array, 3: linear sampler, 4: mra array
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pbr-material-layers"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<astraweave_render::MaterialGpu>() as u64,
                        ),
                    },
                    count: None,
                },
            ],
        });
        Self {
            manager: MaterialManager::new(),
            bgl,
            current_biome: None,
            cache: Default::default(),
        }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bgl
    }
    pub fn bind_group_layout_owned(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pbr-material-layers"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<astraweave_render::MaterialGpu>() as u64,
                        ),
                    },
                    count: None,
                },
            ],
        })
    }

    pub async fn load(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        biome: &str,
        hot_reload_manager: Option<&mut crate::material_hot_reload::MaterialReloadManager>,
    ) -> Result<&MaterialPackRuntime> {
        if self.cache.contains_key(biome) {
            return Ok(self.cache.get(biome).unwrap());
        }
        // Support both authoring roots; prefer assets/materials over assets/textures when both exist
        let base_materials = PathBuf::from(format!("assets/materials/{biome}"));
        let base_textures = PathBuf::from(format!("assets/textures/{biome}"));
        let (base, mats, arrays) = if base_materials.exists() {
            let mats = base_materials.join("materials.toml");
            let arrays = base_materials.join("arrays.toml");
            (base_materials, mats, arrays)
        } else {
            let mats = base_textures.join("materials.toml");
            let arrays = base_textures.join("arrays.toml");
            (base_textures, mats, arrays)
        };
        let (gpu, stats) = self
            .manager
            .load_pack_from_toml(device, queue, &base, &mats, &arrays)
            .await?;

        // Auto-register materials for hot-reload (optimization: zero-allocation iteration)
        if let Some(reload_mgr) = hot_reload_manager {
            use crate::material_hot_reload::{MaterialArrayIndices, MaterialType};
            
            reload_mgr.register_biome(biome, base.clone());
            reload_mgr.set_current_biome(biome);
            
            // Extract indices from layout (zero-allocation: iterate by reference)
            for (material_name, &array_index) in &gpu.layout.layer_indices {
                let material_id = array_index;
                
                // Optimize: construct path once, avoiding repeated allocations
                let toml_path = base.join(format!("{}.toml", material_name));
                
                let array_indices = MaterialArrayIndices {
                    albedo_index: array_index,
                    normal_index: array_index,
                    orm_index: array_index,
                };
                
                reload_mgr.register_material(
                    material_id,
                    MaterialType::Standard,
                    toml_path,
                    array_indices,
                );
                
                // Register texture paths (cache for fast hot-reload routing)
                // Optimization: check existence once, store result
                let albedo_path = base.join(format!("{}_albedo.png", material_name));
                let normal_path = base.join(format!("{}_normal.png", material_name));
                let orm_path = base.join(format!("{}_orm.png", material_name));
                
                // Only check filesystem if any texture path might exist
                // Optimization: short-circuit evaluation, minimal I/O
                let has_textures = albedo_path.exists() || normal_path.exists() || orm_path.exists();
                
                if has_textures {
                    reload_mgr.update_material_textures(
                        material_id,
                        if albedo_path.exists() { Some(albedo_path) } else { None },
                        if normal_path.exists() { Some(normal_path) } else { None },
                        if orm_path.exists() { Some(orm_path) } else { None },
                    );
                }
            }
            
            println!(
                "[hot-reload] Auto-registered {} materials for biome '{}'",
                gpu.layout.layer_indices.len(),
                biome
            );
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("materials-pack"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gpu.albedo),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&gpu.sampler_albedo),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&gpu.normal),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&gpu.sampler_linear),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&gpu.mra),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(
                        gpu.material_buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
        });
        // Telemetry summary
        println!(
            "[materials] biome={} layers={} | albedo L/S={}/{} | normal L/S={}/{} | mra L+P/S={}+{}/{} | gpu={:.2} MiB",
            stats.biome,
            stats.layers_total,
            stats.albedo_loaded,
            stats.albedo_substituted,
            stats.normal_loaded,
            stats.normal_substituted,
            stats.mra_loaded,
            stats.mra_packed,
            stats.mra_substituted,
            (stats.gpu_memory_bytes as f64) / (1024.0*1024.0)
        );
        let runtime = MaterialPackRuntime {
            gpu,
            stats,
            last_loaded: Instant::now(),
            bind_group,
        };
        self.cache.insert(biome.to_string(), runtime);
        Ok(self.cache.get(biome).unwrap())
    }

    pub fn unload_current(&mut self) {
        // Drop current biome's cached runtime to allow a true reload on next load()
        if let Some(curr) = self.current_biome.take() {
            self.cache.remove(&curr);
        }
        self.manager.unload_current();
    }

    pub fn set_current(&mut self, biome: &str) {
        self.current_biome = Some(biome.to_string());
    }

    pub fn current_layout(&self) -> Option<&ArrayLayout> {
        self.current().map(|r| &r.gpu.layout)
    }

    pub fn current(&self) -> Option<&MaterialPackRuntime> {
        self.current_biome.as_ref().and_then(|b| self.cache.get(b))
    }

    /// Force-reload the specified biome pack, bypassing cache.
    pub async fn reload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        biome: &str,
    ) -> Result<&MaterialPackRuntime> {
        // Remove any existing cached pack for this biome and reset manager
        self.cache.remove(biome);
        self.manager.unload_current();

        // Path resolution (same as in load)
        let base_materials = PathBuf::from(format!("assets/materials/{biome}"));
        let base_textures = PathBuf::from(format!("assets/textures/{biome}"));
        let (base, mats, arrays) = if base_materials.exists() {
            let mats = base_materials.join("materials.toml");
            let arrays = base_materials.join("arrays.toml");
            (base_materials, mats, arrays)
        } else {
            let mats = base_textures.join("materials.toml");
            let arrays = base_textures.join("arrays.toml");
            (base_textures, mats, arrays)
        };

        let (gpu, stats) = self
            .manager
            .load_pack_from_toml(device, queue, &base, &mats, &arrays)
            .await?;

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("materials-pack"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gpu.albedo),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&gpu.sampler_albedo),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&gpu.normal),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&gpu.sampler_linear),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&gpu.mra),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Buffer(
                        gpu.material_buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        println!(
            "[materials] biome={} layers={} | albedo L/S={}/{} | normal L/S={}/{} | mra L+P/S={}+{}/{} | gpu={:.2} MiB",
            stats.biome,
            stats.layers_total,
            stats.albedo_loaded,
            stats.albedo_substituted,
            stats.normal_loaded,
            stats.normal_substituted,
            stats.mra_loaded,
            stats.mra_packed,
            stats.mra_substituted,
            (stats.gpu_memory_bytes as f64) / (1024.0*1024.0)
        );

        let runtime = MaterialPackRuntime {
            gpu,
            stats,
            last_loaded: Instant::now(),
            bind_group,
        };
        self.cache.insert(biome.to_string(), runtime);
        self.set_current(biome);
        Ok(self.cache.get(biome).unwrap())
    }

    /// Get reference to internal MaterialManager (for hot-reload texture access)
    pub fn manager(&self) -> &MaterialManager {
        &self.manager
    }
}
