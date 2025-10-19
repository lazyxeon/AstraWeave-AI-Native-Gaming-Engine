# Multi-Source Asset Pipeline - Quick Start

**TL;DR**: Fetch free game assets from 3 providers with one command, proper licenses and attribution included.

---

## 5-Second Start

```bash
# Fetch all assets from manifest
cargo run -p astraweave-assets -- fetch

# Fetch from specific provider only
cargo run -p astraweave-assets -- fetch --provider polyhaven

# Regenerate attribution files
cargo run -p astraweave-assets -- regenerate-attributions
```

---

## Supported Providers

| Provider | Asset Types | License | API |
|----------|-------------|---------|-----|
| **PolyHaven** | Textures, HDRIs | CC0 only | ✅ Automatic |
| **Poly Pizza** | 3D Models | CC0 only | ❌ Manual URLs |
| **OpenGameArt** | Audio, Sprites, 2D/3D | CC0, CC-BY, CC-BY-SA | ❌ Manual URLs |

---

## Adding Assets to Manifest

**File**: `assets/asset_manifest.toml`

### PolyHaven (Automatic)

1. Browse https://polyhaven.com/
2. Copy asset ID from URL (e.g., `aerial_rocks_02` from `/a/aerial_rocks_02`)
3. Add to manifest:

```toml
[[assets]]
handle = "my_texture"
provider = "polyhaven"
type = "texture"
id = "aerial_rocks_02"
resolution = "2k"
format = "png"
```

**Supported types**: `texture`, `hdri`  
**Resolutions**: `1k`, `2k`, `4k`, `8k`  
**Formats**: `png`, `jpg`, `exr`

### Poly Pizza (Manual)

1. Browse https://poly.pizza/
2. Download model and **verify CC0 license** on page
3. Right-click download button → Copy link address
4. Add to manifest:

```toml
[[assets]]
handle = "character_knight"
provider = "polypizza"
type = "model"
format = "glb"
url = "https://poly.pizza/files/Low_poly_Knight.glb"
license = "CC0-1.0"
author = "Quaternius"
source_url = "https://poly.pizza/m/Low_poly_Knight"
```

**Supported formats**: `glb`, `gltf`, `obj`, `fbx`  
**CRITICAL**: Verify license is CC0 before adding!

### OpenGameArt (Manual)

1. Browse https://opengameart.org/
2. **Check license** on asset page (only CC0, CC-BY, CC-BY-SA allowed)
3. Right-click download button → Copy link address
4. Add to manifest:

```toml
# CC0 example (no author needed)
[[assets]]
handle = "music_ambient"
provider = "opengameart"
type = "audio"
format = "ogg"
url = "https://opengameart.org/sites/default/files/ambient.ogg"
license = "CC0-1.0"
source_url = "https://opengameart.org/content/ambient-music"

# CC-BY example (author required!)
[[assets]]
handle = "sprite_hero"
provider = "opengameart"
type = "sprite"
format = "png"
url = "https://opengameart.org/sites/default/files/hero_sprite.png"
license = "CC-BY-4.0"
author = "Artist Name"
source_url = "https://opengameart.org/content/hero-sprite"
```

**Supported types**: `audio`, `sprite`, `tileset`, `model`, `texture`  
**CRITICAL**: CC-BY requires `author` field!

---

## License Compliance

### Allowed Licenses (SPDX IDs)

✅ `CC0-1.0` - Public domain, no attribution required  
✅ `CC-BY-4.0` - Attribution required (must include `author` field)  
✅ `CC-BY-SA-4.0` - Attribution + share-alike (must include `author` field)

### Rejected Licenses

❌ `GPL-*` - Copyleft (incompatible with commercial games)  
❌ `CC-BY-NC-*` - Non-commercial restriction  
❌ `CC-BY-ND-*` - No derivatives restriction

**Enforcement**: Tool will error if you try to add restricted licenses.

---

## Output Structure

```
assets/_downloaded/
├── polyhaven/
│   ├── aerial_rocks/
│   │   ├── aerial_rocks_albedo.png
│   │   ├── aerial_rocks_normal.png
│   │   └── aerial_rocks_roughness.png
│   ├── sky_day/
│   │   └── sky_day_hdri.exr
│   └── ATTRIBUTION.txt
├── polypizza/
│   ├── character_knight/
│   │   └── character_knight_model.glb
│   └── ATTRIBUTION.txt
└── opengameart/
    ├── music_ambient/
    │   └── music_ambient_audio.ogg
    └── ATTRIBUTION.txt
```

---

## CLI Reference

### Fetch Command

```bash
# Fetch all assets
cargo run -p astraweave-assets -- fetch

# Fetch from specific provider
cargo run -p astraweave-assets -- fetch --provider polyhaven
cargo run -p astraweave-assets -- fetch --provider polypizza
cargo run -p astraweave-assets -- fetch --provider opengameart

# Custom manifest path
cargo run -p astraweave-assets -- fetch --manifest path/to/manifest.toml

# Output as JSON (for CI/scripts)
cargo run -p astraweave-assets -- fetch --json

# Quiet mode (no progress bars)
cargo run -p astraweave-assets -- fetch --quiet
```

