//gpu_info/src/mode/gpu.rs
//! GPU monitoring and management implementation
//!
//! Provides cross-platform GPU monitoring capabilities through vendor-specific interfaces:
//! - NVIDIA GPUs via `nvidia-smi`
//! - AMD GPUs via SASS interface
//! - Intel integrated graphics via sysfs
//!
//! # Features
//! - Real-time metrics collection (temperature, utilization, clock speed, power usage)
//! - Multi-GPU support
//! - Waybar integration
//! - Power state monitoring
//! - Vendor-specific data collection
//!
//! # Platform Support
//! - Linux: Full support for all vendors
//! - Windows: Partial support (NVIDIA only)
//! - macOS: Not currently supported
use serde::{Deserialize, Serialize};
/// Enum representing different GPU vendors.
///
/// Represents GPU hardware vendors supported by this library.
///
/// # Variants
/// - `Nvidia`: NVIDIA Corporation devices
/// - `AMD`: Advanced Micro Devices (AMD) devices
/// - `Intel`: Intel Corporation integrated graphics
/// - `Unknown`: Unknown or unsupported GPU vendor
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enum representing different GPU vendors.
pub enum GpuVendor {
    /// nvidia Corporation devices
    Nvidia,
    /// Advanced Micro Devices (amd) devices
    AMD,
    /// Intel Corporation integrated graphics
    Intel,
    /// Unknown or unsupported GPU vendor
    Unknown,
}

impl Default for GpuVendor {
    /// Returns `GpuVendor::Unknown` as the default variant.
    fn default() -> Self {
        GpuVendor::Unknown
    }
}

/// Primary structure representing GPU metrics and status.
///
/// Contains detailed information about a graphics processing unit, including real-time metrics
/// and vendor identification.
///
/// # Fields
/// - `name`: GPU model name (e.g., "NVIDIA GeForce RTX 4090")
/// - `vendor`: Hardware vendor classification (`GpuVendor`)
/// - `temperature`: Current temperature in Celsius (optional)
/// - `utilization`: GPU utilization percentage (0-100, optional)
/// - `clock_speed`: Current clock speed in MHz (optional)
/// - `max_clock_speed`: Maximum supported clock speed in MHz (optional)
/// - `power_usage`: Current power draw in watts (optional)
/// - `max_power_usage`: Maximum power limit in watts (optional)
/// - `is_active`: Indicates if the GPU is currently active
///
/// # Examples
/// ```rust
///use log::error;
///use gpu_info::GpuManager;
///let mut manager = GpuManager::new();
///manager.refresh();
///
///if let Some(gpu) = manager.gpus.first() {
///    println!("{:?}",gpu.vendor_gpu());
///    println!("{}", gpu.name_gpu());
///    println!("{}", gpu.format_get_temperature_gpu());
///    println!("{}", gpu.format_get_utilization_gpu());
///    println!("{}", gpu.format_get_power_usage_gpu());
///    println!("{}", gpu.format_get_clock_speed_gpu());
///} else {
///    error!("No GPUs detected.");
///}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    /// GPU model name (e.g., "nvidia GeForce RTX 4090")
    pub name: String,
    /// Hardware vendor classification
    pub vendor: GpuVendor,
    /// Current temperature in Celsius (if available)
    pub temperature: Option<f32>,
    /// GPU utilization percentage (0-100)
    pub utilization: Option<f32>,
    /// Current clock speed in MHz
    pub clock_speed: Option<u64>,
    /// Maximum supported clock speed in MHz
    pub max_clock_speed: Option<u64>,
    /// Current power draw in watts
    pub power_usage: Option<f32>,
    /// Maximum power limit in watts
    pub max_power_usage: Option<f32>,
    /// Indicates if the GPU is currently active
    pub is_active: bool,
}

