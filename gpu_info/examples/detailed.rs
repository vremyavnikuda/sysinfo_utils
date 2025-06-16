use gpu_info::{vendor::Vendor, Result};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let mut gpu = gpu_info::get();

    println!("Vendor: {}", gpu.vendor);
    println!("Name GPU: {}", gpu.format_name_gpu());
    println!("Driver: {}", gpu.format_driver_version());

    println!("Temperature: {}°C", gpu.format_temperature());
    println!("Utilization: {}%", gpu.format_utilization());
    println!("Core Clock: {} MHz", gpu.format_core_clock());
    println!("Memory Clock: {} MHz", gpu.format_memory_clock());
    println!("Max Clock Speed: {} MHz", gpu.format_max_clock_speed());

    println!("Memory Usage: {}%", gpu.format_memory_util());
    println!("Total Memory: {} GB", gpu.format_memory_total());

    println!("Current Usage: {} W", gpu.format_power_usage());
    println!("Power Limit: {} W", gpu.format_power_limit());

    println!("Active: {}", gpu.format_active());

    for i in 0..5 {
        match gpu.vendor {
            Vendor::Nvidia => {
                if let Err(e) = gpu_info::windows::nvidia::update_nvidia_info(&mut gpu) {
                    println!("Error updating NVIDIA GPU: {}", e);
                }
            }
            Vendor::Amd => {
                if let Err(e) = gpu_info::windows::amd::update_amd_info(&mut gpu) {
                    println!("Error updating AMD GPU: {}", e);
                }
            }
            Vendor::Intel(_) => {
                if let Err(e) = gpu_info::windows::intel::update_intel_info(&mut gpu) {
                    println!("Error updating Intel GPU: {}", e);
                }
            }
            _ => {}
        }

        println!("\nMeasurement #{}", i + 1);
        println!("Temperature: {}°C", gpu.format_temperature());
        println!("Utilization: {}%", gpu.format_utilization());
        println!("Core Clock: {} MHz", gpu.format_core_clock());
        thread::sleep(Duration::from_secs(1));
    }

    if gpu.is_valid() {
        println!("GPU data is valid");
    } else {
        println!("GPU data is invalid");
    }

    Ok(())
}
