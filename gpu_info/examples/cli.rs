//gpu_info/examples/cli.rs
use gpu_info::GpuManager;

/// Prints out information about the first detected GPU, if any.
///
/// The following information is printed out:
/// - The vendor of the GPU
/// - The name of the GPU
/// - The current temperature of the GPU
/// - The current utilization of the GPU
/// - The current power usage of the GPU
/// - The current clock speed of the GPU
///
/// If no GPUs are detected, a message is printed to the console.
fn main() {
    let mut manager = GpuManager::new();
    manager.refresh();

    if let Some(gpu) = manager.gpus.first() {
        println!("{:?}", gpu.vendor_gpu());
        println!("{}", gpu.name_gpu());
        println!("{}", gpu.format_get_temperature_gpu());
        println!("{}", gpu.format_get_utilization_gpu());
        println!("{}", gpu.format_get_power_usage_gpu());
        println!("{}", gpu.format_get_clock_speed_gpu());
        println!("{}", gpu.utilization_gpu());
        println!("{:?}", gpu.clock_speed_gpu());
        println!("{}",gpu.format_is_active_gpu());
        println!("{}",gpu.format_power_usage_gpu())
    } else {
        println!("No GPUs detected.");
    }
}