impl GpuInfo {
    /// # Returns
    /// Returns the GPU model name as a string slice
    ///
    /// # Basic usage with detected GPU
    /// ```
    /// use gpu_info::GpuManager;
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{:?}", gpu.name_gpu())
    /// }else{
    ///     error!("No get current GPU name");
    /// }
    /// ```
    pub fn name_gpu(&self) -> &str {
        &self.name
    }
    /// Returns a reference to the GPU vendor enumeration
    ///
    /// # Returns
    /// - Reference to the `GpuVendor` enum representing the GPU's manufacturer
    ///
    /// # Basic usage with detected GPU
    /// ```
    /// use gpu_info::GpuManager;
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{:?}",gpu.vendor_gpu())
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    pub fn vendor_gpu(&self) -> &GpuVendor {
        &self.vendor
    }
    /// Returns the current temperature of the GPU in Celsius
    ///
    /// # Returns
    /// - `Option<f32>`: The temperature in Celsius if available, otherwise `None`.
    ///
    /// # Examples
    /// ```
    /// use gpu_info::GpuManager;
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{:?}",gpu.temperature_gpu())
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    pub fn temperature_gpu(&self) -> Option<f32> {
        self.temperature
    }

    /// Formats GPU temperature with appropriate units and precision
    ///
    /// # Examples
    ///
    /// ## Basic usage with detected GPU
    /// ```
    /// use gpu_info::GpuManager;
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("GPU Temperature: {}", gpu.format_get_temperature_gpu());
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    /// # Returns
    /// Formatted string representing GPU temperature in one of these formats:
    /// - "Temperature: XX°C" when data is available (shows decimal places if present)
    /// - "Temperature: N/A" when temperature data is unavailable
    pub fn format_get_temperature_gpu(&self) -> String {
        match self.temperature {
            Some(temp) => format!("Temperature: {}°C", temp),
            None => "Temperature: N/A".to_string(),
        }
    }

    /// Returns the current operating clock speed of the GPU in MHz
    ///
    /// # Examples
    ///
    /// ## Basic usage with detected GPU
    /// ```
    /// use log::warn;
    /// use gpu_info::GpuManager;
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{}",gpu.format_clock_speed_gpu())
    /// } else {
    ///     warn!("No GPUs detected");
    /// }
    /// ```
    ///
    /// # Returns
    /// - `Some(u64)`: The current clock speed in MHz if monitoring is available
    /// - `None`: If clock speed monitoring is unsupported or unavailable
    ///
    /// # Notes
    /// Represents real-time clock speed measurement
    /// Represents core clock speed (not memory clock)
    /// - For accurate readings:
    ///   - Call after `refresh()` for current values
    /// - Compare with `format_max_clock_speed_gpu()` to determine performance headroom
    pub fn format_clock_speed_gpu(&self) -> Option<u64> {
        self.clock_speed
    }

    /// Returns the current operating clock speed of the GPU in MHz
    ///
    /// # Returns
    /// - `u64`: The current clock speed in MHz if monitoring is available
    /// - `None`: If clock speed monitoring is unsupported or unavailable
    ///
    /// # Examples
    /// ```
    /// use log::{error};
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{}",gpu.clock_speed_gpu())
    /// }else {
    ///     error!("No get current clock ");
    /// }
    /// ```
    /// # Notes
    /// Represents real-time clock speed measurement
    /// Represents core clock speed (not memory clock)
    /// - For accurate readings:
    ///   - Call after `refresh()` for current values
    /// - Compare with `max_clock_speed_gpu()` to determine performance headroom
    pub fn clock_speed_gpu(&self) -> u64 {
        self.clock_speed.unwrap_or(0)
    }

    /// Returns the maximum supported clock speed of the GPU in MHz
    ///
    /// # Examples
    ///
    /// ## Basic usage with detected GPU
    /// ```
    /// use gpu_info::GpuManager;
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{}",gpu.max_clock_speed())
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    ///
    /// # Returns
    /// - `Some(u64)`: The maximum supported clock speed in MHz if available
    /// - `None`: If clock speed information is unavailable
    ///
    /// # Notes
    /// - Represents the manufacturer-specified maximum boost clock speed
    /// - Measurement notes:
    ///   - Represents maximum potential speed, not current speed
    ///   - For current speed, use `format_clock_speed_gpu()`
    ///   - Values are rounded to nearest MHz by most vendors
    pub fn format_max_clock_speed_gpu(&self) -> Option<u64> {
        self.max_clock_speed
    }

