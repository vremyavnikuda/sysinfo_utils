// src/nvidia/mod.rs
#[cfg(target_os = "windows")]
mod windows {
    use crate::mode::gpu::{GpuInfo, GpuVendor};
    use log::{error, info, warn};
    use std::{
        ffi::CStr,
        os::raw::{c_char, c_uint},
        ptr,
    };
    const NVML_SUCCESS: i32 = 0;
    const NVML_TEMPERATURE_GPU: c_uint = 0;
    const NVML_CLOCK_GRAPHICS: c_uint = 0;
    const NVML_CLOCK_MEM: c_uint = 1;

    const NVML_DEVICE_GET_COUNT_MAX: usize = 64;

    #[repr(C)]
    #[allow(non_camel_case_types)]
    struct nvmlDevice_st;
    type nvmlDevice_t = *mut nvmlDevice_st;

    extern "C" {
        fn nvmlInit_v2() -> i32;
        fn nvmlShutdown() -> i32;
        fn nvmlDeviceGetCount_v2(count: *mut c_uint) -> i32;
        fn nvmlDeviceGetHandleByIndex_v2(index: c_uint, device: *mut nvmlDevice_t) -> i32;
        fn nvmlDeviceGetName(device: nvmlDevice_t, name: *mut c_char, length: c_uint) -> i32;
        fn nvmlDeviceGetMaxClockInfo(device: nvmlDevice_t, clkType: c_uint, clockMHz: *mut c_uint) -> i32;
        fn nvmlDeviceGetPowerManagementLimit(device: nvmlDevice_t, limit: *mut c_uint) -> i32;
        fn nvmlDeviceGetTemperature(
            device: nvmlDevice_t,
            sensorType: c_uint,
            temp: *mut c_uint,
        ) -> i32;
        fn nvmlDeviceGetClockInfo(
            device: nvmlDevice_t,
            clkType: c_uint,
            clockMHz: *mut c_uint,
        ) -> i32;
        fn nvmlDeviceGetUtilizationRates(
            device: nvmlDevice_t,
            utilization: *mut nvmlUtilization_t,
        ) -> i32;
        fn nvmlDeviceGetPowerUsage(device: nvmlDevice_t, milliwatts: *mut c_uint) -> i32;
    }

    #[repr(C)]
    #[allow(non_camel_case_types)]
    #[derive(Debug, Clone, Copy)]
    struct nvmlUtilization_t {
        gpu: c_uint,
        memory: c_uint,
    }

    fn check_nvml_status(code: i32, msg: &str) {
        if code != NVML_SUCCESS {
            error!("NVML error {} at '{}'", code, msg);
        }
    }
    pub fn detect_nvidia_gpus() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();

        let ret = unsafe { nvmlInit_v2() };
        if ret != NVML_SUCCESS {
            error!("nvmlInit_v2 failed with code {}", ret);
            return gpus;
        }

        info!("NVML initialized successfully.");

        let mut count: c_uint = 0;
        let ret2 = unsafe { nvmlDeviceGetCount_v2(&mut count) };
        if ret2 != NVML_SUCCESS {
            error!("nvmlDeviceGetCount_v2 failed: {}", ret2);
            unsafe {
                nvmlShutdown();
            }
            return gpus;
        }

        info!("NVML found {} GPU device(s)", count);

        for i in 0..count {
            let mut dev: nvmlDevice_t = ptr::null_mut();
            let ret3 = unsafe { nvmlDeviceGetHandleByIndex_v2(i, &mut dev) };
            if ret3 != NVML_SUCCESS {
                error!("nvmlDeviceGetHandleByIndex_v2({}) failed: {}", i, ret3);
                continue;
            }

            let mut name_buf = [0i8; 64];
            let ret_name = unsafe { nvmlDeviceGetName(dev, name_buf.as_mut_ptr(), 64) };
            let gpu_name = if ret_name == NVML_SUCCESS {
                let cstr = unsafe { CStr::from_ptr(name_buf.as_ptr()) };
                cstr.to_string_lossy().into_owned()
            } else {
                "NVIDIA GPU".to_string()
            };

            info!("Found GPU #{} => {}", i, gpu_name);

            gpus.push(GpuInfo {
                name: gpu_name,
                vendor: GpuVendor::Nvidia,
                ..Default::default()
            });
        }

