use crate::{gpu_info::GpuInfo, vendor::Vendor};
use libloading::{Library, Symbol};
use log::{debug, error};
use std::{env, os::raw::c_char, ptr};

#[repr(C)]
#[derive(Debug)]
pub struct NvmlDevice {
    _private: [u8; 0],
}
pub type NvmlDevice_t = *mut NvmlDevice;

#[allow(non_camel_case_types)]
pub type nvmlReturn_t = i32;

pub const NVML_SUCCESS: nvmlReturn_t = 0;
pub const NVML_TEMPERATURE_GPU: u32 = 0;

#[repr(C)]
#[derive(Debug)]
struct NvmlUtilization {
    gpu: u32,
    memory: u32,
}

#[repr(C)]
#[derive(Debug)]
struct NvmlMemory {
    total: u64,
    free: u64,
    used: u64,
}

type NvmlInitFn = unsafe extern "C" fn() -> nvmlReturn_t;
type NvmlShutdownFn = unsafe extern "C" fn() -> nvmlReturn_t;
type NvmlDeviceGetHandleByIndexFn = unsafe extern "C" fn(u32, *mut NvmlDevice_t) -> nvmlReturn_t;
type NvmlDeviceGetTemperatureFn = unsafe extern "C" fn(NvmlDevice_t, u32, *mut u32) -> nvmlReturn_t;
type NvmlDeviceGetNameFn = unsafe extern "C" fn(NvmlDevice_t, *mut c_char, u32) -> nvmlReturn_t;
type NvmlDeviceGetUtilizationRatesFn =
    unsafe extern "C" fn(NvmlDevice_t, *mut NvmlUtilization) -> nvmlReturn_t;
type NvmlDeviceGetPowerUsageFn = unsafe extern "C" fn(NvmlDevice_t, *mut u32) -> nvmlReturn_t;
type NvmlDeviceGetClockInfoFn = unsafe extern "C" fn(NvmlDevice_t, u32, *mut u32) -> nvmlReturn_t;
type NvmlDeviceGetMemoryInfoFn =
    unsafe extern "C" fn(NvmlDevice_t, *mut NvmlMemory) -> nvmlReturn_t;

pub const NVML_CLOCK_GRAPHICS: u32 = 0;

pub(crate) fn info_gpu() -> GpuInfo {
    debug!("Fetching GPU info using dynamic NVML loading");

    unsafe {
        let nvml_lib_path =
            env::var("NVML_LIB_PATH").unwrap_or_else(|_| "/usr/lib/libnvidia-ml.so.1".to_string());

        let lib = match Library::new(&nvml_lib_path) {
            Ok(lib) => lib,
            Err(e) => {
                error!("Failed to load NVML from {}: {e}", nvml_lib_path);
                return GpuInfo::default();
            }
        };

        let init: Symbol<NvmlInitFn> = lib.get(b"nvmlInit_v2").unwrap();
        let shutdown: Symbol<NvmlShutdownFn> = lib.get(b"nvmlShutdown").unwrap();
        let get_device_handle: Symbol<NvmlDeviceGetHandleByIndexFn> =
            lib.get(b"nvmlDeviceGetHandleByIndex_v2").unwrap();
        let get_temp: Symbol<NvmlDeviceGetTemperatureFn> =
            lib.get(b"nvmlDeviceGetTemperature").unwrap();
        let get_name: Symbol<NvmlDeviceGetNameFn> = lib.get(b"nvmlDeviceGetName").unwrap();
        let get_util: Symbol<NvmlDeviceGetUtilizationRatesFn> =
            lib.get(b"nvmlDeviceGetUtilizationRates").unwrap();
        let get_power: Symbol<NvmlDeviceGetPowerUsageFn> =
            lib.get(b"nvmlDeviceGetPowerUsage").unwrap();
        let get_clock: Symbol<NvmlDeviceGetClockInfoFn> =
            lib.get(b"nvmlDeviceGetClockInfo").unwrap();
        let get_meminfo: Symbol<NvmlDeviceGetMemoryInfoFn> =
            lib.get(b"nvmlDeviceGetMemoryInfo").unwrap();

        (init)();

        let mut device: NvmlDevice_t = ptr::null_mut();
        if (get_device_handle)(0, &mut device) != NVML_SUCCESS {
            error!("Failed to get NVML device handle");
            (shutdown)();
            return GpuInfo::default();
        }

        let mut temp = 0u32;
        let temperature = if (get_temp)(device, NVML_TEMPERATURE_GPU, &mut temp) == NVML_SUCCESS {
            Some(temp as f32)
        } else {
            None
        };

        let mut name_buf = [0i8; 64];
        let name = if (get_name)(device, name_buf.as_mut_ptr(), 64) == NVML_SUCCESS {
            Some(
                std::ffi::CStr::from_ptr(name_buf.as_ptr())
                    .to_string_lossy()
                    .to_string(),
            )
        } else {
            Some("NVIDIA GPU".to_string())
        };

        let mut util = NvmlUtilization { gpu: 0, memory: 0 };
        let (gpu_util, mem_util) = if (get_util)(device, &mut util) == NVML_SUCCESS {
            (Some(util.gpu as f32), Some(util.memory as f32))
        } else {
            (None, None)
        };

        let mut power = 0u32;
        let power_usage = if (get_power)(device, &mut power) == NVML_SUCCESS {
            Some(power as f32 / 1000.0)
        } else {
            None
        };

        let mut clock = 0u32;
        let core_clock = if (get_clock)(device, NVML_CLOCK_GRAPHICS, &mut clock) == NVML_SUCCESS {
            Some(clock)
        } else {
            None
        };

        let mut mem_info = NvmlMemory {
            total: 0,
            free: 0,
            used: 0,
        };
        let memory_total = if (get_meminfo)(device, &mut mem_info) == NVML_SUCCESS {
            Some((mem_info.total / 1024 / 1024) as u32)
        } else {
            None
        };

        (shutdown)();

        GpuInfo {
            vendor: Vendor::Nvidia,
            name_gpu: name,
            temperature,
            utilization: gpu_util,
            memory_util: mem_util,
            power_usage,
            core_clock,
            memory_clock: None,
            max_clock_speed: None,
            active: Some(true),
            power_limit: None,
            memory_total,
            driver_version: None,
        }
    }
}
