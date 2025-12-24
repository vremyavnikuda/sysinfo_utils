# GPU Info

A cross-platform Rust library for retrieving GPU information and monitoring metrics with unified provider interface.

## Minimum Supported Rust Version (MSRV)

This crate requires **Rust 1.70** or later.

The MSRV is tested in CI and will only be increased in minor or major version updates, never in patch releases.

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
    println!("GPU: {} (temp: {})", gpu.name_or_default(), gpu.display_temperature());
}
if let Some(stats) = manager.get_cache_stats() {
    println!("Cache entries: {}, Accesses: {}", stats.total_entries, stats.total_accesses);
}
```

### Formatting Examples

The library provides two sets of formatting methods:
- **`display_*` methods** (recommended): Return formatted strings with units included
- **`format_*` methods** (deprecated): Legacy methods, use `display_*` instead

```rust
let gpu = gpu_info::get()?;

// New display_* methods (recommended) - units are included in output
println!("Temperature: {}", gpu.display_temperature());     // "65.50°C" or "Not supported"
println!("Utilization: {}", gpu.display_utilization());     // "75.50%" or "N/A"
println!("Core Clock: {}", gpu.display_core_clock());       // "1800 MHz" or "N/A"
println!("Power Usage: {}", gpu.display_power_usage());     // "250.00W" or "Not supported"
println!("Memory Total: {}", gpu.display_memory_total());   // "8.00 GB" or "N/A"
println!("Active: {}", gpu.display_active());               // "Active" or "Inactive"
println!("Name: {}", gpu.display_name_gpu());               // "NVIDIA GeForce RTX 3080" or "Unknown GPU"
println!("Driver: {}", gpu.display_driver_version());       // "545.92.01" or "Unknown Driver Version"

// Raw value access via getter methods (for custom formatting)
if let Some(temp) = gpu.temperature() {
    println!("Temperature: {:.1}°C", temp);
}

// Zero-allocation name access with Cow
println!("Name: {}", gpu.name_or_default());  // Returns Cow<'_, str>
```

## Usage

### Simple Usage - Get Primary GPU

```rust
use gpu_info::GpuInfo;

fn main() {
    let gpu = gpu_info::get();
    
    // Use getter methods for raw values
    println!("Vendor: {}", gpu.vendor());
    println!("Name: {}", gpu.name_or_default());
    
    // Use display_* methods for formatted output with units
    println!("Driver: {}", gpu.display_driver_version());
    println!("Temperature: {}", gpu.display_temperature());
    println!("Utilization: {}", gpu.display_utilization());
    println!("Core Clock: {}", gpu.display_core_clock());
    println!("Memory Clock: {}", gpu.display_memory_clock());
    println!("Max Clock Speed: {}", gpu.display_max_clock_speed());
    println!("Memory Usage: {}", gpu.display_memory_util());
    println!("Total Memory: {}", gpu.display_memory_total());
    println!("Current Usage: {}", gpu.display_power_usage());
    println!("Power Limit: {}", gpu.display_power_limit());
    println!("Active: {}", gpu.display_active());
    
    // Or access raw Option values via getters
    if let Some(temp) = gpu.temperature() {
        println!("Raw temperature: {:.1}°C", temp);
    }
}
```

### Creating GpuInfo Instances

You can create `GpuInfo` instances using the builder pattern, `unknown()`, conversion traits, or direct struct construction:

```rust
use gpu_info::{GpuInfo, vendor::Vendor};

// Using the builder pattern (recommended)
let gpu = GpuInfo::builder()
    .vendor(Vendor::Nvidia)
    .name("GeForce RTX 4090")
    .temperature(65.0)
    .utilization(45.0)
    .build();

// Using unknown() as a base for testing or fallback
let unknown_gpu = GpuInfo::unknown();

// Using From<Vendor> conversion
let nvidia_gpu: GpuInfo = Vendor::Nvidia.into();

// Using TryFrom<&str> for JSON parsing (requires serde feature)
#[cfg(feature = "serde")]
{
    use std::convert::TryFrom;
    let json = r#"{"vendor":"Nvidia","temperature":65.0}"#;
    let gpu = GpuInfo::try_from(json)?;
}
```

### Conversion Traits

`GpuInfo` implements several conversion traits for flexibility:

- `From<Vendor>` - Create a minimal `GpuInfo` from a vendor
- `TryFrom<&str>` - Parse from JSON string (requires `serde` feature)
- `AsRef<GpuInfo>` - Cheap reference conversion for generic functions

### Standard Traits

`GpuInfo` implements the following standard traits:

- `Clone`, `Debug`, `PartialEq` - Standard derivable traits
- `Default` - Creates an unknown GPU instance
- `Display` - Human-readable formatting
- `Hash` - Enables use as HashMap/HashSet keys (hashes by vendor + name only)
- `Serialize`, `Deserialize` - JSON serialization (requires `serde` feature)

Note: `Hash` only considers identity fields (vendor and name) to avoid issues with f32 metrics. Two GPUs with the same vendor and name but different metrics will hash to the same bucket.

`GpuManager` implements collection traits:

- `FromIterator<GpuInfo>` - Construct from an iterator of GPUs
- `Extend<GpuInfo>` - Add GPUs from an iterator
- `IntoIterator` - Iterate over GPUs (via `&GpuManager` and `&mut GpuManager`)

`Vendor` also implements `FromStr` for parsing vendor names from strings:

```rust
use gpu_info::{GpuInfo, vendor::Vendor};
use std::str::FromStr;

