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


def to_l8(img):
    """Bit-depth-safe grayscale conversion (AD.4.A fix, D2). PIL's
    `convert("L")` on 16-bit modes (I;16 / I;16B / I;16L / I) CLAMPS values
    >255 to 255, silently flattening real data to solid white — this destroyed
    derived_1k/plaster_mra.png's AO and tree_bark_mra.png's roughness (both
    PolyHaven 2k sources are mode I;16). Scale 16-bit to 8-bit first."""
    if img.mode in ("I;16", "I;16B", "I;16L", "I"):
        img = img.point(lambda v: v / 257.0).convert("L")
        return img
    return img.convert("L")


def pack_mra(roughness_path: str, ao_path: str, out_path: str) -> tuple:
    """Build an MRA map (R=metallic=0, G=roughness, B=ao) from separate PolyHaven
    roughness + ao maps, at 1024x1024 RGBA PNG. Used for re-acquired families
    (C7) where the source ships separate maps rather than a packed set. Channel
    order matches the engine `_mra.png` convention (canonical_terrain_pack
    swizzles mra→ORM at load)."""
    from PIL import Image as _I
    r = to_l8(_I.open(roughness_path))
    a = to_l8(_I.open(ao_path))
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


def _l8_array(path: str, size: int = SIZE):
    """Bit-depth-safe single-channel read, resized to `size`, as float 0..1."""
    import numpy as np
    from PIL import Image as _I

    img = to_l8(_I.open(path))
    if img.size != (size, size):
        img = img.resize((size, size), _I.LANCZOS)
    return np.asarray(img).astype("float32") / 255.0


def _ao_curve(height, sigma: float = 2.0):
    """Shared AO shaping: soften the height field and map it to [0.55, 1.0].

    **Orientation (T.2a fix).** High displacement means an exposed peak, which
    is LESS occluded, so AO must rise with height. `scripts/import_terrain_
    textures.py::build_mra` used `1.0 - hf`, i.e. it darkened peaks and lit
    crevices — inverted. Measured against same-scan ground truth (a real AO
    map and its own displacement map, resampled to 1024 and correlated):
    `ganges_river_pebbles` r=+0.478 upright vs -0.478 inverted,
    `forest_leaves_02` +0.230, `aerial_rocks_01` +0.122; the remaining
    same-scan pairs on disk are near-flat displacement and discriminate
    neither way. The inversion reached the shipped pack: live
    `assets/materials/mud_mra.png`'s AO correlates **+0.991** with the
    inverted formula and **-0.991** with the upright one.

    `sigma` is in output pixels. The original ran at 2048 with sigma 4 and the
    result was later downscaled to 1024, so 2.0 here reproduces the same
    spatial scale at the 1K contract.
    """
    import numpy as np
    from scipy.ndimage import gaussian_filter

    ao = 0.55 + 0.45 * gaussian_filter(height, sigma=sigma)
    return np.clip(ao, 0.0, 1.0)


def ao_from_displacement(disp_path: str, size: int = SIZE):
    """Derive AO from a displacement/height map (the preferred source)."""
    return _ao_curve(_l8_array(disp_path, size))


def ao_from_albedo_cavity(albedo_path: str, size: int = SIZE, gain: float = 0.15):
    """Derive AO from albedo *local* relief, for sources with no height map.

    A plain luminance→AO mapping would bake the albedo's own low-frequency
    variation into the occlusion term, and since the shader multiplies the two,
    that double-darkens wherever the texture is already dark. So only the
    high-frequency component is used: luminance minus its own heavy blur,
    normalized to unit variance, recentred at 0.5. What survives is local
    cavity structure — for a grass scan, the dark gaps between blades against
    the lit tips — which is what AO is meant to encode.

    This is an approximation and is only used where the upstream genuinely
    ships no displacement map. Stated per-material in the outcome note rather
    than applied silently.
    """
    import numpy as np
    from PIL import Image as _I
    from scipy.ndimage import gaussian_filter

    img = _I.open(albedo_path).convert("RGB")
    if img.size != (size, size):
        img = img.resize((size, size), _I.LANCZOS)
    a = np.asarray(img).astype("float32") / 255.0
    lum = 0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2]
    hp = lum - gaussian_filter(lum, sigma=32.0)
    hp = hp / (hp.std() + 1e-6)
    cavity = np.clip(0.5 + gain * hp, 0.0, 1.0)
    return _ao_curve(cavity)


