pub use crate::gpu_info::{GpuError, GpuInfo, GpuInfoBuilder, Result};
#[macro_use]
pub mod macros;
pub mod adl_api;
pub mod async_api;
pub mod cache_utils;
pub mod extended_info;
pub mod ffi_utils;
pub mod gpu_info;
pub mod gpu_manager;
pub mod monitoring;
pub mod nvml_api;
pub mod provider_manager;
pub mod providers;
pub mod unknown;
pub mod vendor;
pub use async_api::{
    get_all_async, get_all_async_owned, get_async, get_async_owned, update_gpu_async,
};
pub use extended_info::{ExtendedGpuInfo, GpuInfoExtensions};
pub use gpu_manager::{GpuManager, GpuStatistics};
pub use monitoring::{AlertType, GpuMonitor, GpuThresholds, MonitorConfig};
pub use provider_manager::GpuProviderManager;

// Platform-specific implementations
#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
pub mod imp;
#[cfg(target_os = "windows")]
pub use imp as windows;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
pub mod imp;
#[cfg(target_os = "macos")]
pub use imp as macos;

#[allow(missing_debug_implementations, missing_docs, unsafe_code)]
#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
pub mod imp;
#[cfg(target_os = "linux")]
pub use imp as linux;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
#[path = "unknown/mod.rs"]
pub mod imp;
#[cfg(test)]
mod test;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
/// Gets information about the primary GPU in the system.
///
/// Returns a `GpuInfo` struct with GPU metrics including vendor, model name,
/// temperature, utilization, and power usage. If no GPU is detected or the
/// GPU is not supported, returns a `GpuInfo` with `Vendor::Unknown`.
///
/// # Examples
///
/// ```no_run
/// use gpu_info::get;
///
/// let gpu = get();
/// println!("GPU: {} {}", gpu.vendor, gpu.format_name_gpu());
/// println!("Temperature: {}°C", gpu.format_temperature());
/// println!("Utilization: {}%", gpu.format_utilization());
/// ```
///
/// # Platform Support
///
/// Supported on Windows, Linux, and macOS. On unsupported platforms,
/// this function is not available.
///
/// # Performance
///
/// This function performs direct FFI calls without caching. For frequent
/// polling, consider using [`GpuManager`] with caching enabled.
///
/// [`GpuManager`]: crate::GpuManager
pub fn get() -> GpuInfo {
    imp::info_gpu()
}
/// Gets information about all GPUs in the system.
///
/// Returns a vector of `GpuInfo` structs, one for each detected GPU.
/// If no GPUs are found, returns an empty vector.
///
/// # Examples
///
/// ```no_run
/// use gpu_info::get_all;
///
/// let gpus = get_all();
/// println!("Found {} GPU(s)", gpus.len());
///
/// for (i, gpu) in gpus.iter().enumerate() {
///     println!("GPU {}: {} ({})", i, gpu.format_name_gpu(), gpu.vendor);
/// }
/// ```
///
/// # Platform Support
///
/// Supported on Windows, Linux, and macOS. On unsupported platforms,
/// this function is not available.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub fn get_all() -> Vec<GpuInfo> {
    gpu_manager::get_all_gpus()
}

/// Gets the number of GPUs in the system.
///
/// Returns the count of detected GPUs. Returns 0 if no GPUs are found.
///
/// # Examples
///
/// ```no_run
/// use gpu_info::get_count;
///
/// let count = get_count();
/// println!("System has {} GPU(s)", count);
/// ```
///
/// # Platform Support
///
/// Supported on Windows, Linux, and macOS. On unsupported platforms,
/// this function is not available.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub fn get_count() -> usize {
    gpu_manager::get_gpu_count()
}

/// Gets information about the primary GPU with caching enabled.
///
/// Returns `Some(GpuInfo)` for the primary GPU, or `None` if no GPU is detected.
/// This function uses the global GPU manager with caching, making it more
/// efficient for repeated calls compared to [`get()`].
///
/// # Examples
///
/// ```no_run
/// use gpu_info::get_primary;
///
/// if let Some(gpu) = get_primary() {
///     println!("Primary GPU: {}", gpu.format_name_gpu());
/// } else {
///     println!("No GPU detected");
/// }
/// ```
///
/// # Platform Support
///
/// Supported on Windows, Linux, and macOS. On unsupported platforms,
/// this function is not available.
///
/// [`get()`]: crate::get
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub fn get_primary() -> Option<GpuInfo> {
    gpu_manager::get_primary_gpu()
}
