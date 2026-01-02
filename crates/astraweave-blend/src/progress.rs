//! Progress tracking for conversion operations.
//!
//! This module provides a progress reporting system for long-running
//! Blender conversion operations.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// Progress information for a conversion operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionProgress {
    /// Current stage of conversion.
    pub stage: ConversionStage,
    /// Progress within current stage (0.0 - 1.0).
    pub stage_progress: f32,
    /// Overall progress (0.0 - 1.0).
    pub overall_progress: f32,
    /// Human-readable status message.
    pub message: String,
    /// Number of items processed (e.g., meshes, textures).
    pub items_processed: u64,
    /// Total items to process (if known).
    pub items_total: Option<u64>,
    /// Bytes written so far.
    pub bytes_written: u64,
    /// Elapsed time.
    pub elapsed: Duration,
    /// Estimated time remaining (if calculable).
    pub estimated_remaining: Option<Duration>,
    /// Whether the operation has been cancelled.
    pub cancelled: bool,
}

impl Default for ConversionProgress {
    fn default() -> Self {
        Self {
            stage: ConversionStage::Initializing,
            stage_progress: 0.0,
            overall_progress: 0.0,
            message: "Initializing...".to_string(),
            items_processed: 0,
            items_total: None,
            bytes_written: 0,
            elapsed: Duration::ZERO,
            estimated_remaining: None,
            cancelled: false,
        }
    }
}

/// Stages of the conversion process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConversionStage {
    /// Initial setup.
    #[default]
    Initializing,
    /// Loading .blend file.
    LoadingBlendFile,
    /// Processing linked libraries.
    ProcessingLinkedLibraries,
    /// Applying modifiers.
    ApplyingModifiers,
    /// Exporting meshes.
    ExportingMeshes,
    /// Exporting materials.
    ExportingMaterials,
    /// Exporting textures.
    ExportingTextures,
    /// Exporting animations.
    ExportingAnimations,
    /// Exporting armatures.
    ExportingArmatures,
    /// Writing output files.
    WritingOutput,
    /// Compressing (Draco).
    Compressing,
    /// Caching results.
    CachingResults,
    /// Finalizing.
    Finalizing,
    /// Completed successfully.
    Completed,
    /// Failed with error.
    Failed,
    /// Cancelled by user.
    Cancelled,
}

impl ConversionStage {
    /// Returns true if this is a terminal stage.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ConversionStage::Completed | ConversionStage::Failed | ConversionStage::Cancelled
        )
    }

    /// Returns the base progress percentage for this stage.
    pub fn base_progress(&self) -> f32 {
        match self {
            ConversionStage::Initializing => 0.0,
            ConversionStage::LoadingBlendFile => 0.05,
            ConversionStage::ProcessingLinkedLibraries => 0.10,
            ConversionStage::ApplyingModifiers => 0.20,
            ConversionStage::ExportingMeshes => 0.30,
            ConversionStage::ExportingMaterials => 0.50,
            ConversionStage::ExportingTextures => 0.60,
            ConversionStage::ExportingAnimations => 0.70,
            ConversionStage::ExportingArmatures => 0.80,
            ConversionStage::WritingOutput => 0.85,
            ConversionStage::Compressing => 0.90,
            ConversionStage::CachingResults => 0.95,
            ConversionStage::Finalizing => 0.98,
            ConversionStage::Completed => 1.0,
            ConversionStage::Failed => 0.0,
            ConversionStage::Cancelled => 0.0,
        }
    }

    /// Returns the next stage's base progress (for calculating stage weight).
    fn next_stage_progress(&self) -> f32 {
        match self {
            ConversionStage::Initializing => 0.05,
            ConversionStage::LoadingBlendFile => 0.10,
            ConversionStage::ProcessingLinkedLibraries => 0.20,
            ConversionStage::ApplyingModifiers => 0.30,
            ConversionStage::ExportingMeshes => 0.50,
            ConversionStage::ExportingMaterials => 0.60,
            ConversionStage::ExportingTextures => 0.70,
            ConversionStage::ExportingAnimations => 0.80,
            ConversionStage::ExportingArmatures => 0.85,
            ConversionStage::WritingOutput => 0.90,
            ConversionStage::Compressing => 0.95,
            ConversionStage::CachingResults => 0.98,
            ConversionStage::Finalizing => 1.0,
            _ => 1.0,
        }
    }

    /// Returns a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            ConversionStage::Initializing => "Initializing conversion",
            ConversionStage::LoadingBlendFile => "Loading .blend file",
            ConversionStage::ProcessingLinkedLibraries => "Processing linked libraries",
            ConversionStage::ApplyingModifiers => "Applying modifiers",
            ConversionStage::ExportingMeshes => "Exporting meshes",
            ConversionStage::ExportingMaterials => "Exporting materials",
            ConversionStage::ExportingTextures => "Exporting textures",
            ConversionStage::ExportingAnimations => "Exporting animations",
            ConversionStage::ExportingArmatures => "Exporting armatures",
            ConversionStage::WritingOutput => "Writing output files",
            ConversionStage::Compressing => "Compressing meshes",
            ConversionStage::CachingResults => "Caching results",
            ConversionStage::Finalizing => "Finalizing",
            ConversionStage::Completed => "Completed",
            ConversionStage::Failed => "Failed",
            ConversionStage::Cancelled => "Cancelled",
        }
    }
}

