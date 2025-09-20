//! Example of using the async API for GPU information retrieval
use gpu_info::{get_all_async, get_async};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Getting primary GPU info...");
    match get_async().await {
        Ok(gpu) => {
            println!("Primary GPU: {}", gpu.format_name_gpu());
            println!("Temperature: {}°C", gpu.format_temperature());
            println!("Utilization: {}%", gpu.format_utilization());
        }
        Err(e) => println!("Error getting primary GPU: {}", e),
    }
    println!("\nGetting all GPUs info...");
    match get_all_async().await {
        Ok(gpus) => {
            println!("Found {} GPU(s)", gpus.len());
            for (i, gpu) in gpus.iter().enumerate() {
                println!("GPU {}: {}", i, gpu.format_name_gpu());
                println!("  Temperature: {}°C", gpu.format_temperature());
                println!("  Utilization: {}%", gpu.format_utilization());
                println!("  Power Usage: {}W", gpu.format_power_usage());
            }
        }
        Err(e) => println!("Error getting all GPUs: {}", e),
    }
    Ok(())
}
