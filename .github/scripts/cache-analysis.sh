#!/bin/bash
# Cache and Dependency Analysis Script for AstraWeave
# This script analyzes build cache effectiveness and dependency usage

set -euo pipefail

echo "🔍 AstraWeave Cache and Dependency Analysis"
echo "============================================"

# Configuration
WORKSPACE_ROOT=$(pwd)
EXCLUDED_PACKAGES="--exclude astraweave-author --exclude visual_3d --exclude ui_controls_demo --exclude npc_town_demo --exclude rhai_authoring --exclude cutscene_render_demo --exclude weaving_playground --exclude combat_physics_demo --exclude navmesh_demo --exclude physics_demo3d"

# Function to measure build time
measure_build_time() {
    local build_type="$1"
    local start_time=$(date +%s)
    
    echo "⏱️  Measuring $build_type build time..."
    if [ "$build_type" = "clean" ]; then
        cargo clean
        cargo build --workspace $EXCLUDED_PACKAGES
    elif [ "$build_type" = "incremental" ]; then
        cargo build --workspace $EXCLUDED_PACKAGES
    elif [ "$build_type" = "release" ]; then
        cargo build --release --workspace $EXCLUDED_PACKAGES
    fi
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    echo "✅ $build_type build completed in ${duration}s"
    return $duration
}

# Function to analyze cache size
analyze_cache() {
    echo "📊 Analyzing cache sizes..."
    
    if [ -d "target" ]; then
        echo "Target directory size: $(du -sh target 2>/dev/null | cut -f1)"
        echo "Debug artifacts: $(du -sh target/debug 2>/dev/null | cut -f1 || echo 'N/A')"
        echo "Release artifacts: $(du -sh target/release 2>/dev/null | cut -f1 || echo 'N/A')"
        echo "Dependency cache: $(du -sh target/debug/deps 2>/dev/null | cut -f1 || echo 'N/A')"
    fi
    
    if [ -d "~/.cargo" ]; then
        echo "Cargo cache size: $(du -sh ~/.cargo 2>/dev/null | cut -f1 || echo 'N/A')"
    fi
    
    if [ -d "target/sccache" ]; then
        echo "Sccache directory: $(du -sh target/sccache 2>/dev/null | cut -f1 || echo 'N/A')"
    fi
}

# Function to analyze dependencies
analyze_dependencies() {
    echo "🔗 Analyzing dependencies..."
    
    # Count total dependencies
    echo "Total dependencies: $(cargo tree --workspace $EXCLUDED_PACKAGES | wc -l)"
    
    # Find duplicate dependencies
    echo "Checking for duplicate dependencies..."
    cargo tree --workspace $EXCLUDED_PACKAGES --duplicates | head -20 || echo "No significant duplicates found"
    
    # Analyze dependency sizes
    if command -v cargo-bloat &> /dev/null; then
        echo "Top space-consuming dependencies:"
        cargo bloat --release -p astraweave-core --crates | head -10 || echo "Bloat analysis not available"
    fi
}

# Function to test cache effectiveness
test_cache_effectiveness() {
    echo "🧪 Testing cache effectiveness..."
    
    # Clean build
    local clean_time=$(measure_build_time "clean")
    
    # Incremental build (should be much faster)
    local incremental_time=$(measure_build_time "incremental")
    
    # Calculate cache effectiveness
    if [ "$clean_time" -gt 0 ]; then
        local speedup=$(( (clean_time * 100) / (incremental_time + 1) ))
        echo "Cache speedup: ${speedup}% (${clean_time}s -> ${incremental_time}s)"
        
        if [ "$speedup" -gt 500 ]; then
            echo "✅ Excellent cache performance!"
        elif [ "$speedup" -gt 200 ]; then
            echo "✅ Good cache performance"
        else
            echo "⚠️  Cache performance could be improved"
        fi
    fi
}

# Function to generate recommendations
generate_recommendations() {
    echo "💡 Recommendations:"
    
    # Check for large target directory
    local target_size=$(du -s target 2>/dev/null | cut -f1 || echo "0")
    if [ "$target_size" -gt 5000000 ]; then  # 5GB in KB
        echo "- Consider running 'cargo clean' occasionally to reduce disk usage"
    fi
    
    # Check for sccache availability
    if ! command -v sccache &> /dev/null; then
        echo "- Install sccache for improved build caching: cargo install sccache"
    fi
    
    # Check for analysis tools
    if ! command -v cargo-bloat &> /dev/null; then
        echo "- Install cargo-bloat for dependency size analysis: cargo install cargo-bloat"
    fi
    
    if ! command -v cargo-tree &> /dev/null; then
        echo "- Install cargo-tree for dependency analysis: cargo install cargo-tree"
    fi
}

# Main execution
main() {
    cd "$WORKSPACE_ROOT"
    
    echo "Starting analysis at $(date)"
    echo ""
    
    analyze_cache
    echo ""
    
    analyze_dependencies
    echo ""
    
    test_cache_effectiveness
    echo ""
    
    generate_recommendations
    echo ""
    
    echo "✅ Analysis complete!"
}

# Run if called directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi