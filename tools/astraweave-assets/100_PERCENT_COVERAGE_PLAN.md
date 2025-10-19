# 🎯 100% Test Coverage Plan - "Shooting for the Stars"

**Date**: October 17, 2025  
**Current Coverage**: 50.8% (focused) / 25.3% (raw)  
**Target Coverage**: **100%**  
**Estimated Duration**: 4-6 hours  
**Phases**: 7 phases (Discovery → Execution → Validation)

---

## Executive Summary

Comprehensive plan to achieve **100% test coverage** for the astraweave-assets multi-source asset pipeline. Systematic approach tackling modules from highest-impact (public API) to lowest-impact (internal utilities).

**Philosophy**: "Zero untested lines" - Every function, every branch, every error path tested.

**Success Criteria**: 
- ✅ 100% line coverage (all 906 library lines)
- ✅ 100% branch coverage (all if/else paths)
- ✅ 100% error path coverage (all Result::Err cases)
- ✅ Property-based tests for data validation
- ✅ Mock-based tests for external dependencies (HTTP, file I/O)

---

## Phase 1: Discovery & Analysis (15 minutes)

### Current State Analysis

| Module | Current | Target | Gap | Priority |
|--------|---------|--------|-----|----------|
| **lib.rs** | 0% | 100% | +100% | 🔥 CRITICAL |
| **polyhaven.rs** | 1.9% | 100% | +98.1% | 🔥 CRITICAL |
| **organize.rs** | 6.5% | 100% | +93.5% | 🔥 CRITICAL |
| **provider.rs** | 18.6% | 100% | +81.4% | ⚠️ HIGH |
| **summary.rs** | 21.4% | 100% | +78.6% | ⚠️ HIGH |
| **downloader.rs** | 29.6% | 100% | +70.4% | ⚠️ HIGH |
| **unified_config.rs** | 51.4% | 100% | +48.6% | 📋 MEDIUM |
| **direct_url_provider.rs** | 51.0% | 100% | +49.0% | 📋 MEDIUM |
| **config.rs** | 65.6% | 100% | +34.4% | 📋 MEDIUM |
| **polyhaven_provider.rs** | 68.8% | 100% | +31.2% | 📋 MEDIUM |
| **kenney_provider.rs** | 89.1% | 100% | +10.9% | ✅ LOW |

**Total Gap**: **74.7 percentage points** (from 25.3% to 100%)

### Key Challenges

1. **HTTP Mocking** (polyhaven.rs): Need mockito for API calls
2. **File I/O Mocking** (organize.rs): Need tempfile for file operations
3. **Download Simulation** (downloader.rs): Complex retry logic, progress bars
4. **Public API Testing** (lib.rs): Integration-style tests with real providers
5. **Error Path Coverage**: Many Result<T, E> branches untested

### Dependencies Required

```toml
[dev-dependencies]
mockito = "1.5"      # HTTP mocking
tempfile = "3.13"    # Temporary directories for file I/O tests
proptest = "1.5"     # Property-based testing
```

---

## Phase 2: Public API (lib.rs) - 0% → 100% (45 minutes)

**Priority**: 🔥 **CRITICAL** (User-facing functions)

### Functions to Test

1. `ensure_asset()` - Download asset if not present
2. `is_available()` - Check asset existence
3. `resolve()` - Resolve asset URL
4. `download_all()` - Bulk download (if exists)
5. Error handling for all functions

### Test Strategy

**Approach**: Integration-style tests with real providers + mocked file system

**Test File**: `tests/lib_api_tests.rs`

**Test Cases** (20 tests):

#### `ensure_asset()` Tests (8 tests)
1. ✅ `test_ensure_asset_downloads_missing_texture`
2. ✅ `test_ensure_asset_skips_existing_texture`
3. ✅ `test_ensure_asset_creates_output_directory`
4. ✅ `test_ensure_asset_invalid_handle_errors`
5. ✅ `test_ensure_asset_network_failure_errors`
6. ✅ `test_ensure_asset_disk_full_errors`
7. ✅ `test_ensure_asset_with_different_providers`
8. ✅ `test_ensure_asset_concurrent_calls_idempotent`

