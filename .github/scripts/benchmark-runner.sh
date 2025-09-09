#!/bin/bash
# AstraWeave Benchmark Runner Script
# Runs benchmarks with proper JSON output for github-action-benchmark integration

set -euo pipefail

# Configuration
BENCHMARK_PACKAGES_STATIC=(astraweave-core astraweave-input)
RESULTS_DIR="${BENCHMARK_RESULTS_DIR:-benchmark_results}"
SUMMARY_FILE="$RESULTS_DIR/summary.txt"
JSON_FILE="$RESULTS_DIR/benchmarks.json"
VERBOSE="${VERBOSE:-false}"

# Create results directory
mkdir -p "$RESULTS_DIR"

# Logging functions
log_info() {
    echo "[INFO] $*" | tee -a "$SUMMARY_FILE"
}

log_error() {
    echo "[ERROR] $*" | tee -a "$SUMMARY_FILE" >&2
}

log_success() {
    echo "[SUCCESS] $*" | tee -a "$SUMMARY_FILE"
}

# Auto-discover packages with benchmarks
BENCHMARK_PACKAGES=()

# Function to discover benchmark packages
discover_benchmark_packages() {
    local discovered_packages=()
    
    # Start with static list
    for pkg in "${BENCHMARK_PACKAGES_STATIC[@]}"; do
        if [ -d "$pkg/benches" ]; then
            discovered_packages+=("$pkg")
        else
            echo "[WARN] Static package $pkg has no benchmarks directory"
        fi
    done
    
    # Auto-discover additional packages with benchmarks
    for pkg_dir in */; do
        pkg_name=$(basename "$pkg_dir")
        if [ -d "$pkg_dir/benches" ] && [[ ! " ${BENCHMARK_PACKAGES_STATIC[*]} " =~ " ${pkg_name} " ]]; then
            # Check if this is a Rust package with benchmarks
            if [ -f "$pkg_dir/Cargo.toml" ] && grep -q "\[\[bench\]\]" "$pkg_dir/Cargo.toml"; then
                discovered_packages+=("$pkg_name")
                echo "[INFO] Auto-discovered benchmark package: $pkg_name"
            fi
        fi
    done
    
    # Set the global array
    BENCHMARK_PACKAGES=("${discovered_packages[@]}")
    
    if [ ${#BENCHMARK_PACKAGES[@]} -eq 0 ]; then
        log_info "No benchmark packages found"
        return 1
    fi
    
    log_info "Found ${#BENCHMARK_PACKAGES[@]} benchmark packages: ${BENCHMARK_PACKAGES[*]}"
    return 0
}

# Discover benchmark packages
if ! discover_benchmark_packages; then
    echo "[ERROR] No benchmark packages found!" | tee -a "$SUMMARY_FILE"
    exit 1
fi

# Initialize summary file
{
    echo "=== AstraWeave Performance Benchmarks ==="
    echo "Date: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo "Commit: ${GITHUB_SHA:-$(git rev-parse HEAD 2>/dev/null || echo 'unknown')}"
    echo "Runner: ${RUNNER_OS:-$(uname -s)} ${RUNNER_ARCH:-$(uname -m)}"
    echo ""
} > "$SUMMARY_FILE"

# Initialize JSON array
echo '[' > "$JSON_FILE"
FIRST_ENTRY=true
BENCHMARK_COUNT=0
SUCCESS_COUNT=0

# Function to format time units
format_time() {
    local ns=$1
    if (( $(echo "$ns < 1000" | bc -l) )); then
        printf "%.2f ns" "$ns"
    elif (( $(echo "$ns < 1000000" | bc -l) )); then
        printf "%.2f µs" "$(echo "scale=2; $ns / 1000" | bc -l)"
    elif (( $(echo "$ns < 1000000000" | bc -l) )); then
        printf "%.2f ms" "$(echo "scale=2; $ns / 1000000" | bc -l)"
    else
        printf "%.2f s" "$(echo "scale=2; $ns / 1000000000" | bc -l)"
    fi
}

# Function to process benchmark results for a specific package
process_benchmarks() {
    local pkg=$1
    local pkg_success=false
    
    if [ -d "target/criterion" ]; then
        local found_benchmarks=false
        
        for benchmark_dir in target/criterion/*/; do
            if [ -d "$benchmark_dir" ] && [ -f "$benchmark_dir/new/estimates.json" ]; then
                found_benchmarks=true
                bench_name=$(basename "$benchmark_dir")
                
                # Safely extract mean value with error handling
                if mean_ns=$(jq -r '.mean.point_estimate // empty' "$benchmark_dir/new/estimates.json" 2>/dev/null) && [ -n "$mean_ns" ] && [ "$mean_ns" != "null" ]; then
                    # Validate that the value is a valid number
                    if [[ "$mean_ns" =~ ^[0-9]+\.?[0-9]*$ ]] && (( $(echo "$mean_ns > 0" | bc -l) )); then
                        # Add to JSON
                        if [ "$FIRST_ENTRY" != true ]; then
                            echo ',' >> "$JSON_FILE"
                        fi
                        
                        jq -n --arg name "${pkg}::${bench_name}" --argjson value "$mean_ns" \
                            '{name: $name, unit: "ns", value: $value}' >> "$JSON_FILE"
                        FIRST_ENTRY=false
                        BENCHMARK_COUNT=$((BENCHMARK_COUNT + 1))
                        
                        # Add to summary with proper formatting
                        formatted_time=$(format_time "$mean_ns")
                        printf "  %-30s %s\n" "$bench_name" "$formatted_time" | tee -a "$SUMMARY_FILE"
                        
                        pkg_success=true
                    else
                        log_error "Invalid benchmark value for $bench_name: $mean_ns"
                    fi
                else
                    log_error "Could not extract valid benchmark data for $bench_name"
                fi
            fi
        done
        
        if [ "$found_benchmarks" = false ]; then
            log_error "No criterion results found for $pkg"
        fi
    else
        log_error "Criterion target directory not found"
    fi
    
    if [ "$pkg_success" = true ]; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    fi
    
    return 0
}

# Main benchmark execution
log_info "Starting benchmark execution..."

for pkg in "${BENCHMARK_PACKAGES[@]}"; do
    if [ -d "$pkg/benches" ]; then
        log_info "Running benchmarks for $pkg..."
        
        # Clear previous criterion results to avoid cross-contamination
        if [ -d "target/criterion" ]; then
            rm -rf target/criterion/* 2>/dev/null || true
        fi
        
        # Run cargo bench with timeout and capture output
        if timeout 600 cargo bench -p "$pkg" --benches > "${RESULTS_DIR}/${pkg}_stdout.log" 2> "${RESULTS_DIR}/${pkg}_stderr.log"; then
            log_success "Benchmark execution completed for $pkg"
            
            # Process the results immediately for this package
            process_benchmarks "$pkg"
            
            if [ "$VERBOSE" = "true" ]; then
                echo "--- $pkg stdout ---" >> "$SUMMARY_FILE"
                tail -n 20 "${RESULTS_DIR}/${pkg}_stdout.log" >> "$SUMMARY_FILE"
                echo "--- end stdout ---" >> "$SUMMARY_FILE"
            fi
        else
            log_error "Benchmark execution failed for $pkg"
            echo "--- $pkg stderr ---" >> "$SUMMARY_FILE"
            cat "${RESULTS_DIR}/${pkg}_stderr.log" >> "$SUMMARY_FILE"
            echo "--- end stderr ---" >> "$SUMMARY_FILE"
        fi
        echo "" >> "$SUMMARY_FILE"
    else
        log_info "No benchmarks found for $pkg"
    fi
done

# Close JSON array
echo ']' >> "$JSON_FILE"

# Generate final summary
{
    echo "=== Execution Summary ==="
    echo "Total packages processed: ${#BENCHMARK_PACKAGES[@]}"
    echo "Packages with successful benchmarks: $SUCCESS_COUNT"
    echo "Total benchmarks collected: $BENCHMARK_COUNT"
    echo ""
    echo "Results saved to:"
    echo "  - Summary: $SUMMARY_FILE"
    echo "  - JSON data: $JSON_FILE"
} | tee -a "$SUMMARY_FILE"

# Validate JSON output
if jq empty "$JSON_FILE" 2>/dev/null; then
    log_success "Generated valid JSON benchmark data"
else
    log_error "Generated JSON is invalid!"
    exit 1
fi

# Display final results
echo ""
echo "=== Benchmark Results Summary ==="
cat "$SUMMARY_FILE"

# Exit with success if we got at least one benchmark
if [ "$BENCHMARK_COUNT" -gt 0 ]; then
    exit 0
else
    log_error "No benchmarks were successfully collected"
    exit 1
fi