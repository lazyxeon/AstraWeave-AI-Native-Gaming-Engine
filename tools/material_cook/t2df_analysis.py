#!/usr/bin/env python3
"""T.2d.F — before/after analysis for the material-LOD tier retirement.

Inputs are the station captures produced by
`tools/aw_editor/tests/t2df_stations.rs` (`t2df_capture_stations`), one
directory per leg (default `d:/tmp/t2df_staging/{before,after}`).

Three measurements, matching the diagnosis (T2D_CAMERA_LIGHT.md §10):

1. GRAIN metric on the boundary stations — the metric that actually detects the
   LOD1|2 boundary (row-mean luminance moves <2% across it; grain energy 68%).
   Reported at the predicted footprint-2.0 contour row and as a whole-profile
   sustained-rise search.

2. FAR-FIELD SHIMMER proxy — mean grain energy in the far field (LOD2 region
   pre-fix). The diagnosis measured LOD 2 *adding* 40-55% high-frequency energy
   as per-pixel tier dithering; after the deletion that energy should drop
   toward the pinned-tier (`mat_lod = 1u`) leg's level.

3. DISTANCE PROFILE 5 m → 1500 m — luma binned by true per-pixel 3D camera
   distance (ray/ground-plane intersection; row-averaging smears iso-distance
   curves and was retired in §9.2). Composited from the close/contour/far
   stations. The §3.2 multiscatter step (LOD0|1, footprint 0.5) should be GONE
   after; the far-field normal-variance gradient should SURVIVE (different
   mechanism, not this beat's to fix).

Usage:
    python tools/material_cook/t2df_analysis.py [before_dir] [after_dir]

With only before_dir present, prints the before numbers (Phase 0.3); with both,
prints the comparison (Phase 2).
"""

import json
import math
import os
import sys

import numpy as np
from PIL import Image
from scipy.ndimage import uniform_filter

FOVY = math.radians(60.0)

# Station geometry mirrors t2df_stations.rs::stations().
STATIONS = {
    "t2df_boundary_y414": {"pitch": 45.6, "cam_h": 378.5, "kind": "boundary"},
    "t2df_boundary_y536": {"pitch": 45.6, "cam_h": 500.2, "kind": "boundary"},
    "t2df_desert_close_20m": {"pitch": 55.0, "cam_h": 20.0 * math.sin(math.radians(55.0)), "kind": "profile"},
    "t2df_lod01_contour": {"pitch": 40.0, "cam_h": 30.0, "kind": "profile"},
    "t2df_profile_far": {"pitch": 35.0, "cam_h": 300.0, "kind": "profile"},
}


def luma(a):
    return 0.2126 * a[..., 0] + 0.7152 * a[..., 1] + 0.0722 * a[..., 2]


def load(path):
    return np.asarray(Image.open(path).convert("RGB")).astype(np.float64)


def grain_profile(L, x_frac=0.25, smooth=9):
    """Per-row grain energy: mean |L - 3x3 box| over the central columns."""
    HF = np.abs(L - uniform_filter(L, size=3))
    h, w = L.shape
    prof = HF[:, int(w * x_frac): int(w * (1 - x_frac))].mean(axis=1)
    return uniform_filter(prof, size=smooth)


def footprint_row(cam_h, pitch_deg, height_px, target):
    """Normalised screen row where the flat-ground pixel footprint crosses
    `target` (the compute_material_lod quantity); None if unreached."""
    k = 2.0 * math.tan(FOVY / 2.0) / height_px
    pitch = math.radians(pitch_deg)

    def foot(norm):
        ndc = 1.0 - 2.0 * norm
        alpha = pitch - math.atan(ndc * math.tan(FOVY / 2.0))
        s = math.sin(alpha)
        if s <= 1e-6:
            return float("inf")
        return cam_h * k * math.sqrt(1.0 + s * s) / (s * s)

    if foot(0.999) > target:
        return None
    lo, hi = 0.001, 0.999
    for _ in range(80):
        mid = 0.5 * (lo + hi)
        if foot(mid) > target:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def contour_step(prof, norm_row, halfwin_frac=0.12):
    """Grain means in bands just above/below a contour row."""
    h = len(prof)
    i = int(norm_row * h)
    win = max(int(h * halfwin_frac), 8)
    above = prof[max(i - win, 0): i]
    below = prof[i: min(i + win, h)]
    return above.mean(), below.mean()


def sustained_rise(prof, frac=0.125):
    """Largest sustained rise going down the screen (window = h*frac)."""
    h = len(prof)
    win = max(int(h * frac), 8)
    best, at = -1e18, 0
    for t in range(win, h - win):
        a = prof[t - win:t].mean()
        b = prof[t:t + win].mean()
        if b - a > best:
            best, at = b - a, t
    return at / h, best


