//! Intel GPU provider implementation
//!
//! This module implements the GpuProvider trait for Intel GPUs using WMI queries.
use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::vendor::{IntelGpuType, Vendor};
use log::{debug, error, info, warn};
use std::process::Command;
/// Intel GPU provider
pub struct IntelProvider;
impl IntelProvider {
    pub fn new() -> Self {
        Self
    }
    fn determine_intel_gpu_type(&self, name: &str) -> IntelGpuType {
        crate::vendor::determine_intel_gpu_type_from_name(name)
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
                "Name, AdapterRAM, DriverVersion, CurrentRefreshRate, MaxRefreshRate, Status",
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
        let memory_total = output_str
            .lines()
            .find(|line| line.contains("AdapterRAM"))
            .and_then(|line| {
                let bytes = line.split(":").nth(1)?.trim().parse::<u64>().ok()?;
                Some((bytes / 1024 / 1024 / 1024) as u32)
            });
        let core_clock = output_str
            .lines()
            .find(|line| line.contains("CurrentRefreshRate"))
            .and_then(|line| line.split(":").nth(1)?.trim().parse::<u32>().ok());
        let max_clock_speed = output_str
            .lines()
            .find(|line| line.contains("MaxRefreshRate"))
            .and_then(|line| line.split(":").nth(1)?.trim().parse::<u32>().ok());
        let status = output_str
            .lines()
            .find(|line| line.contains("Status"))
            .map(|line| line.split(":").nth(1).unwrap_or("").trim() == "OK");
        info!("Found Intel GPU: {} (Type: {:?})", gpu_name, gpu_type);
        if let Some(ver) = &driver_version {
            info!("Driver version: {}", ver);
        }
        if let Some(mem) = memory_total {
            info!("Total memory: {} GB", mem);
        }
        if let Some(clock) = core_clock {
            info!("Current clock speed: {} MHz", clock);
        }
        if let Some(max_clock) = max_clock_speed {
            info!("Max clock speed: {} MHz", max_clock);
        }
        Some(GpuInfo {
            name_gpu: Some(gpu_name),
            vendor: Vendor::Intel(gpu_type),
            driver_version,
            memory_total,
            core_clock,
            max_clock_speed,
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
            // Don't overwrite: temperature, utilization, power_usage, power_limit, memory_util, memory_clock
        }

        // Try to get additional metrics from Performance Counters
        #[cfg(target_os = "windows")]
        {
            match super::windows::pdh::get_gpu_metrics(0) {
                Ok(pdh_metrics) => {
                    info!("Got PDH metrics for Intel GPU");

                    if let Some(util) = pdh_metrics.utilization() {
                        gpu.utilization = Some(util);
                        info!("GPU utilization from PDH: {:.2}%", util);
                    }

                    if let Some(mem_mb) = pdh_metrics.memory_usage_mb() {
                        // Calculate memory utilization percentage if we know total memory
                        if let Some(total_mb) = gpu.memory_total {
                            let mem_percent = (mem_mb as f32 / (total_mb * 1024) as f32) * 100.0;
                            gpu.memory_util = Some(mem_percent.min(100.0));
                            info!("GPU memory utilization from PDH: {:.2}%", mem_percent);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to get PDH metrics for Intel GPU: {:?}", e);
                }
            }
        }

        // Try to get additional metrics from Intel Metrics API (GPA Framework)
        #[cfg(target_os = "windows")]
        {
            if let Ok(intel_metrics) = super::windows::intel_metrics::get_intel_metrics() {
                info!("Got Intel Metrics API data");

                if let Some(temp) = intel_metrics.temperature {
                    gpu.temperature = Some(temp);
                    info!("GPU temperature from Intel Metrics: {:.2} C", temp);
                }

                if let Some(power) = intel_metrics.power_usage {
                    gpu.power_usage = Some(power);
                    info!("GPU power usage from Intel Metrics: {:.2}W", power);
                }

                if let Some(clock) = intel_metrics.core_clock {
                    gpu.core_clock = Some(clock);
                    info!("GPU core clock from Intel Metrics: {} MHz", clock);
                }

                if let Some(mem_clock) = intel_metrics.memory_clock {
                    gpu.memory_clock = Some(mem_clock);
                    info!("GPU memory clock from Intel Metrics: {} MHz", mem_clock);
                }
            } else {
                debug!("Intel Metrics API not available or not implemented yet");
            }
        }

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
// Backwards compatibility functions
pub fn detect_intel_gpus() -> Vec<GpuInfo> {
    let provider = IntelProvider::new();
    provider.detect_gpus().unwrap_or_default()
}
pub fn update_intel_info(gpu: &mut GpuInfo) -> Result<()> {
    let provider = IntelProvider::new();
    provider.update_gpu(gpu)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_intel_provider_vendor() {
        let provider = IntelProvider::new();
        // Note: We can't directly compare Vendor::Intel values because they contain IntelGpuType
        match provider.get_vendor() {
            Vendor::Intel(_) => {}
            _ => panic!("Expected Intel vendor"),
        }
    }
}