    /// Returns the maximum supported clock speed of the GPU in MHz.
    ///
    /// # Examples
    /// ```
    /// use log::{error};
    /// use gpu_info::GpuManager;
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{:?}", gpu.max_clock_speed());
    /// }else {
    ///     error!("No get current max clock speed GPU");
    /// }
    /// ```
    /// # Returns
    /// - `u64`: The maximum supported clock speed in MHz. If the information is unavailable, returns errors.
    ///
    /// # Notes
    /// - Represents the manufacturer-specified maximum boost clock speed.
    /// - The returned value is an unwrapped value from the `max_clock_speed` field.
    /// - For accurate readings, ensure that you call `refresh()` to update the GPU information.
    pub fn max_clock_speed(&self) -> u64 {
        self.max_clock_speed.unwrap_or(0)
    }

    /// Returns the current power consumption of the GPU in watts
    ///
    /// # Examples
    ///
    /// ## Basic usage with detected GPU
    /// ```
    /// use gpu_info::GpuManager;
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{}", gpu.format_get_power_usage_gpu());
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    /// # Returns
    /// - Format `String``.2`: The current power consumption in watts if monitoring is available
    /// - `N/A`: If power monitoring is unsupported or unavailable
    ///
    /// # Notes
    /// - Represents real-time power consumption, not the maximum capability
    /// - Measurement characteristics:
    ///   - Update frequency varies by vendor (typically 100ms-1s)
    ///   - Accuracy typically ±5% of actual power
    ///   - May show brief power spikes above TDP during load changes
    pub fn format_power_usage_gpu(&self) -> String {
        self.power_usage
            .map_or(String::from("N/A"), |v| format!("{:.2}", v))
    }

    /// Returns the current power consumption of the GPU in watts.
    /// # Examples
    /// ```
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// let current_power = manager.gpus.first().power_usage_gpu();
    /// println!("{}", current_power);
    /// ```
    /// # Returns
    /// - `f32`: The current power consumption in watts if available, otherwise returns 0.0.
    ///
    /// # Notes
    /// - Represents real-time power consumption.
    /// - For accurate readings, ensure the GPU information is refreshed `.refresh()`.

    pub fn power_usage_gpu(&self) -> f32 {
        self.power_usage.unwrap_or(0.00)
    }

    /// Returns the maximum power draw limit of the GPU in watts
    ///
    /// # Examples
    ///
    /// ## Basic usage with detected GPU
    /// ```
    /// use log::error;
    /// use gpu_info::GpuManager;
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("Max power limit: {}", gpu.format_max_power_usage_gpu());
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    /// # Returns
    /// - `Some(f32)`: The maximum power draw limit in watts if available
    /// - `None`: If power limit information is unavailable
    ///
    /// # Notes
    /// - Represents the manufacturer-set maximum power limit (TDP - Thermal Design Power)
    /// - Actual power draw may be lower depending on workload
    /// - Values may vary by ±5% due to manufacturing tolerances
    pub fn format_max_power_usage_gpu(&self) -> Option<f32> {
        self.max_power_usage
    }

    /// Checks whether the GPU is currently in an active state
    ///
    /// # Examples
    ///
    /// ## Basic usage with detected GPU
    /// ```
    /// use log::error;
    /// use gpu_info::GpuManager;
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{}", gpu.bool_is_active_gpu());
    /// }else {
    ///     error!("No GPUs detected");
    /// }
    /// manager.refresh();
    /// ```
    /// # Returns
    /// - `true` if the GPU is currently active
    /// - `false` if the GPU is inactive or in low-power state
    ///
    /// # Notes
    /// - Activation state detection varies by vendor:
    ///   - **NVIDIA**: Active when GPU is in use (not in power-saving mode)
    /// - May return `true` during brief periods of inactivity if GPU hasn't entered power-saving mode
    /// - Some systems may show all GPUs as active when multi-GPU features are enabled
    /// - For accurate results, call after `refresh()` to get current state
    /// - Inactive state doesn't necessarily mean powered off - may be in low-power mode
    pub fn bool_is_active_gpu(&self) -> bool {
        self.is_active
    }

