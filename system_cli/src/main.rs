//! system_info_lib/system_cl/src/main.rs
use clap::Parser;
use log::warn;

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
    #[clap(short = 'n', long = "name")]
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
    let options = Options::parse();
    let info = system_info::get();

    // gpu_info
    let gpu_options = GpuOptions::parse();
    let mut manager = gpu_info::GpuManager::new();
    manager.refresh();

    if let Some(gpu_info) = manager.gpus.first() {
        if gpu_options.all
            || !(gpu_options.vendor
                || gpu_options.temperature
                || gpu_options.utilization
                || gpu_options.power_usage
                || gpu_options.clock_speed)
        {
            if gpu_options.vendor
                || gpu_options.temperature
                || gpu_options.utilization
                || gpu_options.power_usage
                || gpu_options.clock_speed
            {
                warn!("--all supersedes all other options");
            }

            println!(
                "GPU information:\n{}\n{}\n{}\n{}\n{}",
                gpu_info.name_gpu(),
                gpu_info.format_get_temperature_gpu(),
                gpu_info.format_get_utilization_gpu(),
                gpu_info.format_get_power_usage_gpu(),
                gpu_info.format_get_clock_speed_gpu()
            );
        } else {
            if gpu_options.vendor {
                println!("GPU Vendor: {:?}", gpu_info.vendor_gpu());
            }
            if gpu_options.temperature {
                println!("GPU Temperature: {}", gpu_info.format_get_temperature_gpu());
            }
            if gpu_options.utilization {
                println!("GPU Utilization: {}", gpu_info.format_get_utilization_gpu());
            }
            if gpu_options.power_usage {
                println!("GPU Power Usage: {}", gpu_info.format_get_power_usage_gpu());
            }
            if gpu_options.clock_speed {
                println!("GPU Clock Speed: {}", gpu_info.format_get_clock_speed_gpu());
            }
        }
    }

    if options.all
        || !(options.system_type
            || options.system_version
            || options.bit_depth
            || options.architecture)
    {
        if options.system_type
            || options.system_version
            || options.bit_depth
            || options.architecture
        {
            warn!("--all supersedes all other options");
        }

        println!(
            "OS information:\nType: {}\nVersion: {}\nBitDepth: {} \nArchitecture: {}",
            info.system_type(),
            info.version(),
            info.bit_depth(),
            info.architecture().unwrap()
        );
    } else {
        if options.system_type {
            println!("OS Type: {}", info.system_type());
        }

        if options.system_version {
            println!("OS Version: {}", info.version());
        }

        if options.bit_depth {
            println!("OS BitDepth: {}", info.bit_depth());
        }

        if options.architecture {
            println!("OS Architecture: {}", info.architecture().unwrap());
        }
    }
}
