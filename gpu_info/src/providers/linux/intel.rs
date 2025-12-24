//! Linux Intel GPU provider using sysfs.
//!
//! This module implements the [`GpuProvider`] trait for Intel GPUs on Linux
//! using sysfs interfaces.
//!
//! # Sysfs Paths
//!
//! - `/sys/class/drm/cardX/device/` - Device information
//! - `/sys/class/drm/cardX/device/hwmon/` - Hardware monitoring (temperature, power)
//! - `/sys/class/drm/cardX/device/gt_cur_freq_mhz` - Current GPU frequency
//! - `/sys/class/drm/cardX/device/gt_max_freq_mhz` - Maximum GPU frequency
//!
//! [`GpuProvider`]: crate::gpu_info::GpuProvider

use crate::gpu_info::{GpuError, GpuInfo, GpuProvider, Result};
use crate::vendor::{IntelGpuType, Vendor};
use log::{debug, info, warn};
use std::fs;
use std::path::Path;

/// Intel GPU provider for Linux.
///
/// Implements [`GpuProvider`] for Intel GPUs on Linux using sysfs interfaces.
/// This provider reads GPU information from `/sys/class/drm/` and collects
/// metrics from hwmon sensors and i915 driver interfaces.
///
/// # Supported Metrics
///
/// - Temperature (from hwmon temp1_input)
/// - Power usage (from hwmon power1_average)
/// - Core clock (from gt_cur_freq_mhz or gt_act_freq_mhz)
/// - Max clock speed (from gt_max_freq_mhz or gt_boost_freq_mhz)
/// - Power limit (from hwmon power1_cap)
///
/// [`GpuProvider`]: crate::gpu_info::GpuProvider
pub struct IntelLinuxProvider;

impl IntelLinuxProvider {
    /// Create a new Intel Linux provider instance.
    pub fn new() -> Self {
        Self
    }

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

    fn probe_intel_card(&self, card_path: &Path) -> Result<GpuInfo> {
        let device_path = card_path.join("device");
        let vendor_id = self.read_hex_file(&device_path.join("vendor"))?;
        if vendor_id != 0x8086 {
            return Err(GpuError::GpuNotFound);
        }

        let name = self.get_gpu_name(&device_path)?;
        let driver_version = self.get_driver_version();
        let power_usage = self.get_power_usage(&device_path);
        let temperature = self.get_temperature(&device_path);
        let utilization = self.get_gpu_utilization(&device_path);
        let memory_info = self.get_memory_info(&device_path);
        let core_clock = self.get_core_clock(&device_path);
        let memory_clock = self.get_memory_clock(&device_path);
        let power_limit = self.get_power_limit(&device_path);
        let max_clock_speed = self.get_max_clock_speed(&device_path);

        info!("Found Intel GPU: {}", name);

        Ok(GpuInfo {
            vendor: Vendor::Intel(IntelGpuType::Integrated),
            name_gpu: Some(name),
            temperature,
            utilization,
            power_usage,
            memory_total: memory_info.0,
            memory_used: None,
            memory_util: memory_info.1,
            driver_version,
            active: Some(true),
            core_clock,
            memory_clock,
            power_limit,
            max_clock_speed,
        })
    }

    pub(crate) fn read_hex_file(&self, path: &Path) -> Result<u32> {
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
        if let Ok(content) = fs::read_to_string("/sys/module/i915/version") {
            return Some(content.trim().to_string());
        }

        if let Ok(content) = fs::read_to_string("/proc/version") {
            if let Some(version_part) = content.split_whitespace().nth(2) {
                return Some(format!("i915 (kernel {})", version_part));
            }
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
        if let Ok(hwmon_dirs) = fs::read_dir(device_path.join("hwmon")) {
            for hwmon_entry in hwmon_dirs.flatten() {
                let hwmon_path = hwmon_entry.path();
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
        if let Some(card_num) = self.get_card_number(device_path) {
            let engine_info_path = format!("/sys/kernel/debug/dri/{}/i915_engine_info", card_num);
            if let Ok(content) = fs::read_to_string(&engine_info_path) {
                for line in content.lines() {
                    if line.contains("busy") || line.contains("utilization") {
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
        if let Ok(content) = fs::read_to_string(device_path.join("gt_cur_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        if let Ok(content) = fs::read_to_string(device_path.join("gt_act_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        None
    }

    fn get_memory_clock(&self, device_path: &Path) -> Option<u32> {
        if let Ok(content) = fs::read_to_string(device_path.join("gt_mem_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        None
    }

    fn get_power_limit(&self, device_path: &Path) -> Option<f32> {
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
        if let Ok(content) = fs::read_to_string(device_path.join("gt_max_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        if let Ok(content) = fs::read_to_string(device_path.join("gt_boost_freq_mhz")) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return Some(freq);
            }
        }

        None
    }

    pub(crate) fn get_memory_info(&self, _device_path: &Path) -> (Option<u32>, Option<f32>) {
        (None, None)
    }

    fn get_card_number(&self, device_path: &Path) -> Option<usize> {
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
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        debug!("Detecting Intel GPUs on Linux using sysfs");
        self.detect_intel_gpus()
    }

    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        debug!("Updating Intel GPU information on Linux");
        let gpus = self.detect_gpus()?;
        if let Some(updated_gpu) = gpus.first() {
            *gpu = updated_gpu.clone();
        }
        Ok(())
    }

    fn get_vendor(&self) -> Vendor {
        Vendor::Intel(IntelGpuType::Unknown)
    }
}
