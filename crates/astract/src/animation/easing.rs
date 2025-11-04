//! Easing functions for smooth animations
//!
//! Based on Robert Penner's easing equations.
//! See: http://robertpenner.com/easing/

use std::f32::consts::PI;

/// Easing function types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EasingFunction {
    /// No easing (constant rate)
    Linear,

    // Quadratic
    /// Accelerating from zero velocity
    QuadIn,
    /// Decelerating to zero velocity
    QuadOut,
    /// Acceleration until halfway, then deceleration
    QuadInOut,

    // Cubic
    /// Accelerating from zero velocity (cubic)
    CubicIn,
    /// Decelerating to zero velocity (cubic)
    CubicOut,
    /// Acceleration until halfway, then deceleration (cubic)
    CubicInOut,

    // Sine
    /// Sinusoidal acceleration
    SineIn,
    /// Sinusoidal deceleration
    SineOut,
    /// Sinusoidal acceleration and deceleration
    SineInOut,

    // Exponential
    /// Exponential acceleration
    ExpoIn,
    /// Exponential deceleration
    ExpoOut,
    /// Exponential acceleration and deceleration
    ExpoInOut,

    // Elastic
    /// Elastic bounce effect
    ElasticOut,

    // Bounce
    /// Bounce effect
    BounceOut,
}

/// Apply easing function to normalized time (0.0 to 1.0)
pub fn easing(func: EasingFunction, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);

    match func {
        EasingFunction::Linear => t,

        // Quadratic
        EasingFunction::QuadIn => quad_in(t),
        EasingFunction::QuadOut => quad_out(t),
        EasingFunction::QuadInOut => quad_in_out(t),

        // Cubic
        EasingFunction::CubicIn => cubic_in(t),
        EasingFunction::CubicOut => cubic_out(t),
        EasingFunction::CubicInOut => cubic_in_out(t),

        // Sine
        EasingFunction::SineIn => sine_in(t),
        EasingFunction::SineOut => sine_out(t),
        EasingFunction::SineInOut => sine_in_out(t),

        // Exponential
        EasingFunction::ExpoIn => expo_in(t),
        EasingFunction::ExpoOut => expo_out(t),
        EasingFunction::ExpoInOut => expo_in_out(t),

        // Elastic
        EasingFunction::ElasticOut => elastic_out(t),

        // Bounce
        EasingFunction::BounceOut => bounce_out(t),
    }
}

// Quadratic easing functions
fn quad_in(t: f32) -> f32 {
    t * t
}

fn quad_out(t: f32) -> f32 {
    t * (2.0 - t)
}

fn quad_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

// Cubic easing functions
fn cubic_in(t: f32) -> f32 {
    t * t * t
}

fn cubic_out(t: f32) -> f32 {
    let t = t - 1.0;
    t * t * t + 1.0
}

fn cubic_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        let t = 2.0 * t - 2.0;
        1.0 + t * t * t / 2.0
    }
}

// Sine easing functions
fn sine_in(t: f32) -> f32 {
    1.0 - (t * PI / 2.0).cos()
}

fn sine_out(t: f32) -> f32 {
    (t * PI / 2.0).sin()
}

fn sine_in_out(t: f32) -> f32 {
    -(((PI * t).cos() - 1.0) / 2.0)
}

// Exponential easing functions
fn expo_in(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else {
        2.0f32.powf(10.0 * t - 10.0)
    }
}

fn expo_out(t: f32) -> f32 {
    if t == 1.0 {
        1.0
    } else {
        1.0 - 2.0f32.powf(-10.0 * t)
    }
}

fn expo_in_out(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else if t < 0.5 {
        2.0f32.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0f32.powf(-20.0 * t + 10.0)) / 2.0
    }
}

// Elastic easing (bounce with decay)
fn elastic_out(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let p = 0.3;
        let s = p / 4.0;
        2.0f32.powf(-10.0 * t) * ((t - s) * (2.0 * PI) / p).sin() + 1.0
    }
}

