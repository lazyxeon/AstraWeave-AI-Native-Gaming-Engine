"""L.3.B — the c1|c2 seam's magnitude as a function of view distance.

Uses the isolated shadow field (off - on) from twin deterministic flights and
profiles it in 50 m view-distance bins across the 500 m boundary. A rendition
discontinuity at the seam shows as a step between adjacent bins; a smooth
lighting gradient does not.

Usage: python l3b_seam_profile.py <on_dir> <off_dir> <frame_index> [--step 40]
"""
import sys
import math
from pathlib import Path
import numpy as np
from PIL import Image

W, H = 1024, 768
FOVY = 1.0471976
PITCH_DEG, YAW_DEG = 25.0, 45.0
GROUND_H = 36.3
DESERT_FOCAL = np.array([43.1, 36.3, -1961.8])
THRESH = 8.0


def main():
    on_dir, off_dir = Path(sys.argv[1]), Path(sys.argv[2])
    idx = int(sys.argv[3])
    step = 40.0
    for i, a in enumerate(sys.argv):
        if a == "--step":
            step = float(sys.argv[i + 1])

    name = f"f{idx:03}.png"
    def luma(p):
        a = np.asarray(Image.open(p).convert("RGB"), dtype=np.float64)
        return 0.2126 * a[:, :, 0] + 0.7152 * a[:, :, 1] + 0.0722 * a[:, :, 2]
    eff = luma(off_dir / name) - luma(on_dir / name)

    focal = DESERT_FOCAL + np.array([-step * idx, 0.0, 0.0])
    pitch, yaw = math.radians(PITCH_DEG), math.radians(YAW_DEG)
    dist = (219.0 - focal[1]) / math.sin(pitch)
    eye = focal + dist * np.array([math.cos(yaw) * math.cos(pitch), math.sin(pitch),
                                   math.sin(yaw) * math.cos(pitch)])
    f = focal - eye
    f = f / np.linalg.norm(f)
    r = np.cross(f, [0.0, 1.0, 0.0]); r /= np.linalg.norm(r)
    u = np.cross(r, f); u /= np.linalg.norm(u)
    t_half = math.tan(FOVY * 0.5)

    rows_by_bin = {}
    for y in range(H):
        ndc_y = 1.0 - 2.0 * (y + 0.5) / H
        d = f + u * (ndc_y * t_half)
        d = d / np.linalg.norm(d)
        if d[1] >= -1e-4:
            continue
        vd = (GROUND_H - eye[1]) / d[1]
        if vd <= 0 or vd < 250 or vd > 900:
            continue
        b = int(vd // 50) * 50
        rows_by_bin.setdefault(b, []).append(y)

    print(f"frame {idx}: shadow effect vs view distance (50 m bins); c1|c2 seam at 500 m")
    print("  bin_m   rows   mean_effect  shadowed%")
    prev = None
    for b in sorted(rows_by_bin):
        rows = rows_by_bin[b]
        sub = eff[rows, :]
        m = float(sub.mean())
        a = float((sub > THRESH).mean() * 100.0)
        jump = "" if prev is None else f"   d={m - prev:+.2f}"
        star = "  <== SEAM" if b == 500 else ""
        print(f"  {b:5}  {len(rows):5}   {m:10.2f}  {a:8.2f}{jump}{star}")
        prev = m


if __name__ == "__main__":
    main()
