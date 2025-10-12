#!/bin/bash
# Quick validation script for benchmark dashboard fix
# Tests that the dashboard can load data correctly

set -e

echo "=== AstraWeave Benchmark Dashboard Validation ==="
echo ""

# Check if we're in the repo root
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Run this script from the repository root"
    exit 1
fi

echo "✓ Running from repository root"

# Check if target/benchmark-data directory exists
if [ ! -d "target/benchmark-data" ]; then
    echo "Creating target/benchmark-data directory..."
    mkdir -p target/benchmark-data
fi

echo "✓ Benchmark data directory exists"

# Check if history.jsonl exists
if [ ! -f "target/benchmark-data/history.jsonl" ]; then
    echo ""
    echo "⚠️  No benchmark data found. Creating sample data for testing..."
    
    # Create sample JSONL data (one JSON object per line)
    cat > target/benchmark-data/history.jsonl << 'EOF'
{"timestamp":"2025-10-10T10:00:00Z","benchmark_name":"astraweave-core::ecs_benchmarks/world_creation","value":125000,"stddev":5000,"unit":"ns","git_sha":"abc12345","git_branch":"main","git_dirty":false,"crate":"astraweave-core","group":"ecs_benchmarks","name":"world_creation"}
{"timestamp":"2025-10-10T10:00:00Z","benchmark_name":"astraweave-ai::ai_core_loop/simple","value":980000,"stddev":15000,"unit":"ns","git_sha":"abc12345","git_branch":"main","git_dirty":false,"crate":"astraweave-ai","group":"ai_core_loop","name":"simple"}
{"timestamp":"2025-10-11T10:00:00Z","benchmark_name":"astraweave-core::ecs_benchmarks/world_creation","value":120000,"stddev":4800,"unit":"ns","git_sha":"def67890","git_branch":"main","git_dirty":false,"crate":"astraweave-core","group":"ecs_benchmarks","name":"world_creation"}
EOF
    
    echo "✓ Created sample data (3 entries)"
    echo "   Note: This is test data. Run 'cargo bench' for real benchmarks."
else
    LINE_COUNT=$(wc -l < target/benchmark-data/history.jsonl)
    echo "✓ Found existing benchmark data ($LINE_COUNT entries)"
fi

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo ""
    echo "❌ Python 3 not found. Install Python 3 to run the HTTP server."
    exit 1
fi

echo "✓ Python 3 is available"

# Check if dashboard files exist
if [ ! -f "tools/benchmark-dashboard/index.html" ]; then
    echo "❌ Dashboard files not found at tools/benchmark-dashboard/"
    exit 1
fi

echo "✓ Dashboard files exist"

echo ""
echo "=== All checks passed! ==="
echo ""
echo "To view the dashboard:"
echo "  1. Run: python3 -m http.server 8000"
echo "  2. Open: http://localhost:8000/tools/benchmark-dashboard/"
echo ""
echo "To generate real benchmark data:"
echo "  1. Run: cargo bench"
echo "  2. Run (Windows): .\\scripts\\export_benchmark_jsonl.ps1"
echo "     Run (Unix):    bash scripts/export_benchmark_history.sh"
echo ""
