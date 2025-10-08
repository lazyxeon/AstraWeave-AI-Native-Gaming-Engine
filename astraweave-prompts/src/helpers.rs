//! Helper utilities for prompt management
//!
//! This module provides various utility functions for prompt processing.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prompt validation utilities
pub struct PromptValidator;

/// Prompt formatting utilities
pub struct PromptFormatter;

/// Prompt analysis utilities
pub struct PromptAnalyzer;

/// Validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Maximum prompt length
    pub max_length: usize,
    /// Minimum prompt length
    pub min_length: usize,
    /// Required variables
    pub required_variables: Vec<String>,
    /// Forbidden patterns
    pub forbidden_patterns: Vec<String>,
}

/// Formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingOptions {
    /// Indent size for formatting
    pub indent_size: usize,
    /// Use tabs instead of spaces
    pub use_tabs: bool,
    /// Maximum line length
    pub max_line_length: usize,
}

/// Prompt analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Estimated token count
    pub estimated_tokens: usize,
    /// Complexity score (0-100)
    pub complexity_score: u8,
    /// Detected variables
    pub variables: Vec<String>,
    /// Readability score (0-100)
    pub readability_score: u8,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            max_length: 8000,
            min_length: 10,
            required_variables: Vec::new(),
            forbidden_patterns: Vec::new(),
        }
    }
}

impl Default for FormattingOptions {
    fn default() -> Self {
        Self {
            indent_size: 2,
            use_tabs: false,
            max_line_length: 80,
        }
    }
}

impl PromptValidator {
    /// Validate a prompt against rules
    pub fn validate(prompt: &str, rules: &ValidationRules) -> Result<()> {
        // Length validation
        if prompt.len() > rules.max_length {
            anyhow::bail!("Prompt too long: {} > {}", prompt.len(), rules.max_length);
        }

        if prompt.len() < rules.min_length {
            anyhow::bail!("Prompt too short: {} < {}", prompt.len(), rules.min_length);
        }

        // Required variables validation
        for var in &rules.required_variables {
            let placeholder = format!("{{{}}}", var);
            if !prompt.contains(&placeholder) {
                anyhow::bail!("Required variable '{}' not found in prompt", var);
            }
        }

        // Forbidden patterns validation
        for pattern in &rules.forbidden_patterns {
            if prompt.contains(pattern) {
                anyhow::bail!("Forbidden pattern '{}' found in prompt", pattern);
            }
        }

        Ok(())
    }

    /// Check if prompt is well-formed
    pub fn is_well_formed(prompt: &str) -> bool {
        // Check for balanced braces
        let mut brace_count = 0;
        for char in prompt.chars() {
            match char {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
            if brace_count < 0 {
                return false;
            }
        }

        brace_count == 0
    }
}

impl PromptFormatter {
    /// Format a prompt according to options
    pub fn format(prompt: &str, options: &FormattingOptions) -> String {
        let indent = if options.use_tabs {
            "\t".repeat(1)
        } else {
            " ".repeat(options.indent_size)
        };

        // Simple formatting - add indentation to lines
        prompt
            .lines()
            .map(|line| {
                if line.trim().is_empty() {
                    line.to_string()
                } else if line.starts_with("  ") || line.starts_with("\t") {
                    format!("{}{}", indent, line.trim())
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Minify a prompt (remove extra whitespace)
    pub fn minify(prompt: &str) -> String {
        prompt
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Pretty print a prompt with syntax highlighting placeholders
    pub fn pretty_print(prompt: &str) -> String {
        // Simple highlighting - wrap variables in markers
        let mut result = prompt.to_string();

        // Find variables and wrap them
        let variables = PromptAnalyzer::extract_variables(prompt);
        for var in variables {
            let placeholder = format!("{{{}}}", var);
            let highlighted = format!("**{{{}}}**", var);
            result = result.replace(&placeholder, &highlighted);
        }

        result
    }
}

impl PromptAnalyzer {
    /// Analyze a prompt and return analysis results
    pub fn analyze(prompt: &str) -> AnalysisResult {
        let estimated_tokens = Self::estimate_tokens(prompt);
        let complexity_score = Self::calculate_complexity(prompt);
        let variables = Self::extract_variables(prompt);
        let readability_score = Self::calculate_readability(prompt);

        AnalysisResult {
            estimated_tokens,
            complexity_score,
            variables,
            readability_score,
        }
    }

    /// Estimate token count (rough approximation)
    pub fn estimate_tokens(text: &str) -> usize {
        // Rough estimation: ~0.75 tokens per word
        let word_count = text.split_whitespace().count();
        ((word_count as f32) * 0.75) as usize
    }

    /// Calculate complexity score based on various factors
    pub fn calculate_complexity(prompt: &str) -> u8 {
        let mut score = 0u32;

        // Length factor (longer = more complex)
        score += (prompt.len() / 100).min(30) as u32;

        // Variable count factor
        let var_count = Self::extract_variables(prompt).len();
        score += (var_count * 5).min(25) as u32;

        // Nesting depth (based on brace levels)
        let mut max_depth = 0i32;
        let mut current_depth = 0i32;
        for char in prompt.chars() {
            match char {
                '{' => {
                    current_depth += 1;
                    max_depth = max_depth.max(current_depth);
                }
                '}' => current_depth = current_depth.saturating_sub(1),
                _ => {}
            }
        }
        score += (max_depth * 3).min(15) as u32;

        // Line count factor
        let line_count = prompt.lines().count();
        score += (line_count / 10).min(30) as u32;

        score.min(100) as u8
    }

    /// Extract variables from prompt
    pub fn extract_variables(prompt: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = prompt.chars().peekable();

        while let Some(char) = chars.next() {
            if char == '{' {
                let mut var_name = String::new();
                while let Some(char) = chars.next() {
                    if char == '}' {
                        break;
                    }
                    var_name.push(char);
                }

                if !var_name.is_empty() && !variables.contains(&var_name) {
                    variables.push(var_name);
                }
            }
        }

        variables
    }

    /// Calculate readability score
    pub fn calculate_readability(prompt: &str) -> u8 {
        let word_count = prompt.split_whitespace().count();
        let sentence_count = prompt.matches(['.', '!', '?']).count().max(1);
        let avg_words_per_sentence = word_count as f32 / sentence_count as f32;

        // Simple readability calculation
        let score = if avg_words_per_sentence <= 15.0 {
            90 - (avg_words_per_sentence * 2.0) as u8
        } else if avg_words_per_sentence <= 25.0 {
            70 - ((avg_words_per_sentence - 15.0) * 3.0) as u8
        } else {
            40 - ((avg_words_per_sentence - 25.0) * 1.5) as u8
        };

        score.max(0).min(100)
    }

    /// Generate prompt statistics
    pub fn generate_stats(prompt: &str) -> HashMap<String, String> {
        let mut stats = HashMap::new();

        stats.insert("character_count".to_string(), prompt.len().to_string());
        stats.insert("word_count".to_string(), prompt.split_whitespace().count().to_string());
        stats.insert("line_count".to_string(), prompt.lines().count().to_string());
        stats.insert("variable_count".to_string(), Self::extract_variables(prompt).len().to_string());
        stats.insert("estimated_tokens".to_string(), Self::estimate_tokens(prompt).to_string());

        stats
    }
}