use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// A simple thread-safe string interner
pub struct StringInterner {
    map: HashMap<String, u32>,
    vec: Vec<String>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            vec: Vec::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> u32 {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = self.vec.len() as u32;
        self.vec.push(s.to_string());
        self.map.insert(s.to_string(), id);
        id
    }

    pub fn resolve(&self, id: u32) -> Option<&str> {
        self.vec.get(id as usize).map(|s| s.as_str())
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

pub static GLOBAL_INTERNER: LazyLock<Mutex<StringInterner>> =
    LazyLock::new(|| Mutex::new(StringInterner::new()));

/// Intern a string, returning a unique u32 ID
pub fn intern(s: &str) -> u32 {
    GLOBAL_INTERNER.lock().unwrap().intern(s)
}

/// Resolve an ID back to a string
pub fn resolve(id: u32) -> String {
    GLOBAL_INTERNER
        .lock()
        .unwrap()
        .resolve(id)
        .unwrap_or("<unknown>")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interner_new() {
        let interner = StringInterner::new();
        assert!(interner.map.is_empty());
        assert!(interner.vec.is_empty());
    }

    #[test]
    fn test_string_interner_default() {
        let interner = StringInterner::default();
        assert!(interner.map.is_empty());
        assert!(interner.vec.is_empty());
    }

    #[test]
    fn test_intern_and_resolve() {
        let mut interner = StringInterner::new();
        let id1 = interner.intern("hello");
        let id2 = interner.intern("world");
        let id3 = interner.intern("hello"); // duplicate

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 0); // Should return same ID as first "hello"

        assert_eq!(interner.resolve(id1), Some("hello"));
        assert_eq!(interner.resolve(id2), Some("world"));
        assert_eq!(interner.resolve(99), None); // Invalid ID
    }

    #[test]
    fn test_global_interner() {
        let id1 = intern("global_test");
        let id2 = intern("global_test");
        assert_eq!(id1, id2);

        let resolved = resolve(id1);
        assert_eq!(resolved, "global_test");
    }

    #[test]
    fn test_resolve_unknown() {
        let resolved = resolve(u32::MAX);
        assert_eq!(resolved, "<unknown>");
    }
}
