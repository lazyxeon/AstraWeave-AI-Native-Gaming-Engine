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
