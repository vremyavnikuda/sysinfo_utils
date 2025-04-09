// src/intel/mod.rs

use crate::mode::gpu::{GpuInfo, GpuVendor};
use std::fs;
/// Intel GPU detection and info parsing
///
/// Scans the `/sys/class/drm` directory for Intel GPUs and parses
/// their information from sysfs files.
///
/// # Parameters
/// None
///
/// # Returns
/// None
pub fn detect_intel_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("device/subsystem_vendor").exists() {
                if let Ok(vendor) = fs::read_to_string(path.join("device/subsystem_vendor")) {
                    if vendor.trim() == "0x8086" {
                        gpus.push(GpuInfo {
                            name: "Intel GPU".to_string(),
                            vendor: GpuVendor::Intel,
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    gpus
}
// TODO: Требует интеграционного тестирования с реальными процессами
// TODO: Требует обработки ошибок
// TODO: Требует доработки для других платформ( Windows, macOS)
// TODO: Требуется документация

/// Updates the metrics of an Intel GPU
///
/// This function reads various sysfs files to extract the temperature, utilization,
/// clock speed, and power usage of an Intel GPU and updates the provided `GpuInfo`
/// struct with these values.
///
/// # Parameters
/// - `gpu`: A mutable reference to a `GpuInfo` struct that will be updated with
///          the latest metrics.
///
/// # Platform Support
/// - Linux-only detection, as it relies on sysfs paths specific to Linux.
///
/// # Errors
/// - This function does not return errors directly, but the `GpuInfo` fields will
///   remain `None` if reading or parsing any of the sysfs files fails.
pub fn update_intel_info(gpu: &mut GpuInfo) {
    if let Ok(temp) = fs::read_to_string("/sys/class/drm/card0/device/hwmon/hwmon0/temp1_input") {
        gpu.temperature = temp.trim().parse::<f32>().ok().map(|t| t / 1000.0);
    }
    if let Ok(util) = fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
        gpu.utilization = util.trim().parse().ok();
    }
    if let Ok(clock) = fs::read_to_string("/sys/class/drm/card0/device/gt_max_freq_mhz") {
        gpu.clock_speed = clock.trim().parse().ok();
    }
    if let Ok(power) = fs::read_to_string("/sys/class/drm/card0/device/hwmon/hwmon0/power1_average")
    {
        gpu.power_usage = power.trim().parse::<f32>().ok().map(|p| p / 1000000.0);
    }
}
