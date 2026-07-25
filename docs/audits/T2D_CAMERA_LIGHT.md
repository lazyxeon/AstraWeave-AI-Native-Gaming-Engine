# T.2d — The camera-light defect: diagnosis (NOT CONVICTED — STOP for the director)

> **Beat:** T.2d (terrain series) · **Date:** 2026-07-25 · **Baseline commit:** `c2dbf8400`
> **Symptom (director, 2026-07-25):** *"the camera itself is a light source — as I get really close to the terrain it gets brighter and casts shadows at the borders of the light boundary like a light source."*
> **Outcome:** the phenomenon is **partially reproduced**. Camera-dependent shading is **confirmed and measured**. The *dramatic* part of the description — the bright region with a visible boundary — **did not reproduce** in the offscreen path. **No fix has been applied**, because nothing has been convicted at the magnitude the symptom describes.
> Frames + CSVs: `d:/tmp/t2d_staging/{head,lod_const,flat_normal,nofog}/` (session-local).
> Anti-drift honoured: no material acquisition, no shader tuning retained, no scatter/water/hex work. Every shader edit in this beat was a temporary A/B leg and **all were reverted** (`git diff astraweave-render/` is empty at the commit).

---

## 0. Summary — what is and is not established

**Established (measured):** shading in the terrain path *is* camera-dependent. Holding the world fragment fixed at screen centre and moving only the camera, its luminance rises **+5.9%** as the camera approaches (110 m → 12 m). Under a correct lighting model that number must be zero. This is a real defect and it means close-up material judgments are made under a camera-varying exposure.

**Not established:** that this is *the* thing the director saw. The measured effect is a **smooth ~6% gradient with no boundary**. The largest single-row luminance step found anywhere in the sweep is ~1.1 out of ~62 (1.8%), and the position of that step **moves with terrain content, not with the camera** — i.e. it is a world feature, not a camera-anchored ring.

I am not fixing anything on that basis. A 6% smooth drift does not read as "the camera is a light source", and per this beat's own rule a fix applied before the causal chain is evidenced is the named failure mode. §6 is the STOP.

---

## 1. The instrument

`tools/aw_editor/tests/t2d_camera_light.rs` (new, permanent). Renders the editor's own viewport path offscreen at exactly-specified cameras — the same contract as the T.2a station harness, and necessary for the same reason: the editor cannot pin a camera (T2A_OUTCOME.md §1). Verified to call the **same** `ViewportRenderer::render` the live widget calls at `widget.rs:875`; the `None`/`false` arguments are gizmo/grid/physics overlays, not post-processing.

Three experiments:

- **A — distance sweep.** Focal point held fixed, so the **same world fragment** sits at screen centre in every frame. If its luminance moves, shading is camera-dependent. This is the decisive test.
- **B — lateral translation.** Camera and focal translated together, so camera-relative geometry is identical and only world content changes. A boundary that stays at the same screen row is camera-anchored; one that tracks terrain is world-anchored.
- **C — row profile.** Mean luminance per screen row, plus the largest adjacent-row step. A hard `if`-threshold shading term produces a step; fog and smooth falloffs do not.

### 1.1 A methodological failure worth recording

The first run of this harness reported four clean-looking rows at distances 4/6/8/12 — **byte-identical metrics across four different camera distances**, which no real camera move can produce. The camera was *inside the terrain* and every frame was pure fog.

Cause: `OrbitCamera` places the eye at `focal + distance·dir(yaw,pitch)`, so altitude above the focal point is `distance · sin(pitch)`. At the 35° pitch first chosen, a 4 m orbit is 2.3 m over the surface — well inside local relief (base amplitude 50).

Two changes followed, both permanent: the sweep uses an 80° look-down (altitude = 0.985 × distance), and every row now reports **`sky_frac`** and self-flags `<-- DEGENERATE: camera inside terrain, NOT a measurement` above 0.9. Degenerate frames can no longer be silently reported as data. The 3/5/8 m rows below are left in, flagged, as the honest record of the geometric floor at this focal point.

---

## 2. Phase 1 — characterization

### 2.1 Experiment A — camera dependence is real (the decisive result)

Same world fragment at screen centre in every row; only the camera moved.

| distance (m) | 3 | 5 | 8 | 12 | 16 | 20 | 30 | 45 | 70 | 110 |
|---|---|---|---|---|---|---|---|---|---|---|
| centre luma | *degen* | *degen* | *degen* | **65.58** | 64.78 | 64.13 | 63.93 | 62.82 | 61.65 | **61.72** |

