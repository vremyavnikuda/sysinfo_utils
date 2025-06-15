use clap::Parser;

#[derive(Parser)]
#[clap(about, version)]
pub struct Options {
    /// Show all OS information.
    #[clap(long)]
    pub all: bool,
    /// Show OS type.
    #[clap(short = 't', long = "type")]
    pub system_type: bool,
    /// Show OS version.
    #[clap(short = 'v', long)]
    pub system_version: bool,
    /// Show OS bit_depth.
    #[clap(short, long)]
    pub bit_depth: bool,
    /// Show OS architecture.
    #[clap(short = 'A', long = "Arch")]
    pub architecture: bool,
}

#[derive(Parser)]
#[clap(about, version)]
pub struct GpuOptions {
    /// Show all GPU information.
    #[clap(long)]
    pub all: bool,
    /// Show GPU vendor.
    #[clap(short = 'n', long = "name_gpu")]
    pub vendor: bool,
    /// Show GPU temperature.
    #[clap(short = 't', long = "temperature")]
    pub temperature: bool,
    /// Show GPU utilization.
    #[clap(short = 'u', long = "utilization")]
    pub utilization: bool,
    /// Show GPU power usage.
    #[clap(short = 'p', long = "power")]
    pub power_usage: bool,
    /// Show GPU clock speed.
    #[clap(short = 'c', long = "clock")]
    pub clock_speed: bool,
}
