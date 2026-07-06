//! Headless WGSL validation for editor viewport shaders.
//!
//! Editor shaders are embedded via `include_str!` and normally only validated
//! by naga at pipeline creation — i.e., at editor startup, on a GPU machine.
//! A syntax or validation error ships silently until someone launches the
//! editor. This test parses + validates them at `cargo test` time.
//!
//! Regression anchor: the grid shader's missing `@builtin(frag_depth)` output
//! let the fullscreen quad depth-test at z=0 (always passes), so its 85%-opaque
//! ground fill painted over every entity below the eye-level horizon — the
//! "entities only visible at certain angles/heights" bug.

fn validate(name: &str, src: &str) {
    let module = naga::front::wgsl::parse_str(src)
        .unwrap_or_else(|e| panic!("{name}: WGSL parse failed:\n{e}"));
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .unwrap_or_else(|e| panic!("{name}: WGSL validation failed:\n{e:?}"));
}

#[test]
fn grid_shader_is_valid() {
    validate(
        "grid.wgsl",
        include_str!("../src/viewport/shaders/grid.wgsl"),
    );
}

#[test]
fn grid_shader_writes_frag_depth() {
    // The grid renders as a fullscreen quad at raster depth 0.0; without an
    // explicit frag_depth the LessEqual test passes everywhere and the grid
    // fill occludes scene geometry. Guard against regression.
    let src = include_str!("../src/viewport/shaders/grid.wgsl");
    assert!(
        src.contains("@builtin(frag_depth)"),
        "grid.wgsl must write @builtin(frag_depth) — without it the fullscreen \
         quad depth-tests at the near plane and paints over all scene geometry \
         below the eye-level horizon"
    );
}
