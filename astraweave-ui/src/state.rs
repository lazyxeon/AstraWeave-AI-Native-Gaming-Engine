use astraweave_gameplay::quests::QuestLog;
use astraweave_gameplay::stats::Stats;
use astraweave_gameplay::{Inventory, RecipeBook};
use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Accessibility {
    pub high_contrast_ui: bool,
    pub reduce_motion: bool,
    pub subtitles: bool,
    pub subtitle_scale: f32,
    pub colorblind_mode: Option<String>, // "protanopia"|"deuteranopia"|"tritanopia"
}

impl Default for Accessibility {
    fn default() -> Self {
        Self {
            high_contrast_ui: false,
            reduce_motion: false,
            subtitles: true,
            subtitle_scale: 1.0,
            colorblind_mode: None,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct UiFlags {
    pub show_menu: bool,
    pub show_inventory: bool,
    pub show_map: bool,
    pub show_quests: bool,
    pub show_crafting: bool,
    pub show_settings: bool,
}

#[derive(Debug)]
pub struct UiData<'a> {
    pub player_stats: &'a Stats,
    pub player_pos: Vec3,
    pub inventory: &'a mut Inventory,
    pub recipe_book: Option<&'a RecipeBook>,
    pub quest_log: Option<&'a mut QuestLog>,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_default() {
        let acc = Accessibility::default();
        assert!(!acc.high_contrast_ui);
        assert!(!acc.reduce_motion);
        assert!(acc.subtitles); // On by default
        assert_eq!(acc.subtitle_scale, 1.0);
        assert_eq!(acc.colorblind_mode, None);
    }

    #[test]
    fn test_accessibility_serialization() {
        let mut acc = Accessibility::default();
        acc.high_contrast_ui = true;
        acc.colorblind_mode = Some("protanopia".to_string());

        let json = serde_json::to_string(&acc).unwrap();
        let deserialized: Accessibility = serde_json::from_str(&json).unwrap();

        assert!(deserialized.high_contrast_ui);
        assert_eq!(deserialized.colorblind_mode, Some("protanopia".to_string()));
    }

    #[test]
    fn test_ui_flags_default() {
        let flags = UiFlags::default();
        assert!(!flags.show_menu);
        assert!(!flags.show_inventory);
        assert!(!flags.show_map);
        assert!(!flags.show_quests);
        assert!(!flags.show_crafting);
        assert!(!flags.show_settings);
    }

    #[test]
    fn test_ui_flags_toggle() {
        let mut flags = UiFlags::default();
        flags.show_inventory = true;
        flags.show_map = true;

        assert!(flags.show_inventory);
        assert!(flags.show_map);
        assert!(!flags.show_menu);
    }

    #[test]
    fn test_accessibility_subtitle_scale_ranges() {
        let mut acc = Accessibility::default();

        // Small text
        acc.subtitle_scale = 0.5;
        assert_eq!(acc.subtitle_scale, 0.5);

        // Large text
        acc.subtitle_scale = 2.0;
        assert_eq!(acc.subtitle_scale, 2.0);
    }

    #[test]
    fn test_accessibility_colorblind_modes() {
        let mut acc = Accessibility::default();

        acc.colorblind_mode = Some("deuteranopia".to_string());
        assert_eq!(acc.colorblind_mode.as_ref().unwrap(), "deuteranopia");

        acc.colorblind_mode = Some("tritanopia".to_string());
        assert_eq!(acc.colorblind_mode.as_ref().unwrap(), "tritanopia");

        acc.colorblind_mode = None;
        assert!(acc.colorblind_mode.is_none());
    }
}
