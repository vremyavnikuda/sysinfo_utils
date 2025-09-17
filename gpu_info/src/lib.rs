pub use crate::gpu_info::{GpuError, GpuInfo, Result};

pub mod gpu_info;
pub mod gpu_manager;
pub mod extended_info;
pub mod monitoring;
pub mod test_provider;
pub mod integration_tests;
pub mod demo_test;
pub mod linux;
pub mod macos;
pub mod unknown;
pub mod vendor;
pub mod windows;
pub mod ffi_utils;
pub mod nvml_api;
pub mod adl_api;
pub use gpu_manager::{GpuManager, GpuStatistics};
pub use extended_info::{ExtendedGpuInfo, GpuInfoExtensions};
pub use monitoring::{GpuMonitor, MonitorConfig, GpuThresholds, AlertType};

#[allow(missing_debug_implementations, missing_docs, unsafe_code)]
#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
pub mod linux;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
pub mod imp;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
pub mod imp;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
#[path = "unknown/mod.rs"]
pub mod imp;
pub mod test;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
/// Gets the `GpuInfo` of the primary GPU in the system.
///
/// If the system does not have a GPU, or the GPU is not supported, this returns
/// a default `GpuInfo` instance with unknown values.
///
/// # Linux and macOS
///
/// This function is supported on Linux and macOS.
///
/// # Windows
///
/// This function is supported on Windows.
///
/// # Other platforms
///
/// This function is not supported on other platforms.
pub fn get() -> GpuInfo {
    imp::info_gpu()
}

/// Enhanced API: Get all available GPUs in the system
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub fn get_all() -> Vec<GpuInfo> {
    gpu_manager::get_all_gpus()
}

/// Enhanced API: Get GPU count
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub fn get_count() -> usize {
    gpu_manager::get_gpu_count()
}

/// Enhanced API: Get primary GPU with caching
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub fn get_primary() -> Option<GpuInfo> {
    gpu_manager::get_primary_gpu()
}

/// Testing API: Get test GPUs for development and testing
#[cfg(test)]
pub fn get_test_gpus() -> Result<Vec<GpuInfo>> {
    let provider = test_provider::create_test_provider();
    provider.detect_test_gpus()
}
