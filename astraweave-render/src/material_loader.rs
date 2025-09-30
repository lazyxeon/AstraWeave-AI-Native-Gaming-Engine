use std::collections::HashMap;

use anyhow::Result;

use crate::material::{ArrayLayout, MaterialGpuArrays, MaterialLoadStats, MaterialLayerDesc};

pub(crate) mod material_loader_impl {
    use super::*;

    pub fn build_arrays(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layers: &[(String, MaterialLayerDesc)],
        mapping: &HashMap<String, u32>,
        biome_name: &str,
    ) -> Result<(MaterialGpuArrays, MaterialLoadStats, wgpu::Texture, wgpu::Texture, wgpu::Texture)> {
        let width = 1024u32;
        let height = 1024u32;
        let layer_count = mapping.values().max().map(|v| v + 1).unwrap_or(0).max(layers.len() as u32);
        let size = wgpu::Extent3d { width, height, depth_or_array_layers: layer_count };
        let mip_level_count = (32 - (width | height).leading_zeros()) as u32; // approx log2

        fn make_array(device: &wgpu::Device, label: &str, size: wgpu::Extent3d, mips: u32, fmt: wgpu::TextureFormat, samp: &wgpu::SamplerDescriptor) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label), size, mip_level_count: mips, sample_count: 1,
                dimension: wgpu::TextureDimension::D2, format: fmt,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor { label: Some(label), format: Some(fmt), dimension: Some(wgpu::TextureViewDimension::D2Array), aspect: wgpu::TextureAspect::All, base_mip_level: 0, mip_level_count: Some(mips), base_array_layer: 0, array_layer_count: Some(size.depth_or_array_layers) });
            let sampler = device.create_sampler(samp);
            (tex, view, sampler)
        }

        let base_sampler = wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            anisotropy_clamp: 16,
            ..Default::default()
        };

        let (alb_tex, alb_view, samp_alb) = make_array(device, "mat-albedo", size, mip_level_count, wgpu::TextureFormat::Rgba8UnormSrgb, &base_sampler);
        let (nrm_tex, nrm_view, samp_lin) = make_array(device, "mat-normal", size, mip_level_count, wgpu::TextureFormat::Rg8Unorm, &base_sampler);
        let (mra_tex, mra_view, _s) = make_array(device, "mat-mra", size, mip_level_count, wgpu::TextureFormat::Rgba8Unorm, &base_sampler);

        let mut stats = MaterialLoadStats { biome: biome_name.to_string(), ..Default::default() };
        stats.layers_total = layer_count as usize;

        let mut layout = ArrayLayout { layer_indices: HashMap::new(), count: layer_count };

        // For now, write neutral fallbacks; concrete image IO can be added later or delegated to a texture IO module.
        let neutral_albedo = vec![255u8; (width * height * 4) as usize];
        let neutral_normal_rg = vec![128u8; (width * height * 2) as usize]; // xy = 0.5, z reconstructed
        let neutral_mra = {
            let mut v = Vec::with_capacity((width * height * 4) as usize);
            v.resize((width * height * 4) as usize, 0);
            // R=metallic(0), G=roughness(0.5), B=ao(1.0)
            for px in v.chunks_mut(4) { px[0]=0; px[1]=128; px[2]=255; px[3]=255; }
            v
        };

        // Helper to write a whole layer at mip 0
        let write_layer = |tex: &wgpu::Texture, bytes: &[u8], bpr: u32, fmt: wgpu::TextureFormat, layer: u32| {
            let block = wgpu::Extent3d { width, height, depth_or_array_layers: 1 };
            queue.write_texture(
                wgpu::ImageCopyTexture { texture: tex, mip_level: 0, origin: wgpu::Origin3d { x: 0, y: 0, z: layer }, aspect: wgpu::TextureAspect::All },
                bytes,
                wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(bpr), rows_per_image: Some(height) },
                block,
            );
        };

        // Initialize all layers with neutral textures first
        for layer in 0..layer_count {
            write_layer(&alb_tex, &neutral_albedo, width * 4, wgpu::TextureFormat::Rgba8UnormSrgb, layer);
            write_layer(&nrm_tex, &neutral_normal_rg, width * 2, wgpu::TextureFormat::Rg8Unorm, layer);
            write_layer(&mra_tex, &neutral_mra, width * 4, wgpu::TextureFormat::Rgba8Unorm, layer);
        }

        // Record mapping and simple loaded counts; actual image IO and packing can be introduced with crate feature "textures"
        for (key, desc) in layers.iter() {
            if let Some(&idx) = mapping.get(key) {
                layout.layer_indices.insert(key.clone(), idx);
                // Count presence only (IO omitted in this minimal bridge)
                if desc.albedo.is_some() { stats.albedo_loaded += 1; } else { stats.albedo_substituted += 1; }
                if desc.normal.is_some() { stats.normal_loaded += 1; } else { stats.normal_substituted += 1; }
                if desc.mra.is_some() || (desc.metallic.is_some() && desc.roughness.is_some() && desc.ao.is_some()) {
                    if desc.mra.is_none() { stats.mra_packed += 1; }
                    stats.mra_loaded += 1;
                } else { stats.mra_substituted += 1; }
            }
        }

        let gpu = MaterialGpuArrays {
            albedo: alb_view,
            normal: nrm_view,
            mra: mra_view,
            sampler_albedo: samp_alb,
            sampler_linear: samp_lin,
            layout,
        };

        Ok((gpu, stats, alb_tex, nrm_tex, mra_tex))
    }
}