#### `is_available()` Tests (6 tests)
9. ✅ `test_is_available_returns_true_for_existing`
10. ✅ `test_is_available_returns_false_for_missing`
11. ✅ `test_is_available_checks_all_files_in_handle`
12. ✅ `test_is_available_invalid_handle_returns_false`
13. ✅ `test_is_available_partial_download_returns_false`
14. ✅ `test_is_available_with_symlinks`

#### `resolve()` Tests (4 tests)
15. ✅ `test_resolve_returns_url_for_valid_handle`
16. ✅ `test_resolve_errors_on_invalid_handle`
17. ✅ `test_resolve_with_different_providers`
18. ✅ `test_resolve_caches_results`

#### Error Handling (2 tests)
19. ✅ `test_error_messages_are_user_friendly`
20. ✅ `test_all_error_types_covered`

**Coverage Target**: **100%** (45 lines, all branches)

**Estimated Time**: 45 minutes

---

## Phase 3: API Client (polyhaven.rs) - 1.9% → 100% (60 minutes)

**Priority**: 🔥 **CRITICAL** (External dependency)

### Functions to Test

1. `fetch_asset_info()` - Fetch asset metadata from API
2. `resolve_hdri()` - Resolve HDRI download URL
3. `resolve_texture()` - Resolve texture download URL
4. `resolve_model()` - Resolve 3D model download URL
5. Error handling (404, network failures, JSON parsing)

### Test Strategy

**Approach**: Mock HTTP responses with mockito

**Test File**: `tests/polyhaven_api_tests.rs`

**Test Cases** (25 tests):

#### `fetch_asset_info()` Tests (8 tests)
1. ✅ `test_fetch_asset_info_success`
2. ✅ `test_fetch_asset_info_404_errors`
3. ✅ `test_fetch_asset_info_invalid_json_errors`
4. ✅ `test_fetch_asset_info_network_timeout_errors`
5. ✅ `test_fetch_asset_info_retries_on_5xx`
6. ✅ `test_fetch_asset_info_caches_responses`
7. ✅ `test_fetch_asset_info_rate_limiting_respects`
8. ✅ `test_fetch_asset_info_malformed_response`

#### `resolve_hdri()` Tests (6 tests)
9. ✅ `test_resolve_hdri_1k_resolution`
10. ✅ `test_resolve_hdri_4k_resolution`
11. ✅ `test_resolve_hdri_8k_resolution`
12. ✅ `test_resolve_hdri_missing_resolution_defaults`
13. ✅ `test_resolve_hdri_invalid_asset_errors`
14. ✅ `test_resolve_hdri_url_format_correct`

#### `resolve_texture()` Tests (6 tests)
15. ✅ `test_resolve_texture_all_maps_present`
16. ✅ `test_resolve_texture_missing_normal_map`
17. ✅ `test_resolve_texture_resolution_fallback`
18. ✅ `test_resolve_texture_format_preference`
19. ✅ `test_resolve_texture_invalid_type_errors`
20. ✅ `test_resolve_texture_url_construction`

#### `resolve_model()` Tests (3 tests)
21. ✅ `test_resolve_model_gltf_format`
22. ✅ `test_resolve_model_fbx_format`
23. ✅ `test_resolve_model_missing_format_errors`

#### Error Handling (2 tests)
24. ✅ `test_all_http_errors_handled`
25. ✅ `test_connection_failures_graceful`

**Coverage Target**: **100%** (160 lines, all branches)

**Estimated Time**: 60 minutes

---

## Phase 4: File Organization (organize.rs) - 6.5% → 100% (45 minutes)

**Priority**: 🔥 **CRITICAL** (Data integrity)

### Functions to Test

1. `organize_assets()` - Move downloaded files to structured directories
2. `generate_attribution()` - Create attribution text files
3. `update_lockfile()` - Update lockfile with new assets
4. Error handling (permission denied, disk full, invalid paths)

### Test Strategy

**Approach**: Use tempfile for isolated file system tests

**Test File**: `tests/organize_tests.rs`

**Test Cases** (20 tests):

