# GPU Info

A cross-platform Rust library for retrieving GPU information and monitoring metrics with unified provider interface.

## Features

- Support for multiple GPU vendors (NVIDIA, AMD, Intel)
- Real-time GPU metrics monitoring
- Caching support for performance optimization
- Cross-platform compatibility (Windows, Linux, macOS)
- Safe error handling with Result types
- Formatted output for all metrics
- Smart formatting for all data types (Option<T> support)
- **Unified provider interface for all GPU vendors**
- **Reduced code duplication through common utilities**
- **Enhanced extensibility with modular architecture**
- **Asynchronous API for non-blocking operations**

## Supported Metrics

- Vendor and model information
- Temperature
- GPU utilization
- Core and memory clock speeds
- Power usage and limits
- Memory usage and total memory
- Active state
- Driver version
- Max clock speed
- Memory clock
- Power limit
- Memory utilization

## Architecture

The library follows a modular architecture with unified provider interfaces:

```
gpu_info/
├── src/
│   ├── gpu_info.rs          # Core data structures and traits
│   ├── macros.rs            # Code generation macros for reduced duplication
│   ├── provider_manager.rs   # Centralized provider manager
│   ├── providers/            # GPU provider implementations
│   │   ├── nvidia.rs        # NVIDIA provider
│   │   ├── amd.rs           # AMD provider
│   │   ├── intel.rs         # Intel provider
│   │   ├── linux/           # Linux-specific providers
│   │   ├── macos/           # macOS-specific providers
│   │   └── windows/         # Windows-specific provider utilities
│   │       ├── mod.rs       # Module exports
│   │       ├── pdh.rs       # Performance Data Helper API
│   │       └── intel_metrics.rs  # Intel Metrics API (GPA)
│   ├── ffi_utils.rs         # Common FFI utilities
│   ├── nvml_api.rs          # NVML API abstraction
│   ├── adl_api.rs           # ADL API abstraction
│   └── cache_utils.rs       # Common caching utilities
├── examples/                # Usage examples
└── tests/                   # Comprehensive test suite
```

## Unified Provider Interface

The library now provides a unified `GpuProvider` trait that eliminates code duplication:

```rust
pub trait GpuProvider: Send + Sync {
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>>;
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()>;
    fn get_vendor(&self) -> Vendor;
}
```

## Enhanced Caching

The library now provides enhanced caching features for improved performance:

1. **TTL-based expiration**: Cached entries automatically expire after a configurable time-to-live
2. **LRU eviction**: Optional size-limited caching with least-recently-used eviction
3. **Access tracking**: Cache entries track access frequency and last access time
4. **Cache statistics**: Monitor cache performance with detailed statistics

### Usage Examples

```rust
use gpu_info::gpu_manager::GpuManager;
use std::time::Duration;
let manager = GpuManager::with_cache_config(Duration::from_secs(2), 10);
if let Some(gpu) = manager.get_gpu_cached(0) {
    println!("GPU: {} (temp: {}°C)", gpu.format_name_gpu(), gpu.format_temperature());
}
if let Some(stats) = manager.get_cache_stats() {
    println!("Cache entries: {}, Accesses: {}", stats.total_entries, stats.total_accesses);
}
```

### Formatting Examples

```rust
let gpu = gpu_info::get()?;
println!("Temperature: {}", gpu.format_temperature());
println!("Core Clock: {}", gpu.format_core_clock());
println!("Active: {}", gpu.format_active());
println!("Name: {}", gpu.format_name_gpu());
println!("Driver: {}", gpu.format_driver_version());
println!("Temperature: {}", gpu.temperature.fmt_string());
println!("Core Clock: {}", gpu.core_clock.fmt_string());
println!("Active: {}", gpu.active.fmt_string());
println!("Name: {}", gpu.name_gpu.fmt_string());
println!("Driver: {}", gpu.driver_version.fmt_string());
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

### Simple Usage - Get Primary GPU

```rust
use gpu_info::GpuInfo;

