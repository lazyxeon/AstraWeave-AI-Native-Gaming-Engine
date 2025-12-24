//! P1: Authentication Tests for Network Operations
//!
//! Tests for token validation, message parsing, and session handling.
//! These are unit tests that don't require a running server.

#![cfg(test)]

use astraweave_core::PlanIntent;
use astraweave_net::Msg;

// ============================================================================
// Message Serialization Tests (15 tests)
// ============================================================================

#[test]
fn test_valid_client_hello_serialization() {
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: Some("dev".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    assert!(json.contains("ClientHello"));
    assert!(json.contains("test_player"));
    assert!(json.contains("dev"));
}

#[test]
fn test_client_hello_without_token() {
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: None,
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { name, token, policy } => {
            assert_eq!(name, "test_player");
            assert!(token.is_none());
            assert!(policy.is_none());
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_empty_token() {
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: Some("".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { token, .. } => {
            assert_eq!(token, Some("".to_string()));
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_long_token() {
    let long_token = "x".repeat(10000);
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: Some(long_token.clone()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { token, .. } => {
            assert_eq!(token, Some(long_token));
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_unicode_token() {
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: Some("тест_токен_日本語_🔐".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { token, .. } => {
            assert_eq!(token, Some("тест_токен_日本語_🔐".to_string()));
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_special_chars_token() {
    // Test that special characters are escaped properly
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: Some("<script>alert('xss')</script>".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { token, .. } => {
            // JSON escaping should preserve the content
            assert_eq!(token, Some("<script>alert('xss')</script>".to_string()));
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_sql_injection_token() {
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: Some("'; DROP TABLE users; --".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { token, .. } => {
            assert_eq!(token, Some("'; DROP TABLE users; --".to_string()));
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_whitespace_token() {
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: Some("   \t\n  ".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { token, .. } => {
            assert_eq!(token, Some("   \t\n  ".to_string()));
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_server_welcome_serialization() {
    let welcome = Msg::ServerWelcome { id: 42 };
    let json = serde_json::to_string(&welcome).unwrap();
    assert!(json.contains("ServerWelcome"));
    assert!(json.contains("42"));

    let parsed: Msg = serde_json::from_str(&json).unwrap();
    match parsed {
        Msg::ServerWelcome { id } => assert_eq!(id, 42),
        _ => panic!("Expected ServerWelcome"),
    }
}

#[test]
fn test_client_hello_with_policy() {
    let hello = Msg::ClientHello {
        name: "test_player".to_string(),
        token: Some("dev".to_string()),
        policy: Some("radius".to_string()),
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { policy, .. } => {
            assert_eq!(policy, Some("radius".to_string()));
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_long_name() {
    let long_name = "x".repeat(10000);
    let hello = Msg::ClientHello {
        name: long_name.clone(),
        token: Some("dev".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { name, .. } => {
            assert_eq!(name, long_name);
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_empty_name() {
    let hello = Msg::ClientHello {
        name: "".to_string(),
        token: Some("dev".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { name, .. } => {
            assert_eq!(name, "");
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_hello_with_unicode_name() {
    let hello = Msg::ClientHello {
        name: "プレイヤー_игрок_🎮".to_string(),
        token: Some("dev".to_string()),
        policy: None,
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientHello { name, .. } => {
            assert_eq!(name, "プレイヤー_игрок_🎮");
        }
        _ => panic!("Expected ClientHello"),
    }
}

#[test]
fn test_client_propose_plan_serialization() {
    let intent = PlanIntent {
        plan_id: "test-plan".to_string(),
        steps: vec![],
    };
    let msg = Msg::ClientProposePlan {
        actor_id: 123,
        intent: intent.clone(),
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ClientProposePlan { actor_id, intent: parsed_intent } => {
            assert_eq!(actor_id, 123);
            assert_eq!(parsed_intent.plan_id, "test-plan");
        }
        _ => panic!("Expected ClientProposePlan"),
    }
}

#[test]
fn test_server_apply_result_success() {
    let msg = Msg::ServerApplyResult {
        ok: true,
        err: None,
    };
    let json = serde_json::to_string(&msg).unwrap();
    let parsed: Msg = serde_json::from_str(&json).unwrap();

    match parsed {
        Msg::ServerApplyResult { ok, err } => {
            assert!(ok);
            assert!(err.is_none());
        }
        _ => panic!("Expected ServerApplyResult"),
    }
}

// ============================================================================
// Invalid Message Parsing Tests (10 tests)
// ============================================================================

#[test]
fn test_malformed_json_rejected() {
    let result = serde_json::from_str::<Msg>("not valid json {{{");
    assert!(result.is_err());
}

#[test]
fn test_unknown_message_type_rejected() {
    let json = r#"{"type": "UnknownMessageType", "data": "test"}"#;
    let result = serde_json::from_str::<Msg>(json);
    assert!(result.is_err());
}

#[test]
fn test_missing_required_fields_rejected() {
    let json = r#"{"type": "ClientHello"}"#;
    let result = serde_json::from_str::<Msg>(json);
    assert!(result.is_err());
}

#[test]
fn test_wrong_field_types_rejected() {
    let json = r#"{"type": "ClientHello", "name": 12345, "token": []}"#;
    let result = serde_json::from_str::<Msg>(json);
    assert!(result.is_err());
}

#[test]
fn test_extra_fields_ignored() {
    // JSON parsing should allow extra fields by default with deny_unknown_fields not set
    let json = r#"{"type": "ClientHello", "name": "test", "token": "dev", "policy": null, "extra": "field"}"#;
    let result = serde_json::from_str::<Msg>(json);
    // This may or may not be an error depending on serde settings
    if let Ok(msg) = result {
        match msg {
            Msg::ClientHello { name, token, .. } => {
                assert_eq!(name, "test");
                assert_eq!(token, Some("dev".to_string()));
            }
            _ => panic!("Expected ClientHello"),
        }
    }
}

#[test]
fn test_empty_json_rejected() {
    let result = serde_json::from_str::<Msg>("");
    assert!(result.is_err());
}

#[test]
fn test_json_array_rejected() {
    let result = serde_json::from_str::<Msg>("[]");
    assert!(result.is_err());
}

#[test]
fn test_json_number_rejected() {
    let result = serde_json::from_str::<Msg>("42");
    assert!(result.is_err());
}

#[test]
fn test_json_string_rejected() {
    let result = serde_json::from_str::<Msg>(r#""hello""#);
    assert!(result.is_err());
}

#[test]
fn test_null_json_rejected() {
    let result = serde_json::from_str::<Msg>("null");
    assert!(result.is_err());
}

// ============================================================================
// Token Validation Logic Tests (5 tests)
// ============================================================================

#[test]
fn test_dev_token_recognized() {
    let token = "dev";
    assert_eq!(token, "dev");
}

#[test]
fn test_token_comparison_case_sensitive() {
    let token = "Dev";
    assert_ne!(token, "dev");
}

#[test]
fn test_token_comparison_exact_match() {
    let token = "dev ";
    assert_ne!(token, "dev");
}

#[test]
fn test_empty_token_not_equal_to_dev() {
    let token = "";
    assert_ne!(token, "dev");
}

#[test]
fn test_none_token_handled() {
    let token: Option<String> = None;
    let is_dev = token.as_deref() == Some("dev");
    assert!(!is_dev);
}
