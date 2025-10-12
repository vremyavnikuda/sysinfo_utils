//! system_info_lib/system_cl/src/main.rs
use clap::Parser;
use system_cli::{GpuOptions, Options};

/// Initializes the logging framework, parses command-line options, and retrieves system information.
///
/// This function performs the following tasks:
/// - Initializes the logger using `env_logger::init()`.
/// - Parses the command-line arguments using `clap` to determine which pieces of system information
///   to display.
/// - Retrieves the system information using the `system_info_lib::get()` function.
/// - Displays the specified system information, including type, version, bit depth, and architecture.
///   If the `--all` option is specified, all available information is displayed, overriding other options.
/// - Logs a warning if `--all` supersedes other options.
fn main() {
    env_logger::init();
    // system_info_lib
    let _options = Options::parse();
    let info = system_info_lib::get();

    // gpu_info
    let _gpu_options = GpuOptions::parse();
    let gpu = gpu_info::get();

    println!("Vendor: {:?}", gpu.vendor());
    println!("Name: {:?}", gpu.name_gpu());
    println!("Utilization: {:?}", gpu.utilization());
    println!("Temperature: {:?}", gpu.temperature());
    println!("Clock Speed: {:?}", gpu.core_clock());
    println!("Power Usage: {:?}", gpu.power_usage());
    println!("Memory Usage: {:?}", gpu.memory_util());
    println!("Memory Total: {:?}", gpu.memory_total());
    println!("Is active: {:?}", gpu.active());
    println!("Type: {}", info.system_type());
    println!("Version: {}", info.version());
    println!("Edition: {:?}", info.edition());
    println!("Codename: {:?}", info.codename());
    println!("BitDepth: {}", info.bit_depth());
    println!("Architecture: {:?}", info.architecture());
}
