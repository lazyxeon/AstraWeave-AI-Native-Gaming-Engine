#!/usr/bin/env python3
# Map generated PBR outputs into engine's naming in ./assets
# - Copies/renames BaseColor->name.png, Normal->name_n.png, ORM->name_mra.png, and writes black name_e.png
# - Optionally resamples to a given resolution

import os, argparse
from PIL import Image

MAP = {
    "grass": "Moss_Ground",
    "forest_floor": "Moss_Ground",
    "dirt": "Dirt_Mud",
    "sand": "Sand_Desert",
    "stone": "Stone_Terrain_Rock",
}

SUFFIXES = {
    "albedo": ("BaseColor", "png"),
    "normal": ("Normal", "png"),
    "orm": ("ORM", "png"),
}

def load_and_resample(path, res=None):
    img = Image.open(path)
    if res is not None:
        img = img.resize((res, res), Image.Resampling.LANCZOS)
    return img

def write_black(dst_path, res=2048):
    Image.new("RGB", (res, res), (0,0,0)).save(dst_path, compress_level=6)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--src", required=True, help="Root of PBR_4K")
    ap.add_argument("--dst", default="assets", help="Engine assets directory")
    ap.add_argument("--res", type=int, default=2048, help="Output resolution")
    ap.add_argument("--normal-conv", choices=["opengl","directx"], default="opengl")
    # Overrides for mapping
    for k in list(MAP.keys()):
        ap.add_argument(f"--{k}", default=MAP[k])
    args = ap.parse_args()

    # Update mapping from CLI
    mapping = {k: getattr(args, k) for k in MAP.keys()}

    os.makedirs(args.dst, exist_ok=True)

    for out_name, src_name in mapping.items():
        src_dir = os.path.join(args.src, src_name)
        base_src = os.path.join(src_dir, f"{src_name}_{SUFFIXES['albedo'][0]}.png")
        norm_src = os.path.join(src_dir, f"{src_name}_{SUFFIXES['normal'][0]}.png")
        orm_src  = os.path.join(src_dir, f"{src_name}_{SUFFIXES['orm'][0]}.png")
        if not (os.path.exists(base_src) and os.path.exists(norm_src) and os.path.exists(orm_src)):
            raise SystemExit(f"Missing maps for {src_name} in {src_dir}")

        # Albedo
        base = load_and_resample(base_src, args.res)
        base.save(os.path.join(args.dst, f"{out_name}.png"), compress_level=6)
        # Normal (assumed OpenGL from generator). If directx requested, flip G channel.
        normal = load_and_resample(norm_src, args.res).convert("RGB")
        if args.normal_conv == "directx":
            r,g,b = normal.split()
            g = Image.eval(g, lambda v: 255 - v)
            normal = Image.merge("RGB", (r,g,b))
        normal.save(os.path.join(args.dst, f"{out_name}_n.png"), compress_level=6)
        # MRA from ORM
        orm = load_and_resample(orm_src, args.res).convert("RGB")
        orm.save(os.path.join(args.dst, f"{out_name}_mra.png"), compress_level=6)
        # Emissive black
        write_black(os.path.join(args.dst, f"{out_name}_e.png"), args.res)
        print(f"Wrote {out_name} maps to {args.dst}")

    print("Done.")

if __name__ == "__main__":
    main()
