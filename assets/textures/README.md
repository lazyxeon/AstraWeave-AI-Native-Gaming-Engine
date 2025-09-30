# Biome Materials TOML Schema

Place per-biome material mappings under `assets/textures/<biome>/materials.toml`.
Paths inside the file are relative to the TOML file's directory.

## File Structure

[biome]
name = "<biome-name>"

[[layer]]
key = "<material-key>"         # required; must match engine layer key (e.g., grass, dirt, stone)
# Either provide a packed MRA map:
albedo   = "path/to/albedo.png"
normal   = "path/to/normal.png"
mra      = "path/to/mra.png"
# ...or provide separate channels to be packed into MRA:
metallic = "path/to/metallic.png"    # R channel
roughness= "path/to/roughness.png"   # G channel
ao       = "path/to/ao.png"          # B channel

You can mix approaches per-layer; if `mra` is missing but `metallic/roughness/ao` exist, the loader packs them.

## Fallback Behavior
- Missing albedo: uses a neutral but non-flat synthesized albedo based on the layer key.
- Missing normal: uses a flat normal (RGB 128,128,255).
- Missing MRA: uses neutral values (M=0, R=1, AO=1) to avoid overly shiny/dark results.
- Missing files are logged per-layer; a summary prints counts of loaded vs substituted images.

## Example
See `assets/textures/grassland/materials.toml`, `assets/textures/desert/materials.toml`, and `assets/textures/forest/materials.toml` for templates.
