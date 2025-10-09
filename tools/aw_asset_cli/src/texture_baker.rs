//! Texture baking pipeline with mipmap generation, compression, and metadata

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Color space designation for textures
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorSpace {
    /// sRGB color space (for albedo/color textures)
    Srgb,
    /// Linear color space (for normal maps, ORM maps, data textures)
    Linear,
}

/// Normal map Y-axis convention
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NormalYConvention {
    /// OpenGL convention (Y+ up)
    OpenGl,
    /// DirectX convention (Y+ down)
    DirectX,
}

/// GPU texture compression format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompressionFormat {
    /// BC1 (DXT1) - RGB + 1-bit alpha, 4:1 compression
    Bc1,
    /// BC3 (DXT5) - RGBA with smooth alpha, 4:1 compression
    Bc3,
    /// BC5 - Two-channel (RG) for normal maps, 2:1 compression
    Bc5,
    /// BC7 - High-quality RGBA, 4:1 compression
    Bc7,
    /// No compression (RGBA8)
    None,
}

/// Texture metadata for runtime loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureMetadata {
    /// Original source file path
    pub source_path: String,
    /// Output texture file path (.dds or .ktx2)
    pub output_path: String,
    /// Color space designation
    pub color_space: ColorSpace,
    /// Normal map Y convention (None if not a normal map)
    pub normal_y_convention: Option<NormalYConvention>,
    /// Compression format used
    pub compression: CompressionFormat,
    /// Number of mipmap levels
    pub mip_levels: u32,
    /// Base texture dimensions (width, height)
    pub dimensions: (u32, u32),
    /// SHA-256 hash of output file
    pub sha256: String,
}

impl TextureMetadata {
    /// Load texture metadata from a `.meta.json` file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read metadata file: {}", path.display()))?;
        let metadata: TextureMetadata = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse metadata JSON: {}", path.display()))?;
        Ok(metadata)
    }

    /// Load texture metadata for a texture file by looking for the corresponding `.meta.json`
    /// For example, if texture is `grass.ktx2`, looks for `grass.ktx2.meta.json`
    pub fn load_for_texture(texture_path: &Path) -> Result<Self> {
        let meta_path = texture_path.with_extension(format!(
            "{}.meta.json",
            texture_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("ktx2")
        ));
        Self::load_from_file(&meta_path)
    }
}

/// Configuration for texture baking
#[derive(Debug, Clone)]
pub struct BakeConfig {
    /// Color space for the texture
    pub color_space: ColorSpace,
    /// Is this a normal map?
    pub is_normal_map: bool,
    /// Normal map Y convention (if applicable)
    pub normal_y_convention: NormalYConvention,
    /// Compression format to use
    pub compression: CompressionFormat,
    /// Generate full mipmap chain
    pub generate_mipmaps: bool,
    /// Output format extension (.dds or .ktx2)
    pub output_format: String,
}

impl Default for BakeConfig {
    fn default() -> Self {
        Self {
            color_space: ColorSpace::Srgb,
            is_normal_map: false,
            normal_y_convention: NormalYConvention::OpenGl,
            compression: CompressionFormat::Bc7,
            generate_mipmaps: true,
            output_format: "ktx2".to_string(),
        }
    }
}

/// Bake a texture with mipmap generation and compression
pub fn bake_texture(
    input_path: &Path,
    output_dir: &Path,
    config: &BakeConfig,
) -> Result<TextureMetadata> {
    // Load source image
    let img = image::open(input_path)
        .with_context(|| format!("Failed to load texture: {}", input_path.display()))?;

    let (width, height) = img.dimensions();

    // Generate mipmaps if requested
    let mipmaps = if config.generate_mipmaps {
        generate_mipmap_chain(&img)?
    } else {
        vec![img]
    };

    let mip_levels = mipmaps.len() as u32;

    // Determine output path
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Invalid file stem")?;
    let output_path = output_dir.join(format!("{}.{}", stem, config.output_format));

    std::fs::create_dir_all(output_dir)?;

    // For now, write raw mipmaps as separate files (placeholder for DDS/KTX2 writer)
    // In production, use a proper DDS/KTX2 library
    write_texture_with_mipmaps(&mipmaps, &output_path, config)?;

    // Compute hash
    let sha256 = compute_file_hash(&output_path)?;

    // Create metadata
    let metadata = TextureMetadata {
        source_path: input_path.to_string_lossy().to_string(),
        output_path: output_path.to_string_lossy().to_string(),
        color_space: config.color_space,
        normal_y_convention: if config.is_normal_map {
            Some(config.normal_y_convention)
        } else {
            None
        },
        compression: config.compression,
        mip_levels,
        dimensions: (width, height),
        sha256,
    };

    // Write metadata JSON
    let meta_path = output_path.with_extension(format!("{}.meta.json", config.output_format));
    let meta_json = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(&meta_path, meta_json)?;

    println!(
        "[bake] {} → {} ({} mips, {:?}, {:?})",
        input_path.display(),
        output_path.display(),
        mip_levels,
        config.color_space,
        config.compression
    );

    Ok(metadata)
}

