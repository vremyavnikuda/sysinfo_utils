//! Example demonstrating the corrected async GPU update functionality
use gpu_info::{async_api::{get_async, update_gpu_async}, vendor::Vendor};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Async GPU Update Demo");
    let mut gpu = get_async().await?;
    println!("Initial GPU Info:");
    println!("-- Vendor: {}", gpu.vendor);
    println!("-- Name: {}", gpu.format_name_gpu());
    println!("-- Temperature: {}°C", gpu.format_temperature());
    println!("-- Utilization: {}%", gpu.format_utilization());
    println!("-- Power Usage: {}W", gpu.format_power_usage());
    println!("Performing async updates...");
    for i in 1..=3 {
        println!("   Update #{}", i);
        let start = std::time::Instant::now();
        match update_gpu_async(&mut gpu).await {
            Ok(()) => {
                let duration = start.elapsed();
                println!("Success in {:?}", duration);
                println!("-- Temperature: {}°C", gpu.format_temperature());
                println!("-- Utilization: {}%", gpu.format_utilization());
                println!("-- Power Usage: {}W", gpu.format_power_usage());
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        // Small delay between updates  
        std::thread::sleep(Duration::from_millis(500));
    }
    println!("Vendor-specific update behavior:");
    match gpu.vendor {
        Vendor::Nvidia => println!("-- Using NVIDIA NVML API for updates"),
        Vendor::Amd => println!("-- Using AMD ADL API for updates"),
        Vendor::Intel(_) => println!("-- Using Intel WMI API for updates"),
        Vendor::Apple => println!("-- Using Apple Metal API for updates"),
        Vendor::Unknown => println!("-- Using fallback update method"),
        _ => println!("-- Using generic update method"),
    }
    
    println!("Demo completed successfully!");
    Ok(())
}