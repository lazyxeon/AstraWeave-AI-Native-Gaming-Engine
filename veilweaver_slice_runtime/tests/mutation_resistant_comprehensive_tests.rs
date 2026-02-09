//! Mutation-resistant comprehensive tests for veilweaver_slice_runtime.
//!
//! Targets: VeilweaverSliceConfig (Deserialize, field defaults), VeilweaverRuntime
//! (construction, anchors_in_cell, run_tick, refresh_metadata).

use astraweave_scene::world_partition::{GridConfig, GridCoord, WorldPartition};
use veilweaver_slice_runtime::{VeilweaverRuntime, VeilweaverSliceConfig};

// =========================================================================
// VeilweaverSliceConfig — deserialization, field access
// =========================================================================

#[test]
fn config_deserialize_minimal() {
    let json = r#"{"dt": 0.016}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    assert!((config.dt - 0.016).abs() < 1e-6);
    assert!(config.initial_cell.is_none());
    assert!(config.camera_start.is_none());
}

#[test]
fn config_deserialize_with_initial_cell() {
    let json = r#"{"dt": 0.033, "initial_cell": [1, 2, 3]}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    assert!((config.dt - 0.033).abs() < 1e-6);
    let cell = config.initial_cell.unwrap();
    assert_eq!(cell, [1, 2, 3]);
}

#[test]
fn config_deserialize_with_camera_start() {
    let json = r#"{"dt": 0.016, "camera_start": [10.0, 20.0, 30.0]}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    let cam = config.camera_start.unwrap();
    assert!((cam[0] - 10.0).abs() < 1e-6);
    assert!((cam[1] - 20.0).abs() < 1e-6);
    assert!((cam[2] - 30.0).abs() < 1e-6);
}

#[test]
fn config_deserialize_all_fields() {
    let json = r#"{"dt": 0.05, "initial_cell": [0, 0, 0], "camera_start": [1.0, 2.0, 3.0]}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    assert!((config.dt - 0.05).abs() < 1e-6);
    assert_eq!(config.initial_cell.unwrap(), [0, 0, 0]);
    assert!((config.camera_start.unwrap()[0] - 1.0).abs() < 1e-6);
}

#[test]
fn config_deserialize_zero_dt() {
    let json = r#"{"dt": 0.0}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.dt, 0.0);
}

#[test]
fn config_deserialize_negative_dt() {
    let json = r#"{"dt": -1.0}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.dt, -1.0);
}

#[test]
fn config_clone() {
    let json = r#"{"dt": 0.016, "initial_cell": [1, 2, 3], "camera_start": [5.0, 6.0, 7.0]}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    let cloned = config.clone();
    assert!((config.dt - cloned.dt).abs() < 1e-10);
    assert_eq!(config.initial_cell, cloned.initial_cell);
    assert_eq!(config.camera_start, cloned.camera_start);
}

#[test]
fn config_debug_format() {
    let json = r#"{"dt": 0.016}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    let s = format!("{:?}", config);
    assert!(s.contains("VeilweaverSliceConfig"));
}

#[test]
fn config_deserialize_missing_dt_fails() {
    let json = r#"{}"#;
    let result = serde_json::from_str::<VeilweaverSliceConfig>(json);
    assert!(result.is_err());
}

#[test]
fn config_camera_start_default_none() {
    let json = r#"{"dt": 0.016, "initial_cell": [0, 0, 0]}"#;
    let config: VeilweaverSliceConfig = serde_json::from_str(json).unwrap();
    assert!(config.camera_start.is_none());
}

// =========================================================================
// VeilweaverRuntime — construction with empty partition
// =========================================================================

fn make_empty_partition() -> WorldPartition {
    WorldPartition::new(GridConfig::default())
}

fn make_default_config() -> VeilweaverSliceConfig {
    VeilweaverSliceConfig {
        dt: 0.016,
        initial_cell: None,
        camera_start: None,
    }
}

#[test]
fn runtime_new_empty_partition_succeeds() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let runtime = VeilweaverRuntime::new(config, partition);
    assert!(runtime.is_ok());
}

#[test]
fn runtime_metadata_empty_on_empty_partition() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let runtime = VeilweaverRuntime::new(config, partition).unwrap();
    assert!(runtime.metadata.anchors.is_empty());
    assert!(runtime.metadata.trigger_zones.is_empty());
}

#[test]
fn runtime_anchors_in_cell_empty() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let runtime = VeilweaverRuntime::new(config, partition).unwrap();
    let anchors = runtime.anchors_in_cell(GridCoord::new(0, 0, 0));
    assert!(anchors.is_empty());
}

#[test]
fn runtime_anchors_in_cell_nonexistent_coord() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let runtime = VeilweaverRuntime::new(config, partition).unwrap();
    let anchors = runtime.anchors_in_cell(GridCoord::new(99, 99, 99));
    assert!(anchors.is_empty());
}

#[test]
fn runtime_run_tick_no_panic() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let mut runtime = VeilweaverRuntime::new(config, partition).unwrap();
    runtime.run_tick(); // Should not panic on empty world
}

#[test]
fn runtime_run_tick_multiple_no_panic() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let mut runtime = VeilweaverRuntime::new(config, partition).unwrap();
    for _ in 0..10 {
        runtime.run_tick();
    }
}

#[test]
fn runtime_refresh_metadata_no_panic() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let mut runtime = VeilweaverRuntime::new(config, partition).unwrap();
    runtime.refresh_metadata();
    assert!(runtime.metadata.anchors.is_empty());
}

#[test]
fn runtime_dt_clamped_above_zero() {
    // VeilweaverRuntime::new clamps dt with .max(0.0001)
    let partition = make_empty_partition();
    let config = VeilweaverSliceConfig {
        dt: 0.0, // will be clamped to 0.0001
        initial_cell: None,
        camera_start: None,
    };
    let runtime = VeilweaverRuntime::new(config, partition);
    assert!(runtime.is_ok());
}

#[test]
fn runtime_negative_dt_clamped() {
    let partition = make_empty_partition();
    let config = VeilweaverSliceConfig {
        dt: -5.0, // will be clamped to 0.0001
        initial_cell: None,
        camera_start: None,
    };
    let runtime = VeilweaverRuntime::new(config, partition);
    assert!(runtime.is_ok());
}

#[test]
fn runtime_partition_accessible() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let runtime = VeilweaverRuntime::new(config, partition).unwrap();
    // Partition should still be accessible
    assert!(runtime.partition.loaded_cells().is_empty());
}

#[test]
fn runtime_add_post_setup_system_no_panic() {
    let partition = make_empty_partition();
    let config = make_default_config();
    let mut runtime = VeilweaverRuntime::new(config, partition).unwrap();
    fn noop(_: &mut astraweave_ecs::World) {}
    runtime.add_post_setup_system(noop);
    runtime.run_tick(); // Should still work with extra system
}
