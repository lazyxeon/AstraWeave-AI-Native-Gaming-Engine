#!/usr/bin/env python3
"""
Generate static benchmark graphs (PNG) for the GH Pages dashboard.
Reads JSONL history file (Criterion export) and produces:
 - top_series_time.png: Time-series for top N benchmarks (by latest value)
 - distribution_latest.png: Histogram of latest snapshot values across all benchmarks
 - heatmap.png: Heatmap of benchmarks (rows) vs time (columns) values
Requires: pandas, matplotlib, seaborn
Usage: python scripts/generate_benchmark_graphs.py --input target/benchmark-data/history.jsonl --out-dir gh-pages/graphs
"""
import argparse
import os
import json
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from datetime import datetime

def read_jsonl(path):
    records = []
    with open(path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                data = json.loads(line)
                records.append(data)
            except Exception:
                continue
    return pd.DataFrame(records)

def ensure_dir(path):
    os.makedirs(path, exist_ok=True)

def plot_top_series(df, out_dir, top_n=8):
    # Extract latest per benchmark
    latest = df.sort_values('timestamp').groupby('benchmark_name').last().reset_index()
    top = latest.sort_values('value', ascending=False).head(top_n)
    top_names = top['benchmark_name'].tolist()

    plt.figure(figsize=(12, 6))
    for name in top_names:
        s = df[df['benchmark_name'] == name].sort_values('timestamp')
        # Use display_name if available, otherwise benchmark_name
        display_name = s['display_name'].iloc[0] if 'display_name' in s.columns and not pd.isna(s['display_name'].iloc[0]) else name
        plt.plot(pd.to_datetime(s['timestamp']), s['value'], label=display_name)

    plt.legend()
    plt.title('Top Benchmarks Time Series')
    plt.xlabel('Date')
    plt.ylabel('Time (ns)')
    plt.tight_layout()
    plt.savefig(os.path.join(out_dir, 'top_series_time.png'), dpi=150)
    plt.close()

def plot_distribution(df, out_dir):
    latest = df.sort_values('timestamp').groupby('benchmark_name').last().reset_index()
    plt.figure(figsize=(10, 4))
    sns.histplot(latest['value'], bins=40, kde=True, color='#4facfe')
    plt.title('Latest Snapshot Distribution of Values')
    plt.xlabel('Time (ns)')
    plt.tight_layout()
    plt.savefig(os.path.join(out_dir, 'distribution_latest.png'), dpi=150)
    plt.close()

def plot_heatmap(df, out_dir, max_benchmarks=30, max_columns=48):
    # Build pivot: rows = benchmark_name, columns = timestamp
    df_sorted = df.sort_values('timestamp')
    
    # Create display name mapping
    display_map = {}
    if 'display_name' in df_sorted.columns:
        for _, row in df_sorted.groupby('benchmark_name').first().iterrows():
            display_map[row['benchmark_name']] = row.get('display_name', row['benchmark_name']) if not pd.isna(row.get('display_name')) else row['benchmark_name']
    else:
        display_map = {name: name for name in df_sorted['benchmark_name'].unique()}
    
    # Pivot to have timestamps as columns, values as cells
    pivot = df_sorted.pivot_table(index='benchmark_name', columns='timestamp', values='value')
    # Limit to top max_benchmarks by median value
    pivot['median_val'] = pivot.median(axis=1, skipna=True)
    pivot = pivot.sort_values('median_val', ascending=False).head(max_benchmarks)
    pivot = pivot.drop(columns=['median_val'])
    # Reduce columns to most recent max_columns
    pivot = pivot.iloc[:, -max_columns:]
    
    # Replace index with display names
    pivot.index = [display_map.get(name, name) for name in pivot.index]

    plt.figure(figsize=(12, 8))
    sns.heatmap(pivot.fillna(0), cmap='viridis', cbar_kws={'label': 'Time (ns)'})
    plt.title('Benchmark Heatmap (most recent snapshots)')
    plt.xlabel('Timestamp')
    plt.ylabel('Benchmark')
    plt.tight_layout()
    plt.savefig(os.path.join(out_dir, 'heatmap.png'), dpi=150)
    plt.close()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', default='target/benchmark-data/history.jsonl', help='JSONL history input file')
    parser.add_argument('--out-dir', default='gh-pages/graphs', help='Output directory for graphs')
    args = parser.parse_args()

    if not os.path.exists(args.input):
        print(f'Input file {args.input} not found')
        return 1

    df = read_jsonl(args.input)
    if df.empty:
        print('No data found in input file')
        return 1

    # Normalize timestamp format
    if 'timestamp' in df.columns:
        df['timestamp'] = pd.to_datetime(df['timestamp'])

    ensure_dir(args.out_dir)
    plot_top_series(df, args.out_dir)
    plot_distribution(df, args.out_dir)
    plot_heatmap(df, args.out_dir)

    print('Generated graphs in', args.out_dir)
    return 0

if __name__ == '__main__':
    exit(main())
