//! ADL API abstraction using common FFI utilities
//!
//! This module provides a clean abstraction over ADL (AMD Display Library)
//! using the common FFI utilities to reduce code duplication.
//!
//! # Safety
//!
//! This module contains unsafe FFI code for interacting with AMD's ADL library.
//! All unsafe operations are isolated here and wrapped in safe abstractions.
//!
//! # Platform Support
//!
//! ADL is only supported on Windows. On other platforms, the `get_amd_gpus()`
//! function returns an empty vector.

#[cfg(windows)]
use crate::ffi_utils::{AdlResult, ApiResult, ApiTable, DynamicLibrary};
use crate::gpu_info::GpuInfo;
#[cfg(windows)]
use crate::vendor::Vendor;
use log::error;
#[cfg(windows)]
use std::ffi::c_void;

/// ADL success return code.
pub const ADL_OK: i32 = 0;

/// Maximum path length for ADL string fields.
pub const ADL_MAX_PATH: usize = 256;

/// ADL adapter information structure.
///
/// This is a direct mirror of the C `AdapterInfo` struct from the ADL SDK.
/// Fields use the original ADL naming convention for compatibility.
#[repr(C)]
#[derive(Clone)]
#[allow(non_snake_case, missing_docs)]
pub struct AdapterInfo {
    /// Adapter name (e.g., "AMD Radeon RX 6800").
    pub strAdapterName: [u8; ADL_MAX_PATH],
    /// Display name associated with the adapter.
    pub strDisplayName: [u8; ADL_MAX_PATH],
    /// Unique adapter index used for ADL API calls.
    pub iAdapterIndex: i32,
    /// PCI bus number.
    pub iBusNumber: i32,
    /// PCI device number.
    pub iDeviceNumber: i32,
    /// PCI function number.
    pub iFunctionNumber: i32,
    /// Vendor ID (0x1002 for AMD).
    pub iVendorID: i32,
    /// Driver installation path.
    pub strDriverPath: [u8; ADL_MAX_PATH],
    /// Extended driver path.
    pub strDriverPathExt: [u8; ADL_MAX_PATH],
    /// PnP string identifier.
    pub strPNPString: [u8; ADL_MAX_PATH],
    /// Non-zero if adapter exists and is active.
    pub iExist: i32,
    /// Unique device identifier.
    pub strUDID: [u8; ADL_MAX_PATH],
    /// Total memory size in MB.
    pub iMemorySize: i32,
    /// Memory type identifier.
    pub iMemoryType: i32,
    /// Core clock speed in MHz.
    pub iCoreClock: i32,
    /// Memory clock speed in MHz.
    pub iMemoryClock: i32,
}

/// ADL temperature structure.
///
/// This is a direct mirror of the C `ADLTemperature` struct from the ADL SDK.
#[repr(C)]
#[allow(missing_docs)]
pub struct ADLTemperature {
    /// Structure size in bytes.
    pub size: i32,
    /// Temperature in millidegrees Celsius (divide by 1000 for °C).
    pub temperature: i32,
}

/// ADL power management activity structure.
///
/// This is a direct mirror of the C `ADLPMActivity` struct from the ADL SDK.
/// Contains real-time GPU activity and performance metrics.
#[repr(C)]
#[allow(missing_docs)]
pub struct ADLPMActivity {
    /// Structure size in bytes.
    pub size: i32,
    /// Current engine (core) clock in 10 kHz units.
    pub engine_clock: i32,
    /// Current memory clock in 10 kHz units.
    pub memory_clock: i32,
    /// Current VDDC voltage.
    pub vddc: i32,
    /// GPU activity percentage (0-100).
    pub activity_percent: i32,
    /// Current performance level index.
    pub current_performance_level: i32,
    /// Current bus speed.
    pub current_bus_speed: i32,
    /// Current number of bus lanes in use.
    pub current_bus_lanes: i32,
    /// Maximum number of bus lanes available.
    pub maximum_bus_lanes: i32,
    /// Reserved for future use.
    pub reserved: i32,
}

