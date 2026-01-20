// tools/aw_editor/src/ui/progress.rs - Week 6 Day 1-2: Progress Bar System
//
// Provides a centralized progress tracking system for long-running operations
// like scene loading, asset import, build process, and play mode entry.

use egui::Ui;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Unique identifier for a progress task
pub type TaskId = u64;

/// Represents a single progress task
#[derive(Clone, Debug)]
pub struct ProgressTask {
    /// Human-readable label for the task
    pub label: String,
    /// Progress value from 0.0 to 1.0
    pub progress: f32,
    /// Current status message
    pub status: String,
    /// Whether the task can be cancelled
    pub cancellable: bool,
    /// When the task started
    pub started_at: Instant,
    /// Task category for grouping
    pub category: TaskCategory,
    /// Whether cancellation was requested
    pub cancel_requested: bool,
    /// Sub-tasks for hierarchical progress
    pub sub_tasks: Vec<SubTask>,
}

/// Sub-task for detailed progress tracking
#[derive(Clone, Debug)]
pub struct SubTask {
    pub label: String,
    pub progress: f32,
    pub completed: bool,
}

/// Categories of tasks for visual grouping
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskCategory {
    SceneLoading,
    AssetImport,
    Build,
    PlayMode,
    Export,
    Other,
}

impl TaskCategory {
    /// Get all task categories
    pub fn all() -> &'static [TaskCategory] {
        &[
            TaskCategory::SceneLoading,
            TaskCategory::AssetImport,
            TaskCategory::Build,
            TaskCategory::PlayMode,
            TaskCategory::Export,
            TaskCategory::Other,
        ]
    }

    /// Icon for the category
    pub fn icon(&self) -> &'static str {
        match self {
            TaskCategory::SceneLoading => "📂",
            TaskCategory::AssetImport => "📥",
            TaskCategory::Build => "🔨",
            TaskCategory::PlayMode => "▶️",
            TaskCategory::Export => "📤",
            TaskCategory::Other => "⚙️",
        }
    }

    /// Display name for the category
    pub fn name(&self) -> &'static str {
        match self {
            TaskCategory::SceneLoading => "Scene Loading",
            TaskCategory::AssetImport => "Asset Import",
            TaskCategory::Build => "Build",
            TaskCategory::PlayMode => "Play Mode",
            TaskCategory::Export => "Export",
            TaskCategory::Other => "Other",
        }
    }

    /// Color for the progress bar
    pub fn color(&self) -> egui::Color32 {
        match self {
            TaskCategory::SceneLoading => egui::Color32::from_rgb(100, 150, 255),
            TaskCategory::AssetImport => egui::Color32::from_rgb(100, 255, 150),
            TaskCategory::Build => egui::Color32::from_rgb(255, 180, 100),
            TaskCategory::PlayMode => egui::Color32::from_rgb(100, 255, 100),
            TaskCategory::Export => egui::Color32::from_rgb(180, 100, 255),
            TaskCategory::Other => egui::Color32::from_rgb(150, 150, 150),
        }
    }

    /// Check if this category involves file I/O
    pub fn is_io_intensive(&self) -> bool {
        matches!(
            self,
            TaskCategory::SceneLoading
                | TaskCategory::AssetImport
                | TaskCategory::Export
        )
    }

    /// Check if this category can be cancelled safely
    pub fn is_cancellable(&self) -> bool {
        !matches!(self, TaskCategory::PlayMode)
    }
}

impl std::fmt::Display for TaskCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.icon(), self.name())
    }
}

impl Default for ProgressTask {
    fn default() -> Self {
        Self {
            label: String::new(),
            progress: 0.0,
            status: String::new(),
            cancellable: false,
            started_at: Instant::now(),
            category: TaskCategory::Other,
            cancel_requested: false,
            sub_tasks: Vec::new(),
        }
    }
}

/// Manages multiple concurrent progress tasks
pub struct ProgressManager {
    tasks: HashMap<TaskId, ProgressTask>,
    next_id: TaskId,
    /// Completed tasks kept briefly for fade-out animation
    recently_completed: Vec<(TaskId, ProgressTask, Instant)>,
    /// Duration to show completed tasks before removing
    completion_display_duration: Duration,
}

