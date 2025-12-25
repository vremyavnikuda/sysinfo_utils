//! NVML API abstraction using common FFI utilities
//!
//! This module provides a clean abstraction over NVML (NVIDIA Management Library)
//! using the common FFI utilities to reduce code duplication.
//!
//! # Safety
//!
//! This module contains unsafe FFI code for interacting with NVIDIA's NVML library.
//! All unsafe operations are isolated here and wrapped in safe abstractions.
//!
//! # Platform Support
//!
//! NVML is supported on Windows and Linux. On macOS, NVIDIA GPUs are not supported.

use crate::ffi_utils::{
    ApiResult, ApiTable, DynamicLibrary, LibraryLoader, NvmlResult, SymbolResolver,
};
use crate::gpu_info::GpuInfo;
use crate::vendor::Vendor;
#[cfg(unix)]
use libloading::Symbol;
use log::error;
use std::ffi::{c_char, c_uint, CStr};
use std::ptr;

/// NVML success return code.
pub const NVML_SUCCESS: i32 = 0;

/// NVML temperature sensor type for GPU core temperature.
pub const NVML_TEMPERATURE_GPU: i32 = 0;

/// NVML clock type for graphics (core) clock.
pub const NVML_CLOCK_GRAPHICS: i32 = 0;

/// NVML device handle (opaque pointer).
///
/// This is an opaque type representing an NVML device handle.
/// It should only be obtained through NVML API calls and passed
/// to other NVML functions.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct nvmlDevice_st {
    _private: [u8; 0],
}

/// NVML utilization structure.
///
/// This is a direct mirror of the C `nvmlUtilization_t` struct from the NVML SDK.
/// Contains GPU and memory utilization percentages.
#[allow(non_camel_case_types, missing_docs)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct nvmlUtilization_t {
    /// GPU utilization percentage (0-100).
    pub gpu: c_uint,
    /// Memory utilization percentage (0-100).
    pub memory: c_uint,
}

/// NVML memory information structure.
///
/// This is a direct mirror of the C `nvmlMemory_t` struct from the NVML SDK.
/// Contains total, free, and used memory in bytes.
#[allow(non_camel_case_types, missing_docs)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct nvmlMemory_t {
    /// Total installed GPU memory in bytes.
    pub total: u64,
    /// Unallocated GPU memory in bytes.
    pub free: u64,
    /// Allocated GPU memory in bytes.
    pub used: u64,
}

/// NVML function pointer types for Windows.
///
/// Contains function pointers to NVML library functions loaded at runtime.
/// These are direct mappings to the NVML SDK function signatures.
#[cfg(windows)]
#[allow(missing_docs)]
pub struct NvmlFunctions {
    /// nvmlInit_v2 - Initialize NVML library.
    pub init: unsafe extern "C" fn() -> i32,
    /// nvmlShutdown - Shutdown NVML and release resources.
    pub shutdown: unsafe extern "C" fn() -> i32,
    /// nvmlDeviceGetCount_v2 - Get number of NVIDIA GPUs.
    pub device_get_count: unsafe extern "C" fn(*mut c_uint) -> i32,
    /// nvmlDeviceGetHandleByIndex_v2 - Get device handle by index.
    pub device_get_handle_by_index: unsafe extern "C" fn(c_uint, *mut *mut nvmlDevice_st) -> i32,
    /// nvmlDeviceGetName - Get device name string.
    pub device_get_name: unsafe extern "C" fn(*mut nvmlDevice_st, *mut c_char, c_uint) -> i32,
    /// nvmlDeviceGetTemperature - Get device temperature.
    pub device_get_temperature: unsafe extern "C" fn(*mut nvmlDevice_st, i32, *mut c_uint) -> i32,
    /// nvmlDeviceGetUtilizationRates - Get GPU and memory utilization.
    pub device_get_utilization_rates:
        unsafe extern "C" fn(*mut nvmlDevice_st, *mut nvmlUtilization_t) -> i32,
    /// nvmlDeviceGetPowerUsage - Get power usage in milliwatts.
    pub device_get_power_usage: unsafe extern "C" fn(*mut nvmlDevice_st, *mut c_uint) -> i32,
    /// nvmlDeviceGetClockInfo - Get current clock speed.
    pub device_get_clock_info: unsafe extern "C" fn(*mut nvmlDevice_st, i32, *mut c_uint) -> i32,
    /// nvmlDeviceGetMaxClockInfo - Get maximum clock speed.
    pub device_get_max_clock_info:
        unsafe extern "C" fn(*mut nvmlDevice_st, i32, *mut c_uint) -> i32,
    /// nvmlDeviceGetPowerManagementLimit - Get power limit in milliwatts.
    pub device_get_power_management_limit:
        unsafe extern "C" fn(*mut nvmlDevice_st, *mut c_uint) -> i32,
    /// nvmlDeviceGetMemoryInfo - Get memory information.
    pub device_get_memory_info: unsafe extern "C" fn(*mut nvmlDevice_st, *mut nvmlMemory_t) -> i32,
    /// nvmlSystemGetDriverVersion - Get driver version string.
    pub system_get_driver_version: unsafe extern "C" fn(*mut c_char, c_uint) -> i32,
}

