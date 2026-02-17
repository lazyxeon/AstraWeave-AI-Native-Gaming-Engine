//! Mutation-resistant comprehensive tests for astraweave-sdk.
//! Targets exact return values, C ABI layout, error codes, FFI boundaries,
//! and entity spawn params for 90%+ mutation kill rate.

use astraweave_sdk::*;
use std::ffi::CStr;
use std::mem;

/// Create a null AWWorld handle for testing null-safety paths.
/// SAFETY: AWWorld is #[repr(C)] wrapping a raw pointer; zeroed = null pointer.
fn null_world() -> AWWorld {
    unsafe { mem::zeroed::<AWWorld>() }
}

// ========================================================================
// ERROR CODE CONSTANTS — exact values
// ========================================================================

#[test]
fn aw_ok_is_zero() {
    assert_eq!(AW_OK, 0);
}

#[test]
fn aw_err_null_is_minus_one() {
    assert_eq!(AW_ERR_NULL, -1);
}

#[test]
fn aw_err_param_is_minus_two() {
    assert_eq!(AW_ERR_PARAM, -2);
}

#[test]
fn aw_err_parse_is_minus_three() {
    assert_eq!(AW_ERR_PARSE, -3);
}

#[test]
fn aw_err_exec_is_minus_four() {
    assert_eq!(AW_ERR_EXEC, -4);
}

#[test]
fn error_codes_all_distinct() {
    let codes = [AW_OK, AW_ERR_NULL, AW_ERR_PARAM, AW_ERR_PARSE, AW_ERR_EXEC];
    for i in 0..codes.len() {
        for j in (i + 1)..codes.len() {
            assert_ne!(codes[i], codes[j], "codes[{}] == codes[{}]", i, j);
        }
    }
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn error_codes_signs() {
    assert!(AW_OK >= 0, "AW_OK should be non-negative");
    assert!(AW_ERR_NULL < 0);
    assert!(AW_ERR_PARAM < 0);
    assert!(AW_ERR_PARSE < 0);
    assert!(AW_ERR_EXEC < 0);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn error_codes_ordering() {
    // Negative codes decrease: -1 > -2 > -3 > -4
    assert!(AW_ERR_NULL > AW_ERR_PARAM);
    assert!(AW_ERR_PARAM > AW_ERR_PARSE);
    assert!(AW_ERR_PARSE > AW_ERR_EXEC);
}

// ========================================================================
// AW_VERSION — compile-time version parsing
// ========================================================================

#[test]
fn aw_version_major_is_zero() {
    let v = aw_version();
    assert_eq!(v.major, 0);
}

#[test]
fn aw_version_minor_is_four() {
    let v = aw_version();
    assert_eq!(v.minor, 4);
}

#[test]
fn aw_version_patch_is_zero() {
    let v = aw_version();
    assert_eq!(v.patch, 0);
}

#[test]
fn aw_version_fields_not_swapped() {
    let v = aw_version();
    // major < minor for 0.4.0
    assert!(
        v.major < v.minor,
        "major={} should be < minor={}",
        v.major,
        v.minor
    );
}

// ========================================================================
// AWVERSION — repr(C) layout
// ========================================================================

#[test]
fn awversion_size_is_six_bytes() {
    assert_eq!(mem::size_of::<AWVersion>(), 6);
}

#[test]
fn awversion_align_is_two() {
    assert_eq!(mem::align_of::<AWVersion>(), 2);
}

#[test]
fn awversion_is_copy() {
    let v = aw_version();
    let v2 = v; // Copy
    let _ = v; // Still usable after copy
    assert_eq!(v2.major, 0);
}

#[test]
fn awversion_clone() {
    let v = aw_version();
    #[allow(clippy::clone_on_copy)]
    let v2 = v.clone();
    assert_eq!(v2.major, v.major);
    assert_eq!(v2.minor, v.minor);
    assert_eq!(v2.patch, v.patch);
}

#[test]
fn awversion_debug() {
    let v = aw_version();
    let dbg = format!("{:?}", v);
    assert!(dbg.contains("AWVersion"), "debug should contain type name");
}

// ========================================================================
// AW_VERSION_STRING — NUL-terminated C string
// ========================================================================

#[test]
fn aw_version_string_null_buf_returns_required_size() {
    let required = unsafe { aw_version_string(std::ptr::null_mut(), 0) };
    // "0.4.0" = 5 bytes + NUL = 6
    assert!(required > 0);
    assert!(required >= 6, "should need at least 6 bytes for '0.4.0\\0'");
}

#[test]
fn aw_version_string_zero_len_returns_required() {
    let mut buf = [0u8; 64];
    let required = unsafe { aw_version_string(buf.as_mut_ptr(), 0) };
    assert!(required >= 6);
}

#[test]
fn aw_version_string_full_copy() {
    let mut buf = [0u8; 64];
    let n = unsafe { aw_version_string(buf.as_mut_ptr(), buf.len()) };
    assert!(n >= 6);
    // Verify NUL termination
    let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const i8) };
    let s = cstr.to_str().unwrap();
    assert!(
        s.starts_with("0.4"),
        "version string should start with 0.4, got: {}",
        s
    );
}

