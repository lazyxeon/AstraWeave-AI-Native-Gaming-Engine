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
                wgpu::BindGroupLayoutEntry { binding: 0, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2Array, sample_type: wgpu::TextureSampleType::Float { filterable: true } }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2Array, sample_type: wgpu::TextureSampleType::Float { filterable: true } }, count: None },
                wgpu::BindGroupLayoutEntry { binding: 3, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 4, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Texture { multisampled: false, view_dimension: wgpu::TextureViewDimension::D2Array, sample_type: wgpu::TextureSampleType::Float { filterable: true } }, count: None },
            ],
        });
        Self { manager: MaterialManager::new(), bgl, current_biome: None, cache: Default::default() }
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout { &self.bgl }
    pub fn bind_group_layout_owned(&self) -> wgpu::BindGroupLayout { self.bgl.clone() }

    pub async fn load(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, biome: &str) -> Result<&MaterialPackRuntime> {
        if self.cache.contains_key(biome) { return Ok(self.cache.get(biome).unwrap()); }
        let base = PathBuf::from(format!("assets/materials/{biome}"));
        let mats = base.join("materials.toml");
        let arrays = base.join("arrays.toml");
        let (gpu, stats) = self.manager.load_pack_from_toml(device, queue, &base, &mats, &arrays).await?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("materials-pack"),
            layout: &self.bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&gpu.albedo) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&gpu.sampler_albedo) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&gpu.normal) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&gpu.sampler_linear) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&gpu.mra) },
            ],
        });
        let runtime = MaterialPackRuntime { gpu, stats, last_loaded: Instant::now(), bind_group };
        self.cache.insert(biome.to_string(), runtime);
        Ok(self.cache.get(biome).unwrap())
    }

    pub fn unload_current(&mut self) { self.manager.unload_current(); self.current_biome = None; }

    pub fn set_current(&mut self, biome: &str) { self.current_biome = Some(biome.to_string()); }

    pub fn current_layout(&self) -> Option<&ArrayLayout> {
        self.current().map(|r| &r.gpu.layout)
    }

    pub fn current(&self) -> Option<&MaterialPackRuntime> {
        self.current_biome.as_ref().and_then(|b| self.cache.get(b))
    }
}
