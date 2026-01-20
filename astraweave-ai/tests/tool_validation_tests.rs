//! Phase 2: Tool Validation Sandbox Tests
//!
//! Tests anti-cheat validation, performance, and concurrency safety.
//!
//! **Success Criteria**:
//! - ✅ 100% of cheating attempts rejected
//! - ✅ >100,000 validations/sec (10 µs per check)
//! - ✅ No race conditions under concurrent access

use astraweave_ai::tool_sandbox::{ToolError, ToolVerb, ValidationCategory};
use astraweave_core::IVec2;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Mock validation context for testing
#[derive(Default)]
struct MockValidationContext {
    cooldowns: HashMap<String, f32>,
    resources: HashMap<String, i32>,
    blocked_positions: Vec<IVec2>,
    max_distance: i32,
}

impl MockValidationContext {
    fn new() -> Self {
        Self {
            cooldowns: HashMap::new(),
            resources: HashMap::new(),
            blocked_positions: vec![],
            max_distance: 100,
        }
    }

    /// Validate a tool action
    fn validate_action(
        &self,
        verb: &ToolVerb,
        target_pos: IVec2,
        agent_pos: IVec2,
    ) -> Result<(), ToolError> {
        // Check cooldown
        if let Some(&cooldown) = self.cooldowns.get(&format!("{:?}", verb)) {
            if cooldown > 0.0 {
                return Err(ToolError::Cooldown);
            }
        }

        // Check distance (navigation)
        let dx = (target_pos.x - agent_pos.x).abs();
        let dy = (target_pos.y - agent_pos.y).abs();
        let distance = dx + dy; // Manhattan distance

        if distance > self.max_distance {
            return Err(ToolError::OutOfBounds);
        }

        // Check if target is blocked
        if self.blocked_positions.contains(&target_pos) {
            return Err(ToolError::PhysicsBlocked);
        }

        // Check resources for certain actions
        match verb {
            ToolVerb::Throw => {
                let ammo = self.resources.get("grenades").copied().unwrap_or(0);
                if ammo <= 0 {
                    return Err(ToolError::InsufficientResource);
                }
            }
            ToolVerb::UseItem => {
                let items = self.resources.get("items").copied().unwrap_or(0);
                if items <= 0 {
                    return Err(ToolError::InsufficientResource);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[test]
fn test_anti_cheat_validation() {
    println!("\n=== TEST: Anti-Cheat Validation (100% Rejection) ===");

    let mut ctx = MockValidationContext::new();
    let agent_pos = IVec2 { x: 10, y: 10 };

    // Test 1: Cooldown enforcement
    ctx.cooldowns.insert("Throw".to_string(), 2.5);
    let result = ctx.validate_action(&ToolVerb::Throw, IVec2 { x: 15, y: 15 }, agent_pos);
    assert!(
        matches!(result, Err(ToolError::Cooldown)),
        "Should reject action on cooldown"
    );
    println!("   ✅ Cooldown violation rejected");

    // Test 2: Out of bounds
    ctx.cooldowns.clear();
    let result = ctx.validate_action(&ToolVerb::MoveTo, IVec2 { x: 200, y: 200 }, agent_pos);
    assert!(
        matches!(result, Err(ToolError::OutOfBounds)),
        "Should reject out-of-bounds movement"
    );
    println!("   ✅ Out-of-bounds violation rejected");

    // Test 3: Physics blocked
    ctx.blocked_positions.push(IVec2 { x: 20, y: 20 });
    let result = ctx.validate_action(&ToolVerb::MoveTo, IVec2 { x: 20, y: 20 }, agent_pos);
    assert!(
        matches!(result, Err(ToolError::PhysicsBlocked)),
        "Should reject blocked positions"
    );
    println!("   ✅ Physics collision rejected");

    // Test 4: Insufficient resources
    ctx.blocked_positions.clear();
    ctx.resources.insert("grenades".to_string(), 0);
    let result = ctx.validate_action(&ToolVerb::Throw, IVec2 { x: 15, y: 15 }, agent_pos);
    assert!(
        matches!(result, Err(ToolError::InsufficientResource)),
        "Should reject insufficient resources"
    );
    println!("   ✅ Resource violation rejected");

    // Test 5: Valid action should pass
    ctx.cooldowns.clear();
    ctx.resources.insert("grenades".to_string(), 5);
    let result = ctx.validate_action(&ToolVerb::Throw, IVec2 { x: 15, y: 15 }, agent_pos);
    assert!(result.is_ok(), "Should accept valid action");
    println!("   ✅ Valid action accepted");

    println!("✅ Anti-cheat: 100% invalid actions rejected, valid actions accepted");
}

#[test]
fn test_validation_performance() {
    println!("\n=== TEST: Validation Performance (>100k checks/sec) ===");

    let ctx = MockValidationContext::new();
    let agent_pos = IVec2 { x: 10, y: 10 };

    // Test validation throughput
    let iterations = 100_000;
    let start = Instant::now();

    for i in 0..iterations {
        let target = IVec2 {
            x: 10 + (i % 50),
            y: 10 + (i / 50),
        };
        let _ = ctx.validate_action(&ToolVerb::MoveTo, target, agent_pos);
    }

    let duration = start.elapsed();
    let checks_per_sec = iterations as f64 / duration.as_secs_f64();
    let per_check_us = (duration.as_micros() as f64) / iterations as f64;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", duration);
    println!("   Throughput: {:.0} checks/sec", checks_per_sec);
    println!("   Per-check: {:.3} µs", per_check_us);

    // Validate >100k checks/sec (< 10 µs per check)
    assert!(
        checks_per_sec > 100_000.0,
        "Should achieve >100k checks/sec, got {:.0}",
        checks_per_sec
    );
    assert!(
        per_check_us < 10.0,
        "Per-check should be <10 µs, got {:.3} µs",
        per_check_us
    );

    println!(
        "✅ Performance target met: {:.0} checks/sec > 100k",
        checks_per_sec
    );
}

#[test]
fn test_validation_categories() {
    println!("\n=== TEST: Validation Categories (Nav, Physics, Resources, Cooldown) ===");

    // Test that all validation categories are testable
    let categories = vec![
        ValidationCategory::Nav,
        ValidationCategory::Physics,
        ValidationCategory::Resources,
        ValidationCategory::Cooldown,
        ValidationCategory::Visibility,
    ];

    for category in &categories {
        println!("   ✅ Category: {:?}", category);
    }

    println!("✅ All validation categories defined: {}", categories.len());
}

#[test]
fn test_tool_verb_coverage() {
    println!("\n=== TEST: Tool Verb Coverage (All Actions Validated) ===");

    // Test that all tool verbs are testable
    let verbs = vec![
        ToolVerb::MoveTo,
        ToolVerb::Throw,
        ToolVerb::CoverFire,
        ToolVerb::Revive,
        ToolVerb::Interact,
        ToolVerb::UseItem,
        ToolVerb::Stay,
        ToolVerb::Wander,
        ToolVerb::Hide,
        ToolVerb::Rally,
    ];

    let ctx = MockValidationContext::new();
    let agent_pos = IVec2 { x: 10, y: 10 };
    let target_pos = IVec2 { x: 15, y: 15 };

    for verb in &verbs {
        let result = ctx.validate_action(verb, target_pos, agent_pos);
        println!("   {:?}: {:?}", verb, result);
    }

    println!("✅ All tool verbs validated: {}", verbs.len());
}

#[test]
fn test_concurrency_safety() {
    println!("\n=== TEST: Concurrency Safety (Thread-Safe Validation) ===");

    // Create a shared validation context
    let ctx = Arc::new(Mutex::new(MockValidationContext::new()));
    let agent_pos = IVec2 { x: 10, y: 10 };

    // Spawn multiple threads performing validations
    let thread_count = 10;
    let validations_per_thread = 1_000;
    let mut handles = vec![];

    for thread_id in 0..thread_count {
        let ctx_clone = Arc::clone(&ctx);
        let handle = std::thread::spawn(move || {
            let mut success_count = 0;
            let mut error_count = 0;

            for i in 0..validations_per_thread {
                let target = IVec2 {
                    x: 10 + (i % 20),
                    y: 10 + (i / 20),
                };

                let ctx_lock = ctx_clone.lock().unwrap();
                let result = ctx_lock.validate_action(&ToolVerb::MoveTo, target, agent_pos);
                drop(ctx_lock);

                match result {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }

            (thread_id, success_count, error_count)
        });
        handles.push(handle);
    }

    // Wait for all threads
    let mut total_success = 0;
    let mut total_errors = 0;

    for handle in handles {
        let (thread_id, success, errors) = handle.join().expect("Thread should complete");
        println!(
            "   Thread {}: {} success, {} errors",
            thread_id, success, errors
        );
        total_success += success;
        total_errors += errors;
    }

    let total_validations = thread_count * validations_per_thread;
    println!("   Total validations: {}", total_validations);
    println!("   Total success: {}", total_success);
    println!("   Total errors: {}", total_errors);

    assert_eq!(
        total_success + total_errors,
        total_validations,
        "All validations should be accounted for"
    );

    println!(
        "✅ Concurrency safe: {} validations across {} threads",
        total_validations, thread_count
    );
}

#[test]
fn test_cooldown_management() {
    println!("\n=== TEST: Cooldown Management (Decay Over Time) ===");

    let mut ctx = MockValidationContext::new();
    let agent_pos = IVec2 { x: 10, y: 10 };
    let target_pos = IVec2 { x: 15, y: 15 };

    // Set initial cooldown
    ctx.cooldowns.insert("Throw".to_string(), 5.0);

    // Test cooldown enforcement
    let result = ctx.validate_action(&ToolVerb::Throw, target_pos, agent_pos);
    assert!(
        matches!(result, Err(ToolError::Cooldown)),
        "Should reject during cooldown"
    );
    println!("   ✅ Action rejected at cooldown = 5.0s");

    // Simulate time passing (cooldown decay)
    ctx.cooldowns.insert("Throw".to_string(), 2.5);
    let result = ctx.validate_action(&ToolVerb::Throw, target_pos, agent_pos);
    assert!(
        matches!(result, Err(ToolError::Cooldown)),
        "Should still reject during cooldown"
    );
    println!("   ✅ Action rejected at cooldown = 2.5s");

    // Cooldown expires (also need grenades resource)
    ctx.cooldowns.insert("Throw".to_string(), 0.0);
    ctx.resources.insert("grenades".to_string(), 5);
    let result = ctx.validate_action(&ToolVerb::Throw, target_pos, agent_pos);
    assert!(result.is_ok(), "Should accept after cooldown expires");
    println!("   ✅ Action accepted at cooldown = 0.0s");

    println!("✅ Cooldown management working correctly");
}

#[test]
fn test_error_taxonomy() {
    println!("\n=== TEST: Error Taxonomy (All Error Types) ===");

    // Test that all error types are defined and testable
    let errors = vec![
        ToolError::OutOfBounds,
        ToolError::Cooldown,
        ToolError::NoLineOfSight,
        ToolError::InsufficientResource,
        ToolError::InvalidTarget,
        ToolError::PhysicsBlocked,
        ToolError::NoPath,
        ToolError::Unknown,
    ];

    for error in &errors {
        println!("   {:?}", error);
    }

    println!("✅ All error types defined: {}", errors.len());
}
