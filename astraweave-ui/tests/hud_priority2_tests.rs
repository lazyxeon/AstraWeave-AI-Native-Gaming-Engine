/// Priority 2 Tests: HudManager State Management (Visibility, Dialogue, Tooltips, Spawning, Update Loop)
///
/// Sprint: Phase 8.6 UI Testing
/// Day 3-4: 20 tests for HudManager integration
use astraweave_ui::hud::{
    DamageType, DialogueChoice, DialogueNode, HudManager, HudState, TooltipData,
};

// ============================================================================
// Visibility Toggles (5 tests)
// ============================================================================

#[test]
fn test_hud_manager_toggle_visibility_master() {
    let mut hud = HudManager::new();

    // Initially visible
    assert!(hud.is_visible(), "HUD should be visible by default");

    // Toggle off
    hud.toggle_visibility();
    assert!(!hud.is_visible(), "HUD should be hidden after toggle");

    // Toggle back on
    hud.toggle_visibility();
    assert!(
        hud.is_visible(),
        "HUD should be visible after second toggle"
    );
}

#[test]
fn test_hud_manager_set_visible_explicit() {
    let mut hud = HudManager::new();

    // Set explicitly false
    hud.set_visible(false);
    assert!(!hud.is_visible());

    // Set explicitly true
    hud.set_visible(true);
    assert!(hud.is_visible());
}

#[test]
fn test_hud_manager_toggle_debug() {
    let mut hud = HudManager::new();

    // Initially not in debug mode
    assert!(
        !hud.state().debug_mode,
        "Debug mode should be off by default"
    );

    // Toggle debug on
    hud.toggle_debug();
    assert!(hud.state().debug_mode, "Debug mode should be on");

    // Toggle debug off
    hud.toggle_debug();
    assert!(!hud.state().debug_mode, "Debug mode should be off");
}

#[test]
fn test_hud_manager_toggle_quest_tracker() {
    let mut hud = HudManager::new();

    // Initially visible
    assert!(
        hud.state().show_objectives,
        "Quest tracker should be visible by default"
    );

    // Toggle off
    hud.toggle_quest_tracker();
    assert!(
        !hud.state().show_objectives,
        "Quest tracker should be hidden"
    );

    // Toggle back on
    hud.toggle_quest_tracker();
    assert!(
        hud.state().show_objectives,
        "Quest tracker should be visible"
    );
}

#[test]
fn test_hud_manager_toggle_minimap() {
    let mut hud = HudManager::new();

    // Initially visible
    assert!(
        hud.state().show_minimap,
        "Minimap should be visible by default"
    );

    // Toggle off
    hud.toggle_minimap();
    assert!(!hud.state().show_minimap, "Minimap should be hidden");

    // Toggle back on
    hud.toggle_minimap();
    assert!(hud.state().show_minimap, "Minimap should be visible");
}

// ============================================================================
// Dialogue Flow (4 tests)
// ============================================================================

#[test]
fn test_hud_manager_start_dialogue_sets_state() {
    let mut hud = HudManager::new();

    let dialogue = DialogueNode {
        id: 1,
        speaker_name: "NPC Merchant".to_string(),
        text: "Welcome to my shop!".to_string(),
        choices: vec![DialogueChoice {
            id: 1,
            text: "Show me your wares".to_string(),
            next_node: Some(2),
        }],
        portrait_id: None,
    };

    hud.start_dialogue(dialogue);

    assert!(
        hud.state().show_dialogue,
        "show_dialogue flag should be true"
    );
    assert!(
        hud.active_dialogue.is_some(),
        "active_dialogue should be populated"
    );
    assert_eq!(
        hud.active_dialogue.as_ref().unwrap().speaker_name,
        "NPC Merchant"
    );
}

#[test]
fn test_hud_manager_end_dialogue_clears_state() {
    let mut hud = HudManager::new();

    // Start dialogue first
    let dialogue = DialogueNode {
        id: 1,
        speaker_name: "Guard".to_string(),
        text: "Halt!".to_string(),
        choices: vec![],
        portrait_id: None,
    };
    hud.start_dialogue(dialogue);

    // End dialogue
    hud.end_dialogue();

    assert!(
        !hud.state().show_dialogue,
        "show_dialogue flag should be false"
    );
    assert!(
        hud.active_dialogue.is_none(),
        "active_dialogue should be None"
    );
}

