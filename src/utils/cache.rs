use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A simple in-memory cache with TTL support
pub struct Cache<K, V> {
    data: HashMap<K, CacheEntry<V>>,
    default_ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K, V> Cache<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    /// Creates a new cache with the specified default TTL
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            data: HashMap::new(),
            default_ttl,
        }
    }

    /// Inserts a value into the cache with the default TTL
    pub fn insert(&mut self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }

    /// Inserts a value into the cache with a custom TTL
    pub fn insert_with_ttl(&mut self, key: K, value: V, ttl: Duration) {
        let expires_at = Instant::now() + ttl;
        self.data.insert(key, CacheEntry { value, expires_at });
    }

    /// Gets a value from the cache if it exists and hasn't expired
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(entry) = self.data.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            } else {
                // Remove expired entry
                self.data.remove(key);
            }
        }
        None
    }

    /// Removes a value from the cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key).map(|entry| entry.value)
    }

    /// Clears all expired entries from the cache
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, entry| entry.expires_at > now);
    }

    /// Clears all entries from the cache
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns the number of entries in the cache (including expired ones)
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = Cache::new(Duration::from_secs(1));

        // Insert and get
        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Non-existent key
        assert_eq!(cache.get(&"nonexistent".to_string()), None);

        // Remove
        assert_eq!(
            cache.remove(&"key1".to_string()),
            Some("value1".to_string())
        );
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_ttl() {
        let mut cache = Cache::new(Duration::from_millis(100));

        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Wait for expiration
        thread::sleep(Duration::from_millis(150));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_custom_ttl() {
        let mut cache = Cache::new(Duration::from_secs(10));

        cache.insert_with_ttl(
            "key1".to_string(),
            "value1".to_string(),
            Duration::from_millis(50),
        );
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        thread::sleep(Duration::from_millis(100));
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_cache_cleanup() {
        let mut cache = Cache::new(Duration::from_millis(50));

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        assert_eq!(cache.len(), 2);

        thread::sleep(Duration::from_millis(100));
        cache.cleanup_expired();
        assert_eq!(cache.len(), 0);
    }
}
