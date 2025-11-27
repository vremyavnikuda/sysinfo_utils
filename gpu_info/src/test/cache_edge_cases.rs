//! Additional edge case tests for cache utilities
#[cfg(test)]
mod tests {
    use crate::cache_utils::*;
    use crate::gpu_info::GpuInfo;
    use crate::vendor::Vendor;
    use std::time::Duration;
    #[test]
    fn test_cache_with_multiple_threads() {
        use std::sync::Arc;
        use std::thread;
        let cache = Arc::new(MultiGpuInfoCache::new(Duration::from_secs(1)));
        let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        let mut handles = vec![];
        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let gpu_info_clone = gpu_info.clone();
            let handle = thread::spawn(move || {
                let result = cache_clone.get_owned(&key);
                assert_eq!(result, Some(gpu_info_clone));
                cache_clone.set(i + 100, GpuInfo::write_vendor(Vendor::Amd));
                true
            });
            handles.push(handle);
        }
        for handle in handles {
            assert!(handle.join().unwrap());
        }
        assert_eq!(cache.len(), 11);
        assert_eq!(cache.get_owned(&key), Some(gpu_info));
    }
    #[test]
    fn test_cache_with_very_short_ttl() {
        let cache = GpuInfoCache::new(Duration::from_nanos(1));
        let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
        cache.set(gpu_info.clone());
        assert_eq!(cache.get_owned(), None);
    }
    #[test]
    fn test_multi_cache_with_very_short_ttl() {
        let cache = MultiGpuInfoCache::new(Duration::from_nanos(1));
        let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
        let key = 0;
        cache.set(key, gpu_info.clone());
        assert_eq!(cache.get_owned(&key), None);
    }
    #[test]
    fn test_cache_with_zero_max_entries() {
        let cache = MultiGpuInfoCache::with_max_entries(Duration::from_secs(1), 0);
        let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
        for i in 0..100 {
            cache.set(i, gpu_info.clone());
        }
        assert_eq!(cache.len(), 100);
    }
    #[test]
    fn test_cache_with_one_max_entry() {
        let cache = MultiGpuInfoCache::with_max_entries(Duration::from_secs(1), 1);
        let gpu_info1 = GpuInfo::write_vendor(Vendor::Nvidia);
        let gpu_info2 = GpuInfo::write_vendor(Vendor::Amd);
        cache.set(0, gpu_info1.clone());
        assert_eq!(cache.len(), 1);
        cache.set(1, gpu_info2.clone());
        assert!(cache.len() <= 1);
    }
    #[test]
    fn test_cache_stats_with_no_entries() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.total_accesses, 0);
        assert_eq!(stats.oldest_entry_age, Duration::from_secs(0));
    }
    #[test]
    fn test_cache_clear_operations_on_empty_cache() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        cache.clear_all();
        cache.clear_key(&0);
        assert!(cache.is_empty());
    }
    #[test]
    fn test_cache_has_entry_on_nonexistent_key() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        assert!(!cache.has_entry(&999));
    }
    #[test]
    fn test_cache_get_on_nonexistent_key() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        assert_eq!(cache.get_owned(&999), None);
    }
    #[test]
    fn test_cache_with_very_large_key() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
        let large_key = usize::MAX;
        cache.set(large_key, gpu_info.clone());
        assert_eq!(cache.get_owned(&large_key), Some(gpu_info));
        assert!(cache.has_entry(&large_key));
        cache.clear_key(&large_key);
        assert!(!cache.has_entry(&large_key));
    }
    #[test]
    fn test_cache_entry_methods_with_immediate_access() {
        let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
        let entry = CacheEntry::new(gpu_info);
        assert!(entry.is_valid(Duration::from_secs(1)));
        assert!(entry.age() < Duration::from_millis(1));
        assert_eq!(entry.access_count, 0);
    }
    #[test]
    fn test_gpubuffer_cache_methods_with_immediate_access() {
        let cache = GpuInfoCache::new(Duration::from_secs(1));
        assert!(!cache.has_entry());
        assert_eq!(cache.get_owned(), None);
        assert_eq!(cache.age(), None);
    }
    #[test]
    fn test_multi_gpu_cache_methods_with_immediate_access() {
        let cache = MultiGpuInfoCache::new(Duration::from_secs(1));
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert!(!cache.has_entry(&0));
        assert_eq!(cache.get_owned(&0), None);
    }
}
