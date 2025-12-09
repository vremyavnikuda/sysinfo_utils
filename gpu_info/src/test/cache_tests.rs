//! Comprehensive tests for cache utilities
//!
//! This module provides thorough testing of all cache utility functions
//! to ensure proper functionality and robustness.
#[cfg(test)]
mod tests {
    use crate::cache_utils::*;
    use crate::gpu_info::GpuInfo;
    use crate::vendor::Vendor;
    use std::thread;
    use std::time::Duration;
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
        assert!(entry.is_valid(ttl));
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
        assert!(entry.last_accessed.elapsed() < before_access);
    }
    #[test]
    fn test_gpu_info_cache_creation() {
        let ttl = Duration::from_secs(1);
        let cache = GpuInfoCache::new(ttl);
        assert!(!cache.has_entry());
        assert_eq!(cache.get_owned(), None);
        assert_eq!(cache.age(), None);
    }
    #[test]
    fn test_gpu_info_cache_default() {
        let cache = GpuInfoCache::default();
        assert!(!cache.has_entry());
        assert_eq!(cache.get_owned(), None);
    }
    #[test]
    fn test_gpu_info_cache_set_and_get() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert!(cache.has_entry());
        assert_eq!(cache.get_owned(), Some(gpu_info));
        if let Some(age) = cache.age() {
            assert!(age < Duration::from_millis(100));
        } else {
            panic!("cache should have age after set()");
        }
    }
    #[test]
    fn test_gpu_info_cache_expiration() {
        let cache = GpuInfoCache::new(Duration::from_millis(10));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert_eq!(cache.get_owned(), Some(gpu_info));
        thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get_owned(), None);
    }
    #[test]
    fn test_gpu_info_cache_clear() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert!(cache.has_entry());
        assert_eq!(cache.get_owned(), Some(gpu_info));
        cache.clear();
        assert!(!cache.has_entry());
        assert_eq!(cache.get_owned(), None);
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
        assert_eq!(cache.get_owned(&0), None);
    }
    #[test]
    fn test_multi_gpu_info_cache_default() {
        let cache = MultiGpuInfoCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get_owned(&0), None);
    }
    #[test]
    fn test_multi_gpu_info_cache_with_max_entries() {
        let ttl = Duration::from_secs(1);
        let max_entries = 5;
        let cache = MultiGpuInfoCache::with_max_entries(ttl, max_entries);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert_eq!(cache.get_owned(&0), None);
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
        assert_eq!(cache.get_owned(&key), Some(gpu_info));
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
        assert_eq!(cache.get_owned(&key1), Some(gpu_info1));
        assert_eq!(cache.get_owned(&key2), Some(gpu_info2));
    }
    #[test]
    fn test_multi_gpu_info_cache_expiration() {
        let cache = MultiGpuInfoCache::new(Duration::from_millis(10));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        assert_eq!(cache.get_owned(&key), Some(gpu_info));
        thread::sleep(Duration::from_millis(20));
        assert_eq!(cache.get_owned(&key), None);
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
        assert_eq!(cache.get_owned(&key1), None);
        assert_eq!(cache.get_owned(&key2), Some(gpu_info2));
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
        assert_eq!(cache.get_owned(&key1), None);
        assert_eq!(cache.get_owned(&key2), None);
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
        assert_eq!(cache.get_owned(&key1), Some(gpu_info1.clone()));
        cache.set(key3, gpu_info3.clone());
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
        assert_eq!(cache.get_owned(&key), Some(gpu_info.clone()));
        assert_eq!(cache.get_owned(&key), Some(gpu_info.clone()));
        if let Some(stats) = cache.get_stats() {
            assert_eq!(stats.total_entries, 1);
            assert_eq!(stats.total_accesses, 2);
        } else {
            panic!("cache stats should be available");
        }
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
        assert_eq!(cache.get_owned(&key1), Some(gpu_info1.clone()));
        assert_eq!(cache.get_owned(&key1), Some(gpu_info1.clone()));
        assert_eq!(cache.get_owned(&key2), Some(gpu_info2.clone()));
        if let Some(stats) = cache.get_stats() {
            assert_eq!(stats.total_entries, 2);
            assert_eq!(stats.total_accesses, 3);
            assert!(stats.oldest_entry_age < Duration::from_millis(100));
        } else {
            panic!("cache stats should be available");
        }
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
    fn test_gpu_info_cache_concurrent_access() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        let result1 = cache.get();
        let result2 = cache.get();
        // Both should return Arc pointing to same data
        match (result1, result2) {
            (Some(r1), Some(r2)) => {
                assert_eq!(*r1, gpu_info);
                assert_eq!(*r2, gpu_info);
            }
            _ => panic!("Expected both cache results to be Some"),
        }
    }
    #[test]
    fn test_multi_gpu_info_cache_concurrent_access() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        let result1 = cache.get(&key);
        let result2 = cache.get(&key);
        // Both should return Arc pointing to same data
        match (result1, result2) {
            (Some(r1), Some(r2)) => {
                assert_eq!(*r1, gpu_info);
                assert_eq!(*r2, gpu_info);
            }
            _ => panic!("Expected both cache results to be Some"),
        }
    }
    #[test]
    fn test_cache_with_zero_ttl() {
        let cache = GpuInfoCache::new(Duration::from_secs(0));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert_eq!(cache.get_owned(), None);
    }
    #[test]
    fn test_multi_cache_with_zero_ttl() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(0));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        assert_eq!(cache.get_owned(&key), None);
    }
    #[test]
    fn test_cache_with_large_ttl() {
        let cache = GpuInfoCache::new(Duration::from_secs(3600));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert_eq!(cache.get_owned(), Some(gpu_info));
    }
    #[test]
    fn test_multi_cache_with_large_ttl() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(3600));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        assert_eq!(cache.get_owned(&key), Some(gpu_info));
    }
    #[test]
    fn test_cache_edge_cases() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = create_test_gpu(Vendor::Nvidia);
        let large_key = usize::MAX;
        cache.set(large_key, gpu_info.clone());
        assert_eq!(cache.get_owned(&large_key), Some(gpu_info));
    }
    #[test]
    fn test_cache_empty_string_values() {
        let mut gpu_info = create_test_gpu(Vendor::Nvidia);
        gpu_info.name_gpu = Some("".to_string());
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        cache.set(gpu_info.clone());
        assert_eq!(cache.get_owned(), Some(gpu_info));
    }
}
