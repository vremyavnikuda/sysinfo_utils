//! Detailed system information example.
//!
//! This example demonstrates how to retrieve and display comprehensive
//! system information with conditional formatting based on available data.

use system_info_lib::prelude::*;

fn main() {
    let info = get();

    println!("Operating System:");
    println!("Type: {}", info.system_type());
    println!("Version: {}", info.version());

    if let Some(edition) = info.edition() {
        println!("Edition: {}", edition);
    } else {
        println!("Edition: Not available");
    }

    if let Some(codename) = info.codename() {
        println!("Codename: {}", codename);
    } else {
        println!("Codename: Not available");
    }

    println!();
    println!("Architecture:");
    println!("Bit Depth: {}", info.bit_depth());

    if let Some(arch) = info.architecture() {
        println!("Architecture: {}", arch);
    } else {
        println!("Architecture: Not available");
    }

    println!();
    println!("Full Info: {}", info);
    println!();

    match info.system_type() {
        Type::Linux => {
            println!("Running on Linux system");
            if info.bit_depth() == BitDepth::X64 {
                println!("  64-bit architecture detected");
            }
        }
        Type::Windows => {
            println!("Running on Windows system");
            if let Some(edition) = info.edition() {
                println!("Edition: {}", edition);
            }
        }
        Type::Macos => {
            println!("Running on macOS system");
            if let Some(codename) = info.codename() {
                println!("Codename: {}", codename);
            }
        }
        _ => {
            println!("Running on: {}", info.system_type());
        }
    }
}
