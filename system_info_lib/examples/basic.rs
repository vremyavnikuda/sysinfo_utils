//! Basic example of getting system information.
//!
//! This example demonstrates the simplest way to retrieve and display
//! system information using the `system_info_lib` crate.

use system_info_lib::prelude::*;

fn main() {
    let info = get();

    println!("System Information:");
    println!("  OS Type: {}", info.system_type());
    println!("  Version: {}", info.version());
    println!("  Architecture: {}", info.bit_depth());
}