def roughness_from_mra(mra_path: str, size: int = SIZE):
    """Lift the roughness (G) channel out of an existing MRA map, as 0..1.

    For surgical repairs where a family's roughness is already correct and only
    its AO is being replaced. Reading the file through the generic grayscale
    path would mix all three channels into a luminance and silently corrupt the
    roughness it was supposed to preserve.
    """
    import numpy as np
    from PIL import Image as _I

    # In-place safe: materialize a detached copy and close the source handle
    # before the caller writes back over the same path (Windows sharing
    # violation otherwise — same hazard `cook_one` documents).
    with _I.open(mra_path) as src:
        img = src.convert("RGBA")
    if img.size != (size, size):
        img = img.resize((size, size), _I.LANCZOS)
    return np.asarray(img)[..., 1].astype("float32") / 255.0


def ao_from_normal_map(normal_path: str, size: int = SIZE):
    """Derive AO from a tangent-space normal map by reconstructing its height.

    Preferred over `ao_from_albedo_cavity` whenever the family ships a real
    normal map but no displacement, because the recovered height is
    *spatially registered* with the relief the shader actually shades — AO
    darkens the same crevices the normal map bends light into. An albedo-based
    cavity is only registered with the albedo, which may come from a different
    source entirely.

    Method: Frankot-Chellappa integration. Decode the normal to slopes
    `p = -nx/nz`, `q = +ny/nz`, then solve the Poisson equation in the
    frequency domain,

        H(u,v) = (-i·u·P(u,v) - i·v·Q(u,v)) / (u² + v²),   H(0,0) = 0

    and inverse-transform. FFT integration assumes periodicity, which is
    exactly right here: these are tiling textures. The result is normalized to
    [0,1] before the shared AO curve is applied, so only *relative* relief
    matters — the arbitrary integration constant is discarded with H(0,0).

    **The `q` sign is calibrated, not assumed.** OpenGL normal maps put +Y up
    in green while image rows run downward, so the y slope does not take the
    same sign as x. All four sign combinations were scored against ground
    truth — recovered height correlated with the family's own real
    displacement map, on the two live families that ship one:

    | slopes | mountain (rock_face_03) | swamp (forrest_ground_01) |
    |---|---|---|
    | `p=-nx/nz, q=-ny/nz` | -0.122 | -0.196 |
    | **`p=-nx/nz, q=+ny/nz`** | **+0.651** | **+0.632** |
    | `p=+nx/nz, q=-ny/nz` | -0.651 | -0.632 |
    | `p=+nx/nz, q=+ny/nz` | +0.122 | +0.196 |

    Two independent scans agree, so the convention is settled rather than
    guessed. Correlation is ~0.65 rather than ~1.0 because a normal map and a
    displacement map capture different frequency bands of the same surface —
    this recovers relief structure, not the displacement map itself.
    """
    import numpy as np
    from PIL import Image as _I

    img = _I.open(normal_path).convert("RGB")
    if img.size != (size, size):
        img = img.resize((size, size), _I.LANCZOS)
    n = np.asarray(img).astype("float64") / 255.0 * 2.0 - 1.0
    nz = np.clip(n[..., 2], 1e-3, None)
    p = -n[..., 0] / nz
    q = n[..., 1] / nz

    h_, w_ = p.shape
    wy = np.fft.fftfreq(h_)[:, None] * 2.0 * np.pi
    wx = np.fft.fftfreq(w_)[None, :] * 2.0 * np.pi
    denom = wx * wx + wy * wy
    denom[0, 0] = 1.0  # H(0,0) := 0; the integration constant is arbitrary
    num = -1j * wx * np.fft.fft2(p) - 1j * wy * np.fft.fft2(q)
    height = np.real(np.fft.ifft2(num / denom))
    height[0, 0] = height.mean()

    lo, hi = np.percentile(height, [1, 99])
    height = np.clip((height - lo) / (hi - lo + 1e-12), 0.0, 1.0)
    return _ao_curve(height)


def write_mra(roughness_path, ao, out_path: str) -> tuple:
    """Write an MRA map (R=metallic=0, G=roughness, B=AO) leaving the family's
    albedo and normal untouched.

    `ao` is either a path to an AO map or a float 0..1 array from one of the
    `ao_from_*` derivations above. Used for surgical channel repairs where
    re-cooking the whole family would churn maps that are already correct.
    """
    import numpy as np
    from PIL import Image as _I

    if isinstance(roughness_path, str):
        rough = (_l8_array(roughness_path) * 255.0).round().astype("uint8")
    else:
        rough = (np.asarray(roughness_path) * 255.0).round().astype("uint8")
    if isinstance(ao, str):
        ao_u8 = (_l8_array(ao) * 255.0).round().astype("uint8")
    else:
        ao_u8 = (np.asarray(ao) * 255.0).round().astype("uint8")
    metal = np.zeros_like(rough)
    alpha = np.full_like(rough, 255)
    stacked = np.dstack([metal, rough, ao_u8, alpha])
    out = _I.fromarray(stacked, "RGBA")
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    out.save(out_path, "PNG")
    with _I.open(out_path) as o:
        return (o.size[0], o.size[1], o.mode, os.path.getsize(out_path))


