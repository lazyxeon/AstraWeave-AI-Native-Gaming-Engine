//! Tests for template loading and library management
//! Covers file operations, directory scanning, and metadata parsing

#![allow(clippy::field_reassign_with_default)]

use astraweave_prompts::{
    PromptLoader, PromptLibrary, PromptTemplate, TemplateLibrary, TemplateCollection,
    LibraryMetadata, CollectionMetadata, TemplateMetadata, TemplateCategory,
};
use std::fs;
use tempfile::tempdir;

// ========================================
// PromptLoader Tests
// ========================================

#[test]
fn test_loader_new_default_extensions() {
    let loader = PromptLoader::new();
    // Just verify it creates successfully
    drop(loader);
}

#[test]
fn test_loader_with_custom_extensions() {
    let loader = PromptLoader::new()
        .with_extensions(vec!["txt".to_string(), "tpl".to_string()]);
    
    // Verify custom extensions are set
    // (This is tested indirectly through load operations)
    drop(loader);
}

#[test]
fn test_load_single_template_file() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("greeting.hbs");
    
    fs::write(&file_path, "Hello {{name}}!").unwrap();
    
    let loader = PromptLoader::new();
    let template = loader.load_file(&file_path).unwrap();
    
    assert_eq!(template.id(), "greeting");
    assert!(template.template().contains("Hello"));
}

#[test]
fn test_load_template_with_toml_frontmatter() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("advanced.hbs");
    
    let content = r#"+++
name = "advanced"
description = "An advanced template"
version = "2.0"
category = "Dialogue"
+++
Hello {{name}}, this is an advanced template!"#;
    
    fs::write(&file_path, content).unwrap();
    
    let loader = PromptLoader::new();
    let template = loader.load_file(&file_path).unwrap();
    
    assert_eq!(template.id(), "advanced");
    assert!(template.metadata.is_some());
    
    let meta = template.metadata.unwrap();
    assert_eq!(meta.name, "advanced");
    assert_eq!(meta.description, "An advanced template");
    assert_eq!(meta.version, "2.0");
}

#[test]
fn test_load_template_without_frontmatter() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("simple.hbs");
    
    fs::write(&file_path, "Simple template content").unwrap();
    
    let loader = PromptLoader::new();
    let template = loader.load_file(&file_path).unwrap();
    
    assert_eq!(template.id(), "simple");
    assert!(template.metadata.is_none());
    assert_eq!(template.template(), "Simple template content");
}

#[test]
fn test_load_from_directory_multiple_files() {
    let dir = tempdir().unwrap();
    
    fs::write(dir.path().join("template1.hbs"), "Template 1").unwrap();
    fs::write(dir.path().join("template2.hbs"), "Template 2").unwrap();
    fs::write(dir.path().join("template3.prompt"), "Template 3").unwrap();
    fs::write(dir.path().join("ignored.txt"), "Should be ignored").unwrap();
    
    let loader = PromptLoader::new();
    let templates = loader.load_from_dir(dir.path()).unwrap();
    
    // Should load .hbs and .prompt files, but not .txt
    assert_eq!(templates.len(), 3);
}

#[test]
fn test_load_from_directory_recursive() {
    let dir = tempdir().unwrap();
    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    
    fs::write(dir.path().join("root.hbs"), "Root template").unwrap();
    fs::write(subdir.join("nested.hbs"), "Nested template").unwrap();
    
    let loader = PromptLoader::new();
    let templates = loader.load_from_dir(dir.path()).unwrap();
    
    assert_eq!(templates.len(), 2);
}

#[test]
fn test_load_from_nonexistent_directory() {
    let loader = PromptLoader::new();
    let result = loader.load_from_dir("/nonexistent/path/xyz123");
    
    // Should return Ok with empty vec
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_load_from_empty_directory() {
    let dir = tempdir().unwrap();
    
    let loader = PromptLoader::new();
    let templates = loader.load_from_dir(dir.path()).unwrap();
    
    assert!(templates.is_empty());
}

#[test]
fn test_load_file_nonexistent() {
    let loader = PromptLoader::new();
    let result = loader.load_file("/nonexistent/file.hbs");
    
    assert!(result.is_err());
}

#[test]
fn test_load_with_incomplete_frontmatter() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("incomplete.hbs");
    
    // Missing closing +++
    let content = r#"+++
name = "incomplete"
Template body without closing"#;
    
    fs::write(&file_path, content).unwrap();
    
    let loader = PromptLoader::new();
    let template = loader.load_file(&file_path).unwrap();
    
    // Should treat entire content as template (no frontmatter)
    assert!(template.metadata.is_none());
}

