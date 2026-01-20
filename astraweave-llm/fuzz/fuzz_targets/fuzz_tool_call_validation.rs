//! Fuzz target for tool call validation.
//!
//! Tests that arbitrary tool calls don't cause crashes in validation logic.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Arbitrary)]
struct FuzzToolCall {
    name: String,
    parameters: HashMap<String, FuzzParamValue>,
}

#[derive(Debug, Clone, Arbitrary)]
enum FuzzParamValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<FuzzParamValue>),
}

struct ToolRegistry {
    tools: HashSet<String>,
    required_params: HashMap<String, Vec<String>>,
}

impl ToolRegistry {
    fn new() -> Self {
        let mut tools = HashSet::new();
        tools.insert("MoveTo".to_string());
        tools.insert("Attack".to_string());
        tools.insert("Defend".to_string());
        tools.insert("UseItem".to_string());
        tools.insert("Wait".to_string());
        
        let mut required_params = HashMap::new();
        required_params.insert("MoveTo".to_string(), vec!["x".to_string(), "y".to_string()]);
        required_params.insert("Attack".to_string(), vec!["target".to_string()]);
        required_params.insert("UseItem".to_string(), vec!["item_id".to_string()]);
        
        Self { tools, required_params }
    }
    
    fn validate_tool(&self, name: &str) -> bool {
        self.tools.contains(name)
    }
    
    fn validate_params(&self, name: &str, params: &HashMap<String, FuzzParamValue>) -> bool {
        if let Some(required) = self.required_params.get(name) {
            for req in required {
                if !params.contains_key(req) {
                    return false;
                }
            }
        }
        true
    }
}

fn sanitize_tool_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .take(64) // Limit length
        .collect()
}

fn validate_param_value(value: &FuzzParamValue, max_depth: usize) -> bool {
    if max_depth == 0 {
        return false; // Too deep
    }
    
    match value {
        FuzzParamValue::Null | FuzzParamValue::Bool(_) | FuzzParamValue::Int(_) => true,
        FuzzParamValue::Float(f) => f.is_finite(),
        FuzzParamValue::String(s) => s.len() <= 10000, // Limit string length
        FuzzParamValue::Array(arr) => {
            arr.len() <= 100 && arr.iter().all(|v| validate_param_value(v, max_depth - 1))
        }
    }
}

fuzz_target!(|call: FuzzToolCall| {
    let registry = ToolRegistry::new();
    
    // Sanitize tool name
    let sanitized_name = sanitize_tool_name(&call.name);
    
    // Validate tool exists
    let tool_valid = registry.validate_tool(&sanitized_name);
    
    // Validate parameters if tool is valid
    if tool_valid {
        let params_valid = registry.validate_params(&sanitized_name, &call.parameters);
        let _ = params_valid;
    }
    
    // Validate each parameter value
    for (key, value) in &call.parameters {
        // Key validation
        let key_valid = key.len() <= 64 && key.chars().all(|c| c.is_alphanumeric() || c == '_');
        let _ = key_valid;
        
        // Value validation
        let value_valid = validate_param_value(value, 10);
        let _ = value_valid;
    }
    
    // Serialize to JSON for output
    let json_params: HashMap<String, serde_json::Value> = call.parameters
        .iter()
        .map(|(k, v)| {
            let json_v = match v {
                FuzzParamValue::Null => serde_json::Value::Null,
                FuzzParamValue::Bool(b) => serde_json::Value::Bool(*b),
                FuzzParamValue::Int(i) => serde_json::json!(*i),
                FuzzParamValue::Float(f) => {
                    if f.is_finite() {
                        serde_json::json!(*f)
                    } else {
                        serde_json::Value::Null
                    }
                }
                FuzzParamValue::String(s) => serde_json::Value::String(s.clone()),
                FuzzParamValue::Array(_) => serde_json::Value::Array(vec![]),
            };
            (k.clone(), json_v)
        })
        .collect();
    
    let _ = serde_json::to_string(&json_params);
});