#### `organize_assets()` Tests (10 tests)
1. ✅ `test_organize_creates_directory_structure`
2. ✅ `test_organize_moves_files_correctly`
3. ✅ `test_organize_handles_duplicate_names`
4. ✅ `test_organize_preserves_file_extensions`
5. ✅ `test_organize_handles_subdirectories`
6. ✅ `test_organize_skips_already_organized`
7. ✅ `test_organize_errors_on_permission_denied`
8. ✅ `test_organize_errors_on_disk_full`
9. ✅ `test_organize_cleans_up_on_failure`
10. ✅ `test_organize_with_symlinks`

#### `generate_attribution()` Tests (6 tests)
11. ✅ `test_generate_attribution_cc0_format`
12. ✅ `test_generate_attribution_cc_by_format`
13. ✅ `test_generate_attribution_mit_format`
14. ✅ `test_generate_attribution_multiple_assets`
15. ✅ `test_generate_attribution_file_created`
16. ✅ `test_generate_attribution_append_mode`

#### `update_lockfile()` Tests (4 tests)
17. ✅ `test_update_lockfile_adds_new_entries`
18. ✅ `test_update_lockfile_updates_existing_entries`
19. ✅ `test_update_lockfile_preserves_other_entries`
20. ✅ `test_update_lockfile_atomic_write`

**Coverage Target**: **100%** (138 lines, all branches)

**Estimated Time**: 45 minutes

---

## Phase 5: Download Logic (downloader.rs) - 29.6% → 100% (60 minutes)

**Priority**: ⚠️ **HIGH** (Core functionality)

### Functions to Test

1. `download_with_progress()` - Download with progress bar
2. `download_with_retry()` - Download with exponential backoff
3. `verify_hash()` - SHA256 verification
4. `parallel_download()` - Concurrent download manager
5. Error handling (network failures, hash mismatches, timeouts)

### Test Strategy

**Approach**: Mock HTTP server with mockito + progress bar testing

**Test File**: `tests/downloader_tests.rs`

**Test Cases** (25 tests):

#### `download_with_progress()` Tests (8 tests)
1. ✅ `test_download_with_progress_success`
2. ✅ `test_download_with_progress_updates_callback`
3. ✅ `test_download_with_progress_large_file`
4. ✅ `test_download_with_progress_resume_partial`
5. ✅ `test_download_with_progress_errors_on_404`
6. ✅ `test_download_with_progress_timeout`
7. ✅ `test_download_with_progress_cancellation`
8. ✅ `test_download_with_progress_concurrent_safe`

#### `download_with_retry()` Tests (8 tests)
9. ✅ `test_retry_succeeds_on_first_attempt`
10. ✅ `test_retry_succeeds_on_second_attempt`
11. ✅ `test_retry_succeeds_on_third_attempt`
12. ✅ `test_retry_exhausts_after_max_attempts`
13. ✅ `test_retry_exponential_backoff_timing`
14. ✅ `test_retry_jitter_randomization`
15. ✅ `test_retry_different_error_types`
16. ✅ `test_retry_respects_timeout`

#### `verify_hash()` Tests (4 tests)
17. ✅ `test_verify_hash_success`
18. ✅ `test_verify_hash_mismatch_errors`
19. ✅ `test_verify_hash_missing_file_errors`
20. ✅ `test_verify_hash_large_files`

#### `parallel_download()` Tests (5 tests)
21. ✅ `test_parallel_download_respects_concurrency_limit`
22. ✅ `test_parallel_download_all_succeed`
23. ✅ `test_parallel_download_partial_failures`
24. ✅ `test_parallel_download_progress_aggregation`
25. ✅ `test_parallel_download_cancels_on_ctrl_c`

**Coverage Target**: **100%** (186 lines, all branches)

**Estimated Time**: 60 minutes

---

## Phase 6: Provider Infrastructure (provider.rs, summary.rs) - ~20% → 100% (45 minutes)

**Priority**: ⚠️ **HIGH** (Provider abstraction)

### Functions to Test (provider.rs)

1. `resolve()` - Abstract provider resolution
2. `list()` - List available assets
3. `validate_license()` - License compatibility checking
4. `format_attribution()` - Attribution text generation

### Test Strategy

**Approach**: Unit tests with mock providers

**Test File**: `tests/provider_tests.rs`

**Test Cases** (18 tests):

