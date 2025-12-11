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

pub(crate) fn detect_vendor() -> Vendor {
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

    if Path::new("/usr/lib/libnvidia-ml.so.1").exists()
        || Path::new("/usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1").exists()
    {
        return Vendor::Nvidia;
    }

    Vendor::Unknown
}
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
