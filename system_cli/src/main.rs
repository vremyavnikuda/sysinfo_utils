use clap::Parser;
use log::warn;

#[derive(Parser)]
#[clap(about, version)]
struct Options {
    /// Show all OS information.
    #[clap(long)]
    all: bool,
    /// Show OS type.
    #[clap(short = 't', long = "type")]
    system_type: bool,
    /// Show OS version.
    #[clap(short = 'v', long)]
    system_version: bool,
    /// Show OS bit_depth.
    #[clap(short, long)]
    bit_depth: bool,
    /// Show OS architecture.
    #[clap(short = 'A', long = "Arch")]
    architecture: bool,
}


fn main() {
    env_logger::init();

    let options = Options::parse();
    let info = system_info::get();

    if options.all
        || !(options.system_type || options.system_version || options.bit_depth || options.architecture)
    {
        if options.system_type || options.system_version || options.bit_depth || options.architecture {
            warn!("--all supersedes all other options");
        }

        println!(
            "OS information:\nType: {}\nVersion: {}\nBitDepth: {} \nArchitecture: {}",
            info.system_type(),
            info.version(),
            info.bit_depth(),
            info.architecture().unwrap()
        );
    } else {
        if options.system_type {
            println!("OS Type: {}", info.system_type());
        }

        if options.system_version {
            println!("OS Version: {}", info.version());
        }

        if options.bit_depth {
            println!("OS BitDepth: {}", info.bit_depth());
        }

        if options.architecture {
            println!("OS Architecture: {}", info.architecture().unwrap());
        }
    }
}
