//! P2: Concurrent Stress Tests for astraweave-prompts
//!
//! Focus: Sanitization utilities should be thread-safe (no panics, deterministic bounds)
//! under moderate concurrent load.

#![cfg(test)]
#![allow(clippy::field_reassign_with_default)]

use astraweave_prompts::sanitize::{sanitize_input, sanitize_variable_name, TrustLevel, SanitizationConfig};
use std::sync::Arc;

#[test]
fn test_concurrent_sanitize_input_no_panic() {
    let mut cfg = SanitizationConfig::default();
    cfg.block_injection_patterns = false; // avoid expected rejections due to pattern detector
    cfg.allow_unicode = false;
    cfg.allow_control_chars = false;
    cfg.max_user_input_length = 256;
    cfg.max_variable_name_length = 32;

    let cfg = Arc::new(cfg);

    let inputs: Arc<Vec<String>> = Arc::new(vec![
        "hello world".to_string(),
        "tabs\tand\nnewlines".to_string(),
        "日本語🎮ABC".to_string(),
        "spaces    and   punctuation!!!".to_string(),
        "aB3_".repeat(64),
    ]);

    let mut handles = Vec::new();
    for t in 0..8 {
        let cfg = cfg.clone();
        let inputs = inputs.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..2000 {
                let s = &inputs[(i + t) % inputs.len()];
                let out = sanitize_input(s, TrustLevel::User, &cfg).expect("sanitize_input should not error in this config");
                assert!(out.len() <= cfg.max_user_input_length);
                assert!(out.is_ascii());
            }
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }
}

#[test]
fn test_concurrent_sanitize_variable_name_no_panic() {
    let mut cfg = SanitizationConfig::default();
    cfg.max_variable_name_length = 32;
    let cfg = Arc::new(cfg);

    let names: Arc<Vec<String>> = Arc::new(vec![
        "_abc".into(),
        "alpha_beta".into(),
        "A0_b".into(),
        "a".repeat(32),
    ]);

    let mut handles = Vec::new();
    for t in 0..8 {
        let cfg = cfg.clone();
        let names = names.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..2000 {
                let n = &names[(i + t) % names.len()];
                let out = sanitize_variable_name(n, &cfg).expect("expected valid variable name");
                assert!(!out.is_empty());
                assert!(out.len() <= cfg.max_variable_name_length);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }
}
