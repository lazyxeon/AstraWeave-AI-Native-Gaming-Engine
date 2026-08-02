"""L-series station A/B: exact pixel diff between two capture labels.

The L.3 station guarantee is a BYTE-IDENTITY claim ("0 differing pixels at the
close/mid stations"), so the comparison has to be exact-pixel, not perceptual.
This reports, per station PNG present in both directories:

  * differing pixel count and percentage (any channel differs)
  * mean luma on each side, and the delta
  * max absolute per-channel difference

Stations present in only one side are listed separately — a silently missing
capture would otherwise read as "no differences found".

Usage:
    python l3_station_diff.py <baseline_dir> <candidate_dir>
    python l3_station_diff.py d:/tmp/l3_staging/l3c_stations \
                              d:/tmp/l3_staging/l3cres_stations
"""
import sys
from pathlib import Path
import numpy as np
from PIL import Image


def load(path):
    return np.asarray(Image.open(path).convert("RGB"), dtype=np.int16)


def luma(a):
    return 0.2126 * a[:, :, 0] + 0.7152 * a[:, :, 1] + 0.0722 * a[:, :, 2]


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    a_dir, b_dir = Path(sys.argv[1]), Path(sys.argv[2])
    a_names = {p.name for p in a_dir.glob("*.png")}
    b_names = {p.name for p in b_dir.glob("*.png")}
    both = sorted(a_names & b_names)
    if not both:
        print(f"no common PNGs between {a_dir} and {b_dir}")
        return 1

    print(f"baseline  {a_dir}")
    print(f"candidate {b_dir}")
    print(f"{len(both)} stations in common\n")
    print(f"{'station':<40} {'differing px':>16} {'  mean A':>9} {'  mean B':>9} "
          f"{'  delta':>8} {'maxch':>6}")
    identical = 0
    for name in both:
        a, b = load(a_dir / name), load(b_dir / name)
        if a.shape != b.shape:
            print(f"{name:<40} {'SHAPE MISMATCH':>16} {a.shape} vs {b.shape}")
            continue
        d = np.abs(a - b)
        diff_mask = d.max(axis=2) > 0
        n_diff = int(diff_mask.sum())
        total = diff_mask.size
        la, lb = float(luma(a).mean()), float(luma(b).mean())
        if n_diff == 0:
            identical += 1
        print(f"{name:<40} {n_diff:>7} / {total:<6} {la:>9.2f} {lb:>9.2f} "
              f"{lb - la:>+8.2f} {int(d.max()):>6}")

    print(f"\n{identical}/{len(both)} stations byte-identical")
    only_a = sorted(a_names - b_names)
    only_b = sorted(b_names - a_names)
    if only_a:
        print(f"ONLY IN BASELINE  ({len(only_a)}): {', '.join(only_a)}")
    if only_b:
        print(f"ONLY IN CANDIDATE ({len(only_b)}): {', '.join(only_b)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
