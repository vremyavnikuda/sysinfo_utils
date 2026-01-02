//! Comprehensive tests for GPU manager functionality
//!
//! These tests cover multi-GPU detection, management, caching, performance under load,
//! and error handling scenarios for the GPU manager system.
//!
//! ## Performance Considerations
//!
//! GPU manager tests may experience varying performance due to:
//! - Hardware detection overhead (NVML, ADL library initialization)
//! - Absence of real GPU hardware in test environments
//! - FFI library timeouts when no hardware is present
//! - Virtualized or containerized test environments
//! - CI/CD environments with limited resources

#[cfg(test)]
mod tests {
    use crate::gpu_manager::GpuManager;
    use crate::vendor::Vendor;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::task::JoinSet;

    /// Test basic GPU manager creation
    #[test]
    fn test_gpu_manager_creation() {
        let manager = GpuManager::new();
        println!("GPU Manager created with {} GPUs", manager.gpu_count());
        if manager.gpu_count() == 0 {
            println!("No GPUs detected in test environment");
        } else {
            println!("Detected GPUs:");
            for (i, gpu) in manager.get_all_gpus().iter().enumerate() {
                println!("  GPU #{}: {:?} - {:?}", i, gpu.vendor, gpu.name_gpu);
            }
        }
    }

    /// Test GPU manager with custom cache TTL
    #[test]
    fn test_gpu_manager_custom_cache_ttl() {
        let cache_ttl = Duration::from_millis(100);
        let manager = GpuManager::with_cache_ttl(cache_ttl);
        println!("GPU Manager created with custom cache TTL: {:?}", cache_ttl);
        let gpu_count = manager.gpu_count();
        let stats = manager.get_gpu_statistics();
        assert_eq!(gpu_count, stats.total_gpus);
        println!("Manager with custom TTL detected {} GPUs", gpu_count);
    }

    /// Test GPU manager with cache configuration
    #[test]
    fn test_gpu_manager_cache_config() {
        let cache_ttl = Duration::from_millis(200);
        let max_entries = 50;
        let manager = GpuManager::with_cache_config(cache_ttl, max_entries);
        println!(
            "GPU Manager created with cache config: TTL={:?}, max_entries={}",
            cache_ttl, max_entries
        );
        let gpu_count = manager.gpu_count();
        let all_gpus = manager.get_all_gpus();
        assert_eq!(gpu_count, all_gpus.len());
        if gpu_count > 0 {
            let cached_gpu = manager.get_gpu_cached(0);
            assert!(cached_gpu.is_some());
        }
        println!("Manager with cache config detected {} GPUs", gpu_count);
    }

    /// Test GPU retrieval methods
    #[test]
    fn test_gpu_retrieval_methods() {
        let manager = GpuManager::new();
        let all_gpus = manager.get_all_gpus();
        let all_gpus_owned = manager.get_all_gpus_owned();
        assert_eq!(all_gpus.len(), all_gpus_owned.len());
        let primary_gpu = manager.get_primary_gpu();
        let primary_gpu_owned = manager.get_primary_gpu_owned();
        if let Some(primary) = primary_gpu {
            assert!(primary_gpu_owned.is_some());
            let owned = primary_gpu_owned.unwrap();
            assert_eq!(primary.vendor, owned.vendor);
            assert_eq!(primary.name_gpu, owned.name_gpu);
            println!("Primary GPU: {:?} - {:?}", owned.vendor, owned.name_gpu);
        } else {
            println!("No primary GPU available");
        }
        if manager.gpu_count() > 0 {
            let gpu_0 = manager.get_gpu_by_index(0);
            let gpu_0_owned = manager.get_gpu_by_index_owned(0);
            assert!(gpu_0.is_some());
            assert!(gpu_0_owned.is_some());
        }
        let invalid_gpu = manager.get_gpu_by_index(9999);
        assert!(invalid_gpu.is_none());
    }

