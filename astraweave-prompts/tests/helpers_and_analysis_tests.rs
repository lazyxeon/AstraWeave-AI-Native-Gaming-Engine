/// Tests for Helper Functions, Validators, Formatters, and Analyzers
/// Coverage for astraweave-prompts helpers and optimization modules

use astraweave_prompts::{
    PromptContext, PromptTemplate, TemplateEngine, ContextValue,
    helpers::{
        PromptValidator, PromptFormatter, PromptAnalyzer,
        ValidationRules, FormattingOptions, AnalysisResult,
    },
};
use std::collections::HashMap;

// ========================================
// Helper Function Tests (Handlebars Helpers)
// ========================================

#[test]
fn test_helper_uppercase() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "upper",
        PromptTemplate::new("u", "{{uppercase text}}")
    ).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("hello".to_string()));

    let result = engine.render("upper", &ctx).unwrap();
    assert_eq!(result, "HELLO");
}

#[test]
fn test_helper_lowercase() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "lower",
        PromptTemplate::new("l", "{{lowercase text}}")
    ).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("WORLD".to_string()));

    let result = engine.render("lower", &ctx).unwrap();
    assert_eq!(result, "world");
}

#[test]
fn test_helper_trim() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "trim",
        PromptTemplate::new("t", "{{trim text}}")
    ).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("  spaces  ".to_string()));

    let result = engine.render("trim", &ctx).unwrap();
    assert_eq!(result, "spaces");
}

#[test]
fn test_helper_json() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "json",
        PromptTemplate::new("j", "{{json data}}")
    ).unwrap();

    let mut data_obj = HashMap::new();
    data_obj.insert("key".to_string(), ContextValue::String("value".to_string()));

    let mut ctx = PromptContext::new();
    ctx.set("data".to_string(), ContextValue::Object(data_obj));

    let result = engine.render("json", &ctx).unwrap();
    assert!(result.contains("key"));
    assert!(result.contains("value"));
}

#[test]
fn test_helper_length_string() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "len",
        PromptTemplate::new("l", "{{length text}}")
    ).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("hello".to_string()));

    let result = engine.render("len", &ctx).unwrap();
    assert_eq!(result, "5");
}

#[test]
fn test_helper_length_array() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "len",
        PromptTemplate::new("l", "{{length items}}")
    ).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("items".to_string(), ContextValue::from(vec!["a", "b", "c"]));

    let result = engine.render("len", &ctx).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_helper_indent_default() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "indent",
        PromptTemplate::new("i", "{{indent text}}")
    ).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("line1\nline2".to_string()));

    let result = engine.render("indent", &ctx).unwrap();
    assert!(result.contains("  line1"));
    assert!(result.contains("  line2"));
}

#[test]
fn test_helper_indent_custom_spaces() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "indent",
        PromptTemplate::new("i", "{{indent text 4}}")
    ).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("text".to_string(), ContextValue::String("line".to_string()));

    let result = engine.render("indent", &ctx).unwrap();
    assert!(result.starts_with("    "));
}

// ========================================
// PromptValidator Tests
// ========================================

#[test]
fn test_validator_valid_prompt() {
    let rules = ValidationRules::default();
    let prompt = "This is a valid prompt with good length.";
    
    let result = PromptValidator::validate(prompt, &rules);
    assert!(result.is_ok());
}

#[test]
fn test_validator_prompt_too_long() {
    let rules = ValidationRules {
        max_length: 20,
        ..Default::default()
    };
    let prompt = "This prompt is definitely too long for the limit";
    
    let result = PromptValidator::validate(prompt, &rules);
    assert!(result.is_err());
}

#[test]
fn test_validator_prompt_too_short() {
    let rules = ValidationRules {
        min_length: 50,
        ..Default::default()
    };
    let prompt = "Short";
    
    let result = PromptValidator::validate(prompt, &rules);
    assert!(result.is_err());
}

#[test]
fn test_validator_required_variables() {
    let rules = ValidationRules {
        required_variables: vec!["name".to_string(), "age".to_string()],
        ..Default::default()
    };
    let prompt = "Hello {name}, you are {age} years old.";
    
    let result = PromptValidator::validate(prompt, &rules);
    assert!(result.is_ok());
}

