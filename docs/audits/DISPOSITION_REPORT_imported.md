# Disposition Report — `assets/imported/` (9.0 GB) + Road to Vostok Vol.1 (+ adjacent unknowns)

**Date**: 2026-07-05 (AD.1) | **Method**: clusters traced against upstream license documents fetched this session (Poly Haven API `api.polyhaven.com/info/<slug>` + <https://polyhaven.com/license>; itch.io + Wayback Machine for RtV). No git-lfs traffic; all file inspection used the local checkout. Tracing standard per AD.1: a cluster is TRACED only when a source document states license terms; no inference. **Quarantine applies to the release page as well as the repo** — QUARANTINE-RECOMMENDED clusters are never uploaded anywhere.

## 1. `assets/imported/Namaqualand/` — TRACED (CC0, Poly Haven) with 1 exception

The directory is a Blender-scene decomposition whose mesh/texture names map to published Poly Haven Namaqualand-collection assets. 17 slugs verified via API (HTTP 200, model/texture metadata + authors), 2026-07-05: `quiver_tree_01`, `quiver_tree_02`, `namaqualand_boulder_02/03/05/06`, `namaqualand_rocks_01`, `namaqualand_stones_01`, `namaqualand_cliff_02`, `othonna_cerarioides`, `didelta_spinosa` (large/med/small = LOD/size variants of the one slug), `crystalline_iceplant`, `flower_gazania`, `flower_ursinia`, `flower_heliophila`, `searsia_lucida`, plus textures `gravelly_sand`, `cliff_side`. Associated 8K HDRI-derived PNGs trace to `goegap_road`, `kloofendal_48d_partly_cloudy_puresky`, and `rogland_sunset` — all API-verified CC0 2026-07-05 (`goegap_road` appears in both packs' texture sets; see §5). Site license quote (fetched 2026-07-05): *"Our assets are all licensed as CC0, which is effectively Public Domain…"*

**Exception — QUARANTINE-RECOMMENDED**: `fine_leaf_01_{a,b,c}` meshes + textures. `api.polyhaven.com/info/fine_leaf_01` → HTTP 404; site search finds no such slug. Very likely an unpublished sub-asset of the Poly Haven scene, but no source document states its license → untraceable by the standard. Code impact: `astraweave-terrain/src/biome.rs:810` (one desert-scatter species; loader skips missing files gracefully, `engine_adapter.rs:2578`).

**Recommendation**: TRACED → **eligible for release packs**, excluding the `fine_leaf_01_*` files (local quarantine). Note for AD.3 packaging: the `.jpg.png`/`.exr.png` double-extension files are conversion artifacts of the import pipeline; they inherit the source slug's CC0.

## 2. `assets/imported/verdant_trail/` — TRACED (CC0, Poly Haven) with 3 exceptions

API-verified slugs (2026-07-05): `jacaranda_tree`, `island_tree_01`, `dead_tree_trunk`, `dead_tree_trunk_02` (separate published slug), `coast_rocks_05`, `coastal_cliff_02`, `coastal_cliff_04`, `rock_07`, `rock_08`, `shrub_01`–`shrub_04` (all four individually), `grass_medium_02`, `stone_01`, `dry_branches_medium_01`, `tree_small_02`, `forest_leaves_04`, `dirt_floor`, `goegap_road` (HDRI).

**Exceptions — QUARANTINE-RECOMMENDED**: `dirt_bank*` (mesh 368 MB + any textures; slug 404), `sticks_debris*`, `grass_debris*` (slugs 404; unpublished scene sub-assets). Code impact: `biome.rs:1052,1065` scatter entries (graceful skip); `dirt_bank` has no sample-set consumer.

**Recommendation**: TRACED → **eligible for release packs** minus the three exception families.

## 3. The two root `.blend` monoliths — QUARANTINE-RECOMMENDED (as wholes)

`assets/Namaqualand.blend` (122.7 MB) and `assets/verdant_trail.blend` (222.3 MB) are the scene assemblies the above directories were decomposed from. Their contents are ~95% traced-CC0 Poly Haven assets, **but each embeds the untraceable sub-assets listed above** (fine_leaf, dirt_bank, debris), and no license document covers the assembly as published. The decomposed, per-asset-traceable outputs supersede them for engine use (BiomePack manifests consume the decomposition, not the blend). **Recommendation**: keep both blends in local quarantine; ship only the traced decomposed files in packs. (If the director wants the blends shipped as authoring sources, the untraced sub-assets must be stripped first — a mutating operation for a later beat.)

## 4. `assets/Road to Vostok Assets Vol.1/` (186 files, 41 prop dirs) — TRACED (CC0) with a takedown caveat

- **Source**: <https://roadtovostok.itch.io/road-to-vostok-assets-vol1> (Road to Vostok, indie game by a Finnish solo developer).
- **License evidence** (fetched 2026-07-05): Wayback snapshot <http://web.archive.org/web/20260422060538/https://roadtovostok.itch.io/road-to-vostok-assets-vol1> — page states *"5. Usage – 100% Free (CC0) / No attribution needed / Commercial use allowed"* and the itch.io storefront metadata field reads *"Asset license: Creative Commons Zero v1.0 Universal."*
- **Caveat**: the live page now returns itch.io's own HTTP 404 (verified 2026-07-05) — the pack was unpublished sometime after 2026-04-22. CC0 is an irrevocable public-domain dedication made at publication (snapshot shows a long-standing public release: 36 ratings, 48 comments), so the dedication stands; the archived snapshot is the durable evidence and is pinned in `THIRD_PARTY_LICENSES.md` §8.
- **Recommendation**: TRACED → **eligible for release packs**. Include the Wayback URL in the release page's attribution notes. If the director prefers extra caution given the takedown, quarantine is the reversible alternative — but the evidence for the CC0 dedication is unambiguous.

## 5. Corrections / cross-references

`goegap_road` was listed by the tracing agent under the verdant_trail cluster (it appears in both packs' texture sets); the slug is verified regardless of which pack cites it. `assets/models/pine_tree_01_1k.glb` (the 1 GiB AD.0 R2 file) traces to Poly Haven `pine_tree_01` (API-verified 2026-07-05, authors Rob Tuytel/Rico Cilliers) — CC0, pack-eligible; its `_1k` name refers to its texture tier, not its geometry weight.

## 6. Adjacent unknowns (recorded for completeness; all QUARANTINE-RECOMMENDED unless traced later)

| cluster | status | note |
|---|---|---|
| `assets/Forest Scene/` scene art (150 tracked files) | UNTRACEABLE | only Unity SDK package licenses exist (machine-generated `Library/`); no license doc for the scene content; `.sln` name is filename-inference only |
| `assets/Symphonie/` (5) | UNTRACEABLE | support/contact README only, no license text |
| `assets/models/Amber-Npc/` (158) | UNTRACEABLE — **elevated risk** | Character-Creator-style export; such exports are typically licensed to the exporting user and NOT redistributable; treat as quarantine-first |
| `assets/castles_forts_asset_pack/` (5), `assets/Texture/` (8), `assets/Mesh/` (2), root `assets/{Albedo,AO,Displacement,Gloss,Normal,Roughness}.jpg` + `Displacement.exr` | UNTRACEABLE | zero license evidence on disk; no name-match to a verifiable source |
| `assets/hdri/polyhaven/*` catalog HDRIs | **TRACED** | all remaining catalog slugs API-verified CC0 2026-07-05 (see manifest §9) |
| Root `assets/*_4k.gltf` material sets (8) | **TRACED** | all 8 slugs API-verified CC0 2026-07-05 (manifest §9) |
| `assets/models/AnimationLibrary_Godot_Standard.glb` | **TRACED** | Quaternius Universal Animation Library, CC0 (manifest §9) |
| `assets/2D assets`, `3D assets`, `Icons`, `UI assets`, `Archive`, `Other` (Kenney library) | TRACED | per-pack in-repo `License.txt` (CC0) + kenney.nl/support |

## 7. Summary for the AD.1 gate

- **Pack-eligible now**: Namaqualand (minus fine_leaf), verdant_trail (minus dirt_bank/debris), Road to Vostok Vol.1, pine_tree_01_1k.glb, all catalog HDRIs, root glTF material sets, the Kenney/KayKit libraries.
- **Local quarantine (never uploaded)**: fine_leaf_01_*, dirt_bank*, sticks_debris*, grass_debris*, both root .blend monoliths, Forest Scene art, Symphonie, Amber-Npc, castles_forts, Texture/, Mesh/, root Albedo-set, the 18 `assets/materials` C7 files + `assets/textures/cobblestone.png` (sample-set rows — see manifest §6), and the 18 C6 runtime materials pending their re-cook (manifest §5).
- **Blocked on director**: logo authorship confirmation; optional RtV caution call (§4).