    /// Test GPU retrieval by vendor
    #[test]
    fn test_gpu_retrieval_by_vendor() {
        let manager = GpuManager::new();
        let vendors = vec![Vendor::Nvidia, Vendor::Amd, Vendor::Unknown];
        for vendor in vendors {
            let gpus_ref = manager.get_gpus_by_vendor(vendor);
            let gpus_owned = manager.get_gpus_by_vendor_owned(vendor);
            assert_eq!(gpus_ref.len(), gpus_owned.len());
            println!("Found {} GPUs for vendor {:?}", gpus_ref.len(), vendor);
            for (ref_gpu, owned_gpu) in gpus_ref.iter().zip(gpus_owned.iter()) {
                assert_eq!(ref_gpu.vendor, owned_gpu.vendor);
                assert_eq!(ref_gpu.vendor, vendor);
            }
        }
    }

    /// Test primary GPU selection and changing
    #[test]
    fn test_primary_gpu_management() {
        let mut manager = GpuManager::new();
        if manager.gpu_count() == 0 {
            println!("No GPUs to test primary GPU management");
            return;
        }
        let result = manager.set_primary_gpu(0);
        assert!(result.is_ok());
        let invalid_result = manager.set_primary_gpu(9999);
        assert!(invalid_result.is_err());
        println!("Primary GPU management test completed");
    }

    /// Test GPU refresh operations
    #[test]
    fn test_gpu_refresh_operations() {
        let mut manager = GpuManager::new();
        if manager.gpu_count() == 0 {
            println!("No GPUs to test refresh operations");
            return;
        }
        let refresh_all_result = manager.refresh_all_gpus();
        match refresh_all_result {
            Ok(()) => println!("Successfully refreshed all GPUs"),
            Err(e) => println!("Failed to refresh all GPUs (expected in test env): {}", e),
        }
        let refresh_gpu_result = manager.refresh_gpu(0);
        match refresh_gpu_result {
            Ok(()) => println!("Successfully refreshed GPU #0"),
            Err(e) => println!("Failed to refresh GPU #0 (expected in test env): {}", e),
        }
        let refresh_primary_result = manager.refresh_primary_gpu();
        match refresh_primary_result {
            Ok(()) => println!("Successfully refreshed primary GPU"),
            Err(e) => println!(
                "Failed to refresh primary GPU (expected in test env): {}",
                e
            ),
        }
        let refresh_invalid_result = manager.refresh_gpu(9999);
        assert!(refresh_invalid_result.is_err());
    }

    /// Test GPU caching functionality
    #[test]
    fn test_gpu_caching() {
        let manager = GpuManager::new();
        if manager.gpu_count() == 0 {
            println!("No GPUs to test caching");
            return;
        }
        let cached_gpu_1 = manager.get_gpu_cached(0);
        let cached_gpu_2 = manager.get_gpu_cached(0);
        if let (Some(gpu1), Some(gpu2)) = (cached_gpu_1, cached_gpu_2) {
            assert_eq!(gpu1.vendor, gpu2.vendor);
            assert_eq!(gpu1.name_gpu, gpu2.name_gpu);
            println!("GPU caching working correctly");
        }
        let cached_primary_1 = manager.get_primary_gpu_cached();
        let cached_primary_2 = manager.get_primary_gpu_cached();
        if let (Some(primary1), Some(primary2)) = (cached_primary_1, cached_primary_2) {
            assert_eq!(primary1.vendor, primary2.vendor);
            println!("Primary GPU caching working correctly");
        }
        if let Some(cache_stats) = manager.get_cache_stats() {
            println!(
                "Cache stats: entries={}, accesses={}, age={:?}",
                cache_stats.total_entries, cache_stats.total_accesses, cache_stats.oldest_entry_age
            );
        }
    }

