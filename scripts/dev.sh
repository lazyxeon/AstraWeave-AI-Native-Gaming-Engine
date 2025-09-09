#!/bin/bash
# AstraWeave Gaming Engine - Development Helper Script
# Provides convenient commands for common development tasks

set -euo pipefail

# Color codes for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the project root
check_project_root() {
    if [[ ! -f "rust-toolchain.toml" ]] || [[ ! -f "Cargo.toml" ]]; then
        log_error "This script must be run from the AstraWeave project root directory"
        exit 1
    fi
}

# Quick development environment validation
validate_env() {
    log_info "Validating development environment..."
    
    local failed=0
    
    # Check Rust
    if ! command -v rustc >/dev/null 2>&1; then
        log_error "Rust compiler not found. Run './scripts/bootstrap.sh' to set up."
        failed=1
    else
        log_success "Rust compiler: $(rustc --version)"
    fi
    
    # Check Cargo
    if ! command -v cargo >/dev/null 2>&1; then
        log_error "Cargo not found"
        failed=1
    else
        log_success "Cargo: $(cargo --version)"
    fi
    
    # Check essential tools
    local tools=("pkg-config" "cmake")
    for tool in "${tools[@]}"; do
        if command -v "$tool" >/dev/null 2>&1; then
            log_success "$tool: available"
        else
            log_warning "$tool not found - may cause build issues"
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        log_success "Environment validation passed ✓"
        return 0
    else
        log_error "Environment validation failed ✗"
        return 1
    fi
}

# Quick build of core components
quick_build() {
    log_info "Building core components..."
    
    if cargo build-core; then
        log_success "Core components built successfully"
        return 0
    else
        log_error "Core build failed"
        return 1
    fi
}

# Full workspace build (excluding problematic packages)
full_build() {
    log_info "Building all working components..."
    
    if cargo build-working; then
        log_success "All working components built successfully"
        return 0
    else
        log_error "Full build failed"
        return 1
    fi
}

# Run tests
run_tests() {
    log_info "Running tests for all working components..."
    
    if cargo test-all; then
        log_success "All tests passed"
        return 0
    else
        log_error "Some tests failed"
        return 1
    fi
}

# Run code quality checks
quality_check() {
    log_info "Running code quality checks..."
    
    local failed=0
    
    # Format check
    log_info "Checking code formatting..."
    if cargo fmt --all --check; then
        log_success "Code formatting: OK"
    else
        log_error "Code formatting: FAILED (run 'cargo fmt --all' to fix)"
        failed=1
    fi
    
    # Clippy check
    log_info "Running Clippy linter..."
    if cargo clippy-all -- -D warnings; then
        log_success "Clippy: OK"
    else
        log_error "Clippy: FAILED"
        failed=1
    fi
    
    if [[ $failed -eq 0 ]]; then
        log_success "Code quality checks passed ✓"
        return 0
    else
        log_error "Code quality checks failed ✗"
        return 1
    fi
}

# Run security audit
security_audit() {
    log_info "Running security audit..."
    
    local failed=0
    
    # Check if tools are installed
    if ! command -v cargo-audit >/dev/null 2>&1; then
        log_warning "cargo-audit not found. Installing..."
        cargo install cargo-audit --locked || {
            log_error "Failed to install cargo-audit"
            failed=1
        }
    fi
    
    if ! command -v cargo-deny >/dev/null 2>&1; then
        log_warning "cargo-deny not found. Installing..."
        cargo install cargo-deny --locked || {
            log_error "Failed to install cargo-deny"
            failed=1
        }
    fi
    
    if [[ $failed -eq 0 ]]; then
        # Run audit
        log_info "Running cargo-audit..."
        if cargo audit; then
            log_success "Security audit: OK"
        else
            log_warning "Security audit found issues (may be acceptable)"
        fi
        
        # Run deny check
        log_info "Running cargo-deny..."
        if cargo deny check; then
            log_success "Dependency policy check: OK"
        else
            log_warning "Dependency policy check found issues"
        fi
    fi
    
    return 0
}

# Clean build artifacts
clean_build() {
    log_info "Cleaning build artifacts..."
    
    cargo clean
    
    # Also clean sccache if available
    if command -v sccache >/dev/null 2>&1; then
        log_info "Cleaning sccache..."
        sccache --zero-stats || true
    fi
    
    log_success "Build artifacts cleaned"
}

# Run the hello_companion example
run_example() {
    log_info "Running hello_companion example..."
    
    # Build first
    if cargo build --release -p hello_companion; then
        log_info "Starting hello_companion (will timeout after 10 seconds)..."
        timeout 10s cargo run --release -p hello_companion || {
            local exit_code=$?
            if [[ $exit_code -eq 124 ]]; then
                log_success "Example ran and timed out as expected"
            else
                log_warning "Example exited with code $exit_code (may be normal)"
            fi
        }
    else
        log_error "Failed to build hello_companion example"
        return 1
    fi
}

