# L.2 Outcome — IBL for editor terrain: engine fixed, wiring shipped, A/B measured; STOPPED at the calibration gate

**Session**: L.2 (Option A as ratified 2026-07-27) · **Baseline**: `2e938ff9d` (L.1) · STOP report: `L2_PHASE0_STOP.md` (`4c139460d`)
**Machine**: GTX 1660 Ti Max-Q · Vulkan · driver 592.82 (min-spec)

## 0. Status

| Piece | State |
|---|---|
| Irradiance face-blindness fix (ratified, test-first) | **DONE** — `b1295b93f`, regression test run-and-FAILED at `4c139460d` first (spread 0.00%) |
| v-flipped bake-writes fix (found by the test's CPU reference) | **DONE** — same commit; all six irradiance faces now agree with an independent CPU reference to ≤2.1% |
| Wiring: bake-at-init, group(3) bind, sample, ambient replaced, AO on IBL | **DONE** — this commit; design exactly as ratified |
| A/B at the four pinned stations + AO delta + attribution | **DONE** — §3; AO went 0.00% → 99.91–100% of pixels |
| Perf gate | **PASS** — §6; IBL cost below run-to-run noise |
| **Default calibration** | **STOPPED** — §4/§5; the ratified STOP condition fired, with a deeper diagnosis than the LDR clamp; candidate placements previewed for one-round-trip ratification |

## 1. The engine fixes (commit `b1295b93f`)

The ratified test (`astraweave-render/tests/ibl_irradiance_faces.rs`, `#[ignore]`d GPU) bakes
the tracked `kloppenheim_02_puresky_2k.hdr` at Medium and probes the irradiance cube's six
face centres. **Run at `4c139460d` first, it FAILED**: all six probes returned the identical
value — rgb (0.20935, 0.25635, 0.32617), spread 0.00% — the face-blind convolution.

Fix 1 (ratified): the irradiance pass now receives a per-face uniform and derives
`N = uv_to_dir(face, uv)` with the same face table the equirect and specular passes already
used; `conv_pl` gains the face BGL; the false in-source comment is corrected.

Fix 2 (found by the test's CPU reference — an independent integrator over the source
equirect with the shader's exact 60×30 quadrature and tangent basis): after fix 1 the
±X/±Z faces agreed with the reference to 1–2% but ±Y read **0.70× / 1.58×** — the signature
of **v-flipped bake writes**. Every bake pass derived `uv` from clip space without flipping
v (framebuffer row 0 is NDC y=+1; texture v=0 is row 0), storing every cube face vertically
flipped versus the hardware cube convention and the BRDF LUT roughness-flipped. Horizontal
irradiance integrals are invariant under a y-flip (reflection isometry) — which is exactly
why the defect hid; the specular prefilter only *looked* right through a double-flip
cancellation (flipped env read × flipped placement write) that fix 1 alone would have
broken. One-line fix in all five write-pass vertex stages.

**Post-fix validation** (all in the committed test):

| Probe | Result |
|---|---|
| Face-centre spread | 74.12% (pre-fix 0.00%) |
| +Y vs −Y (the ratified assertion) | 0.16538 > 0.07950 ✅ |
| GPU/CPU per-face ratios | 1.010, 1.005, 0.991, 0.979, 1.003, 0.997 (≤2.1%) |
| BRDF LUT A(r=0.05) vs A(r=0.95) at NdotV 0.9 | 0.9697 > 0.3665 ✅ (un-flipped) |

## 2. The wiring (this commit — exactly the ratified design)

- **Bake at init**: `EngineRenderAdapter::new` → `bake_default_environment()` — source
  `assets/hdri/polyhaven/kloppenheim_02_puresky_2k.hdr` (catalog `kloppenheim_daytime`,
  CC0, **git-tracked** — present in a bare clone), tier **Medium** (the convention of both
  pre-existing editor bake sites), probed from `assets` / `../../assets`; absence or bake
  failure → `tracing::warn!` + continue on the fallback bind groups (a fresh clone still
  launches). Bake is logged with source, tier, and wall time.
- **Bind**: dedicated 5-entry `terrain_ibl_bgl` at **group(3)** (spec cube, irradiance
  cube, BRDF LUT, sampler, the *same* `ibl_params_buf`), created by the shared
  `TerrainMaterialManager::create_terrain_ibl_bgl` (renderer + tests, no drift), bind
  group owned by `Renderer` and rebuilt **only** inside `rebuild_ibl_bind_group`
  (§7.7: shared views, one owner, one rebuild point). Terrain fragment stage lands at
  **14/16** sampled textures, preserving headroom for L.3's shadow map (the 9-entry
  group-5 layout would have hit 16/16). Bound once per frame before the chunk loop.
- **Sample**: `compute_ibl` hoisted **verbatim** into `shaders/ibl_common.wgsl`, consumed
  by both `SHADER_SRC` and the terrain concat (no second implementation); terrain declares
  the same globals at group(3). At `pbr_terrain_forward.wgsl` the flat ambient
  (`ambient_color × ambient_intensity × 0.35`) is **replaced** (no double-count) by
  `compute_ibl(N, V, base_color, metallic, roughness, F0) * final_ao` — AO on the indirect
  terms only, never direct sun, per the ratified directive.
- **Panel honesty (binding condition 3)**: the World-panel ambient color/intensity
  controls are relabelled **"(statics)"** with hover text stating terrain is
  environment-lit since L.2 — honestly repurposed, not inert: they still drive the
  static-mesh ambient floor, which is what they now say they do. (Proposed treatment;
  trivially changeable to disabled-with-tooltip if preferred.)
- **Sky flip (accepted)**: with `ibl_resources` present the sky renders the HDRI equirect.
  None of the four pinned stations contains a sky pixel (proven §3), so the A/B is 100%
  ground; the sky change is visible in the live editor.

## 3. The A/B (BEFORE at `2e938ff9d`-state, AFTER at this commit's default)

**Attribution is complete**: the ED-3 normals-debug captures (which bypass terrain
lighting) are **byte-identical** before/after at all four stations — 0 differing pixels —
so the sky/ground mask is empty of sky and every changed lit pixel is terrain lighting.
100.00% of lit pixels differ at every station. Frames: `d:/tmp/l2_staging/{before2,after,…}`.

Mean luma / sd per station (962×501 boundary; 1024×768 others):

| Leg | desert_boundary | desert_close | grass_close | grass_mid |
|---|---|---|---|---|
| **before** (L.1 state) | 114.31 / 9.17 | 112.63 / 11.27 | 82.31 / 12.49 | 83.93 / 9.87 |
| **after** (shipped default: image-avg intensity **2.506**) | 178.61 / 23.53 | 178.18 / 25.02 | 134.01 / 31.91 | 135.86 / 30.16 |
| after + AO→1.0 (leg M-AO) | 184.36 | 184.17 | 154.90 | 156.40 |
| after, unclamped avg (intensity 2.399) | 176.92 | 176.47 | 132.41 | 134.26 |
| after, diffuse-only (spec zeroed) | 175.71 | 175.49 | 130.18 | 131.86 |
| **candidate: env-clamp 2.0 + intensity 1.5** | 145.99 / 10.03 | 145.11 / 13.08 | 102.92 / 15.59 | 104.31 / 11.98 |
| conservative: env-clamp 2.0 + intensity 1.0 | 133.72 / 10.25 | 132.64 / 13.02 | 94.63 / 14.59 | 96.12 / 11.30 |

**The AO number (was 0.00% at T.2f)**: forcing AO to 1.0 vs texture changes
**99.94 / 99.91 / 100.00 / 100.00%** of pixels, mean deltas **+5.75 / +5.99 / +20.89 /
+20.54** — the T.2a-repaired material AO is finally expressed, strongly on grass.

**L.1 controls survive**: pushing the pinned defaults is still byte-neutral (0 differing
pixels); exposure 0.5/1.35/3.0 still 99.94–100% pairwise-different; the late-push test was
recalibrated for the IBL fill (dark push now also pushes exposure 0.5: landed 92.31 vs
~178 dropped, threshold 140 — comment in `l1_proof.rs` cites this).

## 4. The calibration STOP — three compounding defects, one root: the HDRI's sun

At the shipped default the frames are **washed out and glittery** (desert reads as pale
frosted chalk). The ratified STOP condition ("if the LDR clamp visibly distorts the A/B's
intensity placement, STOP and show me") fired — and the diagnosis went deeper than the
LDR clamp:

1. **Intensity over-drive (the wash).** `ibl_intensity = clamp(0.35/avg, 0.3, 3.0)` from
   an image average. Measured for kloppenheim_02: clamped-LDR stride-4 avg **0.1397** →
   intensity **2.506**; unclamped stride-4 **0.1459** → **2.399** (the LDR clamp itself is
   nearly immaterial — the stride grid misses the tiny sun core either way); full-res
   unclamped **0.2292** → 1.527; log-avg **0.1004** → 3.0 (clamped). A linear image
   average of a puresky *without* its sun spike reads the scene as dim and cranks the fill
   to ~2.4–2.5×, making the terrain's diffuse IBL ≈ 1.5–2× the analytic sun's diffuse.
   The rgb32f "fix" alone was measured to change means by only −1.7 luma — **the scheme,
   not the clamp, is the defect**.
2. **Irradiance fireflies (the glitter).** The convolution's fixed 60×30 quadrature
   aliases the sun core: a tilt-fan probe within ~14° of +Y spans **35.1%**
   (0.1468–0.2263) where true sky irradiance varies ~1–2%. The speckled 64² cube, sampled
   by per-pixel boosted normals, is the white glitter — confirmed diffuse-side: zeroing
   specular changed means by only −1.0/−2.2 and the specks persisted.
3. **Sun double-counting.** The HDRI's sun energy enters the IBL terms while the scene
   also runs an analytic directional sun.

**Previewed remedy (temporary legs, reverted; frames on disk):** clamp env radiance for
the *bake cube only* (`min(rgb, 2.0)` in the equirect→cube pass — the visible sky reads
the unclamped equirect and keeps its real sun) + a fixed intensity. At clamp2+1.5 the
glitter is gone, sd returns to 10–16 (the raw default's sd 23–32 was mostly noise), the
grass finally reads as a meadow (moss/russet hue variation, form shading), the desert's
relief reads with soft detail (though cooler/paler than before — blue skylight over warm
albedo; intersects the already-flagged desert-albedo art-direction call).

## 5. The ratification menu (one round trip)

**Recommended: A + B as one small calibration beat (~10 lines, both previewed this
session):**

- **A. Sun-clamp the bake source** (equirect→cube `min(rgb, 2.0)`; visible sky
  unaffected). Kills the fireflies and the double-count. A permanent fan-spread assertion
  (probe already in the test, currently print-only) locks it.
- **B. Fixed default IBL intensity** replacing the image-average normalisation —
  **1.5 previewed** (desert 145 / grass 103) and **1.0 previewed** (133 / 95); the
  exposure slider remains the director's global knob. (A principled alternative —
  normalise on baked upper-hemisphere irradiance — costs more design; the fixed constant
  is honest for a single-HDRI editor default.)
- C. Optional: the rgb32f unclamp in `compute_hdr_avg_luminance` (measured nearly inert;
  logged residue either way).
- D. Desert warmth at the gate: art direction (sun colour vs blue skylight vs procedural
  desert albedo — the T.2a director call).
- E. Bake init cost: **2,480 ms cold** (decode 5.5 MB + 63-submit Medium bake), once per
  adapter. Acceptable as a one-time editor-startup hitch? (Mitigations if not: bake off
  the first frame, decode cache to disk, Low tier at 620 ms-ish.)

## 6. Perf gate (min-spec, wall-clock+sync — TIMESTAMP_QUERY unavailable on this driver)

| Run | median | p10 | p90 | n |
|---|---|---|---|---|
| BEFORE (`b1295b93f`) | 28.663 ms | 27.522 | 42.777 | 300 |
| AFTER (wired) | 27.305 ms | 26.574 | 28.304 | 300 |

The IBL adds ≤ run-to-run noise (the median moved −1.36 ms; the before run carried a noisy
p90). Far under the ~1.5 ms STOP.

## 7. Verification

- `ibl_irradiance_faces` GPU test: **1 passed** (all five assertions; §1 table).
- naga gates: `shader_validation` **4/4** incl. the terrain concat with `ibl_common.wgsl`;
  `ibl_common.wgsl` registered as a concatenation fragment.
- Forward pipeline GPU tests (`--features gpu-tests`): both L.2-touched tests pass
  (`terrain_manager_forward_pipeline_builds_without_validation_errors`,
  `terrain_manager_forward_round_trip`).
- `l1_proof`: exposure honesty invariant **0 differing pixels**; late-push recalibrated
  (§3) and passing.
- `l2_proof` (new): the A/B harness, 4 stations × lit+normals.
- Render lib suite + editor lib suite + `cargo check --workspace`: see the closing report
  of this session for the run records (green apart from the documented environmental
  water-device flakes).
- **Pre-existing red gates found and stash-proven, not caused by L.2**: (a) the
  shader-validation vacuity floor was red at baseline (58 validated vs floor 60, identical
  under the floor-era manifest; recalibrated 60 → 55 with forensic comment); (b) the
  deferred `TERRAIN_SPLAT_SHADER` fails naga under `--features gpu-tests`
  (`height_blend` invalid call) at baseline — logged, untouched (deferred path is not
  L.2's).

## 8. Residue (logged, unowned unless ratified)

1. The §4 calibration items pending ratification (intensity scheme, sun-clamp, fan-spread
   assertion promotion).
2. `compute_hdr_avg_luminance` LDR clamp + doc-comment claiming a log-average it doesn't
   compute (ratified: logged).
3. `hdri_catalog.toml` `default` names a non-existent entry (masked by the fallback
   matrix).
4. Specular prefilter's 512 solid-angle hardcode (exact for Medium — the tier in use).
5. Deferred `TERRAIN_SPLAT_SHADER` naga failure under `gpu-tests` (pre-existing,
   stash-proven).
6. `SKY_WGSL` procedural placeholder still face-blind — today's "Remove HDRI" path rebakes
   into it (unchanged behaviour).
7. Sampler-per-bake allocation in `rebuild_ibl_bind_group` (pre-existing; now 1/bake
   shared by both bind groups).
8. Skinned meshes still have no IBL (hardcoded 0.08 lift) — statics/props lane.
9. Editor never consumes the hdri_catalog biome×time matrix (manual picker + L.2 default
   only).

## 9. Readiness

The wiring is structurally complete and regression-locked; nothing further is needed from
the engine to close the beat. **The director's calibration ratification (§5 A/B/E) is the
only open input**; on ratification the calibration lands as a follow-up commit and the
close-up gate can be judged with the exposure slider in hand.
