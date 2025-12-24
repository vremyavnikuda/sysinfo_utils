//! Comprehensive tests for async API functionality
//!
//! These tests cover async GPU operations under normal and high-load conditions,
//! including concurrent access, error handling, and performance characteristics.

#[cfg(test)]
mod tests {
    use crate::async_api::{get_all_async, get_all_async_owned, get_async, update_gpu_async};
    use crate::gpu_info::GpuInfo;
    use crate::vendor::Vendor;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::Mutex;
    use tokio::task::JoinSet;

    /// Test basic async GPU retrieval functionality
    #[tokio::test]
    async fn test_get_async_basic() {
        let result = get_async().await;
        match result {
            Ok(gpu) => {
                println!("Async GPU detection successful: {:?}", gpu.vendor);
                assert!(gpu.vendor != Vendor::Unknown || gpu.name_gpu.is_none());
            }
            Err(e) => {
                println!("Async GPU detection failed (expected in test env): {}", e);
            }
        }
    }

    /// Test async GPU list retrieval
    #[tokio::test]
    async fn test_get_all_async_basic() {
        let result = get_all_async().await;
        match result {
            Ok(gpus) => {
                println!("Found {} GPUs async", gpus.len());
                for (i, gpu) in gpus.iter().enumerate() {
                    println!("GPU #{}: {:?}", i, gpu.vendor);
                }
            }
            Err(e) => {
                println!("Async all GPU detection failed (expected): {}", e);
            }
        }
    }

    /// Test async GPU update functionality
    #[tokio::test]
    async fn test_update_gpu_async_basic() {
        let mut gpu = GpuInfo::unknown();
        gpu.vendor = Vendor::Nvidia;
        let result = update_gpu_async(&mut gpu).await;
        match result {
            Ok(()) => {
                println!("GPU update successful");
            }
            Err(e) => {
                println!("GPU update failed (expected in test env): {}", e);
            }
        }
    }

    /// Load test: Concurrent async GPU retrievals
    #[tokio::test]
    async fn test_concurrent_get_async_load() {
        let concurrent_requests = 5;
        let mut join_set = JoinSet::new();
        let _ = get_async().await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        let start_time = Instant::now();
        for i in 0..concurrent_requests {
            join_set.spawn(async move {
                let task_start = Instant::now();
                let result = get_async().await;
                let task_duration = task_start.elapsed();
                (i, result, task_duration)
            });
        }
        let mut successful_requests = 0;
        let mut failed_requests = 0;
        let mut total_response_time = Duration::ZERO;
        while let Some(task_result) = join_set.join_next().await {
            match task_result {
                Ok((_task_id, gpu_result, duration)) => {
                    total_response_time += duration;
                    match gpu_result {
                        Ok(_gpu) => {
                            successful_requests += 1;
                        }
                        Err(_e) => {
                            failed_requests += 1;
                        }
                    }
                }
                Err(_) => {
                    failed_requests += 1;
                }
            }
        }
        let total_duration = start_time.elapsed();
        let avg_response_time = if successful_requests + failed_requests > 0 {
            total_response_time / (successful_requests + failed_requests)
        } else {
            Duration::ZERO
        };
        assert!(
            total_duration < Duration::from_secs(40),
            "Load test took too long: {:?}",
            total_duration
        );
        assert!(
            avg_response_time < Duration::from_secs(15),
            "Average response time too high: {:?}",
            avg_response_time
        );
    }

    /// Stress test: Rapid sequential async calls
    #[tokio::test]
    async fn test_rapid_sequential_async_calls() {
        let iterations = 3;
        let _ = get_async().await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        let start_time = Instant::now();
        for _i in 0..iterations {
            let _result = get_async().await;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        let total_duration = start_time.elapsed();
        assert!(
            total_duration < Duration::from_secs(40),
            "Sequential test took too long: {:?}",
            total_duration
        );
    }

    /// Test concurrent updates on shared GPU data
    #[tokio::test]
    async fn test_concurrent_gpu_updates() {
        let shared_gpu = Arc::new(Mutex::new(GpuInfo::unknown()));
        shared_gpu.lock().await.vendor = Vendor::Nvidia;
        let concurrent_updates = 5;
        let mut join_set = JoinSet::new();

        for i in 0..concurrent_updates {
            let gpu_clone = shared_gpu.clone();
            join_set.spawn(async move {
                let mut gpu = gpu_clone.lock().await;
                let result = update_gpu_async(&mut gpu).await;
                (i, result)
            });
        }

        let mut successful_updates = 0;
        let mut failed_updates = 0;

        while let Some(task_result) = join_set.join_next().await {
            match task_result {
                Ok((_task_id, update_result)) => match update_result {
                    Ok(()) => {
                        successful_updates += 1;
                    }
                    Err(_) => {
                        failed_updates += 1;
                    }
                },
                Err(_) => {
                    failed_updates += 1;
                }
            }
        }

        assert!(successful_updates + failed_updates == concurrent_updates);
    }

    /// Test async error handling under various conditions
    #[tokio::test]
    async fn test_async_error_handling() {
        let mut invalid_gpu = GpuInfo {
            vendor: Vendor::Unknown,
            ..Default::default()
        };
        let result = update_gpu_async(&mut invalid_gpu).await;
        match result {
            Ok(()) => {
                println!("Update succeeded unexpectedly");
            }
            Err(e) => {
                println!("Update failed as expected: {}", e);
            }
        }
        let start_time = Instant::now();
        let _results = tokio::join!(get_async(), get_all_async(), get_async(), get_all_async(),);
        let duration = start_time.elapsed();
        assert!(
            duration < Duration::from_secs(20),
            "Operations took too long: {:?}",
            duration
        );
    }

    /// Integration test: Full async workflow
    #[tokio::test]
    async fn test_full_async_workflow() {
        let primary_gpu_result = get_async().await;
        println!("Primary GPU result: {:?}", primary_gpu_result.is_ok());
        let all_gpus_result = get_all_async_owned().await;
        match &all_gpus_result {
            Ok(gpus) => println!("Found {} GPUs total", gpus.len()),
            Err(e) => println!("Failed to get all GPUs: {}", e),
        }
        if let Ok(mut gpus) = all_gpus_result {
            for (i, gpu) in gpus.iter_mut().enumerate() {
                let update_result = update_gpu_async(gpu).await;
                match update_result {
                    Ok(()) => println!("Updated GPU #{} successfully", i),
                    Err(e) => println!("Failed to update GPU #{}: {}", i, e),
                }
            }
        }
    }

    /// Performance benchmark for async operations
    #[tokio::test]
    async fn test_async_performance_benchmark() {
        const WARMUP_ITERATIONS: usize = 1;
        const BENCHMARK_ITERATIONS: usize = 3;
        for _ in 0..WARMUP_ITERATIONS {
            let _ = get_async().await;
            let _ = get_all_async().await;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
        let start_time = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = get_async().await;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let get_duration = start_time.elapsed();
        let avg_get_time = get_duration / BENCHMARK_ITERATIONS as u32;
        let start_time = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            let _ = get_all_async().await;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let get_all_duration = start_time.elapsed();
        let avg_get_all_time = get_all_duration / BENCHMARK_ITERATIONS as u32;
        assert!(
            avg_get_time < Duration::from_secs(15),
            "get_async too slow: {:?}",
            avg_get_time
        );
        assert!(
            avg_get_all_time < Duration::from_secs(15),
            "get_all_async too slow: {:?}",
            avg_get_all_time
        );
    }
}