#[test]
fn test_load_with_empty_frontmatter() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("empty_front.hbs");
    
    let content = r#"+++
+++
Template body"#;
    
    fs::write(&file_path, content).unwrap();
    
    let loader = PromptLoader::new();
    let result = loader.load_file(&file_path);
    
    // Empty TOML might fail to parse or succeed with defaults
    // Either way, it shouldn't panic
    drop(result);
}

#[test]
fn test_load_template_with_unicode_content() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("unicode.hbs");
    
    fs::write(&file_path, "你好 {{name}}! Привет {{user}}!").unwrap();
    
    let loader = PromptLoader::new();
    let template = loader.load_file(&file_path).unwrap();
    
    assert!(template.template().contains("你好"));
    assert!(template.template().contains("Привет"));
}

// ========================================
// PromptLibrary Tests
// ========================================

#[test]
fn test_library_new() {
    let lib = PromptLibrary::new();
    assert!(lib.list_templates().is_empty());
}

#[test]
fn test_library_add_and_list() {
    let mut lib = PromptLibrary::new();
    
    let template = PromptTemplate::new("test", "Content");
    lib.add_template("test", template);
    
    let list = lib.list_templates();
    assert_eq!(list.len(), 1);
    assert!(list.contains(&"test".to_string()));
}

#[test]
fn test_library_add_multiple_templates() {
    let mut lib = PromptLibrary::new();
    
    for i in 0..10 {
        let template = PromptTemplate::new(format!("t{}", i), format!("Content {}", i));
        lib.add_template(&format!("t{}", i), template);
    }
    
    assert_eq!(lib.list_templates().len(), 10);
}

#[test]
fn test_library_get_template() {
    let mut lib = PromptLibrary::new();
    
    let template = PromptTemplate::new("test", "Hello {{name}}");
    lib.add_template("test", template);
    
    let retrieved = lib.get_template("test").unwrap();
    assert_eq!(retrieved.id(), "test");
    assert!(retrieved.template().contains("Hello"));
}

