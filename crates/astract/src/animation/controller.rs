//! Animation controller for managing multiple animations

use std::collections::HashMap;

/// Unique identifier for an animation
pub type AnimationId = u64;

/// Callback function called when animation completes
type CompletionCallback = Box<dyn FnMut()>;

/// Animation controller manages multiple animations
pub struct AnimationController {
    next_id: AnimationId,
    animations: HashMap<AnimationId, AnimationEntry>,
}

struct AnimationEntry {
    update_fn: Box<dyn FnMut(f32) -> bool>, // Returns true if still running
    on_complete: Option<CompletionCallback>,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationController {
    /// Create a new animation controller
    pub fn new() -> Self {
        Self {
            next_id: 1,
            animations: HashMap::new(),
        }
    }

    /// Register an animation
    /// Returns animation ID for later control
    pub fn add<F>(&mut self, update_fn: F) -> AnimationId
    where
        F: FnMut(f32) -> bool + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;

        self.animations.insert(
            id,
            AnimationEntry {
                update_fn: Box::new(update_fn),
                on_complete: None,
            },
        );

        id
    }

    /// Add animation with completion callback
    pub fn add_with_callback<F, C>(&mut self, update_fn: F, on_complete: C) -> AnimationId
    where
        F: FnMut(f32) -> bool + 'static,
        C: FnMut() + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;

        self.animations.insert(
            id,
            AnimationEntry {
                update_fn: Box::new(update_fn),
                on_complete: Some(Box::new(on_complete)),
            },
        );

        id
    }

    /// Remove an animation
    pub fn remove(&mut self, id: AnimationId) -> bool {
        self.animations.remove(&id).is_some()
    }

    /// Update all animations
    /// dt = delta time in seconds
    pub fn update(&mut self, dt: f32) {
        let mut completed = Vec::new();

        for (id, entry) in self.animations.iter_mut() {
            let still_running = (entry.update_fn)(dt);
            if !still_running {
                completed.push(*id);
            }
        }

        // Call completion callbacks and remove finished animations
        for id in completed {
            if let Some(mut entry) = self.animations.remove(&id) {
                if let Some(mut callback) = entry.on_complete.take() {
                    callback();
                }
            }
        }
    }

    /// Get number of active animations
    pub fn active_count(&self) -> usize {
        self.animations.len()
    }

    /// Clear all animations
    pub fn clear(&mut self) {
        self.animations.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_creation() {
        let controller = AnimationController::new();
        assert_eq!(controller.active_count(), 0);
    }

    #[test]
    fn test_add_animation() {
        let mut controller = AnimationController::new();

        let id = controller.add(|_dt| true);
        assert!(id > 0);
        assert_eq!(controller.active_count(), 1);
    }

    #[test]
    fn test_remove_animation() {
        let mut controller = AnimationController::new();
        let id = controller.add(|_dt| true);

        let removed = controller.remove(id);
        assert!(removed);
        assert_eq!(controller.active_count(), 0);
    }

    #[test]
    fn test_animation_completion() {
        let mut controller = AnimationController::new();

        let mut elapsed = 0.0f32;
        controller.add(move |dt| {
            elapsed += dt;
            elapsed < 1.0 // Run for 1 second
        });

        // Update for 0.5s (should still be running)
        controller.update(0.5);
        assert_eq!(controller.active_count(), 1);

        // Update for 0.6s (should complete)
        controller.update(0.6);
        assert_eq!(controller.active_count(), 0);
    }

    #[test]
    fn test_completion_callback() {
        let mut controller = AnimationController::new();

        let mut callback_called = false;
        let callback_called_ptr = &mut callback_called as *mut bool;

        controller.add_with_callback(
            |_dt| false, // Complete immediately
            move || unsafe {
                *callback_called_ptr = true;
            },
        );

        controller.update(0.016);
        assert!(callback_called);
    }

    #[test]
    fn test_multiple_animations() {
        let mut controller = AnimationController::new();

        controller.add(|_dt| true);
        controller.add(|_dt| true);
        controller.add(|_dt| true);

        assert_eq!(controller.active_count(), 3);
    }

    #[test]
    fn test_clear_animations() {
        let mut controller = AnimationController::new();

        controller.add(|_dt| true);
        controller.add(|_dt| true);

        controller.clear();
        assert_eq!(controller.active_count(), 0);
    }
}
