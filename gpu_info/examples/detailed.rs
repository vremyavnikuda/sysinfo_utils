use gpu_info::{ vendor::Vendor, Result };
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
        #[cfg(target_os = "windows")]
        {
            use gpu_info::providers::{ amd, intel, nvidia };
            match gpu.vendor {
                Vendor::Nvidia => {
                    if let Err(e) = nvidia::update_nvidia_info(&mut gpu) {
                        println!("Error updating NVIDIA GPU: {}", e);
                    }
                }
                Vendor::Amd => {
                    if let Err(e) = amd::update_amd_info(&mut gpu) {
                        println!("Error updating AMD GPU: {}", e);
                    }
                }
                Vendor::Intel(_) => {
                    if let Err(e) = intel::update_intel_info(&mut gpu) {
                        println!("Error updating Intel GPU: {}", e);
                    }
                }
                _ => {}
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            match gpu.vendor {
                Vendor::Nvidia => {
                    #[cfg(any(target_os = "linux", target_os = "macos"))]
                    {
                        use gpu_info::providers::nvidia::update_nvidia_info;

                        if let Err(e) = update_nvidia_info(&mut gpu) {
                            println!("Error updating NVIDIA GPU: {}", e);
                        }
                    }
                }
                _ => {}
            }
        }
        println!("Measurement #{}", i + 1);
        println!("Temperature: {}°C", gpu.format_temperature());
        println!("Utilization: {}%", gpu.format_utilization());
        println!("Core Clock: {} MHz", gpu.format_core_clock());
        thread::sleep(Duration::from_secs(1));
    }
    if gpu.is_valid() {
        println!("GPU data is okay");
    } else {
        println!("GPU data has problems");
    }
    Ok(())
}
