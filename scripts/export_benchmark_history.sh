#!/bin/bash
#
# Export Benchmark History to JSONL
# Week 4 Action 15: Benchmark Dashboard Automation
#
# This script exports benchmark history from GitHub Pages (gh-pages branch)
# into a JSONL (JSON Lines) format for dashboard visualization.
#
# Output: docs/benchmark_data/benchmark_history.jsonl
#

set -euo pipefail

# Configuration
GH_PAGES_BRANCH="${GH_PAGES_BRANCH:-gh-pages}"
BENCH_DATA_DIR="${BENCH_DATA_DIR:-dev/bench}"
OUTPUT_DIR="${OUTPUT_DIR:-docs/benchmark_data}"
OUTPUT_FILE="${OUTPUT_FILE:-benchmark_history.jsonl}"
MAX_ENTRIES="${MAX_ENTRIES:-100}"  # Max entries per benchmark (30 days @ 3 runs/day = ~90)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Main script
main() {
    info "=== AstraWeave Benchmark History Export ==="
    
    # Check prerequisites
    if ! command -v git &> /dev/null; then
        error "git is not installed"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        error "jq is not installed (required for JSON processing)"
        exit 1
    fi
    
    # Create output directory
    mkdir -p "$OUTPUT_DIR"
    info "Output directory: $OUTPUT_DIR"
    
    # Get current branch
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
    info "Current branch: $CURRENT_BRANCH"
    
    # Check if gh-pages branch exists
    if ! git show-ref --verify --quiet "refs/heads/$GH_PAGES_BRANCH" && \
       ! git show-ref --verify --quiet "refs/remotes/origin/$GH_PAGES_BRANCH"; then
        warn "Branch '$GH_PAGES_BRANCH' not found - no historical data available"
        warn "Creating empty history file"
        echo -n "" > "$OUTPUT_DIR/$OUTPUT_FILE"
        success "Created empty history file: $OUTPUT_DIR/$OUTPUT_FILE"
        exit 0
    fi
    
    # Fetch latest gh-pages data
    info "Fetching latest data from $GH_PAGES_BRANCH..."
    git fetch origin "$GH_PAGES_BRANCH":"$GH_PAGES_BRANCH" 2>/dev/null || \
        git fetch origin "$GH_PAGES_BRANCH" 2>/dev/null || \
        warn "Could not fetch $GH_PAGES_BRANCH (may not exist remotely yet)"
    
    # Create temporary directory for checkout
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    info "Extracting benchmark data from $GH_PAGES_BRANCH..."
    
    # Export data using git archive (works even if branch not checked out)
    if git show "$GH_PAGES_BRANCH:$BENCH_DATA_DIR/data.js" > "$TEMP_DIR/data.js" 2>/dev/null; then
        info "Found benchmark data in $BENCH_DATA_DIR/data.js"
    elif git show "$GH_PAGES_BRANCH:data.js" > "$TEMP_DIR/data.js" 2>/dev/null; then
        info "Found benchmark data in root data.js"
    else
        warn "No benchmark data found in $GH_PAGES_BRANCH"
        warn "Creating empty history file"
        echo -n "" > "$OUTPUT_DIR/$OUTPUT_FILE"
        success "Created empty history file: $OUTPUT_DIR/$OUTPUT_FILE"
        exit 0
    fi
    
    # Parse data.js (it's a JavaScript file with window.BENCHMARK_DATA = {...})
    info "Parsing benchmark data..."
    
    # Extract JSON from JavaScript
    # Format: window.BENCHMARK_DATA = {"entries": {...}}
    if grep -q "window.BENCHMARK_DATA" "$TEMP_DIR/data.js"; then
        # Remove JavaScript wrapper to get pure JSON
        sed -n 's/^window\.BENCHMARK_DATA = //p' "$TEMP_DIR/data.js" | \
            sed 's/;$//' > "$TEMP_DIR/data.json"
    else
        # Assume it's already JSON
        cp "$TEMP_DIR/data.js" "$TEMP_DIR/data.json"
    fi
    
    # Validate JSON
    if ! jq empty "$TEMP_DIR/data.json" 2>/dev/null; then
        error "Invalid JSON in benchmark data"
        cat "$TEMP_DIR/data.json" | head -20
        exit 1
    fi
    
    # Extract entries
    ENTRY_COUNT=$(jq '.entries | length' "$TEMP_DIR/data.json" 2>/dev/null || echo "0")
    
    if [ "$ENTRY_COUNT" -eq 0 ]; then
        warn "No benchmark entries found in data"
        echo -n "" > "$OUTPUT_DIR/$OUTPUT_FILE"
        success "Created empty history file: $OUTPUT_DIR/$OUTPUT_FILE"
        exit 0
    fi
    
    info "Found $ENTRY_COUNT benchmark entry groups"
    
    # Convert to JSONL format
    # Structure: Each line is a JSON object with {commit, date, benchmarks: [...]}
    info "Converting to JSONL format..."
    
    > "$OUTPUT_DIR/$OUTPUT_FILE"  # Clear file
    
    # Process each entry (commit/date group)
    jq -c '.entries | to_entries | .[] | 
        {
            name: .key,
            commits: .value
        }' "$TEMP_DIR/data.json" | while IFS= read -r entry; do
        
        # Each entry has multiple commits
        BENCHMARK_NAME=$(echo "$entry" | jq -r '.name')
        
        echo "$entry" | jq -c '.commits[] | {
            benchmark: "'$BENCHMARK_NAME'",
            commit: .commit.id,
            date: .commit.timestamp,
            author: .commit.author,
            message: .commit.message,
            benchmarks: .benches
        }' >> "$OUTPUT_DIR/$OUTPUT_FILE"
    done
    
    # Count lines in output
    LINE_COUNT=$(wc -l < "$OUTPUT_DIR/$OUTPUT_FILE")
    
    if [ "$LINE_COUNT" -eq 0 ]; then
        warn "No data was exported (empty JSONL)"
    else
        success "Exported $LINE_COUNT benchmark snapshots to $OUTPUT_DIR/$OUTPUT_FILE"
        
        # Show sample
        info "Sample (first entry):"
        head -1 "$OUTPUT_DIR/$OUTPUT_FILE" | jq '.'
    fi
    
    # Generate summary statistics
    info "Generating summary statistics..."
    
    # Count unique benchmarks
    UNIQUE_BENCHMARKS=$(jq -r '.benchmark' "$OUTPUT_DIR/$OUTPUT_FILE" | sort -u | wc -l)
    
    # Date range
    OLDEST_DATE=$(jq -r '.date' "$OUTPUT_DIR/$OUTPUT_FILE" | sort | head -1)
    NEWEST_DATE=$(jq -r '.date' "$OUTPUT_DIR/$OUTPUT_FILE" | sort | tail -1)
    
    # Create metadata file
    cat > "$OUTPUT_DIR/metadata.json" <<EOF
{
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "source_branch": "$GH_PAGES_BRANCH",
  "total_snapshots": $LINE_COUNT,
  "unique_benchmarks": $UNIQUE_BENCHMARKS,
  "date_range": {
    "oldest": "$OLDEST_DATE",
    "newest": "$NEWEST_DATE"
  },
  "max_entries_per_benchmark": $MAX_ENTRIES,
  "version": "1.0"
}
EOF
    
    success "Generated metadata: $OUTPUT_DIR/metadata.json"
    
    info "=== Export Complete ==="
    echo ""
    echo "📊 Summary:"
    echo "   - Total snapshots: $LINE_COUNT"
    echo "   - Unique benchmarks: $UNIQUE_BENCHMARKS"
    echo "   - Date range: $OLDEST_DATE to $NEWEST_DATE"
    echo "   - Output file: $OUTPUT_DIR/$OUTPUT_FILE"
    echo "   - Metadata: $OUTPUT_DIR/metadata.json"
    echo ""
    echo "Next steps:"
    echo "1. Review output: jq '.' $OUTPUT_DIR/$OUTPUT_FILE | less"
    echo "2. Open dashboard: open docs/benchmark_dashboard/index.html"
    echo "3. Deploy to GitHub Pages: git add docs/ && git commit && git push"
}

# Run main function
main "$@"
