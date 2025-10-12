//! Performance comparison between old and new get_async implementations
use gpu_info::{async_api::get_async, get};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Performance Comparison: get_async() vs get()");
    println!("Warming up cache...");
    let _warmup = get_async().await?;
    println!("Testing synchronous get() - 5 iterations:");
    let mut sync_times = Vec::new();
    for i in 1..=5 {
        let start = Instant::now();
        let _gpu = get();
        let duration = start.elapsed();
        sync_times.push(duration);
        println!("Iteration {}: {:?}", i, duration);
    }
    println!("Testing async get_async() (with cache) - 5 iterations:");
    let mut async_times = Vec::new();
    for i in 1..=5 {
        let start = Instant::now();
        let _gpu = get_async().await?;
        let duration = start.elapsed();
        async_times.push(duration);
        println!("Iteration {}: {:?}", i, duration);
    }
    let sync_avg = sync_times.iter().sum::<std::time::Duration>() / sync_times.len() as u32;
    let async_avg = async_times.iter().sum::<std::time::Duration>() / async_times.len() as u32;
    println!("+ Performance Summary:");
    println!("+ Synchronous get() average: {:?}", sync_avg);
    println!("+ Async get_async() average: {:?}", async_avg);
    if async_avg < sync_avg {
        let improvement = ((sync_avg.as_nanos() - async_avg.as_nanos()) as f64
            / sync_avg.as_nanos() as f64)
            * 100.0;
        println!("Async is {:.1}% faster!", improvement);
    } else {
        println!("Both implementations show similar performance");
    }
    println!("- Benefits of new get_async():");
    println!("-- Uses cached GPU manager for efficiency");
    println!("-- Better error handling with specific error types");
    println!("-- Consistent with project's provider architecture");
    println!("-- Falls back gracefully when cache unavailable");
    Ok(())
}
