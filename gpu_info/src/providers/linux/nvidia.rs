//! Linux NVIDIA GPU provider using NVML API.
//!
//! This module implements the [`GpuProvider`] trait for NVIDIA GPUs on Linux
//! using the NVML (NVIDIA Management Library) API via dynamic loading.
//!
//! # Library Loading
//!
//! The provider attempts to load NVML from:
//! 1. `NVML_LIB_PATH` environment variable (if set)
//! 2. `/usr/lib/libnvidia-ml.so.1` (default)
//!
//! # Requirements
//!
//! NVIDIA drivers must be installed with the NVML library available.
//!
//! [`GpuProvider`]: crate::gpu_info::GpuProvider

use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::vendor::Vendor;
use libloading::{Library, Symbol};
use log::{debug, error};
use std::{env, os::raw::c_char, ptr};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NvmlDevice {
    _private: [u8; 0],
}

#[allow(non_camel_case_types)]
type NvmlDevice_t = *mut NvmlDevice;

#[allow(non_camel_case_types)]
type nvmlReturn_t = i32;

const NVML_SUCCESS: nvmlReturn_t = 0;
const NVML_TEMPERATURE_GPU: u32 = 0;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NvmlUtilization {
    gpu: u32,
    memory: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
const NVML_CLOCK_GRAPHICS: u32 = 0;

/// NVIDIA GPU provider for Linux.
///
/// Implements [`GpuProvider`] for NVIDIA GPUs on Linux using the NVML API.
/// The library is loaded dynamically at runtime, allowing the provider to
/// gracefully handle systems without NVIDIA drivers.
///
/// # Supported Metrics
///
/// - Temperature (GPU core)
/// - GPU utilization percentage
/// - Memory utilization percentage
/// - Power usage (in watts)
/// - Core clock speed
/// - Memory total and used
///
/// [`GpuProvider`]: crate::gpu_info::GpuProvider
pub struct NvidiaLinuxProvider;

impl NvidiaLinuxProvider {
    /// Create a new NVIDIA Linux provider instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for NvidiaLinuxProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuProvider for NvidiaLinuxProvider {
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        debug!("Detecting NVIDIA GPUs using dynamic NVML loading on Linux");
        unsafe {
            let nvml_lib_path = env::var("NVML_LIB_PATH")
                .unwrap_or_else(|_| "/usr/lib/libnvidia-ml.so.1".to_string());
            let lib = match Library::new(&nvml_lib_path) {
                Ok(lib) => lib,
                Err(e) => {
                    error!("Failed to load NVML from {}: {e}", nvml_lib_path);
                    return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                }
            };
            let init: Symbol<NvmlInitFn> = match lib.get(b"nvmlInit_v2") {
                Ok(symbol) => symbol,
                Err(e) => {
                    error!("Failed to get nvmlInit_v2 symbol: {}", e);
                    return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                }
            };
            let shutdown: Symbol<NvmlShutdownFn> = match lib.get(b"nvmlShutdown") {
                Ok(symbol) => symbol,
                Err(e) => {
                    error!("Failed to get nvmlShutdown symbol: {}", e);
                    return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                }
            };
            let get_device_handle: Symbol<NvmlDeviceGetHandleByIndexFn> =
                match lib.get(b"nvmlDeviceGetHandleByIndex_v2") {
                    Ok(symbol) => symbol,
                    Err(e) => {
                        error!("Failed to get nvmlDeviceGetHandleByIndex_v2 symbol: {}", e);
                        return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                    }
                };
            let get_temp: Symbol<NvmlDeviceGetTemperatureFn> =
                match lib.get(b"nvmlDeviceGetTemperature") {
                    Ok(symbol) => symbol,
                    Err(e) => {
                        error!("Failed to get nvmlDeviceGetTemperature symbol: {}", e);
                        return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                    }
                };
            let get_name: Symbol<NvmlDeviceGetNameFn> = match lib.get(b"nvmlDeviceGetName") {
                Ok(symbol) => symbol,
                Err(e) => {
                    error!("Failed to get nvmlDeviceGetName symbol: {}", e);
                    return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                }
            };
            let get_util: Symbol<NvmlDeviceGetUtilizationRatesFn> =
                match lib.get(b"nvmlDeviceGetUtilizationRates") {
                    Ok(symbol) => symbol,
                    Err(e) => {
                        error!("Failed to get nvmlDeviceGetUtilizationRates symbol: {}", e);
                        return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                    }
                };
            let get_power: Symbol<NvmlDeviceGetPowerUsageFn> =
                match lib.get(b"nvmlDeviceGetPowerUsage") {
                    Ok(symbol) => symbol,
                    Err(e) => {
                        error!("Failed to get nvmlDeviceGetPowerUsage symbol: {}", e);
                        return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                    }
                };
            let get_clock: Symbol<NvmlDeviceGetClockInfoFn> =
                match lib.get(b"nvmlDeviceGetClockInfo") {
                    Ok(symbol) => symbol,
                    Err(e) => {
                        error!("Failed to get nvmlDeviceGetClockInfo symbol: {}", e);
                        return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                    }
                };
            let get_meminfo: Symbol<NvmlDeviceGetMemoryInfoFn> =
                match lib.get(b"nvmlDeviceGetMemoryInfo") {
                    Ok(symbol) => symbol,
                    Err(e) => {
                        error!("Failed to get nvmlDeviceGetMemoryInfo symbol: {}", e);
                        return Err(crate::gpu_info::GpuError::DriverNotInstalled);
                    }
                };
            init();
            let mut device: NvmlDevice_t = ptr::null_mut();
            if get_device_handle(0, &mut device) != NVML_SUCCESS {
                error!("Failed to get NVML device handle");
                shutdown();
                return Err(crate::gpu_info::GpuError::GpuNotFound);
            }
            let mut temp = 0u32;
            let temperature = if get_temp(device, NVML_TEMPERATURE_GPU, &mut temp) == NVML_SUCCESS {
                Some(temp as f32)
            } else {
                None
            };
            let mut name_buf = [0i8; 64];
            let name = if get_name(device, name_buf.as_mut_ptr(), 64) == NVML_SUCCESS {
                Some(
                    std::ffi::CStr::from_ptr(name_buf.as_ptr())
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                Some("NVIDIA GPU".to_string())
            };
            let mut util = NvmlUtilization { gpu: 0, memory: 0 };
            let (gpu_util, mem_util) = if get_util(device, &mut util) == NVML_SUCCESS {
                (Some(util.gpu as f32), Some(util.memory as f32))
            } else {
                (None, None)
            };
            let mut power = 0u32;
            let power_usage = if get_power(device, &mut power) == NVML_SUCCESS {
                Some((power as f32) / 1000.0)
            } else {
                None
            };
            let mut clock = 0u32;
            let core_clock = if get_clock(device, NVML_CLOCK_GRAPHICS, &mut clock) == NVML_SUCCESS {
                Some(clock)
            } else {
                None
            };
            let mut mem_info = NvmlMemory {
                total: 0,
                free: 0,
                used: 0,
            };
            let (memory_total, memory_used) = if get_meminfo(device, &mut mem_info) == NVML_SUCCESS
            {
                (
                    Some((mem_info.total / 1024 / 1024) as u32),
                    Some((mem_info.used / 1024 / 1024) as u32),
                )
            } else {
                (None, None)
            };
            shutdown();
            let gpu_info = GpuInfo {
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
                memory_used,
                driver_version: None,
            };
            Ok(vec![gpu_info])
        }
    }
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        let gpus = self.detect_gpus()?;
        if let Some(updated_gpu) = gpus.first() {
            *gpu = updated_gpu.clone();
        }
        Ok(())
    }

    fn get_vendor(&self) -> Vendor {
        Vendor::Nvidia
    }
}
