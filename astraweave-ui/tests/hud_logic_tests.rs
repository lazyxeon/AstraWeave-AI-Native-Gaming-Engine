use astraweave_ui::hud::*;
use astraweave_ui::hud::easing::*;

mod fixtures;
use fixtures::*;

// ============================================================================
// 1. Easing Functions
// ============================================================================

#[test]
fn test_ease_out_cubic() {
    // t=0 -> 0
    assert_float_eq(ease_out_cubic(0.0), 0.0, 0.001);
    // t=1 -> 1
    assert_float_eq(ease_out_cubic(1.0), 1.0, 0.001);
    // t=0.5 -> 0.875 (fast start, slow end)
    // (0.5-1)^3 + 1 = (-0.5)^3 + 1 = -0.125 + 1 = 0.875
    assert_float_eq(ease_out_cubic(0.5), 0.875, 0.001);
}

#[test]
fn test_ease_in_out_quad() {
    // t=0 -> 0
    assert_float_eq(ease_in_out_quad(0.0), 0.0, 0.001);
    // t=1 -> 1
    assert_float_eq(ease_in_out_quad(1.0), 1.0, 0.001);
    // t=0.5 -> 0.5
    assert_float_eq(ease_in_out_quad(0.5), 0.5, 0.001);
    // t=0.25 -> 2 * 0.25^2 = 2 * 0.0625 = 0.125 (slow start)
    assert_float_eq(ease_in_out_quad(0.25), 0.125, 0.001);
    // t=0.75 -> -1 + (4 - 1.5) * 0.75 = -1 + 2.5 * 0.75 = -1 + 1.875 = 0.875 (slow end)
    assert_float_eq(ease_in_out_quad(0.75), 0.875, 0.001);
}

// ============================================================================
// 2. DamageNumber Physics
// ============================================================================

#[test]
fn test_calculate_offset_arc_motion() {
    let dmg = DamageNumber::new(50, 0.0, (0.0, 0.0, 0.0), DamageType::Normal);
    
    // At t=0, offset should be (0,0)
    let (x0, y0) = dmg.calculate_offset(0.0);
    assert_float_eq(x0, 0.0, 0.001);
    assert_float_eq(y0, 0.0, 0.001);

    // At t=0.5
    let (x1, y1) = dmg.calculate_offset(0.5);
    
    // x = vx * t
    // y = vy * t + 0.5 * g * t^2
    // vy = -80, g = 150
    // y = -80 * 0.5 + 0.5 * 150 * 0.25 = -40 + 18.75 = -21.25
    
    assert_float_eq(x1, dmg.velocity_x * 0.5, 0.001);
    assert_float_eq(y1, -21.25, 0.001);
    
    // Verify gravity pulls it down (y increases, but here negative is up so y becomes less negative/more positive)
    // velocity_y = -80.0 (negative = up). gravity = 150.0 (positive = down).
    // So initially moves up (negative y), then gravity pulls it down (positive y).
    
    assert!(y1 < 0.0, "Should be above spawn point at 0.5s");
}

#[test]
fn test_calculate_shake_rotation() {
    let dmg = DamageNumber::new(50, 0.0, (0.0, 0.0, 0.0), DamageType::Critical);
    
    // t=0 -> sin(0) = 0
    assert_float_eq(dmg.calculate_shake(0.0), 0.0, 0.001);
    
    // t=0.1
    let shake = dmg.calculate_shake(0.1);
    assert!(shake.abs() > 0.0);
    
    // Damping check: t=1.0 should be much smaller than t=0.1 (ignoring phase)
    // Actually let's just check it decays
    let shake_late = dmg.calculate_shake(2.0);
    assert!(shake_late.abs() < 0.01);
}

// ============================================================================
// 3. Quest Logic
// ============================================================================

#[test]
fn test_quest_completion_partial() {
    let mut quest = test_quest();
    // 2 objectives. Obj 1: 0/5. Obj 2: Incomplete.
    // Total objectives: 2. Completed: 0.
    
    assert_float_eq(quest.completion(), 0.0, 0.001);
    
    // Complete obj 1
    quest.objectives[0].completed = true;
    assert_float_eq(quest.completion(), 0.5, 0.001);
}

#[test]
fn test_quest_completion_full() {
    let mut quest = test_quest();
    quest.objectives[0].completed = true;
    quest.objectives[1].completed = true;
    assert_float_eq(quest.completion(), 1.0, 0.001);
}

#[test]
fn test_quest_is_complete_true() {
    let mut quest = test_quest();
    quest.objectives[0].completed = true;
    quest.objectives[1].completed = true;
    
    assert!(quest.is_complete());
}

#[test]
fn test_quest_is_complete_false() {
    let mut quest = test_quest();
    quest.objectives[0].completed = true;
    assert!(!quest.is_complete());
}

// ============================================================================
// 4. ComboTracker
// ============================================================================

#[test]
fn test_record_hit_increments_count() {
    let mut tracker = ComboTracker::new();
    tracker.record_hit(0.0, 10);
    assert_eq!(tracker.get_combo_count(0.0), 1);
    
    tracker.record_hit(0.1, 10);
    assert_eq!(tracker.get_combo_count(0.1), 2);
}

#[test]
fn test_get_combo_count() {
    let mut tracker = ComboTracker::new();
    tracker.record_hit(0.0, 10);
    tracker.record_hit(0.5, 10);
    assert_eq!(tracker.get_combo_count(0.5), 2);
}

#[test]
fn test_get_combo_damage() {
    let mut tracker = ComboTracker::new();
    tracker.record_hit(0.0, 10);
    tracker.record_hit(0.5, 20);
    assert_eq!(tracker.get_combo_damage(0.5), 30);
}

