//! Mutation-resistant comprehensive tests for astraweave-steam.
//!
//! Tests the MockPlatform (requires --features mock) and constants.
//!
//! Most tests require `MockPlatform` which is behind the `mock` feature.
//! The CI passes `--all-features` so these tests always run there.
//! Locally, run with: `cargo test -p astraweave-steam --features mock`

#![cfg(feature = "mock")]

use astraweave_steam::*;

// ═══════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_app_id_is_480() {
    assert_eq!(TEST_APP_ID, 480);
}

#[test]
fn test_app_id_not_zero() {
    assert_ne!(TEST_APP_ID, 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// MockPlatform construction
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mock_platform_new() {
    let _mp = MockPlatform::new("TestPlayer");
}

#[test]
fn mock_platform_new_from_string() {
    let name = String::from("StringPlayer");
    let _mp = MockPlatform::new(name);
}

#[test]
fn mock_platform_new_empty_name() {
    let _mp = MockPlatform::new("");
}

// ═══════════════════════════════════════════════════════════════════════════
// Platform trait — player_name
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mock_player_name() {
    let mp = MockPlatform::new("Hero");
    assert_eq!(mp.player_name(), "Hero");
}

#[test]
fn mock_player_name_preserves_unicode() {
    let mp = MockPlatform::new("プレイヤー");
    assert_eq!(mp.player_name(), "プレイヤー");
}

#[test]
fn mock_player_name_empty() {
    let mp = MockPlatform::new("");
    assert_eq!(mp.player_name(), "");
}

// ═══════════════════════════════════════════════════════════════════════════
// Platform trait — is_available
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mock_is_available_true() {
    let mp = MockPlatform::new("P");
    assert!(mp.is_available());
}

// ═══════════════════════════════════════════════════════════════════════════
// Platform trait — cloud
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mock_cloud_enabled_true() {
    let mp = MockPlatform::new("P");
    assert!(mp.cloud_enabled());
}

#[test]
fn mock_cloud_save_ok() {
    let mp = MockPlatform::new("P");
    assert!(mp.cloud_save("save.dat", &[1, 2, 3]).is_ok());
}

#[test]
fn mock_cloud_load_returns_empty() {
    let mp = MockPlatform::new("P");
    let data = mp.cloud_load("save.dat").unwrap();
    assert!(data.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Platform trait — achievements
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mock_unlock_achievement_ok() {
    let mp = MockPlatform::new("P");
    assert!(mp.unlock_achievement("first_blood").is_ok());
}

#[test]
fn mock_unlock_achievement_empty_name_ok() {
    let mp = MockPlatform::new("P");
    assert!(mp.unlock_achievement("").is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// Platform trait — stats
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mock_set_stat_i32_ok() {
    let mp = MockPlatform::new("P");
    assert!(mp.set_stat_i32("kills", 42).is_ok());
}

#[test]
fn mock_set_stat_f32_ok() {
    let mp = MockPlatform::new("P");
    assert!(mp.set_stat_f32("accuracy", 0.75).is_ok());
}

#[test]
fn mock_get_stat_i32_returns_zero() {
    let mp = MockPlatform::new("P");
    let v = mp.get_stat_i32("kills").unwrap();
    assert_eq!(v, 0);
}

#[test]
fn mock_store_stats_ok() {
    let mp = MockPlatform::new("P");
    assert!(mp.store_stats().is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// Combined workflows
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn mock_full_workflow() {
    let mp = MockPlatform::new("Adventurer");
    assert!(mp.is_available());
    assert!(mp.cloud_enabled());
    assert_eq!(mp.player_name(), "Adventurer");

    // Stats
    mp.set_stat_i32("enemies_killed", 100).unwrap();
    mp.set_stat_f32("play_hours", 24.5).unwrap();
    assert_eq!(mp.get_stat_i32("enemies_killed").unwrap(), 0); // mock always returns 0
    mp.store_stats().unwrap();

    // Achievements
    mp.unlock_achievement("completionist").unwrap();

    // Cloud
    mp.cloud_save("profile.dat", b"data").unwrap();
    let loaded = mp.cloud_load("profile.dat").unwrap();
    assert!(loaded.is_empty()); // mock always returns empty
}

#[test]
fn mock_multiple_instances_independent() {
    let p1 = MockPlatform::new("Player1");
    let p2 = MockPlatform::new("Player2");
    assert_eq!(p1.player_name(), "Player1");
    assert_eq!(p2.player_name(), "Player2");
    assert_ne!(p1.player_name(), p2.player_name());
}