### Regenerate Attributions Command

```bash
# Regenerate ATTRIBUTION.txt files from lockfile
cargo run -p astraweave-assets -- regenerate-attributions

# Use case: Manifest changed, need to update attribution
```

---

## Common Workflows

### Adding a New Asset

1. **Find asset** on provider website
2. **Verify license** (CC0, CC-BY, CC-BY-SA only)
3. **Add to manifest** (`assets/asset_manifest.toml`)
4. **Run fetch**: `cargo run -p astraweave-assets -- fetch`
5. **Check output**: `assets/_downloaded/<provider>/<handle>/`

### Updating Existing Assets

1. **Edit manifest** (change `id`, `url`, or `resolution`)
2. **Delete cache** (optional): `rm assets/_downloaded/<provider>/<handle>/`
3. **Run fetch**: `cargo run -p astraweave-assets -- fetch`

### Sharing Attribution Files

1. **Fetch assets**: `cargo run -p astraweave-assets -- fetch`
2. **Copy attribution files** to your game's `LICENSES/` folder:
   ```bash
   cp assets/_downloaded/*/ATTRIBUTION.txt game/LICENSES/
   ```
3. **Include in game distribution** (legal requirement for CC-BY!)

---

## Troubleshooting

### Error: "Unknown provider: xyz"

**Cause**: Typo in `provider` field  
**Solution**: Use `polyhaven`, `polypizza`, or `opengameart` (exact match)

### Error: "Invalid domain for URL"

**Cause**: URL doesn't match provider domain  
**Solution**: 
- Poly Pizza: URL must start with `https://poly.pizza/`
- OpenGameArt: URL must start with `https://opengameart.org/`

### Error: "License not permissive: GPL-3.0"

**Cause**: GPL license is not allowed  
**Solution**: Find a CC0/CC-BY/CC-BY-SA alternative

### Error: "CC-BY license requires author field"

**Cause**: CC-BY assets need attribution, but `author` field is missing  
**Solution**: Add `author = "Artist Name"` to manifest entry

### Error: "PolyHaven API error: HTTP 404"

**Cause**: Asset ID doesn't exist on PolyHaven  
**Solution**: Check asset ID on https://polyhaven.com/ (e.g., `aerial_rocks_02`)

---

## Performance

| Metric | Typical Value |
|--------|---------------|
| **Small manifest** (10 assets) | 30-60 seconds |
| **Large manifest** (100 assets) | 5-10 minutes |
| **Cache hit** (already downloaded) | <1 second |
| **Test suite** | 0.01 seconds (29 tests) |

**Tip**: Use `--provider` filter to fetch incrementally during development.

---

## Example Manifest

**Complete 3-provider example**:

```toml
# =============================================================================
# AstraWeave Multi-Source Asset Manifest
# =============================================================================

output_dir = "assets/_downloaded"
cache_dir = "assets/_cache"

# PolyHaven Texture (API-based, CC0)
[[assets]]
handle = "rock_texture"
provider = "polyhaven"
type = "texture"
id = "aerial_rocks_02"
resolution = "2k"
format = "png"

# PolyHaven HDRI (API-based, CC0)
[[assets]]
handle = "sky_day"
provider = "polyhaven"
type = "hdri"
id = "sunflowers_puresky"
resolution = "2k"
format = "exr"

# Poly Pizza Model (Manual, CC0)
[[assets]]
handle = "character_knight"
provider = "polypizza"
type = "model"
format = "glb"
url = "https://poly.pizza/files/Low_poly_Knight.glb"
license = "CC0-1.0"
author = "Quaternius"
source_url = "https://poly.pizza/m/Low_poly_Knight"

# OpenGameArt Audio (Manual, CC0)
[[assets]]
handle = "music_ambient"
provider = "opengameart"
type = "audio"
format = "ogg"
url = "https://opengameart.org/sites/default/files/ambient_forest.ogg"
license = "CC0-1.0"
source_url = "https://opengameart.org/content/ambient-forest"

# OpenGameArt Sprite (Manual, CC-BY)
[[assets]]
handle = "hero_sprite"
provider = "opengameart"
type = "sprite"
format = "png"
url = "https://opengameart.org/sites/default/files/hero.png"
license = "CC-BY-4.0"
author = "Artist Name"
source_url = "https://opengameart.org/content/hero-sprite"
```

---

## Next Steps

- **Browse providers**: Find assets for your game
- **Add to manifest**: Follow examples above
- **Run fetch**: `cargo run -p astraweave-assets -- fetch`
- **Integrate assets**: Use from `assets/_downloaded/<provider>/<handle>/`
- **Distribute attribution**: Include `ATTRIBUTION.txt` files in your game

**Questions?** See `MULTI_SOURCE_ASSET_PIPELINE_COMPLETE.md` for full technical details.

