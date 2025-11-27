//! system_info_lib/system_cl/src/main.rs
use clap::Parser;
use system_cli::Options;

fn main() {
    env_logger::init();
    let options = Options::parse();
    let info = system_info_lib::get();

    if options.all
        || (!options.system_type
            && !options.system_version
            && !options.bit_depth
            && !options.architecture)
    {
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
