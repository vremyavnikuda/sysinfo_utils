//! Benchmark for comparing cache performance
use gpu_info::gpu_manager::GpuManager;
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Benchmarking GPU information caching performance");
    let manager = GpuManager::new();
    let gpu_count = manager.gpu_count();
    println!("Found {} GPU(s)", gpu_count);
    if gpu_count == 0 {
        println!("No GPUs found, exiting benchmark");
        return Ok(());
    }
    println!("Populating cache...");
    for i in 0..gpu_count {
        let _ = manager.get_gpu_cached(i);
    }
    println!("Benchmark without cache");
    let start = Instant::now();
    for _ in 0..1000 {
        for i in 0..gpu_count {
            if let Some(gpu) = manager.get_gpu_by_index(i) {
                let _ = gpu.temperature;
                let _ = gpu.utilization;
                let _ = gpu.power_usage;
            }
        }
    }
    let without_cache_time = start.elapsed();
    println!("Time without cache: {:?}", without_cache_time);
    println!("Benchmark with cache");
    let start = Instant::now();
    for _ in 0..1000 {
        for i in 0..gpu_count {
            if let Some(gpu) = manager.get_gpu_cached(i) {
                let _ = gpu.temperature;
                let _ = gpu.utilization;
                let _ = gpu.power_usage;
            }
        }
    }
    let with_cache_time = start.elapsed();
    println!("Time with cache: {:?}", with_cache_time);
    if without_cache_time > Duration::from_nanos(0) && with_cache_time < without_cache_time {
        let improvement =
            (without_cache_time.as_nanos() as f64 / with_cache_time.as_nanos() as f64) - 1.0;
        println!("Performance improvement: {:.2}x faster", improvement);
    } else if with_cache_time > without_cache_time {
        let slowdown =
            (with_cache_time.as_nanos() as f64 / without_cache_time.as_nanos() as f64) - 1.0;
        println!("Performance slowdown: {:.2}x slower", slowdown);
    } else {
        println!("Performance is similar");
    }
    println!("Cache Statistics");
    if let Some(stats) = manager.get_cache_stats() {
        println!("Total cache entries: {}", stats.total_entries);
        println!("Total cache accesses: {}", stats.total_accesses);
        println!("Oldest entry age: {:?}", stats.oldest_entry_age);
    }
    Ok(())
}