        gpus
    }

    pub fn update_nvidia_info(gpu: &mut GpuInfo) {
        let mut count: c_uint = 0;
        let ret_count = unsafe { nvmlDeviceGetCount_v2(&mut count) };
        if ret_count != NVML_SUCCESS {
            error!("nvmlDeviceGetCount_v2 failed: {}", ret_count);
            return;
        }

        // Находим устройство по имени
        let gpu_name_lower = gpu.name.to_lowercase();
        let mut found_dev: Option<nvmlDevice_t> = None;

        for i in 0..count {
            let mut dev: nvmlDevice_t = ptr::null_mut();
            let ret = unsafe { nvmlDeviceGetHandleByIndex_v2(i, &mut dev) };
            if ret == NVML_SUCCESS {
                let mut name_buf = [0i8; 64];
                let ret_name = unsafe { nvmlDeviceGetName(dev, name_buf.as_mut_ptr(), 64) };
                if ret_name == NVML_SUCCESS {
                    let cstr = unsafe { CStr::from_ptr(name_buf.as_ptr()) };
                    let this_name = cstr.to_string_lossy().to_lowercase();
                    if this_name.contains(&gpu_name_lower) {
                        found_dev = Some(dev);
                        break;
                    }
                }
            }
        }

        let dev = match found_dev {
            Some(d) => d,
            None => {
                error!("No matching NVML device for '{}'", gpu.name);
                return;
            }
        };

        // Температура
        let mut temp_val: c_uint = 0;
        let ret_temp = unsafe { nvmlDeviceGetTemperature(dev, NVML_TEMPERATURE_GPU, &mut temp_val) };
        if ret_temp == NVML_SUCCESS {
            gpu.temperature = Some(temp_val as f32);
        } else {
            warn!("nvmlDeviceGetTemperature failed with code {}", ret_temp);
            gpu.temperature = None;
        }

        // Утилизация
        let mut util_data = nvmlUtilization_t { gpu: 0, memory: 0 };
        let ret_util = unsafe { nvmlDeviceGetUtilizationRates(dev, &mut util_data) };
        if ret_util == NVML_SUCCESS {
            gpu.utilization = Some(util_data.gpu as f32);
        } else {
            warn!("nvmlDeviceGetUtilizationRates failed with code {}", ret_util);
            gpu.utilization = None;
        }

        // Потребление энергии
        let mut mw: c_uint = 0;
        let ret_pow = unsafe { nvmlDeviceGetPowerUsage(dev, &mut mw) };
        if ret_pow == NVML_SUCCESS {
            gpu.power_usage = Some((mw as f32) / 1000.0); // Конвертируем мВт в Вт
        } else {
            warn!("nvmlDeviceGetPowerUsage failed with code {}", ret_pow);
            gpu.power_usage = None;
        }

        // Тактовая частота
        let mut clk_val: c_uint = 0;
        let ret_clk = unsafe { nvmlDeviceGetClockInfo(dev, NVML_CLOCK_GRAPHICS, &mut clk_val) };
        if ret_clk == NVML_SUCCESS {
            gpu.clock_speed = Some(clk_val as u64);
        } else {
            warn!("nvmlDeviceGetClockInfo(Graphics) failed: {}", ret_clk);
            gpu.clock_speed = None;
        }

        // Максимальная тактовая частота
        let mut max_clk_val: c_uint = 0;
        let ret_max_clk = unsafe { nvmlDeviceGetMaxClockInfo(dev, NVML_CLOCK_GRAPHICS, &mut max_clk_val) };
        if ret_max_clk == NVML_SUCCESS {
            gpu.max_clock_speed = Some(max_clk_val as u64);
        } else {
            warn!("nvmlDeviceGetMaxClockInfo failed: {}", ret_max_clk);
            gpu.max_clock_speed = None;
        }

        // Максимальное энергопотребление
        let mut max_power_val: c_uint = 0;
        let ret_max_pow = unsafe { nvmlDeviceGetPowerManagementLimit(dev, &mut max_power_val) };
        if ret_max_pow == NVML_SUCCESS {
            gpu.max_power_usage = Some((max_power_val as f32) / 1000.0); // Конвертируем мВт в Вт
        } else {
            warn!("nvmlDeviceGetPowerManagementLimit failed: {}", ret_max_pow);
            gpu.max_power_usage = None;
        }

        gpu.is_active = true;
    }
}

