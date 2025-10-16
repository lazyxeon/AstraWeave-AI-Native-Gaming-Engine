// Simple LRU cache implementation using std collections
// Thread-safe via interior mutability (Mutex)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::hash::Hash;

/// A simple LRU (Least Recently Used) cache
/// 
/// This implementation uses a HashMap and tracks access order.
/// When capacity is exceeded, the least recently used item is evicted.
pub struct LruCache<K, V> {
    inner: Arc<Mutex<LruCacheInner<K, V>>>,
}

struct LruCacheInner<K, V> {
    map: HashMap<K, CacheEntry<V>>,
    capacity: usize,
    access_counter: u64,
}

struct CacheEntry<V> {
    value: V,
    last_access: u64,
}

impl<K: Hash + Eq + Clone, V: Clone> LruCache<K, V> {
    /// Create a new LRU cache with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCacheInner {
                map: HashMap::with_capacity(capacity),
                capacity,
                access_counter: 0,
            })),
        }
    }

    /// Get a value from the cache, updating its access time
    pub fn get(&self, key: &K) -> Option<V> {
        let mut inner = self.inner.lock().unwrap();
        
        // Update access counter first
        inner.access_counter += 1;
        let new_access = inner.access_counter;
        
        // Then try to get and update entry
        if let Some(entry) = inner.map.get_mut(key) {
            entry.last_access = new_access;
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Put a value into the cache
    /// 
    /// Returns true if an item was evicted, false otherwise
    pub fn put(&self, key: K, value: V) -> bool {
        let mut inner = self.inner.lock().unwrap();
        
        // Update access counter first
        inner.access_counter += 1;
        let new_access = inner.access_counter;

        // If key already exists, update it
        if let Some(entry) = inner.map.get_mut(&key) {
            entry.value = value;
            entry.last_access = new_access;
            return false;
        }

        // If at capacity, evict LRU item
        let mut evicted = false;
        if inner.map.len() >= inner.capacity {
            // Find key with minimum last_access
            if let Some(lru_key) = inner.map.iter()
                .min_by_key(|(_, entry)| entry.last_access)
                .map(|(k, _)| k.clone())
            {
                inner.map.remove(&lru_key);
                evicted = true;
            }
        }

        // Insert new entry
        inner.map.insert(key, CacheEntry {
            value,
            last_access: new_access,
        });

        evicted
    }

    /// Get the current number of items in the cache
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().map.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clear all items from the cache
    #[allow(dead_code)]
    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.map.clear();
        inner.access_counter = 0;
    }

    /// Phase 7: Get all keys in the cache (for similarity search)
    pub fn keys(&self) -> Vec<K> {
        let inner = self.inner.lock().unwrap();
        inner.map.keys().cloned().collect()
    }
}

// Implement Clone for LruCache (clones the Arc, not the data)
impl<K, V> Clone for LruCache<K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_basic_operations() {
        let cache = LruCache::new(3);

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.len(), 3);
        assert!(!cache.is_empty());

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
        assert_eq!(cache.get(&"d"), None);
    }

    #[test]
    fn test_lru_eviction_order() {
        let cache = LruCache::new(2);

        // Fill cache
        cache.put("a", 1);
        cache.put("b", 2);

        // Insert third item - should evict "a" (least recently used)
        let evicted = cache.put("c", 3);
        assert!(evicted);
        assert_eq!(cache.len(), 2);

        // "a" should be gone
        assert_eq!(cache.get(&"a"), None);
        // "b" and "c" should remain
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_lru_access_updates_order() {
        let cache = LruCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);

        // Access "a" to make it more recently used
        cache.get(&"a");

        // Insert third item - should evict "b" now (not "a")
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(1)); // "a" should still be present
        assert_eq!(cache.get(&"b"), None);    // "b" should be evicted
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_lru_update_existing() {
        let cache = LruCache::new(2);

        cache.put("a", 1);
        assert_eq!(cache.get(&"a"), Some(1));

        // Update existing key - should not evict
        let evicted = cache.put("a", 10);
        assert!(!evicted);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"a"), Some(10));
    }

    #[test]
    fn test_lru_clear() {
        let cache = LruCache::new(5);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        assert_eq!(cache.len(), 3);

        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.get(&"a"), None);
    }

    #[test]
    fn test_lru_clone_shares_data() {
        let cache1 = LruCache::new(3);
        cache1.put("a", 1);

        let cache2 = cache1.clone();

        // Both should see the same data
        assert_eq!(cache1.get(&"a"), Some(1));
        assert_eq!(cache2.get(&"a"), Some(1));

        // Modification in one affects the other
        cache2.put("b", 2);
        assert_eq!(cache1.get(&"b"), Some(2));
    }
}