    /// Formats the GPU activation state as a String
    ///
    /// # Examples
    /// ```
    /// use log::error;
    /// use gpu_info::GpuManager;
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first(){
    ///     println!("{}", gpu.format_is_active_gpu());
    /// }else{
    ///     error!("GPU is inactive");
    /// }
    /// ```
    /// # Returns
    /// Returns the formatted result as a string
    pub fn format_is_active_gpu(&self) -> String {
        self.is_active
            .then(|| "Active".to_string())
            .unwrap_or_else(|| "Inactive".to_string())
    }

    /// Formats GPU utilization percentage with appropriate units and precision
    ///
    /// # Examples
    ///
    /// ## Basic usage with detected GPU
    /// ```
    /// use gpu_info::GpuManager;
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    ///manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("GPU Utilization: {}", gpu.format_get_utilization_gpu());
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    /// # Returns
    /// Formatted string representing GPU utilization in one of these formats:
    /// - "Utilization: XX.X%" when data is available (shows decimal places if present)
    /// - "Utilization: N/A" when utilization data is unavailable
    ///
    /// # Notes
    /// - Utilization represents the percentage of GPU processing power being used
    pub fn format_get_utilization_gpu(&self) -> String {
        match self.utilization {
            Some(util) => format!("Utilization: {}%", util),
            None => "Utilization: N/A".to_string(),
        }
    }

    /// Returns the current utilization of the GPU as a string
    ///
    /// # Examples
    ///```
    ///use log::{error, warn};
    /// use gpu_info::GpuManager;
    /// let mut manager =  GpuManager::new();
    ///manager.refresh();
    ///
    ///if let Some(gpu) = manager.gpus.first() {
    ///    println!("{}",gpu.utilization_gpu());
    ///}else {
    ///    error!("No get current utilization of the GPU");
    ///}
    ///```
    /// ## Basic usage with detected GPU
    ///
    pub fn utilization_gpu(&self) -> f32 {
        self.utilization.unwrap_or(0.00)
    }

    /// Formats GPU clock speeds with appropriate units and precision
    ///
    /// # Examples
    ///
    /// ## Example with detected GPU Current clock speed
    /// ```
    /// use gpu_info::GpuManager;
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("Current clock speed: {}", gpu.format_get_clock_speed_gpu());
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    /// # Returns
    /// Formatted string representing clock speeds in one of following formats:
    /// - "Clock Speed: CURRENT/MAX MHz" when both values are available
    /// - "Clock Speed: N/A" when either current or max clock speed is unavailable
    ///
    /// # Notes
    /// - Values are displayed in megahertz (MHz)
    /// - Uses 0 as fallback value internally but displays N/A to user
    /// - For accurate readings, ensure GPU drivers are properly installed
    pub fn format_get_clock_speed_gpu(&self) -> String {
        let current = self.clock_speed.unwrap_or(0);
        let max = self.max_clock_speed.unwrap_or(0);
        match (self.clock_speed, self.max_clock_speed) {
            (Some(_), Some(_)) => format!("Clock Speed: {}/{} MHz", current, max),
            _ => "Clock Speed: N/A".to_string(),
        }
    }

    /// Formats GPU power usage with appropriate units and precision
    ///
    /// # Examples
    ///
    /// Basic usage with detected GPU:
    /// ```
    /// use gpu_info::{GpuManager, GpuInfo};
    /// use log::{error};
    ///
    /// let mut manager = GpuManager::new();
    /// manager.refresh();
    ///
    /// if let Some(gpu) = manager.gpus.first() {
    ///     println!("{}", gpu.format_get_power_usage_gpu());
    /// } else {
    ///     error!("No GPUs detected");
    /// }
    /// ```
    /// # Returns
    /// Formatted string representing power usage in one of following formats:
    /// - "Power Usage: CURRENT/MAX W" when both current and max power are available
    ///   - CURRENT: formatted with 2 decimal places
    ///   - MAX: formatted as whole number
    /// - "Power Usage: N/A" when power data is unavailable
    ///
    /// # Notes
    /// - Values are displayed in watts (W)
    /// - Uses 0.0 as fallback value for display purposes when data is missing
    pub fn format_get_power_usage_gpu(&self) -> String {
        let current = self.power_usage.unwrap_or(0.0);
        let max = self.max_power_usage.unwrap_or(0.0);
        match (self.power_usage, self.max_power_usage) {
            (Some(_), Some(_)) => format!("Power Usage: {:.2}/{} W", current, max),
            _ => "Power Usage: N/A".to_string(),
        }
    }
}
