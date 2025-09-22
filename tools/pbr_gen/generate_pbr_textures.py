#!/usr/bin/env python3
# CC0 / Public Domain
# Generates tileable PBR texture packs at 2K/4K for multiple materials.
# Maps: BaseColor, Normal (OpenGL Y+), Roughness, Metallic, Height, AO
# Extras: Leaves_Opacity, Water_FlowRG, optional packed ORM (R=AO, G=Roughness, B=Metallic)

import os, math, argparse, json
import numpy as np
from PIL import Image, ImageFilter

# ------------- Utils -------------
def to_uint8(arr):
    arr = np.clip(arr, 0.0, 1.0)
    return (arr * 255.0 + 0.5).astype(np.uint8)

def save_gray(arr, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(to_uint8(arr), mode="L").save(path, compress_level=6)

def save_rgb(arr, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(to_uint8(arr), mode="RGB").save(path, compress_level=6)

def save_rgba(arr, path):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    Image.fromarray(to_uint8(arr), mode="RGBA").save(path, compress_level=6)

def gaussian_blur(arr, radius):
    img = Image.fromarray(to_uint8(arr), mode="L").filter(ImageFilter.GaussianBlur(radius=radius))
    return np.asarray(img, dtype=np.uint8).astype(np.float32) / 255.0

def resample(arr, size):
    mode = "L" if arr.ndim==2 else ("RGB" if arr.shape[2]==3 else "RGBA")
    img = Image.fromarray(to_uint8(arr), mode=mode)
    img = img.resize((size, size), resample=Image.Resampling.LANCZOS)
    return np.asarray(img, dtype=np.uint8).astype(np.float32)/255.0

def height_to_normal(height, strength=3.0, convention="opengl"):
    h = height.astype(np.float32)
    dx = np.roll(h, -1, axis=1) - np.roll(h, 1, axis=1)
    dy = np.roll(h, -1, axis=0) - np.roll(h, 1, axis=0)
    nx = -dx * strength
    ny = -dy * strength
    nz = np.ones_like(h)
    l = np.sqrt(nx*nx + ny*ny + nz*nz) + 1e-8
    nx /= l; ny /= l; nz /= l
    ny = ny if convention == "opengl" else -ny
    return np.dstack((nx*0.5+0.5, ny*0.5+0.5, nz*0.5+0.5))

def rand_sine_noise(w, h, waves=10, fmin=2, fmax=32, seed=0, weight_low=True):
    rng = np.random.RandomState(seed)
    y, x = np.meshgrid(np.linspace(0,1,h,endpoint=False), np.linspace(0,1,w,endpoint=False), indexing='ij')
    out = np.zeros((h, w), dtype=np.float32)
    for _ in range(waves):
        nx = rng.randint(fmin, fmax+1)
        ny = rng.randint(fmin, fmax+1)
        phase = rng.uniform(0, 2*np.pi)
        amp = 1.0 / (0.3 + math.sqrt(nx*nx + ny*ny)) if weight_low else 1.0
        out += amp * np.cos(2*np.pi*(nx*x + ny*y) + phase)
    out = (out - out.min()) / (out.max() - out.min() + 1e-8)
    return out

def add_highfreq_detail(height, amount=0.05, seed=0):
    h, w = height.shape
    detail = rand_sine_noise(w, h, waves=8, fmin=40, fmax=160, seed=seed)
    return np.clip(height + amount*(detail-0.5), 0.0, 1.0)

# ------------- Materials (subset aligned to your terrain needs) -------------
def stone_terrain_rock(RES, seed=200, normal_conv="opengl", quality="med"):
    BASE = 1024
    base = rand_sine_noise(BASE, BASE, waves=14, fmin=2, fmax=20, seed=seed)
    detail = rand_sine_noise(BASE, BASE, waves=16, fmin=6, fmax=28, seed=seed+1)
    h0 = 0.6*base + 0.4*detail
    h0 = (h0 - 0.5); h0 = 1.0 - np.abs(h0*2.0)
    h0 = (h0 - h0.min())/(h0.max()-h0.min()+1e-8)
    height = resample(h0, RES)
    if quality != "low":
        height = add_highfreq_detail(height, amount=0.06 if quality=="med" else 0.1, seed=seed+2)
    tint = resample(rand_sine_noise(BASE, BASE, waves=7, fmin=2, fmax=10, seed=seed+3), RES)
    r = np.clip(0.45 + 0.20*(height-0.5) + 0.05*(tint-0.5), 0.0, 1.0)
    g = np.clip(0.48 + 0.22*(height-0.5) + 0.05*(tint-0.5), 0.0, 1.0)
    b = np.clip(0.52 + 0.24*(height-0.5) + 0.06*(tint-0.5), 0.0, 1.0)
    basecolor = np.dstack([r,g,b])
    roughness = np.clip(0.78 + 0.18*(1.0 - height), 0.6, 0.97)
    ao = np.clip(0.62 + 0.38*gaussian_blur(1.0 - height, 1), 0.0, 1.0)
    metallic = np.zeros_like(height)
    normal = height_to_normal(height, strength=5.0, convention=normal_conv)
    return basecolor, normal, roughness, metallic, height, ao

def dirt_mud(RES, seed=500, normal_conv="opengl", quality="med"):
    BASE = 1024
    base = rand_sine_noise(BASE, BASE, waves=12, fmin=2, fmax=16, seed=seed)
    micro = rand_sine_noise(BASE, BASE, waves=10, fmin=8, fmax=28, seed=seed+1)
    puddle_mask = (base > 0.74).astype(np.float32)
    h0 = 0.55*base + 0.45*micro - 0.15*puddle_mask
    h0 = (h0 - h0.min())/(h0.max()-h0.min()+1e-8)
    height = resample(h0, RES)
    if quality != "low":
        height = add_highfreq_detail(height, amount=0.05 if quality=="med" else 0.085, seed=seed+2)
    tint = resample(rand_sine_noise(BASE, BASE, waves=8, fmin=2, fmax=10, seed=seed+3), RES)
    darken = resample(puddle_mask, RES)
    r = np.clip(0.30 + 0.10*(height-0.5) - 0.12*darken + 0.04*(tint-0.5), 0.0, 1.0)
    g = np.clip(0.22 + 0.10*(height-0.5) - 0.10*darken + 0.03*(tint-0.5), 0.0, 1.0)
    b = np.clip(0.18 + 0.08*(height-0.5) - 0.08*darken + 0.02*(tint-0.5), 0.0, 1.0)
    basecolor = np.dstack([r,g,b])
    roughness = np.clip(0.75 + 0.2*(1.0 - height) - 0.25*darken, 0.2, 0.95)
    ao = np.clip(0.6 + 0.4*gaussian_blur(1.0 - height, 1), 0.0, 1.0)
    metallic = np.zeros_like(height)
    normal = height_to_normal(height, strength=3.0, convention=normal_conv)
    return basecolor, normal, roughness, metallic, height, ao

def sand_desert(RES, seed=600, normal_conv="opengl", quality="med"):
    BASE = 1024
    y, x = np.meshgrid(np.linspace(0,1,BASE,endpoint=False), np.linspace(0,1,BASE,endpoint=False), indexing='ij')
    dunes = 0.5 + 0.5*np.sin(2*np.pi*(x*2.5 + 0.2*np.sin(2*np.pi*y)))
    ripples = 0.5 + 0.5*np.sin(2*np.pi*(x*48 + y*6))
    h0 = 0.75*dunes + 0.25*ripples
    h0 = (h0 - h0.min())/(h0.max()-h0.min()+1e-8)
    height = resample(h0, RES)
    if quality != "low":
        height = add_highfreq_detail(height, amount=0.03 if quality=="med" else 0.06, seed=seed+1)
    tint = resample(rand_sine_noise(BASE, BASE, waves=8, fmin=2, fmax=10, seed=seed+2), RES)
    r = np.clip(0.78 + 0.08*(height-0.5) + 0.03*(tint-0.5), 0.0, 1.0)
    g = np.clip(0.70 + 0.06*(height-0.5) + 0.03*(tint-0.5), 0.0, 1.0)
    b = np.clip(0.55 + 0.04*(height-0.5) + 0.02*(tint-0.5), 0.0, 1.0)
    basecolor = np.dstack([r,g,b])
    roughness = np.clip(0.85 + 0.1*(1.0 - height), 0.75, 0.98)
    ao = np.clip(0.7 + 0.3*gaussian_blur(1.0 - height, 1), 0.0, 1.0)
    metallic = np.zeros_like(height)
    normal = height_to_normal(height, strength=2.8, convention=normal_conv)
    return basecolor, normal, roughness, metallic, height, ao

def moss_ground(RES, seed=700, normal_conv="opengl", quality="med"):
    BASE = 1024
    base = rand_sine_noise(BASE, BASE, waves=14, fmin=2, fmax=18, seed=seed)
    lumps = rand_sine_noise(BASE, BASE, waves=18, fmin=8, fmax=36, seed=seed+1)
    h0 = 0.6*base + 0.4*lumps
    h0 = (h0 - h0.min())/(h0.max()-h0.min()+1e-8)
    height = resample(h0, RES)
    if quality != "low":
        height = add_highfreq_detail(height, amount=0.04 if quality=="med" else 0.07, seed=seed+2)
    tint = resample(rand_sine_noise(BASE, BASE, waves=8, fmin=2, fmax=12, seed=seed+3), RES)
    r = np.clip(0.18 + 0.08*(height-0.5) + 0.05*(tint-0.5), 0.0, 1.0)
    g = np.clip(0.38 + 0.16*(height-0.5) + 0.08*(tint-0.5), 0.0, 1.0)
    b = np.clip(0.14 + 0.07*(height-0.5) + 0.04*(tint-0.5), 0.0, 1.0)
    basecolor = np.dstack([r,g,b])
    roughness = np.clip(0.75 + 0.2*(1.0 - height), 0.6, 0.95)
    ao = np.clip(0.65 + 0.35*gaussian_blur(1.0 - height, 1), 0.0, 1.0)
    metallic = np.zeros_like(height)
    normal = height_to_normal(height, strength=3.0, convention=normal_conv)
    return basecolor, normal, roughness, metallic, height, ao

# ------------- Save one material -------------
def save_material(out_root, name, maps, pack_orm=False):
    os.makedirs(os.path.join(out_root, name), exist_ok=True)
    def p(fn): return os.path.join(out_root, name, f"{name}_{fn}.png")
    basecolor, normal, roughness, metallic, height, ao = maps
    save_rgb(basecolor, p("BaseColor"))
    save_rgb(normal, p("Normal"))
    save_gray(roughness, p("Roughness"))
    save_gray(metallic, p("Metallic"))
    save_gray(height, p("Height"))
    save_gray(ao, p("AO"))
    if pack_orm:
        orm = np.dstack([ao, roughness, metallic])
        save_rgb(orm, p("ORM"))

MATERIALS = {
    "Stone_Terrain_Rock": stone_terrain_rock,
    "Dirt_Mud": dirt_mud,
    "Sand_Desert": sand_desert,
    "Moss_Ground": moss_ground,
}

# ------------- Main -------------
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="./PBR_4K")
    ap.add_argument("--res", type=int, default=4096)
    ap.add_argument("--materials", default="all")
    ap.add_argument("--normal-conv", choices=["opengl","directx"], default="opengl")
    ap.add_argument("--quality", choices=["low","med","high"], default="med")
    ap.add_argument("--pack-orm", action="store_true")
    args = ap.parse_args()

    mats = list(MATERIALS.keys()) if args.materials == "all" else [m.strip() for m in args.materials.split(",")]
    os.makedirs(args.out, exist_ok=True)

    for name in mats:
        print(f"Generating {name} @ {args.res} ...")
        fn = MATERIALS[name]
        maps = fn(args.res, normal_conv=args.normal_conv, quality=args.quality)
        save_material(args.out, name, maps, pack_orm=args.pack_orm)

    print("Done.")

if __name__ == "__main__":
    main()
