import os

file_path = r"examples/unified_showcase/src/main.rs"

with open(file_path, 'r', encoding='utf-8') as f:
    lines = f.readlines()
    # Print lines 840-900
    start = 840
    end = 900
    for i in range(start, end):
        if i < len(lines):
            print(f"{i+1}: {lines[i].rstrip()}")
