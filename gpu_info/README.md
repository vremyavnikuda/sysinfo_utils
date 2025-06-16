# GPU Info

A cross-platform Rust library for retrieving GPU information and monitoring metrics.

## Features

- Support for multiple GPU vendors (NVIDIA, AMD, Intel)
- Real-time GPU metrics monitoring
- Caching support for performance optimization
- Cross-platform compatibility (Windows, Linux, macOS)
- Safe error handling with Result types
- Formatted output for all metrics
- Smart formatting for all data types (Option<T> support)

## Supported Metrics

- Vendor and model information
- Temperature
- GPU utilization
- Core and memory clock speeds
- Power usage and limits
- Memory usage and total memory
- Active state
- Driver version

## Formatting

The library provides smart formatting for all metrics with the following features:
- Automatic handling of `Option<T>` values (returns "N/A" for `None`)
- Precision control for floating-point values (1 decimal place)
- Consistent formatting across all data types
- Support for:
  - `Option<f32>` (temperature, utilization, power)
  - `Option<u32>` (clock speeds, memory)
  - `Option<bool>` (active state)
  - `Option<String>` and `Option<&str>` (names, versions)
  - `String` (raw string values)

### Formatting Examples

```rust
let gpu = gpu_info::get()?;

// Using format_* methods
println!("Temperature: {}", gpu.format_temperature());
println!("Core Clock: {}", gpu.format_core_clock());
println!("Active: {}", gpu.format_active());
println!("Name: {}", gpu.format_name_gpu());
println!("Driver: {}", gpu.format_driver_version());

// Using fmt_string() method directly
println!("Temperature: {}", gpu.temperature.fmt_string());
println!("Core Clock: {}", gpu.core_clock.fmt_string());
println!("Active: {}", gpu.active.fmt_string());
println!("Name: {}", gpu.name_gpu.fmt_string());
println!("Driver: {}", gpu.driver_version.fmt_string());

// Examples with Option<T> values
let temp: Option<f32> = Some(75.5);
let clock: Option<u32> = Some(1800);
let active: Option<bool> = Some(true);
let name: Option<String> = Some("NVIDIA GeForce RTX 3080".to_string());
let driver: Option<&str> = Some("512.95");

println!("Temperature: {}", temp.fmt_string());  // "75.5"
println!("Core Clock: {}", clock.fmt_string());  // "1800"
println!("Active: {}", active.fmt_string());     // "true"
println!("Name: {}", name.fmt_string());         // "NVIDIA GeForce RTX 3080"
println!("Driver: {}", driver.fmt_string());     // "512.95"

// Examples with None values
let temp: Option<f32> = None;
let clock: Option<u32> = None;
let active: Option<bool> = None;
let name: Option<String> = None;
let driver: Option<&str> = None;

println!("Temperature: {}", temp.fmt_string());  // "N/A"
println!("Core Clock: {}", clock.fmt_string());  // "N/A"
println!("Active: {}", active.fmt_string());     // "N/A"
println!("Name: {}", name.fmt_string());         // "N/A"
println!("Driver: {}", driver.fmt_string());     // "N/A"
```

## Usage

```rust
use gpu_info::GpuInfo;

// Get GPU information
let gpu = gpu_info::get()?;

// Access raw metrics
println!("Vendor: {:?}", gpu.vendor);
println!("Name: {:?}", gpu.name_gpu);
println!("Utilization: {:?}", gpu.utilization);
println!("Temperature: {:?}", gpu.temperature);
println!("Clock Speed: {:?}", gpu.core_clock);
println!("Power Usage: {:?}", gpu.power_usage);
println!("Memory Usage: {:?}", gpu.memory_util);
println!("Memory Total: {:?}", gpu.memory_total);
println!("Is active: {:?}", gpu.active);

// Use formatted output
println!("Formatted Name: {}", gpu.format_name_gpu());
println!("Formatted Temperature: {}", gpu.format_temperature());
println!("Formatted Power Usage: {}", gpu.format_power_usage());
println!("Formatted Core Clock: {}", gpu.format_core_clock());
println!("Formatted Memory Usage: {}", gpu.format_memory_util());
println!("Formatted Active State: {}", gpu.format_active());
println!("Formatted Power Limit: {}", gpu.format_power_limit());
println!("Formatted Memory Total: {}", gpu.format_memory_total());
println!("Formatted Driver Version: {}", gpu.format_driver_version());
println!("Formatted Max Clock Speed: {}", gpu.format_max_clock_speed());
```

## Examples

Run the detailed example:
```bash
cargo run --example detailed
```

Run the caching example:
```bash
cargo run --example cache
```

## Dependencies

- Windows: NVIDIA NVML, AMD ADL, or Intel WMI
- Linux: NVIDIA NVML or AMD ADL
- macOS: Apple Metal API (support currently suspended)

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.