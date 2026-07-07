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


if __name__ == "__main__":
    ok = test_gravel_contract()
    print("PASS: cook_1k contract (1024x1024 RGBA PNG x3 maps)" if ok else "FAIL")
    ok2 = test_pack_mra_16bit_safe()
    print("PASS: pack_mra 16-bit-safe (D2 regression)" if ok2 else "FAIL: 16-bit")
    ok3 = test_arm_to_mra_swap_and_guard()
    print("PASS: ARM->MRA swap + guard (D1 regression)" if ok3 else "FAIL: ARM swap")
    sys.exit(0 if (ok and ok2 and ok3) else 1)