def cook_mra_arm_to_mra(src_path: str, out_path: str) -> tuple:
    """AD.4.A fix (D1): cook an ARM-mislabeled `_mra.png` into a true MRA map.

    The 2026-05 A.1 acquisition fetched PolyHaven ARM maps (R=AO, G=roughness,
    B=metallic) and renamed them `<family>_mra.png` in assets_src/materials/
    WITHOUT reordering channels. The engine loader's mra->ORM swizzle
    (canonical_terrain_pack.rs:204-210) assumes true MRA and swaps R<->B, so
    ARM-ordered input reaches the shader inverted: AO=metallic(~0) kills the
    ambient term, metallic=AO(~high) turns terrain into a mirror. Fix at cook
    time: swap R<->B (ARM -> MRA), then downscale to the 1K contract.

    Guarded: refuses to swap unless the source actually shows the ARM profile
    (R varying/high AND B flat ~0) so a true-MRA input can never be corrupted."""
    im = Image.open(src_path)
    d = im.convert("RGBA")
    im.close()
    # profile guard (sampled stats, cheap): ARM = R mean high, B mean ~ 0
    small = d.resize((64, 64))
    px = list(small.getdata())
    r_mean = sum(p[0] for p in px) / len(px)
    b_mean = sum(p[2] for p in px) / len(px)
    if not (r_mean > 100.0 and b_mean < 10.0):
        raise SystemExit(
            f"REFUSED: {src_path} does not match the ARM profile "
            f"(R mean {r_mean:.1f}, B mean {b_mean:.1f}) — not swapping")
    r, g, b, a = d.split()
    d = Image.merge("RGBA", (b, g, r, a))  # ARM -> MRA: R(AO) <-> B(metal)
    if d.size != (SIZE, SIZE):
        d = d.resize((SIZE, SIZE), Image.LANCZOS)
    os.makedirs(os.path.dirname(out_path) or ".", exist_ok=True)
    d.save(out_path, "PNG")
    with Image.open(out_path) as o:
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
    ap.add_argument("--mra-only", action="store_true",
                    help="rebuild ONLY the _mra map; --out is then the full output path")
    ap.add_argument("--rough", help="roughness source (with --mra-only)")
    ap.add_argument("--rough-from-mra",
                    help="preserve the G channel of this existing MRA as roughness "
                         "(with --mra-only; for AO-only repairs)")
    ap.add_argument("--ao", help="AO map source (with --mra-only)")
    ap.add_argument("--ao-disp", help="derive AO from this displacement map (with --mra-only)")
    ap.add_argument("--ao-cavity", help="derive AO from this albedo's local relief (with --mra-only)")
    ap.add_argument("--ao-normal",
                    help="derive AO by integrating this normal map to a height field "
                         "(with --mra-only; preferred when the family has no displacement)")
    args = ap.parse_args()

    if args.mra_only:
        if bool(args.rough) == bool(args.rough_from_mra):
            ap.error("--mra-only requires exactly one of --rough / --rough-from-mra")
        sources = [s for s in (args.ao, args.ao_disp, args.ao_cavity, args.ao_normal) if s]
        if len(sources) != 1:
            ap.error("--mra-only requires exactly one of "
                     "--ao / --ao-disp / --ao-normal / --ao-cavity")
        if args.rough:
            rough, rough_how = args.rough, args.rough
        else:
            rough = roughness_from_mra(args.rough_from_mra)
            rough_how = f"{args.rough_from_mra} (G channel preserved)"
        if args.ao:
            ao, how = args.ao, f"ao map {args.ao}"
        elif args.ao_disp:
            ao, how = ao_from_displacement(args.ao_disp), f"AO from displacement {args.ao_disp}"
        elif args.ao_normal:
            ao, how = ao_from_normal_map(args.ao_normal), f"AO from normal-map height {args.ao_normal}"
        else:
            ao, how = ao_from_albedo_cavity(args.ao_cavity), f"AO from albedo cavity {args.ao_cavity}"
        w, h, mode, nbytes = write_mra(rough, ao, args.out)
        print(f"  {os.path.basename(args.out):32} {w}x{h} {mode} {nbytes:,}B")
        print(f"    roughness <- {rough_how}")
        print(f"    {how}")
        return

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
