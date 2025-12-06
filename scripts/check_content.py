import os

file_path = r"examples/unified_showcase/src/main.rs"

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

idx = content.find("create_plane_mesh(100.0, water_mat)")
print(f"Found at index: {idx}")

if idx != -1:
    print("Context:")
    print(content[idx-50:idx+50])
else:
    print("String not found in content.")
    # Print some potential near matches
    print("Checking for create_plane_mesh...")
    idx2 = content.find("create_plane_mesh")
    print(f"Found generic at: {idx2}")
    if idx2 != -1:
         print(content[idx2-50:idx2+50])
