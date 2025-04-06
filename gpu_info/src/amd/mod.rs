// src/amd/mod.rs
use crate::mode::gpu::{GpuInfo, GpuVendor};
use std::fs;

/// amd GPU detection and info parsing
///
/// Scans the `/sys/class/drm` directory for amd GPUs and parses
/// their information from sysfs files.
///
/// # Parameters
/// None
///
/// # Returns
/// None
pub fn detect_amd_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join("device/vendor").exists() {
                if let Ok(vendor) = fs::read_to_string(path.join("device/vendor")) {
                    if vendor.trim() == "0x1002" {
                        gpus.push(GpuInfo {
                            name: "amd GPU".to_string(),
                            vendor: GpuVendor::AMD,
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
/// [upd_amd_info](Self::update_amd_info)
pub fn update_amd_info(gpu: &mut GpuInfo) {
    if let Ok(temp) = fs::read_to_string("/sys/class/drm/card0/device/hwmon/hwmon0/temp1_input") {
        gpu.temperature = temp.trim().parse::<f32>().ok().map(|t| t / 1000.0);
    }
    if let Ok(util) = fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
        gpu.utilization = util.trim().parse().ok();
    }
    if let Ok(clock) = fs::read_to_string("/sys/class/drm/card0/device/pp_dpm_sclk") {
        gpu.clock_speed = clock
            .lines()
            .last()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|s| s.parse().ok());
    }
    if let Ok(power) = fs::read_to_string("/sys/class/drm/card0/device/hwmon/hwmon0/power1_average")
    {
        gpu.power_usage = power.trim().parse::<f32>().ok().map(|p| p / 1000000.0);
    }
}
