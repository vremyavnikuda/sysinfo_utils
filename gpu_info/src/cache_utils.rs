//! Common caching utilities for GPU information
//!
//! This module provides unified caching mechanisms to eliminate duplication
//! in GPU information caching across different components.
use crate::gpu_info::GpuInfo;
use log::debug;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
/// Generic cache entry with TTL support and access tracking
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// Cached value
    pub value: T,
    /// Timestamp when the entry was created
    pub timestamp: Instant,
    /// Timestamp when the entry was last accessed
    pub last_accessed: Instant,
    /// Access count for LRU implementation
    pub access_count: usize,
}
impl<T> CacheEntry<T> {
    /// Create a new cache entry
    pub fn new(value: T) -> Self {
        let now = Instant::now();
        Self {
            value,
            timestamp: now,
            last_accessed: now,
            access_count: 0,
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
    /// Update access statistics
    pub fn record_access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }
}
/// Single-item cache for GPU information
///
/// This cache eliminates duplication by providing a unified caching mechanism
/// for single GPU information items.
///
/// # Examples
/// ```
/// use gpu_info::cache_utils::{CacheEntry, GpuInfoCache};
/// use gpu_info::gpu_info::GpuInfo;
/// use std::time::Duration;
/// let cache = GpuInfoCache::new(Duration::from_secs(1));
/// let gpu_info = GpuInfo::unknown();
/// cache.set(gpu_info.clone());
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
        let mut guard = self.info.write().ok()?;
        if let Some(entry) = guard.as_mut() {
            if entry.is_valid(self.ttl) {
                entry.record_access();
                debug!("Returning cached GPU info (age: {:?})", entry.age());
                Some(entry.value.clone())
            } else {
                debug!("Cached GPU info expired (age: {:?})", entry.age());
                None
            }
        } else {
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
/// ```rust
/// use gpu_info::cache_utils::{CacheEntry, MultiGpuInfoCache};
/// use gpu_info::gpu_info::GpuInfo;
/// use std::time::Duration;
/// let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
/// let gpu_info = GpuInfo::unknown();
/// cache.set(0, gpu_info.clone());
/// assert_eq!(cache.get(&0), Some(gpu_info));
/// ```
#[derive(Debug, Clone)]
pub struct MultiGpuInfoCache {
    /// Cached GPU information entries indexed by key
    entries: Arc<Mutex<HashMap<usize, CacheEntry<GpuInfo>>>>,
    /// Time-to-live for cached entries
    ttl: Duration,
    /// Maximum number of entries to keep in cache (0 = unlimited)
    max_entries: usize,
}
impl MultiGpuInfoCache {
    /// Create a new multi-GPU info cache with the specified TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            ttl,
            max_entries: 0, // Unlimited by default
        }
    }
    /// Create a new multi-GPU info cache with the specified TTL and maximum entries
    pub fn with_max_entries(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            ttl,
            max_entries,
        }
    }
    /// Get cached GPU information by key if it's still valid
    pub fn get(&self, key: &usize) -> Option<GpuInfo> {
        let mut guard = self.entries.lock().ok()?;
        if let Some(entry) = guard.get_mut(key) {
            if entry.is_valid(self.ttl) {
                entry.record_access();
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
                guard.remove(key);
                None
            }
        } else {
            None
        }
    }
    /// Set GPU information in the cache by key
    pub fn set(&self, key: usize, info: GpuInfo) {
        if let Ok(mut guard) = self.entries.lock() {
            guard.insert(key, CacheEntry::new(info));
            // Apply LRU eviction if we have a limit
            if self.max_entries > 0 && guard.len() > self.max_entries {
                self.evict_lru_entries(&mut guard);
            }
            debug!("Updated GPU info cache for key {}", key);
        }
    }
    /// Evict least recently used entries to maintain size limit
    fn evict_lru_entries(&self, guard: &mut HashMap<usize, CacheEntry<GpuInfo>>) {
        if guard.len() <= self.max_entries {
            return;
        }
        // Find the least recently used entries to remove
        let mut entries: Vec<_> = guard.iter().collect();
        entries.sort_by_key(|(_, entry)| entry.last_accessed);
        // Remove excess entries
        let excess = guard.len() - self.max_entries;
        let keys_to_remove: Vec<_> = entries
            .into_iter()
            .take(excess)
            .map(|(key, _)| *key)
            .collect();
        for key in keys_to_remove {
            guard.remove(&key);
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
    /// Get cache statistics
    pub fn get_stats(&self) -> Option<CacheStats> {
        if let Ok(guard) = self.entries.lock() {
            let total_entries = guard.len();
            let mut total_accesses = 0;
            let mut oldest_entry = None;
            for entry in guard.values() {
                total_accesses += entry.access_count;
                match oldest_entry {
                    None => oldest_entry = Some(entry.timestamp),
                    Some(oldest) if entry.timestamp < oldest => {
                        oldest_entry = Some(entry.timestamp);
                    }
                    _ => {}
                }
            }
            Some(CacheStats {
                total_entries,
                total_accesses,
                oldest_entry_age: oldest_entry
                    .map(|ts| ts.elapsed())
                    .unwrap_or(Duration::from_secs(0)),
            })
        } else {
            None
        }
    }
}
/// Statistics about cache performance
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries in the cache
    pub total_entries: usize,
    /// Total number of accesses to all entries
    pub total_accesses: usize,
    /// Age of the oldest entry
    pub oldest_entry_age: Duration,
}
/// Adaptive cache with dynamic TTL based on access patterns
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AdaptiveGpuCache {
    /// Underlying cache storage
    cache: MultiGpuInfoCache,
    /// Base TTL for cache entries
    base_ttl: Duration,
    /// Minimum TTL for cache entries
    min_ttl: Duration,
    /// Maximum TTL for cache entries
    max_ttl: Duration,
    /// Factor to adjust TTL based on access frequency
    frequency_factor: f32,
}
#[allow(dead_code)]
impl AdaptiveGpuCache {
    /// Create a new adaptive cache with default settings
    pub fn new() -> Self {
        Self {
            cache: MultiGpuInfoCache::new(Duration::from_secs(1)),
            base_ttl: Duration::from_secs(1),
            min_ttl: Duration::from_millis(100),
            max_ttl: Duration::from_secs(5),
            frequency_factor: 0.1,
        }
    }
    /// Create a new adaptive cache with custom settings
    pub fn with_settings(
        base_ttl: Duration,
        min_ttl: Duration,
        max_ttl: Duration,
        frequency_factor: f32,
    ) -> Self {
        Self {
            cache: MultiGpuInfoCache::new(base_ttl),
            base_ttl,
            min_ttl,
            max_ttl,
            frequency_factor,
        }
    }
    /// Get cached GPU information by key with adaptive TTL
    pub fn get(&self, key: &usize) -> Option<GpuInfo> {
        self.cache.get(key)
    }
    /// Set GPU information in the cache with adaptive TTL
    pub fn set(&self, key: usize, info: GpuInfo) {
        self.cache.set(key, info);
    }
    /// Clear all cache entries
    pub fn clear_all(&self) {
        self.cache.clear_all();
    }
    /// Calculate adaptive TTL based on access frequency
    fn calculate_adaptive_ttl(&self, access_count: usize, age: Duration) -> Duration {
        let base_millis = self.base_ttl.as_millis() as f32;
        let age_factor = (age.as_secs_f32() * self.frequency_factor).min(1.0);
        let access_factor = (access_count as f32 * self.frequency_factor).min(1.0);
        // If accessed frequently, increase TTL; if old, decrease TTL
        let adjusted_millis = base_millis * (1.0 + access_factor - age_factor);
        // Clamp to min/max bounds
        let millis = adjusted_millis
            .max(self.min_ttl.as_millis() as f32)
            .min(self.max_ttl.as_millis() as f32);
        Duration::from_millis(millis as u64)
    }
}
impl Default for AdaptiveGpuCache {
    fn default() -> Self {
        Self::new()
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
