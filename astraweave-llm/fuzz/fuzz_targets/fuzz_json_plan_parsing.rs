//! Fuzz target for JSON plan parsing.
//!
//! Tests that LLM responses with arbitrary JSON don't crash the parser.

#![no_main]

use libfuzzer_sys::fuzz_target;

use serde_json::Value;

// Simplified plan structure for fuzzing
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct FuzzPlanIntent {
    plan_id: Option<String>,
    steps: Option<Vec<FuzzActionStep>>,
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct FuzzActionStep {
    action: Option<String>,
    target: Option<Value>,
    parameters: Option<Value>,
}

fuzz_target!(|data: &[u8]| {
    // Try to parse as UTF-8 string
    if let Ok(s) = std::str::from_utf8(data) {
        // Try direct JSON parse
        let _ = serde_json::from_str::<FuzzPlanIntent>(s);
        
        // Try extracting JSON from code fences
        if let Some(start) = s.find("```json") {
            if let Some(end) = s[start..].find("```\n") {
                let json_str = &s[start + 7..start + end];
                let _ = serde_json::from_str::<FuzzPlanIntent>(json_str);
            }
        }
        
        // Try extracting from { } braces
        if let Some(start) = s.find('{') {
            let mut brace_count = 0;
            let mut end = start;
            for (i, c) in s[start..].char_indices() {
                match c {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            end = start + i + 1;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if end > start {
                let json_str = &s[start..end];
                let _ = serde_json::from_str::<FuzzPlanIntent>(json_str);
            }
        }
        
        // Try parsing as Value and extracting fields
        if let Ok(value) = serde_json::from_str::<Value>(s) {
            // Navigate to nested structures
            let _ = value.get("plan_id");
            let _ = value.get("steps");
            
            if let Some(steps) = value.get("steps").and_then(|s| s.as_array()) {
                for step in steps {
                    let _ = step.get("action");
                    let _ = step.get("target");
                }
            }
        }
    }
});
