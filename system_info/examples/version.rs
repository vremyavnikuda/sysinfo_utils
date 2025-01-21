//! Prints OS information.
//!
//! Prints the full information as well as the components of `system_info::Info`
//! separately.

fn main() {
    let info = system_info::get();

    // Print full information:
    println!("OS information: {info}");

    // Print information separately:
    println!("Type: {}", info.os_type());
    println!("Version: {}", info.version());
    println!("Edition: {:?}", info.edition());
    println!("Codename: {:?}", info.codename());
    println!("Bitness: {}", info.bitness());
    println!("Architecture: {:?}", info.architecture());
}
