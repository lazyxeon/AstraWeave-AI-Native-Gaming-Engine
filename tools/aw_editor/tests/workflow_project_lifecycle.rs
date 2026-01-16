//! Integration Tests for Project Lifecycle and Workflow
//! 
//! Covers:
//! 1. Project Initialization (folders, game.toml)
//! 2. Project Serialization (save/load)
//! 3. File Watcher Integration (asset discovery)

use aw_editor_lib::game_project::{GameProject, ProjectMetadata, BuildSettings, AssetSettings};
use aw_editor_lib::file_watcher::{FileWatcher, ReloadEvent};
use std::path::PathBuf;
use std::fs;
use std::thread;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_project_lifecycle_end_to_end() {
    // 1. Setup Sandbox
    let dir = tempdir().expect("Failed to create temp dir");
    let project_root = dir.path().to_path_buf();
    
    // 2. Define Project
    let metadata = ProjectMetadata {
        name: "Integration Test Project".to_string(),
        version: "0.1.0".to_string(),
        author: "Test Runner".to_string(),
        description: "A temporary project for integration testing".to_string(),
        icon: None,
        identifier: Some("com.test.integration".to_string()),
    };
    
    let build = BuildSettings {
        entry_scene: PathBuf::from("scenes/main.scene"),
        default_target: "windows".to_string(),
        default_profile: "debug".to_string(),
        output_dir: PathBuf::from("dist"),
        features: vec![],
    };
    
    let project = GameProject {
        project: metadata,
        build,
        assets: AssetSettings { 
            include: vec![], 
            exclude: vec![],
            compress: true,
            compression_level: 3,
        },
        platforms: Default::default(),
    };
    
    // 3. Serialize to Disk
    let toml_path = project_root.join("game.toml");
    let toml_str = toml::to_string_pretty(&project).expect("Failed to serialize project");
    fs::write(&toml_path, toml_str).expect("Failed to write game.toml");
    
    // 4. Create Project Structure
    let assets_dir = project_root.join("assets");
    let materials_dir = assets_dir.join("materials");
    fs::create_dir_all(&materials_dir).expect("Failed to create assets structure");
    
    // 5. Load Verification (Deserialize)
    let loaded_toml = fs::read_to_string(&toml_path).expect("Failed to read game.toml");
    let loaded_project: GameProject = toml::from_str(&loaded_toml).expect("Failed to deserialize project");
    
    assert_eq!(loaded_project.project.name, "Integration Test Project");
    assert_eq!(loaded_project.build.output_dir, PathBuf::from("dist"));
    
    // 6. File Watcher Integration
    // Start watching the materials directory
    // Note: notify might fail in some sandboxed environments, so we wrap in result check
    // If it fails, we skip this part of the test to avoid flakiness in restrictive CI
    if let Ok(watcher) = FileWatcher::new(materials_dir.to_str().unwrap()) {
        // Create a new material file
        let new_material_path = materials_dir.join("test_mat.toml");
        fs::write(&new_material_path, "material_data = true").expect("Failed to write asset");
        
        // Wait for debounce (500ms) + buffer
        thread::sleep(Duration::from_millis(1500));
        
        // Check for event
        let mut event_received = false;
        while let Ok(event) = watcher.receiver.try_recv() {
            if let ReloadEvent::Material(path) = event {
                // Canonicalize paths for comparison to handle symlinks/relative paths
                if let (Ok(p1), Ok(p2)) = (fs::canonicalize(&path), fs::canonicalize(&new_material_path)) {
                    if p1 == p2 {
                        event_received = true;
                        break;
                    }
                } else {
                    // Fallback to simpler check
                    if path.file_name() == new_material_path.file_name() {
                        event_received = true;
                        break;
                    }
                }
            }
        }
        
        // Assert event was caught
        assert!(event_received, "FileWatcher failed to detect new material creation");
    } else {
        eprintln!("WARNING: Skipped FileWatcher test due to initialization failure (permissions?)");
    }
}
