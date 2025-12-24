//! Intel GPU provider implementation.
//!
//! This module implements the [`GpuProvider`] trait for Intel GPUs using WMI queries.
//!
//! # Platform Support
//!
//! - Windows: Uses WMI (Win32_VideoController) for basic info, Intel MD API for metrics
//! - Linux: Uses sysfs interfaces (see [`linux::IntelLinuxProvider`])
//!
//! # GPU Types
//!
//! Intel GPUs are classified into:
//! - Integrated (UHD, Iris) - Uses shared system memory
//! - Discrete (Arc) - Has dedicated VRAM
//!
//! [`GpuProvider`]: crate::gpu_info::GpuProvider
//! [`linux::IntelLinuxProvider`]: crate::providers::linux::intel::IntelLinuxProvider

use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::vendor::{IntelGpuType, Vendor};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use std::process::Command;

/// Intel GPU provider.
///
/// Implements [`GpuProvider`] for Intel GPUs using WMI queries on Windows.
/// This provider can detect Intel integrated (UHD, Iris) and discrete (Arc) GPUs.
///
/// For integrated GPUs, memory is calculated as 50% of system RAM (shared memory model).
/// For discrete GPUs, dedicated VRAM is reported from WMI.
///
/// [`GpuProvider`]: crate::gpu_info::GpuProvider
pub struct IntelProvider;

impl IntelProvider {
    /// Create a new Intel provider instance.
    pub fn new() -> Self {
        Self
    }

    fn determine_intel_gpu_type(&self, name: &str) -> IntelGpuType {
        crate::vendor::determine_intel_gpu_type_from_name(name)
    }

    /// Get total memory for Intel GPU
    ///
    /// For Integrated GPUs: Returns 50% of system RAM (shared memory model)
    /// For Discrete GPUs: Returns AdapterRAM from WMI (dedicated memory)
    fn get_memory_total(&self, gpu_type: &IntelGpuType, wmi_output: &str) -> Option<u32> {
        match gpu_type {
            IntelGpuType::Integrated => {
                // Integrated GPU uses shared system memory (typically 50% of RAM)
                self.get_system_shared_memory()
            }
            IntelGpuType::Discrete | IntelGpuType::Unknown => {
                // Discrete GPU or unknown: use AdapterRAM from WMI
                wmi_output
                    .lines()
                    .find(|line| line.contains("AdapterRAM"))
                    .and_then(|line| {
                        let bytes = line.split(":").nth(1)?.trim().parse::<u64>().ok()?;
                        // Convert bytes to MB
                        Some((bytes / 1024 / 1024) as u32)
                    })
            }
        }
    }

