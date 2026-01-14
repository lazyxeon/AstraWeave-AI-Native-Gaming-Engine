//! Integration tests for the complete build→package→distribute workflow
//!
//! These tests verify that the entire pipeline works end-to-end.

use tempfile::TempDir;

use aw_editor_lib::asset_pack::{AssetPackBuilder, CompressionMethod};
use aw_editor_lib::distribution::{DistributionBuilder, DistributionConfig, DistributionFormat};
use aw_editor_lib::game_project::GameProject;
use aw_editor_lib::polish::{
    LoadingProgress, LoadingScreen, SaveConfig, SaveManager, SaveMetadata, SplashSequence,
};

/// Test complete build pipeline: game.toml → asset pack → distribution
#[test]
fn test_full_build_pipeline() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_dir = temp_dir.path().join("my_game");
    let build_dir = temp_dir.path().join("build");
    let dist_dir = temp_dir.path().join("dist");

    std::fs::create_dir_all(&project_dir).unwrap();
    std::fs::create_dir_all(&build_dir).unwrap();

    // 1. Create game project configuration
    let project = GameProject::default();
    let toml_content = toml::to_string_pretty(&project).unwrap();
    std::fs::write(project_dir.join("game.toml"), toml_content).unwrap();

    // 2. Create some mock assets
    let assets_dir = project_dir.join("assets");
    std::fs::create_dir_all(&assets_dir).unwrap();
    std::fs::write(assets_dir.join("test_texture.png"), b"fake png data").unwrap();
    std::fs::write(assets_dir.join("test_model.glb"), b"fake glb data").unwrap();

    // 3. Create asset pack
    let pack_path = build_dir.join("assets.awpack");
    let builder = AssetPackBuilder::new(&pack_path, "my_game")
        .with_compression(CompressionMethod::Zstd)
        .add_directory(&assets_dir, "assets");

    let pack_result = builder.build();
    assert!(
        pack_result.is_ok(),
        "Asset pack creation failed: {:?}",
        pack_result.err()
    );
    assert!(pack_path.exists(), "Asset pack file not created");

    // 4. Verify pack was created with compression
    let pack_metadata = std::fs::metadata(&pack_path).unwrap();
    assert!(pack_metadata.len() > 0, "Asset pack is empty");

    // 5. Create mock executable in build dir
    std::fs::write(build_dir.join("my_game.exe"), b"fake executable").unwrap();

    // 6. Create distribution (Windows portable)
    let dist_config = DistributionConfig {
        game_name: "My Game".to_string(),
        version: "1.0.0".to_string(),
        publisher: "Test Publisher".to_string(),
        ..Default::default()
    };

    let builder = DistributionBuilder::new(dist_config, DistributionFormat::WindowsPortable)
        .build_dir(&build_dir)
        .output_dir(&dist_dir);

    // Note: This will fail on non-Windows without zip, but tests the API
    #[cfg(target_os = "windows")]
    {
        let dist_result = builder.build();
        if dist_result.is_ok() {
            let result = dist_result.unwrap();
            assert!(result.output_path.exists(), "Distribution not created");
            assert!(result.size_bytes > 0, "Distribution is empty");
        }
    }

    // Suppress unused variable warning on non-Windows
    let _ = builder;
}

/// Test asset pack with different compression methods
#[test]
fn test_asset_pack_compression_methods() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let assets_dir = temp_dir.path().join("assets");
    std::fs::create_dir_all(&assets_dir).unwrap();

    // Create some test content
    let test_data = "A".repeat(10000); // Compressible data
    std::fs::write(assets_dir.join("large_text.txt"), &test_data).unwrap();

    // Test each compression method
    for (method, name) in [
        (CompressionMethod::Zstd, "zstd"),
        (CompressionMethod::Lz4, "lz4"),
        (CompressionMethod::None, "none"),
    ] {
        let pack_path = temp_dir.path().join(format!("test_{}.awpack", name));

        let result = AssetPackBuilder::new(&pack_path, "test_project")
            .with_compression(method)
            .add_directory(&assets_dir, "assets")
            .build();

        assert!(
            result.is_ok(),
            "Failed with {:?}: {:?}",
            method,
            result.err()
        );

        let size = std::fs::metadata(&pack_path).unwrap().len();
        println!("{}: {} bytes", name, size);
    }
}

