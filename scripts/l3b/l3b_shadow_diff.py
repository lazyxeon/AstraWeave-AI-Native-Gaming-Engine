"""L.3.B — isolate the SHADOW term across a continuous flight.

Two runs of the identical deterministic camera path (shadows on / shadows off)
are differenced frame-by-frame. Everything view-dependent that is not shadow —
terrain texture mips, hex-tile phase, LOD, fog, tonemap — is identical in both
runs and cancels exactly, so `effect[i] = off[i] - on[i]` IS the shadow
darkening field for frame i.

Reported per adjacent frame pair, restricted to the survey cascade's own screen
domain (the row band whose ground intersection lies in 500..3000 m, matching the
harness's band definition), plus the near band for the c0/c1 comparison:

* d_effect_mean  — change in mean shadow darkening (luma)
* d_effect_area  — change in the fraction of band pixels that are shadowed
                   (effect > THRESH), in percentage points

A smooth flight changes these gradually as the view sweeps; a cached-cascade
snap or a camera-relative boundary crossing produces a spike on one frame pair.

Usage: python l3b_shadow_diff.py <on_dir> <off_dir> [--alt 219] [--step 40]
"""
import sys
import math
from pathlib import Path
import numpy as np
from PIL import Image

W, H = 1024, 768
FOVY = 1.0471976
PITCH_DEG = 25.0
YAW_DEG = 45.0
GROUND_H = 36.3
DESERT_FOCAL = np.array([43.1, 36.3, -1961.8])
THRESH = 8.0  # luma of darkening to count a pixel as "shadowed"


def survey_eye(focal, altitude, yaw_deg=YAW_DEG):
    pitch = math.radians(PITCH_DEG)
    yaw = math.radians(yaw_deg)
    dist = (altitude - focal[1]) / math.sin(pitch)
    d = np.array([math.cos(yaw) * math.cos(pitch), math.sin(pitch),
                  math.sin(yaw) * math.cos(pitch)])
    return focal + dist * d


def basis(eye, focal):
    f = focal - eye
    f = f / np.linalg.norm(f)
    r = np.cross(f, np.array([0.0, 1.0, 0.0]))
    r = r / np.linalg.norm(r)
    u = np.cross(r, f)
    return f, r, u / np.linalg.norm(u)


def row_view_distance(eye, bs, y):
    """View distance of the ground point under the frame-centre column at row y."""
    f, r, u = bs
    t_half = math.tan(FOVY * 0.5)
    ndc_y = 1.0 - 2.0 * (y + 0.5) / H
    d = f + u * (ndc_y * t_half)
    d = d / np.linalg.norm(d)
    if d[1] >= -1e-4:
        return None
    t = (GROUND_H - eye[1]) / d[1]
    return t if t > 0 else None


def band_rows(eye, bs, near_m, far_m):
    rows = []
    for y in range(H):
        vd = row_view_distance(eye, bs, y)
        if vd is not None and near_m <= vd <= far_m:
            rows.append(y)
    return rows


def luma(path):
    a = np.asarray(Image.open(path).convert("RGB"), dtype=np.float64)
    return 0.2126 * a[:, :, 0] + 0.7152 * a[:, :, 1] + 0.0722 * a[:, :, 2]


