import os

file_path = r"examples/unified_showcase/src/main.rs"

with open(file_path, 'r', encoding='utf-8') as f:
    content = f.read()

# 1. Water Mesh
old_water = 'let water_mesh = self.create_plane_mesh(100.0, water_mat);'
new_water = 'let water_mesh = self.create_river_mesh(water_mat);'
if old_water in content:
    content = content.replace(old_water, new_water)
else:
    print("Water mesh line not found")

# 2. Trees - Paths
content = content.replace('"assets/models/tree_default.glb"', '"../../assets/models/tree_default.glb"')
content = content.replace('"assets/models/tree_oak.glb"', '"../../assets/models/tree_oak.glb"')
content = content.replace('"assets/models/tree_pineDefaultA.glb"', '"../../assets/models/tree_pineDefaultA.glb"')
content = content.replace('"assets/models/tree_detailed.glb"', '"../../assets/models/tree_detailed.glb"')

# Trees - Scale
old_scale = 'let scale = (0.8 + ((fx + fz) * 0.1).sin().abs() * 0.4) * 20.0; // x20 scale'
new_scale = 'let scale = (0.8 + ((fx + fz) * 0.1).sin().abs() * 0.4) * 2.0; // x2.0 scale'
if old_scale in content:
    content = content.replace(old_scale, new_scale)
else:
    print("Tree scale line not found")

# Trees - Density
old_density = 'if density > 0.3 {'
new_density = 'if density > 0.1 {'
if old_density in content:
    content = content.replace(old_density, new_density)
else:
    print("Density line not found")

# 3. Tents - Paths
content = content.replace('"assets/models/tent_detailedClosed.glb"', '"../../assets/models/tent_detailedClosed.glb"')
content = content.replace('"assets/models/tent_detailedOpen.glb"', '"../../assets/models/tent_detailedOpen.glb"')

# Tents - Positions
old_tent_pos = """        let tent_positions = [
            Vec3::new(5.0, 5.0, 5.0),
            Vec3::new(-6.0, 5.0, 3.0),
            Vec3::new(2.0, 5.0, -7.0),
            Vec3::new(-4.0, 5.0, -5.0),
        ];"""
new_tent_pos = """        let tent_positions = [
            Vec3::new(20.0, 0.0, 20.0),
            Vec3::new(-20.0, 0.0, 20.0),
            Vec3::new(20.0, 0.0, -20.0),
            Vec3::new(-20.0, 0.0, -20.0),
        ];"""
if old_tent_pos in content:
    content = content.replace(old_tent_pos, new_tent_pos)
else:
    # Try with normalized newlines if mismatch
    print("Tent positions not found")

# Tents - Placement Logic
# Replace position: *tent_pos,
# Replace bind group creation
content = content.replace('position: *tent_pos,', 'position: Vec3::new(tent_pos.x, self.calculate_terrain_height(tent_pos.x, tent_pos.z), tent_pos.z),')
content = content.replace('Vec3::splat(20.0), Quat::from_rotation_y(rot_y), *tent_pos', 'Vec3::splat(20.0), Quat::from_rotation_y(rot_y), Vec3::new(tent_pos.x, self.calculate_terrain_height(tent_pos.x, tent_pos.z), tent_pos.z)')

# 4. Tower
content = content.replace('"assets/models/tower.glb"', '"../../assets/models/tower.glb"')
content = content.replace('scale: Vec3::splat(20.0), // x20 scale (matching tree scale)', 'scale: Vec3::splat(5.0), // x5 scale')
content = content.replace('Vec3::splat(20.0), Quat::IDENTITY, Vec3::new(0.0, peak_height, 0.0)', 'Vec3::splat(5.0), Quat::IDENTITY, Vec3::new(0.0, peak_height, 0.0)')

with open(file_path, 'w', encoding='utf-8') as f:
    f.write(content)
