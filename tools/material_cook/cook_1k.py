#!/usr/bin/env python3
# CC0 / Public Domain
"""
AstraWeave 1K-derivative material cook (AD.4).

Takes a licensed source material's 3-map set (albedo / normal / mra) at any
resolution and emits the sample-set 1024x1024 derivative that fills a
`MaterialLibrary` slot: three RGBA PNGs `{out}.png`, `{out}_n.png`,
`{out}_mra.png`.

This is NOT `scripts/import_terrain_textures.py` (that builds 2048 maps from raw
Poly Haven albedo/normal/rough/disp for 9 families and writes into the repo tree).
This cook downscales an already-authored 3-map set to the 1K sample contract and
is source/destination agnostic (staging-safe).

Contract (asserted by test_cook_1k.py):
  - each output is 1024x1024
  - each output is mode RGBA
  - each output is a valid PNG
  - exactly three maps per family: albedo, _n, _mra

Usage:
  python cook_1k.py --albedo A.png --normal N.png --mra M.png --out DIR/name
  python cook_1k.py --src-dir assets_src/materials --name gravel --out DIR/gravel
"""

import argparse
import os
from PIL import Image

SIZE = 1024
MAPS = ("", "_n", "_mra")  # albedo, normal, mra


def cook_one(src_path: str, out_path: str) -> tuple:
    """Downscale one map to 1024x1024 RGBA PNG. Returns (w, h, mode, bytes).

    In-place safe: the source is fully materialized and its file handle closed
    before the output is written, so src_path == out_path (traced-9 in-place
    re-cook) cannot hit a Windows read/write sharing violation."""
    im = Image.open(src_path)
    d = im.convert("RGBA")  # materializes a detached copy
    im.close()
    if d.size != (SIZE, SIZE):
        d = d.resize((SIZE, SIZE), Image.LANCZOS)
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    d.save(out_path, "PNG")
    with Image.open(out_path) as o:
        return (o.size[0], o.size[1], o.mode, os.path.getsize(out_path))


def pack_mra(roughness_path: str, ao_path: str, out_path: str) -> tuple:
    """Build an MRA map (R=metallic=0, G=roughness, B=ao) from separate PolyHaven
    roughness + ao maps, at 1024x1024 RGBA PNG. Used for re-acquired families
    (C7) where the source ships separate maps rather than a packed set. Channel
    order matches the engine `_mra.png` convention (canonical_terrain_pack
    swizzles mra→ORM at load)."""
    from PIL import Image as _I
    r = _I.open(roughness_path).convert("L")
    a = _I.open(ao_path).convert("L")
    if r.size != (SIZE, SIZE):
        r = r.resize((SIZE, SIZE), _I.LANCZOS)
    if a.size != (SIZE, SIZE):
        a = a.resize((SIZE, SIZE), _I.LANCZOS)
    metal = _I.new("L", (SIZE, SIZE), 0)
    alpha = _I.new("L", (SIZE, SIZE), 255)
    mra = _I.merge("RGBA", (metal, r, a, alpha))
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    mra.save(out_path, "PNG")
    with _I.open(out_path) as o:
        return (o.size[0], o.size[1], o.mode, os.path.getsize(out_path))


def cook_family_from_maps(albedo: str, normal: str, roughness: str, ao: str,
                          out_base: str) -> dict:
    """Cook a re-acquired family (separate PolyHaven maps) to the 1K 3-map
    derivative: albedo→{out}.png, normal→{out}_n.png, pack(rough,ao)→{out}_mra.png."""
    res = {}
    w, h, mode, nbytes = cook_one(albedo, f"{out_base}.png")
    res[""] = dict(w=w, h=h, mode=mode, bytes=nbytes, src=albedo, out=f"{out_base}.png")
    w, h, mode, nbytes = cook_one(normal, f"{out_base}_n.png")
    res["_n"] = dict(w=w, h=h, mode=mode, bytes=nbytes, src=normal, out=f"{out_base}_n.png")
    w, h, mode, nbytes = pack_mra(roughness, ao, f"{out_base}_mra.png")
    res["_mra"] = dict(w=w, h=h, mode=mode, bytes=nbytes, src=f"{roughness}+{ao}", out=f"{out_base}_mra.png")
    return res


def cook_family(albedo: str, normal: str, mra: str, out_base: str) -> dict:
    """Cook a 3-map family. `out_base` is a path stem; maps get MAPS suffixes.
    Returns {suffix: (w,h,mode,bytes, src, out)}."""
    srcs = {"": albedo, "_n": normal, "_mra": mra}
    result = {}
    for sfx in MAPS:
        src = srcs[sfx]
        out = f"{out_base}{sfx}.png"
        w, h, mode, nbytes = cook_one(src, out)
        result[sfx] = dict(w=w, h=h, mode=mode, bytes=nbytes, src=src, out=out)
    return result


def _from_src_dir(src_dir: str, name: str) -> tuple:
    return (
        os.path.join(src_dir, f"{name}.png"),
        os.path.join(src_dir, f"{name}_n.png"),
        os.path.join(src_dir, f"{name}_mra.png"),
    )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--albedo")
    ap.add_argument("--normal")
    ap.add_argument("--mra")
    ap.add_argument("--src-dir")
    ap.add_argument("--name")
    ap.add_argument("--out", required=True, help="output path stem (no extension)")
    args = ap.parse_args()

    if args.src_dir and args.name:
        albedo, normal, mra = _from_src_dir(args.src_dir, args.name)
    elif args.albedo and args.normal and args.mra:
        albedo, normal, mra = args.albedo, args.normal, args.mra
    else:
        ap.error("provide either --src-dir + --name, or --albedo + --normal + --mra")

    res = cook_family(albedo, normal, mra, args.out)
    for sfx, r in res.items():
        print(f"  {os.path.basename(r['out']):32} {r['w']}x{r['h']} {r['mode']} "
              f"{r['bytes']:,}B  <- {r['src']}")


if __name__ == "__main__":
    main()