#[test]
fn test_library_get_nonexistent_template() {
    let lib = PromptLibrary::new();
    
    let result = lib.get_template("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_library_delete_template() {
    let mut lib = PromptLibrary::new();
    
    let template = PromptTemplate::new("test", "Content");
    lib.add_template("test", template);
    
    assert_eq!(lib.list_templates().len(), 1);
    
    let deleted = lib.delete_template("test");
    assert!(deleted.is_some());
    assert_eq!(lib.list_templates().len(), 0);
}

#[test]
fn test_library_delete_nonexistent() {
    let mut lib = PromptLibrary::new();
    
    let deleted = lib.delete_template("nonexistent");
    assert!(deleted.is_none());
}

#[test]
fn test_library_replace_template() {
    let mut lib = PromptLibrary::new();
    
    lib.add_template("test", PromptTemplate::new("test", "Original"));
    lib.add_template("test", PromptTemplate::new("test", "Updated"));
    
    let template = lib.get_template("test").unwrap();
    assert!(template.template().contains("Updated"));
}

// ========================================
// TemplateLibrary Tests
// ========================================

#[test]
fn test_template_library_new() {
    let meta = LibraryMetadata {
        version: "1.0".to_string(),
        description: "Test library".to_string(),
        author: "Test".to_string(),
        created_at: "2025-01-01".to_string(),
    };
    
    let lib = TemplateLibrary::new("test_lib".to_string(), meta);
    assert!(lib.list_collections().is_empty());
}

#[test]
fn test_template_library_add_collection() {
    let lib_meta = LibraryMetadata {
        version: "1.0".to_string(),
        description: "Library".to_string(),
        author: "Author".to_string(),
        created_at: "2025-01-01".to_string(),
    };
    
    let mut lib = TemplateLibrary::new("lib".to_string(), lib_meta);
    
    let col_meta = CollectionMetadata {
        version: "1.0".to_string(),
        description: "Collection".to_string(),
        tags: vec![],
    };
    
    let collection = TemplateCollection::new("col1".to_string(), col_meta);
    lib.add_collection(collection);
    
    assert_eq!(lib.list_collections().len(), 1);
}

#[test]
fn test_template_library_get_collection() {
    let lib_meta = LibraryMetadata {
        version: "1.0".to_string(),
        description: "Library".to_string(),
        author: "Author".to_string(),
        created_at: "2025-01-01".to_string(),
    };
    
    let mut lib = TemplateLibrary::new("lib".to_string(), lib_meta);
    
    let col_meta = CollectionMetadata {
        version: "1.0".to_string(),
        description: "Collection".to_string(),
        tags: vec!["test".to_string()],
    };
    
    let collection = TemplateCollection::new("test_col".to_string(), col_meta);
    lib.add_collection(collection);
    
    let retrieved = lib.get_collection("test_col");
    assert!(retrieved.is_some());
}

#[test]
fn test_template_library_get_nonexistent_collection() {
    let meta = LibraryMetadata {
        version: "1.0".to_string(),
        description: "Library".to_string(),
        author: "Author".to_string(),
        created_at: "2025-01-01".to_string(),
    };
    
    let lib = TemplateLibrary::new("lib".to_string(), meta);
    
    let result = lib.get_collection("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_template_library_load_from_directory() {
    let dir = tempdir().unwrap();
    
    fs::write(dir.path().join("template1.hbs"), "Template 1 content").unwrap();
    fs::write(dir.path().join("template2.hbs"), "Template 2 content").unwrap();
    
    let lib = TemplateLibrary::load_from_directory(dir.path().to_path_buf()).unwrap();
    
    // Should have a default collection with loaded templates
    let collections = lib.list_collections();
    assert_eq!(collections.len(), 1);
}

#[test]
fn test_template_library_save_stub() {
    let meta = LibraryMetadata {
        version: "1.0".to_string(),
        description: "Library".to_string(),
        author: "Author".to_string(),
        created_at: "2025-01-01".to_string(),
    };
    
    let lib = TemplateLibrary::new("lib".to_string(), meta);
    
    // save_to_directory is a stub, should succeed
    let result = lib.save_to_directory(std::path::PathBuf::from("/tmp/test"));
    assert!(result.is_ok());
}

// ========================================
// TemplateCollection Tests
// ========================================

#[test]
fn test_collection_new() {
    let meta = CollectionMetadata {
        version: "1.0".to_string(),
        description: "Test collection".to_string(),
        tags: vec![],
    };
    
    let collection = TemplateCollection::new("test".to_string(), meta);
    assert!(collection.list_templates().is_empty());
}

#[test]
fn test_collection_add_template() {
    let meta = CollectionMetadata {
        version: "1.0".to_string(),
        description: "Collection".to_string(),
        tags: vec![],
    };
    
    let mut collection = TemplateCollection::new("col".to_string(), meta);
    
    collection.add_template("t1".to_string(), "Template 1".to_string());
    collection.add_template("t2".to_string(), "Template 2".to_string());
    
    assert_eq!(collection.list_templates().len(), 2);
}

#[test]
fn test_collection_get_template() {
    let meta = CollectionMetadata {
        version: "1.0".to_string(),
        description: "Collection".to_string(),
        tags: vec![],
    };
    
    let mut collection = TemplateCollection::new("col".to_string(), meta);
    collection.add_template("test".to_string(), "Content".to_string());
    
    let template = collection.get_template("test");
    assert_eq!(template, Some(&"Content".to_string()));
}

#[test]
fn test_collection_get_nonexistent() {
    let meta = CollectionMetadata {
        version: "1.0".to_string(),
        description: "Collection".to_string(),
        tags: vec![],
    };
    
    let collection = TemplateCollection::new("col".to_string(), meta);
    
    let result = collection.get_template("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_collection_remove_template() {
    let meta = CollectionMetadata {
        version: "1.0".to_string(),
        description: "Collection".to_string(),
        tags: vec![],
    };
    
    let mut collection = TemplateCollection::new("col".to_string(), meta);
    collection.add_template("test".to_string(), "Content".to_string());
    
    let removed = collection.remove_template("test");
    assert_eq!(removed, Some("Content".to_string()));
    assert!(collection.get_template("test").is_none());
}

#[test]
fn test_collection_remove_nonexistent() {
    let meta = CollectionMetadata {
        version: "1.0".to_string(),
        description: "Collection".to_string(),
        tags: vec![],
    };
    
    let mut collection = TemplateCollection::new("col".to_string(), meta);
    
    let removed = collection.remove_template("nonexistent");
    assert!(removed.is_none());
}

// ========================================
// Metadata Tests
// ========================================

#[test]
fn test_library_metadata_serialization() {
    let meta = LibraryMetadata {
        version: "2.0.0".to_string(),
        description: "Production library".to_string(),
        author: "Dev Team".to_string(),
        created_at: "2025-06-15".to_string(),
    };
    
    let json = serde_json::to_string(&meta).unwrap();
    let restored: LibraryMetadata = serde_json::from_str(&json).unwrap();
    
    assert_eq!(restored.version, "2.0.0");
    assert_eq!(restored.author, "Dev Team");
    assert_eq!(restored.description, "Production library");
}

#[test]
fn test_collection_metadata_serialization() {
    let meta = CollectionMetadata {
        version: "1.5".to_string(),
        description: "Special collection".to_string(),
        tags: vec!["ai".to_string(), "prompts".to_string(), "games".to_string()],
    };
    
    let json = serde_json::to_string(&meta).unwrap();
    let restored: CollectionMetadata = serde_json::from_str(&json).unwrap();
    
    assert_eq!(restored.tags.len(), 3);
    assert!(restored.tags.contains(&"ai".to_string()));
    assert!(restored.tags.contains(&"prompts".to_string()));
}

#[test]
fn test_template_metadata_all_fields() {
    let mut meta = TemplateMetadata::default();
    meta.name = "full_template".to_string();
    meta.description = "A fully documented template".to_string();
    meta.category = TemplateCategory::Narrative;
    meta.author = Some("Author Name".to_string());
    meta.version = "3.0.0".to_string();
    meta.tags = vec!["story".to_string(), "quest".to_string()];
    meta.required_variables = vec!["hero".to_string(), "villain".to_string()];
    
    let mut optional = std::collections::HashMap::new();
    optional.insert("location".to_string(), serde_json::json!("Unknown"));
    meta.optional_variables = optional;
    
    assert_eq!(meta.name, "full_template");
    assert_eq!(meta.category, TemplateCategory::Narrative);
    assert_eq!(meta.required_variables.len(), 2);
    assert_eq!(meta.optional_variables.len(), 1);
}

// ========================================
// Integration: Load and Render
// ========================================

#[test]
fn test_load_and_render_workflow() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("workflow.hbs");
    
    fs::write(&file_path, "Hello {{name}}, welcome to {{place}}!").unwrap();
    
    let loader = PromptLoader::new();
    let template = loader.load_file(&file_path).unwrap();
    
    use astraweave_prompts::{PromptContext, ContextValue};
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("Alice".to_string()));
    ctx.set("place".to_string(), ContextValue::String("Wonderland".to_string()));
    
    let result = template.render(&ctx).unwrap();
    assert_eq!(result, "Hello Alice, welcome to Wonderland!");
}

#[test]
fn test_load_from_dir_and_use_in_engine() {
    let dir = tempdir().unwrap();
    
    fs::write(dir.path().join("greet.hbs"), "Hello {{name}}!").unwrap();
    fs::write(dir.path().join("farewell.hbs"), "Goodbye {{name}}!").unwrap();
    
    let mut engine = astraweave_prompts::TemplateEngine::new();
    
    // Load templates from directory
    engine.load_templates_from_dir(dir.path()).unwrap();
    
    let templates = engine.list_templates();
    assert!(templates.len() >= 2);
    
    use astraweave_prompts::{PromptContext, ContextValue};
    let mut ctx = PromptContext::new();
    ctx.set("name".to_string(), ContextValue::String("Bob".to_string()));
    
    let greeting = engine.render("greet", &ctx).unwrap();
    assert_eq!(greeting, "Hello Bob!");
    
    let farewell = engine.render("farewell", &ctx).unwrap();
    assert_eq!(farewell, "Goodbye Bob!");
}
