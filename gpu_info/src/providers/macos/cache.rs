//! Smart caching system for macOS GPU information
//!
//! Caches static GPU information to minimize expensive system calls.

use crate::gpu_info::GpuInfo;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Static GPU information that doesn't change frequently
#[derive(Debug, Clone)]
pub struct GpuStaticInfo {
    /// Complete GPU information
    pub info: GpuInfo,
    /// Timestamp when this information was cached
    pub cached_at: Instant,
}

/// Smart cache for GPU static information
///
/// Thread-safe cache with TTL (time-to-live) support.
/// Uses RwLock for concurrent read access with exclusive writes.
///
/// # Examples
///
/// ```
/// use gpu_info::providers::macos::cache::GpuCache;
/// use std::time::Duration;
///
/// let cache = GpuCache::new(Duration::from_secs(60));
/// ```
pub struct GpuCache {
    /// Cached static information keyed by GPU identifier
    data: RwLock<HashMap<String, GpuStaticInfo>>,
    /// Time-to-live for cached entries
    ttl: Duration,
}

impl GpuCache {
    /// Creates a new cache with the specified TTL
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::cache::GpuCache;
    /// use std::time::Duration;
    ///
    /// let cache = GpuCache::new(Duration::from_secs(120));
    /// ```
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    /// Retrieves a cached GPU info if it exists and is still valid
    ///
    /// Returns `None` if the entry doesn't exist or has expired.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::cache::GpuCache;
    /// use std::time::Duration;
    ///
    /// let cache = GpuCache::new(Duration::from_secs(60));
    /// let result = cache.get("gpu-0");
    /// assert!(result.is_none()); // Cache is empty
    /// ```
    pub fn get(&self, key: &str) -> Option<GpuInfo> {
        let data = self.data.read().ok()?;
        let entry = data.get(key)?;

        // Check if entry is still valid
        if entry.cached_at.elapsed() < self.ttl {
            Some(entry.info.clone())
        } else {
            // Entry expired, will be removed on next cleanup
            None
        }
    }

    /// Stores GPU information in the cache
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gpu_info::providers::macos::cache::GpuCache;
    /// use gpu_info::gpu_info::GpuInfo;
    /// use std::time::Duration;
    ///
    /// let cache = GpuCache::new(Duration::from_secs(60));
    /// let gpu_info = GpuInfo::default();
    /// cache.insert("gpu-0".to_string(), gpu_info);
    /// ```
    pub fn insert(&self, key: String, info: GpuInfo) {
        if let Ok(mut data) = self.data.write() {
            data.insert(
                key,
                GpuStaticInfo {
                    info,
                    cached_at: Instant::now(),
                },
            );
        }
    }

    /// Retrieves cached GPU info or computes it using the provided function
    ///
    /// This is the primary cache interface - it handles the "get or compute" pattern
    /// atomically. If the cached value is missing or expired, it calls the compute
    /// function and caches the result.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gpu_info::providers::macos::cache::GpuCache;
    /// use gpu_info::gpu_info::GpuInfo;
    /// use std::time::Duration;
    ///
    /// let cache = GpuCache::new(Duration::from_secs(60));
    /// let info = cache.get_or_compute("gpu-0", || {
    ///     // Expensive GPU detection here
    ///     GpuInfo::default()
    /// });
    /// ```
    pub fn get_or_compute<F>(&self, key: &str, compute: F) -> GpuInfo
    where
        F: FnOnce() -> GpuInfo,
    {
        // Try to get from cache first (fast path)
        if let Some(info) = self.get(key) {
            return info;
        }

        // Cache miss - compute the value
        let info = compute();

        // Store in cache for next time
        self.insert(key.to_string(), info.clone());

        info
    }

    /// Clears all cached entries
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::cache::GpuCache;
    /// use std::time::Duration;
    ///
    /// let cache = GpuCache::new(Duration::from_secs(60));
    /// cache.clear();
    /// ```
    pub fn clear(&self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
    }

    /// Removes expired entries from the cache
    ///
    /// This is automatically called periodically, but can be manually invoked
    /// to free up memory.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::cache::GpuCache;
    /// use std::time::Duration;
    ///
    /// let cache = GpuCache::new(Duration::from_secs(60));
    /// cache.cleanup_expired();
    /// ```
    pub fn cleanup_expired(&self) {
        if let Ok(mut data) = self.data.write() {
            data.retain(|_, entry| entry.cached_at.elapsed() < self.ttl);
        }
    }

    /// Returns the number of cached entries (including expired ones)
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::cache::GpuCache;
    /// use std::time::Duration;
    ///
    /// let cache = GpuCache::new(Duration::from_secs(60));
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.data.read().map(|d| d.len()).unwrap_or(0)
    }

    /// Returns true if the cache is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::cache::GpuCache;
    /// use std::time::Duration;
    ///
    /// let cache = GpuCache::new(Duration::from_secs(60));
    /// assert!(cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the TTL duration
    #[inline]
    pub fn ttl(&self) -> Duration {
        self.ttl
    }
}

impl Default for GpuCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache = GpuCache::new(Duration::from_secs(30));
        assert_eq!(cache.ttl(), Duration::from_secs(30));
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache = GpuCache::new(Duration::from_secs(60));
        let gpu = GpuInfo::default();

        cache.insert("test-gpu".to_string(), gpu.clone());
        let retrieved = cache.get("test-gpu");

        assert!(retrieved.is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_miss() {
        let cache = GpuCache::new(Duration::from_secs(60));
        let result = cache.get("nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_get_or_compute() {
        let cache = GpuCache::new(Duration::from_secs(60));
        let mut compute_count = 0;

        // First call should compute
        let _info1 = cache.get_or_compute("test-gpu", || {
            compute_count += 1;
            GpuInfo::default()
        });
        assert_eq!(compute_count, 1);

        // Second call should use cache
        let _info2 = cache.get_or_compute("test-gpu", || {
            compute_count += 1;
            GpuInfo::default()
        });
        assert_eq!(compute_count, 1); // Should not increment
    }

    #[test]
    fn test_cache_clear() {
        let cache = GpuCache::new(Duration::from_secs(60));
        cache.insert("gpu-1".to_string(), GpuInfo::default());
        cache.insert("gpu-2".to_string(), GpuInfo::default());

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_ttl_expiration() {
        let cache = GpuCache::new(Duration::from_millis(100));
        cache.insert("test-gpu".to_string(), GpuInfo::default());

        // Should be available immediately
        assert!(cache.get("test-gpu").is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Should be expired now
        assert!(cache.get("test-gpu").is_none());
    }

    #[test]
    fn test_cleanup_expired() {
        let cache = GpuCache::new(Duration::from_millis(100));
        cache.insert("gpu-1".to_string(), GpuInfo::default());
        cache.insert("gpu-2".to_string(), GpuInfo::default());

        assert_eq!(cache.len(), 2);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Cleanup should remove expired entries
        cache.cleanup_expired();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_default_cache() {
        let cache = GpuCache::default();
        assert_eq!(cache.ttl(), Duration::from_secs(60));
    }
}