#[test]
fn test_hud_manager_select_dialogue_choice_returns_next() {
    let mut hud = HudManager::new();

    let dialogue = DialogueNode {
        id: 1,
        speaker_name: "Quest Giver".to_string(),
        text: "Can you help me?".to_string(),
        choices: vec![
            DialogueChoice {
                id: 1,
                text: "Yes, I'll help!".to_string(),
                next_node: Some(2),
            },
            DialogueChoice {
                id: 2,
                text: "No, I'm busy".to_string(),
                next_node: Some(3),
            },
        ],
        portrait_id: None,
    };
    hud.start_dialogue(dialogue);

    // Select first choice
    let next_node = hud.select_dialogue_choice(1);
    assert_eq!(next_node, Some(2), "Should return next node ID 2");

    // Select second choice
    let next_node = hud.select_dialogue_choice(2);
    assert_eq!(next_node, Some(3), "Should return next node ID 3");
}

#[test]
fn test_hud_manager_select_invalid_choice_returns_none() {
    let mut hud = HudManager::new();

    let dialogue = DialogueNode {
        id: 1,
        speaker_name: "NPC".to_string(),
        text: "Hello".to_string(),
        choices: vec![DialogueChoice {
            id: 1,
            text: "Goodbye".to_string(),
            next_node: None,
        }],
        portrait_id: None,
    };
    hud.start_dialogue(dialogue);

    // Select invalid choice ID
    let next_node = hud.select_dialogue_choice(999);
    assert_eq!(next_node, None, "Invalid choice should return None");
}

// ============================================================================
// Tooltips (2 tests)
// ============================================================================

#[test]
fn test_hud_manager_show_tooltip_sets_data() {
    let mut hud = HudManager::new();

    let tooltip = TooltipData {
        title: "Iron Sword".to_string(),
        description: "A sturdy blade".to_string(),
        stats: vec![
            ("Damage".to_string(), "15".to_string()),
            ("Speed".to_string(), "1.2s".to_string()),
        ],
        flavor_text: Some("Forged in the northern mountains".to_string()),
    };

    hud.show_tooltip(tooltip, (100.0, 200.0));

    assert!(hud.hovered_tooltip.is_some(), "Tooltip should be set");
    assert_eq!(hud.tooltip_position, (100.0, 200.0));
    assert_eq!(hud.hovered_tooltip.as_ref().unwrap().title, "Iron Sword");
}

#[test]
fn test_hud_manager_hide_tooltip_clears_data() {
    let mut hud = HudManager::new();

    // Show tooltip first
    let tooltip = TooltipData {
        title: "Health Potion".to_string(),
        description: "Restores 50 HP".to_string(),
        stats: vec![],
        flavor_text: None,
    };
    hud.show_tooltip(tooltip, (50.0, 50.0));

    // Hide tooltip
    hud.hide_tooltip();

    assert!(
        hud.hovered_tooltip.is_none(),
        "Tooltip should be None after hide"
    );
}

// ============================================================================
// Damage/Pings Spawning (3 tests)
// ============================================================================

#[test]
fn test_hud_manager_spawn_damage_with_combo_tracking() {
    let mut hud = HudManager::new();

    // HudManager starts with game_time = 0.0
    let initial_time = 0.0;

    // Spawn first damage
    hud.spawn_damage(50, (0.0, 1.0, 0.0), DamageType::Normal);

    assert_eq!(hud.damage_numbers.len(), 1, "Should have 1 damage number");
    assert_eq!(
        hud.combo_tracker.get_combo_count(initial_time),
        1,
        "Combo count should be 1"
    );

    // Spawn second damage (within combo window)
    hud.spawn_damage(75, (1.0, 1.0, 0.0), DamageType::Critical);

    assert_eq!(hud.damage_numbers.len(), 2, "Should have 2 damage numbers");
    assert_eq!(
        hud.combo_tracker.get_combo_count(initial_time),
        2,
        "Combo count should be 2"
    );
}

#[test]
fn test_hud_manager_spawn_ping_adds_marker() {
    let mut hud = HudManager::new();

    // Spawn ping
    hud.spawn_ping((10.0, 20.0));

    assert_eq!(hud.ping_markers.len(), 1, "Should have 1 ping marker");
    assert_eq!(hud.ping_markers[0].world_pos, (10.0, 20.0));
    assert_eq!(hud.ping_markers[0].spawn_time, 0.0);
}

#[test]
fn test_hud_manager_update_cleans_old_damage_numbers() {
    let mut hud = HudManager::new();

    // Spawn damage
    hud.spawn_damage(100, (0.0, 0.0, 0.0), DamageType::Normal);
    assert_eq!(hud.damage_numbers.len(), 1);

    // Update time to just before expiry (1.4s < 1.5s lifetime)
    hud.update(1.4);
    assert_eq!(
        hud.damage_numbers.len(),
        1,
        "Damage number should still exist"
    );

    // Update time past expiry (total 1.6s > 1.5s lifetime)
    hud.update(0.2);
    assert_eq!(
        hud.damage_numbers.len(),
        0,
        "Damage number should be cleaned up"
    );
}

// ============================================================================
// HudManager Update Loop (3 tests)
// ============================================================================