def distance_bins(L, cam_h, pitch_deg, bins):
    """Mean luma binned by true 3D camera distance over a flat ground plane."""
    h, w = L.shape
    pitch = math.radians(pitch_deg)
    ty = math.tan(FOVY / 2.0)
    tx = ty * (w / h)
    ny = 1.0 - 2.0 * (np.arange(h) + 0.5) / h
    nx = 2.0 * (np.arange(w) + 0.5) / w - 1.0
    dy = (ny * ty)[:, None] * np.ones((1, w))
    dx = np.ones((h, 1)) * (nx * tx)[None, :]
    dz = -np.ones((h, w))
    cp, sp = math.cos(pitch), math.sin(pitch)
    wy = dy * cp + dz * sp
    # Unit rays; the ground hit's 3D distance is cam_h / (-y component).
    d = np.stack([dx, wy, dz], -1)
    d /= np.linalg.norm(d, axis=-1, keepdims=True)
    with np.errstate(divide="ignore", invalid="ignore"):
        dist3 = np.where(d[..., 1] < -1e-9, cam_h / (-d[..., 1]), np.nan)
    out = []
    for lo, hi in bins:
        m = (dist3 >= lo) & (dist3 < hi)
        if m.sum() > 200:
            out.append((lo, hi, float(L[m].mean()), int(m.sum())))
    return out


def analyse_leg(leg_dir):
    res = {}
    for name, geo in STATIONS.items():
        png = os.path.join(leg_dir, name + ".png")
        if not os.path.isfile(png):
            continue
        L = luma(load(png))
        h, w = L.shape
        prof = grain_profile(L)
        entry = {"shape": (w, h)}
        if geo["kind"] == "boundary":
            r20 = footprint_row(geo["cam_h"], geo["pitch"], h, 2.0)
            if r20 is not None:
                above, below = contour_step(prof, r20)
                entry["contour_norm"] = r20
                entry["grain_above"] = above
                entry["grain_below"] = below
            at, rise = sustained_rise(prof)
            entry["rise_at"] = at
            entry["rise"] = rise
            # Far-field shimmer proxy: grain in the top third (deep LOD2 pre-fix).
            entry["far_grain"] = float(prof[: h // 3].mean())
            entry["near_grain"] = float(prof[-h // 3:].mean())
        res[name] = entry
    return res


def print_leg(tag, res):
    print(f"=== {tag}")
    for name, e in sorted(res.items()):
        if "contour_norm" in e:
            print(
                f"  {name} {e['shape'][0]}x{e['shape'][1]}: footprint-2.0 contour at norm row "
                f"{e['contour_norm']:.3f} -> grain above {e['grain_above']:.3f} / below "
                f"{e['grain_below']:.3f} (x{e['grain_below'] / max(e['grain_above'], 1e-9):.2f}); "
                f"largest sustained rise +{e['rise']:.3f} at norm {e['rise_at']:.3f}; "
                f"far-field grain {e['far_grain']:.3f}, near-field {e['near_grain']:.3f}"
            )
        elif e:
            print(f"  {name} {e['shape'][0]}x{e['shape'][1]}: captured")


def profile_leg(leg_dir):
    """The composite 5 m -> 1500 m luma-vs-distance profile."""
    segs = []
    plan = [
        ("t2df_desert_close_20m", [(5, 8), (8, 12), (12, 18), (18, 28), (28, 45), (45, 70)]),
        ("t2df_lod01_contour", [(30, 45), (45, 70), (70, 100), (100, 140), (140, 200), (200, 300)]),
        ("t2df_profile_far", [(200, 300), (300, 450), (450, 650), (650, 900), (900, 1200), (1200, 1500)]),
    ]
    for name, bins in plan:
        png = os.path.join(leg_dir, name + ".png")
        if not os.path.isfile(png):
            continue
        L = luma(load(png))
        geo = STATIONS[name]
        segs.append((name, distance_bins(L, geo["cam_h"], geo["pitch"], bins)))
    return segs


def print_profile(tag, segs):
    print(f"--- {tag}: luma vs true 3D distance (per-pixel binned, flat-ground model)")
    for name, rows in segs:
        line = f"  {name}: "
        for lo, hi, v, n in rows:
            line += f"[{lo}-{hi}m]={v:.2f} "
        print(line)


def main():
    before = sys.argv[1] if len(sys.argv) > 1 else "d:/tmp/t2df_staging/before"
    after = sys.argv[2] if len(sys.argv) > 2 else "d:/tmp/t2df_staging/after"

    rb = analyse_leg(before)
    if not rb:
        print(f"no captures found in {before}")
        return 1
    print_leg(f"BEFORE ({before})", rb)
    print_profile("BEFORE", profile_leg(before))

    if os.path.isdir(after):
        ra = analyse_leg(after)
        if ra:
            print_leg(f"AFTER ({after})", ra)
            print_profile("AFTER", profile_leg(after))
            print("=== DELTAS (after vs before)")
            for name in sorted(rb):
                if name in ra and "far_grain" in rb[name] and "far_grain" in ra[name]:
                    fb, fa = rb[name]["far_grain"], ra[name]["far_grain"]
                    ab_a, ab_b = rb[name]["grain_above"], ra[name]["grain_above"]
                    print(
                        f"  {name}: far-field grain {fb:.3f} -> {fa:.3f} ({100 * (fa - fb) / fb:+.1f}%);"
                        f" contour-above grain {ab_a:.3f} -> {ab_b:.3f}"
                    )
            pb = {n: dict(((lo, hi), v) for lo, hi, v, _ in rows) for n, rows in profile_leg(before)}
            pa = {n: dict(((lo, hi), v) for lo, hi, v, _ in rows) for n, rows in profile_leg(after)}
            for n in pb:
                if n in pa:
                    line = f"  {n} luma delta: "
                    for k in pb[n]:
                        if k in pa[n]:
                            line += f"[{k[0]}-{k[1]}m]{pa[n][k] - pb[n][k]:+.2f} "
                    print(line)
    return 0


if __name__ == "__main__":
    sys.exit(main())
