//! # GPU Info CLI Example
//!
//! This example demonstrates basic usage of the `gpu_info` library with logging.
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
//! - Accessing GPU metrics (temperature, utilization, power, memory)
//! - Using the `Formattable` trait for user-friendly output
//! - Proper error handling and graceful degradation

use log::info;

/// Entry point that initializes logging and displays GPU metrics.
fn main() {
    env_logger::init();
    let gpu = gpu_info::get();
    info!("Vendor: {}", gpu.vendor());
    info!("Name: {}", gpu.format_name_gpu());
    info!("Utilization: {}", gpu.format_utilization());
    info!("Temperature: {}", gpu.format_temperature());
    info!("Clock Speed: {}", gpu.format_core_clock());
    info!("Power Usage: {}", gpu.format_power_usage());
    info!("Memory Usage: {}", gpu.format_memory_util());
    info!("Memory Total: {}", gpu.format_memory_total());
    match gpu.active() {
        Some(true) => info!("Is active: Yes"),
        Some(false) => info!("Is active: No"),
        None => info!("Is active: Unknown"),
    }
}
