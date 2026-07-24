#!/usr/bin/env python3
"""Per-channel statistics for terrain material aux maps (T.2a Phase 1).

Answers one question: *is this channel real data or a placeholder?*

Why not just standard deviation
-------------------------------
A naive `sd < 2.0` test is wrong in both directions on this pack, measured:

* `assets/materials/grass_mra.png` roughness has sd 13.43 and looks healthy,
  but 99.5% of its pixels are exactly 255 — the sd is manufactured entirely by
  a 0.37% tail of near-zero pixels bleeding in from the source's foliage
  alpha cutout. It is the single worst channel in the pack and sd clears it.
* `derived_1k/gravel_mra.png` roughness has sd 1.43 and `derived_1k/beach_mra.png`
  has sd 2.25 — both fail an sd test, and both are *faithful* to genuinely
  uniform source scans (`gravel_concrete_03`, `coast_sand_01`).

So the primary detector here is **modal fraction** (share of pixels taking the
single most common value) with **IQR** as the corroborating signal. Standard
deviation is reported but never decides.

Channel semantics
-----------------
On-disk `*_mra.png` is **M**etallic-R / **R**oughness-G / **A**O-B; the loader
(`tools/aw_editor/src/viewport/canonical_terrain_pack.rs::load_mra_as_orm_bytes`)
swaps R and B to produce the ORM packing the terrain shader consumes. An `orm`
key loads verbatim (AO-R / roughness-G / metallic-B).

**Metallic is expected to be a hard constant 0** on every terrain layer —
terrain is dielectric. That is the post-AD.4.A-D1 health invariant, not a
defect, and this script reports it as `constant-by-design` rather than flagging
it. A blanket "zero variance means placeholder" sweep would destroy it and
re-introduce the mirror-terrain regression.

Runtime vs disk
---------------
The loader unconditionally resizes normal and ORM to `CANONICAL_AUX_RES` (512)
with a triangle filter, so the variance the *shader* sees is lower than the
variance on disk. `--aux-res 512` measures the downsampled buffer, which is the
honest number when reporting a data fix's effect. Both are printed by default.

Usage
-----
    python tools/material_cook/channel_stats.py                    # the live 8-slot pack
    python tools/material_cook/channel_stats.py --pack <dir>
    python tools/material_cook/channel_stats.py --files a.png b.png
    python tools/material_cook/channel_stats.py --disk-only       # skip the 512 pass
"""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

import numpy as np
from PIL import Image

# The loader's aux clamp (canonical_terrain_pack.rs CANONICAL_AUX_RES).
DEFAULT_AUX_RES = 512

# A channel is a placeholder when a single value owns this share of the image.
MODAL_FLAT_THRESHOLD = 0.90

# MRA on disk. (Index -> label.)
MRA_CHANNELS = ("metallic", "roughness", "ao")
ORM_CHANNELS = ("ao", "roughness", "metallic")


def load_rgba(path: Path) -> np.ndarray:
    """Read an image as float64 RGBA, 0-255, without the 16-bit clamp.

    `Image.convert("RGBA")` silently clamps 16-bit modes (I;16 / I;16B /
    I;16L / I) to 255 — the AD.4.A "D2" bug. Cooked outputs are 8-bit so this
    matters little here, but the same helper is used to inspect sources.
    """
    img = Image.open(path)
    if img.mode in ("I;16", "I;16B", "I;16L", "I"):
        img = img.point(lambda v: v / 257.0).convert("L")
    return np.asarray(img.convert("RGBA")).astype(np.float64)


def resized(path: Path, res: int) -> np.ndarray:
    """The buffer the shader actually samples: aux maps downsampled to `res`.

    PIL BILINEAR is the closest match to the loader's `FilterType::Triangle`.
    """
    img = Image.open(path)
    if img.mode in ("I;16", "I;16B", "I;16L", "I"):
        img = img.point(lambda v: v / 257.0).convert("L")
    img = img.convert("RGBA")
    if img.size != (res, res):
        img = img.resize((res, res), Image.BILINEAR)
    return np.asarray(img).astype(np.float64)


def channel_stats(a: np.ndarray) -> dict:
    """mean / sd / IQR / p95-p5 / modal fraction for one channel plane."""
    flat = a.ravel()
    q1, q3 = np.percentile(flat, [25, 75])
    p5, p95 = np.percentile(flat, [5, 95])
    vals, counts = np.unique(flat.astype(np.uint8), return_counts=True)
    modal = counts.max() / flat.size
    return {
        "mean": float(flat.mean()),
        "sd": float(flat.std()),
        "iqr": float(q3 - q1),
        "span": float(p95 - p5),
        "modal": float(modal),
        "uniq": int(vals.size),
    }