#[test]
fn aw_version_string_truncation() {
    // Only 3 bytes: should get "0." + NUL (truncated)
    let mut buf = [0xFFu8; 3];
    let required = unsafe { aw_version_string(buf.as_mut_ptr(), 3) };
    // Should still report full required size
    assert!(required >= 6);
    // Buffer should have NUL at position 2
    assert_eq!(buf[2], 0, "last byte must be NUL");
}

// ========================================================================
// AW_WORLD_CREATE / DESTROY
// ========================================================================

#[test]
fn aw_world_create_returns_non_null() {
    let w = aw_world_create();
    // AWWorld wraps a raw pointer; not null means valid
    // We can verify by getting a snapshot
    let mut buf = [0u8; 4096];
    let n = aw_world_snapshot_json(w, buf.as_mut_ptr(), buf.len());
    assert!(n > 0, "snapshot should return non-zero for valid world");
    aw_world_destroy(w);
}

#[test]
fn aw_world_destroy_null_safe() {
    // Should not crash on null
    let null_world = null_world();
    aw_world_destroy(null_world);
}

// ========================================================================
// AW_WORLD_SNAPSHOT_JSON — entity verification
// ========================================================================

fn get_snapshot_string(w: AWWorld) -> String {
    let mut buf = [0u8; 8192];
    let _n = aw_world_snapshot_json(w, buf.as_mut_ptr(), buf.len());
    let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const i8) };
    cstr.to_str().unwrap().to_string()
}

#[test]
fn snapshot_is_valid_json() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let _val: serde_json::Value =
        serde_json::from_str(&s).unwrap_or_else(|e| panic!("invalid JSON: {e}\nraw: {s}"));
    aw_world_destroy(w);
}

#[test]
fn snapshot_has_entities_array() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(
        val["entities"].is_array(),
        "snapshot must have entities array"
    );
    aw_world_destroy(w);
}

#[test]
fn snapshot_has_three_entities() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    assert_eq!(entities.len(), 3, "should have P, C, E entities");
    aw_world_destroy(w);
}

#[test]
fn snapshot_has_t_field() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    assert!(val.get("t").is_some(), "snapshot must have t field");
    aw_world_destroy(w);
}

#[test]
fn snapshot_null_world_returns_zero() {
    let null_world = null_world();
    let mut buf = [0u8; 64];
    let n = aw_world_snapshot_json(null_world, buf.as_mut_ptr(), buf.len());
    assert_eq!(n, 0, "null world should return 0");
}

// ========================================================================
// ENTITY SPAWN PARAMS — exact values from aw_world_create
// ========================================================================