fn main() {
    let gpu = gpu_info::get();
    println!("Vendor: {}", gpu.vendor);
    println!("Name GPU: {}", gpu.format_name_gpu());
    println!("Driver: {}", gpu.format_driver_version());
    println!("Temperature: {}°C", gpu.format_temperature());
    println!("Utilization: {}%", gpu.format_utilization());
    println!("Core Clock: {} MHz", gpu.format_core_clock());
    println!("Memory Clock: {} MHz", gpu.format_memory_clock());
    println!("Max Clock Speed: {} MHz", gpu.format_max_clock_speed());
    println!("Memory Usage: {}%", gpu.format_memory_util());
    println!("Total Memory: {} GB", gpu.format_memory_total());
    println!("Current Usage: {} W", gpu.format_power_usage());
    println!("Power Limit: {} W", gpu.format_power_limit());
    println!("Active: {}", gpu.format_active());
}
```

### Working with Multiple GPUs

```rust
use gpu_info::GpuManager;use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create manager with caching enabled
    let manager = GpuManager::with_cache_config(Duration::from_secs(2), 10);
    
    // Get all GPUs (owned copies)
    let gpus = manager.get_all_gpus_owned();
    println!("Found {} GPU(s)", gpus.len());
    
    for i in 0..gpus.len() {
        // Use cached access for better performance (zero-copy via Arc)
        if let Some(gpu) = manager.get_gpu_cached(i) {
            println!("GPU #{}: {} ({})", i, gpu.format_name_gpu(), gpu.vendor);
            println!("  Temperature: {}°C", gpu.format_temperature());
            println!("  Utilization: {}%", gpu.format_utilization());
            println!("  Power Usage: {}W", gpu.format_power_usage());
        }
    }
    
    // Get primary GPU with caching (zero-copy)
    if let Some(primary_gpu) = manager.get_primary_gpu_cached() {
        println!("Primary GPU: {}", primary_gpu.format_name_gpu());
    }
    Ok(())
}
```

### Using Provider Manager (Advanced)

```rust
use gpu_info::{GpuProviderManager, providers::{NvidiaProvider, AmdProvider, IntelProvider}};
use gpu_info::vendor::Vendor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider manager
    let mut provider_manager = GpuProviderManager::new();
    
    // Register providers for different vendors
    #[cfg(target_os = "windows")]
    {
        provider_manager.register_provider(Vendor::Nvidia, Box::new(NvidiaProvider::new()));
        provider_manager.register_provider(Vendor::Amd, Box::new(AmdProvider::new()));
        provider_manager.register_provider(Vendor::Intel(Default::default()), Box::new(IntelProvider::new()));
    }
    #[cfg(target_os = "linux")]
    {
        // For Linux, you might use different providers
        // provider_manager.register_provider(Vendor::Nvidia, Box::new(crate::providers::linux::NvidiaLinuxProvider::new()));
    }
    let gpus = provider_manager.detect_all_gpus();
    println!("Detected {} GPU(s) using provider manager", gpus.len());
    for gpu in gpus {
        println!("GPU: {} ({})", gpu.format_name_gpu(), gpu.vendor);
    }
    Ok(())
}
```

### Using Async API

```rust
use gpu_info::{get_async, get_all_async};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpu = get_async().await?;
    println!("Primary GPU: {}", gpu.format_name_gpu());
    println!("Temperature: {}°C", gpu.format_temperature());
    println!("Utilization: {}%", gpu.format_utilization());
    let gpus = get_all_async().await?;
    println!("Found {} GPU(s)", gpus.len());
    for (i, gpu) in gpus.iter().enumerate() {
        println!("GPU {}: {}", i, gpu.format_name_gpu());
        println!("  Temperature: {}°C", gpu.format_temperature());
        println!("  Utilization: {}%", gpu.format_utilization());
        println!("  Power Usage: {}W", gpu.format_power_usage());
    }
    Ok(())
}
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

Run the enhanced caching example:
```bash
cargo run --example enhanced_cache
```

Run the provider manager example:
```bash
cargo run --example provider_manager
```

Run the async API example:
```bash
cargo run --example async_example
```

## Dependencies

- Windows: NVIDIA NVML, AMD ADL, or Intel WMI
- Linux: NVIDIA NVML or AMD ADL
- macOS: Apple Metal API (support currently suspended)

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.