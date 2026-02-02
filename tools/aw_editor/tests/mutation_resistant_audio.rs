//! Mutation-resistant tests for Audio Panel system
//!
//! These tests target boundary conditions, comparison operators, and boolean return paths
//! to achieve high mutation testing kill rates (90%+).

use aw_editor_lib::panels::{AudioTab, MusicMood, ReverbEnvironment, SpatialPreset};
use std::collections::HashSet;

// ============================================================================
// MUSIC MOOD TESTS
// ============================================================================

mod music_mood_tests {
    use super::*;

    #[test]
    fn test_all_count() {
        assert_eq!(MusicMood::all().len(), 9);
    }

    #[test]
    fn test_all_names_unique() {
        let names: Vec<&str> = MusicMood::all().iter().map(|c| c.name()).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn test_all_icons_unique() {
        let icons: Vec<&str> = MusicMood::all().iter().map(|c| c.icon()).collect();
        let unique: HashSet<_> = icons.iter().collect();
        assert_eq!(icons.len(), unique.len());
    }

    // Test each variant name
    #[test]
    fn test_ambient_name() {
        assert_eq!(MusicMood::Ambient.name(), "Ambient");
    }

    #[test]
    fn test_calm_name() {
        assert_eq!(MusicMood::Calm.name(), "Calm");
    }

    #[test]
    fn test_exploration_name() {
        assert_eq!(MusicMood::Exploration.name(), "Exploration");
    }

    #[test]
    fn test_combat_name() {
        assert_eq!(MusicMood::Combat.name(), "Combat");
    }

    #[test]
    fn test_tension_name() {
        assert_eq!(MusicMood::Tension.name(), "Tension");
    }

    #[test]
    fn test_victory_name() {
        assert_eq!(MusicMood::Victory.name(), "Victory");
    }

    #[test]
    fn test_defeat_name() {
        assert_eq!(MusicMood::Defeat.name(), "Defeat");
    }

    #[test]
    fn test_boss_name() {
        assert_eq!(MusicMood::Boss.name(), "Boss");
    }

    #[test]
    fn test_menu_name() {
        assert_eq!(MusicMood::Menu.name(), "Menu");
    }

    // Test is_combat_related()
    #[test]
    fn test_combat_is_combat_related() {
        assert!(MusicMood::Combat.is_combat_related());
    }

    #[test]
    fn test_tension_is_combat_related() {
        assert!(MusicMood::Tension.is_combat_related());
    }

    #[test]
    fn test_boss_is_combat_related() {
        assert!(MusicMood::Boss.is_combat_related());
    }

    #[test]
    fn test_ambient_is_not_combat_related() {
        assert!(!MusicMood::Ambient.is_combat_related());
    }

    #[test]
    fn test_calm_is_not_combat_related() {
        assert!(!MusicMood::Calm.is_combat_related());
    }

    #[test]
    fn test_exploration_is_not_combat_related() {
        assert!(!MusicMood::Exploration.is_combat_related());
    }

    #[test]
    fn test_victory_is_not_combat_related() {
        assert!(!MusicMood::Victory.is_combat_related());
    }

    #[test]
    fn test_defeat_is_not_combat_related() {
        assert!(!MusicMood::Defeat.is_combat_related());
    }

    #[test]
    fn test_menu_is_not_combat_related() {
        assert!(!MusicMood::Menu.is_combat_related());
    }

    // Test is_positive()
    #[test]
    fn test_victory_is_positive() {
        assert!(MusicMood::Victory.is_positive());
    }

    #[test]
    fn test_calm_is_positive() {
        assert!(MusicMood::Calm.is_positive());
    }

    #[test]
    fn test_combat_is_not_positive() {
        assert!(!MusicMood::Combat.is_positive());
    }

    #[test]
    fn test_defeat_is_not_positive() {
        assert!(!MusicMood::Defeat.is_positive());
    }

    #[test]
    fn test_tension_is_not_positive() {
        assert!(!MusicMood::Tension.is_positive());
    }

    #[test]
    fn test_boss_is_not_positive() {
        assert!(!MusicMood::Boss.is_positive());
    }

    // Test intensity()
    #[test]
    fn test_ambient_intensity() {
        assert_eq!(MusicMood::Ambient.intensity(), 1);
    }

    #[test]
    fn test_calm_intensity() {
        assert_eq!(MusicMood::Calm.intensity(), 1);
    }

    #[test]
    fn test_menu_intensity() {
        assert_eq!(MusicMood::Menu.intensity(), 1);
    }

    #[test]
    fn test_exploration_intensity() {
        assert_eq!(MusicMood::Exploration.intensity(), 2);
    }

    #[test]
    fn test_defeat_intensity() {
        assert_eq!(MusicMood::Defeat.intensity(), 2);
    }

    #[test]
    fn test_tension_intensity() {
        assert_eq!(MusicMood::Tension.intensity(), 3);
    }

    #[test]
    fn test_combat_intensity() {
        assert_eq!(MusicMood::Combat.intensity(), 4);
    }

    #[test]
    fn test_victory_intensity() {
        assert_eq!(MusicMood::Victory.intensity(), 4);
    }

    #[test]
    fn test_boss_intensity() {
        assert_eq!(MusicMood::Boss.intensity(), 5);
    }

    // Test Display trait
    #[test]
    fn test_display_contains_icon() {
        let display = format!("{}", MusicMood::Combat);
        assert!(display.contains("⚔️"));
    }

    #[test]
    fn test_display_contains_name() {
        let display = format!("{}", MusicMood::Victory);
        assert!(display.contains("Victory"));
    }

    // Test default
    #[test]
    fn test_default_is_ambient() {
        assert_eq!(MusicMood::default(), MusicMood::Ambient);
    }
}

// ============================================================================
// SPATIAL PRESET TESTS
// ============================================================================

mod spatial_preset_tests {
    use super::*;

