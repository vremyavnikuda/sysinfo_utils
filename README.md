# System Information Library Doc

### A Rust library for retrieving system and GPU information with a convenient command-line interface.

## Features

This project is a library that can be used in your projects to determine the user's operating system , bit depth, and release date.

- Operating System information retrieval (type, version, architecture, bit depth)
- GPU information retrieval (vendor, temperature, utilization, power usage, clock speed)
- Command-line interface with flexible options
- Logging support

## Installation

```shell
cargo add sysinfo_utils
```

## Dependencies

```toml
[dependencies]
system_info = "0.1"
clap = { version = "4", features = ["derive"] }
log = "0.4"
env_logger = "0.10"
```

#### Logging initialization

```rust
env_logger::init();
```

#### Getting OS information

[list](os.md) of supported operating systems

```rust
env_logger::init();
let info = system_info::get();
"Type: {}", info.system_type()
"Version: {}", info.version()
"Edition: {:?}", info.edition()
"Codename: {:?}", info.codename()
"BitDepth: {}", info.bit_depth()
"Architecture: {:?}", info.architecture()
```

#### Getting GPU information

[list](gpu.md) of supported GPUs

```rust
env_logger::init();
// system_info_lib
let _options = Options::parse();
let info = system_info::get();

// gpu_info
let _gpu_options = GpuOptions::parse();
let gpu = gpu_info::get();
//default output
"Vendor: {:?}", gpu.vendor()
"Name: {:?}", gpu.name_gpu()
"Utilization: {:?}", gpu.utilization()
"Temperature: {:?}", gpu.temperature()
"Clock Speed: {:?}", gpu.core_clock()
"Power Usage: {:?}", gpu.power_usage()
"Memory Usage: {:?}", gpu.memory_util()
"Memory Total: {:?}", gpu.memory_total()
"Is active: {:?}", gpu.active()
"Type: {}", info.system_type()
"Version: {}", info.version()
"Edition: {:?}", info.edition()
"Codename: {:?}", info.codename()
"BitDepth: {}", info.bit_depth()
"Architecture: {:?}", info.architecture()

//.fmt_string() -> format to_string
"Vendor: {:?}", gpu.vendor()
"Name: {:?}", gpu.name_gpu().fmt_string()
"Utilization: {:?}", gpu.utilization().fmt_string()
"Temperature: {:?}", gpu.temperature().fmt_string()
"Clock Speed: {:?}", gpu.core_clock().fmt_string()
"Power Usage: {:?}", gpu.power_usage().fmt_string()
"Memory Usage: {:?}", gpu.memory_util().fmt_string()
"Memory Total: {:?}", gpu.memory_total().fmt_string()

// Formatting with units of measurement
"Utilization: {:?}", gpu.format_utilization()
"Temperature: {:?}", gpu.format_temperature()
"Clock Speed: {:?}", gpu.format_core_clock()
"Power Usage: {:?}", gpu.format_power_usage()
"Memory Usage: {:?}", gpu.format_memory_clock()
"Memory Total: {:?}", gpu.format_memory_total()
```

## Usage

The library provides two main command sets: system information and GPU information.

### System Information Commands

```bash
system_info [OPTIONS]

# Show all system information
system_info --all

# Show specific system information
system_info -t                  # Show OS type
system_info --type             # Show OS type (long format)
system_info -v                 # Show OS version
system_info --system-version   # Show OS version (long format)
system_info -b                 # Show OS bit depth
system_info --bit-depth       # Show OS bit depth (long format)
system_info -A                 # Show OS architecture
system_info --Arch            # Show OS architecture (long format)

# Show specific GPU information
system_info -v                 # Show GPU vendor
system_info --name_gpu            # Show GPU vendor (long format)
system_info -t                 # Show GPU temperature
system_info --temperature     # Show GPU temperature (long format)
system_info -u                 # Show GPU utilization
system_info --utilization     # Show GPU utilization (long format)
system_info -s                 # Show GPU power usage
system_info --power          # Show GPU power usage (long format)
system_info -c                 # Show GPU clock speed
system_info --clock          # Show GPU clock speed (long format)
```

### Output Examples

#### System Information

```bash
OS information:
Type: Windows
Version: 10.0
BitDepth: 64-bit
Architecture: x86_64
```

#### GPU Information

```bash
NVIDIA GeForce RTX 4090
Temperature: 49°C
Utilization: 11%
Power Usage: 69.55/526 W
Clock Speed: 1155/2565 MHz
```

## Notes

- Using the `--all` flag will override any other options and display all available information
- The library includes logging functionality that will warn when `--all` supersedes other options

## Contributing

Contributions are not being considered at the moment!