**Monotonic: +5.9% brighter at 12 m than at 110 m, for the same piece of ground.** That is the camera-dependence the director's description implies, confirmed numerically.

**Fog cannot explain it, and this was tested rather than argued** (§3.5): the fog colour is *brighter* than the terrain, so fog would make distant fragments brighter, not darker — the observed sign is opposite.

### 2.2 Experiments B and C — the boundary did NOT reproduce

Largest adjacent-row luminance step, by lateral offset (identical camera geometry, different world content):

| lateral offset (m) | 0 | 15 | 30 | 60 |
|---|---|---|---|---|
| max row step (luma) | 0.89 | 0.63 | 0.76 | 0.68 |
| **at screen row** | **305** | **328** | **378** | **283** |

**Verdict: the largest step is WORLD-anchored, not camera-anchored.** The row wanders with terrain content. A camera-anchored ring would have pinned the same row in all four frames, because the camera-relative geometry is identical across them.

And the magnitude is not a boundary in any case: ~0.6–1.1 luma against a frame mean of ~62, i.e. **under 2%**. Visual inspection of the frames confirms no bright region and no edge; what *is* visible up close is the hex-tile lattice, which is a separate known item and explicitly out of scope here.

**So: brightening — reproduced and camera-anchored. Boundary — not reproduced.**

---

## 3. Phase 2 — hypothesis census, with verdicts

Each A/B leg is a temporary edit to `pbr_terrain_forward.wgsl`, re-run through the same harness, then reverted.

### 3.1 A camera-attached light ("headlight") in the editor's scene setup — **EXONERATED**

By code read, three independent ways:

- `EngineRenderAdapter::set_lighting_params` (`engine_adapter.rs:3798`) writes only ambient colour/intensity and a **sun direction** override. No positional light.
- No point light exists anywhere in the editor's viewport layer: `rg 'point_light|PointLight|light_pos'` over `engine_adapter.rs` returns nothing.
- `ViewportRenderer::set_scene_lights` (`renderer.rs:1477`) is an explicit **no-op** — entity `Light` components are collected in `widget.rs:777-800` and then dropped. Even those are entity-positional, never camera-derived.

Nothing in the editor derives a light's position or direction from the camera.

### 3.2 The LOD-tiered BRDF (`compute_material_lod` → Kulla-Conty multiscatter) — **IMPLICATED for a brightness offset, EXONERATED as the distance trend**

This was the strongest-looking suspect on inspection and it failed its own test, which is exactly why the test was run.

`brdf_common.wgsl:57` selects a shading tier from `length(fwidth(world_pos))` — the world-space pixel footprint, which is camera-distance dependent — and **LOD 0 adds a multiscatter energy term that LOD 1 does not have** (`brdf_common.wgsl:124-140`). Hard `if` thresholds at 0.5 and 2.0 on a continuous quantity. That is a textbook recipe for a camera-anchored ring, and it is called from three places: `renderer.rs:231` (static PBR), `renderer.rs:571` (skinned PBR), and `pbr_terrain_forward.wgsl:369` (terrain) — so it is **engine-wide, not editor-only**.

A/B leg: `let mat_lod = 1u;` (constant).

| distance | 12 | 20 | 45 | 110 | trend 12→110 |
|---|---|---|---|---|---|
| HEAD | 65.58 | 64.13 | 62.82 | 61.72 | **−5.9%** |
| constant LOD | 59.88 | 58.48 | 57.14 | 55.91 | **−6.6%** |

