//! GPU information library for cross-platform GPU monitoring.
//!
//! This crate provides a unified API for querying GPU information across
//! different vendors (NVIDIA, AMD, Intel, Apple) and platforms (Windows, Linux, macOS).
//!
//! # Features
//!
//! - Multi-vendor support: NVIDIA (NVML), AMD (ADL), Intel (WMI/sysfs), Apple (Metal/IOKit)
//! - Real-time metrics: temperature, utilization, power, memory, clocks
//! - Caching with configurable TTL for performance optimization
//! - Async API via tokio for non-blocking operations
//! - Thread-safe global GPU manager
//!
//! # Quick Start
//!
//! ```no_run
//! use gpu_info::GpuInfo;
//!
//! // Get primary GPU info
//! let gpu = gpu_info::get();
//! println!("GPU: {} {}", gpu.vendor, gpu.name_gpu.unwrap_or_default());
//! ```

#![deny(missing_docs)]

pub use crate::gpu_info::{GpuError, GpuInfo, GpuInfoBuilder, Result};
pub use crate::metric_value::MetricValue;

/// Macros for GPU information formatting and display.
///
/// This module provides utility macros for consistent formatting of GPU metrics
/// and information across the crate.
#[macro_use]
pub mod macros;

/// AMD Display Library (ADL) FFI bindings.
///
/// This module provides low-level bindings to AMD's ADL library for querying
/// AMD/Radeon GPU information. The bindings are used internally by the AMD
/// provider implementation.
///
/// # Safety
///
/// This module contains unsafe FFI code. All unsafe operations are isolated
/// here and wrapped in safe abstractions by the provider layer.
pub mod adl_api;

/// Asynchronous GPU information API.
///
/// This module provides async versions of GPU query functions using tokio.
/// Use these functions in async contexts to avoid blocking the runtime.
///
/// # Examples
///
/// ```no_run
/// use gpu_info::get_async;
///
/// #[tokio::main]
/// async fn main() -> gpu_info::Result<()> {
///     let gpu = get_async().await?;
///     println!("GPU: {}", gpu);
///     Ok(())
/// }
/// ```
pub mod async_api;

/// GPU information caching utilities.
///
/// This module provides caching infrastructure for GPU metrics with
/// configurable TTL (time-to-live) and LRU eviction policies.
/// Caching significantly improves performance for frequent GPU queries.
pub mod cache_utils;

/// Extended GPU information and capabilities.
///
/// This module provides additional GPU information beyond basic metrics,
/// including hardware capabilities, feature support, and detailed specifications.
pub mod extended_info;

/// FFI utility functions and types.
///
/// This module provides common utilities for FFI operations including
/// string conversion, pointer handling, and error mapping.
///
/// # Safety
///
/// This module contains unsafe FFI helper code. All unsafe operations
/// are documented with safety invariants.
pub mod ffi_utils;

/// Core GPU information types and traits.
///
/// This module contains the primary [`GpuInfo`] struct, [`GpuError`] enum,
/// [`GpuInfoBuilder`] for constructing GPU info, and the `GpuProvider` trait
/// for vendor-specific implementations.
///
/// [`GpuInfo`]: crate::GpuInfo
/// [`GpuError`]: crate::GpuError
/// [`GpuInfoBuilder`]: crate::GpuInfoBuilder
pub mod gpu_info;

/// GPU manager for multi-GPU systems.
///
/// This module provides [`GpuManager`] for managing multiple GPUs with
/// caching, automatic detection, and unified access patterns.
///
/// [`GpuManager`]: crate::GpuManager
pub mod gpu_manager;

/// Metric value types for GPU measurements.
///
/// This module provides the [`MetricValue`] enum for representing
/// different types of GPU metrics (temperature, utilization, power, etc.)
/// with appropriate units and formatting.
///
/// [`MetricValue`]: crate::MetricValue
pub mod metric_value;

/// GPU monitoring and alerting.
///
/// This module provides real-time GPU monitoring with configurable
/// thresholds and alert callbacks. Use [`GpuMonitor`] to track GPU
/// metrics and receive notifications when thresholds are exceeded.
///
/// [`GpuMonitor`]: crate::GpuMonitor
pub mod monitoring;

/// NVIDIA Management Library (NVML) FFI bindings.
///
/// This module provides low-level bindings to NVIDIA's NVML library for
/// querying NVIDIA GPU information. The bindings are used internally by
/// the NVIDIA provider implementation.
///
/// # Safety
///
/// This module contains unsafe FFI code. All unsafe operations are isolated
/// here and wrapped in safe abstractions by the provider layer.
pub mod nvml_api;

/// GPU provider management.
///
/// This module provides [`GpuProviderManager`] for managing vendor-specific
/// GPU providers and coordinating GPU detection across different vendors.
///
/// [`GpuProviderManager`]: crate::GpuProviderManager
pub mod provider_manager;

/// Vendor-specific GPU provider implementations.
///
/// This module contains provider implementations for different GPU vendors
/// (NVIDIA, AMD, Intel, Apple) and platforms (Windows, Linux, macOS).
pub mod providers;

/// Fluent query API for filtering GPUs.
///
/// This module provides [`GpuQuery`] for building fluent queries to filter
/// and select GPUs based on various criteria like vendor, temperature,
/// utilization, and more.
///
/// [`GpuQuery`]: crate::GpuQuery
pub mod query;

/// Fallback implementation for unknown platforms.
///
/// This module provides a fallback implementation that returns
/// `GpuInfo::unknown()` for platforms without native GPU support.
pub mod unknown;

/// GPU vendor identification.
///
/// This module provides the [`Vendor`] enum for identifying GPU manufacturers
/// (NVIDIA, AMD, Intel, Apple) and related types like [`IntelGpuType`].
///
/// [`Vendor`]: crate::Vendor
/// [`IntelGpuType`]: crate::vendor::IntelGpuType
pub mod vendor;
pub use async_api::{
    get_all_async, get_all_async_owned, get_async, get_async_owned, update_gpu_async,
};
pub use extended_info::{ExtendedGpuInfo, GpuInfoExtensions};
pub use gpu_manager::{GpuManager, GpuStatistics};
pub use monitoring::{AlertType, GpuMonitor, GpuThresholds, MonitorConfig};
pub use provider_manager::GpuProviderManager;
pub use query::GpuQuery;
pub use vendor::Vendor;

/// Windows platform implementation.
///
/// This module provides GPU detection and metrics collection for Windows
/// using WMI, PDH counters, and vendor-specific APIs (NVML, ADL).
#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
pub mod imp;
#[cfg(target_os = "windows")]
pub use imp as windows;

/// macOS platform implementation.
///
/// This module provides GPU detection and metrics collection for macOS
/// using IOKit, Metal, and system profiler APIs.
#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
pub mod imp;
#[cfg(target_os = "macos")]
pub use imp as macos;

/// Linux platform implementation.
///
/// This module provides GPU detection and metrics collection for Linux
/// using sysfs, hwmon, and vendor-specific APIs (NVML, ADL).
#[allow(missing_debug_implementations, missing_docs, unsafe_code)]
#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
pub mod imp;
#[cfg(target_os = "linux")]
pub use imp as linux;

/// Fallback platform implementation for unsupported operating systems.
///
/// This module provides a no-op implementation that returns unknown GPU info
/// for platforms that are not explicitly supported.
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
