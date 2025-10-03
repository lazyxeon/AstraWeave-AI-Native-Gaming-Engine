use anyhow::Result;
use aw_headless::{render_wgsl_to_image, wrap_fs_into_fullscreen};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};

fn main() -> Result<()> {
    // Simple demo: render a gradient
    let fs = r#"@fragment fn fs_main(i: VSOut) -> @location(0) vec4<f32> { return vec4<f32>(i.uv, 0.0, 1.0); }"#;
    let wgsl = wrap_fs_into_fullscreen(fs);
    let img = pollster::block_on(render_wgsl_to_image(&wgsl, 128, 64))?;
    // write PNG to stdout path
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "headless.png".to_string());
    let encoder = PngEncoder::new(std::fs::File::create(&path)?);
    encoder.write_image(&img, 128, 64, ExtendedColorType::Rgba8)?;
    println!("Wrote {}", path);
    Ok(())
}