    #[test]
    fn test_all_count() {
        assert_eq!(SpatialPreset::all().len(), 5);
    }

    #[test]
    fn test_all_names_unique() {
        let names: Vec<&str> = SpatialPreset::all().iter().map(|c| c.name()).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn test_all_icons_unique() {
        let icons: Vec<&str> = SpatialPreset::all().iter().map(|c| c.icon()).collect();
        let unique: HashSet<_> = icons.iter().collect();
        assert_eq!(icons.len(), unique.len());
    }

    #[test]
    fn test_all_descriptions_unique() {
        let descs: Vec<&str> = SpatialPreset::all().iter().map(|c| c.description()).collect();
        let unique: HashSet<_> = descs.iter().collect();
        assert_eq!(descs.len(), unique.len());
    }

    // Test each variant name
    #[test]
    fn test_standard_name() {
        assert_eq!(SpatialPreset::Standard.name(), "Standard");
    }

    #[test]
    fn test_headphones_name() {
        assert_eq!(SpatialPreset::Headphones.name(), "Headphones");
    }

    #[test]
    fn test_speakers_name() {
        assert_eq!(SpatialPreset::Speakers.name(), "Speakers");
    }

    #[test]
    fn test_surround_name() {
        assert_eq!(SpatialPreset::Surround.name(), "Surround");
    }

    #[test]
    fn test_vr_name() {
        assert_eq!(SpatialPreset::VR.name(), "VR");
    }

    // Test ear_separation()
    #[test]
    fn test_standard_ear_separation() {
        assert_eq!(SpatialPreset::Standard.ear_separation(), 0.2);
    }

    #[test]
    fn test_headphones_ear_separation() {
        assert_eq!(SpatialPreset::Headphones.ear_separation(), 0.18);
    }

    #[test]
    fn test_speakers_ear_separation() {
        assert_eq!(SpatialPreset::Speakers.ear_separation(), 0.5);
    }

    #[test]
    fn test_surround_ear_separation() {
        assert_eq!(SpatialPreset::Surround.ear_separation(), 0.25);
    }

    #[test]
    fn test_vr_ear_separation() {
        assert_eq!(SpatialPreset::VR.ear_separation(), 0.2);
    }

    // Test is_multichannel()
    #[test]
    fn test_surround_is_multichannel() {
        assert!(SpatialPreset::Surround.is_multichannel());
    }

    #[test]
    fn test_vr_is_multichannel() {
        assert!(SpatialPreset::VR.is_multichannel());
    }

    #[test]
    fn test_standard_is_not_multichannel() {
        assert!(!SpatialPreset::Standard.is_multichannel());
    }

    #[test]
    fn test_headphones_is_not_multichannel() {
        assert!(!SpatialPreset::Headphones.is_multichannel());
    }

    #[test]
    fn test_speakers_is_not_multichannel() {
        assert!(!SpatialPreset::Speakers.is_multichannel());
    }

    // Test Display trait
    #[test]
    fn test_display_contains_icon() {
        let display = format!("{}", SpatialPreset::Headphones);
        assert!(display.contains("🎧"));
    }

    #[test]
    fn test_display_contains_name() {
        let display = format!("{}", SpatialPreset::Surround);
        assert!(display.contains("Surround"));
    }

    // Test default
    #[test]
    fn test_default_is_standard() {
        assert_eq!(SpatialPreset::default(), SpatialPreset::Standard);
    }
}

// ============================================================================
// REVERB ENVIRONMENT TESTS
// ============================================================================

mod reverb_environment_tests {
    use super::*;

