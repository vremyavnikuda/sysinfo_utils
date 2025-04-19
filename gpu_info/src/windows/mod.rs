//gpu_info/src/windows/mod.rs
use crate::gpu_info::GpuInfo;
use crate::vendor::Vendor;
use log::{error, info, warn};
use std::ffi::CStr;
use std::os::raw::{c_char, c_uint};
use std::ptr;

pub(crate) const NVML_SUCCESS: i32 = 0;
pub(crate) const NVML_TEMPERATURE_GPU: c_uint = 0;
pub(crate) const NVML_CLOCK_GRAPHICS: c_uint = 0;
#[allow(dead_code)]
pub(crate) const NVML_CLOCK_MEM: c_uint = 1;
#[allow(dead_code)]
pub(crate) const NVML_DEVICE_GET_COUNT_MAX: usize = 64;

#[repr(C)]
#[allow(non_camel_case_types)]
pub(crate) struct nvmlDevice_st {
    _unused: [u8; 0],
}
#[cfg(test)]
use mockall::automock;

//TODO: ИСПОЛЬЗОВАТЬ ЭТОТ trait
//#[warn(dead_code)]
#[cfg_attr(test, automock)]
pub(crate) trait NvmlClient {
    fn init(&self) -> i32;
    fn shutdown(&self) -> i32;
    fn get_count(&self, count: *mut c_uint) -> i32;
    fn get_handle_by_index(&self, index: c_uint, device: *mut nvmlDevice_t) -> i32;
    fn get_name(&self, device: nvmlDevice_t, name: *mut c_char, length: c_uint) -> i32;
    fn get_temperature(&self, device: nvmlDevice_t, sensor_type: c_uint, temp: *mut c_uint) -> i32;
    fn get_utilization_rates(&self, device: nvmlDevice_t, util: *mut nvmlUtilization_t) -> i32;
    fn get_power_usage(&self, device: nvmlDevice_t, milliwatts: *mut c_uint) -> i32;
    fn get_clock_info(&self, device: nvmlDevice_t, clk_type: c_uint, clock: *mut c_uint) -> i32;
    fn get_max_clock_info(&self, device: nvmlDevice_t, clk_type: c_uint, clock: *mut c_uint)
        -> i32;
    fn get_power_management_limit(&self, device: nvmlDevice_t, limit: *mut c_uint) -> i32;
}

pub struct NvmlClientImpl;

#[allow(non_camel_case_types)]
#[allow(unsafe_code)]
pub(crate) type nvmlDevice_t = *mut nvmlDevice_st;

extern "C" {
    pub(crate) fn nvmlInit_v2() -> i32;
    pub(crate) fn nvmlShutdown() -> i32;
    #[allow(unsafe_code)]
    pub(crate) fn nvmlDeviceGetCount_v2(count: *mut c_uint) -> i32;
    #[allow(unsafe_code)]
    pub(crate) fn nvmlDeviceGetHandleByIndex_v2(index: c_uint, device: *mut nvmlDevice_t) -> i32;
    pub(crate) fn nvmlDeviceGetName(device: nvmlDevice_t, name: *mut c_char, length: c_uint)
        -> i32;
    pub(crate) fn nvmlDeviceGetMaxClockInfo(
        device: nvmlDevice_t,
        clkType: c_uint,
        clockMHz: *mut c_uint,
    ) -> i32;
    pub(crate) fn nvmlDeviceGetPowerManagementLimit(
        device: nvmlDevice_t,
        limit: *mut c_uint,
    ) -> i32;
    pub(crate) fn nvmlDeviceGetTemperature(
        device: nvmlDevice_t,
        sensorType: c_uint,
        temp: *mut c_uint,
    ) -> i32;
    pub(crate) fn nvmlDeviceGetClockInfo(
        device: nvmlDevice_t,
        clkType: c_uint,
        clockMHz: *mut c_uint,
    ) -> i32;
    pub(crate) fn nvmlDeviceGetUtilizationRates(
        device: nvmlDevice_t,
        utilization: *mut nvmlUtilization_t,
    ) -> i32;
    pub(crate) fn nvmlDeviceGetPowerUsage(device: nvmlDevice_t, milliwatts: *mut c_uint) -> i32;
}

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct nvmlUtilization_t {
    pub(crate) gpu: c_uint,
    pub(crate) memory: c_uint,
}

impl NvmlClient for NvmlClientImpl {
    fn init(&self) -> i32 {
        unsafe { nvmlInit_v2() }
    }

    fn shutdown(&self) -> i32 {
        unsafe { nvmlShutdown() }
    }

    fn get_count(&self, count: *mut c_uint) -> i32 {
        unsafe { nvmlDeviceGetCount_v2(count) }
    }

    fn get_handle_by_index(&self, index: c_uint, device: *mut nvmlDevice_t) -> i32 {
        unsafe { nvmlDeviceGetHandleByIndex_v2(index, device) }
    }

