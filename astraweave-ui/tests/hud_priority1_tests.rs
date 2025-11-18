/// Priority 1 Tests: Core HUD Logic (Physics, Animations, Quest, Combo, Notifications, Pings)
///
/// Sprint: Phase 8.6 UI Testing
/// Day 1-2: 25 tests for core HUD functionality
use astraweave_ui::hud::{
    ComboTracker, DamageNumber, DamageType, NotificationQueue, PingMarker, QuestNotification,
};

// ============================================================================
// DamageNumber Physics Tests (2 tests)
// ============================================================================

#[test]
fn test_damage_number_calculate_offset_arc_motion() {
    let damage = DamageNumber::new(50, 0.0, (0.0, 1.0, 0.0), DamageType::Normal);

    // Test parabolic arc at different time points
    let (offset_x_t0, offset_y_t0) = damage.calculate_offset(0.0);
    assert_eq!(offset_x_t0, 0.0, "At t=0, horizontal offset should be 0");
    assert_eq!(offset_y_t0, 0.0, "At t=0, vertical offset should be 0");

    // After 0.5 seconds
    let age = 0.5;
    let (offset_x, offset_y) = damage.calculate_offset(age);

    // Verify horizontal motion: x(t) = vx * t
    let expected_x = damage.velocity_x * age;
    assert_float_eq(offset_x, expected_x, 0.01);

    // Verify vertical motion (parabolic): y(t) = vy*t + 0.5*g*t²
    // vy = -80 (negative = up), g = 150 (positive = down)
    // At t=0.5: y = -80*0.5 + 0.5*150*0.25 = -40 + 18.75 = -21.25 (still moving up)
    let expected_y = damage.velocity_y * age + 0.5 * damage.gravity * age * age;
    assert_float_eq(offset_y, expected_y, 0.01);

    // At early time (0.5s), upward velocity dominates, so offset_y should be negative (up)
    assert!(offset_y < 0.0, "At t=0.5s, initial upward velocity (-80) dominates gravity effect, so offset_y should be negative (moving up)");

    // Test gravity effect at later time (1.0s)
    // At t=1.0: y = -80*1.0 + 0.5*150*1.0 = -80 + 75 = -5 (still slightly up)
    // At t=1.2: y = -80*1.2 + 0.5*150*1.44 = -96 + 108 = +12 (now moving down)
    let age_late = 1.2;
    let (_, offset_y_late) = damage.calculate_offset(age_late);
    assert!(
        offset_y_late > 0.0,
        "At t=1.2s, gravity dominates and damage number moves down (positive offset)"
    );
}

#[test]
fn test_damage_number_calculate_shake_rotation() {
    let damage = DamageNumber::new(50, 0.0, (0.0, 1.0, 0.0), DamageType::Critical);

    // At t=0, shake should be minimal (but not zero due to sin(0) = 0)
    let rotation_t0 = damage.calculate_shake(0.0);
    assert_float_eq(rotation_t0, 0.0, 0.001);

    // After 0.1 seconds
    let age = 0.1;
    let rotation = damage.calculate_shake(age);

    // Verify damped oscillation formula: amplitude * sin(t * freq * TAU) * e^(-t*5)
    let expected_damping = (-age * 5.0).exp();
    let expected_rotation = damage.shake_amplitude
        * (age * damage.shake_frequency * std::f32::consts::TAU).sin()
        * expected_damping;
    assert_float_eq(rotation, expected_rotation, 0.001);

    // Verify shake decays over time
    let rotation_early = damage.calculate_shake(0.1);
    let rotation_late = damage.calculate_shake(1.0);
    assert!(
        rotation_late.abs() < rotation_early.abs(),
        "Shake should decay over time due to exponential damping"
    );
}

// ============================================================================
// ComboTracker Tests (5 tests)
// ============================================================================

#[test]
fn test_combo_tracker_record_hit_increments_count() {
    let mut tracker = ComboTracker::new();
    let game_time = 1.0;

    // Record first hit
    tracker.record_hit(game_time, 50);
    assert_eq!(
        tracker.get_combo_count(game_time),
        1,
        "After one hit, combo count should be 1"
    );

    // Record second hit
    tracker.record_hit(game_time + 0.5, 75);
    assert_eq!(
        tracker.get_combo_count(game_time + 0.5),
        2,
        "After two hits, combo count should be 2"
    );
}

#[test]
fn test_combo_tracker_get_combo_count() {
    let mut tracker = ComboTracker::new();
    let game_time = 5.0;

    // Add multiple hits
    tracker.record_hit(game_time, 50);
    tracker.record_hit(game_time + 0.3, 75);
    tracker.record_hit(game_time + 0.6, 100);

    assert_eq!(
        tracker.get_combo_count(game_time + 0.6),
        3,
        "Should have 3 hits in combo"
    );
}

