//! Context management for prompt templates
//!
//! This module provides context and variable management for templates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prompt context manager
#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    /// Context variables
    variables: HashMap<String, ContextValue>,
    /// Nested contexts
    scopes: Vec<HashMap<String, ContextValue>>,
}

/// Context value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContextValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ContextValue>),
    Object(HashMap<String, ContextValue>),
}

impl PromptContext {
    /// Create a new context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable in the current context
    pub fn set(&mut self, key: String, value: ContextValue) {
        self.variables.insert(key, value);
    }

    /// Get a variable from the context
    pub fn get(&self, key: &str) -> Option<&ContextValue> {
        // Check current scope first
        if let Some(value) = self.variables.get(key) {
            return Some(value);
        }

        // Check parent scopes
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(key) {
                return Some(value);
            }
        }

        None
    }

    /// Push a new scope
    pub fn push_scope(&mut self) {
        let old_vars = std::mem::take(&mut self.variables);
        self.scopes.push(old_vars);
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) -> Option<HashMap<String, ContextValue>> {
        if let Some(parent_vars) = self.scopes.pop() {
            let popped_vars = std::mem::replace(&mut self.variables, parent_vars);
            Some(popped_vars)
        } else {
            None
        }
    }

    /// Convert context to string variables for template processing
    pub fn to_string_map(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();

        // Add variables from all scopes
        for scope in &self.scopes {
            for (key, value) in scope {
                result.insert(key.clone(), value.to_string());
            }
        }

        // Add current variables (override scope values)
        for (key, value) in &self.variables {
            result.insert(key.clone(), value.to_string());
        }

        result
    }

    /// Convert context to JSON value for Handlebars processing
    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();

        // Add variables from all scopes
        for scope in &self.scopes {
            for (key, value) in scope {
                map.insert(key.clone(), value.to_json());
            }
        }

        // Add current variables (override scope values)
        for (key, value) in &self.variables {
            map.insert(key.clone(), value.to_json());
        }

        serde_json::Value::Object(map)
    }
}

impl ContextValue {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            ContextValue::String(s) => serde_json::Value::String(s.clone()),
            ContextValue::Number(n) => serde_json::Number::from_f64(*n)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            ContextValue::Boolean(b) => serde_json::Value::Bool(*b),
            ContextValue::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect())
            }
            ContextValue::Object(obj) => {
                let mut map = serde_json::Map::new();
                for (k, v) in obj {
                    map.insert(k.clone(), v.to_json());
                }
                serde_json::Value::Object(map)
            }
        }
    }
}

impl ToString for ContextValue {
    fn to_string(&self) -> String {
        match self {
            ContextValue::String(s) => s.clone(),
            ContextValue::Number(n) => n.to_string(),
            ContextValue::Boolean(b) => b.to_string(),
            ContextValue::Array(arr) => {
                format!(
                    "[{}]",
                    arr.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ContextValue::Object(obj) => {
                format!(
                    "{{{}}}",
                    obj.iter()
                        .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}

impl From<String> for ContextValue {
    fn from(value: String) -> Self {
        ContextValue::String(value)
    }
}

impl From<&str> for ContextValue {
    fn from(value: &str) -> Self {
        ContextValue::String(value.to_string())
    }
}

impl From<f64> for ContextValue {
    fn from(value: f64) -> Self {
        ContextValue::Number(value)
    }
}

impl From<bool> for ContextValue {
    fn from(value: bool) -> Self {
        ContextValue::Boolean(value)
    }
}

impl From<Vec<ContextValue>> for ContextValue {
    fn from(value: Vec<ContextValue>) -> Self {
        ContextValue::Array(value)
    }
}

impl From<Vec<String>> for ContextValue {
    fn from(value: Vec<String>) -> Self {
        ContextValue::Array(value.into_iter().map(ContextValue::String).collect())
    }
}

impl From<Vec<&str>> for ContextValue {
    fn from(value: Vec<&str>) -> Self {
        ContextValue::Array(
            value
                .into_iter()
                .map(|s| ContextValue::String(s.to_string()))
                .collect(),
        )
    }
}
