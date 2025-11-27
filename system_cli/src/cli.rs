use clap::Parser;

#[derive(Parser)]
#[clap(about, disable_version_flag = true)]
pub struct Options {
    /// Show all OS information.
    #[clap(long)]
    pub all: bool,
    /// Show OS type.
    #[clap(short = 't', long = "type")]
    pub system_type: bool,
    /// Show OS version.
    #[clap(short = 'v', long = "version")]
    pub system_version: bool,
    /// Show OS bitness.
    #[clap(short = 'b', long = "bitness")]
    pub bit_depth: bool,
    /// Show OS architecture.
    #[clap(short = 'a', long)]
    pub architecture: bool,
}
