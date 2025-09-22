use anyhow::Result;
#[cfg(feature = "textures")]
use image::GenericImageView;
use std::path::Path;

/// A loaded texture with its GPU resources
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    /// Create a 1x1 white texture as a default/fallback
    pub fn create_default_white(device: &wgpu::Device, queue: &wgpu::Queue, label: &str) -> Result<Self> {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255], // RGBA white
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    /// Create a 1x1 normal map texture pointing upward (0, 0, 1)
    pub fn create_default_normal(device: &wgpu::Device, queue: &wgpu::Queue, label: &str) -> Result<Self> {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[128, 128, 255, 255], // Normal pointing up: (0, 0, 1) in normal map encoding
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }

    /// Load a texture from a file (requires "textures" feature)
    #[cfg(feature = "textures")]
    pub fn from_file(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        path: &Path,
    ) -> Result<Self> {
        println!("Loading texture from: {}", path.display());
        
        if !path.exists() {
            return Err(anyhow::anyhow!("Texture file not found: {}", path.display()));
        }

        let bytes = std::fs::read(path)?;
        Self::from_bytes(device, queue, &bytes, &path.to_string_lossy())
    }

    /// Load a texture from byte data (requires "textures" feature)
    #[cfg(feature = "textures")]
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        println!("Loaded texture '{}': {}x{} pixels", label, dimensions.0, dimensions.1);

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
        })
    }
}

/// Validate that texture files exist and can be loaded
#[cfg(feature = "textures")]
pub fn validate_texture_assets(asset_paths: &[&str]) -> Result<()> {
    println!("🎨 Validating texture assets...");
    
    let mut valid_count = 0;
    
    for texture_path in asset_paths {
        if std::path::Path::new(texture_path).exists() {
            match image::open(texture_path) {
                Ok(img) => {
                    let (w, h) = img.dimensions();
                    println!("  ✅ {}: {}x{} pixels", texture_path, w, h);
                    valid_count += 1;
                }
                Err(e) => {
                    println!("  ❌ {}: Failed to load - {}", texture_path, e);
                }
            }
        } else {
            println!("  ❌ {}: File not found", texture_path);
        }
    }
    
    println!("📊 Texture validation: {}/{} textures valid", valid_count, asset_paths.len());
    
    if valid_count > 0 {
        println!("✅ Found valid textures for rendering!");
        Ok(())
    } else {
        Err(anyhow::anyhow!("No valid textures found"))
    }
}