# System Information Library

### A Rust library for retrieving system and GPU information with a convenient command-line interface.

## Features
This project is a library that can be used in your projects to determine the user's operating system , bit depth, and release date.

- Operating System information retrieval (type, version, architecture, bit depth)
- GPU information retrieval (vendor, temperature, utilization, power usage, clock speed)
- Command-line interface with flexible options
- Logging support

## Installation
```shell
cargo add system_info
```
## Dependencies
```toml
[dependencies]
system_info = "0.1"
gpu_info = "0.1"
clap = { version = "4", features = ["derive"] }
log = "0.4"
env_logger = "0.10"
```

#### Logging initialization
```rust
env_logger::init();
```
#### Getting OS information
```rust
env_logger::init();
let info = system_info::get();
info.system_type();
info.version();
info.bit_depth();
info.architecture();
```
#### Getting GPU information
```rust
env_logger::init();
let mut manager = GpuManager::new();
manager.refresh();

gpu_info.vendor_gpu();
gpu_info.name_gpu();
gpu_info.get_temperature_gpu();
gpu_info.get_utilization_gpu();
gpu_info.get_power_usage_gpu();
gpu_info.get_clock_speed_gpu();
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
system_info --name            # Show GPU vendor (long format)
system_info -t                 # Show GPU temperature
system_info --temperature     # Show GPU temperature (long format)
system_info -u                 # Show GPU utilization
system_info --utilization     # Show GPU utilization (long format)
system_info -s                 # Show GPU power usage
system_info --power          # Show GPU power usage (long format)
system_info -a                 # Show GPU clock speed
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
