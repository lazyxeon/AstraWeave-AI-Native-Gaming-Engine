// tools/aw_editor/src/ui/toast.rs - Week 6 Day 3-4: Enhanced Toast Notifications
//!
//! Enhanced toast notification system with:
//! - Multiple toast stacking with proper spacing
//! - Click to dismiss functionality
//! - Action buttons (Undo, View Details, Retry)
//! - Smooth slide-in/slide-out animations
//! - Toast grouping by type to prevent spam

use egui::{Area, Color32, Context, Frame, Id, Order, Pos2, RichText};
use std::time::{Duration, Instant};

/// Toast notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastLevel {
    /// Get the background color for this toast level
    pub fn color(&self) -> Color32 {
        match self {
            ToastLevel::Info => Color32::from_rgb(60, 120, 200),
            ToastLevel::Success => Color32::from_rgb(40, 160, 80),
            ToastLevel::Warning => Color32::from_rgb(200, 140, 40),
            ToastLevel::Error => Color32::from_rgb(200, 60, 60),
        }
    }

    /// Get the icon for this toast level
    pub fn icon(&self) -> &'static str {
        match self {
            ToastLevel::Info => "ℹ️",
            ToastLevel::Success => "✅",
            ToastLevel::Warning => "⚠️",
            ToastLevel::Error => "❌",
        }
    }
}

/// Action that can be taken from a toast
#[derive(Debug, Clone)]
pub enum ToastAction {
    /// Undo the action that triggered this toast
    Undo,
    /// View more details
    ViewDetails(String),
    /// Retry the failed action
    Retry,
    /// Open a file or path
    Open(String),
    /// Custom action with label and callback ID
    Custom { label: String, action_id: String },
}

impl ToastAction {
    /// Get the display label for this action
    pub fn label(&self) -> &str {
        match self {
            ToastAction::Undo => "Undo",
            ToastAction::ViewDetails(_) => "Details",
            ToastAction::Retry => "Retry",
            ToastAction::Open(_) => "Open",
            ToastAction::Custom { label, .. } => label,
        }
    }
}

/// A toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    /// Unique ID for this toast
    pub id: u64,
    /// Main message
    pub message: String,
    /// Severity level
    pub level: ToastLevel,
    /// When the toast was created
    pub created_at: Instant,
    /// Duration before auto-dismiss
    pub duration: Duration,
    /// Optional action buttons
    pub actions: Vec<ToastAction>,
    /// Whether the user has dismissed this toast
    pub dismissed: bool,
    /// Whether this toast is currently hovered (pauses timeout)
    pub hovered: bool,
    /// Group key for deduplication (if set, only one toast per group)
    pub group_key: Option<String>,
}

impl Toast {
    /// Create a new toast with default duration (4 seconds)
    pub fn new(message: impl Into<String>, level: ToastLevel) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            message: message.into(),
            level,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
            actions: Vec::new(),
            dismissed: false,
            hovered: false,
            group_key: None,
        }
    }

    /// Set the duration for this toast
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add an action button
    pub fn with_action(mut self, action: ToastAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Set a group key for deduplication
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group_key = Some(group.into());
        self
    }

    /// Check if this toast should be removed
    pub fn should_remove(&self) -> bool {
        if self.dismissed {
            return true;
        }
        // Only timeout if not hovered
        if !self.hovered {
            self.created_at.elapsed() > self.duration
        } else {
            false
        }
    }

    /// Get the age of this toast in seconds
    pub fn age(&self) -> f32 {
        self.created_at.elapsed().as_secs_f32()
    }

    /// Calculate animation progress for fade in/out
    pub fn animation_progress(&self) -> f32 {
        let age = self.age();
        let duration = self.duration.as_secs_f32();

        // Slide in animation (first 0.3 seconds)
        if age < 0.3 {
            age / 0.3
        }
        // Fade out animation (last 0.5 seconds)
        else if age > duration - 0.5 && !self.hovered {
            (duration - age) / 0.5
        }
        // Fully visible
        else {
            1.0
        }
    }
}

