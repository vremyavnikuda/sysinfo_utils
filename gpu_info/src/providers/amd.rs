//! AMD GPU provider implementation.
//!
//! This module implements the `GpuProvider` trait for AMD GPUs using the ADL API.
//!
//! # Platform Support
//!
//! - Windows: Uses `atiadlxx.dll` from AMD driver installation
//! - Linux: Uses sysfs and hwmon interfaces (see `linux::AmdLinuxProvider`)
//!
//! # Requirements
//!
//! AMD Radeon drivers must be installed for this provider to work on Windows.

use crate::adl_api;
use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::vendor::Vendor;

/// AMD GPU provider.
///
/// Implements `GpuProvider` for AMD GPUs using the ADL (AMD Display Library) API.
/// This provider can detect multiple AMD GPUs and collect real-time metrics including
/// temperature, utilization, and clock speeds.
///
/// [`GpuProvider`]: crate::gpu_info::GpuProvider
pub struct AmdProvider;

impl AmdProvider {
    /// Create a new AMD provider instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AmdProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuProvider for AmdProvider {
    /// Detect all AMD GPUs in the system.
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        let gpus = adl_api::get_amd_gpus();
        crate::gpu_info::handle_empty_result(gpus)
    }

    /// Update the information for a specific AMD GPU.
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        crate::gpu_info::update_gpu_from_api(gpu, adl_api::get_amd_gpus)
    }

    /// Get the vendor for this provider.
    fn get_vendor(&self) -> Vendor {
        Vendor::Amd
    }
}

/// Detect all AMD GPUs in the system.
///
/// This is a convenience function that creates a temporary [`AmdProvider`]
/// and calls [`detect_gpus`](GpuProvider::detect_gpus).
///
/// # Errors
///
/// Returns an error if no AMD GPUs are found or if ADL initialization fails.
pub fn detect_amd_gpus() -> Result<Vec<GpuInfo>> {
    let provider = AmdProvider::new();
    provider.detect_gpus()
}

/// Update GPU information for an AMD GPU.
///
/// This is a convenience function that creates a temporary [`AmdProvider`]
/// and calls [`update_gpu`](GpuProvider::update_gpu).
///
/// # Errors
///
/// Returns an error if the GPU update fails.
pub fn update_amd_info(gpu: &mut GpuInfo) -> Result<()> {
    let provider = AmdProvider::new();
    provider.update_gpu(gpu)
}

// TODO: there should be no tests here. Transfer them to gpu_info\src\test
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_amd_provider_vendor() {
        let provider = AmdProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Amd);
    }
}