#### `resolve()` Tests (6 tests)
1. ✅ `test_resolve_dispatches_to_correct_provider`
2. ✅ `test_resolve_caches_results`
3. ✅ `test_resolve_errors_on_unknown_provider`
4. ✅ `test_resolve_handles_provider_errors`
5. ✅ `test_resolve_with_custom_options`
6. ✅ `test_resolve_thread_safe`

#### `list()` Tests (4 tests)
7. ✅ `test_list_returns_all_handles`
8. ✅ `test_list_filters_by_type`
9. ✅ `test_list_pagination`
10. ✅ `test_list_empty_provider`

#### `validate_license()` Tests (5 tests)
11. ✅ `test_validate_license_cc0_accepts`
12. ✅ `test_validate_license_gpl_rejects`
13. ✅ `test_validate_license_cc_by_requires_author`
14. ✅ `test_validate_license_mit_accepts`
15. ✅ `test_validate_license_unknown_errors`

#### `format_attribution()` Tests (3 tests)
16. ✅ `test_format_attribution_single_asset`
17. ✅ `test_format_attribution_multiple_assets`
18. ✅ `test_format_attribution_grouped_by_license`

### Functions to Test (summary.rs)

1. `generate_summary()` - Generate download summary
2. `format_bytes()` - Human-readable byte formatting
3. `format_duration()` - Human-readable duration formatting

**Test File**: `tests/summary_tests.rs`

**Test Cases** (12 tests):

#### `generate_summary()` Tests (6 tests)
1. ✅ `test_generate_summary_single_file`
2. ✅ `test_generate_summary_multiple_files`
3. ✅ `test_generate_summary_with_failures`
4. ✅ `test_generate_summary_calculates_totals`
5. ✅ `test_generate_summary_formats_markdown`
6. ✅ `test_generate_summary_formats_json`

#### `format_bytes()` Tests (3 tests)
7. ✅ `test_format_bytes_kilobytes`
8. ✅ `test_format_bytes_megabytes`
9. ✅ `test_format_bytes_gigabytes`

#### `format_duration()` Tests (3 tests)
10. ✅ `test_format_duration_seconds`
11. ✅ `test_format_duration_minutes`
12. ✅ `test_format_duration_hours`

**Coverage Target**: **100%** (188 lines combined, all branches)

**Estimated Time**: 45 minutes

---

## Phase 7: Configuration & Utilities (~50-70% → 100%) (30 minutes)

**Priority**: 📋 **MEDIUM** (Already partially covered)

### Modules to Complete

1. **unified_config.rs** (51.4% → 100%)
   - Test all provider list combinations
   - Test validation edge cases
   - 8 additional tests

2. **direct_url_provider.rs** (51.0% → 100%)
   - Test URL parsing edge cases
   - Test file extension inference
   - 10 additional tests

3. **config.rs** (65.6% → 100%)
   - Test all manifest parsing edge cases
   - Test lockfile corruption handling
   - 6 additional tests

4. **polyhaven_provider.rs** (68.8% → 100%)
   - Test all asset type resolution
   - Test URL construction edge cases
   - 4 additional tests

5. **kenney_provider.rs** (89.1% → 100%)
   - Complete remaining branches
   - 2 additional tests

**Total Additional Tests**: 30 tests

**Coverage Target**: **100%** (remaining ~200 lines)

**Estimated Time**: 30 minutes

---

## Phase 8: Property-Based Testing (45 minutes) [BONUS]

**Priority**: 🎁 **BONUS** (Extra robustness)

### Strategy

Use `proptest` to generate random inputs and test invariants

**Test File**: `tests/property_tests.rs`

**Property Tests** (10 tests):

1. ✅ `prop_download_url_always_valid_http`
2. ✅ `prop_file_paths_never_escape_output_dir`
3. ✅ `prop_attribution_text_always_valid_utf8`
4. ✅ `prop_lockfile_roundtrip_preserves_data`
5. ✅ `prop_parallel_downloads_deterministic_order`
6. ✅ `prop_hash_verification_never_false_positive`
7. ✅ `prop_license_validation_consistent`
8. ✅ `prop_provider_resolution_idempotent`
9. ✅ `prop_error_messages_never_contain_secrets`
10. ✅ `prop_file_organization_preserves_data`

