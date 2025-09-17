//! NVIDIA GPU detection using the new NVML abstraction
//! 
//! This module has been refactored to use the common FFI utilities,
//! eliminating code duplication and improving maintainability.

use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::nvml_api;
use log::error;

/// Detect NVIDIA GPUs using the new NVML abstraction
/// 
/// This function now uses the common FFI utilities to reduce code duplication
/// and provides better error handling and resource management.
pub fn detect_nvidia_gpus() -> Result<Vec<GpuInfo>> {
    let gpus = nvml_api::get_nvidia_gpus();
    if gpus.is_empty() {
        Err(GpuError::GpuNotFound)
    } else {
        Ok(gpus)
    }
}

/// Update NVIDIA GPU information
/// 
/// This function updates the GPU information for an existing NVIDIA GPU
/// using the new NVML abstraction.
pub fn update_nvidia_info(gpu: &mut GpuInfo) -> Result<()> {
    let gpus = nvml_api::get_nvidia_gpus();
    if let Some(updated_gpu) = gpus.first() {
        *gpu = updated_gpu.clone();
        Ok(())
    } else {
        Err(GpuError::GpuNotActive)
    }
}

/// Legacy function for backwards compatibility
/// 
/// This function is kept for backwards compatibility but now delegates
/// to the new NVML client implementation.
pub fn load_local_nvml() -> bool {
    // The NVML client now handles library loading internally
    // with proper fallback paths and error handling
    match crate::nvml_api::NvmlClient::new() {
        Some(_) => true,
        None => {
            error!("Failed to load NVML library");
            false
        }
    }
}
