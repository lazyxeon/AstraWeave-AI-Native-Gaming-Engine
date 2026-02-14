# astraweave-render Hardening Report

**Date**: 2025-01-XX
**Scope**: Pre-mutation-testing hardening of `astraweave-render`
**Test count**: 671 → 682 (+11 new tests)
**Clippy**: 10 errors → 0
**Compilation**: Clean (`cargo check -p astraweave-render`)

---

## Phase 1: Critical Correctness Fixes

### 1A — Texture Streaming State Machine (FIXED)

**Bug**: `request_texture()` inserted `AssetState::Loading` immediately, but `process_next_load()` also set Loading when popping from the queue. If a texture was requested but not yet processed, the premature Loading state prevented the texture from being loaded when `process_next_load()` ran — it saw the key in `assets` and skipped it.

**Fix**:
- Removed premature `assets.insert(id, AssetState::Loading)` from `request_texture()`
- Added `pending_ids: HashSet<AssetId>` field for O(1) dedup of queue entries
- `request_texture()` checks `pending_ids` + `assets` before enqueueing
- `process_next_load()` is now the sole owner of the Loading→Resident lifecycle
- `clear()` resets `pending_ids`

**Files changed**: `texture_streaming.rs`
**Tests added (9)**:
1. `request_queues_without_premature_loading_state`
2. `duplicate_request_does_not_double_queue`
3. `failed_texture_is_not_requeued`
4. `loading_texture_blocks_requeue_but_not_via_assets_map`
5. `stats_counts_pending_correctly`
6. `clear_resets_all_state`
7. `resident_texture_is_returned_and_lru_updated`
8. `evict_lru_removes_oldest`
9. `stats_counts_loaded_with_resident`

### 1B — Shadow Override (FIXED)

**Bug**: WGSL shader had hardcoded `shadow = 1.0;` that bypassed all shadow computation regardless of `force_shadow_override` flag.

**Fix**:
- Added `pub force_shadow_override: bool` to Renderer (defaults `false`)
- CPU: When flag is true, writes `-1.0` as `extras.x` in light UBO (sentinel)
- WGSL: Both main and skinned shaders now check `if (uLight.extras.x < 0.0) { shadow = 1.0; }` — shadows only bypassed when sentinel is set
- Sentinel applied at all 3 light buffer write locations

**Files changed**: `renderer.rs`
**Tests added (2)**:
1. `test_shadow_override_sentinel_logic` — verifies sentinel bytes in light UBO
2. `test_shader_has_conditional_shadow_not_hardcoded` — verifies WGSL source contains conditional, not hardcoded override

---

## Phase 2: Render Path Unification

### 2A — Divergence Audit

Full comparison of `render()` (standalone viewport) vs `draw_into()` (composable offscreen). 12 operations compared across both paths.

### 2B — Alignment Fixes (8 applied)

| # | Operation | Path Fixed | Change |
|---|-----------|-----------|--------|
| 1 | Shadow pass: sphere instances + ext mesh | `draw_into()` | Was plane-only; now draws sphere instances and external mesh |
| 2 | Main pass: external mesh instance count | `render()` | Used `0..1` hardcoded; now uses `self.ext_inst_count` |
| 3 | Main pass: named models | `render()` | Was missing entirely; now renders `self.models.values()` |
| 4 | Shadow pass: ext mesh instance count | `render()` | Used `0..1` hardcoded; now uses `self.ext_inst_count` |
| 5 | Main pass: water rendering | `draw_into()` | Was missing; now renders water after opaque geometry |
| 6 | Sky rendering: VP matrix + IBL | `render()` | Used full VP + `None,None`; now uses rotation-only VP + IBL resources |
| 7 | Main pass: depth load op | `render()` | Was `Clear(1.0)` (overwrote sky depth); now `Load` (matches draw_into) |
| 8 | Scene env UBO upload | `draw_into()` | Was missing; now uploads before main pass |

**Files changed**: `renderer.rs`

---

## Phase 3: Cleanup & Polish

