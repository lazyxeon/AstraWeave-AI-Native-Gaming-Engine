#[cfg(test)]
mod tests {
    use crate::game_project::{GameProject, AssetSettings};
    use std::path::PathBuf;

    #[test]
    fn test_project_metadata_validation() {
        let mut project = GameProject::new("Test", "scene.sc");
        project.project.version = "invalid".to_string(); // Assuming typical semantic versioning checks might exist or be added
        // The current GameProject::validate implementation (from read_file earlier) 
        // checked for empty name. Let's verify that.
        
        project.project.name = "".to_string();
        let res = project.validate();
        assert!(res.is_err());
        assert!(res.unwrap_err().contains(&"Project name is required".to_string()));
    }

    #[test]
    fn test_build_settings_defaults() {
        let project = GameProject::default();
        assert_eq!(project.build.default_target, "windows");
        assert_eq!(project.build.default_profile, "release");
        assert_eq!(project.build.output_dir, PathBuf::from("builds"));
    }

    #[test]
    fn test_asset_settings_modification() {
        let mut settings = AssetSettings::default();
        settings.include.push("extra/**/*".to_string());
        settings.exclude.push("tmp/**/*".to_string());
        settings.compress = false;
        settings.compression_level = 9;

        assert!(settings.include.contains(&"extra/**/*".to_string()));
        assert!(settings.exclude.contains(&"tmp/**/*".to_string()));
        assert!(!settings.compress);
        assert_eq!(settings.compression_level, 9);
    }
    
    // Test serialization round-trip with all fields populated
    #[test]
    fn test_full_serialization_roundtrip() {
        let mut project = GameProject::new("Full Project", "main.scene");
        project.project.author = "Author".to_string();
        project.project.description = "Desc".to_string();
        project.project.version = "0.1.0".to_string();
        project.project.identifier = Some("com.test.game".to_string());
        
        project.build.features = vec!["feat1".to_string(), "feat2".to_string()];
        
        let toml = toml::to_string(&project).unwrap();
        let parsed: GameProject = toml::from_str(&toml).unwrap();
        
        assert_eq!(parsed.project.name, "Full Project");
        assert_eq!(parsed.project.identifier, Some("com.test.game".to_string()));
        assert_eq!(parsed.build.features.len(), 2);
    }
}
