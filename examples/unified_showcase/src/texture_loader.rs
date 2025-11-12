/// Texture Loading System with Proper Format Handling
/// Phase 1.1: TextureUsage-based loading with correct sRGB/Linear format selection
/// and automatic mipmap generation

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

/// Defines how a texture should be interpreted for correct color space handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureUsage {
    /// Color/albedo textures - stored in sRGB, GPU converts to linear
    Albedo,
    /// Normal maps - stored in linear space, no gamma correction
    Normal,
    /// Metallic/Roughness/AO packed texture - linear space
    MRA,
    /// Emissive/glow textures - sRGB space
    Emissive,
    /// Height/displacement maps - linear space
    Height,
}

impl TextureUsage {
    /// Returns the correct wgpu texture format for this usage
    pub fn format(&self) -> wgpu::TextureFormat {
        match self {
            Self::Albedo | Self::Emissive => wgpu::TextureFormat::Rgba8UnormSrgb,
            Self::Normal | Self::MRA | Self::Height => wgpu::TextureFormat::Rgba8Unorm,
        }
    }

    /// Returns whether this texture type should have mipmaps generated
    pub fn needs_mipmaps(&self) -> bool {
        match self {
            Self::Albedo | Self::Emissive | Self::MRA => true,
            Self::Normal | Self::Height => false, // Normal maps can have artifacts with mip blending
        }
    }

    /// Returns a human-readable description of this usage
    pub fn description(&self) -> &'static str {
        match self {
            Self::Albedo => "Albedo (sRGB color)",
            Self::Normal => "Normal Map (linear RGB)",
            Self::MRA => "Metallic/Roughness/AO (linear)",
            Self::Emissive => "Emissive (sRGB color)",
            Self::Height => "Height/Displacement (linear)",
        }
    }
}

/// Load a texture from a file path with proper format and mipmap handling
pub fn load_texture_with_usage(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &str,
    usage: TextureUsage,
) -> Result<wgpu::Texture> {
    // Load image
    let img = image::open(path)
        .with_context(|| format!("Failed to load texture: {}", path))?;
    
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();

    // Calculate mip levels
    let mip_levels = if usage.needs_mipmaps() {
        calculate_mip_levels(width, height)
    } else {
        1
    };

    log::info!(
        "📦 Loading texture '{}' ({}x{}) as {} with {} mip levels",
        path,
        width,
        height,
        usage.description(),
        mip_levels
    );

    // Create texture
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(path),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: mip_levels,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: usage.format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Upload base mip level (level 0)
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
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    // Generate and upload mipmaps
    if mip_levels > 1 {
        generate_and_upload_mipmaps(device, queue, &texture, &rgba, mip_levels)?;
    }

    Ok(texture)
}

/// Calculate the number of mip levels for a texture
fn calculate_mip_levels(width: u32, height: u32) -> u32 {
    let max_dimension = width.max(height) as f32;
    (max_dimension.log2().floor() as u32 + 1).max(1)
}

/// Generate mipmaps using CPU downsampling and upload to GPU
fn generate_and_upload_mipmaps(
    _device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    base_image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    mip_levels: u32,
) -> Result<()> {
    let mut current_image = DynamicImage::ImageRgba8(base_image.clone());

    for level in 1..mip_levels {
        // Calculate mip dimensions
        let mip_width = (base_image.width() >> level).max(1);
        let mip_height = (base_image.height() >> level).max(1);

        // Downsample using high-quality filter
        current_image = current_image.resize(
            mip_width,
            mip_height,
            image::imageops::FilterType::Lanczos3,
        );

        let rgba_mip = current_image.to_rgba8();

        // Upload mip level
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba_mip,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * mip_width),
                rows_per_image: Some(mip_height),
            },
            wgpu::Extent3d {
                width: mip_width,
                height: mip_height,
                depth_or_array_layers: 1,
            },
        );
    }

    Ok(())
}

/// Create a fallback texture for a specific usage type
pub fn create_fallback_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    usage: TextureUsage,
) -> wgpu::Texture {
    let (width, height) = (16, 16); // Small fallback texture

    let data: Vec<u8> = match usage {
        TextureUsage::Albedo => {
            // Magenta checkerboard (missing texture indicator)
            create_checkerboard(width, height, [255, 0, 255, 255], [128, 0, 128, 255])
        }
        TextureUsage::Normal => {
            // Flat normal (pointing up in tangent space)
            vec![128, 128, 255, 255].repeat((width * height) as usize)
        }
        TextureUsage::MRA => {
            // Default: non-metallic, medium roughness, full AO
            vec![0, 128, 0, 255].repeat((width * height) as usize)
        }
        TextureUsage::Emissive => {
            // Black (no emission)
            vec![0, 0, 0, 255].repeat((width * height) as usize)
        }
        TextureUsage::Height => {
            // Flat height (0.5 normalized)
            vec![128, 128, 128, 255].repeat((width * height) as usize)
        }
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("Fallback {}", usage.description())),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: usage.format(),
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
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

    texture
}

