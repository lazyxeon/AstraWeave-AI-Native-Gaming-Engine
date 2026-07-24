#!/usr/bin/env python3
# CC0 / Public Domain
"""Contract test for cook_1k.py — dimensions, channels, format.

Run:  python tools/material_cook/test_cook_1k.py
Cooks the AD.4.R smoke-cook source (`gravel`, a C6 family with a traced,
locally-present assets_src source) into a temp dir and asserts the
MaterialLibrary 1K contract. Writes nothing into the repo tree.
"""

import os
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from cook_1k import cook_family, cook_mra_arm_to_mra, pack_mra, SIZE, MAPS  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(REPO, "assets_src", "materials")


def test_gravel_contract():
    from PIL import Image
    with tempfile.TemporaryDirectory() as tmp:
        out_base = os.path.join(tmp, "gravel")
        res = cook_family(
            os.path.join(SRC, "gravel.png"),
            os.path.join(SRC, "gravel_n.png"),
            os.path.join(SRC, "gravel_mra.png"),
            out_base,
        )
        # exactly three maps
        assert set(res.keys()) == set(MAPS), f"expected 3 maps, got {list(res)}"
        for sfx, r in res.items():
            path = r["out"]
            assert os.path.exists(path), f"missing output {path}"
            with Image.open(path) as im:
                assert im.format == "PNG", f"{sfx}: format {im.format} != PNG"
                assert im.size == (SIZE, SIZE), f"{sfx}: {im.size} != {(SIZE, SIZE)}"
                assert im.mode == "RGBA", f"{sfx}: mode {im.mode} != RGBA"
    return True


def test_pack_mra_16bit_safe():
    """AD.4.A regression (D2): a 16-bit (I;16) roughness/ao source must not
    collapse to flat 255. Pre-fix, PIL convert("L") clamped I;16 values >255
    to solid white — destroyed plaster_mra AO + tree_bark_mra roughness."""
    from PIL import Image
    with tempfile.TemporaryDirectory() as tmp:
        # synthetic 16-bit gradient: values 0..65535 across x
        w = h = 64
        grad = Image.new("I;16", (w, h))
        grad.putdata([int(x * 65535 / (w - 1)) for _ in range(h) for x in range(w)])
        rough_p = os.path.join(tmp, "rough16.png")
        ao_p = os.path.join(tmp, "ao16.png")
        grad.save(rough_p)
        grad.save(ao_p)
        out = os.path.join(tmp, "mra.png")
        pack_mra(rough_p, ao_p, out)
        with Image.open(out) as im:
            px = list(im.getdata())
            g_vals = {p[1] for p in px}
            b_vals = {p[2] for p in px}
            assert len(g_vals) > 32, f"roughness collapsed: {len(g_vals)} distinct values"
            assert len(b_vals) > 32, f"ao collapsed: {len(b_vals)} distinct values"
            g_mean = sum(p[1] for p in px) / len(px)
            assert 100 < g_mean < 155, f"16-bit scaling wrong: G mean {g_mean:.1f} (expect ~127)"
    return True


def test_arm_to_mra_swap_and_guard():
    """AD.4.A regression (D1): ARM-ordered input (R=AO high, B=metal 0) must
    come out true-MRA (R~0, B=AO); true-MRA input must be REFUSED by the guard."""
    from PIL import Image
    with tempfile.TemporaryDirectory() as tmp:
        # ARM-profile source: R=200 (AO), G=180 (rough), B=0 (metal)
        arm = Image.new("RGBA", (128, 128), (200, 180, 0, 255))
        arm_p = os.path.join(tmp, "arm_mislabeled_mra.png")
        arm.save(arm_p)
        out = os.path.join(tmp, "fixed_mra.png")
        cook_mra_arm_to_mra(arm_p, out)
        with Image.open(out) as im:
            assert im.size == (SIZE, SIZE) and im.mode == "RGBA"
            r, g, b, _ = im.getpixel((SIZE // 2, SIZE // 2))
            assert r == 0 and g == 180 and b == 200, f"swap wrong: ({r},{g},{b})"
        # guard: true-MRA source (R=0) must be refused
        mra = Image.new("RGBA", (128, 128), (0, 180, 200, 255))
        mra_p = os.path.join(tmp, "true_mra.png")
        mra.save(mra_p)
        refused = False
        try:
            cook_mra_arm_to_mra(mra_p, os.path.join(tmp, "should_not_exist.png"))
        except SystemExit:
            refused = True
        assert refused, "guard failed: true-MRA input was swapped"
    return True


def test_ao_orientation_and_normal_integration():
    """T.2a regression: AO must rise with height, and the normal-map
    integration must recover height in the same orientation.

    `scripts/import_terrain_textures.py::build_mra` shipped `1.0 - hf`, which
    darkened peaks and lit crevices; the inversion reached the live pack
    (mud_mra AO correlated +0.991 with the inverted curve). Both derivations
    are pinned here so the sign cannot silently flip back.
    """
    import numpy as np
    from PIL import Image
    from cook_1k import ao_from_displacement, ao_from_normal_map

    with tempfile.TemporaryDirectory() as tmp:
        # A smooth bump: bright (high) in the middle, dark (low) at the edges.
        n = 256
        yy, xx = np.mgrid[0:n, 0:n]
        r = np.sqrt((xx - n / 2) ** 2 + (yy - n / 2) ** 2) / (n / 2)
        height = np.clip(1.0 - r, 0.0, 1.0)

        disp_p = os.path.join(tmp, "bump_disp.png")
        Image.fromarray((height * 255).astype("uint8"), "L").save(disp_p)
        ao = ao_from_displacement(disp_p, size=n)
        # The peak must be LESS occluded (higher AO) than the rim.
        peak, rim = ao[n // 2, n // 2], ao[4, 4]
        assert peak > rim, f"AO inverted: peak {peak:.3f} <= rim {rim:.3f}"
        assert np.corrcoef(ao.ravel(), height.ravel())[0, 1] > 0.9, "AO must track height"

        # Encode the same bump as an OpenGL tangent-space normal map and check
        # the integration recovers a height correlated with the original.
        gy, gx = np.gradient(height)
        nx, ny, nz = -gx, gy, np.ones_like(height) * 0.05
        norm = np.sqrt(nx * nx + ny * ny + nz * nz)
        rgb = np.dstack([(nx / norm * 0.5 + 0.5), (ny / norm * 0.5 + 0.5), (nz / norm * 0.5 + 0.5)])
        nrm_p = os.path.join(tmp, "bump_n.png")
        Image.fromarray((rgb * 255).astype("uint8"), "RGB").save(nrm_p)
        ao_n = ao_from_normal_map(nrm_p, size=n)
        c = np.corrcoef(ao_n.ravel(), height.ravel())[0, 1]
        assert c > 0.5, f"normal-map AO orientation wrong (corr {c:+.3f})"
    return True


if __name__ == "__main__":
    ok = test_gravel_contract()
    print("PASS: cook_1k contract (1024x1024 RGBA PNG x3 maps)" if ok else "FAIL")
    ok2 = test_pack_mra_16bit_safe()
    print("PASS: pack_mra 16-bit-safe (D2 regression)" if ok2 else "FAIL: 16-bit")
    ok3 = test_arm_to_mra_swap_and_guard()
    print("PASS: ARM->MRA swap + guard (D1 regression)" if ok3 else "FAIL: ARM swap")
    ok4 = test_ao_orientation_and_normal_integration()
    print("PASS: AO orientation + normal integration (T.2a regression)" if ok4 else "FAIL: AO orientation")
    sys.exit(0 if (ok and ok2 and ok3 and ok4) else 1)
