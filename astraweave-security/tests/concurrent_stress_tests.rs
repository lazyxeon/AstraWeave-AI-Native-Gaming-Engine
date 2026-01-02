//! P2: Concurrent Stress Tests for astraweave-security
//!
//! Focus: Prompt sanitization should be thread-safe and deterministic under
//! moderate concurrent load.

#![cfg(test)]

use astraweave_security::{sanitize_llm_prompt, LLMValidator};
use std::sync::Arc;

fn validator() -> LLMValidator {
    LLMValidator {
        banned_patterns: vec![
            "system(".to_string(),
            "exec(".to_string(),
            "eval(".to_string(),
        ],
        allowed_domains: vec!["localhost".to_string()],
        max_prompt_length: 512,
        enable_content_filtering: true,
    }
}

#[test]
fn test_concurrent_sanitize_llm_prompt_no_panic() {
    let v = Arc::new(validator());
    let inputs: Arc<Vec<String>> = Arc::new(vec![
        "Hello, please summarize the scene.".to_string(),
        "Please help me hack the level".to_string(),
        "This is a normal gameplay prompt.".to_string(),
        "safe text with punctuation!!!".to_string(),
    ]);

    let mut handles = Vec::new();
    for t in 0..8 {
        let v = v.clone();
        let inputs = inputs.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..2000 {
                let s = &inputs[(i + t) % inputs.len()];
                let out = sanitize_llm_prompt(s, &v).expect("sanitization should succeed for these inputs");
                assert!(out.len() <= v.max_prompt_length);
            }
        }));
    }

    for h in handles {
        h.join().expect("thread panicked");
    }
}
