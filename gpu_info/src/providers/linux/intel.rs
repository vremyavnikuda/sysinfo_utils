//! Linux Intel GPU provider implementation
//!
//! This module implements the GpuProvider trait for Intel GPUs on Linux using sysfs.
use crate::gpu_info::{GpuError, GpuInfo, GpuProvider, Result};
use crate::vendor::{IntelGpuType, Vendor};
use log::{debug, info, warn};
use std::fs;
use std::path::Path;

/// Intel GPU provider for Linux
pub struct IntelLinuxProvider;

impl IntelLinuxProvider {
    pub fn new() -> Self {
        Self
    }

    /// Detect Intel GPUs through sysfs interface
    fn detect_intel_gpus(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();
        let drm_path = Path::new("/sys/class/drm");

        if !drm_path.exists() {
            warn!("DRM sysfs path not found, Intel GPU detection unavailable");
            return Ok(gpus);
        }

        for entry in fs::read_dir(drm_path).map_err(|_| GpuError::GpuNotFound)? {
            let entry = entry.map_err(|_| GpuError::GpuNotFound)?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("card") && !name.contains("-") {
                    if let Ok(gpu_info) = self.probe_intel_card(&path) {
                        gpus.push(gpu_info);
                    }
                }
            }
        }

        if gpus.is_empty() {
            Err(GpuError::GpuNotFound)
        } else {
            info!("Detected {} Intel GPU(s) on Linux", gpus.len());
            Ok(gpus)
        }
    }

    /// Probe specific Intel card for information
    fn probe_intel_card(&self, card_path: &Path) -> Result<GpuInfo> {
        let device_path = card_path.join("device");

        // Check vendor ID for Intel (0x8086)
        let vendor_id = self.read_hex_file(&device_path.join("vendor"))?;
        if vendor_id != 0x8086 {
            return Err(GpuError::GpuNotFound);
        }

        // Get basic GPU information
        let name = self.get_gpu_name(&device_path)?;
        let driver_version = self.get_driver_version();

        // Get power management info if available
        let temperature = self.get_temperature(&device_path);
        let utilization = self.get_gpu_utilization(&device_path);
        let memory_info = self.get_memory_info(&device_path);

        // Get clock info
        let core_clock = self.get_core_clock(&device_path);

        info!("Found Intel GPU: {}", name);

        Ok(GpuInfo {
            vendor: Vendor::Intel(IntelGpuType::Integrated), // Most Intel GPUs are integrated
            name_gpu: Some(name),
            temperature,
            utilization,
            power_usage: None, // Intel integrated GPUs don't typically expose power usage via sysfs
            memory_total: memory_info.0,
            memory_util: memory_info.1,
            driver_version,
            active: Some(true),
            core_clock,
            memory_clock: None, // Memory clock not typically available for Intel iGPUs
            power_limit: None,  // Power limit not typically available via sysfs
            max_clock_speed: self.get_max_clock_speed(&device_path),
        })
    }

    fn read_hex_file(&self, path: &Path) -> Result<u32> {
        let content = fs::read_to_string(path).map_err(|_| GpuError::GpuNotFound)?;
        let hex_str = content.trim().trim_start_matches("0x");
        u32::from_str_radix(hex_str, 16).map_err(|_| GpuError::GpuNotFound)
    }

    fn get_gpu_name(&self, device_path: &Path) -> Result<String> {
        // Try to get device ID and map to known models
        if let Ok(device_id) = self.read_hex_file(&device_path.join("device")) {
            // Try to get subsystem device name
            if let Ok(content) = fs::read_to_string(device_path.join("subsystem_device")) {
                return Ok(format!(
                    "Intel GPU (Device ID: 0x{:04x}, {})",
                    device_id,
                    content.trim()
                ));
            }
            return Ok(format!("Intel GPU (Device ID: 0x{:04x})", device_id));
        }

        Ok("Intel GPU".to_string())
    }

    fn get_driver_version(&self) -> Option<String> {
        // Try to get i915 driver version
        if let Ok(content) = fs::read_to_string("/sys/module/i915/version") {
            return Some(content.trim().to_string());
        }

        // Fallback: try to get from kernel version
        if let Ok(content) = fs::read_to_string("/proc/version") {
            if let Some(version_part) = content.split_whitespace().nth(2) {
                return Some(format!("i915 (kernel {})", version_part));
            }
        }

        None
    }

    fn get_temperature(&self, device_path: &Path) -> Option<f32> {
        // Intel GPUs typically expose temperature via hwmon
        if let Ok(hwmon_dirs) = fs::read_dir(device_path.join("hwmon")) {
            for hwmon_entry in hwmon_dirs.flatten() {
                let hwmon_path = hwmon_entry.path();

                // Try temp1_input (common for Intel GPUs)
                if let Ok(content) = fs::read_to_string(hwmon_path.join("temp1_input")) {
                    if let Ok(temp_millidegrees) = content.trim().parse::<u32>() {
                        return Some(temp_millidegrees as f32 / 1000.0);
                    }
                }
            }
        }

        None
    }

    fn get_gpu_utilization(&self, device_path: &Path) -> Option<f32> {
        // Intel i915 driver exposes GPU busy percentage via debugfs
        // This is typically available in /sys/kernel/debug/dri/X/i915_engine_info
        // However, debugfs requires special permissions, so this might not be available

        // Try reading from engine info if available
        if let Some(card_num) = self.get_card_number(device_path) {
            let engine_info_path = format!("/sys/kernel/debug/dri/{}/i915_engine_info", card_num);
            if let Ok(content) = fs::read_to_string(&engine_info_path) {
                // Parse engine info to get utilization
                // This is a simplified approach
                for line in content.lines() {
                    if line.contains("busy") || line.contains("utilization") {
                        // Try to extract percentage
                        if let Some(percent_str) = line.split_whitespace().last() {
                            if let Ok(percent) = percent_str.trim_end_matches('%').parse::<f32>() {
                                return Some(percent);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn get_core_clock(&self, device_path: &Path) -> Option<u32> {
        // Intel GPUs expose current frequency via sysfs
        if let Ok(content) = fs::read_to_string(device_path.join("gt_cur_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        // Fallback: try gt_act_freq_mhz (actual frequency)
        if let Ok(content) = fs::read_to_string(device_path.join("gt_act_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        None
    }

    fn get_max_clock_speed(&self, device_path: &Path) -> Option<u32> {
        // Intel GPUs expose max frequency via sysfs
        if let Ok(content) = fs::read_to_string(device_path.join("gt_max_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        // Fallback: try gt_boost_freq_mhz
        if let Ok(content) = fs::read_to_string(device_path.join("gt_boost_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        None
    }

    fn get_memory_info(&self, _device_path: &Path) -> (Option<u32>, Option<f32>) {
        // Intel integrated GPUs share system memory, so this information is typically not available
        // We can try to read from i915 if available, but it's often limited

        let vram_total = None; // Intel iGPUs don't have dedicated VRAM
        let vram_util = None; // Utilization not typically available

        (vram_total, vram_util)
    }

    fn get_card_number(&self, device_path: &Path) -> Option<usize> {
        // Extract card number from path like /sys/class/drm/card0/device
        if let Some(parent) = device_path.parent() {
            if let Some(name) = parent.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("card") {
                    return name.trim_start_matches("card").parse::<usize>().ok();
                }
            }
        }
        None
    }
}

impl Default for IntelLinuxProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuProvider for IntelLinuxProvider {
    /// Detect all Intel GPUs on Linux using sysfs
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        debug!("Detecting Intel GPUs on Linux using sysfs");
        self.detect_intel_gpus()
    }

    /// Update Intel GPU information on Linux
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        debug!("Updating Intel GPU information on Linux");
        // Re-detect the GPU to get updated information
        let gpus = self.detect_gpus()?;
        if let Some(updated_gpu) = gpus.first() {
            *gpu = updated_gpu.clone();
        }
        Ok(())
    }

    /// Get the vendor for this provider
    fn get_vendor(&self) -> Vendor {
        Vendor::Intel(IntelGpuType::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intel_linux_provider_creation() {
        let provider = IntelLinuxProvider::new();
        let default_provider = IntelLinuxProvider::default();
        // Just ensure they can be created without panicking
        assert!(matches!(provider.get_vendor(), Vendor::Intel(_)));
        assert!(matches!(default_provider.get_vendor(), Vendor::Intel(_)));
    }

    #[test]
    fn test_get_memory_info_with_nonexistent_paths() {
        let provider = IntelLinuxProvider::new();
        let temp_dir = std::env::temp_dir();
        // This tests that the function doesn't panic with non-existent paths
        let result = provider.get_memory_info(&temp_dir);
        // Should return (None, None) for Intel iGPUs
        assert_eq!(result, (None, None));
    }

    #[test]
    fn test_read_hex_file_invalid_path() {
        let provider = IntelLinuxProvider::new();
        let result = provider.read_hex_file(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