**Coverage Impact**: Catches edge cases, increases branch coverage

**Estimated Time**: 45 minutes

---

## Phase 9: Validation & Polish (30 minutes)

### Final Coverage Check

```powershell
cargo tarpaulin --out Html --out Xml --output-dir coverage --exclude-files 'src/main.rs' --exclude-files 'tests/*'
```

**Target**: **100.0% coverage, 906/906 lines covered**

### Cleanup Tasks

1. ✅ Remove any `#[allow(dead_code)]` that's now covered
2. ✅ Add doc comments to all test functions
3. ✅ Organize tests into logical modules
4. ✅ Create test utilities module for shared fixtures
5. ✅ Update PHASE_6_COVERAGE_COMPLETE.md with final results

### Documentation Updates

**Files to Update**:
1. ✅ README.md - Update test count (50 → 150+)
2. ✅ ENHANCEMENT_PLAN.md - Mark Phase 6 as "100% coverage achieved"
3. ✅ Create 100_PERCENT_COVERAGE_ACHIEVED.md - Final report

**Estimated Time**: 30 minutes

---

## Timeline Summary

| Phase | Module(s) | Current | Target | Tests | Duration |
|-------|-----------|---------|--------|-------|----------|
| **1** | Discovery | - | - | 0 | 15 min |
| **2** | lib.rs | 0% | 100% | 20 | 45 min |
| **3** | polyhaven.rs | 1.9% | 100% | 25 | 60 min |
| **4** | organize.rs | 6.5% | 100% | 20 | 45 min |
| **5** | downloader.rs | 29.6% | 100% | 25 | 60 min |
| **6** | provider.rs, summary.rs | ~20% | 100% | 30 | 45 min |
| **7** | Config utilities | ~60% | 100% | 30 | 30 min |
| **8** | Property tests (BONUS) | - | - | 10 | 45 min |
| **9** | Validation & polish | - | - | 0 | 30 min |
| **TOTAL** | **All modules** | **25.3%** | **100%** | **~160** | **5h 55m** |

**Optimistic**: 4 hours (skip property tests, efficient implementation)  
**Realistic**: 5-6 hours (complete all phases)  
**Conservative**: 7-8 hours (with debugging and iterations)

---

## Success Metrics

### Coverage Metrics

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| **Line Coverage** | 25.3% | 100% | 🎯 |
| **Branch Coverage** | ~30% | 100% | 🎯 |
| **Function Coverage** | ~40% | 100% | 🎯 |
| **Test Count** | 50 | 150+ | 🎯 |
| **Test Runtime** | 14s | <30s | 🎯 |

### Quality Metrics

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| **Public API Coverage** | 0% | 100% | 🔥 |
| **Error Path Coverage** | ~20% | 100% | 🔥 |
| **Integration Tests** | 9 | 20+ | 🔥 |
| **Property Tests** | 0 | 10+ | 🎁 |
| **Mock-Based Tests** | 0 | 30+ | 🔥 |

---

## Risk Assessment

### Low Risk ✅

- **kenney_provider.rs** (89.1% → 100%): Only 2-3 tests needed
- **config.rs** (65.6% → 100%): Well-understood code paths
- **Integration tests**: Already have 9 working tests to build on

### Medium Risk ⚠️

- **downloader.rs** (29.6% → 100%): Complex retry logic, progress bars
- **organize.rs** (6.5% → 100%): File I/O mocking can be tricky
- **summary.rs** (21.4% → 100%): Formatting logic edge cases

### High Risk 🔥

- **lib.rs** (0% → 100%): Integration-style tests, need full provider setup
- **polyhaven.rs** (1.9% → 100%): HTTP mocking can be brittle
- **provider.rs** (18.6% → 100%): Abstract trait testing requires careful mocking

### Mitigation Strategies

1. **Start with High-Risk Modules**: Tackle lib.rs and polyhaven.rs first (most learning)
2. **Use Proven Libraries**: mockito (100K+ downloads), tempfile (50M+ downloads)
3. **Incremental Validation**: Run coverage after each phase
4. **Test Utilities**: Create shared fixtures to reduce boilerplate
5. **CI Integration**: Ensure all tests pass in CI before declaring victory