    #[test]
    fn test_all_count() {
        assert_eq!(ReverbEnvironment::all().len(), 8);
    }

    #[test]
    fn test_all_names_unique() {
        let names: Vec<&str> = ReverbEnvironment::all().iter().map(|c| c.name()).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn test_all_icons_unique() {
        let icons: Vec<&str> = ReverbEnvironment::all().iter().map(|c| c.icon()).collect();
        let unique: HashSet<_> = icons.iter().collect();
        assert_eq!(icons.len(), unique.len());
    }

    // Test each variant name
    #[test]
    fn test_none_name() {
        assert_eq!(ReverbEnvironment::None.name(), "None");
    }

    #[test]
    fn test_small_room_name() {
        assert_eq!(ReverbEnvironment::SmallRoom.name(), "Small Room");
    }

    #[test]
    fn test_large_room_name() {
        assert_eq!(ReverbEnvironment::LargeRoom.name(), "Large Room");
    }

    #[test]
    fn test_hall_name() {
        assert_eq!(ReverbEnvironment::Hall.name(), "Hall");
    }

    #[test]
    fn test_cave_name() {
        assert_eq!(ReverbEnvironment::Cave.name(), "Cave");
    }

    #[test]
    fn test_forest_name() {
        assert_eq!(ReverbEnvironment::Forest.name(), "Forest");
    }

    #[test]
    fn test_underwater_name() {
        assert_eq!(ReverbEnvironment::Underwater.name(), "Underwater");
    }

    #[test]
    fn test_cathedral_name() {
        assert_eq!(ReverbEnvironment::Cathedral.name(), "Cathedral");
    }

    // Test decay_time()
    #[test]
    fn test_none_decay_time() {
        assert_eq!(ReverbEnvironment::None.decay_time(), 0.0);
    }

    #[test]
    fn test_small_room_decay_time() {
        assert_eq!(ReverbEnvironment::SmallRoom.decay_time(), 0.5);
    }

    #[test]
    fn test_cathedral_decay_time() {
        assert_eq!(ReverbEnvironment::Cathedral.decay_time(), 5.0);
    }

    #[test]
    fn test_cave_decay_time() {
        assert_eq!(ReverbEnvironment::Cave.decay_time(), 4.0);
    }

    // Test wet_dry_mix()
    #[test]
    fn test_none_wet_dry_mix() {
        assert_eq!(ReverbEnvironment::None.wet_dry_mix(), 0.0);
    }

    #[test]
    fn test_underwater_wet_dry_mix() {
        assert_eq!(ReverbEnvironment::Underwater.wet_dry_mix(), 0.7);
    }

    #[test]
    fn test_cave_wet_dry_mix() {
        assert_eq!(ReverbEnvironment::Cave.wet_dry_mix(), 0.6);
    }

    // Test is_indoor()
    #[test]
    fn test_small_room_is_indoor() {
        assert!(ReverbEnvironment::SmallRoom.is_indoor());
    }

    #[test]
    fn test_large_room_is_indoor() {
        assert!(ReverbEnvironment::LargeRoom.is_indoor());
    }

    #[test]
    fn test_hall_is_indoor() {
        assert!(ReverbEnvironment::Hall.is_indoor());
    }

    #[test]
    fn test_cathedral_is_indoor() {
        assert!(ReverbEnvironment::Cathedral.is_indoor());
    }

    #[test]
    fn test_none_is_not_indoor() {
        assert!(!ReverbEnvironment::None.is_indoor());
    }

    #[test]
    fn test_cave_is_not_indoor() {
        assert!(!ReverbEnvironment::Cave.is_indoor());
    }

    #[test]
    fn test_forest_is_not_indoor() {
        assert!(!ReverbEnvironment::Forest.is_indoor());
    }

    #[test]
    fn test_underwater_is_not_indoor() {
        assert!(!ReverbEnvironment::Underwater.is_indoor());
    }

    // Test is_natural()
    #[test]
    fn test_cave_is_natural() {
        assert!(ReverbEnvironment::Cave.is_natural());
    }

    #[test]
    fn test_forest_is_natural() {
        assert!(ReverbEnvironment::Forest.is_natural());
    }

    #[test]
    fn test_underwater_is_natural() {
        assert!(ReverbEnvironment::Underwater.is_natural());
    }

    #[test]
    fn test_none_is_not_natural() {
        assert!(!ReverbEnvironment::None.is_natural());
    }

    #[test]
    fn test_small_room_is_not_natural() {
        assert!(!ReverbEnvironment::SmallRoom.is_natural());
    }

    #[test]
    fn test_hall_is_not_natural() {
        assert!(!ReverbEnvironment::Hall.is_natural());
    }

    #[test]
    fn test_cathedral_is_not_natural() {
        assert!(!ReverbEnvironment::Cathedral.is_natural());
    }

    // Test Display trait
    #[test]
    fn test_display_contains_icon() {
        let display = format!("{}", ReverbEnvironment::Cathedral);
        assert!(display.contains("⛪"));
    }

    #[test]
    fn test_display_contains_name() {
        let display = format!("{}", ReverbEnvironment::Cave);
        assert!(display.contains("Cave"));
    }

    // Test default
    #[test]
    fn test_default_is_none() {
        assert_eq!(ReverbEnvironment::default(), ReverbEnvironment::None);
    }
}

// ============================================================================
// AUDIO TAB TESTS
// ============================================================================

mod audio_tab_tests {
    use super::*;

