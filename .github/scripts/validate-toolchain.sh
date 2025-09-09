#!/bin/bash
# Veilweaver Toolchain Validation Script
# Comprehensive validation of the Rust toolchain setup

set -uo pipefail  # Removed -e to handle errors gracefully

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

# Validation counters
PASSED=0
FAILED=0
WARNINGS=0

validate_step() {
    local step_name="$1"
    local command="$2"
    local required="${3:-true}"
    
    log "Validating: $step_name"
    
    if eval "$command" >/dev/null 2>&1; then
        success "$step_name: ✓"
        ((PASSED++))
        return 0
    else
        if [ "$required" = "true" ]; then
            error "$step_name: ✗"
            ((FAILED++))
        else
            warn "$step_name: ⚠ (optional)"
            ((WARNINGS++))
        fi
        return 1
    fi
}

# Main validation
main() {
    log "Starting Veilweaver Toolchain Validation"
    echo "========================================"
    
    # 1. Basic Rust toolchain
    log "Validating Rust toolchain..."
    validate_step "Rust compiler" "rustc --version"
    validate_step "Cargo package manager" "cargo --version"
    validate_step "Rustfmt formatter" "rustfmt --version"
    validate_step "Clippy linter" "cargo clippy --version"
    validate_step "Rust analyzer" "rust-analyzer --version" false
    
    # 2. Toolchain configuration
    log "Validating toolchain configuration..."
    validate_step "rust-toolchain.toml exists" "test -f rust-toolchain.toml"
    validate_step "Cargo.toml workspace exists" "test -f Cargo.toml"
    validate_step ".cargo/config.toml exists" "test -f .cargo/config.toml"
    
    # 3. Version consistency check
    log "Checking version consistency..."
    local rust_toolchain_version=$(grep 'channel.*=' rust-toolchain.toml | cut -d'"' -f2)
    local cargo_rust_version=$(grep 'rust-version.*=' Cargo.toml | cut -d'"' -f2)
    
    if [ "$rust_toolchain_version" = "$cargo_rust_version" ]; then
        success "Rust versions are consistent: $rust_toolchain_version"
        ((PASSED++))
    else
        error "Rust version mismatch: toolchain=$rust_toolchain_version, workspace=$cargo_rust_version"
        ((FAILED++))
    fi
    
    # 4. System dependencies
    log "Checking system dependencies..."
    case "$(uname -s)" in
        Linux*)
            validate_step "pkg-config" "pkg-config --version"
            validate_step "cmake" "cmake --version"
            validate_step "ninja" "ninja --version" false
            validate_step "libudev" "pkg-config --exists libudev"
            validate_step "x11" "pkg-config --exists x11"
            validate_step "alsa" "pkg-config --exists alsa"
            ;;
        Darwin*)
            validate_step "pkg-config" "pkg-config --version"
            validate_step "cmake" "cmake --version"
            ;;
        *)
            warn "Unknown platform for system dependency checking"
            ;;
    esac
    
    # 5. Optional tools
    log "Checking optional tools..."
    validate_step "sccache" "sccache --version" false
    validate_step "cargo-nextest" "cargo nextest --version" false
    validate_step "cargo-audit" "cargo audit --version" false
    validate_step "cargo-deny" "cargo deny --version" false
    
    # 6. Build validation
    log "Testing core build functionality..."
    validate_step "Core crates check" "cargo check -p astraweave-core -p astraweave-ai -p astraweave-physics"
    validate_step "Workspace check (excluding problematic)" "cargo check-all"
    
    # 7. Code quality
    log "Testing code quality tools..."
    validate_step "Format check" "cargo fmt --all -- --check"
    validate_step "Clippy check" "cargo clippy-all"
    
    # 8. Script validation
    log "Validating build scripts..."
    validate_step "Build script executable" "test -x .github/scripts/build.sh"
    validate_step "Cache script executable" "test -x .github/scripts/cache-config.sh"
    validate_step "Build script help" ".github/scripts/build.sh --help > /dev/null"
    
    # Summary
    echo ""
    log "Validation Summary"
    echo "=================="
    success "Passed: $PASSED"
    if [ $WARNINGS -gt 0 ]; then
        warn "Warnings: $WARNINGS"
    fi
    if [ $FAILED -gt 0 ]; then
        error "Failed: $FAILED"
        echo ""
        error "Some validations failed. Please address the issues above."
        return 1
    else
        echo ""
        success "All critical validations passed! ✨"
        if [ $WARNINGS -gt 0 ]; then
            warn "Note: Some optional tools are missing but this won't affect core functionality."
        fi
        return 0
    fi
}

# Run validation
main "$@"