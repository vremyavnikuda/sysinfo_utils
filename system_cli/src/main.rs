//! system_info_lib/system_cl/src/main.rs
use clap::Parser;
#[derive(Parser)]
#[clap(about, version)]
struct Options {
    /// Show all OS information.
    #[clap(long)]
    all: bool,
    /// Show OS type.
    #[clap(short = 't', long = "type")]
    system_type: bool,
    /// Show OS version.
    #[clap(short = 'v', long)]
    system_version: bool,
    /// Show OS bit_depth.
    #[clap(short, long)]
    bit_depth: bool,
    /// Show OS architecture.
    #[clap(short = 'A', long = "Arch")]
    architecture: bool,
}
#[derive(Parser)]
#[clap(about, version)]
struct GpuOptions {
    /// Show all GPU information.
    #[clap(long)]
    all: bool,
    /// Show GPU vendor.
    #[clap(short = 'n', long = "name_gpu")]
    vendor: bool,
    /// Show GPU temperature.
    #[clap(short = 't', long = "temperature")]
    temperature: bool,
    /// Show GPU utilization.
    #[clap(short = 'u', long = "utilization")]
    utilization: bool,
    /// Show GPU power usage.
    #[clap(short = 'p', long = "power")]
    power_usage: bool,
    /// Show GPU clock speed.
    #[clap(short = 'c', long = "clock")]
    clock_speed: bool,
}

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
    let info = system_info::get();

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
