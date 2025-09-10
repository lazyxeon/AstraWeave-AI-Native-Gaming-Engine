// examples/unified_showcase/src/texture_synth.rs
use image::{ImageBuffer, Rgba};
use std::{fs, path::Path};

/// Public entry: ensure default textures exist; regen if `force` is true.
pub fn ensure_textures(out_dir: &str, seed: u32, force: bool) -> anyhow::Result<()> {
    fs::create_dir_all(out_dir)?;
    synth_if_missing(out_dir, "grass.png", seed.wrapping_add(101), force, synth_grass)?;
    synth_if_missing(out_dir, "dirt.png",  seed.wrapping_add(202), force, synth_dirt)?;
    synth_if_missing(out_dir, "stone.png", seed.wrapping_add(303), force, synth_stone)?;
    Ok(())
}

fn synth_if_missing<F: Fn(u32, u32, u32) -> ImageBuffer<Rgba<u8>, Vec<u8>>>(
    out_dir: &str, name: &str, seed: u32, force: bool, f: F
) -> anyhow::Result<()> {
    let path = Path::new(out_dir).join(name);
    if force || !path.exists() {
        let img = f(1024, 1024, seed);
        img.save(&path)?;
        // also write a normal map derived from the height channel when relevant
        if name.ends_with("grass.png") || name.ends_with("dirt.png") || name.ends_with("stone.png") {
            let npath = Path::new(out_dir).join(name.replace(".png", "_n.png"));
            let normal = height_to_normal(&img, 2.5);
            normal.save(&npath)?;
        }
    }
    Ok(())
}

// -------- Simple fractal noise ----------
fn hash(mut x: u32) -> u32 { 
    x ^= x >> 17; x = x.wrapping_mul(0xed5ad4bb);
    x ^= x >> 11; x = x.wrapping_mul(0xac4c1b51);
    x ^= x >> 15; x = x.wrapping_mul(0x31848bab);
    x ^= x >> 14; x
}

fn noise2d(x: i32, y: i32, seed: u32) -> f32 {
    let h = hash(x as u32 ^ (y as u32).rotate_left(16) ^ seed);
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn fbm(x: f32, y: f32, seed: u32, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    let (mut f, mut a, mut sum, mut norm) = (1.0, 0.5, 0.0, 0.0);
    for i in 0..octaves {
        let n = smooth_noise(x * f, y * f, seed.wrapping_add(i as u32));
        sum += a * n;
        norm += a;
        f *= lacunarity;
        a *= gain;
    }
    sum / norm.max(1e-6)
}

fn smooth_noise(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let xf = x - x0 as f32;
    let yf = y - y0 as f32;

    let n00 = noise2d(x0,   y0,   seed);
    let n10 = noise2d(x0+1, y0,   seed);
    let n01 = noise2d(x0,   y0+1, seed);
    let n11 = noise2d(x0+1, y0+1, seed);

    let sx = xf * xf * (3.0 - 2.0 * xf);
    let sy = yf * yf * (3.0 - 2.0 * yf);
    let ix0 = n00 * (1.0 - sx) + n10 * sx;
    let ix1 = n01 * (1.0 - sx) + n11 * sx;
    ix0 * (1.0 - sy) + ix1 * sy
}

// -------- Materials ----------
fn synth_grass(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 32.0;
            let v = y as f32 / h as f32 * 32.0;
            let base = fbm(u, v, seed, 5, 2.05, 0.52);
            let clump = fbm(u*0.35, v*0.35, seed ^ 0x55aa, 3, 2.0, 0.6);
            // height: rolling grass clumps
            let height = (0.55 + 0.35*base + 0.30*clump).clamp(0.0, 1.0);
            // coloration
            let green = 90.0 + 120.0*height;
            let yellow = 70.0 + 40.0*(1.0-height);
            let (r,g,b) = (
                (green*0.35 + yellow*0.25) as u8,
                (green) as u8,
                (green*0.2) as u8
            );
            img.put_pixel(x, y, Rgba([r,g,b,255]));
        }
    }
    img
}

fn synth_dirt(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 16.0;
            let v = y as f32 / h as f32 * 16.0;
            let grains = fbm(u*1.4, v*1.2, seed ^ 0xdead00, 6, 2.0, 0.5);
            let pebbles = fbm(u*0.35, v*0.35, seed ^ 0xbeef11, 3, 2.0, 0.55);
            let height = (0.5 + 0.4*grains + 0.25*pebbles).clamp(0.0, 1.0);
            let r = (60.0 + 110.0*height) as u8;
            let g = (45.0 + 65.0*height) as u8;
            let b = (35.0 + 45.0*height) as u8;
            img.put_pixel(x, y, Rgba([r,g,b,255]));
        }
    }
    img
}

fn synth_stone(w: u32, h: u32, seed: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32 * 10.0;
            let v = y as f32 / h as f32 * 10.0;
            let veins = fbm(u*1.6, v*1.3, seed ^ 0x7777, 6, 2.1, 0.5);
            let base = fbm(u, v, seed ^ 0x1111, 5, 2.0, 0.55);
            let height = (0.55 + 0.35*base + 0.25*veins).clamp(0.0, 1.0);
            let r = (120.0 + 90.0*height) as u8;
            let g = (120.0 + 90.0*height) as u8;
            let b = (130.0 + 100.0*height) as u8;
            img.put_pixel(x, y, Rgba([r,g,b,255]));
        }
    }
    img
}

// -------- Height → Normal (Sobel) ----------
fn height_to_normal(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, strength: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let (w, h) = img.dimensions();
    let mut out = ImageBuffer::new(w, h);
    let h_sample = |x: i32, y: i32| -> f32 {
        let xi = ((x % w as i32) + w as i32) % w as i32;
        let yi = ((y % h as i32) + h as i32) % h as i32;
        let p = img.get_pixel(xi as u32, yi as u32);
        // luminance as height
        (0.2126*p[0] as f32 + 0.7152*p[1] as f32 + 0.0722*p[2] as f32)/255.0
    };
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let dx = (h_sample(x+1,y) - h_sample(x-1,y)) * strength;
            let dy = (h_sample(x,y+1) - h_sample(x,y-1)) * strength;
            // normal from gradient (Tangent X -> +x, Tangent Y -> +y, Up -> +z)
            let mut nx = -dx; let mut ny = -dy; let mut nz = 1.0;
            let len = (nx*nx + ny*ny + nz*nz).sqrt();
            nx/=len; ny/=len; nz/=len;
            let r = ((nx*0.5+0.5)*255.0) as u8;
            let g = ((ny*0.5+0.5)*255.0) as u8;
            let b = ((nz*0.5+0.5)*255.0) as u8;
            out.put_pixel(x as u32, y as u32, Rgba([r,g,b,255]));
        }
    }
    out
}
