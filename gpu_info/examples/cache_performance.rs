//! Cache performance comparison: Arc vs Clone
//!
//! This benchmark demonstrates the real benefit of Arc<GpuInfo>:
//! eliminating clones when accessing cached data multiple times.

use gpu_info::GpuManager;
use std::time::Instant;

fn main() {
    println!("Cache Performance: Arc vs Clone\n");

    let manager = GpuManager::new();
    if manager.gpu_count() == 0 {
        println!("No GPUs found");
        return;
    }
    const ITERATIONS: usize = 100_000;
    println!("Testing Arc-based cache access...");
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        if let Some(gpu) = manager.get_gpu_cached(0) {
            let _ = gpu.format_name_gpu();
            let _ = gpu.format_temperature();
        }
    }
    let arc_duration = start.elapsed();
    println!("Arc-based: {} iterations in {:?}", ITERATIONS, arc_duration);
    println!("Average: {:?}\n", arc_duration / ITERATIONS as u32);
    println!("Testing Clone-based cache access...");
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        if let Some(gpu) = manager.get_gpu_cached_owned(0) {
            let _ = gpu.format_name_gpu();
            let _ = gpu.format_temperature();
        }
    }
    let clone_duration = start.elapsed();
    println!(
        "Clone-based: {} iterations in {:?}",
        ITERATIONS, clone_duration
    );
    println!("Average: {:?}\n", clone_duration / ITERATIONS as u32);
    let improvement = if arc_duration.as_nanos() > 0 {
        (clone_duration.as_nanos() as f64 / arc_duration.as_nanos() as f64 - 1.0) * 100.0
    } else {
        0.0
    };
    println!("Results:");
    println!("Arc-based: {:?}", arc_duration);
    println!("Clone-based: {:?}", clone_duration);
    if improvement > 0.0 {
        println!("Improvement: {:.1}% faster with Arc", improvement);
    } else if improvement < 0.0 {
        println!("Warning: Arc is {:.1}% slower (unexpected)", -improvement);
    }
    let gpu_size = std::mem::size_of::<gpu_info::GpuInfo>();
    let arc_overhead = std::mem::size_of::<std::sync::Arc<gpu_info::GpuInfo>>();
    println!("\nMemory analysis:");
    println!("GpuInfo size: {} bytes", gpu_size);
    println!("Arc<GpuInfo> size: {} bytes", arc_overhead);
    println!(
        "Saved per access: {} bytes",
        gpu_size.saturating_sub(arc_overhead)
    );
    let total_clones_avoided = ITERATIONS;
    let total_memory_saved = gpu_size.saturating_sub(arc_overhead) * total_clones_avoided;
    println!("Total clones avoided: {}", total_clones_avoided);
    println!(
        "Total memory saved: {} bytes ({:.1} KB)",
        total_memory_saved,
        total_memory_saved as f64 / 1024.0
    );
}
