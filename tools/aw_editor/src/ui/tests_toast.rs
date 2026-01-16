#[cfg(test)]
mod tests {
    use super::super::toast::*;
    use std::time::Duration;

    #[test]
    fn test_toast_levels() {
        assert_eq!(ToastLevel::Info.icon(), "ℹ️");
        assert_eq!(ToastLevel::Success.icon(), "✅");
        assert_eq!(ToastLevel::Warning.icon(), "⚠️");
        assert_eq!(ToastLevel::Error.icon(), "❌");
    }

    #[test]
    fn test_toast_actions() {
        let action = ToastAction::Undo;
        assert_eq!(action.label(), "Undo");
        
        let action = ToastAction::Retry;
        assert_eq!(action.label(), "Retry");
        
        let action = ToastAction::ViewDetails("log.txt".into());
        assert_eq!(action.label(), "Details");
    }

    #[test]
    fn test_toast_creation() {
        let toast = Toast::new("Test Message", ToastLevel::Info);
        assert_eq!(toast.message, "Test Message");
        assert_eq!(toast.level, ToastLevel::Info);
        assert_eq!(toast.duration.as_secs(), 4);
    }

    #[test]
    fn test_toast_builder_methods() {
        let toast = Toast::new("Test", ToastLevel::Info)
            .with_duration(Duration::from_secs(10))
            .with_group("group1")
            .with_action(ToastAction::Retry);
            
        assert_eq!(toast.duration.as_secs(), 10);
        assert_eq!(toast.group_key, Some("group1".to_string()));
        assert_eq!(toast.actions.len(), 1);
    }

    #[test]
    fn test_toast_should_remove() {
        let toast = Toast::new("Test", ToastLevel::Info)
            .with_duration(Duration::from_secs(0)); // Expired immediately
        
        std::thread::sleep(Duration::from_millis(10));
        assert!(toast.should_remove());
        
        let mut toast_hovered = Toast::new("Test", ToastLevel::Info)
            .with_duration(Duration::from_secs(0));
        toast_hovered.hovered = true;
        assert!(!toast_hovered.should_remove());
        
        let mut toast_dismissed = Toast::new("Test", ToastLevel::Info);
        toast_dismissed.dismissed = true;
        assert!(toast_dismissed.should_remove());
    }

    #[test]
    fn test_toast_manager_deduplication() {
        let mut manager = ToastManager::new();
        
        manager.add(Toast::new("T1", ToastLevel::Info).with_group("g1"));
        assert_eq!(manager.active_count(), 1);
        
        manager.add(Toast::new("T2", ToastLevel::Info).with_group("g1"));
        assert_eq!(manager.active_count(), 1); // Should replace previous
        
        manager.add(Toast::new("T3", ToastLevel::Info).with_group("g2"));
        assert_eq!(manager.active_count(), 2);
    }
}