### 3A — Surface Error Handling (NORMALIZED)

**Issue**: `render_with()` and `render_with_simple()` used raw `surface.get_current_texture()?` without Lost/OutOfMemory handling, while `render()` had full match-based error handling.

**Fix**: Extracted `acquire_surface_texture()` helper method returning `Result<Option<(SurfaceTexture, TextureView)>>`:
- Returns `Ok(None)` when no surface configured
- Returns `Ok(None)` on `SurfaceError::Lost` (after reconfiguring)
- Returns `Err(...)` on `SurfaceError::OutOfMemory`
- All three render methods now use this shared helper

**Files changed**: `renderer.rs`

### 3B — Debug Artifact Cleanup (4 items)

| Artifact | File | Classification | Action |
|---------|------|---------------|--------|
| `// DEBUG: Binary search - Test 3` | `renderer.rs` (draw_into) | Dead comment | Removed |
| `println!("🔍 Light frustum...")` | `shadow_csm.rs` | Dev debug | Converted to `log::debug!()` |
| `println!("🔍 Shadow rendering...")` (5 lines) | `shadow_csm.rs` | Dev debug | Converted to `log::debug!()` |
| Magenta clear color (1,0,1) | `environment.rs` | Diagnostic placeholder | Conditional: magenta in debug, dark grey in release |

**Files changed**: `renderer.rs`, `shadow_csm.rs`, `environment.rs`

### 3C — Clippy Resolution (10 → 0)

| # | Lint | File | Fix |
|---|------|------|-----|
| 1 | `unused_variable` | `renderer.rs:584` | Prefixed `f` → `_f` |
| 2-3 | `manual_div_ceil` ×2 | `nanite_gpu_culling.rs:911-912` | `(x+7)/8` → `x.div_ceil(8)` |
| 4 | `manual_div_ceil` | `nanite_gpu_culling.rs:935` | `(x+63)/64` → `x.div_ceil(64)` |
| 5-6 | `manual_div_ceil` ×2 | `nanite_gpu_culling.rs:953-954` | `(x+7)/8` → `x.div_ceil(8)` |
| 7 | `too_many_arguments` | `nanite_render.rs:36` | `#[allow(clippy::too_many_arguments)]` (constructor) |
| 8 | `unnecessary_cast` | `nanite_render.rs:245` | Removed `as u32` (already u32) |
| 9 | `let_and_return` | `nanite_visibility.rs:159` | Returned expression directly |
| 10 | `needless_range_loop` | `ssao.rs:101` | Converted to `iter_mut().enumerate()` |

**Files changed**: `renderer.rs`, `nanite_gpu_culling.rs`, `nanite_render.rs`, `nanite_visibility.rs`, `ssao.rs`

### 3D — Source Artifact Cleanup

Moved 3 stale files to `src/_archive/` (not referenced by build or code):
- `renderer.rs.baseline`
- `renderer.rs.broken.keep`
- `residency.rs.head`

---

## Summary

| Metric | Before | After |
|--------|--------|-------|
| Tests | 671 | 682 (+11) |
| Clippy errors | 10 | 0 |
| Debug println! calls | 5 | 0 (converted to log::debug) |
| Surface error handling paths | 1/3 correct | 3/3 correct |
| Render path divergences | 8 operations | 0 (unified) |
| Stale source artifacts | 3 files | 0 (archived) |

## Remaining Gaps

- **No GPU integration tests**: All 11 new tests run CPU-side or with headless wgpu device. True GPU render output validation would require screenshot comparison infrastructure.
- **Shadow CSM one-shot debug logging**: Still fires once per process via `AtomicBool` — acceptable for diagnostics but could be gated behind a feature flag for zero-cost in release.
- **render() commented-out plane buffer**: ~10-line block at render() is disabled via `/* */` — retained intentionally per comment "DISABLE to fix interference with TerrainSystem".

## Mutation Testing Readiness

**READY**. All critical correctness issues fixed, render paths unified, clippy clean, tests covering key invariants. The crate is in a stable state for mutation testing.
