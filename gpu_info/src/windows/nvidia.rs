use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::vendor::Vendor;
use log::error;
use std::ffi::CStr;
use std::os::raw::{c_char, c_uint};
use std::ptr;

const NVML_SUCCESS: i32 = 0;
const NVML_TEMPERATURE_GPU: i32 = 0;
const NVML_CLOCK_GRAPHICS: i32 = 0;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct nvmlDevice_st {
    _private: [u8; 0],
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct nvmlUtilization_t {
    pub(crate) gpu: c_uint,
    pub(crate) memory: c_uint,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct nvmlMemory_t {
    pub(crate) total: u64,
    pub(crate) free: u64,
    pub(crate) used: u64,
}

pub trait NvmlClient {
    #[allow(non_snake_case)]
    fn nvmlInit_v2(&self) -> i32;
    #[allow(non_snake_case)]
    fn nvmlShutdown(&self) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetCount_v2(&self, device_count: *mut c_uint) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetHandleByIndex_v2(&self, index: c_uint, device: *mut *mut nvmlDevice_st) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetName(
        &self,
        device: *mut nvmlDevice_st,
        name: *mut c_char,
        length: c_uint,
    ) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetTemperature(
        &self,
        device: *mut nvmlDevice_st,
        sensor_type: i32,
        temp: *mut c_uint,
    ) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetUtilizationRates(
        &self,
        device: *mut nvmlDevice_st,
        utilization: *mut nvmlUtilization_t,
    ) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetPowerUsage(&self, device: *mut nvmlDevice_st, power: *mut c_uint) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetClockInfo(
        &self,
        device: *mut nvmlDevice_st,
        clock_type: i32,
        clock: *mut c_uint,
    ) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetMaxClockInfo(
        &self,
        device: *mut nvmlDevice_st,
        clock_type: i32,
        clock: *mut c_uint,
    ) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetPowerManagementLimit(
        &self,
        device: *mut nvmlDevice_st,
        limit: *mut c_uint,
    ) -> i32;
    #[allow(non_snake_case)]
    fn nvmlDeviceGetMemoryInfo(&self, device: *mut nvmlDevice_st, memory: *mut nvmlMemory_t)
        -> i32;
    #[allow(non_snake_case)]
    fn nvmlSystemGetDriverVersion(&self, version: *mut c_char, length: c_uint) -> i32;
}

fn get_gpu_info(device: *mut nvmlDevice_st, nvml_client: &NvmlClientImpl) -> Option<GpuInfo> {
    let mut name = [0u8; 64];
    let ret = nvml_client.nvmlDeviceGetName(
        device,
        name.as_mut_ptr() as *mut c_char,
        name.len() as c_uint,
    );
    if ret != NVML_SUCCESS {
        error!("Failed to get device name: {}", ret);
        return None;
    }

    let mut temp: c_uint = 0;
    let ret_temp = nvml_client.nvmlDeviceGetTemperature(device, NVML_TEMPERATURE_GPU, &mut temp);
    if ret_temp != NVML_SUCCESS {
        error!("Failed to get temperature: {}", ret_temp);
        return None;
    }

    let mut util = nvmlUtilization_t { gpu: 0, memory: 0 };
    let ret_util = nvml_client.nvmlDeviceGetUtilizationRates(device, &mut util);
    if ret_util != NVML_SUCCESS {
        error!("Failed to get utilization: {}", ret_util);
        return None;
    }

    let mut power: c_uint = 0;
    let ret_pow = nvml_client.nvmlDeviceGetPowerUsage(device, &mut power);
    if ret_pow != NVML_SUCCESS {
        error!("Failed to get power usage: {}", ret_pow);
        return None;
    }

    let mut clock: c_uint = 0;
    let ret_clk = nvml_client.nvmlDeviceGetClockInfo(device, NVML_CLOCK_GRAPHICS, &mut clock);
    if ret_clk != NVML_SUCCESS {
        error!("Failed to get clock info: {}", ret_clk);
        return None;
    }

    let mut max_clock: c_uint = 0;
    let ret_max_clk =
        nvml_client.nvmlDeviceGetMaxClockInfo(device, NVML_CLOCK_GRAPHICS, &mut max_clock);
    if ret_max_clk != NVML_SUCCESS {
        error!("Failed to get max clock info: {}", ret_max_clk);
        return None;
    }

    let mut power_limit: c_uint = 0;
    let ret_limit = nvml_client.nvmlDeviceGetPowerManagementLimit(device, &mut power_limit);
    if ret_limit != NVML_SUCCESS {
        error!("Failed to get power limit: {}", ret_limit);
        return None;
    }

    let mut memory = nvmlMemory_t {
        total: 0,
        free: 0,
        used: 0,
    };
    let ret_mem = nvml_client.nvmlDeviceGetMemoryInfo(device, &mut memory);
    if ret_mem != NVML_SUCCESS {
        error!("Failed to get memory info: {}", ret_mem);
        return None;
    }

    let mut version = [0u8; 80];
    let ret_ver = nvml_client
        .nvmlSystemGetDriverVersion(version.as_mut_ptr() as *mut c_char, version.len() as c_uint);
    if ret_ver != NVML_SUCCESS {
        error!("Failed to get driver version: {}", ret_ver);
        return None;
    }

    let name = unsafe {
        CStr::from_ptr(name.as_ptr() as *const c_char)
            .to_string_lossy()
            .into_owned()
    };

    let temperature = if ret_temp == NVML_SUCCESS {
        Some(temp as f32)
    } else {
        None
    };

    let utilization = if ret_util == NVML_SUCCESS {
        Some(util.gpu as f32)
    } else {
        None
    };

    let power_usage = if ret_pow == NVML_SUCCESS {
        Some(power as f32 / 1000.0)
    } else {
        None
    };

    let core_clock = if ret_clk == NVML_SUCCESS {
        Some(clock)
    } else {
        None
    };

    let max_clock_speed = if ret_max_clk == NVML_SUCCESS {
        Some(max_clock)
    } else {
        None
    };

    let power_limit = if ret_limit == NVML_SUCCESS {
        Some(power_limit as f32 / 1000.0)
    } else {
        None
    };

    let memory_total = if ret_mem == NVML_SUCCESS {
        Some((memory.total / 1024 / 1024 / 1024) as u32)
    } else {
        None
    };

    let driver_version = if ret_ver == NVML_SUCCESS {
        Some(unsafe {
            CStr::from_ptr(version.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned()
        })
    } else {
        None
    };

    Some(GpuInfo {
        vendor: Vendor::Nvidia,
        name_gpu: Some(name),
        temperature,
        utilization,
        power_usage,
        core_clock,
        max_clock_speed,
        power_limit,
        memory_total,
        driver_version,
        active: Some(true),
        ..Default::default()
    })
}

pub fn detect_nvidia_gpus() -> Result<Vec<GpuInfo>> {
    let mut gpus = Vec::new();
    let nvml_client = NvmlClientImpl;

    let ret = nvml_client.nvmlInit_v2();
    if ret != NVML_SUCCESS {
        error!("Failed to initialize NVML: {}", ret);
        return Err(GpuError::DriverNotInstalled);
    }

    let mut device_count: c_uint = 0;
    let ret = nvml_client.nvmlDeviceGetCount_v2(&mut device_count);
    if ret != NVML_SUCCESS {
        error!("Failed to get device count: {}", ret);
        nvml_client.nvmlShutdown();
        return Err(GpuError::GpuNotFound);
    }

    for i in 0..device_count {
        let mut device: *mut nvmlDevice_st = ptr::null_mut();
        let ret = nvml_client.nvmlDeviceGetHandleByIndex_v2(i, &mut device);
        if ret != NVML_SUCCESS {
            error!("Failed to get device handle: {}", ret);
            continue;
        }

        if let Some(gpu_info) = get_gpu_info(device, &nvml_client) {
            gpus.push(gpu_info);
        }
    }

    nvml_client.nvmlShutdown();

    if gpus.is_empty() {
        Err(GpuError::GpuNotFound)
    } else {
        Ok(gpus)
    }
}

pub fn update_nvidia_info(gpu: &mut GpuInfo) -> Result<()> {
    let nvml_client = NvmlClientImpl;

    let ret = nvml_client.nvmlInit_v2();
    if ret != NVML_SUCCESS {
        error!("Failed to initialize NVML: {}", ret);
        return Err(GpuError::DriverNotInstalled);
    }

    let mut device: *mut nvmlDevice_st = ptr::null_mut();
    let ret = nvml_client.nvmlDeviceGetHandleByIndex_v2(0, &mut device);
    if ret != NVML_SUCCESS {
        error!("Failed to get device handle: {}", ret);
        return Err(GpuError::GpuNotFound);
    }

    if let Some(updated_gpu) = get_gpu_info(device, &nvml_client) {
        *gpu = updated_gpu;
    }

    nvml_client.nvmlShutdown();

    if !gpu.is_valid() {
        return Err(GpuError::GpuNotActive);
    }

    Ok(())
}

struct NvmlClientImpl;

impl NvmlClient for NvmlClientImpl {
    #[allow(non_snake_case)]
    fn nvmlInit_v2(&self) -> i32 {
        unsafe { nvmlInit_v2() }
    }

    #[allow(non_snake_case)]
    fn nvmlShutdown(&self) -> i32 {
        unsafe { nvmlShutdown() }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetCount_v2(&self, device_count: *mut c_uint) -> i32 {
        unsafe { nvmlDeviceGetCount_v2(device_count) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetHandleByIndex_v2(&self, index: c_uint, device: *mut *mut nvmlDevice_st) -> i32 {
        unsafe { nvmlDeviceGetHandleByIndex_v2(index, device) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetName(
        &self,
        device: *mut nvmlDevice_st,
        name: *mut c_char,
        length: c_uint,
    ) -> i32 {
        unsafe { nvmlDeviceGetName(device, name, length) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetTemperature(
        &self,
        device: *mut nvmlDevice_st,
        sensor_type: i32,
        temp: *mut c_uint,
    ) -> i32 {
        unsafe { nvmlDeviceGetTemperature(device, sensor_type, temp) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetUtilizationRates(
        &self,
        device: *mut nvmlDevice_st,
        utilization: *mut nvmlUtilization_t,
    ) -> i32 {
        unsafe { nvmlDeviceGetUtilizationRates(device, utilization) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetPowerUsage(&self, device: *mut nvmlDevice_st, power: *mut c_uint) -> i32 {
        unsafe { nvmlDeviceGetPowerUsage(device, power) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetClockInfo(
        &self,
        device: *mut nvmlDevice_st,
        clock_type: i32,
        clock: *mut c_uint,
    ) -> i32 {
        unsafe { nvmlDeviceGetClockInfo(device, clock_type, clock) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetMaxClockInfo(
        &self,
        device: *mut nvmlDevice_st,
        clock_type: i32,
        clock: *mut c_uint,
    ) -> i32 {
        unsafe { nvmlDeviceGetMaxClockInfo(device, clock_type, clock) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetPowerManagementLimit(
        &self,
        device: *mut nvmlDevice_st,
        limit: *mut c_uint,
    ) -> i32 {
        unsafe { nvmlDeviceGetPowerManagementLimit(device, limit) }
    }

    #[allow(non_snake_case)]
    fn nvmlDeviceGetMemoryInfo(
        &self,
        device: *mut nvmlDevice_st,
        memory: *mut nvmlMemory_t,
    ) -> i32 {
        unsafe { nvmlDeviceGetMemoryInfo(device, memory) }
    }

    #[allow(non_snake_case)]
    fn nvmlSystemGetDriverVersion(&self, version: *mut c_char, length: c_uint) -> i32 {
        unsafe { nvmlSystemGetDriverVersion(version, length) }
    }
}

#[link(name = "nvml")]
extern "C" {
    fn nvmlInit_v2() -> i32;
    fn nvmlShutdown() -> i32;
    fn nvmlDeviceGetCount_v2(device_count: *mut c_uint) -> i32;
    fn nvmlDeviceGetHandleByIndex_v2(index: c_uint, device: *mut *mut nvmlDevice_st) -> i32;
    fn nvmlDeviceGetName(device: *mut nvmlDevice_st, name: *mut c_char, length: c_uint) -> i32;
    fn nvmlDeviceGetTemperature(
        device: *mut nvmlDevice_st,
        sensor_type: i32,
        temp: *mut c_uint,
    ) -> i32;
    fn nvmlDeviceGetUtilizationRates(
        device: *mut nvmlDevice_st,
        utilization: *mut nvmlUtilization_t,
    ) -> i32;
    fn nvmlDeviceGetPowerUsage(device: *mut nvmlDevice_st, power: *mut c_uint) -> i32;
    fn nvmlDeviceGetClockInfo(
        device: *mut nvmlDevice_st,
        clock_type: i32,
        clock: *mut c_uint,
    ) -> i32;
    fn nvmlDeviceGetMaxClockInfo(
        device: *mut nvmlDevice_st,
        clock_type: i32,
        clock: *mut c_uint,
    ) -> i32;
    fn nvmlDeviceGetPowerManagementLimit(device: *mut nvmlDevice_st, limit: *mut c_uint) -> i32;
    fn nvmlDeviceGetMemoryInfo(device: *mut nvmlDevice_st, memory: *mut nvmlMemory_t) -> i32;
    fn nvmlSystemGetDriverVersion(version: *mut c_char, length: c_uint) -> i32;
}