// Parse vendor from string (case-insensitive)
let nvidia = Vendor::from_str("nvidia").unwrap();
let amd: Vendor = "AMD".parse().unwrap();
let intel = Vendor::from_str("Intel Arc A770").unwrap();

// Supported aliases: "geforce", "quadro", "radeon", "ati", "arc", "iris", etc.
let nvidia_alias = Vendor::from_str("GeForce").unwrap();
assert_eq!(nvidia_alias, Vendor::Nvidia);

// Generic function accepting any type that can be referenced as GpuInfo
fn print_gpu_info(gpu: impl AsRef<GpuInfo>) {
    let g = gpu.as_ref();
    println!("Vendor: {}", g.vendor());
    println!("Name: {}", g.name_or_default());
}

let gpu = GpuInfo::unknown();
print_gpu_info(&gpu);
```

### Working with Multiple GPUs

```rust
use gpu_info::GpuManager;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create manager with caching enabled
    let manager = GpuManager::with_cache_config(Duration::from_secs(2), 10);
    
    // Get all GPUs (owned copies)
    let gpus = manager.get_all_gpus_owned();
    println!("Found {} GPU(s)", gpus.len());
    
    for i in 0..gpus.len() {
        // Use cached access for better performance (zero-copy via Arc)
        if let Some(gpu) = manager.get_gpu_cached(i) {
            println!("GPU #{}: {} ({})", i, gpu.name_or_default(), gpu.vendor());
            println!("  Temperature: {}", gpu.display_temperature());
            println!("  Utilization: {}", gpu.display_utilization());
            println!("  Power Usage: {}", gpu.display_power_usage());
        }
    }
    
    // Get primary GPU with caching (zero-copy)
    if let Some(primary_gpu) = manager.get_primary_gpu_cached() {
        println!("Primary GPU: {}", primary_gpu.display_name_gpu());
    }
    Ok(())
}
```

### Collection Traits

`GpuManager` implements standard collection traits for flexible construction and extension:

```rust
use gpu_info::{GpuManager, GpuInfo};
use std::iter::FromIterator;

// Create manager from an iterator of GpuInfo
let gpus = vec![GpuInfo::mock_nvidia(), GpuInfo::mock_amd()];
let manager = GpuManager::from_iter(gpus);
assert_eq!(manager.gpu_count(), 2);

// Or use collect() for more idiomatic Rust
let manager: GpuManager = vec![
    GpuInfo::mock_nvidia(),
    GpuInfo::mock_intel(),
].into_iter().collect();

// Extend an existing manager with more GPUs
let mut manager = GpuManager::from_iter(vec![GpuInfo::mock_nvidia()]);
manager.extend(vec![GpuInfo::mock_amd(), GpuInfo::mock_intel()]);
assert_eq!(manager.gpu_count(), 3);
```

This is particularly useful for:
- Testing with mock GPU data
- Loading GPU configurations from files
- Combining GPUs from different detection sources
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
        println!("GPU: {} ({})", gpu.name_or_default(), gpu.vendor());
    }
    Ok(())
}
```

### Using Async API

```rust
use gpu_info::{get_async, get_all_async};
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    let gpu = get_async().await?;
    info!("Primary GPU: {}", gpu.display_name_gpu());
    info!("Temperature: {}", gpu.display_temperature());
    info!("Utilization: {}", gpu.display_utilization());
    
    let gpus = get_all_async().await?;
    info!("Found {} GPU(s)", gpus.len());
    for (i, gpu) in gpus.iter().enumerate() {
        info!("GPU {}: {}", i, gpu.display_name_gpu());
        info!("  Temperature: {}", gpu.display_temperature());
        info!("  Utilization: {}", gpu.display_utilization());
        info!("  Power Usage: {}", gpu.display_power_usage());
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

# With logging enabled:
RUST_LOG=info cargo run --example async_example
```

## Dependencies

- Windows: NVIDIA NVML, AMD ADL, or Intel WMI
- Linux: NVIDIA NVML or AMD ADL
- macOS: Apple Metal API (support currently suspended)

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.