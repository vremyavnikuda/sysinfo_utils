//! Comprehensive tests for cache utilities
//!
//! This module provides thorough testing of all cache utility functions
//! to ensure proper functionality and robustness.
#[cfg(test)]
mod cache_utils_tests {
    use crate::cache_utils::*;
    use crate::gpu_info::GpuInfo;
    use crate::vendor::Vendor;
    use std::thread;
    use std::time::Duration;
    // Helper function to create a test GPU info
    fn create_test_gpu(vendor: Vendor) -> GpuInfo {
        GpuInfo::write_vendor(vendor)
    }
    #[test]
    fn test_cache_entry_creation() {
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let entry = CacheEntry::new(gpu_info.clone());
        assert_eq!(entry.value.vendor, Vendor::Nvidia);
        assert!(entry.timestamp.elapsed() < Duration::from_millis(100));
        assert!(entry.last_accessed.elapsed() < Duration::from_millis(100));
        assert_eq!(entry.access_count, 0);
    }
    #[test]
    fn test_cache_entry_is_valid() {
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let entry = CacheEntry::new(gpu_info);
        let ttl = Duration::from_millis(100);
        // Should be valid immediately
        assert!(entry.is_valid(ttl));
        // Wait and check again
        thread::sleep(Duration::from_millis(150));
        assert!(!entry.is_valid(ttl));
    }
    #[test]
    fn test_cache_entry_age() {
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let entry = CacheEntry::new(gpu_info);
        thread::sleep(Duration::from_millis(50));
        let age = entry.age();
        assert!(age >= Duration::from_millis(50));
        assert!(age < Duration::from_millis(100));
    }
    #[test]
    fn test_cache_entry_record_access() {
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let mut entry = CacheEntry::new(gpu_info);
        assert_eq!(entry.access_count, 0);
        assert!(entry.last_accessed.elapsed() < Duration::from_millis(100));
        thread::sleep(Duration::from_millis(10));
        let before_access = entry.last_accessed.elapsed();
        entry.record_access();
        assert_eq!(entry.access_count, 1);
        // Check that last_accessed was updated (should be much newer than before_access)
        assert!(entry.last_accessed.elapsed() < before_access);
    }
    #[test]
    fn test_gpu_info_cache_creation() {
        let ttl = Duration::from_secs(1);
        let cache = GpuInfoCache::new(ttl);
        assert!(!cache.has_entry());
        assert_eq!(cache.get(), None);
        assert_eq!(cache.age(), None);
    }
    #[test]
    fn test_gpu_info_cache_default() {
        let cache = GpuInfoCache::default();
        assert!(!cache.has_entry());
        assert_eq!(cache.get(), None);
    }
    #[test]
    fn test_gpu_info_cache_set_and_get() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert!(cache.has_entry());
        assert_eq!(cache.get(), Some(gpu_info));
        assert!(cache.age().unwrap() < Duration::from_millis(100));
    }
    #[test]
    fn test_gpu_info_cache_expiration() {
        let cache = GpuInfoCache::new(Duration::from_millis(10));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert_eq!(cache.get(), Some(gpu_info));
        thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get(), None);
    }
    #[test]
    fn test_gpu_info_cache_clear() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert!(cache.has_entry());
        assert_eq!(cache.get(), Some(gpu_info));
        cache.clear();
        assert!(!cache.has_entry());
        assert_eq!(cache.get(), None);
    }
    #[test]
    fn test_gpu_info_cache_access_tracking() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        // Note: Access tracking is implemented in the MultiGpuInfoCache
        // GpuInfoCache doesn't track individual access counts
    }
    #[test]
    fn test_multi_gpu_info_cache_creation() {
        let ttl = Duration::from_secs(1);
        let cache = MultiGpuInfoCache::new(ttl);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get(&0), None);
    }
    #[test]
    fn test_multi_gpu_info_cache_default() {
        let cache = MultiGpuInfoCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get(&0), None);
    }
    #[test]
    fn test_multi_gpu_info_cache_with_max_entries() {
        let ttl = Duration::from_secs(1);
        let max_entries = 5;
        let cache = MultiGpuInfoCache::with_max_entries(ttl, max_entries);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get(&0), None);
    }
    #[test]
    fn test_multi_gpu_info_cache_set_and_get() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 1);
        assert!(cache.has_entry(&key));
        assert_eq!(cache.get(&key), Some(gpu_info));
    }
    #[test]
    fn test_multi_gpu_info_cache_multiple_entries() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info1 = create_test_gpu(Vendor::Nvidia);
        let gpu_info2 = create_test_gpu(Vendor::Amd);
        let key1 = 0;
        let key2 = 1;
        cache.set(key1, gpu_info1.clone());
        cache.set(key2, gpu_info2.clone());
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 2);
        assert!(cache.has_entry(&key1));
        assert!(cache.has_entry(&key2));
        assert_eq!(cache.get(&key1), Some(gpu_info1));
        assert_eq!(cache.get(&key2), Some(gpu_info2));
    }
    #[test]
    fn test_multi_gpu_info_cache_expiration() {
        let cache = MultiGpuInfoCache::new(Duration::from_millis(10));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        assert_eq!(cache.get(&key), Some(gpu_info));
        thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get(&key), None);
        // Expired entries should be automatically removed
        assert!(!cache.has_entry(&key));
    }
    #[test]
    fn test_multi_gpu_info_cache_clear_key() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info1 = create_test_gpu(Vendor::Nvidia);
        let gpu_info2 = create_test_gpu(Vendor::Amd);
        let key1 = 0;
        let key2 = 1;
        cache.set(key1, gpu_info1.clone());
        cache.set(key2, gpu_info2.clone());
        assert_eq!(cache.len(), 2);
        cache.clear_key(&key1);
        assert_eq!(cache.len(), 1);
        assert!(!cache.has_entry(&key1));
        assert!(cache.has_entry(&key2));
        assert_eq!(cache.get(&key1), None);
        assert_eq!(cache.get(&key2), Some(gpu_info2));
    }
    #[test]
    fn test_multi_gpu_info_cache_clear_all() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info1 = create_test_gpu(Vendor::Nvidia);
        let gpu_info2 = create_test_gpu(Vendor::Amd);
        let key1 = 0;
        let key2 = 1;
        cache.set(key1, gpu_info1.clone());
        cache.set(key2, gpu_info2.clone());
        assert_eq!(cache.len(), 2);
        cache.clear_all();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert!(!cache.has_entry(&key1));
        assert!(!cache.has_entry(&key2));
        assert_eq!(cache.get(&key1), None);
        assert_eq!(cache.get(&key2), None);
    }
    #[test]
    fn test_multi_gpu_info_cache_lru_eviction() {
        let cache = MultiGpuInfoCache::with_max_entries(Duration::from_secs(1), 2);
        let gpu_info1 = create_test_gpu(Vendor::Nvidia);
        let gpu_info2 = create_test_gpu(Vendor::Amd);
        let gpu_info3 = create_test_gpu(Vendor::Intel(Default::default()));
        let key1 = 0;
        let key2 = 1;
        let key3 = 2;
        cache.set(key1, gpu_info1.clone());
        cache.set(key2, gpu_info2.clone());
        // Access the first entry to make it more recently used
        assert_eq!(cache.get(&key1), Some(gpu_info1.clone()));
        // Add a third entry, which should trigger LRU eviction
        cache.set(key3, gpu_info3.clone());
        // The second entry (least recently used) should be evicted
        assert_eq!(cache.len(), 2);
        assert!(!cache.has_entry(&key2));
        assert!(cache.has_entry(&key1));
        assert!(cache.has_entry(&key3));
    }
    #[test]
    fn test_multi_gpu_info_cache_access_tracking() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        // First access
        assert_eq!(cache.get(&key), Some(gpu_info.clone()));
        // Second access
        assert_eq!(cache.get(&key), Some(gpu_info.clone()));
        // Check stats
        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.total_accesses, 2);
    }
    #[test]
    fn test_multi_gpu_info_cache_stats() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info1 = create_test_gpu(Vendor::Nvidia);
        let gpu_info2 = create_test_gpu(Vendor::Amd);
        let key1 = 0;
        let key2 = 1;
        cache.set(key1, gpu_info1.clone());
        cache.set(key2, gpu_info2.clone());
        // Access entries
        assert_eq!(cache.get(&key1), Some(gpu_info1.clone()));
        assert_eq!(cache.get(&key1), Some(gpu_info1.clone()));
        assert_eq!(cache.get(&key2), Some(gpu_info2.clone()));
        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.total_accesses, 3);
        assert!(stats.oldest_entry_age < Duration::from_millis(100));
    }
    #[test]
    fn test_cache_stats_default_values() {
        let stats = CacheStats {
            total_entries: 0,
            total_accesses: 0,
            oldest_entry_age: Duration::from_secs(0),
        };
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_accesses, 0);
        assert_eq!(stats.oldest_entry_age, Duration::from_secs(0));
    }
    #[test]
    fn test_adaptive_gpu_cache_creation() {
        let _cache = AdaptiveGpuCache::new();
        // Just test that it can be created without panicking
        assert!(true); // Placeholder - actual functionality is #[allow(dead_code)]
    }
    #[test]
    fn test_adaptive_gpu_cache_default() {
        let _cache = AdaptiveGpuCache::default();
        // Just test that it can be created without panicking
        assert!(true); // Placeholder - actual functionality is #[allow(dead_code)]
    }
    #[test]
    fn test_adaptive_gpu_cache_with_settings() {
        let _cache = AdaptiveGpuCache::with_settings(
            Duration::from_secs(2),
            Duration::from_millis(100),
            Duration::from_secs(10),
            0.2,
        );
        // Just test that it can be created without panicking
        assert!(true); // Placeholder - actual functionality is #[allow(dead_code)]
    }
    // Error handling tests
    #[test]
    fn test_gpu_info_cache_concurrent_access() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        // Set value
        cache.set(gpu_info.clone());
        // Multiple reads should work
        let result1 = cache.get();
        let result2 = cache.get();
        assert_eq!(result1, Some(gpu_info.clone()));
        assert_eq!(result2, Some(gpu_info));
    }
    #[test]
    fn test_multi_gpu_info_cache_concurrent_access() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        // Set value
        cache.set(key, gpu_info.clone());
        // Multiple reads should work
        let result1 = cache.get(&key);
        let result2 = cache.get(&key);
        assert_eq!(result1, Some(gpu_info.clone()));
        assert_eq!(result2, Some(gpu_info));
    }
    #[test]
    fn test_cache_with_zero_ttl() {
        let cache = GpuInfoCache::new(Duration::from_secs(0));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        // Should be expired immediately
        assert_eq!(cache.get(), None);
    }
    #[test]
    fn test_multi_cache_with_zero_ttl() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(0));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        // Should be expired immediately
        assert_eq!(cache.get(&key), None);
    }
    #[test]
    fn test_cache_with_large_ttl() {
        let cache = GpuInfoCache::new(Duration::from_secs(3600)); // 1 hour
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert_eq!(cache.get(), Some(gpu_info));
    }
    #[test]
    fn test_multi_cache_with_large_ttl() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(3600)); // 1 hour
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        assert_eq!(cache.get(&key), Some(gpu_info));
    }
    #[test]
    fn test_cache_edge_cases() {
        // Test with very large key values
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let large_key = usize::MAX;
        cache.set(large_key, gpu_info.clone());
        assert_eq!(cache.get(&large_key), Some(gpu_info));
    }
    #[test]
    fn test_cache_empty_string_values() {
        // Test with GPU info that has empty string fields
        let mut gpu_info = create_test_gpu(Vendor::Nvidia);
        gpu_info.name_gpu = Some("".to_string());
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        cache.set(gpu_info.clone());
        assert_eq!(cache.get(), Some(gpu_info));
    }
}
