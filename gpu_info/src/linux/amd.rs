use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::vendor::Vendor;
use log::{debug, error, info, warn};
use std::fs;
use std::path::Path;

/// AMD GPU detection and monitoring for Linux using sysfs and AMDGPU driver
pub struct AmdLinuxProvider;

impl AmdLinuxProvider {
    /// Detect AMD GPUs through sysfs interface
    pub fn detect_amd_gpus() -> Result<Vec<GpuInfo>> {
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
                    if let Ok(gpu_info) = Self::probe_amd_card(&path) {
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
    fn probe_amd_card(card_path: &Path) -> Result<GpuInfo> {
        let device_path = card_path.join("device");
        // Check vendor ID for AMD (0x1002)
        let vendor_id = Self::read_hex_file(&device_path.join("vendor"))?;
        if vendor_id != 0x1002 {
            return Err(GpuError::GpuNotFound);
        }
        let name = Self::get_gpu_name(&device_path)?;
        let driver_version = Self::get_driver_version();
        let power_usage = Self::get_power_usage(&device_path);
        let temperature = Self::get_temperature(&device_path);
        let utilization = Self::get_gpu_utilization(&device_path);
        let memory_info = Self::get_memory_info(&device_path);
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
            core_clock: None, // TODO: implement via sysfs
            memory_clock: None, // TODO: implement via sysfs
            power_limit: None, // TODO: implement via sysfs
            max_clock_speed: None, // TODO: implement via sysfs
        })
    }

    fn read_hex_file(path: &Path) -> Result<u32> {
        let content = fs::read_to_string(path)
            .map_err(|_| GpuError::GpuNotFound)?;
        let hex_str = content.trim().trim_start_matches("0x");
        u32::from_str_radix(hex_str, 16)
            .map_err(|_| GpuError::GpuNotFound)
    }

    fn get_gpu_name(device_path: &Path) -> Result<String> {
        if let Ok(content) = fs::read_to_string(device_path.join("product_name")) {
            return Ok(content.trim().to_string());
        }
        if let Ok(content) = fs::read_to_string(device_path.join("subsystem_device")) {
            return Ok(format!("AMD GPU (Device ID: {})", content.trim()));
        }
        Ok("AMD GPU".to_string())
    }

    fn get_driver_version() -> Option<String> {
        // Try to get AMDGPU driver version
        if let Ok(content) = fs::read_to_string("/sys/module/amdgpu/version") {
            return Some(content.trim().to_string());
        }
        None
    }

    fn get_power_usage(device_path: &Path) -> Option<f32> {
        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                if let Ok(power_str) = fs::read_to_string(hwmon_device.join("power1_average")) {
                    if let Ok(power_microwatts) = power_str.trim().parse::<u64>() {
                        return Some(power_microwatts as f32 / 1_000_000.0); // Convert to watts
                    }
                }
            }
        }
        None
    }

    fn get_temperature(device_path: &Path) -> Option<f32> {
        // Check hwmon for temperature
        let hwmon_path = device_path.join("hwmon");
        if let Ok(entries) = fs::read_dir(&hwmon_path) {
            for entry in entries.flatten() {
                let hwmon_device = entry.path();
                if let Ok(temp_str) = fs::read_to_string(hwmon_device.join("temp1_input")) {
                    if let Ok(temp_millidegrees) = temp_str.trim().parse::<u32>() {
                        return Some(temp_millidegrees as f32 / 1000.0); // Convert to Celsius
                    }
                }
            }
        }
        None
    }

    fn get_gpu_utilization(_device_path: &Path) -> Option<f32> {
        // TODO: Implement GPU utilization reading from sysfs
        // This might require parsing /sys/class/drm/cardX/engine/*/busy_percent
        None
    }

    fn get_memory_info(_device_path: &Path) -> (Option<u32>, Option<f32>) {
        // TODO: Implement memory information from sysfs
        // Check /sys/class/drm/cardX/mem_info_* files
        (None, None)
    }

    /// Update AMD GPU information
    pub fn update_amd_info(gpu: &mut GpuInfo) -> Result<()> {
        debug!("Updating AMD GPU information on Linux");
        // For now, we'll re-detect the GPU
        // In a more sophisticated implementation, we'd maintain device handles
        let gpus = Self::detect_amd_gpus()?;
        if let Some(updated_gpu) = gpus.first() {
            *gpu = updated_gpu.clone();
        }
        
        Ok(())
    }
}

/// Get AMD GPU information for Linux
pub fn info_gpu() -> GpuInfo {
    match AmdLinuxProvider::detect_amd_gpus() {
        Ok(gpus) if !gpus.is_empty() => gpus[0].clone(),
        _ => {
            error!("Failed to detect AMD GPU on Linux");
            GpuInfo::unknown()
        }
    }
}

/// Update AMD GPU information for Linux
pub fn update_amd_info(gpu: &mut GpuInfo) -> Result<()> {
    AmdLinuxProvider::update_amd_info(gpu)
}