/// Generate a solid color fallback texture
pub fn generate_fallback_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    color: [f32; 4],
) -> wgpu::Texture {
    let (width, height) = (16, 16);
    
    let color_u8 = [
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * 255.0) as u8,
    ];
    
    let data = vec![color_u8[0], color_u8[1], color_u8[2], color_u8[3]]
        .repeat((width * height) as usize);
    
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Fallback Solid Color"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
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
    
    texture
}

/// Create a texture 2D array from multiple textures
/// All input textures will be resized to the same dimensions (largest width/height)
/// Returns (texture, texture_view) where view is configured for array sampling
pub fn create_texture_array(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    textures: &[wgpu::Texture],
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    if textures.is_empty() {
        panic!("Cannot create texture array from empty texture list");
    }

    // Determine target dimensions (use largest texture size)
    let mut max_width = 0;
    let mut max_height = 0;
    let mut format = wgpu::TextureFormat::Rgba8UnormSrgb;

    for texture in textures {
        let size = texture.size();
        max_width = max_width.max(size.width);
        max_height = max_height.max(size.height);
        format = texture.format();
    }

    log::info!(
        "📦 Creating texture array '{}' with {} layers at {}×{}",
        label,
        textures.len(),
        max_width,
        max_height
    );

    // Create texture array
    let array_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: max_width,
            height: max_height,
            depth_or_array_layers: textures.len() as u32,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    // Copy each input texture to a layer in the array
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Texture Array Copy Encoder"),
    });

    for (layer_index, texture) in textures.iter().enumerate() {
        let src_size = texture.size();
        
        // Copy texture to array layer
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &array_texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer_index as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: src_size.width.min(max_width),
                height: src_size.height.min(max_height),
                depth_or_array_layers: 1,
            },
        );
    }

    queue.submit(std::iter::once(encoder.finish()));

    // Create texture view for array sampling
    let array_view = array_texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(&format!("{} View", label)),
        format: Some(format),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: Some(textures.len() as u32),
        usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
    });

    (array_texture, array_view)
}

/// Create a checkerboard pattern (useful for debugging UVs)
fn create_checkerboard(
    width: u32,
    height: u32,
    color1: [u8; 4],
    color2: [u8; 4],
) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);
    let checker_size = 2; // 2x2 pixel checkers

    for y in 0..height {
        for x in 0..width {
            let checker_x = (x / checker_size) % 2;
            let checker_y = (y / checker_size) % 2;
            let color = if (checker_x + checker_y) % 2 == 0 {
                color1
            } else {
                color2
            };
            data.extend_from_slice(&color);
        }
    }

    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_usage_formats() {
        assert_eq!(
            TextureUsage::Albedo.format(),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
        assert_eq!(
            TextureUsage::Normal.format(),
            wgpu::TextureFormat::Rgba8Unorm
        );
        assert_eq!(TextureUsage::MRA.format(), wgpu::TextureFormat::Rgba8Unorm);
        assert_eq!(
            TextureUsage::Emissive.format(),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
    }

    #[test]
    fn test_mip_level_calculation() {
        assert_eq!(calculate_mip_levels(1024, 1024), 11); // log2(1024) + 1 = 11
        assert_eq!(calculate_mip_levels(512, 512), 10);
        assert_eq!(calculate_mip_levels(256, 256), 9);
        assert_eq!(calculate_mip_levels(1, 1), 1);
        assert_eq!(calculate_mip_levels(1920, 1080), 11); // log2(1920) + 1
    }

    #[test]
    fn test_mipmap_needs() {
        assert!(TextureUsage::Albedo.needs_mipmaps());
        assert!(!TextureUsage::Normal.needs_mipmaps());
        assert!(TextureUsage::MRA.needs_mipmaps());
        assert!(TextureUsage::Emissive.needs_mipmaps());
    }

    #[test]
    fn test_checkerboard_generation() {
        let data = create_checkerboard(4, 4, [255, 0, 0, 255], [0, 0, 255, 255]);
        assert_eq!(data.len(), 4 * 4 * 4); // 4x4 pixels, 4 bytes per pixel

        // Check first pixel is red
        assert_eq!(&data[0..4], &[255, 0, 0, 255]);
    }
}
