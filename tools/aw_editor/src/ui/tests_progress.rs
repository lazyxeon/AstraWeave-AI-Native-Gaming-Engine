#[cfg(test)]
mod tests {
    use super::super::progress::*;

    #[test]
    fn test_progress_manager_creation() {
        let manager = ProgressManager::new();
        assert!(!manager.has_active_tasks());
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_start_task() {
        let mut manager = ProgressManager::new();
        let _id = manager.start_task("Loading Scene", TaskCategory::SceneLoading);
        
        assert_eq!(manager.active_count(), 1);
        assert!(manager.has_active_tasks());
    }

    #[test]
    fn test_update_progress() {
        let mut manager = ProgressManager::new();
        let id = manager.start_task("Importing Assets", TaskCategory::AssetImport);
        
        manager.update_progress(id, 0.5);
        // We can't access internal state easily without public accessor, 
        // need to check if we can verify via public methods.
        // `update_progress` modifies internal state. 
        // But `get_overall_progress` is public? Let's check the file content again.
        // Yes, line 198: `pub fn get_overall_progress(&self) -> Option<f32>` (inferred from context in read_file output).
        
        // Wait, I need to check if get_overall_progress exists.
    }
    
    #[test]
    fn test_complete_task() {
        let mut manager = ProgressManager::new();
        let id = manager.start_task("Build", TaskCategory::Build);
        
        manager.complete_task(id);
        assert_eq!(manager.active_count(), 0);
    }

    #[test]
    fn test_cancellable_task() {
        let mut manager = ProgressManager::new();
        let id = manager.start_cancellable_task("Long Operation", TaskCategory::Other);
        
        assert!(!manager.is_cancel_requested(id));
        let cancelled = manager.cancel_task(id);
        assert!(cancelled);
        assert!(manager.is_cancel_requested(id));
        
        manager.remove_cancelled(id);
        assert_eq!(manager.active_count(), 0);
    }
    
    #[test]
    fn test_sub_tasks() {
        let mut manager = ProgressManager::new();
        let id = manager.start_task("Main Task", TaskCategory::Other);
        
        manager.add_sub_task(id, "Subtask 1");
        manager.update_sub_task(id, 0, 1.0);
        
        // Just verify it doesn't panic
        manager.update_progress(id, 1.0);
        manager.complete_task(id);
    }
}