/// Unix function pointer types for NVML.
///
/// Contains function pointers to NVML library functions loaded at runtime.
/// Uses libloading Symbol types for Unix platforms.
#[cfg(unix)]
#[allow(missing_docs)]
pub struct NvmlFunctions<'a> {
    /// nvmlInit_v2 - Initialize NVML library.
    pub init: Symbol<'a, unsafe extern "C" fn() -> i32>,
    /// nvmlShutdown - Shutdown NVML and release resources.
    pub shutdown: Symbol<'a, unsafe extern "C" fn() -> i32>,
    /// nvmlDeviceGetHandleByIndex_v2 - Get device handle by index.
    pub device_get_handle_by_index:
        Symbol<'a, unsafe extern "C" fn(u32, *mut *mut nvmlDevice_st) -> i32>,
    /// nvmlDeviceGetName - Get device name string.
    pub device_get_name:
        Symbol<'a, unsafe extern "C" fn(*mut nvmlDevice_st, *mut c_char, u32) -> i32>,
    /// nvmlDeviceGetTemperature - Get device temperature.
    pub device_get_temperature:
        Symbol<'a, unsafe extern "C" fn(*mut nvmlDevice_st, u32, *mut u32) -> i32>,
    /// nvmlDeviceGetUtilizationRates - Get GPU and memory utilization.
    pub device_get_utilization_rates:
        Symbol<'a, unsafe extern "C" fn(*mut nvmlDevice_st, *mut nvmlUtilization_t) -> i32>,
    /// nvmlDeviceGetPowerUsage - Get power usage in milliwatts.
    pub device_get_power_usage:
        Symbol<'a, unsafe extern "C" fn(*mut nvmlDevice_st, *mut u32) -> i32>,
    /// nvmlDeviceGetClockInfo - Get current clock speed.
    pub device_get_clock_info:
        Symbol<'a, unsafe extern "C" fn(*mut nvmlDevice_st, u32, *mut u32) -> i32>,
    /// nvmlDeviceGetMemoryInfo - Get memory information.
    pub device_get_memory_info:
        Symbol<'a, unsafe extern "C" fn(*mut nvmlDevice_st, *mut nvmlMemory_t) -> i32>,
}

/// NVIDIA Management Library (NVML) client for GPU monitoring.
///
/// This struct provides a safe wrapper around the NVML FFI interface,
/// handling library loading, initialization, and GPU metric collection.
///
/// # Platform Support
///
/// - **Windows**: Loads `nvml.dll` from NVIDIA driver installation
/// - **Linux**: Loads `libnvidia-ml.so` from system library paths
///
/// # Safety
///
/// All unsafe FFI calls are isolated within this module. The client
/// ensures proper initialization and cleanup of NVML resources.
#[cfg(windows)]
pub struct NvmlClient {
    _library: DynamicLibrary,
    api_table: ApiTable<NvmlFunctions>,
}

