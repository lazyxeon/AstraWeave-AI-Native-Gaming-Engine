# Unified Showcase Visual Fix Report

## Problem
The `unified_showcase` example was exhibiting several visual artifacts:
1.  **Grid Lines**: The ground mesh had visible gaps between tiles, creating a grid pattern.
2.  **White/Untextured Objects**: Buildings and some rocks appeared as white untextured meshes.
3.  **Incorrect Models**: The "buildings" were actually rendering as large rocks.

## Diagnosis
1.  **Grid Lines**: Caused by exact 1.0 scaling of tiles leading to z-fighting or sub-pixel gaps, and lack of height variation.
2.  **White Objects**: 
    *   `tower.glb` used materials named "bricks" and "stones" which were not mapped in `main.rs`.
    *   `rock_largeA.glb` used "dirt" which was mapped to `skin_mat` (reddish) or unmapped in some contexts.
3.  **Incorrect Models**: The code was loading `rock_largeA.glb` into the `building_model` variable instead of `tower.glb`.

## Fixes Implemented
1.  **Terrain Generation**:
    *   Increased tile scale to `1.01` to ensure slight overlap and eliminate gaps.
    *   Added Perlin-like noise (sin/cos) to tile height to break the flat grid look.
    *   Added random rotation to grass tiles for variety.
2.  **Material Mapping**:
    *   Added mappings for `bricks` -> `skin_mat` (Rock Slate texture, looks like reddish stone/brick).
    *   Added mappings for `stones` -> `gray_mat` (Stone texture).
    *   Mapped `dirt` -> `gray_mat` to ensure rocks look like stone, not skin/white.
3.  **Model Loading**:
    *   Switched `building_model` to load `assets/models/tower.glb`.

## Verification
*   **Console Output**: Confirmed `tower.glb` loads with "bricks" and "stones" materials correctly assigned.
*   **Visuals**: 
    *   Terrain is now a continuous surface with gentle undulations.
    *   Buildings are proper towers with brick/stone textures.
    *   Rocks are gray/stone colored.
    *   Trees have proper bark and leaf textures.

## Status
✅ **FIXED** - The showcase now renders correctly with high-quality PBR materials and no missing assets.
