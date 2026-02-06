#!/usr/bin/env python3
"""
AstraWeave Extended Material Generator
=======================================
Generates additional PBR material sets for biomes not yet covered:
  - snow (tundra/mountain peaks)
  - ice (frozen surfaces)
  - mountain_rock (exposed mountain stone)
  - mud (wet terrain)
  - gravel (loose stones)
  - moss (wet organic growth)
  - cobblestone (urban paths)
  - wood_planks (structures)
  - metal_rusted (props/structures)
  
Each material gets: albedo.png, normal.png (procedural), mra.png
"""

import os
import sys
import random
import math
from pathlib import Path

try:
    from PIL import Image
except ImportError:
    print("ERROR: Pillow not installed. Run: pip install Pillow")
    sys.exit(1)

RESOLUTION = 1024

def clamp(val, lo=0.0, hi=1.0):
    return max(lo, min(hi, val))

def noise_layer(w, h, scale=64.0, seed=42):
    random.seed(seed)
    gw = int(w / scale) + 2
    gh = int(h / scale) + 2
    grid = [[random.random() for _ in range(gw)] for _ in range(gh)]
    out = []
    for y in range(h):
        row = []
        for x in range(w):
            gx, gy = x / scale, y / scale
            ix, iy = min(int(gx), gw - 2), min(int(gy), gh - 2)
            fx = (gx - ix); fx = fx * fx * (3 - 2 * fx)
            fy = (gy - iy); fy = fy * fy * (3 - 2 * fy)
            v = (grid[iy][ix] * (1 - fx) + grid[iy][ix + 1] * fx) * (1 - fy) + \
                (grid[iy + 1][ix] * (1 - fx) + grid[iy + 1][ix + 1] * fx) * fy
            row.append(v)
        out.append(row)
    return out

def multi_octave_noise(w, h, octaves=4, seed=42):
    result = [[0.0] * w for _ in range(h)]
    amp = 1.0
    freq = 1.0
    total_amp = 0.0
    for o in range(octaves):
        n = noise_layer(w, h, scale=max(8, 128 / freq), seed=seed + o * 1000)
        for y in range(h):
            for x in range(w):
                result[y][x] += n[y][x] * amp
        total_amp += amp
        amp *= 0.5
        freq *= 2.0
    for y in range(h):
        for x in range(w):
            result[y][x] /= total_amp
    return result

EXTENDED_MATERIALS = {
    "snow": {
        "color": (235, 240, 245),
        "color_var": 8,
        "metallic": 0.0,
        "roughness": 0.70,
        "ao_base": 0.95,
        "variation": 0.05,
    },
    "ice": {
        "color": (180, 210, 230),
        "color_var": 12,
        "metallic": 0.02,
        "roughness": 0.15,
        "ao_base": 0.98,
        "variation": 0.04,
    },
    "mountain_rock": {
        "color": (110, 105, 100),
        "color_var": 20,
        "metallic": 0.0,
        "roughness": 0.80,
        "ao_base": 0.82,
        "variation": 0.14,
    },
    "mud": {
        "color": (95, 75, 55),
        "color_var": 15,
        "metallic": 0.0,
        "roughness": 0.45,  # Wet = smoother
        "ao_base": 0.85,
        "variation": 0.10,
    },
    "gravel": {
        "color": (150, 145, 135),
        "color_var": 18,
        "metallic": 0.0,
        "roughness": 0.88,
        "ao_base": 0.80,
        "variation": 0.12,
    },
    "moss": {
        "color": (65, 100, 45),
        "color_var": 15,
        "metallic": 0.0,
        "roughness": 0.82,
        "ao_base": 0.88,
        "variation": 0.08,
    },
    "cobblestone": {
        "color": (135, 130, 120),
        "color_var": 20,
        "metallic": 0.0,
        "roughness": 0.72,
        "ao_base": 0.78,
        "variation": 0.12,
    },
    "wood_planks": {
        "color": (140, 105, 65),
        "color_var": 18,
        "metallic": 0.0,
        "roughness": 0.75,
        "ao_base": 0.85,
        "variation": 0.10,
    },
    "metal_rusted": {
        "color": (130, 80, 50),
        "color_var": 22,
        "metallic": 0.35,  # Partially metallic (rust reduces it)
        "roughness": 0.78,
        "ao_base": 0.82,
        "variation": 0.15,
    },
}