    fn get_name(&self, device: nvmlDevice_t, name: *mut c_char, length: c_uint) -> i32 {
        unsafe { nvmlDeviceGetName(device, name, length) }
    }

    fn get_temperature(&self, device: nvmlDevice_t, sensor_type: c_uint, temp: *mut c_uint) -> i32 {
        unsafe { nvmlDeviceGetTemperature(device, sensor_type, temp) }
    }

    fn get_utilization_rates(&self, device: nvmlDevice_t, util: *mut nvmlUtilization_t) -> i32 {
        unsafe { nvmlDeviceGetUtilizationRates(device, util) }
    }

    fn get_power_usage(&self, device: nvmlDevice_t, milliwatts: *mut c_uint) -> i32 {
        unsafe { nvmlDeviceGetPowerUsage(device, milliwatts) }
    }

    fn get_clock_info(&self, device: nvmlDevice_t, clk_type: c_uint, clock: *mut c_uint) -> i32 {
        unsafe { nvmlDeviceGetClockInfo(device, clk_type, clock) }
    }

    fn get_max_clock_info(
        &self,
        device: nvmlDevice_t,
        clk_type: c_uint,
        clock: *mut c_uint,
    ) -> i32 {
        unsafe { nvmlDeviceGetMaxClockInfo(device, clk_type, clock) }
    }

    fn get_power_management_limit(&self, device: nvmlDevice_t, limit: *mut c_uint) -> i32 {
        unsafe { nvmlDeviceGetPowerManagementLimit(device, limit) }
    }
}
/// Detects NVIDIA GPUs using the NVML library and returns their information.
///
/// This function initializes the NVML library, retrieves the count of available NVIDIA GPUs,
/// and collects basic information (such as the name and vendor) for each detected GPU.
/// If the initialization or any NVML function call fails, appropriate error messages are logged.
///
/// # Returns
/// A `Vec<GpuInfo>` containing information about the detected NVIDIA GPUs. If no GPUs are found
/// or an error occurs during detection, an empty vector is returned.
///
pub fn detect_nvidia_gpus() -> Vec<GpuInfo> {
    let nvml = NvmlClientImpl;
    let mut gpus = Vec::new();

    // Инициализируем NVML
    let ret = nvml.init();
    if ret != NVML_SUCCESS {
        error!("nvmlInit_v2 failed with code {}", ret);
        return gpus;
    }
    info!("NVML initialized successfully.");

    let mut count: c_uint = 0;
    let ret2 = nvml.get_count(&mut count);
    if ret2 != NVML_SUCCESS {
        error!("nvmlDeviceGetCount_v2 failed: {}", ret2);
        nvml.shutdown();
        return gpus;
    }
    info!("NVML found {} GPU device(s)", count);

    for i in 0..count {
        let mut dev: nvmlDevice_t = ptr::null_mut();
        let ret3 = nvml.get_handle_by_index(i, &mut dev);
        if ret3 != NVML_SUCCESS {
            error!("nvmlDeviceGetHandleByIndex_v2({}) failed: {}", i, ret3);
            continue;
        }

        let mut name_buf = [0i8; 64];
        let ret_name = nvml.get_name(dev, name_buf.as_mut_ptr(), 64);
        let gpu_name = if ret_name == NVML_SUCCESS {
            let cstr = unsafe { CStr::from_ptr(name_buf.as_ptr()) };
            cstr.to_string_lossy().into_owned()
        } else {
            "NVIDIA GPU".to_string()
        };

        info!("Found GPU #{} => {}", i, gpu_name);

        gpus.push(GpuInfo {
            name_gpu: Some(gpu_name),
            vendor: Vendor::Nvidia,
            ..Default::default()
        });
    }

    nvml.shutdown();
    gpus
}

