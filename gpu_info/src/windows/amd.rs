//! AMD GPU detection using the new ADL abstraction
//!
//! This module has been refactored to use the common FFI utilities,
//! eliminating code duplication and improving maintainability.
use crate::adl_api;
use crate::gpu_info::{GpuError, GpuInfo, Result};
/// Detect AMD GPUs using the new ADL abstraction
///
/// This function now uses the common FFI utilities to reduce code duplication
/// and provides better error handling and resource management.
pub fn detect_amd_gpus() -> Result<Vec<GpuInfo>> {
    let gpus = adl_api::get_amd_gpus();
    if gpus.is_empty() {
        Err(GpuError::GpuNotFound)
    } else {
        Ok(gpus)
    }
}
/// Update AMD GPU information
///
/// This function updates the GPU information for an existing AMD GPU
/// using the new ADL abstraction.
pub fn update_amd_info(gpu: &mut GpuInfo) -> Result<()> {
    let gpus = adl_api::get_amd_gpus();
    if let Some(updated_gpu) = gpus.first() {
        *gpu = updated_gpu.clone();
        Ok(())
    } else {
        Err(GpuError::GpuNotActive)
    }
}