/// NVIDIA Management Library (NVML) client for GPU monitoring.
///
/// This struct provides a safe wrapper around the NVML FFI interface,
/// handling library loading, initialization, and GPU metric collection.
///
/// # Platform Support
///
/// - **Windows**: Loads `nvml.dll` from NVIDIA driver installation
/// - **Linux**: Loads `libnvidia-ml.so` from system library paths
///
/// # Safety
///
/// All unsafe FFI calls are isolated within this module. The client
/// ensures proper initialization and cleanup of NVML resources.
#[cfg(unix)]
pub struct NvmlClient {
    _library: DynamicLibrary,
    api_table: ApiTable<NvmlFunctions<'static>>,
}
#[cfg(windows)]
impl NvmlClient {
    /// Load NVML library and initialize API table
    pub fn new() -> Option<Self> {
        // Try loading from system paths
        // NVIDIA installation → System32 → System PATH
        let library = LibraryLoader::new("nvml.dll")
            .with_fallback_path("C:\\Program Files\\NVIDIA Corporation\\NVSMI\\nvml.dll")
            .with_fallback_path("C:\\Windows\\System32\\nvml.dll")
            .load()
            .map_err(|e| {
                error!("Failed to load NVML library: {}", e);
                error!("Make sure NVIDIA drivers are installed");
            })
            .ok()?;
        let resolver = SymbolResolver::new(&library);
        let functions = NvmlFunctions {
            init: resolver.resolve("nvmlInit_v2")?,
            shutdown: resolver.resolve("nvmlShutdown")?,
            device_get_count: resolver.resolve("nvmlDeviceGetCount_v2")?,
            device_get_handle_by_index: resolver.resolve("nvmlDeviceGetHandleByIndex_v2")?,
            device_get_name: resolver.resolve("nvmlDeviceGetName")?,
            device_get_temperature: resolver.resolve("nvmlDeviceGetTemperature")?,
            device_get_utilization_rates: resolver.resolve("nvmlDeviceGetUtilizationRates")?,
            device_get_power_usage: resolver.resolve("nvmlDeviceGetPowerUsage")?,
            device_get_clock_info: resolver.resolve("nvmlDeviceGetClockInfo")?,
            device_get_max_clock_info: resolver.resolve("nvmlDeviceGetMaxClockInfo")?,
            device_get_power_management_limit: resolver
                .resolve("nvmlDeviceGetPowerManagementLimit")?,
            device_get_memory_info: resolver.resolve("nvmlDeviceGetMemoryInfo")?,
            system_get_driver_version: resolver.resolve("nvmlSystemGetDriverVersion")?,
        };
        Some(Self {
            _library: library,
            api_table: ApiTable::new(functions),
        })
    }
}

#[cfg(unix)]
impl NvmlClient {
    /// Load NVML library on Unix systems
    ///
    /// Uses unsafe lifetime extension to store Symbol<'static> in the struct.
    /// This is safe because the library is owned by the struct and will live
    /// as long as the struct itself.
    pub fn new() -> Option<Self> {
        let nvml_path = std::env::var("NVML_LIB_PATH")
            .unwrap_or_else(|_| "/usr/lib/libnvidia-ml.so.1".to_string());
        let library = LibraryLoader::new(&nvml_path)
            .with_fallback_path("/usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1")
            .with_fallback_path("/usr/lib64/libnvidia-ml.so.1")
            .load()
            .map_err(|e| {
                error!("Failed to load NVML library: {}", e);
            })
            .ok()?;

        let resolver = SymbolResolver::new(&library);

        let functions = NvmlFunctions {
            init: resolver.resolve(b"nvmlInit_v2")?,
            shutdown: resolver.resolve(b"nvmlShutdown")?,
            device_get_handle_by_index: resolver.resolve(b"nvmlDeviceGetHandleByIndex_v2")?,
            device_get_name: resolver.resolve(b"nvmlDeviceGetName")?,
            device_get_temperature: resolver.resolve(b"nvmlDeviceGetTemperature")?,
            device_get_utilization_rates: resolver.resolve(b"nvmlDeviceGetUtilizationRates")?,
            device_get_power_usage: resolver.resolve(b"nvmlDeviceGetPowerUsage")?,
            device_get_clock_info: resolver.resolve(b"nvmlDeviceGetClockInfo")?,
            device_get_memory_info: resolver.resolve(b"nvmlDeviceGetMemoryInfo")?,
        };

        // SAFETY: We extend the lifetime of Symbol to 'static.
        // This is safe because:
        // 1. The library is owned by this struct
        // 2. The library will live as long as this struct
        // 3. The Symbol pointers are valid as long as the library is loaded
        let functions_static: NvmlFunctions<'static> = unsafe { std::mem::transmute(functions) };

        Some(Self {
            _library: library,
            api_table: ApiTable::new(functions_static),
        })
    }
}