    /// Get system shared memory available for integrated GPU
    ///
    /// Integrated Intel GPUs can use up to 50% of system RAM as shared memory.
    /// This matches what Windows Task Manager shows for integrated GPUs.
    fn get_system_shared_memory(&self) -> Option<u32> {
        let output = Command::new("powershell")
            .args([
                "Get-WmiObject",
                "Win32_ComputerSystem",
                "|",
                "Select-Object",
                "TotalPhysicalMemory",
                "|",
                "Format-List",
            ])
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        output_str
            .lines()
            .find(|line| line.contains("TotalPhysicalMemory"))
            .and_then(|line| {
                let bytes = line.split(":").nth(1)?.trim().parse::<u64>().ok()?;
                // Integrated GPU can use 50% of system RAM
                let shared_memory_mb = (bytes / 2 / 1024 / 1024) as u32;
                info!(
                    "System RAM: {} MB, Shared memory for iGPU: {} MB",
                    bytes / 1024 / 1024,
                    shared_memory_mb
                );
                Some(shared_memory_mb)
            })
    }
    fn get_intel_gpu_info(&self) -> Result<String> {
        let output = Command::new("powershell")
            .args([
                "Get-WmiObject",
                "Win32_VideoController",
                "|",
                "Where-Object",
                "{ $_.Name -like '*Intel*' }",
                "|",
                "Select-Object",
                "Name, AdapterRAM, DriverVersion, Status",
                "|",
                "Format-List",
            ])
            .output()
            .map_err(|e| {
                error!("Failed to execute PowerShell command: {}", e);
                crate::gpu_info::GpuError::DriverNotInstalled
            })?;
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
    fn parse_gpu_info(&self, output_str: &str) -> Option<GpuInfo> {
        let gpu_name = output_str
            .lines()
            .find(|line| line.contains("Name"))
            .map(|line| line.split(":").nth(1).unwrap_or("").trim().to_string())
            .unwrap_or_else(|| {
                warn!("Failed to get GPU name, using default");
                "Intel GPU".to_string()
            });
        let gpu_type = self.determine_intel_gpu_type(&gpu_name);
        let driver_version = output_str
            .lines()
            .find(|line| line.contains("DriverVersion"))
            .map(|line| line.split(":").nth(1).unwrap_or("").trim().to_string());
        let memory_total = self.get_memory_total(&gpu_type, output_str);

        // Note: WMI Win32_VideoController does not provide GPU clock speeds
        // CurrentRefreshRate/MaxRefreshRate are monitor refresh rates, not GPU clocks
        // GPU clock speeds are obtained from Intel Metrics Discovery API instead

        let status = output_str
            .lines()
            .find(|line| line.contains("Status"))
            .map(|line| line.split(":").nth(1).unwrap_or("").trim() == "OK");
        info!("Found Intel GPU: {} (Type: {:?})", gpu_name, gpu_type);
        if let Some(ver) = &driver_version {
            info!("Driver version: {}", ver);
        }
        if let Some(mem) = memory_total {
            info!("Total memory: {} MB", mem);
        }
        Some(GpuInfo {
            name_gpu: Some(gpu_name),
            vendor: Vendor::Intel(gpu_type),
            driver_version,
            memory_total,
            memory_used: None,
            // Will be set by Intel MD API
            core_clock: None,
            // Will be set by Intel MD API
            max_clock_speed: None,
            active: status,
            temperature: None,
            utilization: None,
            power_usage: None,
            power_limit: None,
            memory_util: None,
            memory_clock: None,
        })
    }
}
impl Default for IntelProvider {
    fn default() -> Self {
        Self::new()
    }
}
impl GpuProvider for IntelProvider {
    /// Detect Intel GPUs using PowerShell WMI queries
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        let output_str = self.get_intel_gpu_info()?;
        let gpus = match self.parse_gpu_info(&output_str) {
            Some(gpu) => vec![gpu],
            None => Vec::new(),
        };
        crate::gpu_info::handle_empty_result(gpus)
    }
    /// Update the information for a specific Intel GPU
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        info!("Updating Intel GPU information");
        let output_str = self.get_intel_gpu_info()?;
        if let Some(updated_gpu) = self.parse_gpu_info(&output_str) {
            gpu.name_gpu = updated_gpu.name_gpu;
            gpu.vendor = updated_gpu.vendor;
            gpu.driver_version = updated_gpu.driver_version;
            gpu.memory_total = updated_gpu.memory_total;
            gpu.core_clock = updated_gpu.core_clock;
            gpu.max_clock_speed = updated_gpu.max_clock_speed;
            gpu.active = updated_gpu.active;
            // TODO:Don't overwrite: temperature, utilization, power_usage, power_limit, memory_util, memory_clock
        }

        // Note: This is the platform-agnostic Intel provider that only uses WMI.
        // For advanced metrics (PDH, Intel MD API), use IntelWindowsProvider on Windows.

        if !gpu.is_valid() {
            warn!("GPU data validation failed");
            return Err(crate::gpu_info::GpuError::GpuNotActive);
        }

        info!("Successfully updated Intel GPU information");
        Ok(())
    }
    /// Get the vendor for this provider
    fn get_vendor(&self) -> Vendor {
        Vendor::Intel(IntelGpuType::Unknown)
    }
}
/// Detect all Intel GPUs in the system.
///
/// This is a convenience function that creates a temporary [`IntelProvider`]
/// and calls [`detect_gpus`](GpuProvider::detect_gpus).
///
/// Returns an empty vector if no Intel GPUs are found.
pub fn detect_intel_gpus() -> Vec<GpuInfo> {
    let provider = IntelProvider::new();
    provider.detect_gpus().unwrap_or_default()
}

/// Update GPU information for an Intel GPU.
///
/// This is a convenience function that uses the appropriate provider for the platform.
/// On Windows, it uses `IntelWindowsProvider` for better metrics via Intel MD API.
/// On other platforms, it uses the basic [`IntelProvider`].
///
/// # Errors
///
/// Returns an error if the GPU update fails.
pub fn update_intel_info(gpu: &mut GpuInfo) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Use IntelWindowsProvider for better metrics via Intel MD API
        let provider = super::windows::intel::IntelWindowsProvider::new();
        provider.update_gpu(gpu)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let provider = IntelProvider::new();
        provider.update_gpu(gpu)
    }
}

// TODO: there should be no tests here. Transfer them to gpu_info\src\test
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_intel_provider_vendor() {
        let provider = IntelProvider::new();
        match provider.get_vendor() {
            Vendor::Intel(_) => {}
            _ => panic!("Expected Intel vendor"),
        }
    }
}
