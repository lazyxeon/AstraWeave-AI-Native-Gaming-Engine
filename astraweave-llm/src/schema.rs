//! LLM Output Schema Validation
//!
//! This module provides compile-time type-safe schema validation for LLM outputs.
//! It ensures LLM responses conform to expected schemas before deserialization.
//!
//! # Features
//! - Type-safe schema definitions via trait implementation
//! - JSON Schema generation for LLM prompts
//! - Validation before deserialization
//! - Clear error messages for schema violations
//!
//! # Example
//! ```rust
//! use astraweave_llm::schema::{LlmOutputSchema, ValidationError};
//! use astraweave_core::PlanIntent;
//!
//! // Validate LLM output conforms to PlanIntent schema
//! let llm_response = r#"{"plan_id": "p1", "steps": [{"act": "MoveTo", "x": 5, "y": 5}]}"#;
//! let plan = PlanIntent::parse_validated(llm_response)?;
//! ```

use anyhow::{bail, Result};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Error types for schema validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// JSON parsing failed
    ParseError(String),
    /// Missing required field
    MissingField { field: String, path: String },
    /// Field has wrong type
    WrongType {
        field: String,
        expected: String,
        actual: String,
    },
    /// Field value out of valid range
    OutOfRange {
        field: String,
        value: String,
        constraint: String,
    },
    /// Unknown field in strict mode
    UnknownField { field: String },
    /// Array length constraint violated
    ArrayLength {
        field: String,
        actual: usize,
        constraint: String,
    },
    /// Custom validation rule failed
    CustomRule { rule: String, message: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::ParseError(msg) => write!(f, "JSON parse error: {}", msg),
            ValidationError::MissingField { field, path } => {
                write!(f, "Missing required field '{}' at '{}'", field, path)
            }
            ValidationError::WrongType {
                field,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Field '{}' has wrong type: expected {}, got {}",
                    field, expected, actual
                )
            }
            ValidationError::OutOfRange {
                field,
                value,
                constraint,
            } => {
                write!(
                    f,
                    "Field '{}' value '{}' violates constraint: {}",
                    field, value, constraint
                )
            }
            ValidationError::UnknownField { field } => {
                write!(f, "Unknown field '{}' in strict mode", field)
            }
            ValidationError::ArrayLength {
                field,
                actual,
                constraint,
            } => {
                write!(
                    f,
                    "Array '{}' has {} elements, constraint: {}",
                    field, actual, constraint
                )
            }
            ValidationError::CustomRule { rule, message } => {
                write!(f, "Validation rule '{}' failed: {}", rule, message)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Trait for types that can be validated against an LLM output schema
pub trait LlmOutputSchema: Sized + DeserializeOwned {
    /// Human-readable name for this schema (used in error messages)
    fn schema_name() -> &'static str;

    /// Generate a JSON schema description for LLM prompts
    ///
    /// This should return a string that can be included in LLM prompts
    /// to guide the model toward generating valid output.
    fn schema_description() -> String;

    /// Validate a JSON value against this schema's constraints
    ///
    /// This is called after basic JSON parsing but before deserialization.
    /// Returns Ok(()) if valid, or a list of validation errors.
    fn validate_json(value: &Value) -> Vec<ValidationError>;

    /// Parse and validate LLM output in one step
    ///
    /// This is the primary entry point for schema validation.
    fn parse_validated(json_text: &str) -> Result<Self> {
        // Step 1: Parse JSON
        let value: Value = serde_json::from_str(json_text.trim())
            .map_err(|e| anyhow::anyhow!("JSON parse error: {}", e))?;

        // Step 2: Validate structure
        let errors = Self::validate_json(&value);
        if !errors.is_empty() {
            let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            bail!(
                "Schema validation failed for {}:\n  - {}",
                Self::schema_name(),
                error_messages.join("\n  - ")
            );
        }

        // Step 3: Deserialize
        serde_json::from_value(value).map_err(|e| anyhow::anyhow!("Deserialization error: {}", e))
    }

    /// Check if a value is present and non-null
    fn require_field(value: &Value, field: &str) -> Option<ValidationError> {
        if value.get(field).is_none() || value.get(field) == Some(&Value::Null) {
            Some(ValidationError::MissingField {
                field: field.to_string(),
                path: "/".to_string(),
            })
        } else {
            None
        }
    }

    /// Check if a field is a string
    fn require_string(value: &Value, field: &str) -> Option<ValidationError> {
        match value.get(field) {
            Some(v) if v.is_string() => None,
            Some(v) => Some(ValidationError::WrongType {
                field: field.to_string(),
                expected: "string".to_string(),
                actual: json_type_name(v),
            }),
            None => Some(ValidationError::MissingField {
                field: field.to_string(),
                path: "/".to_string(),
            }),
        }
    }

    /// Check if a field is an array
    fn require_array(value: &Value, field: &str) -> Option<ValidationError> {
        match value.get(field) {
            Some(v) if v.is_array() => None,
            Some(v) => Some(ValidationError::WrongType {
                field: field.to_string(),
                expected: "array".to_string(),
                actual: json_type_name(v),
            }),
            None => Some(ValidationError::MissingField {
                field: field.to_string(),
                path: "/".to_string(),
            }),
        }
    }

    /// Check if a field is a number
    fn require_number(value: &Value, field: &str) -> Option<ValidationError> {
        match value.get(field) {
            Some(v) if v.is_number() => None,
            Some(v) => Some(ValidationError::WrongType {
                field: field.to_string(),
                expected: "number".to_string(),
                actual: json_type_name(v),
            }),
            None => Some(ValidationError::MissingField {
                field: field.to_string(),
                path: "/".to_string(),
            }),
        }
    }
}

/// Get a human-readable type name for a JSON value
fn json_type_name(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct TestPlan {
        plan_id: String,
        steps: Vec<TestStep>,
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct TestStep {
        act: String,
        x: Option<i32>,
        y: Option<i32>,
    }

    impl LlmOutputSchema for TestPlan {
        fn schema_name() -> &'static str {
            "TestPlan"
        }

        fn schema_description() -> String {
            r#"{"plan_id": "string", "steps": [{"act": "string", "x": int?, "y": int?}]}"#
                .to_string()
        }

        fn validate_json(value: &Value) -> Vec<ValidationError> {
            let mut errors = Vec::new();

            // Check required fields
            if let Some(err) = Self::require_string(value, "plan_id") {
                errors.push(err);
            }
            if let Some(err) = Self::require_array(value, "steps") {
                errors.push(err);
            }

            // Validate steps array
            if let Some(steps) = value.get("steps").and_then(|v| v.as_array()) {
                if steps.is_empty() {
                    errors.push(ValidationError::ArrayLength {
                        field: "steps".to_string(),
                        actual: 0,
                        constraint: "minimum 1 step required".to_string(),
                    });
                }

                for (i, step) in steps.iter().enumerate() {
                    if step.get("act").is_none() {
                        errors.push(ValidationError::MissingField {
                            field: "act".to_string(),
                            path: format!("/steps/{}", i),
                        });
                    }
                }
            }

            errors
        }
    }

    #[test]
    fn test_valid_plan() {
        let json = r#"{"plan_id": "test-1", "steps": [{"act": "MoveTo", "x": 5, "y": 10}]}"#;
        let result = TestPlan::parse_validated(json);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.plan_id, "test-1");
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].act, "MoveTo");
    }

    #[test]
    fn test_missing_plan_id() {
        let json = r#"{"steps": [{"act": "MoveTo"}]}"#;
        let result = TestPlan::parse_validated(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("plan_id"),
            "Error should mention plan_id: {}",
            err
        );
    }

    #[test]
    fn test_wrong_type_steps() {
        let json = r#"{"plan_id": "test", "steps": "not-an-array"}"#;
        let result = TestPlan::parse_validated(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("wrong type"),
            "Error should mention wrong type: {}",
            err
        );
    }

    #[test]
    fn test_empty_steps_array() {
        let json = r#"{"plan_id": "test", "steps": []}"#;
        let result = TestPlan::parse_validated(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("minimum 1 step"),
            "Error should mention minimum steps: {}",
            err
        );
    }

    #[test]
    fn test_step_missing_act() {
        let json = r#"{"plan_id": "test", "steps": [{"x": 5, "y": 10}]}"#;
        let result = TestPlan::parse_validated(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("act"),
            "Error should mention act field: {}",
            err
        );
    }

    #[test]
    fn test_invalid_json() {
        let json = r#"not valid json"#;
        let result = TestPlan::parse_validated(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("parse error"),
            "Error should mention parse error: {}",
            err
        );
    }

    #[test]
    fn test_schema_description() {
        let desc = TestPlan::schema_description();
        assert!(desc.contains("plan_id"));
        assert!(desc.contains("steps"));
    }
}