**The trend survives at full strength.** Forcing the LOD constant did not remove the camera dependence; it only shifted the absolute level (LOD 0's multiscatter is worth **+9.5%** at 12 m and **+10.4%** at 110 m).

That the offset is nearly *equal at both ends* is the informative part: the screen-centre fragment is in LOD 0 at **both** 12 m and 110 m, i.e. the sweep never crosses a threshold. The LOD boundary is real but sits far outside the tested range — order 240 m perpendicular at this FOV and resolution, and much nearer at grazing incidence where the footprint is divided by `cos(incidence)`.

**Verdict: convicted of adding a ~10% camera-tier-dependent energy step, acquitted of producing the measured gradient. Its threshold ring remains a live candidate for the director's boundary — see §6.**

### 3.3 Mip-gated / normal-map terms (`NORMAL_XY_STRENGTH`) — **IMPLICATED (partial)**

A/B leg: `let N = N_geom;` — normal mapping removed entirely, which subsumes `NORMAL_XY_STRENGTH` and all mip-dependent normal behaviour.

| distance | 12 | 20 | 45 | 110 | trend |
|---|---|---|---|---|---|
| HEAD | 65.58 | 64.13 | 62.82 | 61.72 | −5.9% |
| geometric normal | 64.01 | 63.69 | 62.95 | 61.64 | **−3.7%** |

The trend **shrinks but survives**, and the near field flattens markedly (HEAD falls monotonically 65.58→63.93 over 12–30 m; with the geometric normal it is essentially flat at 64.01/63.69/63.98). So the normal-map mip chain owns roughly the near-field third of the effect and something else owns the rest.

**Verdict: implicated, partial cause. Not sufficient alone.**

### 3.4 IBL / ambient / specular with a bad view-dependent factor — **NOT EXONERATED, NOT TESTED**

The ambient term (`pbr_terrain_forward.wgsl:379-380`) is `ambient_color · ambient_intensity · 0.35 · AO` with no view term, so it cannot vary with camera position. The specular path is inside `evaluate_brdf_lod`, which §3.2's constant-LOD leg held fixed while the trend persisted — but that leg did not isolate `V` itself. The residual −3.7% after removing normal mapping is unattributed and this is where it most likely lives.

**Verdict: open. This is the strongest remaining lead for the measured gradient**, and the honest reason it is unresolved is that I stopped on the STOP rather than continuing to bisect a ~6% effect that does not match the reported symptom.

### 3.5 Fog / exposure / tonemap — **EXONERATED**

A/B leg: fog term removed.

Result: **numerically identical to HEAD in every row** (65.5821 / 64.7770 / 64.1251 / 63.9280 / 62.8191 / 61.6543 / 61.7212). Fog contributes exactly nothing over 12–110 m.

That identity is also a free control: two independent full runs producing byte-identical numbers establishes the harness is deterministic, which is what makes the other legs' differences trustworthy.

Exposure and tonemap are separately excluded by configuration: `PostProcessChain::default()` (`hdr_pipeline.rs:110-125`) has `auto_exposure_enabled: false`, `bloom_enabled: false`, `ssao_enabled: false`, and **no editor code path enables any of them** — the only `bloom_enabled` writes in the editor are panel-local state in `post_process_panel.rs` / `project_settings_panel.rs` that never reach `set_post_process_chain`.

**Verdict: exonerated.** (SSAO was worth checking because "brighter up close + shadows at borders" fits it well — a screen-space AO radius covers less world space as you approach, and SSAO haloes at depth discontinuities. It is simply not enabled.)

---

## 4. Phase 3 — conviction: NOT REACHED

The causal chain this beat asked for cannot be honestly written, because the two halves of the symptom do not both reproduce:

| element of the director's description | status |
|---|---|
| "as I get really close … it gets brighter" | **reproduced**, +5.9% at fixed world point, camera-anchored |
| "casts shadows at the borders of the light boundary" | **not reproduced** — largest step <2%, and world-anchored, not camera-anchored |
| "like a light source" (a bounded bright region) | **not reproduced** — no bounded region found at any tested station |

Two contributors to the *brightening* are identified and separated (§3.2 ~10% tier offset, §3.3 ~2.2 points of the 5.9% gradient), with a residual that is not yet attributed. None of them produces a boundary.

**Why I am not fixing the identified pieces now:** removing the multiscatter tier step or flattening `NORMAL_XY_STRENGTH` would each change close-up appearance measurably, and would be tuning against a defect I have not shown is the one the director is looking at. If the real cause turns out to be motion- or configuration-dependent (§6), those edits would have moved the materials for nothing — and material appearance is precisely what T.2a and T.2c spent two beats stabilising.

---

## 5. Phase 5 — the re-judgment list (independent of the STOP)

This list stands **regardless** of what the remaining diagnosis finds, because §2.1 already proves close-up shading varies with camera distance. Judgments made at an unrecorded camera distance carry an unrecorded ±6% exposure. **I have re-tuned nothing; the list is the deliverable and the director decides what gets re-judged.**

1. **The T.2a "flat and shiny up close" complaint** — the complaint that started the whole terrain-material line. Made at close range, where luminance runs ~6% hot relative to the far field.
2. **T.2c's three close-up reads** (grassland shards gone, desert relief, tundra structure). The *structural* findings are safe — a black shard is present or absent, and the desert's 4.3× Laplacian gain is a contrast measure, not a brightness one. The **brightness** findings are not: specifically the T.2c §4.3 report that **the tundra now reads too dark**. That was judged at station distance 20, inside the affected range.
3. **The T.2c grassland colour verdict** ("reads olive/brown rather than green") — a tone judgment at close range.
4. **Every T.2a shader-constant A/B** (`NORMAL_XY_STRENGTH` 1.8→1.4, hex pow 4→2). These were measured at *pinned, identical* camera stations per leg, so the **comparisons remain valid** — both legs carried the same camera-dependent offset and it cancels. Their *absolute* appearance verdicts do not.

The pinned-station method is what protects items 4 and most of 2: because T.2a and T.2c registered every leg to identical cameras, their A/B deltas survive this defect even though their absolute reads do not.

---

## 6. Phase 4 — STOP, with what I need and what the options are

The beat directs a STOP when the choice is not obvious. It is not obvious, so:

### 6.1 What would let me finish the diagnosis

1. **A screenshot of the effect**, and the approximate camera distance. One frame settles whether the boundary is a ring, a step, or a gradient, and gives its radius — which discriminates §3.2's LOD threshold (a ring at a fixed world radius, order 240 m perpendicular / much closer at grazing) from everything else.
2. **Does it appear while the camera is MOVING, or does it persist when parked?** This is my leading hypothesis for the non-reproduction: **TAA is enabled by default** (`hdr_pipeline.rs:119`, in the pass list at `:152`) and my harness renders *static* cameras with a two-frame settle, so it cannot exhibit motion-dependent history artifacts. A TAA reprojection fault would brighten during approach and produce edges at disocclusions — which matches "brighter as I get close" and "shadows at the borders" far better than anything I measured. If the effect vanishes when you stop moving, that is close to conclusive.
3. **Which world/biome and roughly where**, so I can re-pin a station on the ground you were actually looking at.
4. **Does it affect props and water, or terrain only?** Phase 1 item 3. I can predict but not yet confirm: `compute_material_lod` is called from the static and skinned PBR shaders too (`renderer.rs:231`, `:571`), so a LOD-threshold cause *should* affect props; a terrain-shader-only cause should not.

### 6.2 Options on the piece that IS convicted, when you want it dealt with

The LOD tier step (§3.2) is a genuine defect independent of the boundary question: shading quality that changes discontinuously with camera distance means **any** close-up judgment sits on a different BRDF than the far field. Options, with costs:

| option | effect | cost |
|---|---|---|
| **A. Force LOD 0 everywhere in the terrain shader** | Removes the tier discontinuity; close-up appearance unchanged, far field gains multiscatter (~+10%) | Loses the ALU saving the tiers were added for; far-field terrain brightens ~10%, which rots T.2a/T.2c far-field frames |
| **B. Smoothly blend the multiscatter term across the threshold** | Removes the visible step while keeping most of the saving | Real shader work; needs its own A/B |
| **C. Leave it, document it as a known camera-dependence** | No churn | Every future close-up judgment keeps the ±10% tier offset |

I recommend deciding this **after** §6.1, not before: if the reported symptom turns out to be TAA, option B is wasted work on the wrong term, and if it turns out to be the LOD ring then the choice is A or B on evidence rather than on my guess.

---

## 7. Verification

| rung | result |
|---|---|
| `cargo fmt -p aw_editor` | clean |
| `cargo check -p aw_editor --tests` | **exit 0** (pre-existing warnings only) |
| `git diff astraweave-render/` | **empty** — all four A/B shader legs reverted |
| T.2d harness, 4 legs | 1 passed / 0 failed each; `head` and `nofog` byte-identical (determinism control) |
| `cargo test -p aw_editor --lib palette_remap` | **8 passed; 0 failed** (4,022 filtered) |
| `cargo test -p aw_editor --lib canonical_terrain_pack` | **2 passed; 0 failed** (4,028 filtered) |

No regression test is added: this beat convicted no defect, and a test asserting the *current* camera-dependent behaviour would pin the bug rather than catch it. The harness itself is the permanent instrument (ED-1 precedent) — it is `#[ignore]`d like the T.2a stations, so it does not slow CI, and it now self-flags degenerate frames.

## 8. Residue

- The **residual unattributed −3.7% gradient** (§3.4) is the open technical thread. `V`-vector normalization and the specular path are where I would look next.
- The **LOD threshold ring** is predicted but never observed; §6.1 item 1 would confirm or kill it in one frame.
- **The geometric floor at the T.2a station-01 focal is ~12 m** — closer cameras are inside local relief. A genuine "really close" sweep needs a focal on flat or raised ground; `sky_frac` now makes the failure self-evident rather than silent.
- Not investigated: whether `camera-relative` (a declared feature of `astraweave-render`) is active in this path and interacts with `world_pos`-keyed terms.
