//! NVIDIA GPU provider implementation.
//!
//! This module implements the [`GpuProvider`] trait for NVIDIA GPUs using the NVML API.
//!
//! # Platform Support
//!
//! - Windows: Uses `nvml.dll` from NVIDIA driver installation
//! - Linux: Uses `libnvidia-ml.so` from NVIDIA driver installation
//!
//! # Requirements
//!
//! NVIDIA drivers must be installed for this provider to work.
//!
//! [`GpuProvider`]: crate::gpu_info::GpuProvider

use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::nvml_api;
use crate::vendor::Vendor;

/// NVIDIA GPU provider.
///
/// Implements [`GpuProvider`] for NVIDIA GPUs using the NVML (NVIDIA Management Library) API.
/// This provider can detect multiple NVIDIA GPUs and collect real-time metrics including
/// temperature, utilization, power usage, and memory information.
///
/// [`GpuProvider`]: crate::gpu_info::GpuProvider
pub struct NvidiaProvider;

impl NvidiaProvider {
    /// Create a new NVIDIA provider instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NvidiaProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuProvider for NvidiaProvider {
    /// Detect all NVIDIA GPUs in the system.
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        let gpus = nvml_api::get_nvidia_gpus();
        crate::gpu_info::handle_empty_result(gpus)
    }

    /// Update the information for a specific NVIDIA GPU.
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        crate::gpu_info::update_gpu_from_api(gpu, nvml_api::get_nvidia_gpus)
    }

    /// Get the vendor for this provider.
    fn get_vendor(&self) -> Vendor {
        Vendor::Nvidia
    }
}

/// Detect all NVIDIA GPUs in the system.
///
/// This is a convenience function that creates a temporary [`NvidiaProvider`]
/// and calls [`detect_gpus`](GpuProvider::detect_gpus).
///
/// # Errors
///
/// Returns an error if no NVIDIA GPUs are found or if NVML initialization fails.
pub fn detect_nvidia_gpus() -> Result<Vec<GpuInfo>> {
    let provider = NvidiaProvider::new();
    provider.detect_gpus()
}

/// Update GPU information for an NVIDIA GPU.
///
/// This is a convenience function that creates a temporary [`NvidiaProvider`]
/// and calls [`update_gpu`](GpuProvider::update_gpu).
///
/// # Errors
///
/// Returns an error if the GPU update fails.
pub fn update_nvidia_info(gpu: &mut GpuInfo) -> Result<()> {
    let provider = NvidiaProvider::new();
    provider.update_gpu(gpu)
}

// TODO: there should be no tests here. Transfer them to gpu_info\src\test
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nvidia_provider_vendor() {
        let provider = NvidiaProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Nvidia);
    }
}