/// ADL function pointer types for Windows.
///
/// Contains function pointers to ADL library functions loaded at runtime.
/// These are direct mappings to the ADL SDK function signatures.
#[cfg(windows)]
#[allow(missing_docs)]
pub struct AdlFunctions {
    /// ADL_Main_Control_Create - Initialize ADL with memory allocation callback.
    pub main_control_create:
        unsafe extern "C" fn(Option<unsafe extern "C" fn(usize) -> *mut c_void>, i32) -> i32,
    /// ADL_Main_Control_Destroy - Shutdown ADL and free resources.
    pub main_control_destroy: unsafe extern "C" fn() -> i32,
    /// ADL_Adapter_NumberOfAdapters_Get - Get total adapter count.
    pub adapter_number_of_adapters_get: unsafe extern "C" fn(*mut i32) -> i32,
    /// ADL_Adapter_AdapterInfo_Get - Get adapter information for all adapters.
    pub adapter_adapter_info_get: unsafe extern "C" fn(*mut AdapterInfo, i32) -> i32,
    /// ADL_Overdrive5_Temperature_Get - Get adapter temperature.
    pub overdrive5_temperature_get: unsafe extern "C" fn(i32, i32, *mut ADLTemperature) -> i32,
    /// ADL_Overdrive5_CurrentActivity_Get - Get current GPU activity.
    pub overdrive5_current_activity_get: unsafe extern "C" fn(i32, *mut ADLPMActivity) -> i32,
    /// ADL_Overdrive5_PowerControl_Get - Get power control settings.
    pub overdrive5_power_control_get: unsafe extern "C" fn(i32, *mut i32, *mut i32) -> i32,
}
/// ADL API client that abstracts library loading and function calls
#[cfg(windows)]
pub struct AdlClient {
    _library: DynamicLibrary,
    api_table: ApiTable<AdlFunctions>,
    initialized: bool,
}

#[cfg(windows)]
impl AdlClient {
    /// Load ADL library and initialize API table
    pub fn new() -> Option<Self> {
        use crate::ffi_utils::{LibraryLoader, SymbolResolver};

        let library = LibraryLoader::new("atiadlxx.dll")
            .with_fallback_path("atiadlxy.dll")
            .with_fallback_path("C:\\Windows\\System32\\atiadlxx.dll")
            .load()
            .map_err(|e| {
                error!("Failed to load ADL library: {}", e);
            })
            .ok()?;
        let resolver = SymbolResolver::new(&library);
        let functions = AdlFunctions {
            main_control_create: resolver.resolve("ADL_Main_Control_Create")?,
            main_control_destroy: resolver.resolve("ADL_Main_Control_Destroy")?,
            adapter_number_of_adapters_get: resolver.resolve("ADL_Adapter_NumberOfAdapters_Get")?,
            adapter_adapter_info_get: resolver.resolve("ADL_Adapter_AdapterInfo_Get")?,
            overdrive5_temperature_get: resolver.resolve("ADL_Overdrive5_Temperature_Get")?,
            overdrive5_current_activity_get: resolver
                .resolve("ADL_Overdrive5_CurrentActivity_Get")?,
            overdrive5_power_control_get: resolver.resolve("ADL_Overdrive5_PowerControl_Get")?,
        };
        Some(Self {
            _library: library,
            api_table: ApiTable::new(functions),
            initialized: false,
        })
    }
    /// Initialize ADL
    pub fn initialize(&mut self) -> AdlResult<()> {
        if self.initialized {
            return AdlResult {
                code: ADL_OK,
                value: (),
            };
        }
        let code =
            unsafe { (self.api_table.functions().main_control_create)(Some(Self::adl_malloc), 1) };
        let result = AdlResult { code, value: () };
        if result.is_success() {
            self.initialized = true;
        }
        AdlResult { code, value: () }
    }
    /// Shutdown ADL
    pub fn shutdown(&mut self) -> AdlResult<()> {
        if !self.initialized {
            return AdlResult {
                code: ADL_OK,
                value: (),
            };
        }
        let code = unsafe { (self.api_table.functions().main_control_destroy)() };
        let result = AdlResult { code, value: () };
        if result.is_success() {
            self.initialized = false;
        }
        AdlResult { code, value: () }
    }
    /// ADL memory allocation callback
    unsafe extern "C" fn adl_malloc(size: usize) -> *mut c_void {
        std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(size, 1)) as *mut c_void
    }
    /// Get number of adapters
    pub fn get_adapter_count(&self) -> AdlResult<i32> {
        let mut count = 0;
        let code =
            unsafe { (self.api_table.functions().adapter_number_of_adapters_get)(&mut count) };
        AdlResult { code, value: count }
    }
    /// Get adapter information for all adapters
    pub fn get_adapter_info(&self, count: i32) -> AdlResult<Vec<AdapterInfo>> {
        let mut adapters = vec![unsafe { std::mem::zeroed::<AdapterInfo>() }; count as usize];
        let buffer_size = count * (std::mem::size_of::<AdapterInfo>() as i32);
        let code = unsafe {
            (self.api_table.functions().adapter_adapter_info_get)(
                adapters.as_mut_ptr(),
                buffer_size,
            )
        };
        AdlResult {
            code,
            value: adapters,
        }
    }
    /// Get adapter temperature
    pub fn get_adapter_temperature(&self, adapter_index: i32) -> AdlResult<f32> {
        let mut temperature = ADLTemperature {
            size: std::mem::size_of::<ADLTemperature>() as i32,
            temperature: 0,
        };
        let code = unsafe {
            (self.api_table.functions().overdrive5_temperature_get)(
                adapter_index,
                0,
                &mut temperature,
            )
        };
        AdlResult {
            code,
            value: (temperature.temperature as f32) / 1000.0,
        }
    }
    /// Get adapter activity
    pub fn get_adapter_activity(&self, adapter_index: i32) -> AdlResult<ADLPMActivity> {
        let mut activity = ADLPMActivity {
            size: std::mem::size_of::<ADLPMActivity>() as i32,
            engine_clock: 0,
            memory_clock: 0,
            vddc: 0,
            activity_percent: 0,
            current_performance_level: 0,
            current_bus_speed: 0,
            current_bus_lanes: 0,
            maximum_bus_lanes: 0,
            reserved: 0,
        };
        let code = unsafe {
            (self.api_table.functions().overdrive5_current_activity_get)(
                adapter_index,
                &mut activity,
            )
        };
        AdlResult {
            code,
            value: activity,
        }
    }
    /// Get adapter power control information
    pub fn get_adapter_power_info(&self, adapter_index: i32) -> AdlResult<(i32, i32)> {
        let mut current_value = 0;
        let mut default_value = 0;
        let code = unsafe {
            (self.api_table.functions().overdrive5_power_control_get)(
                adapter_index,
                &mut current_value,
                &mut default_value,
            )
        };
        AdlResult {
            code,
            value: (current_value, default_value),
        }
    }
    /// Create GpuInfo from ADL adapter
    pub fn create_gpu_info(&self, adapter: &AdapterInfo) -> Option<GpuInfo> {
        let name = std::ffi::CStr::from_bytes_until_nul(&adapter.strAdapterName)
            .ok()?
            .to_string_lossy()
            .to_string();
        let temperature = self
            .get_adapter_temperature(adapter.iAdapterIndex)
            .to_option();
        let activity = self.get_adapter_activity(adapter.iAdapterIndex).to_option();
        let (utilization, core_clock, memory_clock) = if let Some(act) = activity {
            (
                Some(act.activity_percent as f32),
                Some(act.engine_clock as u32),
                Some(act.memory_clock as u32),
            )
        } else {
            (None, None, None)
        };
        let _power_info = self
            .get_adapter_power_info(adapter.iAdapterIndex)
            .to_option();
        Some(GpuInfo {
            name_gpu: Some(name),
            vendor: Vendor::Amd,
            temperature,
            utilization,
            core_clock,
            memory_clock,
            memory_total: if adapter.iMemorySize > 0 {
                Some(adapter.iMemorySize as u32)
            } else {
                None
            },
            memory_used: None,
            memory_util: None,
            active: Some(true),
            power_usage: None,
            power_limit: None,
            driver_version: None,
            max_clock_speed: None,
        })
    }
}