#[test]
fn test_hud_manager_update_progresses_animations() {
    let mut hud = HudManager::new();

    // Set player health to trigger animation
    hud.player_stats.health = 50.0;
    let initial_visual = hud.player_stats.health_animation.visual_health();

    // Update should progress animation
    hud.update(0.016); // 16ms frame

    let after_visual = hud.player_stats.health_animation.visual_health();
    assert!(
        after_visual != initial_visual,
        "Animation should progress after update"
    );
}

#[test]
fn test_hud_manager_update_cleans_expired_notifications() {
    let mut hud = HudManager::new();

    // Add notification via notification_queue
    let notif =
        astraweave_ui::hud::QuestNotification::new_quest("Test".to_string(), "Desc".to_string());
    hud.notification_queue.push(notif);

    assert!(
        hud.notification_queue.has_active(),
        "Should have active notification"
    );

    // Update past notification duration (2.5s > 2.0s duration)
    hud.update(2.5);

    assert!(
        !hud.notification_queue.has_active(),
        "Notification should be expired and removed"
    );
}

#[test]
fn test_hud_manager_update_cleans_expired_pings() {
    let mut hud = HudManager::new();

    // Spawn ping
    hud.spawn_ping((5.0, 5.0));
    assert_eq!(hud.ping_markers.len(), 1);

    // Update to just before expiry (2.9s < 3.0s lifetime)
    hud.update(2.9);
    assert_eq!(hud.ping_markers.len(), 1, "Ping should still exist");

    // Update past expiry (total 3.1s > 3.0s lifetime)
    hud.update(0.2);
    assert_eq!(hud.ping_markers.len(), 0, "Ping should be cleaned up");
}

// ============================================================================
// HudState Serialization (2 tests)
// ============================================================================

#[test]
fn test_hud_state_default_values() {
    let state = HudState::default();

    assert!(state.visible, "Should be visible by default");
    assert!(!state.debug_mode, "Debug mode should be off");
    assert!(state.show_health_bars, "Health bars should be shown");
    assert!(state.show_objectives, "Objectives should be shown");
    assert!(state.show_minimap, "Minimap should be shown");
    assert!(state.show_subtitles, "Subtitles should be shown");
    assert!(!state.show_dialogue, "Dialogue should not be shown");
}

#[test]
fn test_hud_state_toggle_flags() {
    let mut hud = HudManager::new();

    // Get initial state
    let initial_state = hud.state().clone();
    assert!(initial_state.visible);
    assert!(initial_state.show_minimap);

    // Create modified state
    let mut modified_state = initial_state.clone();
    modified_state.visible = false;
    modified_state.show_minimap = false;
    modified_state.debug_mode = true;

    // Set modified state
    hud.set_state(modified_state);

    // Verify state was updated
    assert!(!hud.state().visible);
    assert!(!hud.state().show_minimap);
    assert!(hud.state().debug_mode);
}

// ============================================================================
// PoiType Helpers (1 test)
// ============================================================================

#[test]
fn test_poi_type_icon_color_mappings() {
    use astraweave_ui::hud::PoiType;

    // Test all PoiType variants have valid icon/color
    let objective_poi = PoiType::Objective;
    let waypoint_poi = PoiType::Waypoint;
    let vendor_poi = PoiType::Vendor;
    let danger_poi = PoiType::Danger;

    // Verify icon() returns non-empty string for all types
    assert!(
        !objective_poi.icon().is_empty(),
        "Objective icon should not be empty"
    );
    assert!(
        !waypoint_poi.icon().is_empty(),
        "Waypoint icon should not be empty"
    );
    assert!(
        !vendor_poi.icon().is_empty(),
        "Vendor icon should not be empty"
    );
    assert!(
        !danger_poi.icon().is_empty(),
        "Danger icon should not be empty"
    );

    // Verify specific icon mappings
    assert_eq!(
        objective_poi.icon(),
        "🎯",
        "Objective should be target emoji"
    );
    assert_eq!(waypoint_poi.icon(), "📍", "Waypoint should be pin emoji");
    assert_eq!(vendor_poi.icon(), "🏪", "Vendor should be shop emoji");
    assert_eq!(danger_poi.icon(), "⚔️", "Danger should be swords emoji");

    // Verify color() returns valid egui::Color32
    // (Color32 doesn't expose RGB directly, so we test it doesn't panic)
    let _ = objective_poi.color();
    let _ = waypoint_poi.color();
    let _ = vendor_poi.color();
    let _ = danger_poi.color();

    // Verify different types have different colors by checking raw values
    let objective_color = objective_poi.color();
    let waypoint_color = waypoint_poi.color();

    assert_ne!(
        objective_color, waypoint_color,
        "Different PoiType variants should have different colors"
    );
}
