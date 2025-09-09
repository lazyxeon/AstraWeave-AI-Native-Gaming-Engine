#!/bin/bash
# Veilweaver Build Helper Script
# Optimized build orchestration for the Astraweave workspace

set -euo pipefail

# Source the cache configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/cache-config.sh"

# Build configuration
BUILD_TYPE="${BUILD_TYPE:-debug}"
VERBOSE="${VERBOSE:-false}"
PARALLEL="${PARALLEL:-true}"
FEATURES="${FEATURES:-default}"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $*"
}

error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

usage() {
    cat << EOF
Veilweaver Build Helper Script

Usage: $0 [OPTIONS] COMMAND

COMMANDS:
    clean           Clean all build artifacts and cache
    build           Build core workspace components  
    test            Run tests for core components
    check           Run cargo check on workspace
    fmt             Format all code
    clippy          Run clippy lints
    bench           Run benchmarks
    full            Run complete build pipeline
    cache-warm      Warm up the build cache
    cache-stats     Show cache statistics
    install-deps    Install system dependencies

OPTIONS:
    --release       Build in release mode
    --debug         Build in debug mode (default)
    --verbose       Enable verbose output
    --no-parallel   Disable parallel builds
    --features F    Specify feature set (default|all|minimal)
    --help          Show this help message

EXAMPLES:
    $0 build --release
    $0 test --verbose  
    $0 full --features all
    $0 clean && $0 cache-warm && $0 build

ENVIRONMENT VARIABLES:
    BUILD_TYPE      Build type (debug|release)
    VERBOSE         Enable verbose output (true|false)
    PARALLEL        Enable parallel builds (true|false)
    FEATURES        Feature set (default|all|minimal)
EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --debug)
            BUILD_TYPE="debug"
            shift
            ;;
        --verbose)
            VERBOSE="true"
            shift
            ;;
        --no-parallel)
            PARALLEL="false"
            shift
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --help)
            usage
            exit 0
            ;;
        -*)
            error "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            COMMAND="$1"
            shift
            ;;
    esac
done

# Validate command
if [[ -z "${COMMAND:-}" ]]; then
    error "No command specified"
    usage
    exit 1
fi

# Setup build arguments
CARGO_ARGS=("--locked")
if [[ "$BUILD_TYPE" == "release" ]]; then
    CARGO_ARGS+=("--release")
fi

if [[ "$VERBOSE" == "true" ]]; then
    CARGO_ARGS+=("--verbose")
fi

# Feature selection
case "$FEATURES" in
    "all")
        CARGO_ARGS+=("--all-features")
        ;;
    "minimal")
        CARGO_ARGS+=("--no-default-features")
        ;;
    "default")
        # Use default features
        ;;
    *)
        CARGO_ARGS+=("--features" "$FEATURES")
        ;;
esac

# Core packages arguments
CORE_PACKAGES=()
for crate in "${CORE_CRATES[@]}"; do
    CORE_PACKAGES+=("-p" "$crate")
done

# Exclude problematic packages
EXCLUDE_PACKAGES=()
for crate in "${EXCLUDED_CRATES[@]}"; do
    EXCLUDE_PACKAGES+=("--exclude" "$crate")
done