def main():
    on_dir, off_dir = Path(sys.argv[1]), Path(sys.argv[2])
    alt = 219.0
    step = 40.0
    yaw_step = 0.0  # degrees/frame; non-zero = PAN route (position held)
    for i, a in enumerate(sys.argv):
        if a == "--alt":
            alt = float(sys.argv[i + 1])
        if a == "--step":
            step = float(sys.argv[i + 1])
        if a == "--yaw-step":
            yaw_step = float(sys.argv[i + 1])

    frames = sorted(p.name for p in on_dir.glob("f[0-9][0-9][0-9].png"))
    if not frames:
        print("no frames found")
        return
    print(f"{len(frames)} frames; alt {alt} m, {step} m/frame; shadow threshold {THRESH} luma")
    print(" frame  surveyEff dEff  surveyArea% dArea%   nearEff dEff  nearArea% dArea%")

    # World-anchored feature tracking in the ISOLATED effect field. On the PAN
    # route the camera position is held, so a fixed world point keeps a
    # constant view distance, mip level and cascade selection — any step in its
    # shadow EFFECT there is unambiguously the shadow system, with texture and
    # shading already cancelled by the differencing.
    feature_pts = []
    feature_hist = {}

    def unproject(eye, bs, px, py):
        f, r, u = bs
        t_half = math.tan(FOVY * 0.5)
        aspect = W / H
        ndc_x = 2.0 * (px + 0.5) / W - 1.0
        ndc_y = 1.0 - 2.0 * (py + 0.5) / H
        d = f + r * (ndc_x * t_half * aspect) + u * (ndc_y * t_half)
        d = d / np.linalg.norm(d)
        if d[1] >= -1e-4:
            return None
        t = (GROUND_H - eye[1]) / d[1]
        return eye + d * t if t > 0 else None

    def project(eye, bs, p):
        f, r, u = bs
        d = p - eye
        z = float(np.dot(d, f))
        if z <= 0.1:
            return None
        t_half = math.tan(FOVY * 0.5)
        aspect = W / H
        nx = float(np.dot(d, r)) / (z * t_half * aspect)
        ny = float(np.dot(d, u)) / (z * t_half)
        if abs(nx) > 0.92 or abs(ny) > 0.92:
            return None
        return ((nx + 1.0) * 0.5 * W - 0.5, (1.0 - ny) * 0.5 * H - 0.5)

    prev = None
    worst = {"d_eff": (0.0, ""), "d_area": (0.0, ""),
             "near_d_eff": (0.0, ""), "near_d_area": (0.0, ""),
             "seam_d_eff": (0.0, ""), "seam_d_area": (0.0, "")}
    for idx, name in enumerate(frames):
        off_p, on_p = off_dir / name, on_dir / name
        if not off_p.exists():
            print(f"{name}: missing in off-run")
            continue
        eff = luma(off_p) - luma(on_p)  # positive where shadows darken
        if yaw_step != 0.0:
            focal = DESERT_FOCAL
            eye = survey_eye(focal, alt, YAW_DEG + yaw_step * idx)
        else:
            focal = DESERT_FOCAL + np.array([-step * idx, 0.0, 0.0])
            eye = survey_eye(focal, alt)
        bs = basis(eye, focal)
        srows = band_rows(eye, bs, 500.0, 3000.0)
        nrows = band_rows(eye, bs, 0.0, 500.0)
        # SEAM band: straddles the c1|c2 boundary at 500 m, where the 6.4x
        # rendition discontinuity lives and where the 15% blend band
        # (425..500 m) acts. This is the band the fix must improve.
        krows = band_rows(eye, bs, 350.0, 650.0)
        def stats(rows):
            if not rows:
                return (0.0, 0.0)
            sub = eff[rows, :]
            return (float(sub.mean()), float((sub > THRESH).mean() * 100.0))
        s_eff, s_area = stats(srows)
        n_eff, n_area = stats(nrows)
        k_eff, k_area = stats(krows)
        if prev is not None:
            d_eff = abs(s_eff - prev[0])
            d_area = abs(s_area - prev[1])
            nd_eff = abs(n_eff - prev[2])
            nd_area = abs(n_area - prev[3])
            kd_eff = abs(k_eff - prev[4])
            kd_area = abs(k_area - prev[5])
            print(f"{idx:6}  {s_eff:9.2f} {d_eff:5.2f}  {s_area:10.2f} {d_area:6.2f}   "
                  f"{n_eff:7.2f} {nd_eff:5.2f}  {n_area:8.2f} {nd_area:6.2f}   "
                  f"SEAM {k_eff:6.2f} {kd_eff:5.2f} {k_area:7.2f} {kd_area:6.2f}")
            for key, val in (("d_eff", d_eff), ("d_area", d_area),
                             ("near_d_eff", nd_eff), ("near_d_area", nd_area),
                             ("seam_d_eff", kd_eff), ("seam_d_area", kd_area)):
                if val > worst[key][0]:
                    worst[key] = (val, f"frame {idx}")
        prev = (s_eff, s_area, n_eff, n_area, k_eff, k_area)

        # Seed features on the first frame from strongly-shadowed pixels spread
        # across the frame; then track them every frame.
        if not feature_pts:
            cand = []
            for py in range(120, H - 120, 24):
                for px in range(120, W - 120, 24):
                    v = float(eff[py, px])
                    if v > 20.0:
                        cand.append((v, px, py))
            cand.sort(reverse=True)
            for v, px, py in cand:
                if len(feature_pts) >= 6:
                    break
                wp = unproject(eye, bs, px, py)
                if wp is None:
                    continue
                if all(np.linalg.norm(wp - q) > 200.0 for q in feature_pts):
                    feature_pts.append(wp)
            for i, wp in enumerate(feature_pts):
                feature_hist[i] = []
            print(f"  seeded {len(feature_pts)} world-anchored features")
        for i, wp in enumerate(feature_pts):
            pr = project(eye, bs, wp)
            if pr is None:
                feature_hist[i].append(None)
                continue
            px, py = int(pr[0]), int(pr[1])
            patch = eff[max(0, py - 5):py + 6, max(0, px - 5):px + 6]
            feature_hist[i].append(float(patch.mean()) if patch.size else None)

    print("\nWORST frame-to-frame steps in the ISOLATED shadow field:")
    print(f"  survey band: mean effect {worst['d_eff'][0]:.2f} luma ({worst['d_eff'][1]}), "
          f"shadow area {worst['d_area'][0]:.2f} pp ({worst['d_area'][1]})")
    print(f"  near band:   mean effect {worst['near_d_eff'][0]:.2f} luma ({worst['near_d_eff'][1]}), "
          f"shadow area {worst['near_d_area'][0]:.2f} pp ({worst['near_d_area'][1]})")
    print(f"  SEAM band (350..650 m, straddles the c1|c2 boundary): "
          f"mean effect {worst['seam_d_eff'][0]:.2f} luma ({worst['seam_d_eff'][1]}), "
          f"shadow area {worst['seam_d_area'][0]:.2f} pp ({worst['seam_d_area'][1]})")

    print("\nWORLD-ANCHORED FEATURES in the isolated shadow field "
          "(step = |effect[i] - effect[i-1]| luma):")
    overall = 0.0
    for i, hist in sorted(feature_hist.items()):
        steps = [(abs(b - a), k) for k, (a, b) in enumerate(zip(hist, hist[1:]), start=1)
                 if a is not None and b is not None]
        if not steps:
            print(f"  feature{i}: never trackable")
            continue
        worst_step, at = max(steps)
        seen = [v for v in hist if v is not None]
        overall = max(overall, worst_step)
        print(f"  feature{i}: effect {min(seen):5.1f}..{max(seen):5.1f} luma, "
              f"worst step {worst_step:5.2f} at frame {at} ({len(steps)} pairs)")
    print(f"  WORST feature step across all features: {overall:.2f} luma")


if __name__ == "__main__":
    main()
