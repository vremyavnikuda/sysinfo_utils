//! Example of using the enhanced caching features for GPU information
use gpu_info::gpu_manager::GpuManager;
use std::time::Duration;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Demonstration of enhanced GPU information caching");
    let manager = GpuManager::with_cache_config(Duration::from_secs(2), 5);
    println!("\n1. Getting all GPUs:");
    let gpus = manager.get_all_gpus_owned();
    println!("Found {} GPU(s)", gpus.len());
    for (i, gpu) in gpus.iter().enumerate() {
        println!("  GPU {}: {}", i, gpu.format_name_gpu());
    }
    println!("\n2. Accessing GPUs multiple times (should use cache):");
    if let Some(gpu) = manager.get_gpu_by_index_owned(0) {
        manager.get_gpu_cached(0);
        println!("  Populated cache with GPU 0: {}", gpu.format_name_gpu());
    }
    for i in 0..3 {
        if let Some(gpu) = manager.get_gpu_cached(0) {
            println!(
                "  Access {}: {} (temp: {}°C)",
                i + 1,
                gpu.format_name_gpu(),
                gpu.format_temperature()
            );
        }
    }
    println!("\n3. Cache statistics:");
    if let Some(stats) = manager.get_cache_stats() {
        println!("  Total entries: {}", stats.total_entries);
        println!("  Total accesses: {}", stats.total_accesses);
        println!("  Oldest entry age: {:?}", stats.oldest_entry_age);
    }
    println!("\n4. Waiting for cache expiration...");
    std::thread::sleep(Duration::from_secs(3));
    println!("\n5. Accessing after expiration (should refresh cache):");
    if let Some(gpu) = manager.get_gpu_cached(0) {
        println!(
            "  GPU: {} (temp: {}°C)",
            gpu.format_name_gpu(),
            gpu.format_temperature()
        );
    }
    println!("\n6. Cache statistics after expiration:");
    if let Some(stats) = manager.get_cache_stats() {
        println!("  Total entries: {}", stats.total_entries);
        println!("  Total accesses: {}", stats.total_accesses);
        println!("  Oldest entry age: {:?}", stats.oldest_entry_age);
    }
    Ok(())
}
