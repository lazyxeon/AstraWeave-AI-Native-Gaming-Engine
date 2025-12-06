//! Template loading and management
//!
//! This module provides functionality for loading templates from files
//! and managing template collections.

use crate::{PromptTemplate, TemplateMetadata};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Loader for prompt templates
pub struct PromptLoader {
    /// Supported file extensions
    extensions: Vec<String>,
}

impl Default for PromptLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptLoader {
    /// Create a new prompt loader
    pub fn new() -> Self {
        Self {
            extensions: vec!["hbs".to_string(), "prompt".to_string()],
        }
    }

    /// Set supported extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Load templates from a directory recursively
    pub fn load_from_dir<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<PromptTemplate>> {
        let mut templates = Vec::new();
        let dir_path = dir.as_ref();

        if !dir_path.exists() {
            return Ok(templates);
        }

        for entry in WalkDir::new(dir_path).follow_links(true) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if self.extensions.iter().any(|e| e == ext.to_string_lossy().as_ref()) {
                        match self.load_file(entry.path()) {
                            Ok(template) => templates.push(template),
                            Err(e) => {
                                eprintln!("Failed to load template from {:?}: {}", entry.path(), e);
                            }
                        }
                    }
                }
            }
        }
        Ok(templates)
    }

    /// Load a single template file
    pub fn load_file<P: AsRef<Path>>(&self, path: P) -> Result<PromptTemplate> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read template file: {:?}", path.as_ref()))?;
        
        let (metadata, template_content) = self.parse_frontmatter(&content)?;
        
        // Use filename as ID if not specified in metadata
        let id = if let Some(ref meta) = metadata {
            meta.name.clone()
        } else {
            path.as_ref()
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        };
        
        let mut template = PromptTemplate::new(id, template_content);
        template.metadata = metadata;
        
        Ok(template)
    }

    /// Parse frontmatter from template content
    /// Supports TOML frontmatter delimited by +++
    fn parse_frontmatter(&self, content: &str) -> Result<(Option<TemplateMetadata>, String)> {
        if content.starts_with("+++") {
            if let Some(end) = content[3..].find("+++") {
                let frontmatter = &content[3..end+3];
                let body = &content[end+6..];
                
                let metadata: TemplateMetadata = toml::from_str(frontmatter)
                    .context("Failed to parse TOML frontmatter")?;
                    
                return Ok((Some(metadata), body.trim().to_string()));
            }
        }
        
        Ok((None, content.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_prompt_loader_new() {
        let loader = PromptLoader::new();
        assert!(loader.extensions.contains(&"hbs".to_string()));
        assert!(loader.extensions.contains(&"prompt".to_string()));
    }

    #[test]
    fn test_prompt_loader_default() {
        let loader: PromptLoader = Default::default();
        assert!(loader.extensions.contains(&"hbs".to_string()));
    }

    #[test]
    fn test_prompt_loader_with_extensions() {
        let loader = PromptLoader::new()
            .with_extensions(vec!["txt".to_string(), "tpl".to_string()]);
        
        assert!(loader.extensions.contains(&"txt".to_string()));
        assert!(loader.extensions.contains(&"tpl".to_string()));
        assert!(!loader.extensions.contains(&"hbs".to_string()));
    }

    #[test]
    fn test_load_from_nonexistent_dir() {
        let loader = PromptLoader::new();
        let result = loader.load_from_dir("/nonexistent/path/xyz123");
        
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_load_from_empty_dir() {
        let dir = tempdir().unwrap();
        let loader = PromptLoader::new();
        
        let result = loader.load_from_dir(dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_load_file_simple() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.hbs");
        
        let mut file = std::fs::File::create(&file_path).unwrap();
        writeln!(file, "Hello {{{{name}}}}!").unwrap();
        
        let loader = PromptLoader::new();
        let result = loader.load_file(&file_path);
        
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.id, "test");
        assert!(template.template.contains("Hello"));
    }

    #[test]
    fn test_load_file_with_frontmatter() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("greeting.hbs");
        
        let content = r#"+++
name = "greeting"
description = "A greeting template"
version = "1.0"
+++
Hello {{name}}!"#;
        
        std::fs::write(&file_path, content).unwrap();
        
        let loader = PromptLoader::new();
        let result = loader.load_file(&file_path);
        
        assert!(result.is_ok());
        let template = result.unwrap();
        assert_eq!(template.id, "greeting");
        assert!(template.metadata.is_some());
    }

    #[test]
    fn test_load_from_dir_with_files() {
        let dir = tempdir().unwrap();
        
        // Create some template files
        std::fs::write(dir.path().join("template1.hbs"), "Template 1 content").unwrap();
        std::fs::write(dir.path().join("template2.hbs"), "Template 2 content").unwrap();
        std::fs::write(dir.path().join("ignored.txt"), "This should be ignored").unwrap();
        
        let loader = PromptLoader::new();
        let result = loader.load_from_dir(dir.path());
        
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 2); // Only .hbs files
    }

    #[test]
    fn test_load_from_dir_recursive() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        
        std::fs::write(dir.path().join("root.hbs"), "Root template").unwrap();
        std::fs::write(subdir.join("nested.hbs"), "Nested template").unwrap();
        
        let loader = PromptLoader::new();
        let result = loader.load_from_dir(dir.path());
        
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 2); // Both root and nested
    }

    #[test]
    fn test_load_file_nonexistent() {
        let loader = PromptLoader::new();
        let result = loader.load_file("/nonexistent/file.hbs");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_no_frontmatter() {
        let loader = PromptLoader::new();
        let content = "Just plain content";
        
        let result = loader.parse_frontmatter(content);
        assert!(result.is_ok());
        let (meta, body) = result.unwrap();
        assert!(meta.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_parse_frontmatter_with_frontmatter() {
        let loader = PromptLoader::new();
        let content = r#"+++
name = "test"
description = "Test template"
version = "1.0"
+++
Body content here"#;
        
        let result = loader.parse_frontmatter(content);
        assert!(result.is_ok());
        let (meta, body) = result.unwrap();
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(meta.name, "test");
        assert_eq!(body, "Body content here");
    }

    #[test]
    fn test_parse_frontmatter_incomplete() {
        let loader = PromptLoader::new();
        // Missing closing +++
        let content = r#"+++
name = "test"
Body without closing delimiter"#;
        
        let result = loader.parse_frontmatter(content);
        assert!(result.is_ok());
        let (meta, body) = result.unwrap();
        // Should treat as no frontmatter
        assert!(meta.is_none());
        assert_eq!(body, content);
    }

    #[test]
    fn test_load_prompt_extension() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("custom.prompt");
        
        std::fs::write(&file_path, "Custom prompt content").unwrap();
        
        let loader = PromptLoader::new();
        let result = loader.load_from_dir(dir.path());
        
        assert!(result.is_ok());
        let templates = result.unwrap();
        assert_eq!(templates.len(), 1);
    }

    #[test]
    fn test_custom_extensions() {
        let dir = tempdir().unwrap();
        
        std::fs::write(dir.path().join("file.txt"), "TXT content").unwrap();
        std::fs::write(dir.path().join("file.hbs"), "HBS content").unwrap();
        
        let loader = PromptLoader::new()
            .with_extensions(vec!["txt".to_string()]);
        
        let result = loader.load_from_dir(dir.path());
        assert!(result.is_ok());
        let templates = result.unwrap();
        
        // Only .txt files
        assert_eq!(templates.len(), 1);
        assert!(templates[0].template.contains("TXT"));
    }
}