    /// Test GPU statistics calculation
    #[test]
    fn test_gpu_statistics() {
        let manager = GpuManager::new();
        let stats = manager.get_gpu_statistics();
        println!("GPU Statistics:");
        println!("  Total GPUs: {}", stats.total_gpus);
        println!("  NVIDIA count: {}", stats.nvidia_count);
        println!("  AMD count: {}", stats.amd_count);
        println!("  Intel count: {}", stats.intel_count);
        println!("  Apple count: {}", stats.apple_count);
        println!("  Unknown count: {}", stats.unknown_count);
        if let Some(avg_temp) = stats.average_temperature() {
            println!("  Average temperature: {:.1}°C", avg_temp);
            assert!((0.0..=150.0).contains(&avg_temp));
        }
        if let Some(total_power) = stats.total_power_consumption() {
            println!("  Total power consumption: {:.1}W", total_power);
            assert!(total_power >= 0.0);
        }
        let vendor_sum = stats.nvidia_count
            + stats.amd_count
            + stats.intel_count
            + stats.apple_count
            + stats.unknown_count;
        assert_eq!(vendor_sum, stats.total_gpus);
    }

    /// Test active GPU detection
    #[test]
    fn test_active_gpu_detection() {
        let manager = GpuManager::new();
        let all_active = manager.all_gpus_active();
        let active_indices = manager.get_active_gpu_indices();
        println!("All GPUs active: {}", all_active);
        println!("Active GPU indices: {:?}", active_indices);
        for index in active_indices {
            assert!(index < manager.gpu_count());
        }
    }

    /// Lightweight load test for resource-constrained environments
    #[tokio::test]
    async fn test_lightweight_managers_load() {
        const MANAGER_COUNT: usize = 5; // Very small load
        let mut successful_creations = 0;
        println!("Starting lightweight load test for constrained environments");
        for i in 0..MANAGER_COUNT {
            let start = std::time::Instant::now();
            let manager = GpuManager::new();
            let creation_time = start.elapsed();
            let gpu_count = manager.gpu_count();
            successful_creations += 1;
            println!("Manager #{}: {} GPUs in {:?}", i, gpu_count, creation_time);
            if creation_time > Duration::from_secs(10) {
                println!("Note: Slow creation detected - likely no real GPUs available");
            }
        }
        println!(
            "Lightweight load test: {}/{} managers created",
            successful_creations, MANAGER_COUNT
        );
        assert_eq!(successful_creations, MANAGER_COUNT);
    }