#[cfg(windows)]
impl Drop for AdlClient {
    fn drop(&mut self) {
        if self.initialized {
            let _ = self.shutdown();
        }
    }
}
/// Convenience function to get all AMD GPUs using the new abstraction
#[cfg(windows)]
pub fn get_amd_gpus() -> Vec<GpuInfo> {
    use crate::handle_api_result_vec;
    let mut client = match AdlClient::new() {
        Some(client) => client,
        None => {
            error!("Failed to initialize ADL client");
            return Vec::new();
        }
    };
    handle_api_result_vec!(client.initialize(), "Failed to initialize ADL");
    let adapter_count = handle_api_result_vec!(
        client.get_adapter_count(),
        "Failed to get ADL adapter count or no adapters found"
    );
    let adapters = handle_api_result_vec!(
        client.get_adapter_info(adapter_count),
        "Failed to get ADL adapter information"
    );
    let mut gpus = Vec::new();
    for adapter in &adapters {
        if adapter.iExist != 0 && adapter.iVendorID == 0x1002 {
            if let Some(gpu_info) = client.create_gpu_info(adapter) {
                gpus.push(gpu_info);
            }
        }
    }
    gpus
}
/// Stub for non-Windows platforms
#[cfg(not(windows))]
pub fn get_amd_gpus() -> Vec<GpuInfo> {
    error!("ADL is only supported on Windows");
    Vec::new()
}
