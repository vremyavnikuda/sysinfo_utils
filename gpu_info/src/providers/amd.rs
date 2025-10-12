//! AMD GPU provider implementation
//!
//! This module implements the GpuProvider trait for AMD GPUs using the ADL API.
use crate::adl_api;
use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::vendor::Vendor;
/// AMD GPU provider
pub struct AmdProvider;
impl AmdProvider {
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
    /// Detect all AMD GPUs in the system
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        let gpus = adl_api::get_amd_gpus();
        crate::gpu_info::handle_empty_result(gpus)
    }
    /// Update the information for a specific AMD GPU
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        crate::gpu_info::update_gpu_from_api(gpu, adl_api::get_amd_gpus)
    }
    /// Get the vendor for this provider
    fn get_vendor(&self) -> Vendor {
        Vendor::Amd
    }
}
// Backwards compatibility functions
pub fn detect_amd_gpus() -> Result<Vec<GpuInfo>> {
    let provider = AmdProvider::new();
    provider.detect_gpus()
}
pub fn update_amd_info(gpu: &mut GpuInfo) -> Result<()> {
    let provider = AmdProvider::new();
    provider.update_gpu(gpu)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_amd_provider_vendor() {
        let provider = AmdProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Amd);
    }
}
