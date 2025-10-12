//examples/version.rs
//! Prints OS information.
//!
//! Prints the full information as well as the components of `system_info_lib::Info`
//! separately.

fn main() {
    let info = system_info_lib::get();

    // Print full information:
    println!("OS information: {info}");

    // Print information separately:
    println!("Type: {}", info.system_type());
    println!("Version: {}", info.version());
    println!("Edition: {:?}", info.edition());
    println!("Codename: {:?}", info.codename());
    println!("BitDepth: {}", info.bit_depth());
    println!("Architecture: {:?}", info.architecture());
}
