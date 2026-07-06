# Third-Party Asset Licenses

**Scope**: the ratified AD.0 sample set (153 in-repo rows + 1 ratified acquisition) and the release-pack contents traced to date (see `docs/audits/DISPOSITION_REPORT_imported.md`), **plus the AD.1.A missed-file-class sweep (§10)** — video/audio/fonts/SVG and other classes the original audit never enumerated. Quarantined material is **intentionally absent** from this manifest — untraceable assets are never shipped in-repo or on release pages. | **Produced**: AD.1, 2026-07-05; AD.1.A addendum (§4 Artlist rows closed, §10 added), 2026-07-05. | **Standard**: an asset appears as TRACED only when a source document states its license terms (upstream license page fetched this session, a LICENSE/README shipped inside the pack, or a documented storefront license). Every URL cited below was fetched on the stated retrieval date; operative terms are quoted. Filename conventions and "site X is usually CC0" were not accepted as evidence. Byte-identity (SHA-256) to a file inside a licensed pack is accepted as evidence that the copy is that pack's asset.

Manifest row format: `paths | upstream source + URL | license (operative term quoted) | attribution requirement | evidence location | retrieval/verification date`.

---

## 1. Pack licenses (in-repo `License.txt` + upstream confirmation)

### 1.1 Kenney packs (sample set uses 5 packs; the wider in-repo Kenney library carries per-pack License.txt files of the same template)

Upstream: <https://kenney.nl/support> — fetched 2026-07-05: *"all game assets on the asset pages are public domain licensed (CC0) … You're free to use them, even in commercial projects. … Attribution is not required."* (Note: the legacy `kenney.nl/data/license.txt` URL now returns 404.)

| paths | upstream | license | attribution | evidence | date |
|---|---|---|---|---|---|
| `assets/3D assets/Nature Kit/**` (18 sample scatter GLBs + bed/tree/tent/campfire/grass twins) | Kenney, <https://kenney.nl> | "License: (Creative Commons Zero, CC0)" | not required ("this is not mandatory") | in-repo `assets/3D assets/Nature Kit/License.txt` (Nature Kit 2.1); upstream <https://kenney.nl/support> | 2026-07-05 |
| `assets/3D assets/Castle Kit/Models/GLB format/tower-square.glb` + `Textures/colormap.png` | Kenney | CC0 (same template) | not required | in-repo `assets/3D assets/Castle Kit/License.txt` | 2026-07-05 |
| `assets/3D assets/Survival Kit/Models/GLB format/{box-large,campfire-pit,rock-a}.glb` + `Textures/colormap.png` | Kenney | CC0 | not required | in-repo `assets/3D assets/Survival Kit/License.txt` | 2026-07-05 |
| `assets/3D assets/Fantasy Town Kit/Models/GLB format/cart.glb` + `Textures/colormap.png` | Kenney | CC0 | not required | in-repo `assets/3D assets/Fantasy Town Kit/License.txt` | 2026-07-05 |
| `assets/3D assets/Retro Medieval Kit/**` (source pack of `assets/models/barrels.glb`) | Kenney | CC0 | not required | in-repo `assets/3D assets/Retro Medieval Kit/License.txt` | 2026-07-05 |

### 1.2 KayKit (Kay Lousberg)

Upstream: <https://kaylousberg.itch.io/kaykit-complete> — fetched 2026-07-05: *"These assets are CC0 so you can use them freely. But please don't resell unmodified copies or claim them as your own. … Creative Commons Zero v1.0 Universal."*

| paths | upstream | license | attribution | evidence | date |
|---|---|---|---|---|---|
| `assets/The Complete KayKit Collection v4/KayKit Adventurers 2.0/Characters/gltf/{Rogue,Barbarian,Mage,Knight}.glb` | Kay Lousberg, <https://kaylousberg.itch.io/kaykit-complete> | "License: (Creative Commons Zero, CC0)" + <http://creativecommons.org/publicdomain/zero/1.0/> | optional ("not mandatory") | in-repo `assets/The Complete KayKit Collection v4/License.txt`; upstream itch page | 2026-07-05 |
| `…/KayKit Skeletons 1.1/characters/gltf/{Skeleton_Warrior,Skeleton_Golem}.glb` | same | CC0 | optional | same License.txt | 2026-07-05 |
| `…/KayKit Forest Nature Pack 1.0/Assets/gltf/Color{1,2}/*.gltf` (9 sample trees/rocks) + `Color{1,2}/forest_texture.png` | same | CC0 | optional | same License.txt | 2026-07-05 |