fn find_entity_by_team(entities: &[serde_json::Value], team: u8) -> &serde_json::Value {
    entities
        .iter()
        .find(|e| e["team"].as_u64().unwrap() == team as u64)
        .unwrap_or_else(|| panic!("no entity with team {}", team))
}

#[test]
fn entity_p_position_2_2() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let p = find_entity_by_team(entities, 0);
    assert_eq!(p["x"].as_i64().unwrap(), 2, "P.x should be 2");
    assert_eq!(p["y"].as_i64().unwrap(), 2, "P.y should be 2");
    aw_world_destroy(w);
}

#[test]
fn entity_c_position_3_2() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let c = find_entity_by_team(entities, 1);
    assert_eq!(c["x"].as_i64().unwrap(), 3, "C.x should be 3");
    assert_eq!(c["y"].as_i64().unwrap(), 2, "C.y should be 2");
    aw_world_destroy(w);
}

#[test]
fn entity_e_position_7_2() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let e = find_entity_by_team(entities, 2);
    assert_eq!(e["x"].as_i64().unwrap(), 7, "E.x should be 7");
    assert_eq!(e["y"].as_i64().unwrap(), 2, "E.y should be 2");
    aw_world_destroy(w);
}

#[test]
fn entity_p_hp_100() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let p = find_entity_by_team(entities, 0);
    assert_eq!(p["hp"].as_i64().unwrap(), 100);
    aw_world_destroy(w);
}

#[test]
fn entity_c_hp_80() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let c = find_entity_by_team(entities, 1);
    assert_eq!(c["hp"].as_i64().unwrap(), 80);
    aw_world_destroy(w);
}

#[test]
fn entity_e_hp_60() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let e = find_entity_by_team(entities, 2);
    assert_eq!(e["hp"].as_i64().unwrap(), 60);
    aw_world_destroy(w);
}

#[test]
fn entity_c_ammo_10() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let c = find_entity_by_team(entities, 1);
    assert_eq!(c["ammo"].as_i64().unwrap(), 10);
    aw_world_destroy(w);
}

#[test]
fn entity_p_ammo_zero() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let p = find_entity_by_team(entities, 0);
    assert_eq!(p["ammo"].as_i64().unwrap(), 0);
    aw_world_destroy(w);
}

#[test]
fn entity_e_ammo_zero() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let e = find_entity_by_team(entities, 2);
    assert_eq!(e["ammo"].as_i64().unwrap(), 0);
    aw_world_destroy(w);
}

#[test]
fn entity_teams_are_0_1_2() {
    let w = aw_world_create();
    let s = get_snapshot_string(w);
    let val: serde_json::Value = serde_json::from_str(&s).unwrap();
    let entities = val["entities"].as_array().unwrap();
    let mut teams: Vec<u64> = entities
        .iter()
        .map(|e| e["team"].as_u64().unwrap())
        .collect();
    teams.sort();
    assert_eq!(teams, vec![0, 1, 2]);
    aw_world_destroy(w);
}

// ========================================================================
// SUBMIT INTENT — error paths
// ========================================================================

#[test]
fn submit_intent_null_world() {
    let null_world = null_world();
    let json = b"{}\0";
    let code =
        unsafe { aw_world_submit_intent_json(null_world, 0, json.as_ptr() as *const i8, None) };
    assert_eq!(code, AW_ERR_NULL);
}

#[test]
fn submit_intent_null_json() {
    let w = aw_world_create();
    let code = unsafe { aw_world_submit_intent_json(w, 0, std::ptr::null(), None) };
    assert_eq!(code, AW_ERR_PARAM);
    aw_world_destroy(w);
}

#[test]
fn submit_intent_bad_json() {
    let w = aw_world_create();
    let bad = b"not valid json\0";
    let code = unsafe { aw_world_submit_intent_json(w, 0, bad.as_ptr() as *const i8, None) };
    assert_eq!(code, AW_ERR_PARSE);
    aw_world_destroy(w);
}

