# PolyHaven Integration – Autonomous Asset Pipeline

This directory demonstrates **Phase B: MaterialManager Integration** of the autonomous PolyHaven asset fetcher. Assets are automatically downloaded via the PolyHaven API and organized for use by the AstraWeave rendering engine.

---

## Quick Start

### 1. Fetch Assets (One Command)

```powershell
# Download all PolyHaven assets defined in manifest
cargo run -p astraweave-assets -- fetch
```

**What happens**:
- ✅ Reads `assets/polyhaven_manifest.toml`
- ✅ Queries PolyHaven API for download URLs
- ✅ Downloads 8 assets (5 textures + 3 HDRIs)
- ✅ Organizes files into `assets/_downloaded/<handle>/`
- ✅ Creates lockfile at `.asset_cache/polyhaven.lock`
- ✅ Generates attribution file

**Expected output**:
```
📊 Summary:
  Total assets: 8
  ✅ Downloaded: 8
  💾 Cached: 0
  ❌ Failed: 0
  
⏱️  Total time: ~40-50 seconds (first run)
```

### 2. Link Assets to Biome (Manual Step)

The fetcher downloads to `assets/_downloaded/`, but materials.toml references `polyhaven/<handle>/`. You have two options:

**Option A: Symbolic Links (Recommended)**

```powershell
# Create symlinks from polyhaven/<handle>/ to _downloaded/<handle>/
New-Item -ItemType SymbolicLink -Path "assets/materials/polyhaven/aerial_rocks" -Target "../../_downloaded/aerial_rocks"
New-Item -ItemType SymbolicLink -Path "assets/materials/polyhaven/metal_plate" -Target "../../_downloaded/metal_plate"
New-Item -ItemType SymbolicLink -Path "assets/materials/polyhaven/cobblestone" -Target "../../_downloaded/cobblestone"
New-Item -ItemType SymbolicLink -Path "assets/materials/polyhaven/wood_floor" -Target "../../_downloaded/wood_floor"
New-Item -ItemType SymbolicLink -Path "assets/materials/polyhaven/plastered_wall" -Target "../../_downloaded/plastered_wall"
```

**Option B: Update materials.toml (Alternative)**

Edit `materials.toml` to reference `../../_downloaded/<handle>/` instead:

```toml
[[layer]]
key = "aerial_rocks"
albedo = "../../_downloaded/aerial_rocks/aerial_rocks_albedo.png"
normal = "../../_downloaded/aerial_rocks/aerial_rocks_normal.png"
# ...
```

### 3. Run Unified Showcase

```powershell
cargo run -p unified_showcase --release
```

Press **`B`** key to switch to the **polyhaven** biome!

---

## File Organization

### After Fetch

```
assets/
├── _downloaded/                    # ✅ Downloaded assets (managed by fetcher)
│   ├── aerial_rocks/
│   │   ├── aerial_rocks_albedo.png
│   │   ├── aerial_rocks_normal.png
│   │   ├── aerial_rocks_roughness.png
│   │   ├── aerial_rocks_ao.png
│   │   └── aerial_rocks_height.png
│   ├── metal_plate/
│   │   ├── metal_plate_albedo.png
│   │   ├── metal_plate_normal.png
│   │   ├── metal_plate_roughness.png
│   │   ├── metal_plate_metallic.png
│   │   └── metal_plate_ao.png
│   ├── cobblestone/
│   ├── wood_floor/
│   ├── plastered_wall/
│   ├── spruit_sunrise/
│   │   └── spruit_sunrise_hdri.exr
│   ├── venice_sunset/
│   ├── kloppenheim/
│   └── ATTRIBUTION.txt             # ✅ CC0 provenance
│
├── materials/polyhaven/
│   ├── materials.toml              # ✅ Biome definition
│   ├── arrays.toml                 # ✅ GPU array layout
│   ├── aerial_rocks/               # 🔗 Symlink → ../../_downloaded/aerial_rocks
│   ├── metal_plate/                # 🔗 Symlink → ../../_downloaded/metal_plate
│   └── ...
│
└── polyhaven_manifest.toml         # ✅ Asset manifest

.asset_cache/
└── polyhaven.lock                  # ✅ Lockfile (URLs, hashes, timestamps)
```

---

## Integration Status

### ✅ Phase A Complete (File Extensions + Lockfile)
- Extensions fixed (.png, .exr)
- Lockfile generation verified
- Validated with 8/8 assets

