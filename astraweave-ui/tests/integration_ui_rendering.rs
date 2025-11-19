use astraweave_ui::{draw_ui, Accessibility, UiFlags, MenuManager, MenuAction};
use astraweave_gameplay::{Inventory, RecipeBook, Stats, QuestLog};
use egui::Context;
use glam::Vec3;
use std::collections::HashMap;

#[test]
fn test_ui_rendering_all_flags_on() {
    let ctx = Context::default();
    let mut flags = UiFlags {
        show_menu: true,
        show_inventory: true,
        show_crafting: true,
        show_map: true,
        show_quests: true,
        show_settings: true,
        ..Default::default()
    };
    let mut acc = Accessibility::default();
    let mut menu_manager = MenuManager::default();
    // Default state is MainMenu, which matches "show_menu: true" intent
    
    let stats = Stats::new(100);
    let pos = Vec3::new(10.0, 20.0, 30.0);
    let mut inventory = Inventory::default();
    let recipes = RecipeBook { recipes: vec![] };
    let mut quests = QuestLog { quests: HashMap::new() };

    let _ = ctx.run(Default::default(), |ctx| {
        draw_ui(
            ctx,
            &mut flags,
            &mut menu_manager,
            &mut acc,
            &stats,
            pos,
            &mut inventory,
            Some(&recipes),
            Some(&mut quests),
        );
    });
}

#[test]
fn test_ui_rendering_all_flags_off() {
    let ctx = Context::default();
    let mut flags = UiFlags::default(); 
    let mut acc = Accessibility::default();
    let mut menu_manager = MenuManager::default();
    // Simulate starting game to hide menu
    menu_manager.handle_action(MenuAction::NewGame);
    
    let stats = Stats::new(100);
    let pos = Vec3::new(0.0, 0.0, 0.0);
    let mut inventory = Inventory::default();
    let recipes = RecipeBook { recipes: vec![] };
    let mut quests = QuestLog { quests: HashMap::new() };

    let _ = ctx.run(Default::default(), |ctx| {
        draw_ui(
            ctx,
            &mut flags,
            &mut menu_manager,
            &mut acc,
            &stats,
            pos,
            &mut inventory,
            Some(&recipes),
            Some(&mut quests),
        );
    });
}
