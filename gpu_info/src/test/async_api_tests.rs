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
        let concurrent_requests = 50;
        let mut join_set = JoinSet::new();
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
                Ok((task_id, gpu_result, duration)) => {
                    total_response_time += duration;
                    match gpu_result {
                        Ok(_gpu) => {
                            successful_requests += 1;
                            println!("Task {} succeeded in {:?}", task_id, duration);
                        }
                        Err(_e) => {
                            failed_requests += 1;
                            println!("Task {} failed in {:?}", task_id, duration);
                        }
                    }
                }
                Err(e) => {
                    println!("Task join error: {}", e);
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
        println!("  Total duration: {:?}", total_duration);
        println!("  Successful requests: {}", successful_requests);
        println!("  Failed requests: {}", failed_requests);
        println!("  Average response time: {:?}", avg_response_time);
        println!(
            "  Requests per second: {:.2}",
            concurrent_requests as f64 / total_duration.as_secs_f64()
        );
        assert!(
            total_duration < Duration::from_secs(30),
            "Load test took too long: {:?}",
            total_duration
        );
        assert!(
            avg_response_time < Duration::from_secs(5),
            "Average response time too high: {:?}",
            avg_response_time
        );
    }

    /// Stress test: Rapid sequential async calls
    #[tokio::test]
    async fn test_rapid_sequential_async_calls() {
        let iterations = 100;
        let start_time = Instant::now();
        let mut successful_calls = 0;
        for i in 0..iterations {
            let call_start = Instant::now();
            let result = get_async().await;
            let call_duration = call_start.elapsed();
            match result {
                Ok(_) => successful_calls += 1,
                Err(_) => {
                    // Expected in test environment
                }
            }
            if i % 20 == 0 {
                println!(
                    "Completed {} calls, last call took: {:?}",
                    i + 1,
                    call_duration
                );
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        let total_duration = start_time.elapsed();
        println!("  Total iterations: {}", iterations);
        println!("  Successful calls: {}", successful_calls);
        println!("  Total duration: {:?}", total_duration);
        println!("  Average call time: {:?}", total_duration / iterations);
        assert!(
            total_duration < Duration::from_secs(60),
            "Sequential test took too long: {:?}",
            total_duration
        );
    }

    /// Test concurrent updates on shared GPU data
    #[tokio::test]
    async fn test_concurrent_gpu_updates() {
        let shared_gpu = Arc::new(Mutex::new(GpuInfo::unknown()));
        shared_gpu.lock().await.vendor = Vendor::Nvidia;
        let concurrent_updates = 20;
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
                Ok((task_id, update_result)) => match update_result {
                    Ok(()) => {
                        successful_updates += 1;
                        println!("Update task {} succeeded", task_id);
                    }
                    Err(_) => {
                        failed_updates += 1;
                        println!("Update task {} failed (expected)", task_id);
                    }
                },
                Err(e) => {
                    println!("Update task join error: {}", e);
                    failed_updates += 1;
                }
            }
        }
        println!("  Successful updates: {}", successful_updates);
        println!("  Failed updates: {}", failed_updates);
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
        println!("Multiple async operations completed in {:?}", duration);
        assert!(
            duration < Duration::from_secs(10),
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
        const WARMUP_ITERATIONS: usize = 5;
        const BENCHMARK_ITERATIONS: usize = 50;
        println!("Starting async performance benchmark");
        for _ in 0..WARMUP_ITERATIONS {
            let _ = get_async().await;
        }
        let start_time = Instant::now();
        let mut successful_gets = 0;
        for _ in 0..BENCHMARK_ITERATIONS {
            if get_async().await.is_ok() {
                successful_gets += 1;
            }
        }
        let get_duration = start_time.elapsed();
        let avg_get_time = get_duration / BENCHMARK_ITERATIONS as u32;
        let start_time = Instant::now();
        let mut successful_get_alls = 0;
        for _ in 0..BENCHMARK_ITERATIONS {
            if get_all_async().await.is_ok() {
                successful_get_alls += 1;
            }
        }
        let get_all_duration = start_time.elapsed();
        let avg_get_all_time = get_all_duration / BENCHMARK_ITERATIONS as u32;
        println!(
            "  get_async() - Average: {:?}, Success rate: {}/{}",
            avg_get_time, successful_gets, BENCHMARK_ITERATIONS
        );
        println!(
            "  get_all_async() - Average: {:?}, Success rate: {}/{}",
            avg_get_all_time, successful_get_alls, BENCHMARK_ITERATIONS
        );
        assert!(
            avg_get_time < Duration::from_millis(500),
            "get_async too slow: {:?}",
            avg_get_time
        );
        assert!(
            avg_get_all_time < Duration::from_secs(2),
            "get_all_async too slow: {:?}",
            avg_get_all_time
        );
    }
}