// Bounce easing
fn bounce_out(t: f32) -> f32 {
    if t < 1.0 / 2.75 {
        7.5625 * t * t
    } else if t < 2.0 / 2.75 {
        let t = t - 1.5 / 2.75;
        7.5625 * t * t + 0.75
    } else if t < 2.5 / 2.75 {
        let t = t - 2.25 / 2.75;
        7.5625 * t * t + 0.9375
    } else {
        let t = t - 2.625 / 2.75;
        7.5625 * t * t + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert_eq!(easing(EasingFunction::Linear, 0.0), 0.0);
        assert_eq!(easing(EasingFunction::Linear, 0.5), 0.5);
        assert_eq!(easing(EasingFunction::Linear, 1.0), 1.0);
    }

    #[test]
    fn test_quad_in() {
        assert_eq!(easing(EasingFunction::QuadIn, 0.0), 0.0);
        assert_eq!(easing(EasingFunction::QuadIn, 0.5), 0.25);
        assert_eq!(easing(EasingFunction::QuadIn, 1.0), 1.0);
    }

    #[test]
    fn test_quad_out() {
        assert_eq!(easing(EasingFunction::QuadOut, 0.0), 0.0);
        assert_eq!(easing(EasingFunction::QuadOut, 0.5), 0.75);
        assert_eq!(easing(EasingFunction::QuadOut, 1.0), 1.0);
    }

    #[test]
    fn test_cubic_in() {
        assert_eq!(easing(EasingFunction::CubicIn, 0.0), 0.0);
        assert_eq!(easing(EasingFunction::CubicIn, 0.5), 0.125);
        assert_eq!(easing(EasingFunction::CubicIn, 1.0), 1.0);
    }

    #[test]
    fn test_sine_in() {
        let result = easing(EasingFunction::SineIn, 0.5);
        assert!((result - 0.2928).abs() < 0.001); // ~0.2928...
    }

    #[test]
    fn test_expo_in() {
        assert_eq!(easing(EasingFunction::ExpoIn, 0.0), 0.0);
        assert_eq!(easing(EasingFunction::ExpoIn, 1.0), 1.0);

        let mid = easing(EasingFunction::ExpoIn, 0.5);
        assert!(mid < 0.1); // Starts very slow
    }

    #[test]
    fn test_elastic_out() {
        assert_eq!(easing(EasingFunction::ElasticOut, 0.0), 0.0);
        assert_eq!(easing(EasingFunction::ElasticOut, 1.0), 1.0);

        // Elastic overshoots then settles
        let overshoot = easing(EasingFunction::ElasticOut, 0.5);
        assert!(overshoot > 0.0); // Should be positive (may overshoot 1.0)
    }

    #[test]
    fn test_bounce_out() {
        assert_eq!(easing(EasingFunction::BounceOut, 0.0), 0.0);
        assert_eq!(easing(EasingFunction::BounceOut, 1.0), 1.0);

        // Bounce has multiple peaks
        let bounce1 = easing(EasingFunction::BounceOut, 0.25);
        let bounce2 = easing(EasingFunction::BounceOut, 0.5);
        assert!(bounce1 > 0.0 && bounce1 < 1.0);
        assert!(bounce2 > 0.0 && bounce2 < 1.0);
    }

    #[test]
    fn test_easing_bounds() {
        // All easing functions should map 0.0 -> 0.0 and 1.0 -> 1.0
        let funcs = [
            EasingFunction::Linear,
            EasingFunction::QuadIn,
            EasingFunction::QuadOut,
            EasingFunction::QuadInOut,
            EasingFunction::CubicIn,
            EasingFunction::CubicOut,
            EasingFunction::CubicInOut,
            EasingFunction::SineIn,
            EasingFunction::SineOut,
            EasingFunction::SineInOut,
            EasingFunction::ExpoIn,
            EasingFunction::ExpoOut,
            EasingFunction::ExpoInOut,
            EasingFunction::ElasticOut,
            EasingFunction::BounceOut,
        ];

        for func in funcs {
            assert_eq!(easing(func, 0.0), 0.0, "{:?} should map 0.0 -> 0.0", func);
            assert_eq!(easing(func, 1.0), 1.0, "{:?} should map 1.0 -> 1.0", func);
        }
    }

    #[test]
    fn test_easing_clamping() {
        // Test that values outside [0, 1] are clamped
        assert_eq!(easing(EasingFunction::Linear, -0.5), 0.0);
        assert_eq!(easing(EasingFunction::Linear, 1.5), 1.0);
    }
}