#[test]
fn test_cleanup_expired_combos() {
    let mut tracker = ComboTracker::new();
    tracker.record_hit(0.0, 10);
    
    // Window is 1.0s. At 1.1s, it should be expired.
    tracker.cleanup(1.1);
    assert_eq!(tracker.get_combo_count(1.1), 0);
}

#[test]
fn test_combo_window_reset() {
    let mut tracker = ComboTracker::new();
    tracker.record_hit(0.0, 10);
    
    // At 1.1s, first hit expires.
    // If we add a new hit at 1.1s, count should be 1 (only the new hit).
    tracker.record_hit(1.1, 10);
    
    // get_combo_count filters by window, so it should only see the hit at 1.1
    assert_eq!(tracker.get_combo_count(1.1), 1);
}

// ============================================================================
// 5. QuestNotification
// ============================================================================

#[test]
fn test_new_quest_notification() {
    let notif = QuestNotification::new_quest("Title".into(), "Desc".into());
    assert_eq!(notif.title, "Title");
    assert_eq!(notif.description, "Desc");
    assert!(matches!(notif.notification_type, NotificationType::NewQuest));
}

#[test]
fn test_objective_complete_notification() {
    let notif = QuestNotification::objective_complete("Obj".into());
    assert_eq!(notif.title, "Objective Complete!");
    assert_eq!(notif.description, "Obj");
    assert!(matches!(notif.notification_type, NotificationType::ObjectiveComplete { .. }));
}

#[test]
fn test_quest_complete_notification() {
    let notif = QuestNotification::quest_complete("Title".into(), vec!["Reward".into()]);
    assert_eq!(notif.title, "Title");
    assert_eq!(notif.description, "Quest Complete!");
    assert!(matches!(notif.notification_type, NotificationType::QuestComplete { .. }));
}

#[test]
fn test_notification_update_aging() {
    let mut notif = QuestNotification::new_quest("T".into(), "D".into());
    assert_float_eq(notif.animation_time, 0.0, 0.001);
    
    let finished = notif.update(0.5);
    assert_float_eq(notif.animation_time, 0.5, 0.001);
    assert!(!finished);
    
    let finished = notif.update(2.0); // Total 2.5 > 2.0 duration
    assert!(finished);
}

#[test]
fn test_calculate_slide_offset_ease_in() {
    let mut notif = QuestNotification::new_quest("T".into(), "D".into());
    // Ease in is 0.3s.
    // At 0.0, offset should be -100.0 (above screen)
    notif.animation_time = 0.0;
    assert_float_eq(notif.calculate_slide_offset(), -100.0, 0.001);
    
    // At 0.3, offset should be 0.0 (on screen)
    notif.animation_time = 0.3;
    assert_float_eq(notif.calculate_slide_offset(), 0.0, 0.001);
}

#[test]
fn test_calculate_alpha_fade_out() {
    let mut notif = QuestNotification::new_quest("T".into(), "D".into());
    // Duration 2.0. Fade out starts at 1.7 (2.0 - 0.3).
    
    // At 1.0 (middle), alpha 255
    notif.animation_time = 1.0;
    assert_eq!(notif.calculate_alpha(), 255);
    
    // At 1.85 (halfway through fade out), alpha ~127
    notif.animation_time = 1.85;
    let alpha = notif.calculate_alpha();
    assert!(alpha > 100 && alpha < 155);
    
    // At 2.0, alpha 0
    notif.animation_time = 2.0;
    assert_eq!(notif.calculate_alpha(), 0);
}

// ============================================================================
// 6. NotificationQueue
// ============================================================================

#[test]
fn test_push_notification() {
    let mut queue = NotificationQueue::new();
    let n1 = QuestNotification::new_quest("1".into(), "1".into());
    let n2 = QuestNotification::new_quest("2".into(), "2".into());
    
    queue.push(n1);
    assert!(queue.active.is_some());
    assert!(queue.pending.is_empty());
    
    queue.push(n2);
    assert!(queue.active.is_some());
    assert_eq!(queue.pending.len(), 1);
}

#[test]
fn test_update_removes_expired() {
    let mut queue = NotificationQueue::new();
    let n1 = QuestNotification::new_quest("1".into(), "1".into());
    queue.push(n1);
    
    // Update with time > duration
    queue.update(3.0);
    assert!(queue.active.is_none());
}

#[test]
fn test_has_active_notifications() {
    let mut queue = NotificationQueue::new();
    assert!(!queue.has_active());
    
    queue.push(QuestNotification::new_quest("1".into(), "1".into()));
    assert!(queue.has_active());
}

// ============================================================================
// 7. PingMarker
// ============================================================================

#[test]
fn test_ping_new_active() {
    let ping = PingMarker::new((0.0, 0.0), 10.0);
    assert!(ping.is_active(10.0));
    assert!(ping.is_active(12.0));
}

#[test]
fn test_ping_is_active_lifetime() {
    let ping = PingMarker::new((0.0, 0.0), 10.0);
    // Duration is 3.0s
    assert!(ping.is_active(12.9));
    assert!(!ping.is_active(13.1));
}

#[test]
fn test_ping_age_normalized() {
    let ping = PingMarker::new((0.0, 0.0), 10.0);
    // At 10.0, age 0 -> 0.0
    assert_float_eq(ping.age_normalized(10.0), 0.0, 0.001);
    
    // At 11.5, age 1.5 -> 0.5 (since duration 3.0)
    assert_float_eq(ping.age_normalized(11.5), 0.5, 0.001);
    
    // At 13.0, age 3.0 -> 1.0
    assert_float_eq(ping.age_normalized(13.0), 1.0, 0.001);
}
