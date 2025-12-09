//! System information CLI tool
//!
//! Displays OS and GPU information.
use clap::Parser;
use system_cli::Options;

fn main() {
    env_logger::init();
    let options = Options::parse();

    let show_all = options.all
        || (!options.system_type
            && !options.system_version
            && !options.bit_depth
            && !options.architecture
            && !options.gpu);

    // Show OS information
    if show_all
        || options.system_type
        || options.system_version
        || options.bit_depth
        || options.architecture
    {
        let info = system_info_lib::get();

        if show_all {
            println!("OS information:");
            println!("  Type: {}", info.system_type());
            println!("  Version: {}", info.version());
            if let Some(edition) = info.edition() {
                println!("  Edition: {}", edition);
            }
            if let Some(codename) = info.codename() {
                println!("  Codename: {}", codename);
            }
            println!("  Bitness: {}", info.bit_depth());
            if let Some(arch) = info.architecture() {
                println!("  Architecture: {}", arch);
            }
        } else {
            if options.system_type {
                println!("OS type: {}", info.system_type());
            }
            if options.system_version {
                println!("OS version: {}", info.version());
            }
            if options.bit_depth {
                println!("OS bitness: {}", info.bit_depth());
            }
            if options.architecture {
                if let Some(arch) = info.architecture() {
                    println!("OS architecture: {}", arch);
                }
            }
        }
    }

    // Show GPU information
    if show_all || options.gpu {
        let gpu = gpu_info::get();

        if show_all {
            println!();
        }
        println!("GPU information:");
        println!("  Vendor: {}", gpu.vendor());
        println!("  Name: {}", gpu.format_name_gpu());
        println!("  Driver: {}", gpu.format_driver_version());
        println!("  Temperature: {} C", gpu.format_temperature());
        println!("  Utilization: {}%", gpu.format_utilization());
        println!("  Core Clock: {} MHz", gpu.format_core_clock());
        println!("  Memory: {} GB", gpu.format_memory_total());
        println!("  Memory Usage: {}%", gpu.format_memory_util());
        println!("  Power: {} W", gpu.format_power_usage());
        println!("  Status: {}", gpu.format_active());
    }
}
