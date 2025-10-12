//! Legacy Linux GPU detection implementation
//!
//! This module provides backward compatibility for the main `get()` API function.
//! Now refactored to use the modern provider system internally.
//!
//! For full multi-GPU support, use the modern provider system directly:
//! - `crate::providers::linux::NvidiaLinuxProvider`
//! - `crate::providers::linux::AmdLinuxProvider`
//!
//! # Deprecation Notice
//!
//! This legacy API is maintained for backward compatibility but internally delegates
//! to the provider system. Consider using `GpuManager` or providers directly.
use crate::{
    gpu_info::{GpuInfo, GpuProvider},
    providers::linux::{AmdLinuxProvider, IntelLinuxProvider, NvidiaLinuxProvider},
    vendor::Vendor,
};
use log::{debug, warn};
use std::{fs, path::Path};
/// Detect GPU vendor by reading sysfs
///
/// Returns the detected vendor or Vendor::Unknown if detection fails.
/// This function reads `/sys/class/drm/card0/device/vendor` to identify the GPU vendor.
fn detect_vendor() -> Vendor {
    let vendor_path = Path::new("/sys/class/drm/card0/device/vendor");

    if let Ok(vendor_id) = fs::read_to_string(vendor_path) {
        let vendor_id = vendor_id.trim();
        match vendor_id {
            "0x10de" => return Vendor::Nvidia,
            "0x1002" => return Vendor::Amd,
            "0x8086" => return Vendor::Intel(crate::vendor::IntelGpuType::Unknown),
            _ => debug!("Unknown vendor ID: {}", vendor_id),
        }
    }

    // Fallback: check if NVIDIA libraries are available
    if Path::new("/usr/lib/libnvidia-ml.so.1").exists()
        || Path::new("/usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1").exists()
    {
        return Vendor::Nvidia;
    }

    Vendor::Unknown
}
/// Fetches detailed information about the primary GPU.
///
/// This function now uses the modern provider system internally to detect and query GPUs.
/// It automatically detects the GPU vendor and uses the appropriate provider.
///
/// # Returns
///
/// Returns a `GpuInfo` struct with information about the primary GPU, or a default
/// `GpuInfo` with `Vendor::Unknown` if no supported GPU is found.
///
/// # Note
///
/// For multi-GPU systems or more control, use `GpuManager` or providers directly.
/// This function only returns information about the first detected GPU.
///
/// # Example
///
/// ```no_run
/// use gpu_info::linux;
/// let gpu = linux::info_gpu();
/// println!("GPU: {:?}", gpu.name_gpu);
/// ```
pub fn info_gpu() -> GpuInfo {
    debug!("Fetching primary GPU info using provider system");

    let vendor = detect_vendor();
    debug!("Detected GPU vendor: {:?}", vendor);

    let gpus = match vendor {
        Vendor::Nvidia => {
            let provider = NvidiaLinuxProvider::new();
            provider.detect_gpus()
        }
        Vendor::Amd => {
            let provider = AmdLinuxProvider::new();
            provider.detect_gpus()
        }
        Vendor::Intel(_) => {
            let provider = IntelLinuxProvider::new();
            provider.detect_gpus()
        }
        _ => {
            warn!("No supported GPU vendor detected, returning default GpuInfo");
            return GpuInfo::default();
        }
    };

    match gpus {
        Ok(mut gpu_list) if !gpu_list.is_empty() => {
            debug!("Successfully detected {} GPU(s)", gpu_list.len());
            gpu_list.remove(0)
        }
        Ok(_) => {
            warn!("Provider detected 0 GPUs");
            GpuInfo::default()
        }
        Err(e) => {
            warn!("Failed to detect GPUs: {:?}", e);
            GpuInfo::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_vendor_returns_valid_vendor() {
        let vendor = detect_vendor();
        // Should return either a valid vendor or Unknown
        // We can't test exact value as it depends on the system
        assert!(matches!(
            vendor,
            Vendor::Nvidia | Vendor::Amd | Vendor::Intel(_) | Vendor::Unknown
        ));
    }

    #[test]
    fn test_info_gpu_returns_gpuinfo() {
        // Should always return GpuInfo, never panic
        let gpu = info_gpu();
        // Verify it's a valid GpuInfo struct
        assert!(matches!(
            gpu.vendor,
            Vendor::Nvidia | Vendor::Amd | Vendor::Intel(_) | Vendor::Unknown
        ));
    }

    #[test]
    fn test_info_gpu_does_not_panic_without_gpu() {
        // This test ensures backward compatibility
        // Even without GPU, should return default GpuInfo, not panic
        let _gpu = info_gpu();
        // If we reached here, no panic occurred
    }
}