# Development workflow - comprehensive check
dev_check() {
    log_info "Running comprehensive development checks..."
    
    local failed=0
    
    # Validate environment
    if ! validate_env; then
        failed=1
    fi
    
    # Quick build
    if ! quick_build; then
        failed=1
    fi
    
    # Run tests
    if ! run_tests; then
        failed=1
    fi
    
    # Quality checks
    if ! quality_check; then
        failed=1
    fi
    
    # Security audit
    security_audit  # Don't fail on security audit warnings
    
    if [[ $failed -eq 0 ]]; then
        log_success "🎉 All development checks passed! Your code is ready."
        return 0
    else
        log_error "❌ Some development checks failed. Please review the errors above."
        return 1
    fi
}

# Watch mode for development
watch_mode() {
    local package="${1:-astraweave-core}"
    
    if ! command -v cargo-watch >/dev/null 2>&1; then
        log_info "Installing cargo-watch..."
        cargo install cargo-watch --locked
    fi
    
    log_info "Starting watch mode for package: $package"
    log_info "Press Ctrl+C to stop watching"
    
    cargo watch -x "check -p $package" -x "test -p $package --lib"
}

# Update dependencies
update_deps() {
    log_info "Updating dependencies..."
    
    # Update Cargo dependencies
    cargo update --workspace
    
    # Update Rust toolchain components
    log_info "Updating Rust toolchain components..."
    rustup component add rustfmt clippy rust-src rust-analyzer llvm-tools-preview || true
    
    log_success "Dependencies updated"
}

# Show project status
show_status() {
    log_info "AstraWeave Gaming Engine - Project Status"
    echo ""
    
    # Toolchain info
    echo -e "${BLUE}Rust Toolchain:${NC}"
    rustc --version 2>/dev/null || echo "  Rust not found"
    cargo --version 2>/dev/null || echo "  Cargo not found"
    echo ""
    
    # Git status
    echo -e "${BLUE}Git Status:${NC}"
    git branch --show-current 2>/dev/null || echo "  Not a git repository"
    git status --porcelain 2>/dev/null | wc -l | xargs echo "  Modified files:"
    echo ""
    
    # Build status
    echo -e "${BLUE}Build Status:${NC}"
    if [[ -d "target" ]]; then
        echo "  Build directory exists"
        if [[ -d "target/debug" ]]; then
            echo "  Debug builds available"
        fi
        if [[ -d "target/release" ]]; then
            echo "  Release builds available"
        fi
    else
        echo "  No builds found"
    fi
    echo ""
    
    # Available examples
    echo -e "${BLUE}Available Examples:${NC}"
    find examples -name "Cargo.toml" -exec dirname {} \; | sort | sed 's/^/  /'
}

# Print usage information
print_usage() {
    cat << EOF
AstraWeave Gaming Engine - Development Helper

Usage: $0 COMMAND [OPTIONS]

COMMANDS:
    validate        Validate development environment
    build           Quick build of core components
    build-all       Build all working components
    test            Run tests for all working components
    quality         Run code quality checks (format + clippy)
    audit           Run security audit
    clean           Clean build artifacts
    example         Run hello_companion example
    check           Run comprehensive development checks
    watch [PKG]     Watch for changes and rebuild (default: astraweave-core)
    update          Update dependencies and toolchain
    status          Show project status
    help            Show this help message

EXAMPLES:
    $0 validate                 # Check if environment is set up correctly
    $0 build                    # Quick build of core components
    $0 check                    # Run all checks before committing
    $0 watch astraweave-ai      # Watch astraweave-ai package for changes
    $0 example                  # Test run the hello_companion example

For initial setup, run: ./scripts/bootstrap.sh
EOF
}

# Main execution function
main() {
    # Check if we're in the project root
    check_project_root
    
    # Parse command
    local command="${1:-help}"
    shift || true
    
    case "$command" in
        validate|val)
            validate_env
            ;;
        build|b)
            quick_build
            ;;
        build-all|ba)
            full_build
            ;;
        test|t)
            run_tests
            ;;
        quality|q)
            quality_check
            ;;
        audit|a)
            security_audit
            ;;
        clean|c)
            clean_build
            ;;
        example|ex)
            run_example
            ;;
        check|ch)
            dev_check
            ;;
        watch|w)
            watch_mode "$@"
            ;;
        update|up)
            update_deps
            ;;
        status|st)
            show_status
            ;;
        help|h|-h|--help)
            print_usage
            ;;
        *)
            log_error "Unknown command: $command"
            print_usage
            exit 1
            ;;
    esac
}

# Execute main function with all arguments
main "$@"