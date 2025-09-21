//! Comprehensive tests for provider manager functionality
//!
//! These tests cover provider registration, GPU detection across multiple vendors,
//! update operations, error handling, and performance under load conditions.

#[cfg(test)]
mod provider_manager_tests {
    use crate::gpu_info::{GpuInfo, GpuProvider, Result};
    use crate::provider_manager::GpuProviderManager;
    use crate::vendor::{IntelGpuType, Vendor};
    use std::sync::Arc;
    use std::sync::Mutex;

    /// Mock provider for testing different vendors
    #[derive(Debug)]
    struct MockProvider {
        vendor: Vendor,
        gpu_count: usize,
        should_fail: bool,
        update_count: Arc<Mutex<usize>>,
    }

    impl MockProvider {
        fn new(vendor: Vendor, gpu_count: usize) -> Self {
            Self {
                vendor,
                gpu_count,
                should_fail: false,
                update_count: Arc::new(Mutex::new(0)),
            }
        }

        fn new_failing(vendor: Vendor) -> Self {
            Self {
                vendor,
                gpu_count: 0,
                should_fail: true,
                update_count: Arc::new(Mutex::new(0)),
            }
        }

        fn get_update_count(&self) -> usize {
            *self.update_count.lock().unwrap()
        }
    }

    impl GpuProvider for MockProvider {
        fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
            if self.should_fail {
                return Err(crate::gpu_info::GpuError::GpuNotFound);
            }
            let mut gpus = Vec::new();
            for i in 0..self.gpu_count {
                let mut gpu = GpuInfo::write_vendor(self.vendor.clone());
                gpu.name_gpu = Some(format!("Mock {} GPU #{}", self.vendor, i));
                gpu.temperature = Some(50.0 + i as f32 * 5.0);
                gpu.utilization = Some(30.0 + i as f32 * 10.0);
                gpu.active = Some(true);
                gpus.push(gpu);
            }
            Ok(gpus)
        }

        fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
            if self.should_fail {
                return Err(crate::gpu_info::GpuError::GpuNotActive);
            }
            *self.update_count.lock().unwrap() += 1;
            gpu.temperature = Some(gpu.temperature.unwrap_or(50.0) + 1.0);
            gpu.utilization = Some(gpu.utilization.unwrap_or(30.0) + 5.0);
            if gpu.name_gpu.is_none() {
                gpu.name_gpu = Some(format!("Updated {} GPU", self.vendor));
            }
            Ok(())
        }

        fn get_vendor(&self) -> Vendor {
            self.vendor.clone()
        }
    }

    /// Test basic provider manager creation
    #[test]
    fn test_provider_manager_creation() {
        let manager = GpuProviderManager::new();
        assert!(manager.get_registered_vendors().is_empty());
        let default_manager = GpuProviderManager::default();
        assert!(default_manager.get_registered_vendors().is_empty());
        println!("Provider manager created successfully");
    }

    /// Test single provider registration
    #[test]
    fn test_single_provider_registration() {
        let mut manager = GpuProviderManager::new();
        let nvidia_provider = MockProvider::new(Vendor::Nvidia, 2);
        manager.register_provider(Vendor::Nvidia, nvidia_provider);
        let vendors = manager.get_registered_vendors();
        assert_eq!(vendors.len(), 1);
        assert!(manager.is_vendor_supported(&Vendor::Nvidia));
        assert!(!manager.is_vendor_supported(&Vendor::Amd));
        println!("Single provider registered successfully");
    }

    /// Test multiple provider registration
    #[test]
    fn test_multiple_provider_registration() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new(Vendor::Nvidia, 2));
        manager.register_provider(Vendor::Amd, MockProvider::new(Vendor::Amd, 1));
        manager.register_provider(
            Vendor::Intel(IntelGpuType::Integrated), 
            MockProvider::new(Vendor::Intel(IntelGpuType::Integrated), 1)
        );
        let vendors = manager.get_registered_vendors();
        assert_eq!(vendors.len(), 3);
        assert!(manager.is_vendor_supported(&Vendor::Nvidia));
        assert!(manager.is_vendor_supported(&Vendor::Amd));
        assert!(manager.is_vendor_supported(&Vendor::Intel(IntelGpuType::Integrated)));
        assert!(!manager.is_vendor_supported(&Vendor::Apple));
        println!("Multiple providers registered: {:?}", vendors);
    }

    /// Test GPU detection from all providers
    #[test]
    fn test_gpu_detection_all_providers() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new(Vendor::Nvidia, 2));
        manager.register_provider(Vendor::Amd, MockProvider::new(Vendor::Amd, 1));
        manager.register_provider(
            Vendor::Intel(IntelGpuType::Discrete),
            MockProvider::new(Vendor::Intel(IntelGpuType::Discrete), 1)
        );
        let detected_gpus = manager.detect_all_gpus();
        assert_eq!(detected_gpus.len(), 4);
        let nvidia_count = detected_gpus.iter().filter(|g| matches!(g.vendor, Vendor::Nvidia)).count();
        let amd_count = detected_gpus.iter().filter(|g| matches!(g.vendor, Vendor::Amd)).count();
        let intel_count = detected_gpus.iter().filter(|g| matches!(g.vendor, Vendor::Intel(_))).count();
        assert_eq!(nvidia_count, 2);
        assert_eq!(amd_count, 1);
        assert_eq!(intel_count, 1);
        println!("Detected {} GPUs: {} NVIDIA, {} AMD, {} Intel", 
                 detected_gpus.len(), nvidia_count, amd_count, intel_count);
        for (i, gpu) in detected_gpus.iter().enumerate() {
            assert!(gpu.name_gpu.is_some());
            assert!(gpu.temperature.is_some());
            assert!(gpu.active == Some(true));
            println!("  GPU #{}: {:?} - {:?}", i, gpu.vendor, gpu.name_gpu);
        }
    }

    /// Test GPU detection with failing providers
    #[test]
    fn test_gpu_detection_with_failures() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new(Vendor::Nvidia, 1));
        manager.register_provider(Vendor::Amd, MockProvider::new_failing(Vendor::Amd));
        manager.register_provider(
            Vendor::Intel(IntelGpuType::Unknown),
            MockProvider::new(Vendor::Intel(IntelGpuType::Unknown), 2)
        );
        let detected_gpus = manager.detect_all_gpus();
        assert_eq!(detected_gpus.len(), 3);
        let nvidia_count = detected_gpus.iter().filter(|g| matches!(g.vendor, Vendor::Nvidia)).count();
        let amd_count = detected_gpus.iter().filter(|g| matches!(g.vendor, Vendor::Amd)).count();
        let intel_count = detected_gpus.iter().filter(|g| matches!(g.vendor, Vendor::Intel(_))).count();
        assert_eq!(nvidia_count, 1);
        assert_eq!(amd_count, 0);
        assert_eq!(intel_count, 2);
        println!("Detection with failures: {} total GPUs (AMD provider failed)", detected_gpus.len());
    }

    /// Test GPU update operations with update count verification
    #[test]
    fn test_gpu_update_operations() {
        let mut manager = GpuProviderManager::new();
        let nvidia_provider = MockProvider::new(Vendor::Nvidia, 1);
        let amd_provider = MockProvider::new(Vendor::Amd, 1);
        let intel_provider = MockProvider::new(Vendor::Intel(IntelGpuType::Integrated), 1);
        let nvidia_update_count = nvidia_provider.update_count.clone();
        let amd_update_count = amd_provider.update_count.clone();
        let intel_update_count = intel_provider.update_count.clone();
        manager.register_provider(Vendor::Nvidia, nvidia_provider);
        manager.register_provider(Vendor::Amd, amd_provider);
        manager.register_provider(
            Vendor::Intel(IntelGpuType::Integrated), 
            intel_provider
        );
        let test_cases = vec![
            (Vendor::Nvidia, "NVIDIA GeForce RTX 4080", nvidia_update_count),
            (Vendor::Amd, "AMD Radeon RX 7800 XT", amd_update_count),
            (Vendor::Intel(IntelGpuType::Integrated), "Intel UHD Graphics", intel_update_count),
        ];
        for (vendor, gpu_name, update_count_ref) in test_cases {
            let initial_count = *update_count_ref.lock().unwrap();
            let mut gpu = GpuInfo::write_vendor(vendor.clone());
            gpu.name_gpu = Some(gpu_name.to_string());
            gpu.temperature = Some(60.0);
            gpu.utilization = Some(40.0);
            let update_result = manager.update_gpu(&mut gpu);
            assert!(update_result.is_ok(), "Failed to update {:?} GPU", vendor);
            assert_eq!(gpu.temperature, Some(61.0));
            assert_eq!(gpu.utilization, Some(45.0));
            let final_count = *update_count_ref.lock().unwrap();
            assert_eq!(final_count, initial_count + 1, 
                      "Update count for {:?} should increase by 1", vendor);
            println!("Successfully updated {:?} GPU: {:?} (update count: {})", 
                    vendor, gpu.name_gpu, final_count);
        }
    }

    /// Test update operations with Intel vendor matching
    #[test]
    fn test_intel_vendor_update_matching() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(
            Vendor::Intel(IntelGpuType::Discrete),
            MockProvider::new(Vendor::Intel(IntelGpuType::Discrete), 1)
        );
        let mut intel_gpu = GpuInfo::write_vendor(Vendor::Intel(IntelGpuType::Integrated));
        intel_gpu.temperature = Some(55.0);
        let update_result = manager.update_gpu(&mut intel_gpu);
        assert!(update_result.is_ok());
        assert_eq!(intel_gpu.temperature, Some(56.0));
        println!("Intel vendor matching works correctly");
    }

    /// Test get_update_count method functionality
    #[test]
    fn test_get_update_count_method() {
        let provider = MockProvider::new(Vendor::Nvidia, 1);
        assert_eq!(provider.get_update_count(), 0);
        println!("Initial update count: {}", provider.get_update_count());
        let mut gpu = GpuInfo::write_vendor(Vendor::Nvidia);
        gpu.temperature = Some(60.0);
        gpu.utilization = Some(40.0);
        for i in 1..=5 {
            let update_result = provider.update_gpu(&mut gpu);
            assert!(update_result.is_ok(), "Update {} should succeed", i);
            let current_count = provider.get_update_count();
            assert_eq!(current_count, i, "Update count should be {} after {} updates", i, i);
            println!("After update {}: count = {}, temp = {:?}, util = {:?}", 
                    i, current_count, gpu.temperature, gpu.utilization);
        }
        
        // Final verification
        assert_eq!(provider.get_update_count(), 5);
        println!("Final update count: {}", provider.get_update_count());
    }

    /// Test update count with failing provider
    #[test]
    fn test_get_update_count_with_failures() {
        let failing_provider = MockProvider::new_failing(Vendor::Amd);
        assert_eq!(failing_provider.get_update_count(), 0);
        let mut gpu = GpuInfo::write_vendor(Vendor::Amd);
        gpu.temperature = Some(70.0);
        for i in 1..=3 {
            let update_result = failing_provider.update_gpu(&mut gpu);
            assert!(update_result.is_err(), "Update {} should fail", i);
            assert_eq!(failing_provider.get_update_count(), 0, 
                      "Update count should remain 0 for failing provider");
        }
        println!("Failing provider update count remains: {}", failing_provider.get_update_count());
    }

    /// Test update with unsupported vendor
    #[test]
    fn test_update_unsupported_vendor() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new(Vendor::Nvidia, 1));
        let mut amd_gpu = GpuInfo::write_vendor(Vendor::Amd);
        let update_result = manager.update_gpu(&mut amd_gpu);
        assert!(update_result.is_err());
        println!("Correctly failed to update unsupported vendor");
    }

    /// Test update with failing provider
    #[test]
    fn test_update_with_failing_provider() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new_failing(Vendor::Nvidia));
        let mut nvidia_gpu = GpuInfo::write_vendor(Vendor::Nvidia);
        let update_result = manager.update_gpu(&mut nvidia_gpu);
        assert!(update_result.is_err());
        println!("Correctly handled failing provider update");
    }

    /// Load test: Multiple concurrent detections
    #[tokio::test]
    async fn test_concurrent_gpu_detections() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new(Vendor::Nvidia, 2));
        manager.register_provider(Vendor::Amd, MockProvider::new(Vendor::Amd, 1));
        let manager = Arc::new(tokio::sync::Mutex::new(manager));
        let mut join_set = tokio::task::JoinSet::new();
        const CONCURRENT_DETECTIONS: usize = 20;
        for i in 0..CONCURRENT_DETECTIONS {
            let manager_clone = manager.clone();
            join_set.spawn(async move {
                let mgr = manager_clone.lock().await;
                let gpus = mgr.detect_all_gpus();
                (i, gpus.len())
            });
        }
        let mut total_detections = 0;
        let mut successful_detections = 0;
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((task_id, gpu_count)) => {
                    successful_detections += 1;
                    total_detections += gpu_count;
                    if task_id % 5 == 0 {
                        println!("Detection #{}: {} GPUs", task_id, gpu_count);
                    }
                    assert_eq!(gpu_count, 3);
                }
                Err(e) => {
                    println!("Detection task failed: {}", e);
                }
            }
        }
        println!("Concurrent detections: {}/{} successful, {} total GPUs", 
                 successful_detections, CONCURRENT_DETECTIONS, total_detections);
        assert_eq!(successful_detections, CONCURRENT_DETECTIONS);
    }

    /// Load test: Rapid update operations with update count verification
    #[test]
    fn test_rapid_update_operations() {
        let mut manager = GpuProviderManager::new();
        let nvidia_provider = MockProvider::new(Vendor::Nvidia, 1);
        let update_count_ref = nvidia_provider.update_count.clone();
        manager.register_provider(Vendor::Nvidia, nvidia_provider);
        const UPDATE_COUNT: usize = 1000;
        let start_time = std::time::Instant::now();
        let mut successful_updates = 0;
        let initial_update_count = *update_count_ref.lock().unwrap();
        for i in 0..UPDATE_COUNT {
            let mut gpu = GpuInfo::write_vendor(Vendor::Nvidia);
            gpu.temperature = Some(50.0);
            gpu.utilization = Some(30.0);
            let update_result = manager.update_gpu(&mut gpu);
            if update_result.is_ok() {
                successful_updates += 1;
                assert_eq!(gpu.temperature, Some(51.0));
                assert_eq!(gpu.utilization, Some(35.0));
            }
            if i % 200 == 0 {
                println!("Completed {} updates", i + 1);
            }
        }
        let final_update_count = *update_count_ref.lock().unwrap();
        let update_time = start_time.elapsed();
        let updates_per_second = UPDATE_COUNT as f64 / update_time.as_secs_f64();
        println!("Rapid update test results:");
        println!("  {} updates in {:?}", successful_updates, update_time);
        println!("  Updates per second: {:.0}", updates_per_second);
        println!("  Provider update count: {} -> {} (delta: {})", 
                initial_update_count, final_update_count, 
                final_update_count - initial_update_count);
        assert_eq!(successful_updates, UPDATE_COUNT);
        assert_eq!(final_update_count - initial_update_count, UPDATE_COUNT, 
                  "Provider should be called exactly {} times", UPDATE_COUNT);
        assert!(updates_per_second > 1000.0, "Updates too slow: {:.0}/sec", updates_per_second);
    }

    /// Stress test: Many providers with many GPUs
    #[test]
    fn test_many_providers_many_gpus() {
        let mut manager = GpuProviderManager::new();
        const PROVIDERS_PER_VENDOR: usize = 10;
        const GPUS_PER_PROVIDER: usize = 5;
        for _i in 0..PROVIDERS_PER_VENDOR {
            let vendor = Vendor::Nvidia;
            manager.register_provider(vendor, MockProvider::new(vendor, GPUS_PER_PROVIDER));
        }
        for _i in 0..PROVIDERS_PER_VENDOR {
            let vendor = Vendor::Amd;
            manager.register_provider(vendor, MockProvider::new(vendor, GPUS_PER_PROVIDER));
        }
        let actual_expected = GPUS_PER_PROVIDER * 2;
        let detection_start = std::time::Instant::now();
        let detected_gpus = manager.detect_all_gpus();
        let detection_time = detection_start.elapsed();
        println!("Many providers stress test:");
        println!("  Detected {} GPUs in {:?}", detected_gpus.len(), detection_time);
        println!("  Expected {} GPUs", actual_expected);
        assert_eq!(detected_gpus.len(), actual_expected);
        assert!(detection_time < std::time::Duration::from_secs(5), 
                "Detection too slow: {:?}", detection_time);
    }

    /// Test provider replacement
    #[test]
    fn test_provider_replacement() {
        let mut manager = GpuProviderManager::new();
        let initial_provider = MockProvider::new(Vendor::Nvidia, 1);
        manager.register_provider(Vendor::Nvidia, initial_provider);
        let initial_gpus = manager.detect_all_gpus();
        assert_eq!(initial_gpus.len(), 1);
        let replacement_provider = MockProvider::new(Vendor::Nvidia, 3);
        manager.register_provider(Vendor::Nvidia, replacement_provider);
        let updated_gpus = manager.detect_all_gpus();
        assert_eq!(updated_gpus.len(), 3);
        println!("Provider replacement successful: 1 -> 3 GPUs");
    }

    /// Performance benchmark for provider operations
    #[test]
    fn test_provider_manager_performance() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new(Vendor::Nvidia, 2));
        manager.register_provider(Vendor::Amd, MockProvider::new(Vendor::Amd, 1));
        manager.register_provider(
            Vendor::Intel(IntelGpuType::Integrated),
            MockProvider::new(Vendor::Intel(IntelGpuType::Integrated), 1)
        );
        const ITERATIONS: usize = 100;
        let detection_start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = manager.detect_all_gpus();
        }
        let detection_time = detection_start.elapsed();
        let update_start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let mut gpu = GpuInfo::write_vendor(Vendor::Nvidia);
            gpu.temperature = Some(60.0);
            let _ = manager.update_gpu(&mut gpu);
        }
        let update_time = update_start.elapsed();
        let vendor_check_start = std::time::Instant::now();
        for _ in 0..ITERATIONS {
            let _ = manager.is_vendor_supported(&Vendor::Nvidia);
            let _ = manager.is_vendor_supported(&Vendor::Amd);
            let _ = manager.is_vendor_supported(&Vendor::Apple);
        }
        let vendor_check_time = vendor_check_start.elapsed();
        println!("Provider manager performance ({} iterations):", ITERATIONS);
        println!("  Detection: {:?} (avg: {:?})", 
                 detection_time, detection_time / ITERATIONS as u32);
        println!("  Updates: {:?} (avg: {:?})", 
                 update_time, update_time / ITERATIONS as u32);
        println!("  Vendor checks: {:?} (avg: {:?})", 
                 vendor_check_time, vendor_check_time / ITERATIONS as u32);
        let avg_detection = detection_time / ITERATIONS as u32;
        let avg_update = update_time / ITERATIONS as u32;
        assert!(avg_detection < std::time::Duration::from_millis(50), 
                "Detection too slow: {:?}", avg_detection);
        assert!(avg_update < std::time::Duration::from_millis(10), 
                "Updates too slow: {:?}", avg_update);
    }

    /// Integration test: Full provider manager workflow
    #[test]
    fn test_full_provider_manager_workflow() {
        println!("Starting full provider manager workflow test");
        let mut manager = GpuProviderManager::new();
        assert!(manager.get_registered_vendors().is_empty());
        println!("Created empty provider manager");
        manager.register_provider(Vendor::Nvidia, MockProvider::new(Vendor::Nvidia, 2));
        manager.register_provider(Vendor::Amd, MockProvider::new(Vendor::Amd, 1));
        manager.register_provider(
            Vendor::Intel(IntelGpuType::Discrete),
            MockProvider::new(Vendor::Intel(IntelGpuType::Discrete), 1)
        );
        println!("Registered 3 providers");
        let vendors = manager.get_registered_vendors();
        assert_eq!(vendors.len(), 3);
        assert!(manager.is_vendor_supported(&Vendor::Nvidia));
        assert!(manager.is_vendor_supported(&Vendor::Amd));
        println!("Verified provider registrations");
        let detected_gpus = manager.detect_all_gpus();
        assert_eq!(detected_gpus.len(), 4);
        println!("Detected {} GPUs", detected_gpus.len());
        let mut update_results = Vec::new();
        for gpu in detected_gpus {
            let mut gpu_copy = gpu.clone();
            let original_temp = gpu_copy.temperature;
            let update_result = manager.update_gpu(&mut gpu_copy);
            update_results.push((gpu.vendor.clone(), update_result.is_ok()));
            if update_result.is_ok() {
                if let (Some(orig), Some(new)) = (original_temp, gpu_copy.temperature) {
                    assert!(new > orig, "Temperature should increase after update");
                }
            }
        }
        println!("Completed {} update operations", update_results.len());
        let successful_updates = update_results.iter().filter(|(_, success)| *success).count();
        assert_eq!(successful_updates, 4);
        println!("All {} updates successful", successful_updates);
        let mut unknown_gpu = GpuInfo::write_vendor(Vendor::Unknown);
        let unknown_update = manager.update_gpu(&mut unknown_gpu);
        assert!(unknown_update.is_err());
        println!("Unknown vendor update correctly failed");
        println!("Full provider manager workflow test completed successfully");
    }
}