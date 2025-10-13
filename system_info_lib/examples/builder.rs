//! Builder pattern example.
//!
//! This example demonstrates how to use the `InfoBuilder` to construct
//! custom `Info` instances with a fluent API using method chaining.

use system_info_lib::prelude::*;

fn main() {
    println!("Building custom system information using the builder pattern:");

    println!("Example 1: Creating a Linux system configuration");
    let linux_info = Info::builder()
        .system_type(Type::Linux)
        .version(SystemVersion::semantic(5, 15, 0))
        .edition("Ubuntu")
        .codename("Jammy Jellyfish")
        .bit_depth(BitDepth::X64)
        .architecture("x86_64")
        .build();

    println!("{}", linux_info);

    println!("Example 2: Creating a Windows system configuration");
    let windows_info = Info::builder()
        .system_type(Type::Windows)
        .version(SystemVersion::custom("NT 10.0"))
        .edition("Pro")
        .bit_depth(BitDepth::X64)
        .build();

    println!("{}", windows_info);

    println!("Example 3: Creating a macOS system configuration");
    let macos_info = Info::builder()
        .system_type(Type::Macos)
        .version(SystemVersion::semantic(14, 0, 0))
        .codename("Sonoma")
        .bit_depth(BitDepth::X64)
        .architecture("arm64")
        .build();

    println!("{}", macos_info);

    println!("Example 4: Minimal configuration with defaults");
    let minimal_info = Info::builder().system_type(Type::Arch).build();

    println!("{}", minimal_info);

    println!("Example 5: Rolling release system");
    let arch_info = Info::builder()
        .system_type(Type::Arch)
        .version(SystemVersion::rolling(Some("latest")))
        .bit_depth(BitDepth::X64)
        .architecture("x86_64")
        .build();

    println!("{}", arch_info);

    println!("Comparing with actual system information:");
    let actual_info = get();
    println!("{}", actual_info);

    println!("Builder pattern allows flexible construction:");
    println!("Method chaining for fluent API");
    println!("Optional fields can be omitted");
    println!("Accepts both &str and String");
    println!("Type-safe construction");
}