/// Updates the information of a given NVIDIA GPU using the NVML library.
///
/// This function attempts to match the provided `GpuInfo` instance with an NVIDIA GPU
/// detected by the NVML library. If a match is found, it updates the GPU's information,
/// including temperature, utilization, power usage, clock speeds, and power limits.
///
/// # Arguments
/// * `gpu` - A mutable reference to a `GpuInfo` instance that will be updated with
///           the latest information from the NVML library.
///
/// If the NVML library fails to initialize or the GPU cannot be matched, the function
/// logs appropriate error messages and exits early.
///
/// # Safety
/// This function uses unsafe blocks to interact with the NVML library, which requires
/// careful handling of pointers and external function calls
pub fn update_nvidia_info(gpu: &mut GpuInfo) {
    let nvml = NvmlClientImpl;

    let ret_init = nvml.init();
    if ret_init != NVML_SUCCESS {
        error!("nvmlInit_v2 failed with code {}", ret_init);
        return;
    }

    let mut count: c_uint = 0;
    let ret_count = nvml.get_count(&mut count);
    if ret_count != NVML_SUCCESS {
        error!("nvmlDeviceGetCount_v2 failed: {}", ret_count);
        nvml.shutdown();
        return;
    }

    let gpu_name_lower = match &gpu.name_gpu {
        Some(name) => name.to_lowercase(),
        None => {
            error!("GPU name is not set");
            nvml.shutdown();
            return;
        }
    };

    let mut found_dev: Option<nvmlDevice_t> = None;
    for i in 0..count {
        let mut dev: nvmlDevice_t = ptr::null_mut();
        let ret = nvml.get_handle_by_index(i, &mut dev);
        if ret == NVML_SUCCESS {
            let mut name_buf = [0i8; 64];
            let ret_name = nvml.get_name(dev, name_buf.as_mut_ptr(), 64);
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
            error!(
                "No matching NVML device for '{}'",
                gpu.name_gpu.as_deref().unwrap_or("unknown")
            );
            nvml.shutdown();
            return;
        }
    };

    // Обновление температуры
    let mut temp_val: c_uint = 0;
    let ret_temp = nvml.get_temperature(dev, NVML_TEMPERATURE_GPU, &mut temp_val);
    if ret_temp == NVML_SUCCESS {
        gpu.temperature = Some(temp_val as f32);
    } else {
        warn!("nvmlDeviceGetTemperature failed with code {}", ret_temp);
        gpu.temperature = None;
    }

    // Обновление использования GPU и памяти
    let mut util_data = nvmlUtilization_t { gpu: 0, memory: 0 };
    let ret_util = nvml.get_utilization_rates(dev, &mut util_data);
    if ret_util == NVML_SUCCESS {
        gpu.utilization = Some(util_data.gpu as f32);
        gpu.memory_util = Some(util_data.memory as f32);
    } else {
        warn!(
            "nvmlDeviceGetUtilizationRates failed with code {}",
            ret_util
        );
        gpu.utilization = None;
        gpu.memory_util = None;
    }

    // Обновление потребления энергии
    let mut mw: c_uint = 0;
    let ret_pow = nvml.get_power_usage(dev, &mut mw);
    if ret_pow == NVML_SUCCESS {
        gpu.power_usage = Some((mw as f32) / 1000.0);
    } else {
        warn!("nvmlDeviceGetPowerUsage failed with code {}", ret_pow);
        gpu.power_usage = None;
    }

    // Обновление текущей тактовой частоты
    let mut clk_val: c_uint = 0;
    let ret_clk = nvml.get_clock_info(dev, NVML_CLOCK_GRAPHICS, &mut clk_val);
    if ret_clk == NVML_SUCCESS {
        gpu.core_clock = Some(clk_val as u32);
    } else {
        warn!("nvmlDeviceGetClockInfo(Graphics) failed: {}", ret_clk);
        gpu.core_clock = None;
    }

    // Обновление максимальной тактовой частоты
    let mut max_clk_val: c_uint = 0;
    let ret_max_clk = nvml.get_max_clock_info(dev, NVML_CLOCK_GRAPHICS, &mut max_clk_val);
    if ret_max_clk == NVML_SUCCESS {
        gpu.max_clock_speed = Some(max_clk_val as u32);
    } else {
        warn!("nvmlDeviceGetMaxClockInfo failed: {}", ret_max_clk);
        gpu.max_clock_speed = None;
    }

    let mut max_power_val: c_uint = 0;
    let ret_max_pow = nvml.get_power_management_limit(dev, &mut max_power_val);
    if ret_max_pow == NVML_SUCCESS {
        gpu.power_limit = Some((max_power_val as f32) / 1000.0);
    } else {
        warn!("nvmlDeviceGetPowerManagementLimit failed: {}", ret_max_pow);
        gpu.power_limit = None;
    }

    gpu.active = Some(true);

    nvml.shutdown();
}

/// Retrieves information about the first detected NVIDIA GPU.
///
/// This function uses `detect_nvidia_gpus` to find all available NVIDIA GPUs
/// and selects the first one from the list. It then updates the GPU's information
/// using `update_nvidia_info`. If no NVIDIA GPUs are detected, it returns a default
/// `GpuInfo` instance with unknown or empty values.
///
/// # Returns
/// A `GpuInfo` instance containing detailed information about the first detected
/// NVIDIA GPU. If no GPUs are found, the returned instance will have default values.
///
pub fn info_gpu() -> GpuInfo {
    let mut gpus = detect_nvidia_gpus();
    if !gpus.is_empty() {
        let mut gpu = gpus.remove(0);
        update_nvidia_info(&mut gpu);
        gpu
    } else {
        error!("No NVIDIA GPUs detected.");
        GpuInfo {
            vendor: Vendor::Unknown,
            name_gpu: None,
            temperature: None,
            utilization: None,
            power_usage: None,
            core_clock: None,
            max_clock_speed: None,
            power_limit: None,
            memory_total: None,
            memory_clock: None,
            active: Some(false),
            memory_util: None,
            driver_version: None,
        }
    }
}