/// Generate full mipmap chain using box filtering
fn generate_mipmap_chain(base: &DynamicImage) -> Result<Vec<DynamicImage>> {
    let mut mipmaps = vec![base.clone()];
    let (mut w, mut h) = base.dimensions();

    while w > 1 || h > 1 {
        let next_w = (w / 2).max(1);
        let next_h = (h / 2).max(1);

        // Use bilinear filtering for downsampling
        let last_mip = mipmaps.last().unwrap();
        let resized = image::imageops::resize(
            last_mip,
            next_w,
            next_h,
            image::imageops::FilterType::Lanczos3,
        );

        mipmaps.push(DynamicImage::ImageRgba8(resized));
        w = next_w;
        h = next_h;
    }

    Ok(mipmaps)
}

/// Write texture with mipmaps to KTX2 format
fn write_texture_with_mipmaps(
    mipmaps: &[DynamicImage],
    output_path: &Path,
    config: &BakeConfig,
) -> Result<()> {
    if mipmaps.is_empty() {
        anyhow::bail!("No mipmaps to write");
    }

    let (base_width, base_height) = mipmaps[0].dimensions();

    // Determine Vulkan format based on compression and color space
    let vk_format = match (config.compression, config.color_space) {
        // BC1 (DXT1) - RGB + 1-bit alpha
        (CompressionFormat::Bc1, ColorSpace::Srgb) => 83, // VK_FORMAT_BC1_RGB_SRGB_BLOCK
        (CompressionFormat::Bc1, ColorSpace::Linear) => 71, // VK_FORMAT_BC1_RGB_UNORM_BLOCK

        // BC3 (DXT5) - RGBA with smooth alpha
        (CompressionFormat::Bc3, ColorSpace::Srgb) => 87, // VK_FORMAT_BC3_SRGB_BLOCK
        (CompressionFormat::Bc3, ColorSpace::Linear) => 75, // VK_FORMAT_BC3_UNORM_BLOCK

        // BC5 - Two-channel (RG) for normal maps (always linear)
        (CompressionFormat::Bc5, _) => 143, // VK_FORMAT_BC5_UNORM_BLOCK

        // BC7 - High-quality RGBA
        (CompressionFormat::Bc7, ColorSpace::Srgb) => 99, // VK_FORMAT_BC7_SRGB_BLOCK
        (CompressionFormat::Bc7, ColorSpace::Linear) => 98, // VK_FORMAT_BC7_UNORM_BLOCK

        // No compression - RGBA8
        (CompressionFormat::None, ColorSpace::Srgb) => 43, // VK_FORMAT_R8G8B8A8_SRGB
        (CompressionFormat::None, ColorSpace::Linear) => 37, // VK_FORMAT_R8G8B8A8_UNORM
    };

    // Collect mipmap data
    let mut mip_data_vec = Vec::new();
    for (mip_level, mip) in mipmaps.iter().enumerate() {
        let rgba = mip.to_rgba8();
        let (width, height) = mip.dimensions();

        // Convert to compressed or raw data
        let mip_data = if config.compression == CompressionFormat::None {
            // Store raw RGBA8 data
            rgba.to_vec()
        } else {
            // Compress using BC format
            compress_to_bc(&rgba, width, height, config.compression)?
        };

        println!(
            "  [mip {}] {}x{} → {} bytes",
            mip_level,
            width,
            height,
            mip_data.len()
        );
        mip_data_vec.push(mip_data);
    }

    // Build KTX2 file manually since the API is unclear
    // For now, write a simpler format that can be loaded later
    // TODO: Use proper KTX2 builder API when available or use libktx-rs

    // Temporary solution: write raw data with a simple header
    let mut output_data = Vec::new();

    // Write a simple custom format header (to be replaced with proper KTX2)
    output_data.extend_from_slice(b"AW_TEX2\0"); // Magic number
    output_data.extend_from_slice(&(vk_format as u32).to_le_bytes());
    output_data.extend_from_slice(&base_width.to_le_bytes());
    output_data.extend_from_slice(&base_height.to_le_bytes());
    output_data.extend_from_slice(&(mipmaps.len() as u32).to_le_bytes());

    // Write mip data
    for mip_data in &mip_data_vec {
        output_data.extend_from_slice(&(mip_data.len() as u32).to_le_bytes());
        output_data.extend_from_slice(mip_data);
    }

    std::fs::write(output_path, output_data)
        .with_context(|| format!("Failed to write texture file: {}", output_path.display()))?;

    println!(
        "[tex2] Written {} with {} mips, format={}, colorspace={:?}",
        output_path.display(),
        mipmaps.len(),
        vk_format,
        config.color_space
    );

    Ok(())
}