// ========================================================================
// AW_LAST_ERROR_STRING
// ========================================================================

#[test]
fn last_error_null_buf_returns_required() {
    let n = aw_last_error_string(std::ptr::null_mut(), 0);
    assert!(n >= 1, "should need at least 1 byte for NUL");
}

#[test]
fn last_error_has_content_after_null_submit() {
    let null_world = null_world();
    let json = b"{}\0";
    unsafe {
        aw_world_submit_intent_json(null_world, 0, json.as_ptr() as *const i8, None);
    }
    let mut buf = [0u8; 256];
    let n = aw_last_error_string(buf.as_mut_ptr(), buf.len());
    assert!(n > 1, "should have error content after null-world submit");
    let cstr = unsafe { CStr::from_ptr(buf.as_ptr() as *const i8) };
    let s = cstr.to_str().unwrap();
    assert!(
        s.contains("null"),
        "error should mention 'null', got: {}",
        s
    );
}

// ========================================================================
// VERSION STRUCT (Rust side)
// ========================================================================

#[test]
fn version_struct_serde_roundtrip() {
    let v = Version {
        major: 1,
        minor: 2,
        patch: 3,
    };
    let json = serde_json::to_string(&v).unwrap();
    let v2: Version = serde_json::from_str(&json).unwrap();
    assert_eq!(v2.major, 1);
    assert_eq!(v2.minor, 2);
    assert_eq!(v2.patch, 3);
}

#[test]
fn version_clone() {
    let v = Version {
        major: 5,
        minor: 6,
        patch: 7,
    };
    let v2 = v.clone();
    assert_eq!(v2.major, 5);
    assert_eq!(v2.minor, 6);
    assert_eq!(v2.patch, 7);
}

// ========================================================================
// SDK ERROR
// ========================================================================

#[test]
fn sdk_error_schema_display() {
    let e = SdkError::Schema("test mismatch".to_string());
    let s = format!("{}", e);
    assert!(s.contains("schema mismatch"), "got: {}", s);
    assert!(s.contains("test mismatch"), "got: {}", s);
}

#[test]
fn sdk_error_io_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let e = SdkError::Io(io_err);
    let s = format!("{}", e);
    assert!(s.contains("io:"), "got: {}", s);
}

#[test]
fn sdk_error_from_io() {
    let io_err = std::io::Error::other("test");
    let sdk_err: SdkError = io_err.into();
    assert!(matches!(sdk_err, SdkError::Io(_)));
}

#[test]
fn sdk_error_debug() {
    let e = SdkError::Schema("x".to_string());
    let dbg = format!("{:?}", e);
    assert!(dbg.contains("Schema"), "debug should contain variant name");
}

// ========================================================================
// AW_WORLD_TICK — basic smoke
// ========================================================================

#[test]
fn aw_world_tick_null_safe() {
    let null_world = null_world();
    aw_world_tick(null_world, 1.0 / 60.0);
    // Should not crash
}

#[test]
fn aw_world_tick_valid_world() {
    let w = aw_world_create();
    aw_world_tick(w, 1.0 / 60.0);
    // Verify world still valid after tick
    let mut buf = [0u8; 4096];
    let n = aw_world_snapshot_json(w, buf.as_mut_ptr(), buf.len());
    assert!(n > 0);
    aw_world_destroy(w);
}

// ========================================================================
// CALLBACK REGISTRATION — null safety
// ========================================================================

#[test]
fn set_snapshot_callback_null_world() {
    let null_world = null_world();
    aw_world_set_snapshot_callback(null_world, None);
    // Should not crash
}

#[test]
fn set_delta_callback_null_world() {
    let null_world = null_world();
    aw_world_set_delta_callback(null_world, None);
    // Should not crash
}