# Check system dependencies
check_system_deps() {
    log "Checking system dependencies..."
    local missing_deps=()
    
    case "$(uname -s)" in
        Linux*)
            # Check for required development libraries
            for dep in "libudev" "x11" "xi" "xcursor" "xrandr" "xinerama" "xkbcommon" "alsa"; do
                if ! pkg-config --exists "$dep" 2>/dev/null; then
                    missing_deps+=("$dep")
                fi
            done
            
            # Check for specific system tools
            for tool in "cmake" "ninja" "pkg-config"; do
                if ! command -v "$tool" &> /dev/null; then
                    missing_deps+=("$tool")
                fi
            done
            
            if [ ${#missing_deps[@]} -ne 0 ]; then
                warn "Missing dependencies: ${missing_deps[*]}"
                warn "Run '$0 install-deps' to install them automatically"
                return 1
            fi
            ;;
        Darwin*)
            for tool in "pkg-config" "cmake"; do
                if ! command -v "$tool" &> /dev/null; then
                    missing_deps+=("$tool")
                fi
            done
            
            if [ ${#missing_deps[@]} -ne 0 ]; then
                warn "Missing dependencies: ${missing_deps[*]}"
                warn "Install with: brew install ${missing_deps[*]}"
                return 1
            fi
            ;;
        *)
            warn "Unsupported platform for dependency checking: $(uname -s)"
            ;;
    esac
    
    success "All system dependencies found"
    return 0
}

# Install system dependencies
install_system_deps() {
    log "Installing system dependencies..."
    
    case "$(uname -s)" in
        Linux*)
            sudo apt-get update
            sudo apt-get install -y \
                build-essential pkg-config cmake ninja-build \
                libx11-dev libxi-dev libxcursor-dev libxrandr-dev libxinerama-dev \
                libxkbcommon-dev libxkbcommon-x11-dev libx11-xcb-dev \
                libxcb1-dev libxcb-randr0-dev libxcb-xfixes0-dev libxcb-shape0-dev libxcb-xkb-dev \
                libgl1-mesa-dev libegl1-mesa-dev wayland-protocols libwayland-dev \
                libasound2-dev libpulse-dev libudev-dev mesa-vulkan-drivers vulkan-tools
            ;;
        Darwin*)
            brew update
            brew install pkg-config
            ;;
        *)
            warn "Unsupported platform for automatic dependency installation"
            ;;
    esac
}

# Build core components
build_core() {
    log "Building core components ($BUILD_TYPE mode)..."
    init_sccache
    
    cargo build "${CARGO_ARGS[@]}" "${CORE_PACKAGES[@]}"
    
    show_cache_stats
    success "Core build completed"
}

# Build entire workspace (excluding problematic crates)
build_workspace() {
    log "Building workspace ($BUILD_TYPE mode)..."
    init_sccache
    
    cargo build "${CARGO_ARGS[@]}" --workspace "${EXCLUDE_PACKAGES[@]}"
    
    show_cache_stats
    success "Workspace build completed"
}

# Run tests
run_tests() {
    log "Running tests..."
    
    cargo test "${CARGO_ARGS[@]}" --workspace "${EXCLUDE_PACKAGES[@]}" --lib
    
    success "Tests completed"
}

# Run cargo check
run_check() {
    log "Running cargo check..."
    
    cargo check "${CARGO_ARGS[@]}" --workspace "${EXCLUDE_PACKAGES[@]}"
    
    success "Check completed"
}

# Format code
format_code() {
    log "Formatting code..."
    
    cargo fmt --all
    
    success "Code formatted"
}

# Run clippy
run_clippy() {
    log "Running clippy..."
    
    cargo clippy "${CARGO_ARGS[@]}" --workspace "${EXCLUDE_PACKAGES[@]}" \
        --all-targets -- -D warnings
    
    success "Clippy completed"
}

# Run benchmarks
run_benchmarks() {
    log "Running benchmarks..."
    
    cargo bench "${CARGO_ARGS[@]}" "${CORE_PACKAGES[@]}" || true
    
    success "Benchmarks completed"
}

# Full build pipeline
run_full_pipeline() {
    log "Running full build pipeline..."
    
    check_system_deps
    warm_cache
    format_code
    run_check
    build_workspace
    run_tests
    run_clippy
    
    if [[ "$BUILD_TYPE" == "release" ]]; then
        run_benchmarks
    fi
    
    success "Full pipeline completed"
}

# Execute command
case "$COMMAND" in
    "clean")
        log "Cleaning build artifacts..."
        clean_cache all
        success "Clean completed"
        ;;
    "build")
        build_core
        ;;
    "build-all")
        build_workspace
        ;;
    "test")
        run_tests
        ;;
    "check")
        run_check
        ;;
    "fmt")
        format_code
        ;;
    "clippy")
        run_clippy
        ;;
    "bench")
        run_benchmarks
        ;;
    "full")
        run_full_pipeline
        ;;
    "cache-warm")
        warm_cache
        ;;
    "cache-stats")
        show_cache_stats
        ;;
    "install-deps")
        install_system_deps
        ;;
    *)
        error "Unknown command: $COMMAND"
        usage
        exit 1
        ;;
esac