impl Default for ProgressManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
            recently_completed: Vec::new(),
            completion_display_duration: Duration::from_secs(2),
        }
    }

    /// Start a new task and return its ID
    pub fn start_task(&mut self, label: impl Into<String>, category: TaskCategory) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;

        let task = ProgressTask {
            label: label.into(),
            progress: 0.0,
            status: "Starting...".to_string(),
            cancellable: false,
            started_at: Instant::now(),
            category,
            cancel_requested: false,
            sub_tasks: Vec::new(),
        };

        self.tasks.insert(id, task);
        id
    }

    /// Start a cancellable task
    pub fn start_cancellable_task(
        &mut self,
        label: impl Into<String>,
        category: TaskCategory,
    ) -> TaskId {
        let id = self.start_task(label, category);
        if let Some(task) = self.tasks.get_mut(&id) {
            task.cancellable = true;
        }
        id
    }

    /// Update task progress (0.0 to 1.0)
    pub fn update_progress(&mut self, id: TaskId, progress: f32) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.progress = progress.clamp(0.0, 1.0);
        }
    }

    /// Update task status message
    pub fn update_status(&mut self, id: TaskId, status: impl Into<String>) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.status = status.into();
        }
    }

    /// Update both progress and status
    pub fn update(&mut self, id: TaskId, progress: f32, status: impl Into<String>) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.progress = progress.clamp(0.0, 1.0);
            task.status = status.into();
        }
    }

    /// Add a sub-task for detailed progress
    pub fn add_sub_task(&mut self, id: TaskId, label: impl Into<String>) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.sub_tasks.push(SubTask {
                label: label.into(),
                progress: 0.0,
                completed: false,
            });
        }
    }

    /// Update a sub-task's progress
    pub fn update_sub_task(&mut self, id: TaskId, sub_index: usize, progress: f32) {
        if let Some(task) = self.tasks.get_mut(&id) {
            if let Some(sub) = task.sub_tasks.get_mut(sub_index) {
                sub.progress = progress.clamp(0.0, 1.0);
                sub.completed = progress >= 1.0;
            }
        }
    }

    /// Complete a task successfully
    pub fn complete_task(&mut self, id: TaskId) {
        if let Some(mut task) = self.tasks.remove(&id) {
            task.progress = 1.0;
            task.status = "Complete".to_string();
            self.recently_completed.push((id, task, Instant::now()));
        }
    }

    /// Fail a task with an error message
    pub fn fail_task(&mut self, id: TaskId, error: impl Into<String>) {
        if let Some(mut task) = self.tasks.remove(&id) {
            task.status = format!("Failed: {}", error.into());
            self.recently_completed.push((id, task, Instant::now()));
        }
    }

    /// Cancel a task (if cancellable)
    pub fn cancel_task(&mut self, id: TaskId) -> bool {
        if let Some(task) = self.tasks.get_mut(&id) {
            if task.cancellable {
                task.cancel_requested = true;
                return true;
            }
        }
        false
    }

    /// Check if a task has cancellation requested
    pub fn is_cancel_requested(&self, id: TaskId) -> bool {
        self.tasks
            .get(&id)
            .map(|t| t.cancel_requested)
            .unwrap_or(false)
    }

    /// Remove a cancelled task
    pub fn remove_cancelled(&mut self, id: TaskId) {
        if let Some(mut task) = self.tasks.remove(&id) {
            task.status = "Cancelled".to_string();
            self.recently_completed.push((id, task, Instant::now()));
        }
    }

    /// Get active task count
    pub fn active_count(&self) -> usize {
        self.tasks.len()
    }

    /// Check if any tasks are running
    pub fn has_active_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }

    /// Get overall progress (average of all tasks)
    pub fn overall_progress(&self) -> f32 {
        if self.tasks.is_empty() {
            return 1.0;
        }
        let sum: f32 = self.tasks.values().map(|t| t.progress).sum();
        sum / self.tasks.len() as f32
    }

    /// Clean up old completed tasks
    pub fn cleanup(&mut self) {
        let now = Instant::now();
        self.recently_completed
            .retain(|(_, _, completed_at)| now.duration_since(*completed_at) < self.completion_display_duration);
    }

    /// Get iterator over active tasks
    pub fn active_tasks(&self) -> impl Iterator<Item = (&TaskId, &ProgressTask)> {
        self.tasks.iter()
    }

    /// Get iterator over recently completed tasks
    pub fn completed_tasks(&self) -> impl Iterator<Item = &(TaskId, ProgressTask, Instant)> {
        self.recently_completed.iter()
    }

    /// Show progress UI in a panel
    pub fn show(&mut self, ui: &mut Ui) -> Option<TaskId> {
        self.cleanup();

        let mut cancelled_task = None;

        if self.tasks.is_empty() && self.recently_completed.is_empty() {
            ui.label("No active tasks");
            return None;
        }

        // Active tasks
        for (&id, task) in &self.tasks {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(task.category.icon());
                    ui.strong(&task.label);

                    if task.cancellable && !task.cancel_requested {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("❌").clicked() {
                                cancelled_task = Some(id);
                            }
                        });
                    }
                });

                // Progress bar
                let progress_bar = egui::ProgressBar::new(task.progress)
                    .fill(task.category.color())
                    .animate(true);
                ui.add(progress_bar);

                // Status and elapsed time
                ui.horizontal(|ui| {
                    ui.small(&task.status);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let elapsed = task.started_at.elapsed();
                        ui.small(format_duration(elapsed));
                    });
                });

                // Sub-tasks (collapsed by default)
                if !task.sub_tasks.is_empty() {
                    ui.collapsing("Details", |ui| {
                        for sub in &task.sub_tasks {
                            ui.horizontal(|ui| {
                                let icon = if sub.completed { "✅" } else { "⏳" };
                                ui.label(icon);
                                ui.label(&sub.label);
                                ui.add(
                                    egui::ProgressBar::new(sub.progress)
                                        .desired_width(100.0)
                                        .show_percentage(),
                                );
                            });
                        }
                    });
                }
            });
            ui.add_space(4.0);
        }

        // Recently completed tasks (fading out)
        let now = Instant::now();
        for (_, task, completed_at) in &self.recently_completed {
            let age = now.duration_since(*completed_at);
            let alpha = 1.0
                - (age.as_secs_f32() / self.completion_display_duration.as_secs_f32()).clamp(0.0, 1.0);

            if alpha > 0.1 {
                ui.scope(|ui| {
                    ui.visuals_mut().override_text_color =
                        Some(ui.style().visuals.text_color().gamma_multiply(alpha));

                    ui.horizontal(|ui| {
                        ui.label(task.category.icon());
                        ui.label(&task.label);
                        ui.label("—");
                        let status_color = if task.status.starts_with("Failed") {
                            egui::Color32::from_rgb(255, 100, 100)
                        } else if task.status == "Cancelled" {
                            egui::Color32::from_rgb(255, 180, 100)
                        } else {
                            egui::Color32::from_rgb(100, 255, 100)
                        };
                        ui.colored_label(status_color.gamma_multiply(alpha), &task.status);
                    });
                });
            }
        }

        if let Some(id) = cancelled_task {
            self.cancel_task(id);
        }

        cancelled_task
    }

    /// Show compact progress in status bar
    pub fn show_compact(&self, ui: &mut Ui) {
        if self.tasks.is_empty() {
            return;
        }

        let task_count = self.tasks.len();
        let overall = self.overall_progress();

        ui.horizontal(|ui| {
            ui.label(format!("⏳ {} task{}", task_count, if task_count == 1 { "" } else { "s" }));
            ui.add(
                egui::ProgressBar::new(overall)
                    .desired_width(80.0)
                    .show_percentage(),
            );
        });
    }
}

