//gpu_info/src/windows/mod.rs
use crate::gpu_info::GpuInfo;
use crate::vendor::Vendor;
use log::{error, info};
use std::process::Command;

mod amd;
/// lib detect discrete gpu
mod intel;
mod nvidia;

fn detect_gpu_vendor() -> Option<Vendor> {
    let output = Command::new("powershell")
        .args(&[
            "Get-WmiObject",
            "Win32_VideoController",
            "|",
            "Select-Object",
            "Name",
            "|",
            "Format-List",
        ])
        .output()
        .ok()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    if output_str.contains("NVIDIA") {
        info!("Detected NVIDIA GPU");
        Some(Vendor::Nvidia)
    } else if output_str.contains("AMD") || output_str.contains("Radeon") {
        info!("Detected AMD GPU");
        Some(Vendor::Amd)
    } else if output_str.contains("Intel") {
        info!("Detected Intel GPU");
        Some(Vendor::Intel(crate::vendor::IntelGpuType::Integrated))
    } else {
        info!("Unknown GPU vendor");
        None
    }
}

/// Returns information about the GPU.
/// Automatically detects GPU vendor and returns appropriate information.
pub fn info_gpu() -> GpuInfo {
    match detect_gpu_vendor() {
        Some(Vendor::Nvidia) => match nvidia::detect_nvidia_gpus() {
            Ok(nvidia_gpus) if !nvidia_gpus.is_empty() => {
                let mut gpu = nvidia_gpus[0].clone();
                if nvidia::update_nvidia_info(&mut gpu).is_ok() {
                    return gpu;
                }
            }
            _ => {
                error!("Failed to get NVIDIA GPU information");
            }
        },
        Some(Vendor::Amd) => {
            let amd_gpus = amd::detect_amd_gpus();
            if !amd_gpus.is_empty() {
                let mut gpu = amd_gpus[0].clone();
                if amd::update_amd_info(&mut gpu).is_ok() {
                    return gpu;
                }
            }
        }
        Some(Vendor::Intel(_)) => {
            let intel_gpu = intel::detect_intel_gpus();
            if !intel_gpu.is_empty() {
                let mut gpu = intel_gpu[0].clone();
                if intel::update_intel_info(&mut gpu).is_ok() {
                    return gpu;
                }
            }
        }
        _ => {
            error!("No supported GPU detected");
        }
    }

    error!("Failed to get GPU information");
    GpuInfo::unknown()
}
