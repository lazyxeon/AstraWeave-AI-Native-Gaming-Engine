use astraweave_prompts::{
    context::{ContextValue, PromptContext}, library::{PromptLibrary, TemplateLibrary}, optimization::{ABTestingEngine, OptimizationConfig, OptimizationEngine}, template::PromptTemplate, TemplateEngine, TemplateMetadata
};
use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use std::sync::{Arc, Mutex};
use std::thread;

// ============================================================================
// Helpers Tests
// ============================================================================

#[test]
fn test_register_helper() {
    let mut engine = TemplateEngine::new();
    engine.register_helper(
        "hello",
        Box::new(
            |_: &Helper,
             _: &Handlebars,
             _: &Context,
             _: &mut RenderContext,
             out: &mut dyn Output|
             -> HelperResult {
                out.write("Hello World")?;
                Ok(())
            },
        ),
    );

    let template = PromptTemplate::new("test", "{{hello}}");
    engine.register_template("test", template).unwrap();
    let result = engine.render("test", &Default::default()).unwrap();
    assert_eq!(result, "Hello World");
}

#[test]
fn test_invoke_helper() {
    let mut engine = TemplateEngine::new();
    // Built-in helpers are registered by default in TemplateEngine::new()
    let template = PromptTemplate::new("test", "{{trim '  hello  '}}");
    engine.register_template("test", template).unwrap();
    let result = engine.render("test", &Default::default()).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_helper_with_arguments() {
    let mut engine = TemplateEngine::new();
    let template = PromptTemplate::new("test", "{{indent text 4}}");
    engine.register_template("test", template).unwrap();
    
    let mut context = PromptContext::new();
    context.set("text".to_string(), ContextValue::from("line1\nline2"));
    
    let result = engine.render("test", &context).unwrap();
    assert_eq!(result, "    line1\n    line2");
}

#[test]
fn test_helper_error_handling() {
    let mut engine = TemplateEngine::new();
    // Missing argument for trim
    let template = PromptTemplate::new("test", "{{trim}}");
    engine.register_template("test", template).unwrap();
    let result = engine.render("test", &Default::default());
    assert!(result.is_err());
}

#[test]
fn test_builtin_helpers() {
    let mut engine = TemplateEngine::new();
    
    // Uppercase
    let t1 = PromptTemplate::new("t1", "{{uppercase 'hello'}}");
    engine.register_template("t1", t1).unwrap();
    assert_eq!(engine.render("t1", &Default::default()).unwrap(), "HELLO");

    // Lowercase
    let t2 = PromptTemplate::new("t2", "{{lowercase 'HELLO'}}");
    engine.register_template("t2", t2).unwrap();
    assert_eq!(engine.render("t2", &Default::default()).unwrap(), "hello");

    // Length
    let t3 = PromptTemplate::new("t3", "{{length 'hello'}}");
    engine.register_template("t3", t3).unwrap();
    assert_eq!(engine.render("t3", &Default::default()).unwrap(), "5");
}

#[test]
fn test_helper_override() {
    let mut engine = TemplateEngine::new();
    
    // Override 'trim' to do nothing
    engine.register_helper(
        "trim",
        Box::new(
            |h: &Helper,
             _: &Handlebars,
             _: &Context,
             _: &mut RenderContext,
             out: &mut dyn Output|
             -> HelperResult {
                let param = h.param(0).unwrap();
                out.write(param.value().as_str().unwrap())?;
                Ok(())
            },
        ),
    );

    let template = PromptTemplate::new("test", "{{trim '  hello  '}}");
    engine.register_template("test", template).unwrap();
    let result = engine.render("test", &Default::default()).unwrap();
    assert_eq!(result, "  hello  "); // Should NOT be trimmed
}

#[test]
fn test_helper_thread_safety() {
    let engine = Arc::new(Mutex::new(TemplateEngine::new()));
    let mut handles = vec![];

    for _ in 0..10 {
        let engine_clone = engine.clone();
        handles.push(thread::spawn(move || {
            let mut engine = engine_clone.lock().unwrap();
            let template = PromptTemplate::new("test", "{{uppercase 'test'}}");
            engine.register_template("test", template).unwrap();
            let result = engine.render("test", &Default::default()).unwrap();
            assert_eq!(result, "TEST");
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_helper_debug_output() {
    // Not directly testable via public API easily, but we can verify helpers don't crash on debug inputs
    let mut engine = TemplateEngine::new();
    let template = PromptTemplate::new("test", "{{json 'test'}}");
    engine.register_template("test", template).unwrap();
    let result = engine.render("test", &Default::default()).unwrap();
    assert_eq!(result, "\"test\"");
}

// ============================================================================
// Library Tests
// ============================================================================

#[test]
fn test_load_library_from_directory() {
    // Create a temp directory with some templates
    let temp_dir = std::env::temp_dir().join("astraweave_test_lib");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("test1.hbs"), "content1").unwrap();
    std::fs::write(temp_dir.join("test2.hbs"), "content2").unwrap();

    let library = TemplateLibrary::load_from_directory(temp_dir.clone()).unwrap();
    let collection = library.get_collection("default").unwrap();
    
    assert!(collection.get_template("test1").is_some());
    assert!(collection.get_template("test2").is_some());
    assert_eq!(collection.get_template("test1").unwrap(), "content1");

    // Cleanup
    std::fs::remove_dir_all(temp_dir).unwrap();
}

#[test]
fn test_library_list_templates() {
    let mut library = PromptLibrary::new();
    library.add_template("t1", PromptTemplate::new("t1", "c1"));
    library.add_template("t2", PromptTemplate::new("t2", "c2"));

    let list = library.list_templates();
    assert_eq!(list.len(), 2);
    assert!(list.contains(&"t1".to_string()));
    assert!(list.contains(&"t2".to_string()));
}

#[test]
fn test_library_get_template() {
    let mut library = PromptLibrary::new();
    library.add_template("t1", PromptTemplate::new("t1", "content"));
    
    let t = library.get_template("t1").unwrap();
    assert_eq!(t.template, "content");
    
    assert!(library.get_template("missing").is_err());
}

#[test]
fn test_library_delete_template() {
    let mut library = PromptLibrary::new();
    library.add_template("t1", PromptTemplate::new("t1", "content"));
    
    assert!(library.get_template("t1").is_ok());
    library.delete_template("t1");
    assert!(library.get_template("t1").is_err());
}

#[test]
fn test_library_template_metadata() {
    // PromptLibrary uses PromptTemplate which has metadata support
    let mut library = PromptLibrary::new();
    let mut template = PromptTemplate::new("t1", "content");
    
    let mut metadata = TemplateMetadata::default();
    metadata.author = Some("me".to_string());
    template.metadata = Some(metadata);
    
    library.add_template("t1", template);
    let t = library.get_template("t1").unwrap();
    assert_eq!(t.metadata.unwrap().author.unwrap(), "me");
}

// ============================================================================
// Optimization Tests
// ============================================================================

#[test]
fn test_optimize_template_compilation() {
    // OptimizationEngine doesn't expose compilation caching directly, 
    // but we can test that optimize_prompt works
    let mut config = OptimizationConfig::default();
    config.max_prompt_length = 0; // Force compression
    let mut engine = OptimizationEngine::new(config);
    let prompt = "  hello  ";
    let optimized = engine.optimize_prompt(prompt).unwrap();
    // Default config enables compression (trimming)
    assert_eq!(optimized, "hello");
}

#[test]
fn test_ab_testing_variants() {
    let mut engine = ABTestingEngine::new();
    engine.register_variant("welcome_msg".to_string(), "A".to_string());
    engine.register_variant("welcome_msg".to_string(), "B".to_string());

    // First selection should be A (index 0)
    let v1 = engine.select_variant("welcome_msg").unwrap();
    assert_eq!(v1, "A");

    // Second selection should be B (index 1)
    let v2 = engine.select_variant("welcome_msg").unwrap();
    assert_eq!(v2, "B");

    // Third selection should be A (index 0)
    let v3 = engine.select_variant("welcome_msg").unwrap();
    assert_eq!(v3, "A");
}

#[test]
fn test_ab_testing_metrics() {
    let mut engine = ABTestingEngine::new();
    engine.register_variant("test".to_string(), "A".to_string());
    
    engine.select_variant("test");
    engine.record_success("test", "A");
    
    let metrics = engine.get_metrics("test").unwrap();
    assert_eq!(*metrics.selections.get("A").unwrap(), 1);
    assert_eq!(*metrics.successes.get("A").unwrap(), 1);
}

#[test]
fn test_optimization_benchmarks() {
    let mut engine = OptimizationEngine::new(OptimizationConfig::default());
    let prompt = "test prompt";
    
    let start = std::time::Instant::now();
    let _ = engine.optimize_prompt(prompt).unwrap();
    let elapsed = start.elapsed();
    
    assert!(elapsed.as_millis() < 5); // Should be very fast
}

#[test]
fn test_template_precompilation() {
    // Verify we can register and render without errors (pre-compilation check)
    let mut engine = TemplateEngine::new();
    let template = PromptTemplate::new("test", "{{invalid}"); // Invalid syntax
    // register_template compiles it, so this should fail
    assert!(engine.register_template("test", template).is_err());
}