/// Toast notification manager
///
/// Handles displaying, stacking, and removing toast notifications.
pub struct ToastManager {
    /// Active toasts
    toasts: Vec<Toast>,
    /// Width of toast notifications
    toast_width: f32,
    /// Padding from screen edge
    padding: f32,
    /// Vertical spacing between toasts
    spacing: f32,
    /// Maximum number of visible toasts
    max_visible: usize,
    /// Triggered actions to be processed by the editor
    pending_actions: Vec<(u64, ToastAction)>,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    /// Create a new toast manager
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            toast_width: 350.0,
            padding: 12.0,
            spacing: 8.0,
            max_visible: 5,
            pending_actions: Vec::new(),
        }
    }

    /// Get the number of active toasts
    pub fn active_count(&self) -> usize {
        self.toasts.len()
    }

    /// Add a simple toast
    pub fn toast(&mut self, message: impl Into<String>, level: ToastLevel) {
        self.add(Toast::new(message, level));
    }

    /// Add a toast
    pub fn add(&mut self, toast: Toast) {
        // Check for group deduplication
        if let Some(ref group) = toast.group_key {
            // Remove existing toast with same group
            self.toasts.retain(|t| t.group_key.as_ref() != Some(group));
        }

        self.toasts.push(toast);
    }

    /// Add a success toast
    pub fn success(&mut self, message: impl Into<String>) {
        self.toast(message, ToastLevel::Success);
    }

    /// Add an error toast
    pub fn error(&mut self, message: impl Into<String>) {
        self.toast(message, ToastLevel::Error);
    }

    /// Add an info toast
    pub fn info(&mut self, message: impl Into<String>) {
        self.toast(message, ToastLevel::Info);
    }

    /// Add a warning toast
    pub fn warning(&mut self, message: impl Into<String>) {
        self.toast(message, ToastLevel::Warning);
    }

    /// Add an error toast with retry action
    pub fn error_with_retry(&mut self, message: impl Into<String>) {
        self.add(
            Toast::new(message, ToastLevel::Error)
                .with_action(ToastAction::Retry)
                .with_duration(Duration::from_secs(8)),
        );
    }

    /// Add a success toast with undo action
    pub fn success_with_undo(&mut self, message: impl Into<String>) {
        self.add(
            Toast::new(message, ToastLevel::Success)
                .with_action(ToastAction::Undo)
                .with_duration(Duration::from_secs(6)),
        );
    }

    /// Get pending actions to process
    pub fn take_pending_actions(&mut self) -> Vec<(u64, ToastAction)> {
        std::mem::take(&mut self.pending_actions)
    }

    /// Check if there are active toasts
    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    /// Get the number of active toasts
    pub fn count(&self) -> usize {
        self.toasts.len()
    }

    /// Clear all toasts
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Update and render toasts
    ///
    /// Call this every frame to update and render toasts.
    pub fn show(&mut self, ctx: &Context) {
        // Remove expired toasts
        self.toasts.retain(|t| !t.should_remove());

        if self.toasts.is_empty() {
            return;
        }

        let screen_rect = ctx.screen_rect();
        let mut y_offset = screen_rect.max.y - self.padding;

        // Track dismissals and actions
        let mut toasts_to_dismiss: Vec<u64> = Vec::new();
        let mut hover_updates: Vec<(u64, bool)> = Vec::new();

        // Show most recent toasts at the bottom, iterate in reverse
        let _visible_count = self.toasts.len().min(self.max_visible);
        let start_index = self.toasts.len().saturating_sub(self.max_visible);

        for (_i, toast) in self.toasts[start_index..].iter().enumerate().rev() {
            let toast_id = toast.id;
            let progress = toast.animation_progress();
            
            // Calculate toast height
            let toast_height = if toast.actions.is_empty() { 50.0 } else { 85.0 };
            
            // Slide animation offset
            let slide_offset = (1.0 - progress) * 50.0;
            
            let toast_pos = Pos2::new(
                screen_rect.max.x - self.toast_width - self.padding + slide_offset,
                y_offset - toast_height,
            );

            let bg_color = toast.level.color();
            let alpha_byte = (220.0 * progress) as u8;
            let frame_color = Color32::from_rgba_unmultiplied(
                bg_color.r(),
                bg_color.g(),
                bg_color.b(),
                alpha_byte,
            );

            let response = Area::new(Id::new(("toast", toast_id)))
                .order(Order::Foreground)
                .fixed_pos(toast_pos)
                .show(ctx, |ui| {
                    Frame::NONE
                        .fill(frame_color)
                        .corner_radius(8.0)
                        .inner_margin(12.0)
                        .shadow(egui::epaint::Shadow {
                            spread: 2,
                            blur: 8,
                            color: Color32::from_rgba_unmultiplied(0, 0, 0, (60.0 * progress) as u8),
                            offset: [2, 2],
                        })
                        .show(ui, |ui| {
                            ui.set_width(self.toast_width - 24.0);
                            
                            let text_alpha = (255.0 * progress) as u8;
                            let text_color = Color32::from_rgba_unmultiplied(255, 255, 255, text_alpha);
                            
                            // Header row with icon, message, and close button
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(toast.level.icon()).size(16.0));
                                ui.add_space(4.0);
                                
                                // Message (takes remaining space)
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true), |ui| {
                                    ui.set_width(self.toast_width - 80.0);
                                    ui.label(RichText::new(&toast.message).color(text_color).size(14.0));
                                });
                                
                                // Close button
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("✕").clicked() {
                                        toasts_to_dismiss.push(toast_id);
                                    }
                                });
                            });
                            
                            // Action buttons row
                            if !toast.actions.is_empty() {
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    for action in &toast.actions {
                                        if ui.small_button(action.label()).clicked() {
                                            self.pending_actions.push((toast_id, action.clone()));
                                            toasts_to_dismiss.push(toast_id);
                                        }
                                        ui.add_space(4.0);
                                    }
                                    
                                    // Progress bar showing remaining time
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let progress_frac = 1.0 - (toast.age() / toast.duration.as_secs_f32()).min(1.0);
                                        ui.add(
                                            egui::ProgressBar::new(progress_frac)
                                                .desired_width(60.0)
                                                .fill(Color32::from_rgba_unmultiplied(255, 255, 255, (100.0 * progress) as u8))
                                        );
                                    });
                                });
                            }
                        });
                })
                .response;

            // Track hover state
            hover_updates.push((toast_id, response.hovered()));
            
            // Click to dismiss (anywhere on toast)
            if response.clicked() {
                toasts_to_dismiss.push(toast_id);
            }

            y_offset -= toast_height + self.spacing;
        }

        // Apply hover updates
        for (id, hovered) in hover_updates {
            if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
                toast.hovered = hovered;
            }
        }

        // Process dismissals
        for id in toasts_to_dismiss {
            if let Some(toast) = self.toasts.iter_mut().find(|t| t.id == id) {
                toast.dismissed = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_new() {
        let toast = Toast::new("Test message", ToastLevel::Info);
        assert_eq!(toast.message, "Test message");
        assert_eq!(toast.level, ToastLevel::Info);
        assert!(!toast.dismissed);
        assert!(!toast.hovered);
    }

    #[test]
    fn test_toast_with_action() {
        let toast = Toast::new("Deleted", ToastLevel::Success)
            .with_action(ToastAction::Undo);
        assert_eq!(toast.actions.len(), 1);
    }

    #[test]
    fn test_toast_with_duration() {
        let toast = Toast::new("Test", ToastLevel::Info)
            .with_duration(Duration::from_secs(10));
        assert_eq!(toast.duration, Duration::from_secs(10));
    }

    #[test]
    fn test_toast_with_group() {
        let toast = Toast::new("Test", ToastLevel::Info)
            .with_group("save-status");
        assert_eq!(toast.group_key, Some("save-status".to_string()));
    }

    #[test]
    fn test_toast_level_color() {
        assert_ne!(ToastLevel::Info.color(), ToastLevel::Error.color());
        assert_ne!(ToastLevel::Success.color(), ToastLevel::Warning.color());
    }

    #[test]
    fn test_toast_level_icon() {
        assert_eq!(ToastLevel::Info.icon(), "ℹ️");
        assert_eq!(ToastLevel::Success.icon(), "✅");
        assert_eq!(ToastLevel::Warning.icon(), "⚠️");
        assert_eq!(ToastLevel::Error.icon(), "❌");
    }

    #[test]
    fn test_toast_manager_new() {
        let manager = ToastManager::new();
        assert!(!manager.has_toasts());
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_toast_manager_add() {
        let mut manager = ToastManager::new();
        manager.success("Test success");
        assert!(manager.has_toasts());
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_toast_manager_group_dedup() {
        let mut manager = ToastManager::new();
        manager.add(Toast::new("First", ToastLevel::Info).with_group("status"));
        manager.add(Toast::new("Second", ToastLevel::Info).with_group("status"));
        // Group should replace, so still 1 toast
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.toasts[0].message, "Second");
    }

    #[test]
    fn test_toast_manager_clear() {
        let mut manager = ToastManager::new();
        manager.success("Test 1");
        manager.error("Test 2");
        manager.info("Test 3");
        assert_eq!(manager.count(), 3);
        manager.clear();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_toast_action_label() {
        assert_eq!(ToastAction::Undo.label(), "Undo");
        assert_eq!(ToastAction::Retry.label(), "Retry");
        assert_eq!(ToastAction::ViewDetails("x".to_string()).label(), "Details");
        assert_eq!(ToastAction::Open("x".to_string()).label(), "Open");
        assert_eq!(
            ToastAction::Custom {
                label: "Custom".to_string(),
                action_id: "id".to_string()
            }
            .label(),
            "Custom"
        );
    }

    #[test]
    fn test_toast_should_remove_dismissed() {
        let mut toast = Toast::new("Test", ToastLevel::Info);
        assert!(!toast.should_remove());
        toast.dismissed = true;
        assert!(toast.should_remove());
    }

    #[test]
    fn test_toast_animation_progress() {
        let toast = Toast::new("Test", ToastLevel::Info);
        // Very new toast should have progress near 0
        let progress = toast.animation_progress();
        assert!((0.0..=1.0).contains(&progress));
    }

    #[test]
    fn test_toast_manager_helpers() {
        let mut manager = ToastManager::new();
        manager.success_with_undo("Deleted entity");
        assert_eq!(manager.count(), 1);
        assert_eq!(manager.toasts[0].actions.len(), 1);

        manager.error_with_retry("Build failed");
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_toast_helpers_explicit() {
        let mut manager = ToastManager::new();
        manager.info("Info");
        manager.warning("Warning");
        manager.error("Error");
        
        assert_eq!(manager.count(), 3);
        assert_eq!(manager.toasts[0].level, ToastLevel::Info);
        assert_eq!(manager.toasts[1].level, ToastLevel::Warning);
        assert_eq!(manager.toasts[2].level, ToastLevel::Error);
    }
    
    #[test]
    fn test_take_pending_actions() {
        let mut manager = ToastManager::new();
        // Since pending_actions are private and mostly set via show(), we might not be able to populate them easily without running show()
        // But we can check it returns empty initially
        let actions = manager.take_pending_actions();
        assert!(actions.is_empty());
    }

    #[test]
    fn test_toast_unique_ids() {
        let t1 = Toast::new("First", ToastLevel::Info);
        let t2 = Toast::new("Second", ToastLevel::Info);
        assert_ne!(t1.id, t2.id);
        assert!(t2.id > t1.id);
    }

    #[test]
    fn test_toast_builder_methods() {
        let t = Toast::new("Msg", ToastLevel::Info)
            .with_duration(Duration::from_secs(10))
            .with_group("g1");
            
        assert_eq!(t.duration, Duration::from_secs(10));
        assert_eq!(t.group_key, Some("g1".to_string()));
    }
    
    #[test]
    fn test_toast_display_helpers() {
        let mut manager = ToastManager::new();
        // Just verify show doesn't panic on empty
        // We can't easily mock Context here without more setup, so we skip show() tests
        // But we can test checking for empty
        assert!(!manager.has_toasts());
        manager.success("msg");
        assert!(manager.has_toasts());
    }
}
