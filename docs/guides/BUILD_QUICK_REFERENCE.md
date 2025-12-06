# Veilweaver Build Quick Reference

## Daily Development

```bash
# Quick core build (debug)
cargo build -p astraweave-core -p astraweave-ai -p hello_companion

# Quick check all working crates
cargo check --workspace --exclude astraweave-author --exclude visual_3d --exclude ui_controls_demo --exclude npc_town_demo --exclude rhai_authoring

# Format and lint
cargo fmt --all
cargo clippy --workspace --exclude astraweave-author --exclude visual_3d --exclude ui_controls_demo --exclude npc_town_demo --exclude rhai_authoring -- -D warnings

# Run tests
cargo test -p astraweave-input -p astraweave-core
```

## Build Helper Usage

```bash
# Full development cycle
.github/scripts/build.sh full

# Quick builds
.github/scripts/build.sh build                 # Debug build
.github/scripts/build.sh build --release       # Release build

# Testing
.github/scripts/build.sh test --verbose        # Run tests

# Maintenance
.github/scripts/build.sh clean                 # Clean all
.github/scripts/build.sh cache-warm           # Warm cache
.github/scripts/build.sh cache-stats          # Show stats
```

## CI/CD Workflows

### Automatically Triggered
- **Push/PR**: `rust-cache-optimized.yml` - Main build workflow
- **Toolchain changes**: `toolchain-management.yml` - Compatibility testing
- **Daily 2 AM UTC**: `dependency-management.yml` - Cache warming and audits

### Manual Triggers
- **Dependency Management**: Cache actions (warm/clean/rebuild)
- **Toolchain Testing**: Custom Rust version testing
- **Performance Analysis**: Build benchmarking

## Cache Management

```bash
# Check cache status
sccache --show-stats

# Clear specific cache
.github/scripts/build.sh clean sccache    # Just sccache
.github/scripts/build.sh clean cargo      # Just cargo artifacts
.github/scripts/build.sh clean all        # Everything

# Warm cache for faster builds
.github/scripts/build.sh cache-warm
```

## Troubleshooting

### Build Failures
1. Check system dependencies: `.github/scripts/build.sh install-deps`
2. Clean and rebuild: `.github/scripts/build.sh clean && .github/scripts/build.sh build`
3. Check excluded crates list if adding new dependencies

### Performance Issues
1. Check cache stats: `.github/scripts/build.sh cache-stats`
2. Warm cache: `.github/scripts/build.sh cache-warm`
3. Use release mode for benchmarks: `--release`

### Compatibility Issues
1. Check MSRV: Rust 1.89.0 minimum
2. Test with stable: `rustup default stable`
3. Check platform dependencies

## Key Files

- `rust-toolchain.toml` - Rust version and components
- `.cargo/config.toml` - Build optimization and sccache
- `.github/workflows/rust-cache-optimized.yml` - Main CI workflow
- `.github/scripts/build.sh` - Build orchestration
- `docs/RUST_TOOLCHAIN_GUIDE.md` - Complete documentation

## Working vs Problematic Crates

### ✅ Core Working Crates
- astraweave-core, astraweave-ai, astraweave-render
- astraweave-physics, astraweave-nav, astraweave-gameplay
- astraweave-audio, astraweave-input, astraweave-ui
- hello_companion, physics_demo3d, navmesh_demo

### ❌ Excluded (Known Issues)
- astraweave-author (rhai sync/send traits)
- visual_3d (winit/egui API mismatches)
- ui_controls_demo, npc_town_demo (API issues)
- rhai_authoring (depends on broken crate)

## Environment Variables

```bash
export BUILD_TYPE=release        # release|debug
export VERBOSE=true             # Enable verbose output
export FEATURES=all             # all|default|minimal
export SCCACHE_CACHE_SIZE=10G   # sccache size limit
```