#[cfg(target_os = "windows")]
pub use windows::{detect_nvidia_gpus, update_nvidia_info};

#[cfg(target_os = "linux")]
mod linux {
    use crate::mode::gpu::{GpuInfo, GpuVendor};
    use std::fs;

    /// Detects all nvidia GPUs on the system and parses their information from sysfs files.
    ///
    /// The function scans the `/sys/class/drm` directory for NVIDIA GPUs and parses
    /// their information from sysfs files.
    ///
    /// # Parameters
    /// None
    ///
    /// # Returns
    /// A vector of `GpuInfo` instances, one for each detected nvidia GPU.
    pub fn detect_nvidia_gpus() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();

        if let Ok(entries) = fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("device/vendor").exists() {
                    if let Ok(vendor) = fs::read_to_string(path.join("device/vendor")) {
                        if vendor.trim() == "0x10de" {
                            let name = fs::read_to_string(path.join("device/model"))
                                .unwrap_or_else(|_| "Unknown NVIDIA GPU".to_string());
                            println!("Detected NVIDIA GPU: {}", name.trim()); // Отладочная информация
                            let temperature =
                                fs::read_to_string(path.join("device/hwmon/hwmon0/temp1_input"))
                                    .ok()
                                    .and_then(|temp| {
                                        temp.trim().parse().ok().map(|t: f32| t / 1000.0)
                                    });
                            let utilization =
                                fs::read_to_string(path.join("device/gpu_busy_percent"))
                                    .ok()
                                    .and_then(|util| util.trim().parse().ok());
                            let clock_speed = fs::read_to_string(path.join("device/pp_dpm_sclk"))
                                .ok()
                                .and_then(|clock| {
                                    clock
                                        .lines()
                                        .last()
                                        .and_then(|line| line.split_whitespace().nth(1))
                                        .and_then(|s| s.parse().ok())
                                });
                            let max_clock_speed =
                                fs::read_to_string(path.join("device/pp_dpm_sclk"))
                                    .ok()
                                    .and_then(|clock| {
                                        clock
                                            .lines()
                                            .last()
                                            .and_then(|line| line.split_whitespace().nth(1))
                                            .and_then(|s| s.parse().ok())
                                    });
                            let power_usage =
                                fs::read_to_string(path.join("device/hwmon/hwmon0/power1_average"))
                                    .ok()
                                    .and_then(|power| {
                                        power.trim().parse::<f32>().ok().map(|p| p / 1000000.0)
                                    });
                            let max_power_usage =
                                fs::read_to_string(path.join("device/hwmon/hwmon0/power1_cap"))
                                    .ok()
                                    .and_then(|power| {
                                        power.trim().parse::<f32>().ok().map(|p| p / 1000000.0)
                                    });

                            gpus.push(GpuInfo {
                                name: name.trim().to_string(),
                                vendor: GpuVendor::Nvidia,
                                temperature,
                                utilization,
                                clock_speed,
                                max_clock_speed,
                                power_usage,
                                max_power_usage,
                                is_active: true,
                            });
                        }
                    }
                }
            }
        }

        gpus
    }
}

#[cfg(target_os = "linux")]
pub use linux::detect_nvidia_gpus;