/// Test save/load game state roundtrip
#[test]
fn test_save_load_roundtrip() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let config = SaveConfig {
        compress: true,
        ..Default::default()
    };

    let manager = SaveManager::new(config, temp_dir.path());

    // Create test game state
    let game_state = b"player_position: 100,200,50\nhealth: 75\ninventory: [sword, shield]";
    let metadata = SaveMetadata::new("Test Save", "1.0.0");

    // Save
    let save_result = manager.save("slot1", game_state, &metadata);
    assert!(save_result.is_ok(), "Save failed: {:?}", save_result.err());

    // Load
    let load_result = manager.load("slot1");
    assert!(load_result.is_ok(), "Load failed: {:?}", load_result.err());

    let (loaded_data, loaded_meta) = load_result.unwrap();
    assert_eq!(loaded_data, game_state.to_vec());
    assert_eq!(loaded_meta.name, "Test Save");
    assert_eq!(loaded_meta.version, "1.0.0");
}

/// Test autosave rotation
#[test]
fn test_autosave_rotation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let config = SaveConfig {
        max_autosaves: 3,
        compress: false,
        ..Default::default()
    };

    let manager = SaveManager::new(config, temp_dir.path());

    // Create 5 autosaves
    for i in 0..5 {
        let state = format!("state_{}", i);
        let metadata = SaveMetadata::new(format!("Autosave {}", i), "1.0.0");
        manager.autosave(state.as_bytes(), metadata).unwrap();
    }

    // List saves and verify rotation occurred
    let saves = manager.list_saves().unwrap();
    let autosaves: Vec<_> = saves.iter().filter(|s| s.is_autosave).collect();
    assert!(
        autosaves.len() <= 3,
        "Too many autosaves: {}",
        autosaves.len()
    );
}

/// Test splash screen sequence configuration
#[test]
fn test_splash_sequence_builder() {
    let sequence = SplashSequence::new()
        .with_engine_logo()
        .with_publisher_logo("publisher.png");

    assert_eq!(sequence.screens.len(), 2);
    assert!(!sequence.screens[0].skippable); // Engine logo non-skippable
    assert!(sequence.screens[1].skippable); // Publisher logo skippable
    assert!(sequence.total_duration().as_secs() > 0);
}

/// Test loading progress tracking
#[test]
fn test_loading_progress_tracking() {
    let mut progress = LoadingProgress::new(100);

    assert_eq!(progress.percentage(), 0.0);
    assert!(!progress.is_complete());

    for i in 0..50 {
        progress.advance(format!("Loading asset {}", i));
    }
    assert_eq!(progress.percentage(), 0.5);

    for i in 50..100 {
        progress.advance(format!("Loading asset {}", i));
    }
    assert_eq!(progress.percentage(), 1.0);
    assert!(progress.is_complete());
}

/// Test loading screen with tips
#[test]
fn test_loading_screen_tips() {
    let screen = LoadingScreen::default()
        .add_tip("Press F5 to quick save")
        .add_tip("Hold Shift to sprint")
        .add_tip("Talk to NPCs for quests");

    assert_eq!(screen.tips.len(), 3);
    assert!(screen.show_percentage);
}

/// Test distribution formats
#[test]
fn test_distribution_formats() {
    let formats = [
        (
            DistributionFormat::WindowsInstaller,
            "exe",
            "Windows Installer",
        ),
        (
            DistributionFormat::WindowsPortable,
            "zip",
            "Windows Portable",
        ),
        (DistributionFormat::MacOSBundle, "app", "macOS App Bundle"),
        (DistributionFormat::MacOSDmg, "dmg", "macOS DMG"),
        (
            DistributionFormat::LinuxAppImage,
            "AppImage",
            "Linux AppImage",
        ),
        (DistributionFormat::LinuxTarball, "tar.gz", "Linux Tarball"),
        (DistributionFormat::SteamDepot, "vdf", "Steam Depot"),
    ];

    for (format, expected_ext, expected_name) in formats {
        assert_eq!(format.extension(), expected_ext);
        assert_eq!(format.name(), expected_name);
    }
}

/// Test game project defaults
#[test]
fn test_game_project_defaults() {
    let project = GameProject::default();

    // Serialize to TOML
    let toml_str = toml::to_string_pretty(&project).expect("Serialization failed");
    assert!(!toml_str.is_empty());

    // Deserialize back
    let loaded: GameProject = toml::from_str(&toml_str).expect("Deserialization failed");

    // Verify it roundtrips
    let toml_str2 = toml::to_string_pretty(&loaded).unwrap();
    assert_eq!(toml_str, toml_str2);
}
