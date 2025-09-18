//! ADL API abstraction using common FFI utilities
//!
//! This module provides a clean abstraction over ADL (AMD Display Library)
//! using the common FFI utilities to reduce code duplication.

use crate::ffi_utils::{
    AdlResult, ApiResult, ApiTable, DynamicLibrary, LibraryLoader, SymbolResolver,
};
use crate::gpu_info::GpuInfo;
use crate::vendor::Vendor;
use log::error;
use std::ffi::c_void;

/// ADL constants
pub const ADL_OK: i32 = 0;
pub const ADL_MAX_PATH: usize = 256;

/// ADL adapter information structure
#[repr(C)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct AdapterInfo {
    pub strAdapterName: [u8; ADL_MAX_PATH],
    pub strDisplayName: [u8; ADL_MAX_PATH],
    pub iAdapterIndex: i32,
    pub iBusNumber: i32,
    pub iDeviceNumber: i32,
    pub iFunctionNumber: i32,
    pub iVendorID: i32,
    pub strDriverPath: [u8; ADL_MAX_PATH],
    pub strDriverPathExt: [u8; ADL_MAX_PATH],
    pub strPNPString: [u8; ADL_MAX_PATH],
    pub iExist: i32,
    pub strUDID: [u8; ADL_MAX_PATH],
    // Simplified structure - in real ADL this has many more clock fields
    pub iMemorySize: i32,
    pub iMemoryType: i32,
    pub iCoreClock: i32,
    pub iMemoryClock: i32,
}

/// ADL temperature structure
#[repr(C)]
pub struct ADLTemperature {
    pub size: i32,
    pub temperature: i32,
}

/// ADL power management activity structure
#[repr(C)]
pub struct ADLPMActivity {
    pub size: i32,
    pub engine_clock: i32,
    pub memory_clock: i32,
    pub vddc: i32,
    pub activity_percent: i32,
    pub current_performance_level: i32,
    pub current_bus_speed: i32,
    pub current_bus_lanes: i32,
    pub maximum_bus_lanes: i32,
    pub reserved: i32,
}

/// ADL function pointer types for Windows
#[cfg(windows)]
pub struct AdlFunctions {
    pub main_control_create:
        unsafe extern "C" fn(Option<unsafe extern "C" fn(usize) -> *mut c_void>, i32) -> i32,
    pub main_control_destroy: unsafe extern "C" fn() -> i32,
    pub adapter_number_of_adapters_get: unsafe extern "C" fn(*mut i32) -> i32,
    pub adapter_adapter_info_get: unsafe extern "C" fn(*mut AdapterInfo, i32) -> i32,
    pub overdrive5_temperature_get: unsafe extern "C" fn(i32, i32, *mut ADLTemperature) -> i32,
    pub overdrive5_current_activity_get: unsafe extern "C" fn(i32, *mut ADLPMActivity) -> i32,
    pub overdrive5_power_control_get: unsafe extern "C" fn(i32, *mut i32, *mut i32) -> i32,
}

/// ADL API client that abstracts library loading and function calls
pub struct AdlClient {
    _library: DynamicLibrary,
    api_table: ApiTable<AdlFunctions>,
    initialized: bool,
}

impl AdlClient {
    /// Load ADL library and initialize API table
    #[cfg(windows)]
    pub fn new() -> Option<Self> {
        // Try to load ADL library with fallback paths
        let library = LibraryLoader::new("atiadlxx.dll")
            .with_fallback_path("atiadlxy.dll") // Alternative ADL library name
            .with_fallback_path("C:\\Windows\\System32\\atiadlxx.dll")
            .load()
            .map_err(|e| {
                error!("Failed to load ADL library: {}", e);
            })
            .ok()?;

        let resolver = SymbolResolver::new(&library);

        // Resolve all ADL functions
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

        let code = unsafe {
            (self.api_table.functions().main_control_create)(
                Some(Self::adl_malloc), // Memory allocation callback
                1,                      // ADL version
            )
        };

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
        let buffer_size = count * std::mem::size_of::<AdapterInfo>() as i32;

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
                0, // Thermal controller index
                &mut temperature,
            )
        };

        AdlResult {
            code,
            value: temperature.temperature as f32 / 1000.0, // Convert from millidegrees
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
        // Extract adapter name
        let name = std::ffi::CStr::from_bytes_until_nul(&adapter.strAdapterName)
            .ok()?
            .to_string_lossy()
            .to_string();

        // Get temperature
        let temperature = self
            .get_adapter_temperature(adapter.iAdapterIndex)
            .to_option();

        // Get activity information
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

        // Get power information (optional)
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
                Some((adapter.iMemorySize as u32 / 1024).max(1)) // Convert MB to GB, minimum 1GB
            } else {
                None
            },
            memory_util: None, // Not easily available from ADL
            active: Some(true),
            power_usage: None, // Could be added with power control info
            power_limit: None,
            driver_version: None,
            max_clock_speed: None,
        })
    }
}

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
    let mut client = match AdlClient::new() {
        Some(client) => client,
        None => {
            error!("Failed to initialize ADL client");
            return Vec::new();
        }
    };
    if client.initialize().to_option().is_none() {
        error!("Failed to initialize ADL");
        return Vec::new();
    }
    let adapter_count = match client.get_adapter_count().to_option() {
        Some(count) if count > 0 => count,
        _ => {
            error!("Failed to get ADL adapter count or no adapters found");
            return Vec::new();
        }
    };
    let adapters = match client.get_adapter_info(adapter_count).to_option() {
        Some(adapters) => adapters,
        None => {
            error!("Failed to get ADL adapter information");
            return Vec::new();
        }
    };
    let mut gpus = Vec::new();
    for adapter in &adapters {
        // Only include active adapters
        if adapter.iExist != 0 && adapter.iVendorID == 0x1002 {
            // AMD vendor ID
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