#[test]
fn test_validator_missing_required_variable() {
    let rules = ValidationRules {
        required_variables: vec!["missing".to_string()],
        ..Default::default()
    };
    let prompt = "This prompt lacks the required variable.";
    
    let result = PromptValidator::validate(prompt, &rules);
    assert!(result.is_err());
}

#[test]
fn test_validator_forbidden_pattern() {
    let rules = ValidationRules {
        forbidden_patterns: vec!["banned".to_string()],
        ..Default::default()
    };
    let prompt = "This contains a banned word.";
    
    let result = PromptValidator::validate(prompt, &rules);
    assert!(result.is_err());
}

#[test]
fn test_validator_well_formed_balanced() {
    let prompt = "Text with {{variable}} is balanced.";
    assert!(PromptValidator::is_well_formed(prompt));
}

#[test]
fn test_validator_well_formed_unbalanced_open() {
    let prompt = "Text with {{variable is not balanced.";
    assert!(!PromptValidator::is_well_formed(prompt));
}

#[test]
fn test_validator_well_formed_unbalanced_close() {
    let prompt = "Text with variable}} is not balanced.";
    assert!(!PromptValidator::is_well_formed(prompt));
}

#[test]
fn test_validator_well_formed_no_braces() {
    let prompt = "Simple text without any braces.";
    assert!(PromptValidator::is_well_formed(prompt));
}

// ========================================
// PromptFormatter Tests
// ========================================

#[test]
fn test_formatter_format_with_spaces() {
    let options = FormattingOptions {
        indent_size: 2,
        use_tabs: false,
        max_line_length: 80,
    };
    let prompt = "  indented line\nnormal line";
    
    let result = PromptFormatter::format(prompt, &options);
    assert!(result.contains("indented line"));
}

#[test]
fn test_formatter_format_with_tabs() {
    let options = FormattingOptions {
        indent_size: 2,
        use_tabs: true,
        max_line_length: 80,
    };
    let prompt = "  indented line";
    
    let result = PromptFormatter::format(prompt, &options);
    assert!(result.contains("\t"));
}

#[test]
fn test_formatter_minify() {
    let prompt = "Line 1\n  Line 2  \n   Line 3   \n\n";
    
    let result = PromptFormatter::minify(prompt);
    assert_eq!(result, "Line 1 Line 2 Line 3");
}

#[test]
fn test_formatter_minify_empty_lines() {
    let prompt = "Content\n\n\nMore content";
    
    let result = PromptFormatter::minify(prompt);
    assert_eq!(result, "Content More content");
}

#[test]
fn test_formatter_pretty_print() {
    let prompt = "Hello {name}, you have {count} items.";
    
    let result = PromptFormatter::pretty_print(prompt);
    assert!(result.contains("**{name}**"));
    assert!(result.contains("**{count}**"));
}

// ========================================
// PromptAnalyzer Tests
// ========================================

#[test]
fn test_analyzer_analyze() {
    let prompt = "This is a test prompt with {variable1} and {variable2}.";
    
    let result = PromptAnalyzer::analyze(prompt);
    assert!(result.estimated_tokens > 0);
    assert!(result.complexity_score > 0);
    assert_eq!(result.variables.len(), 2);
    assert!(result.readability_score > 0);
}

#[test]
fn test_analyzer_estimate_tokens() {
    let prompt = "one two three four five";
    let tokens = PromptAnalyzer::estimate_tokens(prompt);
    
    // Should be around 3-4 tokens (5 words * 0.75)
    assert!(tokens >= 3 && tokens <= 4);
}

#[test]
fn test_analyzer_estimate_tokens_empty() {
    let prompt = "";
    let tokens = PromptAnalyzer::estimate_tokens(prompt);
    assert_eq!(tokens, 0);
}

#[test]
fn test_analyzer_calculate_complexity_simple() {
    let prompt = "Simple text.";
    let score = PromptAnalyzer::calculate_complexity(prompt);
    
    // Simple prompt should have low complexity
    assert!(score < 20);
}

#[test]
fn test_analyzer_calculate_complexity_complex() {
    let mut complex_prompt = String::new();
    for i in 0..100 {
        complex_prompt.push_str(&format!("{{var{}}} ", i));
    }
    complex_prompt.push_str(&"x".repeat(500));
    
    let score = PromptAnalyzer::calculate_complexity(&complex_prompt);
    
    // Complex prompt should have higher score
    assert!(score > 30);
}

