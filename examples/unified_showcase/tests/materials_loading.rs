// Use the binary crate's internal modules by declaring a path to main.rs.
// This allows tests to access pub(crate) helpers.
#[path = "../src/main.rs"]
mod main_src;
use main_src::{pack_mra_from_planes, load_biome_toml};

#[test]
fn mra_packing_from_planes() {
    let w = 2u32; let h = 2u32; let px = (w*h*4) as usize;
    let mut m = vec![0u8; px];
    let mut r = vec![0u8; px];
    let mut a = vec![0u8; px];
    // Fill R channels with test pattern: M=10, R=200, AO=255; other channels differ to ensure we pick R channel
    for i in (0..px).step_by(4) {
        m[i+0]=10; m[i+1]=11; m[i+2]=12; m[i+3]=13;
        r[i+0]=200; r[i+1]=201; r[i+2]=202; r[i+3]=203;
        a[i+0]=255; a[i+1]=100; a[i+2]=50; a[i+3]=7;
    }
    let out = pack_mra_from_planes(&m, &r, &a, w, h);
    assert_eq!(out.width(), w);
    assert_eq!(out.height(), h);
    for y in 0..h {
        for x in 0..w {
            let p = out.get_pixel(x, y);
            assert_eq!(p[0], 10, "metallic in R");
            assert_eq!(p[1], 200, "roughness in G");
            assert_eq!(p[2], 255, "ao in B");
            assert_eq!(p[3], 255, "alpha forced to 255");
        }
    }
}

#[test]
fn toml_path_resolution_variants() {
    // Ensure both assets/textures/... and textures/... variants are supported
    let ok1 = load_biome_toml("grassland");
    assert!(ok1.is_some(), "grassland materials.toml must be found");
    let ok2 = load_biome_toml("desert");
    assert!(ok2.is_some(), "desert materials.toml must be found");
}