#[test]
fn test_combo_tracker_get_combo_damage() {
    let mut tracker = ComboTracker::new();
    let game_time = 10.0;

    // Add hits with known damage values
    tracker.record_hit(game_time, 50);
    tracker.record_hit(game_time + 0.2, 75);
    tracker.record_hit(game_time + 0.4, 100);

    let total_damage = tracker.get_combo_damage(game_time + 0.4);
    assert_eq!(
        total_damage, 225,
        "Total combo damage should be 50 + 75 + 100 = 225"
    );
}

#[test]
fn test_combo_tracker_cleanup_expired() {
    let mut tracker = ComboTracker::new();
    let game_time = 20.0;

    // Add hit at t=20.0
    tracker.record_hit(game_time, 50);

    // Verify hit is active
    assert_eq!(tracker.get_combo_count(game_time), 1);

    // Advance time past combo window (1.5 seconds later)
    tracker.cleanup(game_time + 1.5);

    // Hit should be cleaned up
    assert_eq!(
        tracker.get_combo_count(game_time + 1.5),
        0,
        "After cleanup past combo window, count should be 0"
    );
}

#[test]
fn test_combo_tracker_window_expiry() {
    let mut tracker = ComboTracker::new();
    let game_time = 30.0;

    // Add hit at t=30.0
    tracker.record_hit(game_time, 50);

    // Within window (0.5s later)
    assert_eq!(
        tracker.get_combo_count(game_time + 0.5),
        1,
        "Hit should still be in combo window after 0.5s"
    );

    // Just outside window (1.1s later, window is 1.0s)
    assert_eq!(
        tracker.get_combo_count(game_time + 1.1),
        0,
        "Hit should be outside combo window after 1.1s (window is 1.0s)"
    );
}

// ============================================================================
// QuestNotification Tests (6 tests)
// ============================================================================

#[test]
fn test_quest_notification_new_quest() {
    let notif = QuestNotification::new_quest(
        "The Lost Artifact".to_string(),
        "Find the ancient relic".to_string(),
    );

    assert_eq!(notif.title, "The Lost Artifact");
    assert_eq!(notif.description, "Find the ancient relic");
    assert_eq!(notif.animation_time, 0.0);
    assert_eq!(notif.total_duration, 2.0, "New quest duration is 2.0s");
}

#[test]
fn test_quest_notification_objective_complete() {
    let notif = QuestNotification::objective_complete("Collect 5 herbs".to_string());

    assert_eq!(notif.title, "Objective Complete!");
    assert_eq!(notif.description, "Collect 5 herbs");
    assert_eq!(notif.animation_time, 0.0);
    assert_eq!(
        notif.total_duration, 2.0,
        "Objective complete duration is 2.0s"
    );
}

#[test]
fn test_quest_notification_quest_complete() {
    let rewards = vec!["100 Gold".to_string(), "Iron Sword".to_string()];
    let notif = QuestNotification::quest_complete("The Lost Artifact".to_string(), rewards);

    assert_eq!(notif.title, "The Lost Artifact");
    assert_eq!(notif.description, "Quest Complete!");
    assert_eq!(notif.animation_time, 0.0);
    assert_eq!(
        notif.total_duration, 2.8,
        "Quest complete duration is 2.8s (longer for rewards)"
    );
}

#[test]
fn test_quest_notification_update_aging() {
    let mut notif = QuestNotification::new_quest("Test".to_string(), "Desc".to_string());

    // Initially not finished
    let finished = notif.update(0.5);
    assert!(!finished, "After 0.5s, notification should not be finished");
    assert_eq!(notif.animation_time, 0.5);

    // Advance to finish
    let finished = notif.update(1.6); // Total 2.1s > 2.0s duration
    assert!(
        finished,
        "After 2.1s total, notification should be finished"
    );
}

#[test]
fn test_quest_notification_calculate_slide_offset_phases() {
    let mut notif = QuestNotification::new_quest("Test".to_string(), "Desc".to_string());

    // Ease-in phase (0-0.3s): sliding down from -100 to 0
    notif.animation_time = 0.15;
    let offset = notif.calculate_slide_offset();
    assert!(
        offset < 0.0 && offset > -100.0,
        "During ease-in, offset should be between -100 and 0"
    );

    // Hold phase (0.3-1.7s): on-screen at 0
    notif.animation_time = 1.0;
    let offset = notif.calculate_slide_offset();
    assert_float_eq(offset, 0.0, 0.01);

    // Ease-out phase (1.7-2.0s): sliding up from 0 to -100
    notif.animation_time = 1.9;
    let offset = notif.calculate_slide_offset();
    assert!(
        offset < 0.0,
        "During ease-out, offset should be negative (sliding up)"
    );
}

