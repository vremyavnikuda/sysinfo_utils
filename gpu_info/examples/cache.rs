use gpu_info::gpu_info::GpuInfoCache;
use std::thread;
use std::time::Duration;

fn main() {
    let cache = GpuInfoCache::new(Duration::from_secs(2));

    println!("=== Demonstration of GPU information caching ===");

    println!("\nFirst data retrieval:");
    if let Some(gpu) = cache.get() {
        println!("Temperature: {}°C", gpu.format_temperature());
        println!("Utilization: {}%", gpu.format_utilization());
    } else {
        println!("Cache is empty, getting new data...");
        let gpu = gpu_info::get();
        cache.set(gpu);
    }

    println!("\nSecond data retrieval (should be from cache):");
    if let Some(gpu) = cache.get() {
        println!("Temperature: {}°C", gpu.format_temperature());
        println!("Utilization: {}%", gpu.format_utilization());
    }

    println!("\nWaiting 3 seconds...");
    thread::sleep(Duration::from_secs(3));

    println!("\nThird data retrieval (cache should be empty):");
    if let Some(gpu) = cache.get() {
        println!("Temperature: {}°C", gpu.format_temperature());
        println!("Utilization: {}%", gpu.format_utilization());
    } else {
        println!("Cache expired, getting new data...");
        let gpu = gpu_info::get();
        cache.set(gpu);
    }

    println!("\nCache update:");
    let gpu = gpu_info::get();
    cache.set(gpu);
    println!("Cache updated");

    println!("\nChecking updated data:");
    if let Some(gpu) = cache.get() {
        println!("Temperature: {}°C", gpu.format_temperature());
        println!("Utilization: {}%", gpu.format_utilization());
    }
}