/// Format duration for display
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_manager_new() {
        let pm = ProgressManager::new();
        assert_eq!(pm.active_count(), 0);
        assert!(!pm.has_active_tasks());
    }

    #[test]
    fn test_start_task() {
        let mut pm = ProgressManager::new();
        let id = pm.start_task("Test Task", TaskCategory::Other);
        assert_eq!(pm.active_count(), 1);
        assert!(pm.has_active_tasks());
        assert!(id > 0);
    }

    #[test]
    fn test_update_progress() {
        let mut pm = ProgressManager::new();
        let id = pm.start_task("Test", TaskCategory::Build);
        
        pm.update_progress(id, 0.5);
        assert_eq!(pm.tasks.get(&id).unwrap().progress, 0.5);
        
        // Test clamping
        pm.update_progress(id, 1.5);
        assert_eq!(pm.tasks.get(&id).unwrap().progress, 1.0);
        
        pm.update_progress(id, -0.5);
        assert_eq!(pm.tasks.get(&id).unwrap().progress, 0.0);
    }

    #[test]
    fn test_update_status() {
        let mut pm = ProgressManager::new();
        let id = pm.start_task("Test", TaskCategory::AssetImport);
        
        pm.update_status(id, "Loading textures...");
        assert_eq!(pm.tasks.get(&id).unwrap().status, "Loading textures...");
    }

    #[test]
    fn test_complete_task() {
        let mut pm = ProgressManager::new();
        let id = pm.start_task("Test", TaskCategory::SceneLoading);
        
        pm.complete_task(id);
        
        assert_eq!(pm.active_count(), 0);
        assert_eq!(pm.recently_completed.len(), 1);
        assert_eq!(pm.recently_completed[0].1.status, "Complete");
    }

    #[test]
    fn test_fail_task() {
        let mut pm = ProgressManager::new();
        let id = pm.start_task("Test", TaskCategory::Build);
        
        pm.fail_task(id, "Compilation error");
        
        assert_eq!(pm.active_count(), 0);
        assert!(pm.recently_completed[0].1.status.contains("Failed"));
    }

    #[test]
    fn test_cancellable_task() {
        let mut pm = ProgressManager::new();
        let id = pm.start_cancellable_task("Test", TaskCategory::Export);
        
        assert!(pm.tasks.get(&id).unwrap().cancellable);
        assert!(!pm.is_cancel_requested(id));
        
        assert!(pm.cancel_task(id));
        assert!(pm.is_cancel_requested(id));
    }

    #[test]
    fn test_non_cancellable_task() {
        let mut pm = ProgressManager::new();
        let id = pm.start_task("Test", TaskCategory::PlayMode);
        
        assert!(!pm.tasks.get(&id).unwrap().cancellable);
        assert!(!pm.cancel_task(id)); // Should return false
    }

    #[test]
    fn test_sub_tasks() {
        let mut pm = ProgressManager::new();
        let id = pm.start_task("Main Task", TaskCategory::Build);
        
        pm.add_sub_task(id, "Compile shaders");
        pm.add_sub_task(id, "Process assets");
        
        assert_eq!(pm.tasks.get(&id).unwrap().sub_tasks.len(), 2);
        
        pm.update_sub_task(id, 0, 1.0);
        assert!(pm.tasks.get(&id).unwrap().sub_tasks[0].completed);
    }

    #[test]
    fn test_overall_progress() {
        let mut pm = ProgressManager::new();
        
        // No tasks = 1.0 (complete)
        assert_eq!(pm.overall_progress(), 1.0);
        
        let id1 = pm.start_task("Task 1", TaskCategory::Other);
        let id2 = pm.start_task("Task 2", TaskCategory::Other);
        
        pm.update_progress(id1, 0.5);
        pm.update_progress(id2, 1.0);
        
        // Average of 0.5 and 1.0 = 0.75
        assert!((pm.overall_progress() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_task_category_icons() {
        assert_eq!(TaskCategory::SceneLoading.icon(), "📂");
        assert_eq!(TaskCategory::Build.icon(), "🔨");
        assert_eq!(TaskCategory::PlayMode.icon(), "▶️");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
    }

    #[test]
    fn test_multiple_tasks_different_categories() {
        let mut pm = ProgressManager::new();
        
        let id1 = pm.start_task("Loading scene", TaskCategory::SceneLoading);
        let id2 = pm.start_task("Importing texture", TaskCategory::AssetImport);
        let id3 = pm.start_task("Building project", TaskCategory::Build);
        
        assert_eq!(pm.active_count(), 3);
        
        pm.update(id1, 0.5, "Loading entities...");
        pm.update(id2, 0.8, "Processing mipmaps...");
        pm.update(id3, 0.2, "Compiling crate 1/10...");
        
        // Verify each task has correct category
        assert_eq!(pm.tasks.get(&id1).unwrap().category, TaskCategory::SceneLoading);
        assert_eq!(pm.tasks.get(&id2).unwrap().category, TaskCategory::AssetImport);
        assert_eq!(pm.tasks.get(&id3).unwrap().category, TaskCategory::Build);
    }

    // === TaskCategory Display and helper tests ===

    #[test]
    fn test_task_category_all() {
        let all = TaskCategory::all();
        assert_eq!(all.len(), 6);
        assert!(all.contains(&TaskCategory::SceneLoading));
        assert!(all.contains(&TaskCategory::AssetImport));
        assert!(all.contains(&TaskCategory::Build));
        assert!(all.contains(&TaskCategory::PlayMode));
        assert!(all.contains(&TaskCategory::Export));
        assert!(all.contains(&TaskCategory::Other));
    }

    #[test]
    fn test_task_category_display() {
        assert_eq!(format!("{}", TaskCategory::SceneLoading), "📂 Scene Loading");
        assert_eq!(format!("{}", TaskCategory::AssetImport), "📥 Asset Import");
        assert_eq!(format!("{}", TaskCategory::Build), "🔨 Build");
        assert_eq!(format!("{}", TaskCategory::PlayMode), "▶️ Play Mode");
        assert_eq!(format!("{}", TaskCategory::Export), "📤 Export");
        assert_eq!(format!("{}", TaskCategory::Other), "⚙️ Other");
    }

    #[test]
    fn test_task_category_name() {
        assert_eq!(TaskCategory::SceneLoading.name(), "Scene Loading");
        assert_eq!(TaskCategory::AssetImport.name(), "Asset Import");
        assert_eq!(TaskCategory::Build.name(), "Build");
        assert_eq!(TaskCategory::PlayMode.name(), "Play Mode");
        assert_eq!(TaskCategory::Export.name(), "Export");
        assert_eq!(TaskCategory::Other.name(), "Other");
    }

    #[test]
    fn test_task_category_is_io_intensive() {
        assert!(TaskCategory::SceneLoading.is_io_intensive());
        assert!(TaskCategory::AssetImport.is_io_intensive());
        assert!(TaskCategory::Export.is_io_intensive());
        assert!(!TaskCategory::Build.is_io_intensive());
        assert!(!TaskCategory::PlayMode.is_io_intensive());
        assert!(!TaskCategory::Other.is_io_intensive());
    }

    #[test]
    fn test_task_category_is_cancellable() {
        assert!(TaskCategory::SceneLoading.is_cancellable());
        assert!(TaskCategory::AssetImport.is_cancellable());
        assert!(TaskCategory::Build.is_cancellable());
        assert!(TaskCategory::Export.is_cancellable());
        assert!(TaskCategory::Other.is_cancellable());
        assert!(!TaskCategory::PlayMode.is_cancellable());
    }

    #[test]
    fn test_task_category_color() {
        // Just verify colors are unique and valid
        let colors = [
            TaskCategory::SceneLoading.color(),
            TaskCategory::AssetImport.color(),
            TaskCategory::Build.color(),
            TaskCategory::PlayMode.color(),
            TaskCategory::Export.color(),
            TaskCategory::Other.color(),
        ];
        // At least verify they're different
        assert_ne!(colors[0], colors[1]);
        assert_ne!(colors[2], colors[3]);
    }

    #[test]
    fn test_task_category_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TaskCategory::SceneLoading);
        set.insert(TaskCategory::Build);
        assert_eq!(set.len(), 2);
    }
}