---

## 2. Byte-identical pack copies under `assets/models/` (SHA-256 evidence, computed 2026-07-05)

These 10 loose files are exact copies of files inside licensed Kenney packs; the pack license above applies.

| path | identical to (SHA-256 match) | license |
|---|---|---|
| `assets/models/barrels.glb` | `assets/3D assets/Retro Medieval Kit/Models/GLB format/barrels.glb` | CC0 (Kenney) |
| `assets/models/bed.glb` | `assets/3D assets/Nature Kit/Models/GLTF format/bed.glb` | CC0 (Kenney) |
| `assets/models/tree_pineDefaultA.glb` | Nature Kit twin | CC0 (Kenney) |
| `assets/models/tree_pineRoundA.glb` | Nature Kit twin | CC0 (Kenney) |
| `assets/models/tree_pineTallA.glb` | Nature Kit twin | CC0 (Kenney) |
| `assets/models/tent_smallOpen.glb` | Nature Kit twin | CC0 (Kenney) |
| `assets/models/tent_detailedClosed.glb` | Nature Kit twin | CC0 (Kenney) |
| `assets/models/campfire_logs.glb` | Nature Kit twin | CC0 (Kenney) |
| `assets/models/tree_default.glb` | Nature Kit twin | CC0 (Kenney) |
| `assets/models/grass.glb` | Nature Kit twin | CC0 (Kenney) |

---

## 3. Poly Haven assets (upstream verified per-slug this session)

Site license: <https://polyhaven.com/license> — fetched 2026-07-05: *"Our assets are all licensed as CC0, which is effectively Public Domain even in jurisdictions that do not support the Public Domain."* Attribution: not required. Per-slug existence verified via `https://api.polyhaven.com/info/<slug>` (HTTP 200 + asset metadata), 2026-07-05.

> **Evidence-tier disclosure (applies to §3.1 and §3.2, NOT §3.3):** the *license* of each slug is upstream-verified, but the link between the local file and its slug rests on the decomposition-preserved filename — there is no in-pack LICENSE and no byte-identity to a fresh download (the LFS bandwidth embargo precludes re-downloading for byte comparison this session). This is a weaker identity tier than §1 (in-pack license) or §2 (SHA-256): rows here are **TRACED (name-linked)**. Rationale for accepting it: these files are outputs of a Blender-scene decomposition whose every *resolvable* member is a published Poly Haven asset, the names carry Poly Haven's exact slug + suffix conventions, and CC0 imposes no attribution/redistribution obligation, so the residual risk is misidentification of CC0-for-CC0. Names that did NOT resolve were honestly excluded (§5/§6), not force-matched. A post-embargo hardening step may byte-compare one representative mesh against a fresh Poly Haven download. §3.3 is stronger: the import script's `SOURCES` map is repo-committed documentation of origin, independent of filenames.

### 3.1 Sample-set HDRIs and textures

| paths | slug (API-verified) | authors (from API) | evidence | date |
|---|---|---|---|---|
| `assets/hdri/polyhaven/kloppenheim_02_puresky_2k.hdr` | `kloppenheim_02_puresky` | Greg Zaal / Jarod Guest | API + <https://polyhaven.com/license> | 2026-07-05 |
| `assets/hdri/polyhaven/kloppenheim/kloppenheim_06_puresky_2k.hdr` | `kloppenheim_06_puresky` | (in-repo README + API) | in-repo `assets/hdri/polyhaven/kloppenheim/README.md`; polyhaven.com/license | 2026-07-05 |
| `assets/hdri/polyhaven/spruit_sunrise/spruit_sunrise_2k.hdr` | `spruit_sunrise` | (in-repo README) | in-repo README.md; polyhaven.com/license | 2026-07-05 |
| `assets/hdri/polyhaven/venice_sunset/venice_sunset_2k.hdr` | `venice_sunset` | (in-repo README) | in-repo README.md; polyhaven.com/license | 2026-07-05 |
| `assets/textures/grass_bermuda_01_diff_1k.jpg` | `grass_bermuda_01` | (API) | API + polyhaven.com/license | 2026-07-05 |

### 3.2 Sample-set scatter meshes (verdant_trail blend-decomposition, 22 files ≤3 MB)

