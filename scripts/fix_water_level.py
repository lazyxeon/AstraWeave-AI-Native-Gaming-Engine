import os

file_path = r"examples/unified_showcase/src/main.rs"

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

old_level = 'let water_level = base_height - 2.0;'
new_level = 'let water_level = base_height + 8.0;'

if old_level in content:
    content = content.replace(old_level, new_level)
    print("Fixed water level.")
    with open(file_path, 'w', encoding='utf-8') as f:
        f.write(content)
else:
    print("Water level line not found.")