def verdict(name: str, s: dict) -> str:
    """Classify a channel. Metallic is exempt — constant 0 is the contract."""
    if name == "metallic":
        return "constant-by-design" if s["uniq"] == 1 and s["mean"] == 0.0 else "CHECK (metallic should be 0)"
    if s["uniq"] == 1:
        return "FLAT (constant)"
    if s["modal"] > MODAL_FLAT_THRESHOLD or s["iqr"] == 0.0:
        return "FLAT (degenerate)"
    if s["sd"] < 3.0:
        return "low-variance"
    return "real"


def resolve_pack(pack_dir: Path) -> list[tuple[int, str, Path, tuple[str, ...]]]:
    """Resolve a biome pack's aux maps to (slot, key, path, channel-names)."""
    materials = pack_dir / "materials.toml"
    arrays = pack_dir / "arrays.toml"
    with materials.open("rb") as f:
        mats = tomllib.load(f)
    slot_of: dict[str, int] = {}
    if arrays.is_file():
        with arrays.open("rb") as f:
            arr = tomllib.load(f)
        # arrays.toml maps layer key -> array index; accept either a flat table
        # or a nested one, whichever the file uses.
        for k, v in arr.items():
            if isinstance(v, int):
                slot_of[k] = v
            elif isinstance(v, dict):
                for k2, v2 in v.items():
                    if isinstance(v2, int):
                        slot_of[k2] = v2

    out = []
    for i, layer in enumerate(mats.get("layer", [])):
        key = layer.get("key", f"layer{i}")
        slot = slot_of.get(key, i)
        if "orm" in layer:
            out.append((slot, key, (pack_dir / layer["orm"]).resolve(), ORM_CHANNELS))
        elif "mra" in layer:
            out.append((slot, key, (pack_dir / layer["mra"]).resolve(), MRA_CHANNELS))
    out.sort(key=lambda r: r[0])
    return out


def emit(rows: list[tuple[str, str, Path, tuple[str, ...]]], aux_res: int | None) -> int:
    """Print a markdown table. Returns the number of FLAT channels found."""
    header = "| slot | layer | channel | mean | sd | IQR | p95-p5 | modal% | uniq | verdict |"
    print(header)
    print("|---|---|---|---|---|---|---|---|---|---|")
    flat_count = 0
    for slot, key, path, chans in rows:
        if not path.is_file():
            print(f"| {slot} | {key} | — | — | — | — | — | — | — | **MISSING {path}** |")
            flat_count += 1
            continue
        img = resized(path, aux_res) if aux_res else load_rgba(path)
        for idx, cname in enumerate(chans):
            s = channel_stats(img[..., idx])
            v = verdict(cname, s)
            if v.startswith("FLAT"):
                flat_count += 1
            print(
                f"| {slot} | {key} | {cname} | {s['mean']:.2f} | {s['sd']:.2f} | "
                f"{s['iqr']:.0f} | {s['span']:.0f} | {s['modal'] * 100:.1f}% | "
                f"{s['uniq']} | {v} |"
            )
    return flat_count


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument(
        "--pack",
        default="assets/materials/biomes",
        help="biome pack directory containing materials.toml (default: the live 8-slot pack)",
    )
    ap.add_argument("--files", nargs="*", help="measure these images instead of a pack")
    ap.add_argument(
        "--channels",
        choices=["mra", "orm"],
        default="mra",
        help="channel order for --files (default: mra)",
    )
    ap.add_argument("--aux-res", type=int, default=DEFAULT_AUX_RES)
    ap.add_argument(
        "--disk-only",
        action="store_true",
        help="report only the on-disk resolution (skip the runtime-downsampled pass)",
    )
    args = ap.parse_args()

    if args.files:
        chans = MRA_CHANNELS if args.channels == "mra" else ORM_CHANNELS
        rows = [(i, Path(f).stem, Path(f), chans) for i, f in enumerate(args.files)]
    else:
        pack = Path(args.pack)
        if not (pack / "materials.toml").is_file():
            print(f"no materials.toml under {pack}", file=sys.stderr)
            return 2
        rows = resolve_pack(pack)

    print(f"### On disk\n")
    flat_disk = emit(rows, None)
    if not args.disk_only:
        print(f"\n### As sampled at runtime (aux resized to {args.aux_res}²)\n")
        emit(rows, args.aux_res)

    print(f"\n{flat_disk} flat channel(s) on disk "
          f"(modal > {MODAL_FLAT_THRESHOLD * 100:.0f}% or IQR == 0; metallic exempt).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
