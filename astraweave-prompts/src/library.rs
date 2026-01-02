//! Template library management
//!
//! This module provides template library and collection management.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Template library
#[derive(Debug, Clone)]
pub struct TemplateLibrary {
    /// Library name
    #[allow(dead_code)]
    name: String,
    /// Template collections
    #[allow(dead_code)]
    collections: HashMap<String, TemplateCollection>,
    /// Library metadata
    #[allow(dead_code)]
    metadata: LibraryMetadata,
}

/// Backwards-compatible PromptLibrary used by consumers
#[derive(Debug, Clone)]
pub struct PromptLibrary {
    templates: HashMap<String, crate::template::PromptTemplate>,
}

impl Default for PromptLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptLibrary {
    /// Create a new prompt library
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Add a template to the library
    pub fn add_template(&mut self, name: &str, template: crate::template::PromptTemplate) {
        self.templates.insert(name.to_string(), template);
    }

    /// Get a template by name (cloned)
    pub fn get_template(&self, name: &str) -> anyhow::Result<crate::template::PromptTemplate> {
        if let Some(t) = self.templates.get(name) {
            Ok(t.clone())
        } else {
            anyhow::bail!("template not found: {}", name)
        }
    }

    /// Delete a template from the library
    pub fn delete_template(&mut self, name: &str) -> Option<crate::template::PromptTemplate> {
        self.templates.remove(name)
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }
}

/// Template collection
#[derive(Debug, Clone)]
pub struct TemplateCollection {
    /// Collection name
    name: String,
    /// Templates in this collection
    templates: HashMap<String, String>,
    /// Collection metadata
    #[allow(dead_code)]
    metadata: CollectionMetadata,
}

/// Library metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetadata {
    /// Library version
    pub version: String,
    /// Library description
    pub description: String,
    /// Library author
    pub author: String,
    /// Creation date
    pub created_at: String,
}

/// Collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    /// Collection version
    pub version: String,
    /// Collection description
    pub description: String,
    /// Collection tags
    pub tags: Vec<String>,
}

impl TemplateLibrary {
    /// Create a new template library
    pub fn new(name: String, metadata: LibraryMetadata) -> Self {
        Self {
            name,
            collections: HashMap::new(),
            metadata,
        }
    }

    /// Add a collection to the library
    pub fn add_collection(&mut self, collection: TemplateCollection) {
        self.collections.insert(collection.name.clone(), collection);
    }

    /// Get a collection by name
    pub fn get_collection(&self, name: &str) -> Option<&TemplateCollection> {
        self.collections.get(name)
    }

    /// List all collections
    pub fn list_collections(&self) -> Vec<&String> {
        self.collections.keys().collect()
    }

    /// Load library from directory
    pub fn load_from_directory(path: PathBuf) -> Result<Self> {
        let mut library = Self::new(
            "default".to_string(),
            LibraryMetadata {
                version: "1.0.0".to_string(),
                description: "Loaded from directory".to_string(),
                author: "AstraWeave".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        );

        let mut default_collection = TemplateCollection::new(
            "default".to_string(),
            CollectionMetadata {
                version: "1.0.0".to_string(),
                description: "Default collection".to_string(),
                tags: vec![],
            },
        );

        if path.exists() && path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "hbs" {
                            let name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let content = std::fs::read_to_string(&path)?;
                            default_collection.add_template(name, content);
                        }
                    }
                }
            }
        }

        library.add_collection(default_collection);
        Ok(library)
    }

    /// Save library to directory
    pub fn save_to_directory(&self, path: PathBuf) -> Result<()> {
        // Stub implementation - would actually write to filesystem
        let _ = path;
        Ok(())
    }
}

impl TemplateCollection {
    /// Create a new template collection
    pub fn new(name: String, metadata: CollectionMetadata) -> Self {
        Self {
            name,
            templates: HashMap::new(),
            metadata,
        }
    }

    /// Add a template to the collection
    pub fn add_template(&mut self, name: String, template: String) {
        self.templates.insert(name, template);
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&String> {
        self.templates.get(name)
    }

    /// List all templates in the collection
    pub fn list_templates(&self) -> Vec<&String> {
        self.templates.keys().collect()
    }

    /// Remove a template from the collection
    pub fn remove_template(&mut self, name: &str) -> Option<String> {
        self.templates.remove(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_library_new() {
        let lib = PromptLibrary::new();
        assert!(lib.list_templates().is_empty());
    }

    #[test]
    fn test_prompt_library_add_and_get() {
        let mut lib = PromptLibrary::new();
        let template = crate::template::PromptTemplate::new("test".to_string(), "Hello {{name}}".to_string());
        
        lib.add_template("greeting", template);
        
        let result = lib.get_template("greeting");
        assert!(result.is_ok());
    }

    #[test]
    fn test_prompt_library_get_nonexistent() {
        let lib = PromptLibrary::new();
        let result = lib.get_template("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_prompt_library_delete_template() {
        let mut lib = PromptLibrary::new();
        let template = crate::template::PromptTemplate::new("test".to_string(), "content".to_string());
        
        lib.add_template("test", template);
        assert!(!lib.list_templates().is_empty());
        
        let deleted = lib.delete_template("test");
        assert!(deleted.is_some());
        assert!(lib.list_templates().is_empty());
    }

    #[test]
    fn test_prompt_library_delete_nonexistent() {
        let mut lib = PromptLibrary::new();
        let deleted = lib.delete_template("nonexistent");
        assert!(deleted.is_none());
    }

    #[test]
    fn test_prompt_library_list_templates() {
        let mut lib = PromptLibrary::new();
        
        lib.add_template("a", crate::template::PromptTemplate::new("a".to_string(), "A".to_string()));
        lib.add_template("b", crate::template::PromptTemplate::new("b".to_string(), "B".to_string()));
        lib.add_template("c", crate::template::PromptTemplate::new("c".to_string(), "C".to_string()));
        
        let list = lib.list_templates();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&"a".to_string()));
        assert!(list.contains(&"b".to_string()));
        assert!(list.contains(&"c".to_string()));
    }

    #[test]
    fn test_template_collection_new() {
        let meta = CollectionMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            tags: vec!["test".to_string()],
        };
        let collection = TemplateCollection::new("test_collection".to_string(), meta);
        assert!(collection.list_templates().is_empty());
    }

    #[test]
    fn test_template_collection_add_and_get() {
        let meta = CollectionMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            tags: vec![],
        };
        let mut collection = TemplateCollection::new("test".to_string(), meta);
        
        collection.add_template("greeting".to_string(), "Hello {{name}}".to_string());
        
        let result = collection.get_template("greeting");
        assert_eq!(result, Some(&"Hello {{name}}".to_string()));
    }

