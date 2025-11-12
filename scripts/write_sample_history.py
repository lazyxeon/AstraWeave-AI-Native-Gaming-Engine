#!/usr/bin/env python3
import json
import os

os.makedirs('target/benchmark-data', exist_ok=True)
records = [
    {"timestamp":"2025-10-10T10:00:00Z","benchmark_name":"astraweave-core::ecs_benchmarks/world_creation","value":125000,"stddev":5000,"unit":"ns","git_sha":"abc12345","git_branch":"main","git_dirty":False,"crate":"astraweave-core","group":"ecs_benchmarks","name":"world_creation"},
    {"timestamp":"2025-10-11T10:00:00Z","benchmark_name":"astraweave-core::ecs_benchmarks/world_creation","value":120000,"stddev":4800,"unit":"ns","git_sha":"def67890","git_branch":"main","git_dirty":False,"crate":"astraweave-core","group":"ecs_benchmarks","name":"world_creation"},
    {"timestamp":"2025-10-12T10:00:00Z","benchmark_name":"astraweave-ai::ai_core_loop/simple","value":980000,"stddev":12000,"unit":"ns","git_sha":"abc12345","git_branch":"main","git_dirty":False,"crate":"astraweave-ai","group":"ai_core_loop","name":"simple"}
]

with open('target/benchmark-data/history.jsonl', 'w', encoding='utf-8') as f:
    for rec in records:
        f.write(json.dumps(rec) + '\n')

print('Wrote sample history to target/benchmark-data/history.jsonl')
