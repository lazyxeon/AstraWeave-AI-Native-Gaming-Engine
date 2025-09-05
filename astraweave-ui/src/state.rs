use glam::Vec3;
use serde::{Serialize, Deserialize};
use astraweave_gameplay::{Inventory, RecipeBook};
use astraweave_gameplay::quests::QuestLog;
use astraweave_gameplay::stats::Stats;

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
        Self { high_contrast_ui:false, reduce_motion:false, subtitles:true, subtitle_scale:1.0, colorblind_mode: None }
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

#[derive(Clone, Debug)]
pub struct UiData<'a> {
    pub player_stats: &'a Stats,
    pub player_pos: Vec3,
    pub inventory: &'a mut Inventory,
    pub recipe_book: Option<&'a RecipeBook>,
    pub quest_log: Option<&'a mut QuestLog>,
}
