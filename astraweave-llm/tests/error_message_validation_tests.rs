fn assert_message_sane(msg: &str) {
    assert!(
        msg.len() >= 10,
        "error message too short / not descriptive: {msg:?}"
    );
    assert!(
        msg.chars().any(|c| c.is_ascii_alphabetic()),
        "error message should contain some words: {msg:?}"
    );

    // Avoid leaking local filesystem paths.
    // (We keep this conservative; it is fine for errors to mention filenames.)
    for forbidden in ["C:\\Users\\", "\\Users\\", "/home/", "/Users/"] {
        assert!(
            !msg.contains(forbidden),
            "error message leaks local path ({forbidden}): {msg:?}"
        );
    }
}

#[test]
fn plan_parser_error_message_is_descriptive_and_no_path_leak() {
    let reg = astraweave_core::default_tool_registry();
    let err = astraweave_llm::plan_parser::parse_llm_response("not json", &reg).unwrap_err();
    let msg = err.to_string();

    assert!(msg.contains("Failed to parse LLM response"));
    assert_message_sane(&msg);
}

#[tokio::test]
async fn rate_limiter_error_message_is_descriptive_and_no_path_leak() {
    use astraweave_llm::rate_limiter::{RateLimitContext, RateLimiter, RateLimiterConfig, RequestPriority};
    use std::time::Duration;

    let mut cfg = RateLimiterConfig::default();
    cfg.default_rpm = 1;
    cfg.default_tpm = 1_000_000;
    cfg.user_rpm = 10;
    cfg.global_rpm = 10;
    cfg.window_duration = Duration::from_secs(60);
    cfg.adaptive_limiting = false;

    let rl = RateLimiter::new(cfg);
    let ctx = RateLimitContext {
        user_id: None,
        model: "test-model".to_string(),
        estimated_tokens: 0,
        priority: RequestPriority::Normal,
    };

    let _first = rl.acquire(&ctx).await.expect("first acquire should succeed");
    let err = rl.acquire(&ctx).await.unwrap_err();
    let msg = err.to_string();

    assert!(msg.contains("Request rate limit exceeded for model"));
    assert_message_sane(&msg);
}
