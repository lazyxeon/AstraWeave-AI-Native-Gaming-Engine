//! Tween-based animation with easing functions

use egui::{Color32, Vec2};

/// Animation state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    /// Animation not started
    Idle,
    /// Animation currently playing
    Running,
    /// Animation paused (can resume)
    Paused,
    /// Animation finished
    Complete,
}

/// Trait for types that can be animated (interpolated)
pub trait Animatable: Copy {
    /// Linearly interpolate between start and end
    /// t = 0.0 returns start, t = 1.0 returns end
    fn lerp(start: Self, end: Self, t: f32) -> Self;
}

impl Animatable for f32 {
    fn lerp(start: Self, end: Self, t: f32) -> Self {
        start + (end - start) * t
    }
}

impl Animatable for Vec2 {
    fn lerp(start: Self, end: Self, t: f32) -> Self {
        Vec2::new(f32::lerp(start.x, end.x, t), f32::lerp(start.y, end.y, t))
    }
}

impl Animatable for Color32 {
    fn lerp(start: Self, end: Self, t: f32) -> Self {
        Color32::from_rgba_unmultiplied(
            (start.r() as f32 + (end.r() as f32 - start.r() as f32) * t) as u8,
            (start.g() as f32 + (end.g() as f32 - start.g() as f32) * t) as u8,
            (start.b() as f32 + (end.b() as f32 - start.b() as f32) * t) as u8,
            (start.a() as f32 + (end.a() as f32 - start.a() as f32) * t) as u8,
        )
    }
}

/// Tween animation with easing
pub struct Tween<T: Animatable> {
    start: T,
    end: T,
    duration: f32,
    elapsed: f32,
    state: AnimationState,
    easing: super::EasingFunction,
}

impl<T: Animatable> Tween<T> {
    /// Create a new tween animation
    pub fn new(start: T, end: T, duration: f32) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
            state: AnimationState::Idle,
            easing: super::EasingFunction::Linear,
        }
    }

    /// Set easing function
    pub fn with_easing(mut self, easing: super::EasingFunction) -> Self {
        self.easing = easing;
        self
    }

    /// Start the animation
    pub fn play(&mut self) {
        self.state = AnimationState::Running;
        self.elapsed = 0.0;
    }

    /// Pause the animation
    pub fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
        }
    }

    /// Resume the animation
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Running;
        }
    }

    /// Stop and reset the animation
    pub fn stop(&mut self) {
        self.state = AnimationState::Idle;
        self.elapsed = 0.0;
    }

    /// Restart the animation from the beginning
    pub fn restart(&mut self) {
        self.elapsed = 0.0;
        self.state = AnimationState::Running;
    }

    /// Update animation state (call each frame)
    /// Returns true if animation is still running
    pub fn update(&mut self, dt: f32) -> bool {
        if self.state != AnimationState::Running {
            return false;
        }

        self.elapsed += dt;

        if self.elapsed >= self.duration {
            self.elapsed = self.duration;
            self.state = AnimationState::Complete;
            false
        } else {
            true
        }
    }

    /// Get current animation state
    pub fn state(&self) -> AnimationState {
        self.state
    }

    /// Get current value (interpolated based on elapsed time and easing)
    pub fn value(&self) -> T {
        let t = if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let eased_t = super::easing::easing(self.easing, t);
        T::lerp(self.start, self.end, eased_t)
    }

    /// Get progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }

    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.state == AnimationState::Complete
    }

    /// Check if animation is running
    pub fn is_running(&self) -> bool {
        self.state == AnimationState::Running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animatable_f32() {
        assert_eq!(f32::lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(f32::lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(f32::lerp(0.0, 10.0, 1.0), 10.0);
    }

    #[test]
    fn test_animatable_vec2() {
        let start = Vec2::new(0.0, 0.0);
        let end = Vec2::new(10.0, 20.0);

        let mid = Vec2::lerp(start, end, 0.5);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 10.0);
    }

    #[test]
    fn test_animatable_color32() {
        let start = Color32::from_rgb(0, 0, 0);
        let end = Color32::from_rgb(100, 200, 255);

        let mid = Color32::lerp(start, end, 0.5);
        assert_eq!(mid.r(), 50);
        assert_eq!(mid.g(), 100);
        assert_eq!(mid.b(), 127); // Note: integer rounding
    }

    #[test]
    fn test_tween_creation() {
        let tween = Tween::new(0.0f32, 10.0, 1.0);
        assert_eq!(tween.state(), AnimationState::Idle);
        assert_eq!(tween.value(), 0.0);
    }

    #[test]
    fn test_tween_play() {
        let mut tween = Tween::new(0.0f32, 10.0, 1.0);
        tween.play();
        assert_eq!(tween.state(), AnimationState::Running);
    }

    #[test]
    fn test_tween_update() {
        let mut tween = Tween::new(0.0f32, 10.0, 1.0);
        tween.play();

        // Update halfway
        tween.update(0.5);
        assert_eq!(tween.progress(), 0.5);
        assert_eq!(tween.value(), 5.0);
        assert!(tween.is_running());

        // Complete animation
        tween.update(0.5);
        assert_eq!(tween.progress(), 1.0);
        assert_eq!(tween.value(), 10.0);
        assert!(tween.is_complete());
    }

    #[test]
    fn test_tween_pause_resume() {
        let mut tween = Tween::new(0.0f32, 10.0, 1.0);
        tween.play();
        tween.update(0.5);

        tween.pause();
        assert_eq!(tween.state(), AnimationState::Paused);

        // Update while paused (should not progress)
        tween.update(0.5);
        assert_eq!(tween.progress(), 0.5);

        tween.resume();
        assert_eq!(tween.state(), AnimationState::Running);

        tween.update(0.5);
        assert!(tween.is_complete());
    }

    #[test]
    fn test_tween_restart() {
        let mut tween = Tween::new(0.0f32, 10.0, 1.0);
        tween.play();
        tween.update(1.0);
        assert!(tween.is_complete());

        tween.restart();
        assert_eq!(tween.state(), AnimationState::Running);
        assert_eq!(tween.progress(), 0.0);
    }

    #[test]
    fn test_tween_stop() {
        let mut tween = Tween::new(0.0f32, 10.0, 1.0);
        tween.play();
        tween.update(0.5);

        tween.stop();
        assert_eq!(tween.state(), AnimationState::Idle);
        assert_eq!(tween.progress(), 0.0);
    }
}
