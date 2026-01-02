fn assert_message_sane(msg: &str) {
    assert!(
        msg.len() >= 10,
        "error message too short / not descriptive: {msg:?}"
    );
    assert!(
        msg.chars().any(|c| c.is_ascii_alphabetic()),
        "error message should contain some words: {msg:?}"
    );

    for forbidden in ["C:\\Users\\", "\\Users\\", "/home/", "/Users/"] {
        assert!(
            !msg.contains(forbidden),
            "error message leaks local path ({forbidden}): {msg:?}"
        );
    }
}

#[test]
fn sanitize_llm_prompt_banned_pattern_error_is_descriptive_and_no_path_leak() {
    use astraweave_security::{sanitize_llm_prompt, LLMValidator};

    let validator = LLMValidator {
        banned_patterns: vec!["system(".to_string()],
        allowed_domains: vec![],
        max_prompt_length: 10_000,
        enable_content_filtering: false,
    };

    let err = sanitize_llm_prompt("please system(exit)", &validator).unwrap_err();
    let msg = err.to_string();

    assert!(msg.contains("banned pattern"));
    assert_message_sane(&msg);
}

#[test]
fn sanitize_llm_prompt_too_long_error_is_descriptive_and_no_path_leak() {
    use astraweave_security::{sanitize_llm_prompt, LLMValidator};

    let validator = LLMValidator {
        banned_patterns: vec![],
        allowed_domains: vec![],
        max_prompt_length: 5,
        enable_content_filtering: false,
    };

    let err = sanitize_llm_prompt("this is too long", &validator).unwrap_err();
    let msg = err.to_string();

    assert!(msg.contains("Prompt too long"));
    assert_message_sane(&msg);
}
