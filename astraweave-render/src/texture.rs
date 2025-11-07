use anyhow::Result;
#[cfg(feature = "textures")]
use image::GenericImageView;
#[cfg(feature = "textures")]
use std::path::Path;

/// A loaded texture with its GPU resources
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    /// Create a 1x1 white texture as a default/fallback
    pub fn create_default_white(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
    ) -> Result<Self> {
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
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255], // RGBA white
            wgpu::TexelCopyBufferLayout {
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
    pub fn create_default_normal(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        label: &str,
    ) -> Result<Self> {
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
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[128, 128, 255, 255], // Normal pointing up: (0, 0, 1) in normal map encoding
            wgpu::TexelCopyBufferLayout {
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
    pub fn from_file(device: &wgpu::Device, queue: &wgpu::Queue, path: &Path) -> Result<Self> {
        println!("Loading texture from: {}", path.display());

        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Texture file not found: {}",
                path.display()
            ));
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

        println!(
            "Loaded texture '{}': {}x{} pixels",
            label, dimensions.0, dimensions.1
        );

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
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
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

    println!(
        "📊 Texture validation: {}/{} textures valid",
        valid_count,
        asset_paths.len()
    );

    if valid_count > 0 {
        println!("✅ Found valid textures for rendering!");
        Ok(())
    } else {
        Err(anyhow::anyhow!("No valid textures found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_device() -> (wgpu::Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: true,
                compatible_surface: None,
            })
            .await
            .expect("Failed to find adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("test_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: Default::default(),
            })
            .await
            .expect("Failed to create device")
    }

    #[test]
    fn test_create_default_white() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let result = Texture::create_default_white(&device, &queue, "test_white");

            assert!(result.is_ok(), "Should create white texture successfully");
            let texture = result.unwrap();

            // Verify texture properties
            assert_eq!(texture.texture.size().width, 1);
            assert_eq!(texture.texture.size().height, 1);
            assert_eq!(
                texture.texture.format(),
                wgpu::TextureFormat::Rgba8UnormSrgb
            );
        });
    }

    #[test]
    fn test_create_default_normal() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let result = Texture::create_default_normal(&device, &queue, "test_normal");

            assert!(result.is_ok(), "Should create normal texture successfully");
            let texture = result.unwrap();

            // Verify texture properties
            assert_eq!(texture.texture.size().width, 1);
            assert_eq!(texture.texture.size().height, 1);
            assert_eq!(
                texture.texture.format(),
                wgpu::TextureFormat::Rgba8UnormSrgb
            );
        });
    }

    #[test]
    fn test_white_and_normal_different_labels() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;

            let white = Texture::create_default_white(&device, &queue, "white").unwrap();
            let normal = Texture::create_default_normal(&device, &queue, "normal").unwrap();

            // Both should be 1x1 textures
            assert_eq!(white.texture.size().width, 1);
            assert_eq!(normal.texture.size().width, 1);
        });
    }

    #[cfg(feature = "textures")]
    #[test]
    fn test_from_bytes_valid_png() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;

            // Create a simple 2x2 PNG in memory (red pixel)
            let png_data = vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
                0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
                0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02, // 2x2 dimensions
                0x08, 0x02, 0x00, 0x00, 0x00, 0xFD, 0xD4, 0x9A, // RGB, no interlace
                0x73, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT chunk
                0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, // Compressed data
                0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
                0x45, 0x4E, // IEND
                0x44, 0xAE, 0x42, 0x60, 0x82,
            ];

            let result = Texture::from_bytes(&device, &queue, &png_data, "test_png");
            assert!(result.is_ok(), "Should load PNG from bytes");
        });
    }

    #[cfg(feature = "textures")]
    #[test]
    fn test_from_bytes_invalid_data() {
        pollster::block_on(async {
            let (device, queue) = create_test_device().await;
            let invalid_data = vec![0, 1, 2, 3, 4, 5]; // Not a valid image

            let result = Texture::from_bytes(&device, &queue, &invalid_data, "invalid");
            assert!(result.is_err(), "Should fail on invalid image data");
        });
    }

    #[cfg(feature = "textures")]
    #[test]
    fn test_validate_texture_assets_empty() {
        let result = validate_texture_assets(&[]);
        assert!(result.is_err(), "Empty asset list should fail validation");
    }

    #[cfg(feature = "textures")]
    #[test]
    fn test_validate_texture_assets_nonexistent() {
        let result = validate_texture_assets(&["nonexistent_file.png"]);
        assert!(result.is_err(), "Nonexistent file should fail validation");
    }
}
