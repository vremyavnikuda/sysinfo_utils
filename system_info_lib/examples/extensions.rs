//! Extension traits example.
//!
//! This example demonstrates how to use the extension traits to conveniently
//! query system information properties.

use system_info_lib::prelude::*;

fn main() {
    let info = get();

    println!("Is Windows: {}", info.is_windows());
    println!("Is Linux: {}", info.is_linux());
    println!("Is macOS: {}", info.is_macos());
    println!("Is BSD: {}", info.is_bsd());
    println!("Is Unknown: {}", info.is_unknown_system());
    println!("Is 64-bit: {}", info.is_64bit());
    println!("Is 32-bit: {}", info.is_32bit());
    println!("Unknown bit depth: {}", info.has_unknown_bit_depth());
    println!("Has kernel version: {}", info.has_kernel_version());
    if let Some(kernel) = info.kernel_version() {
        println!("Kernel: {}", kernel);
    }
    let version = info.version();
    println!("Is semantic: {}", version.is_semantic());
    println!("Is rolling: {}", version.is_rolling());
    println!("Is custom: {}", version.is_custom());
    println!("Is unknown: {}", version.is_unknown());
    if version.is_semantic() {
        println!("Semantic version components:");
        if let Some(major) = version.major() {
            println!("Major: {}", major);
        }
        if let Some(minor) = version.minor() {
            println!("Minor: {}", minor);
        }
        if let Some(patch) = version.patch() {
            println!("Patch: {}", patch);
        }
    }

    if version.is_rolling() {
        if let Some(codename) = version.codename() {
            println!("Rolling release codename: {}", codename);
        } else {
            println!("Rolling release (no codename)");
        }
    }

    let custom = Info::builder()
        .system_type(Type::Ubuntu)
        .version(SystemVersion::semantic(22, 4, 0))
        .bit_depth(BitDepth::X64)
        .kernel_version("5.15.0-76-generic")
        .build();

    println!("Custom info is Linux: {}", custom.is_linux());
    println!("Custom info is 64-bit: {}", custom.is_64bit());
    println!("Custom info has kernel: {}", custom.has_kernel_version());

    let custom_version = custom.version();
    if custom_version.is_semantic() {
        println!(
            "Custom version: {}.{}.{}",
            custom_version.major().unwrap_or(0),
            custom_version.minor().unwrap_or(0),
            custom_version.patch().unwrap_or(0)
        );
    }

    if info.is_linux() && info.is_64bit() {
        println!("Running on 64-bit Linux system");
    } else if info.is_windows() {
        println!("Running on Windows system");
    } else if info.is_macos() {
        println!("Running on macOS system");
    } else if info.is_bsd() {
        println!("Running on BSD system");
    } else {
        println!("Running on unknown system");
    }

    if info.version().is_semantic() {
        println!("System uses semantic versioning");
    } else if info.version().is_rolling() {
        println!("System uses rolling release model");
    }
}
