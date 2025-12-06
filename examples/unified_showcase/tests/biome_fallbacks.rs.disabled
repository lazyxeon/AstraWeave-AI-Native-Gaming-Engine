#[path = "../src/main.rs"]
mod main_src;
use main_src::stats_for_biome_loading;

#[test]
fn test_synthetic_biome_substitution_counts() {
    // This synthetic biome is located at assets/tests/biome/materials.toml
    // We expect some layers to be missing normal/MRA and thus use fallbacks.
    // Width/height can be small, loading path resizes anyway.
    let result = stats_for_biome_loading("demo", 128, 128).expect("biome load");
    let (total_layers, alb_ok, nor_ok, mra_ok) = result;

    // We don't know MATERIAL_LAYERS contents precisely here, but at least verify
    // we recorded counts without panicking and the totals aren't exceeding layer count.
    assert!(alb_ok <= total_layers);
    assert!(nor_ok <= total_layers);
    assert!(mra_ok <= total_layers);
}

#[test]
fn test_dedicated_test_biome_exact_fallbacks() {
    // This dedicated test biome lives at assets/tests/biome/materials.toml
    // It uses non-matching keys on purpose, so none of the standard MATERIAL_LAYERS
    // will be loaded; this validates that our loader gracefully substitutes fallbacks
    // without panicking and reports exact zero loads.
    // For diagnostics, print whether load_biome_toml can find the TOML and what path it chooses.
    // Initialize logging for tests (no-op if already initialized).
    main_src::init_logging_for_tests();
    let result = stats_for_biome_loading("tests/biome", 64, 64).expect("biome load");
    let (total_layers, alb_ok, nor_ok, mra_ok) = result;
    assert_eq!(
        alb_ok, 0,
        "No albedos should match standard layers in the test biome"
    );
    assert_eq!(
        nor_ok, 0,
        "No normals should match standard layers in the test biome"
    );
    assert_eq!(
        mra_ok, 0,
        "No MRAs should match standard layers in the test biome"
    );
    assert!(
        total_layers > 0,
        "We still report the total number of material layers"
    );
}