def generate_texture_set(name, preset, out_dir):
    """Generate albedo, normal, and MRA textures for a material."""
    seed_base = hash(name) & 0xFFFF
    noise1 = multi_octave_noise(RESOLUTION, RESOLUTION, octaves=4, seed=seed_base)
    noise2 = multi_octave_noise(RESOLUTION, RESOLUTION, octaves=4, seed=seed_base + 100)
    noise3 = multi_octave_noise(RESOLUTION, RESOLUTION, octaves=3, seed=seed_base + 200)
    
    # --- Albedo ---
    albedo = Image.new("RGBA", (RESOLUTION, RESOLUTION))
    pix = albedo.load()
    c = preset["color"]
    v = preset["color_var"]
    for y in range(RESOLUTION):
        for x in range(RESOLUTION):
            n = (noise1[y][x] - 0.5) * 2.0 * v
            r = int(clamp(c[0] + n, 0, 255))
            g = int(clamp(c[1] + n * 0.9, 0, 255))
            b = int(clamp(c[2] + n * 0.8, 0, 255))
            pix[x, y] = (r, g, b, 255)
    albedo.save(str(out_dir / f"{name}.png"), "PNG", optimize=True)

    # --- Normal map (from height derivative) ---
    normal = Image.new("RGBA", (RESOLUTION, RESOLUTION))
    pix = normal.load()
    strength = 2.0
    for y in range(RESOLUTION):
        for x in range(RESOLUTION):
            # Sobel-like derivative from noise
            x0 = noise1[y][(x - 1) % RESOLUTION]
            x1 = noise1[y][(x + 1) % RESOLUTION]
            y0 = noise1[(y - 1) % RESOLUTION][x]
            y1 = noise1[(y + 1) % RESOLUTION][x]
            dx = (x1 - x0) * strength
            dy = (y1 - y0) * strength
            # Normal = normalize(-dx, -dy, 1)
            mag = math.sqrt(dx * dx + dy * dy + 1.0)
            nx = (-dx / mag) * 0.5 + 0.5
            ny = (-dy / mag) * 0.5 + 0.5
            nz = (1.0 / mag) * 0.5 + 0.5
            pix[x, y] = (int(nx * 255), int(ny * 255), int(nz * 255), 255)
    normal.save(str(out_dir / f"{name}_n.png"), "PNG", optimize=True)

    # --- MRA (Metallic, Roughness, AO) ---
    mra = Image.new("RGBA", (RESOLUTION, RESOLUTION))
    pix = mra.load()
    m_base = preset["metallic"]
    r_base = preset["roughness"]
    ao_base = preset["ao_base"]
    var = preset["variation"]
    for y in range(RESOLUTION):
        for x in range(RESOLUTION):
            n = (noise2[y][x] - 0.5) * 2.0
            n2 = (noise3[y][x] - 0.5) * 2.0
            m = clamp(m_base + n * var * 0.2)
            r = clamp(r_base + n * var)
            ao = clamp(ao_base + n2 * var * 1.2)
            pix[x, y] = (int(m * 255), int(r * 255), int(ao * 255), 255)
    mra.save(str(out_dir / f"{name}_mra.png"), "PNG", optimize=True)


def main():
    repo_root = Path(__file__).parent.parent
    materials_dir = repo_root / "assets" / "materials"

    print("=" * 60)
    print("AstraWeave Extended Material Generator")
    print("=" * 60)
    print(f"Resolution: {RESOLUTION}x{RESOLUTION}")
    print(f"Materials: {len(EXTENDED_MATERIALS)}")
    print()

    for name, preset in EXTENDED_MATERIALS.items():
        print(f"  Generating {name} set (albedo + normal + MRA)...", flush=True)
        generate_texture_set(name, preset, materials_dir)
        # Report sizes
        sizes = []
        for suffix in ["", "_n", "_mra"]:
            p = materials_dir / f"{name}{suffix}.png"
            sizes.append(f"{p.stat().st_size // 1024}KB")
        print(f"    → {', '.join(sizes)}")

    print()
    print(f"Generated {len(EXTENDED_MATERIALS) * 3} texture files")
    print("=" * 60)


if __name__ == "__main__":
    main()