---

## Dependencies to Add

```toml
# Add to Cargo.toml [dev-dependencies]
mockito = "1.5"          # HTTP mocking for polyhaven.rs tests
tempfile = "3.13"        # Temporary directories for organize.rs tests
proptest = "1.5"         # Property-based testing (Phase 8 bonus)
wiremock = "0.6"         # Alternative HTTP mocking (if mockito insufficient)
assert_fs = "1.1"        # File system assertions for organize.rs
predicates = "3.1"       # Better assertion helpers
```

**Installation Command**:
```powershell
cd tools/astraweave-assets
cargo add --dev mockito tempfile proptest wiremock assert_fs predicates
```

---

## Execution Strategy

### Parallel Work Streams (if multiple developers)

**Stream A** (Critical Path): lib.rs → polyhaven.rs → organize.rs  
**Stream B** (Core Logic): downloader.rs → provider.rs → summary.rs  
**Stream C** (Polish): Config utilities → Property tests → Documentation

### Sequential Strategy (single developer)

**Week 1 Day 1** (2 hours): Phase 1-2 (Discovery + lib.rs)  
**Week 1 Day 2** (2 hours): Phase 3 (polyhaven.rs)  
**Week 1 Day 3** (2 hours): Phase 4-5 (organize.rs + downloader.rs)  
**Week 1 Day 4** (1.5 hours): Phase 6-7 (provider.rs, summary.rs, utilities)  
**Week 1 Day 5** (1 hour): Phase 8-9 (Property tests + validation)

**Total**: 8.5 hours over 5 days (or 1 focused Saturday)

---

## Quality Gates

Before declaring 100% coverage achieved, verify:

### Gate 1: Coverage Metrics ✅
- [ ] `cargo tarpaulin` reports **100.0%** line coverage
- [ ] All 906 library lines covered
- [ ] HTML report shows no red (uncovered) lines
- [ ] XML report validates in Codecov

### Gate 2: Test Quality ✅
- [ ] All tests pass (`cargo test` = 100% pass rate)
- [ ] No flaky tests (run 10 times, all pass)
- [ ] No ignored tests (no `#[ignore]` attributes)
- [ ] All error paths tested (no unreachable Result::Err)

### Gate 3: Performance ✅
- [ ] Test suite runs in <30 seconds
- [ ] Integration tests run in <20 seconds
- [ ] Unit tests run in <10 seconds
- [ ] CI pipeline completes in <5 minutes

### Gate 4: Documentation ✅
- [ ] All test functions have doc comments
- [ ] README updated with new test count
- [ ] ENHANCEMENT_PLAN.md marked complete
- [ ] 100_PERCENT_COVERAGE_ACHIEVED.md created

### Gate 5: CI Integration ✅
- [ ] GitHub Actions workflow passes
- [ ] Codecov badge shows 100%
- [ ] No failing tests in CI
- [ ] Coverage report artifact uploaded

---

## Celebration Checklist 🎉

When 100% coverage is achieved:

- [ ] Update all badges in README.md
- [ ] Post completion report (100_PERCENT_COVERAGE_ACHIEVED.md)
- [ ] Update project status to "Gold Standard Quality"
- [ ] Add "100% Test Coverage" badge to README
- [ ] Share achievement in project updates
- [ ] Take a well-deserved break! ☕

---

## Next Steps After 100%

1. **Phase 7**: Benchmark Suite (validate 5× speedup claim)
2. **Phase 8**: CLI Improvements (-j flag, better errors, ETA)
3. **Phase 9**: Documentation improvements (tutorials, examples)
4. **Phase 10**: Performance optimization (if benchmarks reveal issues)

---

## Conclusion

Achieving **100% test coverage** is ambitious but achievable with systematic execution. This plan provides a clear roadmap from **25.3% → 100%** in **5-6 hours**.

**Philosophy**: "Zero untested lines" - Every function, every branch, every error path tested.

**Let's shoot for the stars!** 🚀✨

---

**Status**: 📋 **PLANNING COMPLETE** - Ready to begin Phase 1  
**Next Action**: Install dev dependencies and start Phase 2 (lib.rs tests)
