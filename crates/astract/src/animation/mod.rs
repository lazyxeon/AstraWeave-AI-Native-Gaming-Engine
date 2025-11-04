//! Animation module
//!
//! Provides animation primitives for smooth UI transitions and effects.

mod controller;
mod easing;
mod spring;
mod tween;

pub use controller::{AnimationController, AnimationId};
pub use easing::{easing, EasingFunction};
pub use spring::{Spring, SpringParams};
pub use tween::{Animatable, AnimationState, Tween};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_state_transitions() {
        // Test state machine
        let state = AnimationState::Idle;
        assert!(matches!(state, AnimationState::Idle));

        let state = AnimationState::Running;
        assert!(matches!(state, AnimationState::Running));

        let state = AnimationState::Complete;
        assert!(matches!(state, AnimationState::Complete));
    }
}
