//! Linux AMD GPU provider implementation
//!
//! This module implements the GpuProvider trait for AMD GPUs on Linux using sysfs.
use crate::gpu_info::{ GpuError, GpuInfo, GpuProvider, Result };
use crate::vendor::Vendor;
use log::{ debug, info, warn };
use std::fs;
use std::path::Path;
/// AMD GPU provider for Linux
pub struct AmdLinuxProvider;
impl AmdLinuxProvider {
    pub fn new() -> Self {
        Self
    }
    /// Detect AMD GPUs through sysfs interface
    fn detect_amd_gpus(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();
        let drm_path = Path::new("/sys/class/drm");
        if !drm_path.exists() {
            warn!("DRM sysfs path not found, AMD GPU detection unavailable");
            return Ok(gpus);
        }
        for entry in fs::read_dir(drm_path).map_err(|_| GpuError::GpuNotFound)? {
            let entry = entry.map_err(|_| GpuError::GpuNotFound)?;
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("card") && !name.contains("-") {
                    if let Ok(gpu_info) = self.probe_amd_card(&path) {
                        gpus.push(gpu_info);
                    }
                }
            }
        }
        if gpus.is_empty() {
            Err(GpuError::GpuNotFound)
        } else {
            info!("Detected {} AMD GPU(s) on Linux", gpus.len());
            Ok(gpus)
        }
    }
    /// Probe specific AMD card for information
    fn probe_amd_card(&self, card_path: &Path) -> Result<GpuInfo> {
        let device_path = card_path.join("device");
        // Check vendor ID for AMD (0x1002)
        let vendor_id = self.read_hex_file(&device_path.join("vendor"))?;
        if vendor_id != 0x1002 {
            return Err(GpuError::GpuNotFound);
        }
        // Get basic GPU information
        let name = self.get_gpu_name(&device_path)?;
        let driver_version = self.get_driver_version();
        // Get power management info if available
        let power_usage = self.get_power_usage(&device_path);
        let temperature = self.get_temperature(&device_path);
        let utilization = self.get_gpu_utilization(&device_path);
        let memory_info = self.get_memory_info(&device_path);
        // Get clock and power info
        let core_clock = self.get_core_clock(&device_path);
        let memory_clock = self.get_memory_clock(&device_path);
        let power_limit = self.get_power_limit(&device_path);
        let max_clock_speed = self.get_max_clock_speed(&device_path);

        info!("Found AMD GPU: {}", name);
        Ok(GpuInfo {
            vendor: Vendor::Amd,
            name_gpu: Some(name),
            temperature,
            utilization,
            power_usage,
            memory_total: memory_info.0,
            memory_util: memory_info.1,
            driver_version,
            active: Some(true),
            core_clock,
            memory_clock,
            power_limit,
            max_clock_speed,
        })
    }
    fn read_hex_file(&self, path: &Path) -> Result<u32> {
        let content = fs::read_to_string(path).map_err(|_| GpuError::GpuNotFound)?;
        let hex_str = content.trim().trim_start_matches("0x");
        u32::from_str_radix(hex_str, 16).map_err(|_| GpuError::GpuNotFound)
    }
    fn get_gpu_name(&self, device_path: &Path) -> Result<String> {
        if let Ok(content) = fs::read_to_string(device_path.join("product_name")) {
            return Ok(content.trim().to_string());
        }
        if let Ok(content) = fs::read_to_string(device_path.join("subsystem_device")) {
            return Ok(format!("AMD GPU (Device ID: {})", content.trim()));
        }
        Ok("AMD GPU".to_string())
    }
    fn get_driver_version(&self) -> Option<String> {
        // Try to get AMDGPU driver version
        if let Ok(content) = fs::read_to_string("/sys/module/amdgpu/version") {
            return Some(content.trim().to_string());
        }
        None
    }
    fn get_power_usage(&self, device_path: &Path) -> Option<f32> {
        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                if let Ok(power_str) = fs::read_to_string(hwmon_device.join("power1_average")) {
                    if let Ok(power_microwatts) = power_str.trim().parse::<u64>() {
                        return Some((power_microwatts as f32) / 1_000_000.0); // Convert to watts
                    }
                }
            }
        }
        None
    }
    fn get_temperature(&self, device_path: &Path) -> Option<f32> {
        // Check hwmon for temperature
        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                if let Ok(temp_str) = fs::read_to_string(hwmon_device.join("temp1_input")) {
                    if let Ok(temp_millidegrees) = temp_str.trim().parse::<u32>() {
                        return Some((temp_millidegrees as f32) / 1000.0); // Convert to Celsius
                    }
                }
            }
        }
        None
    }
    fn get_gpu_utilization(&self, device_path: &Path) -> Option<f32> {
        // Try to get GPU utilization from DRM engine interface first
        let drm_path = Path::new("/sys/class/drm");
        if drm_path.exists() {
            // Find the card directory that matches our device
            if let Some(card_name) = device_path.parent().and_then(|p| p.file_name()) {
                let engine_path = drm_path.join(card_name).join("engine");
                if engine_path.exists() {
                    // Try different engine types in order of preference
                    for engine_type in &["gfx", "compute"] {
                        let busy_percent_path = engine_path.join(engine_type).join("busy_percent");
                        if let Ok(content) = fs::read_to_string(&busy_percent_path) {
                            if let Ok(utilization) = content.trim().parse::<u32>() {
                                return Some(utilization as f32);
                            }
                        }
                    }
                }
            }
        }

        // Fallback to hwmon interface
        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                // Check if this hwmon device is for amdgpu
                if let Ok(name) = fs::read_to_string(hwmon_device.join("name")) {
                    if name.trim() == "amdgpu" {
                        if
                            let Ok(content) = fs::read_to_string(
                                hwmon_device.join("gpu_busy_percent")
                            )
                        {
                            if let Ok(utilization) = content.trim().parse::<u32>() {
                                return Some(utilization as f32);
                            }
                        }
                    }
                }
            }
        }

        None
    }
    fn get_core_clock(&self, device_path: &Path) -> Option<u32> {
        // Try to get core clock from pp_dpm_sclk
        let sclk_path = device_path.join("pp_dpm_sclk");
        if let Ok(content) = fs::read_to_string(&sclk_path) {
            // Parse the content to find the active clock speed
            // Format is typically:
            // 0: 300Mhz *
            // 1: 600Mhz
            // 2: 900Mhz *
            for line in content.lines() {
                if line.contains('*') {
                    if let Some(freq_str) = line.split(':').nth(1) {
                        let freq_str = freq_str
                            .trim()
                            .trim_end_matches(" *")
                            .trim_end_matches("Mhz");
                        if let Ok(freq) = freq_str.parse::<u32>() {
                            return Some(freq);
                        }
                    }
                }
            }
        }

        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                if let Ok(freq_str) = fs::read_to_string(hwmon_device.join("freq1_input")) {
                    if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                        return Some((freq_hz / 1_000_000) as u32); // Convert Hz to MHz
                    }
                }
            }
        }

        None
    }

    fn get_memory_clock(&self, device_path: &Path) -> Option<u32> {
        // Try to get memory clock from pp_dpm_mclk
        let mclk_path = device_path.join("pp_dpm_mclk");
        if let Ok(content) = fs::read_to_string(&mclk_path) {
            // Parse the content to find the active clock speed
            // Format is typically:
            // 0: 150Mhz *
            // 1: 500Mhz
            // 2: 800Mhz *
            for line in content.lines() {
                if line.contains('*') {
                    if let Some(freq_str) = line.split(':').nth(1) {
                        let freq_str = freq_str
                            .trim()
                            .trim_end_matches(" *")
                            .trim_end_matches("Mhz");
                        if let Ok(freq) = freq_str.parse::<u32>() {
                            return Some(freq);
                        }
                    }
                }
            }
        }

        // Fallback to hwmon freq2_input
        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                if let Ok(freq_str) = fs::read_to_string(hwmon_device.join("freq2_input")) {
                    if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                        return Some((freq_hz / 1_000_000) as u32); // Convert Hz to MHz
                    }
                }
            }
        }

        None
    }

    fn get_power_limit(&self, device_path: &Path) -> Option<f32> {
        // Try to get power limit from power1_cap (in microWatts)
        if let Ok(power_str) = fs::read_to_string(device_path.join("power1_cap")) {
            if let Ok(power_microwatts) = power_str.trim().parse::<u64>() {
                return Some((power_microwatts as f32) / 1_000_000.0); // Convert to watts
            }
        }

        // Fallback to hwmon power1_cap
        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                if let Ok(power_str) = fs::read_to_string(hwmon_device.join("power1_cap")) {
                    if let Ok(power_microwatts) = power_str.trim().parse::<u64>() {
                        return Some((power_microwatts as f32) / 1_000_000.0); // Convert to watts
                    }
                }
            }
        }

        None
    }

    fn get_max_clock_speed(&self, device_path: &Path) -> Option<u32> {
        // Try to get max core clock from pp_dpm_sclk
        let sclk_path = device_path.join("pp_dpm_sclk");
        if let Ok(content) = fs::read_to_string(&sclk_path) {
            let mut max_freq = 0u32;
            // Parse the content to find the maximum clock speed
            // Format is typically:
            // 0: 300Mhz *
            // 1: 600Mhz
            // 2: 900Mhz *
            for line in content.lines() {
                if let Some(freq_str) = line.split(':').nth(1) {
                    let freq_str = freq_str.trim().trim_end_matches(" *").trim_end_matches("Mhz");
                    if let Ok(freq) = freq_str.parse::<u32>() {
                        if freq > max_freq {
                            max_freq = freq;
                        }
                    }
                }
            }
            if max_freq > 0 {
                return Some(max_freq);
            }
        }

        // Fallback to hwmon freq1_max
        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                if let Ok(freq_str) = fs::read_to_string(hwmon_device.join("freq1_max")) {
                    if let Ok(freq_hz) = freq_str.trim().parse::<u64>() {
                        return Some((freq_hz / 1_000_000) as u32); // Convert Hz to MHz
                    }
                }
            }
        }

        None
    }

    fn get_memory_info(&self, device_path: &Path) -> (Option<u32>, Option<f32>) {
        // Try to get memory information from sysfs
        // Check /sys/class/drm/cardX/device/mem_info_* files

        // Get total VRAM size
        let vram_total_path = device_path.join("mem_info_vram_total");
        let vram_total = if let Ok(content) = fs::read_to_string(&vram_total_path) {
            if let Ok(bytes) = content.trim().parse::<u64>() {
                // Convert bytes to MB
                Some((bytes / (1024 * 1024)) as u32)
            } else {
                None
            }
        } else {
            None
        };

        // Get used VRAM size and calculate utilization
        let vram_used_path = device_path.join("mem_info_vram_used");
        let vram_util = if let Ok(content) = fs::read_to_string(&vram_used_path) {
            if let Ok(bytes) = content.trim().parse::<u64>() {
                // Calculate utilization percentage if we have total memory
                if let Some(total_mb) = vram_total {
                    if total_mb > 0 {
                        let used_mb = (bytes / (1024 * 1024)) as u32;
                        Some(((used_mb as f32) / (total_mb as f32)) * 100.0)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        (vram_total, vram_util)
    }
}
impl Default for AmdLinuxProvider {
    fn default() -> Self {
        Self::new()
    }
}
impl GpuProvider for AmdLinuxProvider {
    /// Detect all AMD GPUs on Linux using sysfs
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        debug!("Detecting AMD GPUs on Linux using sysfs");
        self.detect_amd_gpus()
    }
    /// Update AMD GPU information on Linux
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        debug!("Updating AMD GPU information on Linux");
        // For now, we'll re-detect the GPU
        // In a more sophisticated implementation, we'd maintain device handles
        let gpus = self.detect_gpus()?;
        if let Some(updated_gpu) = gpus.first() {
            *gpu = updated_gpu.clone();
        }
        Ok(())
    }
    /// Get the vendor for this provider
    fn get_vendor(&self) -> Vendor {
        Vendor::Amd
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vendor::Vendor;

    #[test]
    fn test_amd_linux_provider_vendor() {
        let provider = AmdLinuxProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Amd);
    }

    #[test]
    fn test_get_memory_info_with_nonexistent_paths() {
        let provider = AmdLinuxProvider::new();
        let temp_dir = std::env::temp_dir();
        // This tests that the function doesn't panic with non-existent paths
        let result = provider.get_memory_info(&temp_dir);
        // Should return (None, None) for non-existent files
        assert_eq!(result, (None, None));
    }
}