#[test]
fn test_quest_notification_calculate_alpha_fade() {
    let mut notif = QuestNotification::new_quest("Test".to_string(), "Desc".to_string());

    // Fade-in phase (0-0.2s)
    notif.animation_time = 0.1;
    let alpha = notif.calculate_alpha();
    assert!(alpha < 255, "During fade-in, alpha should be less than 255");

    // Fully visible phase (0.2-1.7s)
    notif.animation_time = 1.0;
    let alpha = notif.calculate_alpha();
    assert_eq!(alpha, 255, "During hold phase, alpha should be 255");

    // Fade-out phase (1.7-2.0s)
    notif.animation_time = 1.9;
    let alpha = notif.calculate_alpha();
    assert!(
        alpha < 255,
        "During fade-out, alpha should be less than 255"
    );
}

// ============================================================================
// NotificationQueue Tests (3 tests)
// ============================================================================

#[test]
fn test_notification_queue_push() {
    let mut queue = NotificationQueue::new();

    // Push first notification
    let notif1 = QuestNotification::new_quest("Quest 1".to_string(), "Desc 1".to_string());
    queue.push(notif1);

    assert!(queue.active.is_some(), "Active notification should be set");
    assert_eq!(queue.pending.len(), 0, "No pending notifications yet");

    // Push second notification
    let notif2 = QuestNotification::new_quest("Quest 2".to_string(), "Desc 2".to_string());
    queue.push(notif2);

    assert_eq!(
        queue.pending.len(),
        1,
        "Second notification should be pending"
    );
}

#[test]
fn test_notification_queue_update_removes_expired() {
    let mut queue = NotificationQueue::new();

    // Add notification with 2.0s duration
    let notif = QuestNotification::new_quest("Test".to_string(), "Desc".to_string());
    queue.push(notif);

    // Update past duration
    queue.update(2.5);

    assert!(
        queue.active.is_none(),
        "After expiry, active notification should be None"
    );
}

#[test]
fn test_notification_queue_has_active() {
    let mut queue = NotificationQueue::new();

    // Initially no active
    assert!(
        !queue.has_active(),
        "Initially should have no active notification"
    );

    // Add notification
    let notif = QuestNotification::new_quest("Test".to_string(), "Desc".to_string());
    queue.push(notif);

    assert!(queue.has_active(), "Should have active notification");
}

// ============================================================================
// PingMarker Tests (3 tests)
// ============================================================================

#[test]
fn test_ping_marker_new_active() {
    let ping = PingMarker::new((10.0, 20.0), 5.0);

    assert_eq!(ping.spawn_time, 5.0);
    assert_eq!(ping.world_pos, (10.0, 20.0));
    assert!(ping.is_active(5.0), "Newly created ping should be active");
}

#[test]
fn test_ping_marker_is_active_lifetime() {
    let ping = PingMarker::new((0.0, 0.0), 10.0);

    // Active within lifetime (3.0s default duration)
    assert!(
        ping.is_active(10.5),
        "Ping should be active at 0.5s after spawn"
    );
    assert!(
        ping.is_active(12.9),
        "Ping should be active at 2.9s after spawn"
    );

    // Expired after lifetime
    assert!(
        !ping.is_active(13.1),
        "Ping should be inactive after 3.0s lifetime"
    );
}

#[test]
fn test_ping_marker_age_normalized() {
    let ping = PingMarker::new((5.0, 5.0), 15.0);

    // At spawn time (age = 0.0)
    let age_0 = ping.age_normalized(15.0);
    assert_float_eq(age_0, 0.0, 0.001);

    // At half lifetime (age = 1.5, duration = 3.0)
    let age_half = ping.age_normalized(16.5);
    assert_float_eq(age_half, 0.5, 0.01);

    // At end of lifetime (age = 3.0)
    let age_end = ping.age_normalized(18.0);
    assert_float_eq(age_end, 1.0, 0.01);

    // Past lifetime (clamped to 1.0)
    let age_past = ping.age_normalized(20.0);
    assert_float_eq(age_past, 1.0, 0.01);
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Float comparison with epsilon tolerance
fn assert_float_eq(a: f32, b: f32, epsilon: f32) {
    assert!(
        (a - b).abs() < epsilon,
        "Expected {}, got {} (diff {} > epsilon {})",
        a,
        b,
        (a - b).abs(),
        epsilon
    );
}
