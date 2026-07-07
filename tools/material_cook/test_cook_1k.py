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
from cook_1k import cook_family, SIZE, MAPS  # noqa: E402

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


if __name__ == "__main__":
    ok = test_gravel_contract()
    print("PASS: cook_1k contract (1024x1024 RGBA PNG x3 maps)" if ok else "FAIL")
    sys.exit(0 if ok else 1)
