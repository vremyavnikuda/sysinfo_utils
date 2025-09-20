//! NVIDIA GPU provider implementation
//!
//! This module implements the GpuProvider trait for NVIDIA GPUs using the NVML API.
use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::nvml_api;
use crate::vendor::Vendor;
/// NVIDIA GPU provider
pub struct NvidiaProvider;
impl NvidiaProvider {
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
    /// Detect all NVIDIA GPUs in the system
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        let gpus = nvml_api::get_nvidia_gpus();
        crate::gpu_info::handle_empty_result(gpus)
    }
    /// Update the information for a specific NVIDIA GPU
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        crate::gpu_info::update_gpu_from_api(gpu, || nvml_api::get_nvidia_gpus())
    }
    /// Get the vendor for this provider
    fn get_vendor(&self) -> Vendor {
        Vendor::Nvidia
    }
}
// Backwards compatibility functions
pub fn detect_nvidia_gpus() -> Result<Vec<GpuInfo>> {
    let provider = NvidiaProvider::new();
    provider.detect_gpus()
}
pub fn update_nvidia_info(gpu: &mut GpuInfo) -> Result<()> {
    let provider = NvidiaProvider::new();
    provider.update_gpu(gpu)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nvidia_provider_vendor() {
        let provider = NvidiaProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Nvidia);
    }
}