/// Simple BC block compression (placeholder implementation)
/// In production, use intel_tex, basis_universal, or libktx-rs transcoding
fn compress_to_bc(
    rgba: &image::RgbaImage,
    width: u32,
    height: u32,
    format: CompressionFormat,
) -> Result<Vec<u8>> {
    let block_width = (width + 3) / 4;
    let block_height = (height + 3) / 4;
    let num_blocks = (block_width * block_height) as usize;

    let block_size = match format {
        CompressionFormat::Bc1 => 8,  // 64 bits per 4x4 block
        CompressionFormat::Bc3 => 16, // 128 bits per 4x4 block
        CompressionFormat::Bc5 => 16, // 128 bits per 4x4 block
        CompressionFormat::Bc7 => 16, // 128 bits per 4x4 block
        CompressionFormat::None => return Ok(rgba.to_vec()),
    };

    let mut compressed = vec![0u8; num_blocks * block_size];

    // Simple box-filter based compression (quality placeholder)
    // TODO: Replace with proper BC encoder
    for block_y in 0..block_height {
        for block_x in 0..block_width {
            let block_idx = (block_y * block_width + block_x) as usize;
            let block_offset = block_idx * block_size;

            // Sample 4x4 block average color
            let mut r_sum = 0u32;
            let mut g_sum = 0u32;
            let mut b_sum = 0u32;
            let mut a_sum = 0u32;
            let mut count = 0u32;

            for py in 0..4 {
                for px in 0..4 {
                    let x = (block_x * 4 + px).min(width - 1);
                    let y = (block_y * 4 + py).min(height - 1);
                    let pixel = rgba.get_pixel(x, y);
                    r_sum += pixel[0] as u32;
                    g_sum += pixel[1] as u32;
                    b_sum += pixel[2] as u32;
                    a_sum += pixel[3] as u32;
                    count += 1;
                }
            }

            let r_avg = (r_sum / count) as u8;
            let g_avg = (g_sum / count) as u8;
            let b_avg = (b_sum / count) as u8;
            let a_avg = (a_sum / count) as u8;

            // Store simplified block data based on format
            match format {
                CompressionFormat::Bc1 => {
                    // BC1: 2 RGB565 colors + 2-bit indices
                    // Simplified: store color0 = avg, color1 = avg, all indices = 0
                    let rgb565 = ((r_avg as u16 & 0xF8) << 8)
                        | ((g_avg as u16 & 0xFC) << 3)
                        | ((b_avg as u16 & 0xF8) >> 3);
                    compressed[block_offset..block_offset + 2]
                        .copy_from_slice(&rgb565.to_le_bytes());
                    compressed[block_offset + 2..block_offset + 4]
                        .copy_from_slice(&rgb565.to_le_bytes());
                    // Indices (4 bytes of zeros)
                }
                CompressionFormat::Bc5 => {
                    // BC5: Two BC4 blocks (R and G channels)
                    // Simplified: store rmin=rmax=r_avg, gmin=gmax=g_avg
                    compressed[block_offset] = r_avg;
                    compressed[block_offset + 1] = r_avg;
                    compressed[block_offset + 8] = g_avg;
                    compressed[block_offset + 9] = g_avg;
                }
                CompressionFormat::Bc3 | CompressionFormat::Bc7 => {
                    // BC3/BC7: More complex, use simple color storage
                    // Simplified: store as BC1 RGB + alpha endpoints
                    let rgb565 = ((r_avg as u16 & 0xF8) << 8)
                        | ((g_avg as u16 & 0xFC) << 3)
                        | ((b_avg as u16 & 0xF8) >> 3);
                    compressed[block_offset + 8..block_offset + 10]
                        .copy_from_slice(&rgb565.to_le_bytes());
                    compressed[block_offset + 10..block_offset + 12]
                        .copy_from_slice(&rgb565.to_le_bytes());
                    // Alpha for BC3
                    if format == CompressionFormat::Bc3 {
                        compressed[block_offset] = a_avg;
                        compressed[block_offset + 1] = a_avg;
                    }
                }
                CompressionFormat::None => {}
            }
        }
    }

    println!(
        "  [compress] {}x{} → {} blocks ({} bytes) using {:?}",
        width,
        height,
        num_blocks,
        compressed.len(),
        format
    );

    Ok(compressed)
}