impl std::fmt::Display for ConversionStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A tracker for monitoring and controlling conversion progress.
pub struct ProgressTracker {
    /// Current progress state.
    sender: watch::Sender<ConversionProgress>,
    /// Receiver for progress updates.
    receiver: watch::Receiver<ConversionProgress>,
    /// Start time of the operation.
    start_time: Instant,
    /// Cancellation flag.
    cancelled: Arc<AtomicBool>,
    /// Items processed counter.
    items_processed: Arc<AtomicU64>,
}

impl ProgressTracker {
    /// Creates a new progress tracker.
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(ConversionProgress::default());
        Self {
            sender,
            receiver,
            start_time: Instant::now(),
            cancelled: Arc::new(AtomicBool::new(false)),
            items_processed: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Returns a handle for receiving progress updates.
    pub fn subscribe(&self) -> ProgressReceiver {
        ProgressReceiver {
            receiver: self.receiver.clone(),
        }
    }

    /// Returns a handle for cancelling the operation.
    pub fn cancellation_token(&self) -> CancellationToken {
        CancellationToken {
            cancelled: self.cancelled.clone(),
        }
    }

    /// Sets the current conversion stage.
    pub fn set_stage(&self, stage: ConversionStage) {
        self.update(|p| {
            p.stage = stage;
            p.stage_progress = 0.0;
            p.message = stage.description().to_string();
            self.recalculate_overall_progress(p);
        });
    }

    /// Sets the stage progress (0.0 - 1.0).
    pub fn set_stage_progress(&self, progress: f32) {
        self.update(|p| {
            p.stage_progress = progress.clamp(0.0, 1.0);
            self.recalculate_overall_progress(p);
        });
    }

    /// Sets a status message.
    pub fn set_message(&self, message: impl Into<String>) {
        self.update(|p| {
            p.message = message.into();
        });
    }

    /// Sets the total number of items.
    pub fn set_total_items(&self, total: u64) {
        self.update(|p| {
            p.items_total = Some(total);
        });
    }

    /// Increments the processed item count.
    pub fn increment_items(&self) {
        let count = self.items_processed.fetch_add(1, Ordering::Relaxed) + 1;
        self.update(|p| {
            p.items_processed = count;
            if let Some(total) = p.items_total {
                if total > 0 {
                    p.stage_progress = count as f32 / total as f32;
                    self.recalculate_overall_progress(p);
                }
            }
        });
    }

    /// Sets the bytes written count.
    pub fn set_bytes_written(&self, bytes: u64) {
        self.update(|p| {
            p.bytes_written = bytes;
        });
    }

    /// Marks the operation as completed.
    pub fn complete(&self) {
        self.update(|p| {
            p.stage = ConversionStage::Completed;
            p.stage_progress = 1.0;
            p.overall_progress = 1.0;
            p.message = "Conversion completed successfully".to_string();
        });
    }

    /// Marks the operation as failed.
    pub fn fail(&self, message: impl Into<String>) {
        self.update(|p| {
            p.stage = ConversionStage::Failed;
            p.message = message.into();
        });
    }

    /// Marks the operation as cancelled.
    pub fn mark_cancelled(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.update(|p| {
            p.stage = ConversionStage::Cancelled;
            p.cancelled = true;
            p.message = "Operation cancelled".to_string();
        });
    }

    /// Checks if the operation has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Returns the current progress.
    pub fn current(&self) -> ConversionProgress {
        self.receiver.borrow().clone()
    }

    /// Updates progress with a closure.
    fn update<F: FnOnce(&mut ConversionProgress)>(&self, f: F) {
        self.sender.send_modify(|p| {
            p.elapsed = self.start_time.elapsed();
            p.cancelled = self.cancelled.load(Ordering::Relaxed);
            f(p);
            
            // Calculate estimated remaining time
            if p.overall_progress > 0.0 && p.overall_progress < 1.0 {
                let elapsed_secs = p.elapsed.as_secs_f64();
                let estimated_total = elapsed_secs / p.overall_progress as f64;
                let remaining = estimated_total - elapsed_secs;
                if remaining > 0.0 {
                    p.estimated_remaining = Some(Duration::from_secs_f64(remaining));
                }
            } else {
                p.estimated_remaining = None;
            }
        });
    }

    /// Recalculates overall progress from stage and stage progress.
    fn recalculate_overall_progress(&self, p: &mut ConversionProgress) {
        let base = p.stage.base_progress();
        let next = p.stage.next_stage_progress();
        let stage_weight = next - base;
        p.overall_progress = base + (stage_weight * p.stage_progress);
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Receiver for progress updates.
#[derive(Clone)]
pub struct ProgressReceiver {
    receiver: watch::Receiver<ConversionProgress>,
}

impl ProgressReceiver {
    /// Returns the current progress.
    pub fn current(&self) -> ConversionProgress {
        self.receiver.borrow().clone()
    }

    /// Waits for the next progress update.
    pub async fn changed(&mut self) -> Result<(), watch::error::RecvError> {
        self.receiver.changed().await
    }

    /// Returns true if the operation is complete (success, failure, or cancelled).
    pub fn is_complete(&self) -> bool {
        self.receiver.borrow().stage.is_terminal()
    }
}

/// Token for cancelling an operation.
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Cancels the operation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Checks if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// A simple progress callback type.
pub type ProgressCallback = Box<dyn Fn(&ConversionProgress) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_basic() {
        let tracker = ProgressTracker::new();
        
        tracker.set_stage(ConversionStage::LoadingBlendFile);
        let progress = tracker.current();
        assert_eq!(progress.stage, ConversionStage::LoadingBlendFile);
        assert_eq!(progress.overall_progress, 0.05);
    }

    #[test]
    fn test_progress_tracker_stage_progress() {
        let tracker = ProgressTracker::new();
        
        tracker.set_stage(ConversionStage::ExportingMeshes);
        tracker.set_stage_progress(0.5);
        
        let progress = tracker.current();
        // ExportingMeshes base: 0.30, next: 0.50, weight: 0.20
        // overall = 0.30 + 0.20 * 0.5 = 0.40
        assert!((progress.overall_progress - 0.40).abs() < 0.01);
    }

    #[test]
    fn test_cancellation() {
        let tracker = ProgressTracker::new();
        let token = tracker.cancellation_token();
        
        assert!(!tracker.is_cancelled());
        assert!(!token.is_cancelled());
        
        token.cancel();
        
        assert!(tracker.is_cancelled());
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_item_counting() {
        let tracker = ProgressTracker::new();
        tracker.set_stage(ConversionStage::ExportingMeshes);
        tracker.set_total_items(10);
        
        for _ in 0..5 {
            tracker.increment_items();
        }
        
        let progress = tracker.current();
        assert_eq!(progress.items_processed, 5);
        assert_eq!(progress.stage_progress, 0.5);
    }

    #[test]
    fn test_completion() {
        let tracker = ProgressTracker::new();
        tracker.complete();
        
        let progress = tracker.current();
        assert_eq!(progress.stage, ConversionStage::Completed);
        assert_eq!(progress.overall_progress, 1.0);
    }

    #[test]
    fn test_stage_descriptions() {
        assert_eq!(ConversionStage::LoadingBlendFile.description(), "Loading .blend file");
        assert!(ConversionStage::Completed.is_terminal());
        assert!(!ConversionStage::LoadingBlendFile.is_terminal());
    }
}
