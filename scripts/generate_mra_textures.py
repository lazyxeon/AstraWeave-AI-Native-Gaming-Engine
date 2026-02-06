#!/usr/bin/env python3
"""
AstraWeave MRA Texture Generator
=================================
Generates physically-based MRA (Metallic/Roughness/AO) textures for all materials.

Channel layout (matches engine expectations):
  R = Metallic  (0.0 = dielectric, 1.0 = metal)
  G = Roughness (0.0 = mirror, 1.0 = matte)
  B = AO        (0.0 = fully occluded, 1.0 = no occlusion)
  A = 255       (unused, fully opaque)

Each material gets physically-accurate PBR values based on real-world measurements.
"""

import os
import sys
import random
import math
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFilter
except ImportError:
    print("ERROR: Pillow not installed. Run: pip install Pillow")
    sys.exit(1)

# Texture resolution - matches engine's internal 1024x1024 pipeline
RESOLUTION = 1024

# ============================================================================
# Physically-based material definitions
# ============================================================================
# Values based on real-world PBR references:
# - https://google.github.io/filament/Filament.md.html
# - Substance 3D material library
# ============================================================================

MATERIAL_PRESETS = {
    # --- Natural terrain ---
    "grass": {
        "metallic": 0.0,
        "roughness": 0.85,
        "ao_base": 0.90,
        "variation": 0.08,  # Natural variation in roughness/AO
        "description": "Green grass - non-metallic, fairly rough"
    },
    "dirt": {
        "metallic": 0.0,
        "roughness": 0.90,
        "ao_base": 0.85,
        "variation": 0.10,
        "description": "Loose dirt/soil - very rough, some AO in cracks"
    },
    "sand": {
        "metallic": 0.0,
        "roughness": 0.80,
        "ao_base": 0.95,
        "variation": 0.06,
        "description": "Fine sand - slightly less rough than dirt"
    },
    "stone": {
        "metallic": 0.0,
        "roughness": 0.75,
        "ao_base": 0.88,
        "variation": 0.12,
        "description": "Rough stone - moderate roughness, crevice AO"
    },
    "forest_floor": {
        "metallic": 0.0,
        "roughness": 0.92,
        "ao_base": 0.80,
        "variation": 0.10,
        "description": "Forest floor - leaves/twigs, very rough, heavy AO"
    },

    # --- Rock variants ---
    "rock_lichen": {
        "metallic": 0.0,
        "roughness": 0.82,
        "ao_base": 0.85,
        "variation": 0.10,
        "description": "Lichen-covered rock - organic coating varies roughness"
    },
    "rock_slate": {
        "metallic": 0.0,
        "roughness": 0.60,
        "ao_base": 0.90,
        "variation": 0.08,
        "description": "Slate rock - smoother than average rock"
    },

    # --- Building materials ---
    "plaster": {
        "metallic": 0.0,
        "roughness": 0.78,
        "ao_base": 0.92,
        "variation": 0.06,
        "description": "Wall plaster - medium rough, minimal AO"
    },
    "roof_tile": {
        "metallic": 0.0,
        "roughness": 0.65,
        "ao_base": 0.88,
        "variation": 0.08,
        "description": "Ceramic roof tile - somewhat glossy, crevice AO"
    },
    "cloth": {
        "metallic": 0.0,
        "roughness": 0.88,
        "ao_base": 0.92,
        "variation": 0.05,
        "description": "Cloth fabric - rough woven surface"
    },

    # --- Vegetation ---
    "tree_bark": {
        "metallic": 0.0,
        "roughness": 0.90,
        "ao_base": 0.78,
        "variation": 0.12,
        "description": "Tree bark - very rough with deep crevice AO"
    },
    "tree_leaves": {
        "metallic": 0.0,
        "roughness": 0.70,
        "ao_base": 0.85,
        "variation": 0.08,
        "description": "Tree leaves - waxy surface, moderate roughness"
    },

    # --- Default fallback ---
    "default": {
        "metallic": 0.0,
        "roughness": 0.50,
        "ao_base": 1.0,
        "variation": 0.0,
        "description": "Neutral default - mid-roughness, no AO"
    },
}


def clamp(val: float, lo: float = 0.0, hi: float = 1.0) -> float:
    return max(lo, min(hi, val))


def generate_perlin_noise(width: int, height: int, scale: float = 64.0, seed: int = 42) -> list:
    """Generate simple value noise for natural-looking variation."""
    random.seed(seed)

    # Create grid of random values
    grid_w = int(width / scale) + 2
    grid_h = int(height / scale) + 2
    grid = [[random.random() for _ in range(grid_w)] for _ in range(grid_h)]

    noise = []
    for y in range(height):
        row = []
        for x in range(width):
            # Bilinear interpolation
            gx = x / scale
            gy = y / scale
            ix = int(gx)
            iy = int(gy)
            fx = gx - ix
            fy = gy - iy

            # Smoothstep for smoother interpolation
            fx = fx * fx * (3.0 - 2.0 * fx)
            fy = fy * fy * (3.0 - 2.0 * fy)

            ix = min(ix, grid_w - 2)
            iy = min(iy, grid_h - 2)

            v00 = grid[iy][ix]
            v10 = grid[iy][ix + 1]
            v01 = grid[iy + 1][ix]
            v11 = grid[iy + 1][ix + 1]

            v0 = v00 + (v10 - v00) * fx
            v1 = v01 + (v11 - v01) * fx
            val = v0 + (v1 - v0) * fy

            row.append(val)
        noise.append(row)
    return noise


