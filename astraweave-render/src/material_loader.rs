use std::{borrow::Cow, collections::HashMap, path::Path};

use anyhow::{Context, Result};

use crate::material::{ArrayLayout, MaterialGpuArrays, MaterialLayerDesc, MaterialLoadStats};

pub(crate) mod material_loader_impl {
    use super::*;

    fn mip_level_count_for(size: wgpu::Extent3d) -> u32 {
        let max_dim = size.width.max(size.height).max(1);
        32 - max_dim.leading_zeros()
    }

    fn generate_mipmaps(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        format: wgpu::TextureFormat,
        mip_levels: u32,
        base_layer: u32,
        layer_count: u32,
    ) {
        if mip_levels <= 1 {
            return;
        }

        const MIPMAP_SHADER: &str = r#"
struct VsOut { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@builtin(vertex_index) vi: u32) -> VsOut {
  var out: VsOut;
  let xy = vec2<f32>(f32(i32(vi) - 1), f32((i32(vi) & 1) * 2 - 1));
  out.pos = vec4<f32>(xy, 0.0, 1.0);
  out.uv = vec2<f32>( (xy.x+1.0)*0.5, 1.0 - (xy.y+1.0)*0.5 );
  return out;
}

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_smp: sampler;

@fragment fn fs(in: VsOut) -> @location(0) vec4<f32> {
  let c = textureSample(src_tex, src_smp, in.uv);
  return c;
}
"#;

        let sm = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mipmap-gen-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(MIPMAP_SHADER)),
        });

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mipmap-bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
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
            ],
        });

        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mipmap-pl"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let rp = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mipmap-pipeline"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &sm,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &sm,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("mipmap-linear-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("mipmap-encoder"),
        });

        for layer in base_layer..(base_layer + layer_count) {
            for level in 1..mip_levels {
                let src_view = texture.create_view(&wgpu::TextureViewDescriptor {
                    usage: None,
                    label: Some("mip-src-view"),
                    format: Some(format),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: level - 1,
                    mip_level_count: Some(1),
                    base_array_layer: layer,
                    array_layer_count: Some(1),
                });

                let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("mipmap-bg"),
                    layout: &bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&src_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                });

                let dst_view = texture.create_view(&wgpu::TextureViewDescriptor {
                    usage: None,
                    label: Some("mip-dst-view"),
                    format: Some(format),
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    base_array_layer: layer,
                    array_layer_count: Some(1),
                });

                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("mipmap-pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &dst_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                rpass.set_pipeline(&rp);
                rpass.set_bind_group(0, &bind, &[]);
                rpass.draw(0..3, 0..1);
                drop(rpass);
            }
        }

        queue.submit(Some(encoder.finish()));
    }

    fn load_rgba(path: &Path) -> Result<image::RgbaImage> {
        let img = image::ImageReader::open(path)
            .with_context(|| format!("open {}", path.display()))?
            .decode()
            .with_context(|| format!("decode {}", path.display()))?;
        Ok(img.to_rgba8())
    }

    fn load_gray(path: &Path) -> Result<image::GrayImage> {
        let img = image::ImageReader::open(path)
            .with_context(|| format!("open {}", path.display()))?
            .decode()
            .with_context(|| format!("decode {}", path.display()))?;
        Ok(img.to_luma8())
    }

    pub fn build_arrays(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layers: &[(String, MaterialLayerDesc)],
        mapping: &HashMap<String, u32>,
        biome_name: &str,
    ) -> Result<(
        MaterialGpuArrays,
        MaterialLoadStats,
        wgpu::Texture,
        wgpu::Texture,
        wgpu::Texture,
    )> {
        let width = 1024u32;
        let height = 1024u32;
        let layer_count = mapping
            .values()
            .max()
            .map(|v| v + 1)
            .unwrap_or(0)
            .max(layers.len() as u32);
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: layer_count,
        };
        let mip_level_count = mip_level_count_for(size);

        fn make_array(
            device: &wgpu::Device,
            label: &str,
            size: wgpu::Extent3d,
            mips: u32,
            fmt: wgpu::TextureFormat,
            samp: &wgpu::SamplerDescriptor,
        ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: mips,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: fmt,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor {
                usage: None,
                label: Some(label),
                format: Some(fmt),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(mips),
                base_array_layer: 0,
                array_layer_count: Some(size.depth_or_array_layers),
            });
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

        let alb_fmt = wgpu::TextureFormat::Rgba8UnormSrgb;
        let nrm_fmt = wgpu::TextureFormat::Rg8Unorm;
        let mra_fmt = wgpu::TextureFormat::Rgba8Unorm;
        let (alb_tex, alb_view, samp_alb) = make_array(
            device,
            "mat-albedo",
            size,
            mip_level_count,
            alb_fmt,
            &base_sampler,
        );
        let (nrm_tex, nrm_view, samp_lin) = make_array(
            device,
            "mat-normal",
            size,
            mip_level_count,
            nrm_fmt,
            &base_sampler,
        );
        let (mra_tex, mra_view, _s) = make_array(
            device,
            "mat-mra",
            size,
            mip_level_count,
            mra_fmt,
            &base_sampler,
        );

        // Diagnostic: print chosen formats and mip count so test runs can validate expectations
        println!("[materials-debug] building arrays: layers={} size={}x{} mips={} formats: albedo={:?} normal={:?} mra={:?}",
            layer_count, width, height, mip_level_count, alb_fmt, nrm_fmt, mra_fmt);

        let mut stats = MaterialLoadStats {
            biome: biome_name.to_string(),
            ..Default::default()
        };
        stats.layers_total = layer_count as usize;

        let mut layout = ArrayLayout {
            layer_indices: HashMap::new(),
            count: layer_count,
        };

        // Neutral patterns used for fallbacks
        let neutral_albedo = vec![255u8; (width * height * 4) as usize];
        let neutral_normal_rg = vec![128u8; (width * height * 2) as usize]; // xy = 0.5
        let neutral_mra = {
            let mut v = vec![0u8; (width * height * 4) as usize];
            for px in v.chunks_mut(4) {
                px[0] = 0;
                px[1] = 128;
                px[2] = 255;
                px[3] = 255;
            }
            v
        };

        // Helper to write a whole layer at mip 0
        let write_layer =
            |tex: &wgpu::Texture, bytes: &[u8], bpr: u32, _fmt: wgpu::TextureFormat, layer: u32| {
                let block = wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                };
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: tex,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    bytes,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(bpr),
                        rows_per_image: Some(height),
                    },
                    block,
                );
            };

        // Start with neutral values for every layer so missing content is defined
        for layer in 0..layer_count {
            write_layer(
                &alb_tex,
                &neutral_albedo,
                width * 4,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                layer,
            );
            write_layer(
                &nrm_tex,
                &neutral_normal_rg,
                width * 2,
                wgpu::TextureFormat::Rg8Unorm,
                layer,
            );
            write_layer(
                &mra_tex,
                &neutral_mra,
                width * 4,
                wgpu::TextureFormat::Rgba8Unorm,
                layer,
            );
        }

        // Load, pack, and upload per-layer
        for (key, desc) in layers.iter() {
            if let Some(&idx) = mapping.get(key) {
                layout.layer_indices.insert(key.clone(), idx);

                // Albedo
                if let Some(ref p) = desc.albedo {
                    match load_rgba(p) {
                        Ok(img) => {
                            let img = image::imageops::resize(
                                &img,
                                width,
                                height,
                                image::imageops::FilterType::Lanczos3,
                            );
                            write_layer(
                                &alb_tex,
                                &img,
                                width * 4,
                                wgpu::TextureFormat::Rgba8UnormSrgb,
                                idx,
                            );
                            stats.albedo_loaded += 1;
                        }
                        Err(e) => {
                            eprintln!("[materials] WARN missing/bad albedo for {}/{}: {} → substituting neutral", biome_name, key, e);
                            stats.albedo_substituted += 1;
                        }
                    }
                } else {
                    // 1x1 policy mentioned → we log and keep the neutral (already written full-res neutral)
                    eprintln!(
                        "[materials] WARN albedo not provided for {}/{} → substituting 1×1 neutral",
                        biome_name, key
                    );
                    stats.albedo_substituted += 1;
                }

                // Normal (RG from XY)
                if let Some(ref p) = desc.normal {
                    match load_rgba(p) {
                        Ok(img_rgba) => {
                            // Extract RG channels
                            let img = image::imageops::resize(
                                &img_rgba,
                                width,
                                height,
                                image::imageops::FilterType::Lanczos3,
                            );
                            let mut rg = vec![0u8; (width * height * 2) as usize];
                            let mut o = 0usize;
                            for px in img.pixels() {
                                rg[o] = px[0];
                                rg[o + 1] = px[1];
                                o += 2;
                            }
                            write_layer(
                                &nrm_tex,
                                &rg,
                                width * 2,
                                wgpu::TextureFormat::Rg8Unorm,
                                idx,
                            );
                            stats.normal_loaded += 1;
                        }
                        Err(e) => {
                            eprintln!("[materials] WARN missing/bad normal for {}/{}: {} → substituting neutral", biome_name, key, e);
                            stats.normal_substituted += 1;
                        }
                    }
                } else {
                    eprintln!(
                        "[materials] WARN normal not provided for {}/{} → substituting neutral",
                        biome_name, key
                    );
                    stats.normal_substituted += 1;
                }

                // MRA
                if let Some(ref p) = desc.mra {
                    match load_rgba(p) {
                        Ok(img) => {
                            let img = image::imageops::resize(
                                &img,
                                width,
                                height,
                                image::imageops::FilterType::Lanczos3,
                            );
                            write_layer(
                                &mra_tex,
                                &img,
                                width * 4,
                                wgpu::TextureFormat::Rgba8Unorm,
                                idx,
                            );
                            stats.mra_loaded += 1;
                        }
                        Err(e) => {
                            eprintln!(
                                "[materials] WARN missing/bad MRA for {}/{}: {}",
                                biome_name, key, e
                            );
                            // Try packing from separate channels
                            if let (Some(m), Some(r), Some(a)) =
                                (&desc.metallic, &desc.roughness, &desc.ao)
                            {
                                match (load_gray(m), load_gray(r), load_gray(a)) {
                                    (Ok(m), Ok(r), Ok(a)) => {
                                        let m = image::imageops::resize(
                                            &m,
                                            width,
                                            height,
                                            image::imageops::FilterType::Lanczos3,
                                        );
                                        let r = image::imageops::resize(
                                            &r,
                                            width,
                                            height,
                                            image::imageops::FilterType::Lanczos3,
                                        );
                                        let a = image::imageops::resize(
                                            &a,
                                            width,
                                            height,
                                            image::imageops::FilterType::Lanczos3,
                                        );
                                        let mut out = vec![0u8; (width * height * 4) as usize];
                                        let mut o = 0usize;
                                        for y in 0..height {
                                            for x in 0..width {
                                                let mi = m.get_pixel(x, y)[0];
                                                let ri = r.get_pixel(x, y)[0];
                                                let ai = a.get_pixel(x, y)[0];
                                                out[o] = mi;
                                                out[o + 1] = ri;
                                                out[o + 2] = ai;
                                                out[o + 3] = 255;
                                                o += 4;
                                            }
                                        }
                                        write_layer(
                                            &mra_tex,
                                            &out,
                                            width * 4,
                                            wgpu::TextureFormat::Rgba8Unorm,
                                            idx,
                                        );
                                        stats.mra_loaded += 1;
                                        stats.mra_packed += 1;
                                    }
                                    _ => {
                                        eprintln!("[materials] WARN cannot pack MRA for {}/{} → substituting neutral", biome_name, key);
                                        stats.mra_substituted += 1;
                                    }
                                }
                            } else {
                                stats.mra_substituted += 1;
                            }
                        }
                    }
                } else if let (Some(m), Some(r), Some(a)) =
                    (&desc.metallic, &desc.roughness, &desc.ao)
                {
                    match (load_gray(m), load_gray(r), load_gray(a)) {
                        (Ok(m), Ok(r), Ok(a)) => {
                            let m = image::imageops::resize(
                                &m,
                                width,
                                height,
                                image::imageops::FilterType::Lanczos3,
                            );
                            let r = image::imageops::resize(
                                &r,
                                width,
                                height,
                                image::imageops::FilterType::Lanczos3,
                            );
                            let a = image::imageops::resize(
                                &a,
                                width,
                                height,
                                image::imageops::FilterType::Lanczos3,
                            );
                            let mut out = vec![0u8; (width * height * 4) as usize];
                            let mut o = 0usize;
                            for y in 0..height {
                                for x in 0..width {
                                    let mi = m.get_pixel(x, y)[0];
                                    let ri = r.get_pixel(x, y)[0];
                                    let ai = a.get_pixel(x, y)[0];
                                    out[o] = mi;
                                    out[o + 1] = ri;
                                    out[o + 2] = ai;
                                    out[o + 3] = 255;
                                    o += 4;
                                }
                            }
                            write_layer(
                                &mra_tex,
                                &out,
                                width * 4,
                                wgpu::TextureFormat::Rgba8Unorm,
                                idx,
                            );
                            stats.mra_loaded += 1;
                            stats.mra_packed += 1;
                        }
                        _ => {
                            eprintln!(
                                "[materials] WARN cannot pack MRA for {}/{} → substituting neutral",
                                biome_name, key
                            );
                            stats.mra_substituted += 1;
                        }
                    }
                } else {
                    eprintln!(
                        "[materials] WARN MRA not provided for {}/{} → substituting neutral",
                        biome_name, key
                    );
                    stats.mra_substituted += 1;
                }
            }
        }

        // Generate mips for all layers
        generate_mipmaps(
            device,
            queue,
            &alb_tex,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            mip_level_count,
            0,
            layer_count,
        );
        generate_mipmaps(
            device,
            queue,
            &nrm_tex,
            wgpu::TextureFormat::Rg8Unorm,
            mip_level_count,
            0,
            layer_count,
        );
        generate_mipmaps(
            device,
            queue,
            &mra_tex,
            wgpu::TextureFormat::Rgba8Unorm,
            mip_level_count,
            0,
            layer_count,
        );

        // GPU memory estimate (sum of mips for each texture)
        let sum_mips = |bpp: u32| -> u64 {
            let mut total: u64 = 0;
            let mut w = width as u64;
            let mut h = height as u64;
            for _ in 0..mip_level_count {
                total += w.max(1) * h.max(1) * bpp as u64 * layer_count as u64;
                w = (w / 2).max(1);
                h = (h / 2).max(1);
            }
            total
        };
        stats.gpu_memory_bytes = sum_mips(4) + sum_mips(2) + sum_mips(4);

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
