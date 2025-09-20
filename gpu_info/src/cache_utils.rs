//! Common caching utilities for GPU information
//!
//! This module provides unified caching mechanisms to eliminate duplication
//! in GPU information caching across different components.
use crate::gpu_info::GpuInfo;
use log::debug;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
/// Generic cache entry with TTL support
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// Cached value
    pub value: T,
    /// Timestamp when the entry was created
    pub timestamp: Instant,
}
impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }
    /// Check if the entry is still valid based on TTL
    pub fn is_valid(&self, ttl: Duration) -> bool {
        self.timestamp.elapsed() < ttl
    }
    /// Get the age of the entry
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}
/// Single-item cache for GPU information
///
/// This cache eliminates duplication by providing a unified caching mechanism
/// for single GPU information items.
///
/// # Examples
/// ```
/// use gpu_info::cache_utils::{GpuInfoCache, CacheEntry};
/// use gpu_info::gpu_info::GpuInfo;
/// use std::time::Duration;
///
/// let cache = GpuInfoCache::new(Duration::from_secs(1));
/// let gpu_info = GpuInfo::unknown();
/// cache.set(gpu_info.clone());
///
/// assert_eq!(cache.get(), Some(gpu_info));
/// ```
pub struct GpuInfoCache {
    /// Cached GPU information with timestamp
    info: RwLock<Option<CacheEntry<GpuInfo>>>,
    /// Time-to-live for cached entries
    ttl: Duration,
}
impl GpuInfoCache {
    /// Create a new GPU info cache with the specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            info: RwLock::new(None),
            ttl,
        }
    }
    /// Get cached GPU information if it's still valid
    pub fn get(&self) -> Option<GpuInfo> {
        let guard = self.info.read().ok()?;
        let entry = guard.as_ref()?;
        if entry.is_valid(self.ttl) {
            debug!("Returning cached GPU info (age: {:?})", entry.age());
            Some(entry.value.clone())
        } else {
            debug!("Cached GPU info expired (age: {:?})", entry.age());
            None
        }
    }
    /// Set GPU information in the cache
    pub fn set(&self, info: GpuInfo) {
        if let Ok(mut guard) = self.info.write() {
            *guard = Some(CacheEntry::new(info));
            debug!("Updated GPU info cache");
        }
    }
    /// Clear the cache
    pub fn clear(&self) {
        if let Ok(mut guard) = self.info.write() {
            *guard = None;
            debug!("Cleared GPU info cache");
        }
    }
    /// Check if there's a cached entry (regardless of validity)
    pub fn has_entry(&self) -> bool {
        if let Ok(guard) = self.info.read() {
            guard.is_some()
        } else {
            false
        }
    }
    /// Get the age of the cached entry if it exists
    pub fn age(&self) -> Option<Duration> {
        if let Ok(guard) = self.info.read() {
            guard.as_ref().map(|entry| entry.age())
        } else {
            None
        }
    }
}
impl Default for GpuInfoCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
/// Multi-item cache for multiple GPU information entries
///
/// This cache eliminates duplication by providing a unified caching mechanism
/// for multiple GPU information items indexed by key.
///
/// # Examples
/// ```
/// use gpu_info::cache_utils::{MultiGpuInfoCache, CacheEntry};
/// use gpu_info::gpu_info::GpuInfo;
/// use std::time::Duration;
///
/// let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
/// let gpu_info = GpuInfo::unknown();
/// cache.set(0, gpu_info.clone());
///
/// assert_eq!(cache.get(&0), Some(gpu_info));
/// ```
#[derive(Debug, Clone)]
pub struct MultiGpuInfoCache {
    /// Cached GPU information entries indexed by key
    entries: Arc<Mutex<HashMap<usize, CacheEntry<GpuInfo>>>>,
    /// Time-to-live for cached entries
    ttl: Duration,
}
impl MultiGpuInfoCache {
    /// Create a new multi-GPU info cache with the specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }
    /// Get cached GPU information by key if it's still valid
    pub fn get(&self, key: &usize) -> Option<GpuInfo> {
        let guard = self.entries.lock().ok()?;
        let entry = guard.get(key)?;
        if entry.is_valid(self.ttl) {
            debug!(
                "Returning cached GPU info for key {} (age: {:?})",
                key,
                entry.age()
            );
            Some(entry.value.clone())
        } else {
            debug!(
                "Cached GPU info for key {} expired (age: {:?})",
                key,
                entry.age()
            );
            None
        }
    }
    /// Set GPU information in the cache by key
    pub fn set(&self, key: usize, info: GpuInfo) {
        if let Ok(mut guard) = self.entries.lock() {
            guard.insert(key, CacheEntry::new(info));
            debug!("Updated GPU info cache for key {}", key);
        }
    }
    /// Clear the cache entry for a specific key
    pub fn clear_key(&self, key: &usize) {
        if let Ok(mut guard) = self.entries.lock() {
            guard.remove(key);
            debug!("Cleared GPU info cache for key {}", key);
        }
    }
    /// Clear all cache entries
    pub fn clear_all(&self) {
        if let Ok(mut guard) = self.entries.lock() {
            guard.clear();
            debug!("Cleared all GPU info cache entries");
        }
    }
    /// Check if there's a cached entry for a specific key (regardless of validity)
    pub fn has_entry(&self, key: &usize) -> bool {
        if let Ok(guard) = self.entries.lock() {
            guard.contains_key(key)
        } else {
            false
        }
    }
    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        if let Ok(guard) = self.entries.lock() {
            guard.len()
        } else {
            0
        }
    }
    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl Default for MultiGpuInfoCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_info::GpuInfo;
    use std::thread;
    use std::time::Duration;
    #[test]
    fn test_gpu_info_cache_creation() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        assert!(!cache.has_entry());
        assert_eq!(cache.get(), None);
    }
    #[test]
    fn test_gpu_info_cache_set_and_get() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = GpuInfo::unknown();
        cache.set(gpu_info.clone());
        assert!(cache.has_entry());
        assert_eq!(cache.get(), Some(gpu_info));
    }
    #[test]
    fn test_gpu_info_cache_expiration() {
        let cache = GpuInfoCache::new(Duration::from_millis(10));
        let gpu_info = GpuInfo::unknown();
        cache.set(gpu_info.clone());
        assert_eq!(cache.get(), Some(gpu_info));
        thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get(), None);
    }
    #[test]
    fn test_multi_gpu_info_cache_creation() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        assert!(cache.is_empty());
        assert_eq!(cache.get(&0), None);
    }
    #[test]
    fn test_multi_gpu_info_cache_set_and_get() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = GpuInfo::unknown();
        cache.set(0, gpu_info.clone());
        assert!(!cache.is_empty());
        assert_eq!(cache.get(&0), Some(gpu_info));
    }
    #[test]
    fn test_multi_gpu_info_cache_multiple_keys() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info1 = GpuInfo::write_vendor(crate::vendor::Vendor::Nvidia);
        let gpu_info2 = GpuInfo::write_vendor(crate::vendor::Vendor::Amd);
        cache.set(0, gpu_info1.clone());
        cache.set(1, gpu_info2.clone());
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&0), Some(gpu_info1));
        assert_eq!(cache.get(&1), Some(gpu_info2));
    }
}
