//! Fuzz target for prompt template rendering.
//!
//! Tests variable substitution and template safety.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use std::collections::HashMap;

#[derive(Debug, Arbitrary)]
struct FuzzTemplateInput {
    template: String,
    variables: HashMap<String, String>,
}

fn render_template(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    
    // Replace {{var}} patterns
    for (key, value) in vars {
        let pattern = format!("{{{{{}}}}}", key);
        result = result.replace(&pattern, value);
    }
    
    result
}

fn escape_for_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

fn validate_template_safety(template: &str) -> bool {
    // Check for potentially dangerous patterns
    let dangerous = [
        "{{#each",      // Template loops
        "{{#if",        // Template conditionals
        "{{{",          // Unescaped output
        "{{>",          // Partials
        "{{!",          // Comments with potential injection
    ];
    
    !dangerous.iter().any(|p| template.contains(p))
}

fn extract_variable_names(template: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut chars = template.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next(); // consume second {
            let mut var_name = String::new();
            while let Some(&c) = chars.peek() {
                if c == '}' {
                    break;
                }
                var_name.push(chars.next().unwrap());
            }
            if !var_name.is_empty() {
                vars.push(var_name);
            }
            // consume closing }}
            if chars.next() == Some('}') {
                chars.next();
            }
        }
    }
    
    vars
}

fuzz_target!(|input: FuzzTemplateInput| {
    // Validate template safety
    let is_safe = validate_template_safety(&input.template);
    let _ = is_safe;
    
    // Extract expected variables
    let expected_vars = extract_variable_names(&input.template);
    
    // Check for missing variables
    let mut missing = Vec::new();
    for var in &expected_vars {
        if !input.variables.contains_key(var) {
            missing.push(var.clone());
        }
    }
    
    // Escape all variable values for JSON safety
    let escaped_vars: HashMap<String, String> = input.variables
        .iter()
        .map(|(k, v)| (k.clone(), escape_for_json(v)))
        .collect();
    
    // Render template
    let rendered = render_template(&input.template, &escaped_vars);
    
    // Verify output length is bounded
    let max_len = input.template.len() + input.variables.values().map(|v| v.len()).sum::<usize>() * 2;
    assert!(rendered.len() <= max_len + 1000, "Output grew unexpectedly large");
    
    // Try parsing rendered output as JSON (if it looks like JSON)
    if rendered.starts_with('{') || rendered.starts_with('[') {
        let _ = serde_json::from_str::<serde_json::Value>(&rendered);
    }
});