Each mesh name maps to an API-verified Poly Haven model slug: `dead_tree_trunk` (+`dead_tree_trunk_02`), `jacaranda_tree`, `rock_07`, `rock_08`, `shrub_01`…`shrub_04` (all four verified individually, author Rico Cilliers), `grass_medium_02`, `stone_01`, `dry_branches_medium_01`, `tree_small_02`. Files: `assets/imported/verdant_trail/meshes/{dead_tree_trunk.001,dead_tree_trunk_02,jacaranda_tree_trunk,rock_07.001,rock_08.001,shrub_01_a,shrub_01_b,shrub_01_d,shrub_02_a,shrub_02_c,shrub_03_a,shrub_03_b,shrub_04_a,shrub_04_b,grass_medium_02_a..e,stone_01_LOD0.001,dry_branches_medium_01_a.002,tree_small_02_a}.glb`. License CC0 per polyhaven.com/license. Retrieval 2026-07-05. **Exceptions — see §5/§6:** `sticks_debris_a.glb`, `grass_debris_a.glb` (slugs do not resolve).

### 3.3 Biome material derivatives in `assets/materials/` (21 files: grass, forest_floor, mountain_rock, mud, stone, rock_slate, dirt × base/_n/_mra)

Chain of evidence: `scripts/import_terrain_textures.py:53-106` (`SOURCES` map) documents each family's Poly Haven source; the script downscales to 2048 and repacks MRA; adding commit `1fc266c93` (2026-03-18). Source slugs all API-verified 2026-07-05:

| material family | Poly Haven source slug | authors (API) |
|---|---|---|
| grass | `grass_medium_01` | Rob Tuytel / Rico Cilliers |
| forest_floor | `forest_ground_04` | Rob Tuytel / Rico Cilliers |
| mountain_rock | `rock_face_03` | Dario Barresi / Rico Cilliers |
| mud | `forest_ground_01` (script spells `forrest_ground_01`; API resolves) | Rob Tuytel |
| stone | `rocky_trail` | Amal Kumar |
| rock_slate | `rock_moss_set_01` | Kless Gyzen |
| dirt | `ganges_river_pebbles` | Amal Kumar |

License CC0 (polyhaven.com/license). The planned in-repo 1K derivatives (AD.4) inherit the same provenance. Transparency note: the adding commit (`1fc266c93`) titles these "backup placeholder textures," and repo docs describe the current material tier as pipeline scaffolding slated for replacement — provenance is sound regardless of that art-direction status.

---

## 4. First-party (project-original, MIT — engine license)

| paths | evidence | date |
|---|---|---|
| `assets/models/greybox/*.gltf` (6 files) | generated by `examples/greybox_generator/src/{echo_grove,boss_courtyard,fractured_cliffs,loom_crossroads,loomspire,side_alcove}.rs` (each `fs::write`s its gltf) | verified 2026-07-05 |
| `assets/materials/{sand,snow}{,_n,_mra}.png` (6 files) | procedurally generated: `scripts/import_terrain_textures.py:108` `PROCEDURAL=["sand","snow"]`, `gen_sand`/`gen_snow` (FBM noise, fixed seeds); commit `1fc266c93` | verified 2026-07-05 |
| `assets/Astraweave_logo.jpg` (2048², 1,469,159 B; byte-identical tracked copies at `docs/src/assets/` + `gh-pages/assets/`, SHA-256 `1c8d90cba6dc…`; plus derived 1024² `docs/branding/Astraweave_logo.jpg`, 114,916 B, distinct hash — separately produced crop; plus one untracked mdBook build copy under `docs/book/`) | **CLOSED (AD.1.A, director-ratified)**: generated via Artlist **AI Toolkit** from the author's own prompt + the author's own ChatGPT-generated input concept image; classified **"AI Output"** under Artlist ToS — rights assigned to the author, no standalone-distribution restriction, survives subscription expiry; AI-generation disclosed (copyrightability of pure AI output contested; immaterial for MIT distribution). Evidence: generation-history screenshots, `docs/audits/evidence/artlist_generation_history_2026-07-05/` (archived; **name/visual-linked** tier). Live consumers: editor window icon + top-bar brand mark (`tools/aw_editor/src/ui/branding.rs:11,28` → `main.rs:10033`, `main.rs:5808`), splash logo phase (`tools/aw_editor/src/splash.rs:21`) | 2026-07-05 |
| `assets/8-second_Cinematic_logo_opening.mp4` (19,149,666 B; **plain git blob — `.mp4` is not an LFS pattern in `.gitattributes`**) | **CLOSED (AD.1.A, director-ratified)**: generated via Artlist **AI Toolkit** from the author's own prompt; classified **"AI Output"** under Artlist ToS — rights assigned to the author, no standalone-distribution restriction, survives subscription expiry; AI-generation disclosed. Same evidence directory as the logo (same generation session, prompt "logo forming in space", items badged "Video"). Live consumer: editor splash video, decoded at every launch via the `mp4` + `openh264` crates (`tools/aw_editor/src/splash.rs:22,444-500`; `tools/aw_editor/Cargo.toml:104-105`); missing-file fallback is graceful (0.8 s logo-only splash + one `tracing::warn`, `splash.rs:129-134,299-301`); click/key skips (`splash.rs:115-122`). **Scope limit (director instruction)**: this Artlist AI-Output disposition covers ONLY these two files; any other Artlist-sourced material requires its own Output-vs-Asset determination | 2026-07-05 |
| `astraweave-audio/tests/assets/*.wav` (13) + `astraweave-audio/tests/fixtures/{music_test.ogg,sfx_test.wav,voice_test.wav}` | **FIRST-PARTY-GENERATED** — synthetic sine/chord/formant audio produced by in-repo generators `astraweave-audio/tests/test_asset_generator.rs` and `tests/fixtures/generate_fixtures.rs`; every file's byte size matches the generator math exactly (e.g. `test_beep_440hz.wav` 22,094 = 44 + 0.5 s × 22,050 Hz × 2 B). Note: `fixtures/README.md:25` ("NOT checked into git") is stale — all are tracked. `music_test.ogg` is RIFF/WAVE despite the extension (generator's own note) | verified 2026-07-05 |
| `docs/src/assets/og-image.svg` (2,936 B) | **first-party** — project-authored Open Graph/Twitter social card (literal "AstraWeave / AI-Native Game Engine" text + project URL), wired via `docs/theme/head.hbs:5,12`; added `faf8ef43` | verified 2026-07-05 |