#[test]
fn test_analyzer_extract_variables() {
    let prompt = "Hello {name}, you have {count} messages from {sender}.";
    let vars = PromptAnalyzer::extract_variables(prompt);
    
    assert_eq!(vars.len(), 3);
    assert!(vars.contains(&"name".to_string()));
    assert!(vars.contains(&"count".to_string()));
    assert!(vars.contains(&"sender".to_string()));
}

#[test]
fn test_analyzer_extract_variables_none() {
    let prompt = "No variables here.";
    let vars = PromptAnalyzer::extract_variables(prompt);
    
    assert_eq!(vars.len(), 0);
}

#[test]
fn test_analyzer_extract_variables_duplicates() {
    let prompt = "{x} and {x} and {x}";
    let vars = PromptAnalyzer::extract_variables(prompt);
    
    // Should deduplicate
    assert_eq!(vars.len(), 1);
}

#[test]
fn test_analyzer_calculate_readability_easy() {
    let prompt = "Short. Simple. Easy. Words.";
    let score = PromptAnalyzer::calculate_readability(prompt);
    
    // Short sentences should be more readable
    assert!(score > 50);
}

#[test]
fn test_analyzer_calculate_readability_hard() {
    let prompt = "This is an extremely long and convoluted sentence that goes on and on and on without stopping, making it very difficult to read and understand, which is why it should have a lower readability score.";
    let score = PromptAnalyzer::calculate_readability(prompt);
    
    // Long sentences should be less readable
    assert!(score < 60);
}

#[test]
fn test_analyzer_generate_stats() {
    let prompt = "Test prompt with {var1} and {var2}.\nSecond line here.";
    let stats = PromptAnalyzer::generate_stats(prompt);
    
    assert!(stats.contains_key("character_count"));
    assert!(stats.contains_key("word_count"));
    assert!(stats.contains_key("line_count"));
    assert!(stats.contains_key("variable_count"));
    assert!(stats.contains_key("estimated_tokens"));
    
    assert_eq!(stats.get("variable_count").unwrap(), "2");
}

#[test]
fn test_analyzer_generate_stats_empty() {
    let prompt = "";
    let stats = PromptAnalyzer::generate_stats(prompt);
    
    assert_eq!(stats.get("character_count").unwrap(), "0");
    assert_eq!(stats.get("word_count").unwrap(), "0");
}

// ========================================
// Configuration Tests
// ========================================

#[test]
fn test_validation_rules_default() {
    let rules = ValidationRules::default();
    
    assert_eq!(rules.max_length, 8000);
    assert_eq!(rules.min_length, 10);
    assert!(rules.required_variables.is_empty());
    assert!(rules.forbidden_patterns.is_empty());
}

#[test]
fn test_formatting_options_default() {
    let options = FormattingOptions::default();
    
    assert_eq!(options.indent_size, 2);
    assert!(!options.use_tabs);
    assert_eq!(options.max_line_length, 80);
}

// ========================================
// Integration Tests with Helpers
// ========================================

#[test]
fn test_helpers_combined_in_template() {
    let mut engine = TemplateEngine::new();
    engine.register_template(
        "combined",
        PromptTemplate::new("c", "{{uppercase (trim name)}}: {{length items}} items")
    ).unwrap();

    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("  alice  ".to_string()));
    ctx.set("items".to_string(), ContextValue::from(vec!["a", "b", "c"]));

    let result = engine.render("combined", &ctx).unwrap();
    assert!(result.contains("ALICE"));
    assert!(result.contains("3 items"));
}

#[test]
fn test_validation_and_analysis_workflow() {
    let prompt = "Generate dialogue for {character_name} who is {character_mood}.";
    
    // Validate
    let rules = ValidationRules {
        required_variables: vec!["character_name".to_string(), "character_mood".to_string()],
        ..Default::default()
    };
    assert!(PromptValidator::validate(prompt, &rules).is_ok());
    
    // Analyze
    let analysis = PromptAnalyzer::analyze(prompt);
    assert_eq!(analysis.variables.len(), 2);
    assert!(analysis.estimated_tokens > 0);
    
    // Format
    let formatted = PromptFormatter::minify(prompt);
    assert!(!formatted.is_empty());
}