    #[test]
    fn test_all_count() {
        assert_eq!(AudioTab::all().len(), 5);
    }

    #[test]
    fn test_all_names_unique() {
        let names: Vec<&str> = AudioTab::all().iter().map(|c| c.name()).collect();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn test_all_icons_unique() {
        let icons: Vec<&str> = AudioTab::all().iter().map(|c| c.icon()).collect();
        let unique: HashSet<_> = icons.iter().collect();
        assert_eq!(icons.len(), unique.len());
    }

    // Test each variant name
    #[test]
    fn test_mixer_name() {
        assert_eq!(AudioTab::Mixer.name(), "Mixer");
    }

    #[test]
    fn test_music_name() {
        assert_eq!(AudioTab::Music.name(), "Music");
    }

    #[test]
    fn test_spatial_name() {
        assert_eq!(AudioTab::Spatial.name(), "Spatial");
    }

    #[test]
    fn test_emitters_name() {
        assert_eq!(AudioTab::Emitters.name(), "Emitters");
    }

    #[test]
    fn test_preview_name() {
        assert_eq!(AudioTab::Preview.name(), "Preview");
    }

    // Test Display trait
    #[test]
    fn test_display_contains_icon() {
        let display = format!("{}", AudioTab::Mixer);
        assert!(display.contains("🎚️"));
    }

    #[test]
    fn test_display_contains_name() {
        let display = format!("{}", AudioTab::Music);
        assert!(display.contains("Music"));
    }

    // Test default
    #[test]
    fn test_default_is_mixer() {
        assert_eq!(AudioTab::default(), AudioTab::Mixer);
    }
}
