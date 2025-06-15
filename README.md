# System Info Utils

A utility library for system and GPU information.

## Features

- System information (OS type, version, architecture, etc.)
- GPU information (vendor, temperature, utilization, etc.)
- Command-line interface for both system and GPU information

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
sysinfo_utils = { path = "path/to/sysinfo_utils" }
```

### System Information

```rust
use sysinfo_utils::system_info_lib;

fn main() {
    let info = system_info_lib::get();

    println!("OS information: {info}");
    println!("Type: {}", info.system_type());
    println!("Version: {}", info.version());
    println!("Edition: {:?}", info.edition());
    println!("Codename: {:?}", info.codename());
    println!("BitDepth: {}", info.bit_depth());
    println!("Architecture: {:?}", info.architecture());
}
```

### GPU Information

```rust
use sysinfo_utils::gpu_info;

fn main() {
    let gpu = gpu_info::get();

    println!("Vendor: {:?}", gpu.vendor());
    println!("Name: {:?}", gpu.name_gpu());
    println!("Utilization: {:?}", gpu.utilization());
    println!("Temperature: {:?}", gpu.temperature());
    println!("Clock Speed: {:?}", gpu.core_clock());
    println!("Power Usage: {:?}", gpu.power_usage());
    println!("Memory Usage: {:?}", gpu.memory_util());
    println!("Memory Total: {:?}", gpu.memory_total());
    println!("Is active: {:?}", gpu.active());
}
```

## Command-line Interface

The package includes a command-line interface for both system and GPU information:

```bash
# Show all system information
system_cli --all

# Show specific system information
system_cli --type
system_cli --version
system_cli --bit-depth
system_cli --architecture

# Show all GPU information
system_cli --all

# Show specific GPU information
system_cli --name-gpu
system_cli --temperature
system_cli --utilization
system_cli --power
system_cli --clock
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
