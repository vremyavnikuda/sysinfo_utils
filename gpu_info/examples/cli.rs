//! # GPU Info CLI Example
//!
//! This example demonstrates comprehensive GPU information display with all available metrics.
//!
//! ## Usage
//!
//! Run with default logging (errors only):
//! ```bash
//! cargo run --example cli
//! ```
//!
//! Run with info-level logging:
//! ```bash
//! RUST_LOG=info cargo run --example cli
//! ```
//!
//! Run with debug-level logging:
//! ```bash
//! RUST_LOG=debug cargo run --example cli
//! ```
//!
//! ## Features Demonstrated
//!
//! - Basic GPU detection with `gpu_info::get()`
//! - Accessing all GPU metrics (temperature, utilization, power, memory, clocks)
//! - Using format methods that automatically show "N/A" for unsupported metrics
//! - Proper error handling and graceful degradation

use log::info;

/// Entry point that initializes logging and displays all GPU metrics.
fn main() {
    env_logger::init();
    let gpu = gpu_info::get();

    info!("Vendor: {}", gpu.vendor());
    info!("Name: {}", gpu.format_name_gpu());
    info!("Driver Version: {}", gpu.format_driver_version());
    match gpu.active() {
        Some(true) => info!("Status: Active"),
        Some(false) => info!("Status: Inactive"),
        None => info!("Status: Unknown"),
    }
    info!("GPU Utilization: {}", gpu.format_utilization());
    info!("Temperature: {}", gpu.format_temperature());
    info!("Core Clock: {}", gpu.format_core_clock());
    info!("Memory Clock: {}", gpu.format_memory_clock());
    info!("Max Clock Speed: {}", gpu.format_max_clock_speed());
    info!("Memory Used: {}", gpu.format_memory_used());
    info!("Memory Total: {}", gpu.format_memory_total());
    info!("Memory Utilization: {}", gpu.format_memory_util());
    info!("Power Usage: {}", gpu.format_power_usage());
    info!("Power Limit: {}", gpu.format_power_limit());
}
