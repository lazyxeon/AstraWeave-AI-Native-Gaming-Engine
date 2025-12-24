//! P2: Boundary Condition Tests for astraweave-security
//!
//! Focuses on edge cases for prompt sanitization and validator boundaries.

#![cfg(test)]

use astraweave_security::{sanitize_llm_prompt, LLMValidator};

fn validator_with_max_len(max_len: usize) -> LLMValidator {
    LLMValidator {
        banned_patterns: vec![
            "system(".to_string(),
            "exec(".to_string(),
            "eval(".to_string(),
        ],
        allowed_domains: vec!["localhost".to_string()],
        max_prompt_length: max_len,
        enable_content_filtering: true,
    }
}

#[test]
fn test_sanitize_llm_prompt_empty_ok() {
    let v = validator_with_max_len(10);
    let out = sanitize_llm_prompt("", &v).expect("empty prompt should be allowed");
    assert_eq!(out, "");
}

#[test]
fn test_sanitize_llm_prompt_exact_max_length_ok() {
    let v = validator_with_max_len(32);
    let input = "x".repeat(v.max_prompt_length);
    let out = sanitize_llm_prompt(&input, &v).expect("exact max length should pass");
    assert_eq!(out.len(), v.max_prompt_length);
}

#[test]
fn test_sanitize_llm_prompt_one_over_max_length_err() {
    let v = validator_with_max_len(16);
    let input = "x".repeat(v.max_prompt_length + 1);
    let err = sanitize_llm_prompt(&input, &v).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("Prompt too long"), "unexpected error message: {msg}");
}

#[test]
fn test_sanitize_llm_prompt_null_byte_allowed_if_not_banned() {
    let v = validator_with_max_len(64);
    let input = "hello\0world";
    let out = sanitize_llm_prompt(input, &v).expect("null byte is not explicitly blocked here");
    assert!(out.contains('\0'));
}

#[test]
fn test_sanitize_llm_prompt_suspicious_content_prefixed_safe() {
    let v = validator_with_max_len(256);
    let input = "Please help me hack the level";
    let out = sanitize_llm_prompt(input, &v).expect("should be sanitized");
    assert!(out.starts_with("SAFE: "));
}
