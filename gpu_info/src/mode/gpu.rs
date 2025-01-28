//! GPU monitoring and management implementation
//!
//! Provides cross-platform GPU monitoring capabilities through vendor-specific interfaces:
//! - NVIDIA GPUs via `nvidia-smi`
//! — AMD GPUs via sass interface
//! — Intel integrated graphics via sysfs.
//!
//! # Features
//! — Real-time metrics collection
//! — Multi-GPU support
//! — Waybar integration
//! — Power state monitoring
//! — Vendor-specific data collection.
// FIXME: Improve macOS and Windows support
//! # Platform Support
//! — Linux: Full support
//! — Windows: Partial NVIDIA support
//! — macOS: Not currently supported.
use serde::{Deserialize, Serialize};
/// Primary structure representing GPU metrics and status
///
/// Contains detailed information about a graphics processing unit,
/// including real-time metrics and vendor identification.
///
/// # Examples
/// ```
/// use gpu_info::mode::gpu::GpuVendor;
/// use gpu_info::GpuInfo;
///
/// let gpu = GpuInfo {
///     name: "RTX 4090".into(),
///     vendor: GpuVendor::Nvidia,
///     temperature: Some(65.0),
///     utilization: Some(45.5),
///     ..Default::default()
/// };
///
/// assert_eq!(gpu.get_utilization(), "Utilization: 45.5%");
/// ```
/// Represents GPU hardware vendors
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enum representing different GPU vendors.
pub enum GpuVendor {
    /// NVIDIA Corporation devices
    Nvidia,
    /// Advanced Micro Devices (AMD) devices
    AMD,
    /// Intel Corporation integrated graphics
    Intel,
    /// Unknown or unsupported GPU vendor
    Unknown,
}

impl Default for GpuVendor {
    /// Default vendor is Unknown
    fn default() -> Self {
        GpuVendor::Unknown
    }
}

/// Contains detailed metrics and information about a GPU device
///
/// # Examples
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    /// GPU model name (e.g., "NVIDIA GeForce RTX 4090")
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
    /// Returns the GPU model name as a string slice
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns reference to vendor classification
    pub fn get_vendor(&self) -> &GpuVendor {
        &self.vendor
    }

    /// Formats temperature with icon and units
    ///
    /// # Returns
    /// - Formatted string: "Temperature: XX°C"
    /// - "N/A" if temperature unavailable
    pub fn get_temperature(&self) -> String {
        match self.temperature {
            Some(temp) => format!(" Temperature: {}°C", temp),
            None => " Temperature: N/A".to_string(),
        }
    }

    /// Formats GPU utilization percentage with icon
    ///
    /// # Returns
    /// - Formatted string: "Utilization: XX%"
    /// - "N/A" if utilization data unavailable
    pub fn get_utilization(&self) -> String {
        match self.utilization {
            Some(util) => format!("󰾆 Utilization: {}%", util),
            None => "󰾆 Utilization: N/A".to_string(),
        }
    }

    /// Formats clock speeds with icon and units
    ///
    /// # Returns
    /// String in format "Clock Speed: CURRENT/MAX MHz"
    /// Uses 0 for missing values
    pub fn get_clock_speed(&self) -> String {
        let current = self.clock_speed.unwrap_or(0);
        let max = self.max_clock_speed.unwrap_or(0);
        format!(" Clock Speed: {}/{} MHz", current, max)
    }

    /// Formats power usage with icon and precision
    ///
    /// # Returns
    /// String in format "Power Usage: CURRENT/MAX W"
    /// - CURRENT: 2 decimal places
    /// - MAX: 0 decimal places
    pub fn get_power_usage(&self) -> String {
        let current = self.power_usage.unwrap_or(0.0);
        let max = self.max_power_usage.unwrap_or(0.0);
        format!("󱪉 Power Usage: {:.2}/{} W", current, max)
    }

    /// Indicates if the GPU is currently active
    ///
    /// # Note
    /// Activation state detection depends on vendor implementation
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}