    /// Load test: Multiple managers with optimized performance
    #[tokio::test]
    async fn test_multiple_managers_load() {
        const MANAGER_COUNT: usize = 10;
        let mut join_set = JoinSet::new();
        let start_time = std::time::Instant::now();
        for i in 0..MANAGER_COUNT {
            join_set.spawn(async move {
                let manager_start = std::time::Instant::now();
                let manager = GpuManager::new();
                let creation_time = manager_start.elapsed();
                let gpu_count = manager.gpu_count();
                let stats = manager.get_gpu_statistics();
                (i, gpu_count, stats.total_gpus, creation_time)
            });
        }
        let mut total_detections = 0;
        let mut successful_creations = 0;
        let mut total_creation_time = Duration::from_secs(0);
        let _timeout_duration = Duration::from_secs(30); // Available for future timeout implementation
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((manager_id, gpu_count, stats_total, creation_time)) => {
                    successful_creations += 1;
                    total_detections += gpu_count;
                    total_creation_time += creation_time;
                    if manager_id % 5 == 0 {
                        println!(
                            "Manager #{}: {} GPUs detected in {:?}",
                            manager_id, gpu_count, creation_time
                        );
                    }
                    assert_eq!(gpu_count, stats_total);
                    if creation_time > Duration::from_secs(5) {
                        println!(
                            "Warning: Manager #{} creation took {:?} (slow)",
                            manager_id, creation_time
                        );
                    }
                }
                Err(e) => {
                    println!("Manager creation failed: {}", e);
                }
            }
        }
        let total_time = start_time.elapsed();
        let avg_creation_time = if successful_creations > 0 {
            total_creation_time / successful_creations as u32
        } else {
            Duration::from_secs(0)
        };
        println!("Multiple managers load test results:");
        println!(
            "  Created {} managers in {:?}",
            successful_creations, total_time
        );
        println!("  Average creation time: {:?}", avg_creation_time);
        println!("  Total GPU detections: {}", total_detections);
        println!(
            "  Test environment performance: {}",
            if avg_creation_time > Duration::from_secs(2) {
                "Slow (likely no real GPUs)"
            } else {
                "Good"
            }
        );
        assert_eq!(successful_creations, MANAGER_COUNT);
        assert!(
            avg_creation_time < Duration::from_secs(10),
            "Manager creation too slow: {:?} (may indicate no real GPUs)",
            avg_creation_time
        );

        assert!(
            total_time < Duration::from_secs(60),
            "Test took too long: {:?}",
            total_time
        );
    }

    /// Stress test: Rapid cache access
    ///
    /// Note: Reduced iteration count to avoid long test times with PDH metrics (500ms per cache miss)
    #[test]
    fn test_rapid_cache_access() {
        let manager = GpuManager::new();
        if manager.gpu_count() == 0 {
            println!("No GPUs to test cache access");
            return;
        }
        const ACCESS_COUNT: usize = 100;
        let start_time = std::time::Instant::now();
        let mut cache_hits = 0;
        for i in 0..ACCESS_COUNT {
            let gpu_index = i % manager.gpu_count();
            if manager.get_gpu_cached(gpu_index).is_some() {
                cache_hits += 1;
            }
        }
        let access_time = start_time.elapsed();
        let accesses_per_second = ACCESS_COUNT as f64 / access_time.as_secs_f64();
        println!("Rapid cache access test:");
        println!("  {} accesses in {:?}", ACCESS_COUNT, access_time);
        println!("  Cache hits: {}", cache_hits);
        println!("  Accesses per second: {:.0}", accesses_per_second);
        assert!(cache_hits > 0, "Cache should have some hits");
        assert!(
            access_time.as_secs() < 60,
            "Test took too long: {:?}",
            access_time
        );
    }

    /// Test concurrent GPU operations
    #[tokio::test]
    async fn test_concurrent_gpu_operations() {
        let manager = Arc::new(tokio::sync::Mutex::new(GpuManager::new()));
        let mut join_set = JoinSet::new();
        const CONCURRENT_OPS: usize = 20;
        for i in 0..CONCURRENT_OPS {
            let manager_clone = manager.clone();
            join_set.spawn(async move {
                let mgr = manager_clone.lock().await;
                let operation_type = i % 4;
                let result: Result<String, crate::gpu_info::GpuError> = match operation_type {
                    0 => {
                        let gpus = mgr.get_all_gpus();
                        Ok(format!("GetAll: {} GPUs", gpus.len()))
                    }
                    1 => {
                        let primary = mgr.get_primary_gpu();
                        Ok(format!("GetPrimary: {:?}", primary.is_some()))
                    }
                    2 => {
                        let stats = mgr.get_gpu_statistics();
                        Ok(format!("GetStats: {} total", stats.total_gpus))
                    }
                    3 => {
                        let active_count = mgr.get_active_gpu_indices().len();
                        Ok(format!("GetActive: {} active", active_count))
                    }
                    _ => unreachable!(),
                };
                (i, result)
            });
        }
        let mut successful_ops = 0;
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((op_id, operation_result)) => match operation_result {
                    Ok(description) => {
                        successful_ops += 1;
                        if op_id % 5 == 0 {
                            println!("Operation #{}: {}", op_id, description);
                        }
                    }
                    Err(e) => {
                        println!("Operation #{} failed: {}", op_id, e);
                    }
                },
                Err(e) => {
                    println!("Task #{} join error: {}", 0, e);
                }
            }
        }
        println!(
            "Concurrent operations test: {}/{} successful",
            successful_ops, CONCURRENT_OPS
        );
        assert_eq!(successful_ops, CONCURRENT_OPS);
    }

    /// Performance benchmark for GPU manager operations
    ///
    /// Note: Reduced iteration count to avoid long test times with PDH metrics (500ms per cache miss)
    #[test]
    fn test_gpu_manager_performance() {
        let manager = GpuManager::new();
        if manager.gpu_count() == 0 {
            println!("No GPUs for performance testing");
            return;
        }
        const ITERATIONS: usize = 10;
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = manager.get_all_gpus();
        }
        let get_all_time = start.elapsed();
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = manager.get_primary_gpu();
        }
        let get_primary_time = start.elapsed();
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = manager.get_gpu_statistics();
        }
        let get_stats_time = start.elapsed();
        let start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = manager.get_gpu_cached(0);
        }
        let cached_access_time = start.elapsed();
        println!(
            "GPU Manager performance benchmark ({} iterations):",
            ITERATIONS
        );
        println!(
            "  get_all_gpus: {:?} (avg: {:?})",
            get_all_time,
            get_all_time / ITERATIONS as u32
        );
        println!(
            "  get_primary_gpu: {:?} (avg: {:?})",
            get_primary_time,
            get_primary_time / ITERATIONS as u32
        );
        println!(
            "  get_gpu_statistics: {:?} (avg: {:?})",
            get_stats_time,
            get_stats_time / ITERATIONS as u32
        );
        println!(
            "  cached_access: {:?} (avg: {:?})",
            cached_access_time,
            cached_access_time / ITERATIONS as u32
        );
        let avg_get_all = get_all_time / ITERATIONS as u32;
        let avg_cached = cached_access_time / ITERATIONS as u32;
        assert!(
            avg_get_all < Duration::from_secs(1),
            "get_all_gpus too slow: {:?}",
            avg_get_all
        );
        assert!(
            avg_cached < Duration::from_secs(1),
            "cached access too slow: {:?}",
            avg_cached
        );
    }

    /// Integration test: Full GPU manager workflow
    #[tokio::test]
    async fn test_full_gpu_manager_workflow() {
        println!("Starting full GPU manager workflow test");
        let mut manager = GpuManager::new();
        println!("Created GPU manager with {} GPUs", manager.gpu_count());
        let _all_gpus = manager.get_all_gpus();
        let stats = manager.get_gpu_statistics();
        println!("GPU analysis:");
        println!("  Total GPUs: {}", stats.total_gpus);
        println!(
            "  Vendor distribution: N={}, A={}, I={}, Ap={}, U={}",
            stats.nvidia_count,
            stats.amd_count,
            stats.intel_count,
            stats.apple_count,
            stats.unknown_count
        );
        if let Some(primary_gpu) = manager.get_primary_gpu() {
            println!(
                "Primary GPU: {:?} - {:?}",
                primary_gpu.vendor, primary_gpu.name_gpu
            );
            match manager.refresh_primary_gpu() {
                Ok(()) => println!("  Primary GPU refreshed successfully"),
                Err(e) => println!("  Primary GPU refresh failed: {}", e),
            }
        }
        for vendor in [Vendor::Nvidia, Vendor::Amd, Vendor::Unknown] {
            let vendor_gpus = manager.get_gpus_by_vendor(vendor);
            if !vendor_gpus.is_empty() {
                println!("Found {} {:?} GPUs", vendor_gpus.len(), vendor);
            }
        }
        let cache_start = std::time::Instant::now();
        for i in 0..100 {
            let gpu_index = i % manager.gpu_count().max(1);
            let _ = manager.get_gpu_cached(gpu_index);
        }
        let cache_time = cache_start.elapsed();
        println!("100 cached accesses in {:?}", cache_time);
        if manager.gpu_count() > 0 {
            let refresh_start = std::time::Instant::now();
            match manager.refresh_all_gpus() {
                Ok(()) => {
                    let refresh_time = refresh_start.elapsed();
                    println!("All GPUs refreshed in {:?}", refresh_time);
                }
                Err(e) => {
                    println!("GPU refresh failed (expected): {}", e);
                }
            }
        }
        let final_stats = manager.get_gpu_statistics();
        if let Some(cache_stats) = manager.get_cache_stats() {
            println!(
                "Final cache stats - entries: {}, accesses: {}",
                cache_stats.total_entries, cache_stats.total_accesses
            );
        }
        assert_eq!(final_stats.total_gpus, manager.gpu_count());
        assert!(cache_time < Duration::from_millis(100));
    }
}