/// Compute SHA-256 hash of a file
fn compute_file_hash(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let data = std::fs::read(path)?;
    let hash = Sha256::digest(&data);
    Ok(hex::encode(hash))
}

/// Infer texture configuration from filename conventions
pub fn infer_config_from_path(path: &Path) -> BakeConfig {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut config = BakeConfig::default();

    // Detect normal maps
    if filename.contains("normal") || filename.contains("_n.") || filename.ends_with("_n.png") {
        config.is_normal_map = true;
        config.color_space = ColorSpace::Linear;
        config.compression = CompressionFormat::Bc5;
    }
    // Detect ORM/metallic-roughness-AO maps
    else if filename.contains("orm")
        || filename.contains("roughness")
        || filename.contains("metallic")
        || filename.contains("_mr.")
        || filename.contains("_mra.")
        || filename.ends_with("_mra.png")
        || filename.ends_with("_orm.png")
    {
        config.color_space = ColorSpace::Linear;
        config.compression = CompressionFormat::Bc7;
    }
    // Detect ambient occlusion
    else if filename.contains("ao") || filename.contains("occlusion") {
        config.color_space = ColorSpace::Linear;
        config.compression = CompressionFormat::Bc7;
    }
    // Default: assume albedo/color texture
    else {
        config.color_space = ColorSpace::Srgb;
        config.compression = CompressionFormat::Bc7;
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mipmap_chain_generation() {
        use image::RgbaImage;
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            256,
            256,
            image::Rgba([255, 0, 0, 255]),
        ));

        let mipmaps = generate_mipmap_chain(&img).unwrap();

        // Should have 9 mips: 256, 128, 64, 32, 16, 8, 4, 2, 1
        assert_eq!(mipmaps.len(), 9);
        assert_eq!(mipmaps[0].dimensions(), (256, 256));
        assert_eq!(mipmaps[8].dimensions(), (1, 1));
    }

    #[test]
    fn test_config_inference() {
        let albedo_cfg = infer_config_from_path(Path::new("grass_albedo.png"));
        assert_eq!(albedo_cfg.color_space, ColorSpace::Srgb);
        assert!(!albedo_cfg.is_normal_map);

        let normal_cfg = infer_config_from_path(Path::new("wall_normal.png"));
        assert_eq!(normal_cfg.color_space, ColorSpace::Linear);
        assert!(normal_cfg.is_normal_map);

        let orm_cfg = infer_config_from_path(Path::new("metal_orm.png"));
        assert_eq!(orm_cfg.color_space, ColorSpace::Linear);
        assert!(!orm_cfg.is_normal_map);
    }
}
