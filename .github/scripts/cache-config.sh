# Veilweaver Caching Strategy Configuration
# This file defines the optimal caching strategy for the Veilweaver workspace

# sccache configuration
export SCCACHE_CACHE_SIZE="10G"
export SCCACHE_DIR="./target/sccache"
# RUSTC_WRAPPER will be set by init_sccache function

# Cargo environment variables for optimal caching
export CARGO_INCREMENTAL=0
export CARGO_NET_RETRY=10
export CARGO_NET_TIMEOUT=30

# Build parallelism - use system default for job count

# Cache directories that should be preserved
CACHE_DIRECTORIES=(
    "~/.cargo/registry/index"
    "~/.cargo/registry/cache" 
    "~/.cargo/git/db"
    "./target/debug"
    "./target/release"
    "./target/sccache"
    "~/.rustup"
)

# Workspace-specific cache keys based on content
CACHE_KEY_COMPONENTS=(
    "Cargo.lock"
    "rust-toolchain.toml"
    ".cargo/config.toml"
    "astraweave-*/Cargo.toml"
    "examples/*/Cargo.toml"
)

# Crates to always include in core cache
CORE_CRATES=(
    "astraweave-core"
    "astraweave-ai"
    "astraweave-render"
    "astraweave-physics"
    "astraweave-nav"
    "astraweave-gameplay"
    "astraweave-audio"
    "astraweave-input"
    "astraweave-ui"
    "hello_companion"
)

# Crates with compilation issues to exclude
EXCLUDED_CRATES=(
    "debug_toolkit_demo"   # Multiple API mismatches in egui/winit/renderer integration
    "aw_editor"           # eframe/glutin sync/send trait issues
    "aw_debug"            # eframe API mismatches
    "visual_3d"           # clippy deny-level errors
    "navmesh_demo"        # clippy deny-level errors (approx_constant)
    "physics_demo3d"      # clippy deny-level errors (approx_constant)
)

# Platform-specific optimizations
case "$(uname -s)" in
    Linux*)
        export CARGO_TARGET_DIR="./target"
        ;;
    Darwin*)
        export CARGO_TARGET_DIR="./target"
        ;;
    MINGW*|CYGWIN*|MSYS*)
        export CARGO_TARGET_DIR="./target"
        ;;
esac

# Functions for cache management

# Initialize sccache
init_sccache() {
    if command -v sccache > /dev/null 2>&1; then
        echo "Starting sccache server..."
        export RUSTC_WRAPPER=sccache
        # Stop any existing server first
        sccache --stop-server 2>/dev/null || true
        # Start server
        sccache --start-server
        # Show initial stats
        sccache --show-stats
    else
        echo "sccache not found, using standard rustc..."
        unset RUSTC_WRAPPER
        # Try to install sccache if cargo is available
        if command -v cargo > /dev/null 2>&1; then
            echo "Attempting to install sccache..."
            cargo install sccache --locked || echo "Failed to install sccache, continuing without it"
        fi
    fi
}

# Show cache statistics
show_cache_stats() {
    if command -v sccache > /dev/null 2>&1; then
        echo "=== sccache Statistics ==="
        sccache --show-stats
    fi
    
    echo "=== Cargo Cache Size ==="
    du -sh ~/.cargo/ 2>/dev/null || echo "Cargo cache not found"
    
    echo "=== Target Directory Size ==="
    du -sh ./target/ 2>/dev/null || echo "Target directory not found"
}

# Clean cache selectively
clean_cache() {
    local cache_type="${1:-all}"
    
    case "$cache_type" in
        "sccache")
            sccache --zero-stats
            rm -rf "$SCCACHE_DIR"
            ;;
        "cargo")
            cargo clean
            ;;
        "registry")
            rm -rf ~/.cargo/registry/cache/*
            ;;
        "all")
            cargo clean
            sccache --zero-stats
            rm -rf "$SCCACHE_DIR"
            rm -rf ~/.cargo/registry/cache/*
            ;;
        *)
            echo "Unknown cache type: $cache_type"
            echo "Valid options: sccache, cargo, registry, all"
            return 1
            ;;
    esac
    
    echo "Cache cleaned: $cache_type"
}

# Warm up cache with core dependencies
warm_cache() {
    echo "Warming up cache with core dependencies..."
    
    # Fetch all dependencies
    cargo fetch --locked
    
    # Check core crates to populate cache
    for crate in "${CORE_CRATES[@]}"; do
        echo "Warming cache for $crate..."
        cargo check -p "$crate" --all-features || true
    done
    
    echo "Cache warming complete"
    show_cache_stats
}

# Build with cache optimization
build_with_cache() {
    local build_type="${1:-debug}"
    local extra_args="${2:-}"
    
    init_sccache
    
    echo "Building with cache optimization (${build_type})..."
    
    if [ "$build_type" = "release" ]; then
        cargo build --release --locked \
            $(printf -- "-p %s " "${CORE_CRATES[@]}") \
            $extra_args
    else
        cargo build --locked \
            $(printf -- "-p %s " "${CORE_CRATES[@]}") \
            $extra_args
    fi
    
    show_cache_stats
}

# Export functions for use in scripts
export -f init_sccache
export -f show_cache_stats  
export -f clean_cache
export -f warm_cache
export -f build_with_cache