---

## 5. SUBSTITUTE-proposed (usable slot, current file untraceable — traced replacement identified)

| paths | problem | proposed substitute (traced) | consuming code path |
|---|---|---|---|
| `assets/materials/{cobblestone,gravel,ice,metal_rusted,moss,wood_planks}{,_n,_mra}.png` (18 files) | on-disk runtime copies added 2026-02-06 (`54d10f736`, message silent on source); no generator produces them; SHA-256 differs from the documented `assets_src` copies — origin unknown | **re-cook from `assets_src/materials/` same-name sources**, whose provenance is documented in commits `f56a76124`/`f5387f20e`/`76b15948d` and `assets/asset_manifest.toml`, and whose upstreams were all verified 2026-07-05: gravel→`gravel_concrete_03`✓, metal_rusted→`rust_coarse_01`✓, moss→`moss_01`✓, wood_planks→`wood_floor_deck`✓, cobblestone→`rocky_trail`✓ (all Poly Haven CC0), ice→ambientCG `Ice003` (https://ambientcg.com/view?id=Ice003 fetched 2026-07-05: *"All assets are released under the Creative Commons CC0 license"*) | `assets/materials/<biome>/materials.toml` layers; editor terrain `canonical_terrain_pack.rs:181,215` |
| `assets/imported/verdant_trail/meshes/{sticks_debris_a,grass_debris_a}.glb` (2 files) | slugs `sticks_debris`/`grass_debris` return 404 on the Poly Haven API and no search hit — unpublished scene sub-assets of the verdant_trail blend; no license doc names them | substitute with a traced debris asset in a later acquisition (Poly Haven has published debris-family slugs, e.g. `bark_debris_01` — **candidate only, not yet traced**), or QUARANTINE | `astraweave-terrain/src/biome.rs:1065` (sticks_debris), `:1052` (grass_debris) — scatter skips missing files gracefully (`engine_adapter.rs:2578`) |

---

## 6. QUARANTINE-RECOMMENDED (sample-set rows; leave the public repo until traced)

| paths | why untraceable | code path that degrades |
|---|---|---|
| `assets/materials/{cloth,plaster,rock_lichen,roof_tile,tree_bark,tree_leaves}{,_n,_mra}.png` (18 files) | added 2025-09-22/29 (`c166bdc27`, `b24317195`) with source-silent messages; no generator emits them; the `assets_src` same-name copies are equally untraceable (no commit/doc states a source); circumstantial name-echoes to Poly Haven slugs did NOT meet the tracing standard | biome `materials.toml` layers naming these families (forest loses tree_bark/tree_leaves/rock_lichen layers; terrain/biomes sets lose cloth/plaster/roof_tile) — editor terrain falls back per-layer; grassland default (5 layers) is unaffected |
| `assets/textures/cobblestone.png` | added via GitHub web-UI uploads ("Add files via upload", `be6db6fb2`/`8cfdff8b8`/`d4d8bdfba`); no generator, no attribution linkage; NOT derived from the Poly Haven cobblestone download (pixel cross-correlation 0.0095) and not a Kenney colormap | `examples/unified_showcase/src/main.rs:1130` (ground texture) — substitute: re-point to a traced texture |

---

## 7. Ratified acquisition (D5)

| paths (incoming) | upstream | license | evidence | dates |
|---|---|---|---|---|
| Quaternius Stylized Nature MegaKit — `CommonTree_1` (gltf + bin + 3 PNGs, ~10.3 MB) | <https://quaternius.com/packs/stylizednaturemegakit.html> (CC0 zip mirror: opengameart.org, 104,088,529 B) | CC0 — <https://quaternius.com/faq.html> fetched 2026-07-05: *"All models are under the CC0 License. … attribution is not necessary."* | download parsed 2026-07-04 (AD.0): materials `alphaMode:"MASK"`, `alphaCutoff:0.2`, `doubleSided:true` verified | acquired 2026-07-04; license verified 2026-07-05 |

---

## 8. PENDING-DIRECTOR checklist

1. **AlkaKrab music — `assets/audio/{Tracks,Loops}/` (60 audio files, ~922.6 MB, currently tracked + pushed)** — the license is TRACED (§10.3) but **forbids redistribution as-is** and requires direct permission for open-source games. Choose one: **(a)** ratify QUARANTINE — remove from the public repo in the AD.6 history rewrite, never ship in packs or on the release page; or **(b)** email the licensor (the PDF invites it: `alkakrab04@gmail.com`) for written open-source-redistribution permission — if granted, the permission email becomes a tracked evidence file and the cluster becomes pack-eligible. Until decided, the cluster is treated as quarantine.
2. **Ratify the AD.1.A sweep quarantine additions** (§10.4): `water_ambient_*` (2.196 GB), `assets/Other/Fonts` (16 files), `assets/Other/Miniguides`, `assets/Archive/Isometric Renders`, `assets/Goodies` (+ 2 root `.url` shortcuts), `assets/textures/billboard_foliage_billboards_*.psd` (3 files, 150.9 MB).
3. **Road to Vostok takedown awareness** (informational, decision optional): the Vol.1 itch.io page is unpublished as of 2026-07-05; the CC0 dedication is evidenced by the 2026-04-22 Wayback snapshot (see disposition report). CC0 is irrevocable, but the director may prefer to keep the archived-evidence URL pinned in this manifest permanently: <http://web.archive.org/web/20260422060538/https://roadtovostok.itch.io/road-to-vostok-assets-vol1>

*Resolved 2026-07-05 (AD.1.A)*: the former item 1 — `Astraweave_logo.jpg` authorship — is closed as first-party Artlist AI Output (§4), together with the splash video.

---

## 9. Release-pack contents traced to date

Full cluster tables and quarantine recommendations: `docs/audits/DISPOSITION_REPORT_imported.md`. Summary: **Namaqualand** — 17 Poly Haven model/texture slugs API-verified CC0; `fine_leaf_01` quarantine-recommended. **verdant_trail** — all major slugs verified CC0; `dirt_bank`/`sticks_debris`/`grass_debris` quarantine-recommended. **Road to Vostok Vol.1** — CC0 per archived itch.io storefront terms. **`assets/models/pine_tree_01_1k.glb`** (the 1 GiB R2 file) — Poly Haven `pine_tree_01`, CC0 (API-verified 2026-07-05). **HDRI catalog set** — `goegap`, `table_mountain_2`, `misty_pines`, `rainforest_trail`, `qwantani_moonrise_puresky`, `rogland_clear_night`, `kloofendal_48d_partly_cloudy_puresky`, `rogland_sunset` all API-verified CC0. **Root `assets/*_4k.gltf` material sets** — `aerial_beach_01`, `aerial_rocks_01`, `coast_land_rocks_01`, `sandy_gravel_02`, `snow_02`, `leafy_grass`, `ganges_river_pebbles`, `forest_leaves_02` all verified CC0. **`assets/models/AnimationLibrary_Godot_Standard.glb`** — Quaternius Universal Animation Library (https://quaternius.itch.io/universal-animation-library, CC0 per Quaternius site-wide statement). **AD.1.A additions**: the 16 `assets/audio` Kenney packs + 3 ogg-bearing Kenney packs (§10.1) and `assets/textures/alps_field_2k.hdr` (§10.6) are pack-eligible.

---

## 10. AD.1.A — missed-file-class sweep (2026-07-05)

The original asset audit enumerated only 14 model/texture extensions (see `docs/audits/ASSET_AUDIT_REPORT.md`, Addendum A — the enumeration-coverage correction). This sweep covers the missed classes: video (`.mp4 .webm .mov .avi .mkv`), audio (`.wav .mp3 .ogg .flac .aiff`), fonts (`.ttf .otf .woff .woff2`), skipped image classes (`.gif .ico .svg .webp`), plus a full extension census for anything else binary/media-like. **6,419 files found** in the swept core classes (svg 4,849 · ogg 1,363 · wav 125 · ttf 36 · mp3 20 · otf 14 · woff2 11 · mp4 1); 6,406 tracked, 13 untracked (all gitignored `docs/book/` mdBook build output). Zero `.gif .ico .webp .mov .avi .mkv .flac .aiff .woff` exist repo-wide.

**File-level roll-up (sums to 6,419)**: 6,218 TRACED pack-eligible (Kenney) · 60 TRACED-but-redistribution-restricted → quarantine (AlkaKrab) · 18 first-party (§4) · 110 UNTRACEABLE → quarantine (84 water_ambient + 16 `Other/Fonts` + 4 Miniguides SVGs + 6 Forest Scene WAVs, the last extending the existing AD.1 quarantine cluster) · 13 excluded (build output). Zero silently unresolved.

### 10.1 TRACED — Kenney audio packs (in-repo `License.txt`, CC0)

All 16 pack dirs under `assets/audio/` ship a `License.txt` naming Kenney (www.kenney.nl) and CC0 — every one read in full 2026-07-05. Standard operative text (15 of 16, quoted from `assets/audio/Digital Audio/License.txt`): *"License: (Creative Commons Zero, CC0) http://creativecommons.org/publicdomain/zero/1.0/ / This content is free to use in personal, educational and commercial projects. / Support us by crediting Kenney or www.kenney.nl (this is not mandatory)"*. Casino Audio carries Kenney's older phrasing (*"by Kenney Vleugels (Kenney.nl)"* … *"You may use these assets in personal and commercial projects. Credit … would be nice but is not mandatory."*) — same CC0 grant. Packs (files/bytes incl. the license file itself): Casino Audio 57/962,713 · Digital Audio 64/944,151 · Foley Sounds 87/1,207,694 · Impact Sounds 132/1,312,195 · Interface Sounds 102/1,177,108 · Music Jingles 87/1,459,721 · Music Loops 31/6,029,702 · RPG Audio 53/838,630 · Retro Sounds 1 36/261,522 · Retro Sounds 2 67/825,887 · Sci-Fi Sounds 75/6,169,270 · Synth Voice 1 102/1,111,415 · Synth Voice 2 228/2,382,439 · UI Audio 53/650,115 · Voiceover Pack 96/1,992,578 (voice-actor credits in file: Giselle, Jeffrey M. Smith — non-mandatory) · Voiceover Pack Fighter 49/1,302,941. **Total 1,319 files, 28,628,081 B — pack-eligible.** Also TRACED via root `License.txt` (CC0, read 2026-07-05): `assets/2D assets/Desert Shooter Pack` (40 ogg), `assets/2D assets/New Platformer Pack` (10 ogg), `assets/UI assets/UI Pack` (6 ogg under `Sounds/`).

**Liveness note**: the editor's `EditorAudioBridge` blanket-scans `assets/audio` recursively (`tools/aw_editor/src/audio_bridge.rs:77,242`, ext filter `:31`) and plays any hit via `AudioEngine::{play_music,play_sfx_file,play_voice_file}` (`astraweave-audio/src/engine.rs:64-66,343-345,373-375`) — so **everything under `assets/audio` is live-reachable**, including the quarantine-recommended clusters below.

### 10.2 TRACED — Kenney collection coverage for the swept classes (svg/fonts/swf/ai/capx/tmx/url/zip/stl/3ds)

Per-pack-dir coverage check across the collection roots (counts of swept-class files: `2D assets` 1,425 · `Icons` 1,968 · `UI assets` 1,783 · `3D assets` 223 · `Other` 37 · `Archive` 46): **`Icons` (7 pack dirs), `UI assets` (8), `3D assets` (53) — full root-`License.txt` coverage, zero gaps.** In `2D assets`, 5 zip-only dirs (`Isometric Miniature {Dungeon,Farm,Library,Overworld,Prototype}`) and in `Archive`, 10 zip-only dirs each embed a `License.txt` **inside the zip** (verified by `unzip -l`/`unzip -p`, standard Kenney CC0 text) — TRACED, not gaps. Kenney fonts inside licensed packs (`Icons/Input Prompts/*/Fonts/` 28 files, `UI assets/UI Pack*/Font{,s}/` 6 files) are covered by those packs' licenses. **Three genuine coverage gaps → §10.4**: `assets/Other/Fonts`, `assets/Other/Miniguides`, `assets/Archive/Isometric Renders`.

### 10.3 TRACED but redistribution-restricted — AlkaKrab music → QUARANTINE-RECOMMENDED (pending §8 item 1)

| paths | source doc | terms | status |
|---|---|---|---|
| `assets/audio/Tracks/` + `assets/audio/Loops/` (index casing; worktree lowercase — see the case-mismatch discrepancy in the audit addendum): 10 titles × {mp3,ogg,wav} × 2 dirs = 60 audio files, ~922.6 MB, added `54d10f736`, reachable from `origin/main` | `AlkaKrab Music License Info.pdf`, shipped in-pack in 3 byte-identical copies (md5 `40d1fcff12a01920ca9a5aef3c67ead4`); authorship corroborated by embedded metadata (ID3 `TPE1=alkakrab`, `TYER=2024`, Studio One encoder; Vorbis `ARTIST=alkakrab`) | Royalty-free, commercial use allowed, credit optional — **but**: *"No reselling, sublicensing, or redistribution of the track as-is. If you want to use these tracks in open-source games, contact me directly for a permission."* and prohibited: *"Redistribution or resale of the music file as-is without permission (e.g., in sound packs, music libraries)."* | **The current public-repo distribution appears to violate the license.** Never upload to packs/release page. Remove in the AD.6 rewrite unless written permission is obtained (§8 item 1). |

### 10.4 UNTRACEABLE → QUARANTINE-RECOMMENDED (sweep additions; never uploaded anywhere)

| paths | why untraceable | notes |
|---|---|---|
| `assets/audio/water_ambient_{mono,stereo_1,stereo_2}/` — 84 WAV, **2,195,628,728 B (2.196 GB)**, tracked (LFS), added `54d10f736` | No license doc in-pack or anywhere in repo; `assets/README.md` license notes silent; `scripts/validate_assets.ps1:252` *asserts* "third-party" without a source. Measured signature (non-evidence, recorded honestly): REAPER-originated `bext` chunks, origination dates 2019-08, 96 kHz/24-bit, pre-rename dirs `loadless_WATER_*` (`docs/current/ASSET_CLEANUP_REPORT.md:76-80`), naming scheme `<Mono\|Stereo> CATEGORY (descriptors).wav` | Largest unresolved cluster by bytes in the entire repo. Live-reachable via the editor audio scan (§10.1 note) but no curated consumer. Quarantine = also excluded from release page. |
| `assets/Other/Fonts/` — 16 TTF ("Kenney Blocks" … "Kenney Thick") + Preview.png + 2 webfont zips | No `License.txt` in dir, tree, zips (woff/css only), or any ancestor up to `assets/` — the only Kenney sub-tree with fonts and no license doc | All 16 are **orphaned-in-code** (zero loaders reference them; UI text renders via egui's embedded fonts, §10.5) |
| `assets/Other/Miniguides/` (incl. 4 SVGs: miniguide{1_tiles,2_character,4_background,5_ui}.svg) | No license doc anywhere in tree or ancestors — the dir's `Readme.txt` only points to Kenney YouTube tutorials, not a rights grant | |
| `assets/Archive/Isometric Renders/` — 16 zips of PNG sprites | Only Archive dir whose zips embed **no** `License.txt` (all 16 verified) | |
| `assets/Goodies/` (9 files: 2 PDFs, Thanks.txt, 6 wallpaper JPGs) + `assets/Visit Kenney.url` + `assets/Visit Kenney Bluesky.url` | `Thanks.txt` is a thank-you note, not a license (*"…token of appreciationn for your purchase…"* — no rights grant); "Kenney is usually CC0" is inadmissible per the standard | Low practical risk; standard applied honestly |
| `assets/textures/billboard_foliage_billboards_{albedo,alphamask,normal}.psd` — 3 × 50,331,688 B (150.9 MB), added `4c611964` | No source doc, no in-repo generator writes PSD. Measured: bare metadata-free raw-PSD headers whose byte math (26 + 12 + 2 + 3×4096²) matches file size exactly; albedo/alphamask pixel data opens all-zero — a pattern real Photoshop saves essentially never produce | Referenced only INDIRECTly via the Namaqualand blend-decomposition manifest (`assets/imported/Namaqualand/manifest.json`, `biome_pack.rs:235,352`) — they are textures *named inside* the .blend, not importer-authored |

### 10.5 TRACED — runtime UI fonts (egui embedded; not repo files)

No workspace code registers custom fonts (broad grep for `FontDefinitions|FontData|add_font|set_fonts|include_bytes!(*.ttf/otf)` → zero; `egui_extras` absent from the dependency tree). All UI text renders from `epaint_default_fonts` 0.32.3 (via egui/eframe/epaint 0.32.3, `default_fonts` feature on — root `Cargo.toml:185`); license files read from the local cargo-registry crate copy (`…\.cargo\registry\src\index.crates.io-…\epaint_default_fonts-0.32.3\fonts\`):

| font | role | license | evidence file (in-crate) |
|---|---|---|---|
| Ubuntu-Light | proportional primary (all body text) | Ubuntu Font Licence 1.0 | `UFL.txt` — *"This licence allows the licensed fonts to be used, studied, modified and redistributed freely."* |
| Hack-Regular | monospace primary | MIT (Source Foundry, 2018; bundles Bitstream Vera terms — moot, unmodified) | `Hack-Regular.txt` |
| NotoEmoji-Regular | emoji, first priority | SIL OFL 1.1 | `OFL.txt` |
| emoji-icon-font | emoji/icons, secondary | MIT (John Slegers, 2014) | `emoji-icon-font-mit-license.txt` |

Crate SPDX: `(MIT OR Apache-2.0) AND OFL-1.1 AND Ubuntu-font-1.0` (`Cargo.toml.orig:9`); `deny.toml:23` already allowlists the UFL component. **No distributed font has an unknown license**: the 50 tracked in-repo TTF/OTF are all orphaned-in-code (34 covered by Kenney pack licenses, 16 → §10.4); the 11 woff2 are untracked `docs/book/` build output (their local license files state Open Sans = Apache-2.0, Source Code Pro = OFL 1.1); `gh-pages/` ships zero fonts (system font stacks only); `git ls-files` shows zero tracked woff/woff2/eot. Cosmetic note: `docs/benchmarks/index.html:60` references a `../fonts/…css` that does not exist (dangling, zero font bytes behind it).

### 10.6 Remaining sweep rows

- **`assets/textures/alps_field_2k.hdr`** (6,531,494 B) — Poly Haven `alps_field` verified via `https://api.polyhaven.com/info/alps_field` (HTTP 200, name "Alps Field", author Andreas Mischok) 2026-07-05, same standard as §3 → **TRACED, pack-eligible**. (The other three `assets/textures` 8K HDRs were already API-verified in AD.1/§9.)
- **`assets/audio/ambient/manifest.toml`** — first-party placeholder config; the 13 `.ogg` files it names do not exist and nothing parses the TOML (`astraweave-render/src/biome_audio.rs:32-45` hardcodes the same paths independently; its only non-test caller is a demo example). No asset to license.
- **`assets/Forest Scene`** — the sweep adds 6 WAV (262,118 B) plus census classes (.tif 53/362.2 MB, .mdb 64, .dll 114, .cube 6, .psd 48, .unitypackage 3, .pdf 1) to the existing AD.1 QUARANTINE-RECOMMENDED cluster; **`assets/Road to Vostok Assets Vol.1`** adds 39 PSD (1,229,911,141 B) to its existing TRACED-CC0 cluster; **KayKit** adds 1 PDF + 8 .url to its TRACED cluster. Cluster verdicts unchanged.
- **`docs/book/`** — gitignored mdBook build output (`.gitignore:13`), excluded from the provenance denominator (13 files: 11 woff2 + og-image.svg + favicon copies).
- **SVG class liveness**: no SVG decoder exists workspace-wide (no resvg/usvg/egui_extras anywhere in `Cargo.lock`); the class is asset-library payload only.