    #[test]
    fn test_template_collection_get_nonexistent() {
        let meta = CollectionMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            tags: vec![],
        };
        let collection = TemplateCollection::new("test".to_string(), meta);
        
        let result = collection.get_template("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_template_collection_remove() {
        let meta = CollectionMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            tags: vec![],
        };
        let mut collection = TemplateCollection::new("test".to_string(), meta);
        
        collection.add_template("greeting".to_string(), "Hello".to_string());
        
        let removed = collection.remove_template("greeting");
        assert_eq!(removed, Some("Hello".to_string()));
        assert!(collection.get_template("greeting").is_none());
    }

    #[test]
    fn test_template_collection_list() {
        let meta = CollectionMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            tags: vec![],
        };
        let mut collection = TemplateCollection::new("test".to_string(), meta);
        
        collection.add_template("a".to_string(), "A".to_string());
        collection.add_template("b".to_string(), "B".to_string());
        
        let list = collection.list_templates();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_template_library_new() {
        let meta = LibraryMetadata {
            version: "1.0".to_string(),
            description: "Test library".to_string(),
            author: "Test".to_string(),
            created_at: "2025-01-01".to_string(),
        };
        let library = TemplateLibrary::new("test_lib".to_string(), meta);
        assert!(library.list_collections().is_empty());
    }

    #[test]
    fn test_template_library_add_and_get_collection() {
        let lib_meta = LibraryMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            created_at: "2025-01-01".to_string(),
        };
        let mut library = TemplateLibrary::new("lib".to_string(), lib_meta);
        
        let col_meta = CollectionMetadata {
            version: "1.0".to_string(),
            description: "Coll".to_string(),
            tags: vec![],
        };
        let collection = TemplateCollection::new("my_collection".to_string(), col_meta);
        
        library.add_collection(collection);
        
        let result = library.get_collection("my_collection");
        assert!(result.is_some());
    }

    #[test]
    fn test_template_library_get_nonexistent_collection() {
        let meta = LibraryMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            created_at: "2025-01-01".to_string(),
        };
        let library = TemplateLibrary::new("lib".to_string(), meta);
        
        let result = library.get_collection("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_template_library_list_collections() {
        let lib_meta = LibraryMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            created_at: "2025-01-01".to_string(),
        };
        let mut library = TemplateLibrary::new("lib".to_string(), lib_meta);
        
        let col_meta1 = CollectionMetadata {
            version: "1.0".to_string(),
            description: "A".to_string(),
            tags: vec![],
        };
        let col_meta2 = CollectionMetadata {
            version: "1.0".to_string(),
            description: "B".to_string(),
            tags: vec![],
        };
        
        library.add_collection(TemplateCollection::new("col1".to_string(), col_meta1));
        library.add_collection(TemplateCollection::new("col2".to_string(), col_meta2));
        
        let list = library.list_collections();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_template_library_save_stub() {
        let meta = LibraryMetadata {
            version: "1.0".to_string(),
            description: "Test".to_string(),
            author: "Test".to_string(),
            created_at: "2025-01-01".to_string(),
        };
        let library = TemplateLibrary::new("lib".to_string(), meta);
        
        // Stub should succeed
        let result = library.save_to_directory(std::path::PathBuf::from("/tmp/test"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_library_metadata_serialization() {
        let meta = LibraryMetadata {
            version: "2.0.0".to_string(),
            description: "My library".to_string(),
            author: "Developer".to_string(),
            created_at: "2025-06-15".to_string(),
        };
        
        let json = serde_json::to_string(&meta).unwrap();
        let restored: LibraryMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.version, "2.0.0");
        assert_eq!(restored.author, "Developer");
    }

    #[test]
    fn test_collection_metadata_serialization() {
        let meta = CollectionMetadata {
            version: "1.0".to_string(),
            description: "Test collection".to_string(),
            tags: vec!["ai".to_string(), "prompts".to_string()],
        };
        
        let json = serde_json::to_string(&meta).unwrap();
        let restored: CollectionMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(restored.tags.len(), 2);
        assert!(restored.tags.contains(&"ai".to_string()));
    }

    #[test]
    fn test_template_library_load_from_nonexistent_directory() {
        let path = std::path::PathBuf::from("/nonexistent/path/xyz123");
        let result = TemplateLibrary::load_from_directory(path);
        // Should succeed with empty default collection
        assert!(result.is_ok());
    }
}