impl NvmlClient {
    /// Initialize NVML
    pub fn initialize(&self) -> NvmlResult<()> {
        let code = unsafe { (self.api_table.functions().init)() };
        NvmlResult { code, value: () }
    }
    /// Shutdown NVML
    pub fn shutdown(&self) -> NvmlResult<()> {
        let code = unsafe { (self.api_table.functions().shutdown)() };
        NvmlResult { code, value: () }
    }
    /// Get device count
    #[cfg(windows)]
    pub fn get_device_count(&self) -> NvmlResult<u32> {
        let mut count = 0;
        let code = unsafe { (self.api_table.functions().device_get_count)(&mut count) };
        NvmlResult { code, value: count }
    }
    /// Get device handle by index
    #[cfg(windows)]
    pub fn get_device_handle(&self, index: u32) -> NvmlResult<*mut nvmlDevice_st> {
        let mut device = ptr::null_mut();
        let code =
            unsafe { (self.api_table.functions().device_get_handle_by_index)(index, &mut device) };
        NvmlResult {
            code,
            value: device,
        }
    }
    /// Get device handle by index on Unix
    #[cfg(unix)]
    pub fn get_device_handle(&self, index: u32) -> NvmlResult<*mut nvmlDevice_st> {
        let mut device = ptr::null_mut();
        let code =
            unsafe { (self.api_table.functions().device_get_handle_by_index)(index, &mut device) };
        NvmlResult {
            code,
            value: device,
        }
    }
    /// Get device name
    ///
    /// # Safety
    /// The caller must ensure that `device` is a valid NVML device handle.
    pub unsafe fn get_device_name(&self, device: *mut nvmlDevice_st) -> NvmlResult<String> {
        let mut name_buf = [0u8; 64];
        let code = unsafe {
            #[cfg(windows)]
            {
                (self.api_table.functions().device_get_name)(
                    device,
                    name_buf.as_mut_ptr() as *mut c_char,
                    name_buf.len() as c_uint,
                )
            }
            #[cfg(unix)]
            {
                (self.api_table.functions().device_get_name)(
                    device,
                    name_buf.as_mut_ptr() as *mut c_char,
                    name_buf.len() as u32,
                )
            }
        };
        let name = if code == NVML_SUCCESS {
            CStr::from_bytes_until_nul(&name_buf)
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        };
        NvmlResult { code, value: name }
    }
    /// Get device temperature
    ///
    /// # Safety
    /// The caller must ensure that `device` is a valid NVML device handle.
    pub unsafe fn get_device_temperature(&self, device: *mut nvmlDevice_st) -> NvmlResult<f32> {
        let mut temp = 0u32;
        let code = unsafe {
            #[cfg(windows)]
            {
                (self.api_table.functions().device_get_temperature)(
                    device,
                    NVML_TEMPERATURE_GPU,
                    &mut temp,
                )
            }
            #[cfg(unix)]
            {
                (self.api_table.functions().device_get_temperature)(
                    device,
                    NVML_TEMPERATURE_GPU as u32,
                    &mut temp,
                )
            }
        };
        NvmlResult {
            code,
            value: temp as f32,
        }
    }
    /// Get device utilization rates
    ///
    /// # Safety
    /// The caller must ensure that `device` is a valid NVML device handle.
    pub unsafe fn get_device_utilization(
        &self,
        device: *mut nvmlDevice_st,
    ) -> NvmlResult<(f32, f32)> {
        let mut util = nvmlUtilization_t { gpu: 0, memory: 0 };
        let code =
            unsafe { (self.api_table.functions().device_get_utilization_rates)(device, &mut util) };
        NvmlResult {
            code,
            value: (util.gpu as f32, util.memory as f32),
        }
    }
    /// Get device power usage
    ///
    /// # Safety
    /// The caller must ensure that `device` is a valid NVML device handle.
    pub unsafe fn get_device_power_usage(&self, device: *mut nvmlDevice_st) -> NvmlResult<f32> {
        let mut power = 0u32;
        let code =
            unsafe { (self.api_table.functions().device_get_power_usage)(device, &mut power) };
        NvmlResult {
            code,
            // mW to W
            value: (power as f32) / 1000.0,
        }
    }
    /// Get device clock info
    ///
    /// # Safety
    /// The caller must ensure that `device` is a valid NVML device handle.
    pub unsafe fn get_device_clock_info(&self, device: *mut nvmlDevice_st) -> NvmlResult<u32> {
        let mut clock = 0u32;
        let code = unsafe {
            #[cfg(windows)]
            {
                (self.api_table.functions().device_get_clock_info)(
                    device,
                    NVML_CLOCK_GRAPHICS,
                    &mut clock,
                )
            }
            #[cfg(unix)]
            {
                (self.api_table.functions().device_get_clock_info)(
                    device,
                    NVML_CLOCK_GRAPHICS as u32,
                    &mut clock,
                )
            }
        };
        NvmlResult { code, value: clock }
    }
    /// Get device memory info
    ///
    /// # Safety
    /// The caller must ensure that `device` is a valid NVML device handle.
    pub unsafe fn get_device_memory_info(
        &self,
        device: *mut nvmlDevice_st,
    ) -> NvmlResult<(u64, u64, u64)> {
        let mut memory = nvmlMemory_t {
            total: 0,
            free: 0,
            used: 0,
        };
        let code =
            unsafe { (self.api_table.functions().device_get_memory_info)(device, &mut memory) };
        NvmlResult {
            code,
            value: (memory.total, memory.free, memory.used),
        }
    }
    /// Create GpuInfo from NVML device
    ///
    /// # Safety
    /// The caller must ensure that `device` is a valid NVML device handle.
    pub unsafe fn create_gpu_info(&self, device: *mut nvmlDevice_st) -> Option<GpuInfo> {
        use crate::handle_api_result;
        let name = handle_api_result!(self.get_device_name(device), "Failed to get device name");
        let temperature = handle_api_result!(
            self.get_device_temperature(device),
            "Failed to get device temperature"
        );
        let (gpu_util, mem_util) = handle_api_result!(
            self.get_device_utilization(device),
            "Failed to get device utilization"
        );
        let power_usage = handle_api_result!(
            self.get_device_power_usage(device),
            "Failed to get device power usage"
        );
        let core_clock = handle_api_result!(
            self.get_device_clock_info(device),
            "Failed to get device clock info"
        );
        let (total_memory, _free_memory, used_memory) = handle_api_result!(
            self.get_device_memory_info(device),
            "Failed to get device memory info"
        );
        Some(GpuInfo {
            name_gpu: Some(name),
            vendor: Vendor::Nvidia,
            temperature: Some(temperature),
            utilization: Some(gpu_util),
            memory_util: Some(mem_util),
            power_usage: Some(power_usage),
            core_clock: Some(core_clock),
            // Convert bytes to MB
            memory_total: Some((total_memory / (1024 * 1024)) as u32),
            // Convert bytes to MB
            memory_used: Some((used_memory / (1024 * 1024)) as u32),
            // Not available in this version
            memory_clock: None,
            active: Some(true),
            // Could be added later
            power_limit: None,
            // Could be added later
            driver_version: None,
            // Could be added later
            max_clock_speed: None,
        })
    }
}
/// Convenience function to get all NVIDIA GPUs using the new abstraction
pub fn get_nvidia_gpus() -> Vec<GpuInfo> {
    let client = match NvmlClient::new() {
        Some(client) => client,
        None => {
            error!("Failed to initialize NVML client");
            return Vec::new();
        }
    };
    if client.initialize().to_option().is_none() {
        error!("Failed to initialize NVML");
        return Vec::new();
    }
    #[cfg(windows)]
    {
        let count = match client.get_device_count().to_option() {
            Some(count) => count,
            None => {
                error!("Failed to get NVML device count");
                client.shutdown();
                return Vec::new();
            }
        };
        let mut gpus = Vec::new();
        for i in 0..count {
            if let Some(device) = client.get_device_handle(i).to_option() {
                if let Some(gpu_info) = unsafe { client.create_gpu_info(device) } {
                    gpus.push(gpu_info);
                }
            }
        }
        client.shutdown();
        gpus
    }
    #[cfg(unix)]
    {
        if let Some(device) = client.get_device_handle(0).to_option() {
            if let Some(gpu_info) = unsafe { client.create_gpu_info(device) } {
                client.shutdown();
                return vec![gpu_info];
            }
        }
        client.shutdown();
        Vec::new()
    }
}
