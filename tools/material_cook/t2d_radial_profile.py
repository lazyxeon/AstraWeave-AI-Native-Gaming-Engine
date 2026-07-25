"""T.2d — true radial profile: luma vs camera-to-fragment distance, per pixel.

The row-profile in the harness averages each screen row, but iso-distance
contours on a ground plane are CURVES, so row-averaging smears any ring into a
ramp. This bins every pixel by its actual 3D distance from the camera, which is
the quantity `compute_material_lod` (footprint) and the CSM cascade splits are
keyed to, and reports the binned profile plus its largest step.
"""
import sys

import numpy as np
from PIL import Image as I

I.MAX_IMAGE_PIXELS = None

W, H = 1024, 768
FOVY = np.radians(60.0)
ASPECT = W / H
DESERT_FOCAL = np.array([43.1, 36.3, -1961.8])
YAW = np.radians(45.0)


def camera_pos(focal, dist, yaw, pitch):
    return focal + np.array(
        [
            dist * np.cos(yaw) * np.cos(pitch),
            dist * np.sin(pitch),
            dist * np.sin(yaw) * np.cos(pitch),
        ]
    )


def radial_profile(png, cam_y, pitch_deg, bin_m=25.0, dmax=1500.0):
    pitch = np.radians(pitch_deg)
    altitude = cam_y - DESERT_FOCAL[1]
    dist = altitude / np.sin(pitch)
    eye = camera_pos(DESERT_FOCAL, dist, YAW, pitch)

    fwd = DESERT_FOCAL - eye
    fwd = fwd / np.linalg.norm(fwd)
    up_w = np.array([0.0, 1.0, 0.0])
    right = np.cross(fwd, up_w)
    right /= np.linalg.norm(right)
    up = np.cross(right, fwd)

    xs = (np.arange(W) + 0.5) / W * 2.0 - 1.0
    ys = 1.0 - (np.arange(H) + 0.5) / H * 2.0
    gx, gy = np.meshgrid(xs, ys)
    th = np.tan(FOVY * 0.5)
    dirs = (
        fwd[None, None, :]
        + right[None, None, :] * (gx * th * ASPECT)[..., None]
        + up[None, None, :] * (gy * th)[..., None]
    )
    dirs /= np.linalg.norm(dirs, axis=2, keepdims=True)

    # Intersect the ground plane y = DESERT_FOCAL[1]
    denom = dirs[:, :, 1]
    t = (DESERT_FOCAL[1] - eye[1]) / np.where(np.abs(denom) < 1e-9, np.nan, denom)
    t = np.where(t > 0, t, np.nan)  # only forward hits

    rgba = np.asarray(I.open(png).convert("RGB")).astype(np.float64)
    luma = 0.2126 * rgba[:, :, 0] + 0.7152 * rgba[:, :, 1] + 0.0722 * rgba[:, :, 2]

    valid = np.isfinite(t) & (t < dmax)
    d = t[valid]
    l = luma[valid]
    if d.size == 0:
        print(f"  {png}: no ground hits within {dmax} m")
        return

    edges = np.arange(0, dmax + bin_m, bin_m)
    idx = np.digitize(d, edges) - 1
    prof = []
    for b in range(len(edges) - 1):
        m = idx == b
        if m.sum() < 200:
            continue
        prof.append((0.5 * (edges[b] + edges[b + 1]), l[m].mean(), int(m.sum())))

    print(f"\n=== {png}  (camY={cam_y}, pitch={pitch_deg}, eye_y={eye[1]:.1f}) ===")
    print("   dist_m    luma     n      step_from_prev")
    prev = None
    steps = []
    for c, mean, n in prof:
        s = "" if prev is None else f"{mean - prev:+7.3f}"
        if prev is not None:
            steps.append((abs(mean - prev), c, mean - prev))
        print(f"  {c:7.1f}  {mean:7.3f}  {n:6d}   {s}")
        prev = mean
    if steps:
        steps.sort(reverse=True)
        print("  largest binned steps:")
        for s, c, signed in steps[:4]:
            print(f"    {signed:+7.3f} luma at ~{c:.0f} m")


if __name__ == "__main__":
    base = "d:/tmp/t2d_staging/head_D"
    for cam_y, p in [(414.5, 30.0), (536.2, 30.0)]:
        radial_profile(f"{base}/D_desert_y{cam_y:.0f}_p{p:.0f}.png", cam_y, p)
