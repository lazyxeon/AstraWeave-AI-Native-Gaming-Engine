#!/bin/bash
set -euo pipefail
echo "=== Test: Generate benchmark graphs ==="
INPUT=${1:-target/benchmark-data/history.jsonl}
OUT=${2:-gh-pages/graphs}
python3 scripts/generate_benchmark_graphs.py --input "$INPUT" --out-dir "$OUT"
for f in top_series_time.png distribution_latest.png heatmap.png; do
  if [ ! -f "$OUT/$f" ]; then
    echo "Missing generated file: $OUT/$f"; exit 1
  fi
done
echo "All graphs generated successfully in $OUT"
