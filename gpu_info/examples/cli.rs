use gpu_info::gpu_info::Formattable;

/// The main function demonstrating the usage of the `gpu_info` library.
///
/// This function retrieves GPU information using the `gpu_info` library
/// and prints various details about the GPU to the console.
fn main() {
    // Create a GPU manager instance
    let gpu = gpu_info::get();

    // Print the GPU vendor
    println!("Vendor: {:?}", gpu.vendor());
    // Print the GPU name
    println!("Name: {:?}", gpu.name_gpu());
    // Print the GPU utilization level
    println!("Utilization: {:?}", gpu.utilization());
    // Print the GPU temperature
    println!("Temperature: {}", gpu.temperature().fmt_string());
    // Print the GPU core clock speed
    println!("Clock Speed: {:?}", gpu.core_clock());
    // Print the GPU power usage
    println!("Power Usage: {:?}", gpu.power_usage());
    // Print the GPU memory usage
    println!("Memory Usage: {:?}", gpu.memory_util());
    // Print the total GPU memory
    println!("Memory Total: {:?}", gpu.memory_total());
    // Print the GPU activity status
    println!("Is active: {:?}", gpu.active());
    // Print the formatted GPU name
    println!("Formated Name Gpu: {}", gpu.format_name_gpu());
}