def generate_mra_texture(
    material_name: str,
    preset: dict,
    resolution: int = RESOLUTION,
) -> Image.Image:
    """Generate a physically-based MRA texture with natural variation."""

    metallic_val = preset["metallic"]
    roughness_val = preset["roughness"]
    ao_base = preset["ao_base"]
    variation = preset["variation"]

    # Generate noise layers at different frequencies for natural look
    noise_low = generate_perlin_noise(resolution, resolution, scale=128.0, seed=hash(material_name) & 0xFFFF)
    noise_med = generate_perlin_noise(resolution, resolution, scale=48.0, seed=(hash(material_name) + 1) & 0xFFFF)
    noise_hi = generate_perlin_noise(resolution, resolution, scale=16.0, seed=(hash(material_name) + 2) & 0xFFFF)

    img = Image.new("RGBA", (resolution, resolution))
    pixels = img.load()

    for y in range(resolution):
        for x in range(resolution):
            # Combine noise at multiple frequencies
            n = (noise_low[y][x] * 0.5 + noise_med[y][x] * 0.35 + noise_hi[y][x] * 0.15)
            n = (n - 0.5) * 2.0  # Normalize to [-1, 1]

            # Metallic: almost always 0 for dielectric materials
            m = clamp(metallic_val + n * variation * 0.1)

            # Roughness: varies naturally based on noise
            r = clamp(roughness_val + n * variation)

            # AO: noise creates subtle depth variation, with more in crevices
            ao_noise = (noise_hi[y][x] - 0.5) * variation * 1.5
            ao = clamp(ao_base + ao_noise)

            pixels[x, y] = (
                int(m * 255),
                int(r * 255),
                int(ao * 255),
                255,
            )

    return img


def generate_flat_normal() -> Image.Image:
    """Generate a proper flat normal map (128, 128, 255) in tangent space."""
    img = Image.new("RGBA", (RESOLUTION, RESOLUTION), (128, 128, 255, 255))
    return img


def generate_default_albedo(material_name: str) -> Image.Image:
    """Generate a neutral, non-flat albedo for materials missing albedo textures."""

    # Sensible default colors per material type
    color_map = {
        "grass":        (120, 160, 80),
        "dirt":         (140, 110, 80),
        "sand":         (210, 195, 160),
        "stone":        (145, 140, 135),
        "forest_floor": (100, 90, 60),
        "rock_lichen":  (130, 140, 110),
        "rock_slate":   (120, 120, 130),
        "plaster":      (220, 215, 200),
        "roof_tile":    (180, 100, 70),
        "cloth":        (160, 150, 140),
        "tree_bark":    (95, 75, 55),
        "tree_leaves":  (80, 130, 50),
        "default":      (180, 180, 180),
    }

    base_color = color_map.get(material_name, (180, 180, 180))

    img = Image.new("RGBA", (RESOLUTION, RESOLUTION))
    noise = generate_perlin_noise(RESOLUTION, RESOLUTION, scale=64.0, seed=hash(material_name + "_albedo") & 0xFFFF)
    pixels = img.load()

    for y in range(RESOLUTION):
        for x in range(RESOLUTION):
            n = (noise[y][x] - 0.5) * 30  # Subtle color variation
            r = int(clamp(base_color[0] + n, 0, 255))
            g = int(clamp(base_color[1] + n, 0, 255))
            b = int(clamp(base_color[2] + n, 0, 255))
            pixels[x, y] = (r, g, b, 255)

    return img


def main():
    repo_root = Path(__file__).parent.parent
    materials_dir = repo_root / "assets" / "materials"

    if not materials_dir.exists():
        print(f"ERROR: Materials directory not found: {materials_dir}")
        sys.exit(1)

    print("=" * 60)
    print("AstraWeave MRA Texture Generator")
    print("=" * 60)
    print(f"Resolution: {RESOLUTION}x{RESOLUTION}")
    print(f"Materials dir: {materials_dir}")
    print(f"Materials to process: {len(MATERIAL_PRESETS)}")
    print()

    generated = 0
    skipped = 0

    for material_name, preset in MATERIAL_PRESETS.items():
        mra_path = materials_dir / f"{material_name}_mra.png"
        old_size = mra_path.stat().st_size if mra_path.exists() else 0

        # Generate MRA texture
        print(f"  Generating {material_name}_mra.png ... ", end="", flush=True)
        print(f"(M={preset['metallic']:.1f}, R={preset['roughness']:.2f}, AO={preset['ao_base']:.2f})")

        mra_img = generate_mra_texture(material_name, preset)
        mra_img.save(str(mra_path), "PNG", optimize=True)

        new_size = mra_path.stat().st_size
        generated += 1

        print(f"    → {new_size:,} bytes (was {old_size:,} bytes)")

        # Check if albedo exists and is valid
        albedo_path = materials_dir / f"{material_name}.png"
        if not albedo_path.exists() or albedo_path.stat().st_size < 500:
            print(f"  Generating {material_name}.png (default albedo) ...", flush=True)
            albedo_img = generate_default_albedo(material_name)
            albedo_img.save(str(albedo_path), "PNG", optimize=True)
            generated += 1

        # Check if normal exists and is valid
        normal_path = materials_dir / f"{material_name}_n.png"
        if not normal_path.exists() or normal_path.stat().st_size < 500:
            print(f"  Generating {material_name}_n.png (flat normal) ...", flush=True)
            normal_img = generate_flat_normal()
            normal_img.save(str(normal_path), "PNG", optimize=True)
            generated += 1

    print()
    print("=" * 60)
    print(f"Generated: {generated} textures")
    print(f"Resolution: {RESOLUTION}x{RESOLUTION} RGBA PNG")
    print("=" * 60)
    print()
    print("Next steps:")
    print("  1. Run: cargo run -p aw-asset-cli -- bake-texture assets/materials/")
    print("     to compress to KTX2 for GPU upload")
    print("  2. Or let the engine load PNGs directly (slower but works)")


if __name__ == "__main__":
    main()