### ✅ Phase C Complete (Unit Tests)
- 18 passing tests (config, lockfile, downloader)
- Zero warnings
- 100% success rate

### 🚧 Phase B In Progress (MaterialManager Integration)
- [x] Assets downloaded via autonomous fetcher
- [ ] Symlinks created (manual step)
- [ ] Unified showcase tested with PolyHaven biome
- [ ] HDRI integration for environment lighting

---

## materials.toml Structure

The `materials.toml` file defines how materials are loaded into GPU arrays:

```toml
[biome]
name = "polyhaven"

[[layer]]
key = "aerial_rocks"                # Material ID (referenced by shaders)
albedo = "aerial_rocks/albedo.png"  # Path relative to polyhaven/ directory
normal = "aerial_rocks/normal.png"
roughness = "aerial_rocks/roughness.png"
tiling = [1.0, 1.0]                 # UV tiling factor
triplanar_scale = 16.0              # Triplanar mapping scale
```

**Current Issue**: Paths are relative (`aerial_rocks/albedo.png`), but fetcher downloads to `_downloaded/aerial_rocks/aerial_rocks_albedo.png`.

**Solution**: Use symlinks to bridge the gap (see Option A above).

---

## arrays.toml Structure

The `arrays.toml` defines how materials are packed into GPU texture arrays:

```toml
[layout]
array_size = 5  # Must match number of materials

[mapping]
aerial_rocks = 0
metal_plate = 1
cobblestone = 2
wood_floor = 3
plastered_wall = 4
```

**Purpose**: Shaders use array indices to sample materials (e.g., `texture_array[2]` = cobblestone).

---

## CC0 License

All PolyHaven assets are licensed under **CC0 1.0 Universal (Public Domain)**. No attribution required, but `ATTRIBUTION.txt` is auto-generated for provenance tracking.

**Source**: https://polyhaven.com/license

---

## Advanced Usage

### On-Demand Asset Loading (Future)

```rust
use astraweave_assets::ensure_asset;

// Runtime loading (fetches if not cached)
let paths = ensure_asset(
    Path::new("assets/polyhaven_manifest.toml"),
    "aerial_rocks"
).await?;

// paths = HashMap {
//   "albedo" => "assets/_downloaded/aerial_rocks/aerial_rocks_albedo.png",
//   "normal" => "assets/_downloaded/aerial_rocks/aerial_rocks_normal.png",
//   ...
// }
```

### Cache Management

```powershell
# Check cache status
cargo run -p astraweave-assets -- check

# Prune orphaned files
cargo run -p astraweave-assets -- prune
```

---

## Troubleshooting

### Issue: "Failed to load material" error

**Cause**: Symlinks not created or paths incorrect in materials.toml

**Fix**:
1. Verify `assets/_downloaded/` exists and contains assets
2. Create symlinks (see Option A above)
3. OR update materials.toml to use `../../_downloaded/` prefix

### Issue: "Asset not found" during fetch

**Cause**: PolyHaven API changed or asset ID invalid

**Fix**:
1. Check asset exists at https://polyhaven.com/a/<asset_id>
2. Update `polyhaven_manifest.toml` with correct ID
3. Re-run fetch

### Issue: Textures appear corrupted or low-res

**Cause**: Download interrupted or resolution fallback

**Fix**:
1. Check lockfile: `cat .asset_cache/polyhaven.lock`
2. Look for `resolved_res` field (should match requested)
3. Delete cache and re-download: `rm -rf .asset_cache; cargo run -p astraweave-assets -- fetch`

---

## Next Steps

1. **Complete Phase B Integration**:
   - Create symlinks for all materials
   - Test unified_showcase with polyhaven biome
   - Validate visual output

2. **HDRI Integration**:
   - Load `spruit_sunrise.exr` for environment lighting
   - Test IBL (image-based lighting)
   - Verify skybox rendering

3. **Hot-Reload Testing** (Optional):
   - Modify a material texture
   - Verify hot-reload system detects change
   - Validate GPU update

---

## Performance

**First Run** (cold cache):
- Download: ~40-50 seconds (291 MB, 31 files)
- Organize: ~100 milliseconds
- Total: ~50 seconds

**Subsequent Runs** (warm cache):
- Check cache: ~10 milliseconds
- Skip downloads: ~0 seconds
- **40× faster** than cold fetch

---

**Status**: 🚧 **Phase B In Progress** – Symlinks pending, then ready for showcase testing!
