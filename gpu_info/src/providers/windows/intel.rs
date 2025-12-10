//! Windows Intel GPU provider implementation - Unified Metrics Collection
//!
//! This module is the **single entry point** for all Intel GPU metrics on Windows.
//! It implements a three-tier fallback strategy to maximize metric availability.
//!
//! # Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │          IntelWindowsProvider (Public API)                  │
//! │  Single unified interface for all Intel GPU metrics         │
//! └─────────────────────────────────────────────────────────────┘
//!                            │
//!          ┌─────────────────┼─────────────────┐
//!          ▼                 ▼                 ▼
//!   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
//!   │   WMI API   │   │  Intel MD   │   │   PDH API   │
//!   │  (Basic)    │   │   API       │   │ (Fallback)  │
//!   └─────────────┘   └─────────────┘   └─────────────┘
//!        │                  │                  │
//!        ▼                  ▼                  ▼
//!   GPU Name          Temperature        Utilization
//!   Driver Ver        Power Usage        Memory Usage
//!   Memory Total      Frequency
//! ```
//!
//! # Three-Tier Strategy
//!
//! 1. **WMI (Windows Management Instrumentation)** - Basic GPU detection
//!    - GPU name, vendor, driver version
//!    - Total memory size
//!    - Always available, no special drivers needed
//!
//! 2. **Intel Metrics Discovery API (igdmd64.dll)** - Advanced metrics
//!    - Temperature, power usage, frequency
//!    - Most accurate, lowest latency (~10-50ms)
//!    - Requires Intel Graphics Driver with MD support
//!    - DLL Location: `C:\Windows\System32\DriverStore\FileRepository\iigd_dch.inf_amd64_*\igdmd64.dll`
//!
//! 3. **PDH (Performance Data Helper)** - Fallback for utilization/memory
//!    - GPU utilization percentage (when MD API unavailable)
//!    - Memory usage in MB
//!    - Slower (~500ms) but widely available
//!    - Used via internal `pdh.rs` module
//!
//! # Metric Collection Flow
//!
//! ```text
//! detect_gpus() / update_gpu()
//!   │
//!   ├─► get_basic_gpu_info()        [WMI]
//!   │     └─► Name, Driver, Memory Total
//!   │
//!   ├─► enhance_with_md_api()       [Intel MD API]
//!   │     ├─► Temperature  ✓ Primary
//!   │     ├─► Power Usage  ✓ Primary
//!   │     ├─► Frequency    ✓ Primary
//!   │     └─► Utilization  ✓ Primary (if available)
//!   │
//!   └─► enhance_with_pdh()          [PDH Fallback]
//!         ├─► Utilization  ✓ Fallback (if MD API failed)
//!         └─► Memory Usage ✓ Primary (MD API doesn't provide)
//! ```
//!
//! # Why This Design?
//!
//! - **Centralized**: All Intel GPU logic in one place (`intel.rs`)
//! - **Layered**: Clear separation between APIs (WMI → MD → PDH)
//! - **Resilient**: Graceful fallback when advanced APIs unavailable
//! - **Maintainable**: PDH is internal utility, not exposed to callers
//!
//! # References
//! - Intel MD API: https://github.com/intel/metrics-discovery
//! - Header: https://github.com/intel/metrics-discovery/blob/master/instrumentation/metrics_discovery/common/inc/metrics_discovery_api.h

// Allow dead_code for FFI infrastructure that will be used in future enhancements
use crate::gpu_info::{GpuError, GpuInfo, GpuProvider, Result};
use crate::vendor::{IntelGpuType, Vendor};
use libloading::Library;
use log::{debug, error, info, warn};
use std::path::{Path, PathBuf};
use std::ptr;

// Constants from metrics_discovery_api.h
/// Return codes (TCompletionCode)
const CC_OK: i32 = 0;
const CC_ERROR_GENERAL: i32 = 42;
const CC_ERROR_INVALID_PARAMETER: i32 = 40;
const CC_ERROR_NOT_SUPPORTED: i32 = 44;

/// Legacy return codes (for compatibility)
const MD_SUCCESS: i32 = CC_OK;
const MD_ERROR_GENERAL: i32 = -1;
const MD_ERROR_INVALID_PARAMETER: i32 = -2;
const MD_ERROR_NOT_SUPPORTED: i32 = -3;

// Parameter structures

/// Metrics Device parameters
#[repr(C)]
#[allow(non_camel_case_types)]
struct TMetricsDeviceParams_1_0 {
    pub major_number: u32,
    pub minor_number: u32,
    pub build_number: u32,
    pub concurrent_groups_count: u32,
    pub global_symbols_count: u32,
    pub delta_functions_count: u32,
    pub equation_element_types_count: u32,
    pub equation_operations_count: u32,
    pub device_name: *const i8,
}

/// Concurrent Group parameters
#[repr(C)]
#[allow(non_camel_case_types)]
struct TConcurrentGroupParams_1_0 {
    pub symbol_name: *const i8,
    pub description: *const i8,
    pub measurement_type_mask: u32,
    pub metric_sets_count: u32,
    pub io_measurement_information_count: u32,
    pub io_gpu_context_information_count: u32,
}

/// API specific identifiers for different graphics APIs
#[repr(C)]
#[allow(non_camel_case_types)]
struct TApiSpecificId_1_0 {
    pub d3d9_query_id: u32,
    pub d3d9_fourcc: u32,
    pub d3d1x_query_id: u32,
    pub d3d1x_dev_dependent_id: u32,
    pub d3d1x_dev_dependent_name: *const i8,
    pub ogl_query_intel_id: u32,
    pub ogl_query_intel_name: *const i8,
    pub ogl_query_arb_target_id: u32,
    pub ocl: u32,
    pub hw_config_id: u32,
    pub placeholder: [u32; 1],
}

/// Metric Set parameters
#[repr(C)]
#[allow(non_camel_case_types)]
struct TMetricSetParams_1_0 {
    pub symbol_name: *const i8,
    pub short_name: *const i8,
    pub api_mask: u32,
    pub category_mask: u32,
    pub raw_report_size: u32,
    pub query_report_size: u32,
    pub metrics_count: u32,
    pub information_count: u32,
    pub complementary_sets_count: u32,
    pub api_specific_id: TApiSpecificId_1_0,
    pub platform_mask: u32,
}

/// Metric parameters
#[repr(C)]
#[allow(non_camel_case_types)]
struct TMetricParams_1_0 {
    pub id_in_set: u32,
    pub group_id: u32,
    pub symbol_name: *const i8,
    pub short_name: *const i8,
    pub group_name: *const i8,
    pub long_name: *const i8,
    pub dx_to_ogl_alias: *const i8,
    pub usage_flags_mask: u32,
    pub api_mask: u32,
    pub result_type: u32,
    pub metric_result_units: *const i8,
    pub metric_type: u32,
    // Additional fields omitted for simplicity
}

/// Typed value union
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
union TTypedValueData {
    pub value_uint32: u32,
    pub value_uint64: u64,
    pub value_float: f32,
    pub value_bool: bool,
    pub value_cstring: *const i8,
}

/// Typed value structure
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
struct TTypedValue_1_0 {
    pub value_type: u32,
    pub value_data: TTypedValueData,
}

// COM-like vtable structures
/// IMetricsDevice_1_0 vtable
#[repr(C)]
#[allow(non_camel_case_types)]
struct IMetricsDevice_1_0_Vtbl {
    pub destructor: unsafe extern "C" fn(*mut IMetricsDevice_1_0),
    pub get_params: unsafe extern "C" fn(*mut IMetricsDevice_1_0) -> *mut TMetricsDeviceParams_1_0,
    pub get_concurrent_group:
        unsafe extern "C" fn(*mut IMetricsDevice_1_0, u32) -> *mut IConcurrentGroup_1_0,
    pub get_global_symbol: unsafe extern "C" fn(*mut IMetricsDevice_1_0, u32) -> *mut u8,
    pub get_global_symbol_value_by_name:
        unsafe extern "C" fn(*mut IMetricsDevice_1_0, *const i8) -> *mut u8,
    pub get_last_error: unsafe extern "C" fn(*mut IMetricsDevice_1_0) -> i32,
    pub get_gpu_cpu_timestamps:
        unsafe extern "C" fn(*mut IMetricsDevice_1_0, *mut u64, *mut u64, *mut u32) -> i32,
}

/// IMetricsDevice_1_0 with vtable pointer
#[repr(C)]
#[allow(non_camel_case_types)]
struct IMetricsDevice_1_0 {
    vtbl: *const IMetricsDevice_1_0_Vtbl,
}

/// IConcurrentGroup_1_0 vtable
#[repr(C)]
#[allow(non_camel_case_types)]
struct IConcurrentGroup_1_0_Vtbl {
    pub destructor: unsafe extern "C" fn(*mut IConcurrentGroup_1_0),
    pub get_params:
        unsafe extern "C" fn(*mut IConcurrentGroup_1_0) -> *mut TConcurrentGroupParams_1_0,
    pub get_metric_set: unsafe extern "C" fn(*mut IConcurrentGroup_1_0, u32) -> *mut IMetricSet_1_0,
    pub open_io_stream: unsafe extern "C" fn(
        *mut IConcurrentGroup_1_0,
        *mut IMetricSet_1_0,
        u32,
        *mut u32,
        *mut u32,
    ) -> i32,
    pub read_io_stream:
        unsafe extern "C" fn(*mut IConcurrentGroup_1_0, *mut u32, *mut u8, u32) -> i32,
    pub close_io_stream: unsafe extern "C" fn(*mut IConcurrentGroup_1_0) -> i32,
    pub wait_for_reports: unsafe extern "C" fn(*mut IConcurrentGroup_1_0, u32) -> i32,
    // Additional methods omitted for simplicity
}

/// IConcurrentGroup_1_0 with vtable pointer
#[repr(C)]
#[allow(non_camel_case_types)]
struct IConcurrentGroup_1_0 {
    vtbl: *const IConcurrentGroup_1_0_Vtbl,
}

/// IMetricSet_1_0 vtable
#[repr(C)]
#[allow(non_camel_case_types)]
struct IMetricSet_1_0_Vtbl {
    pub destructor: unsafe extern "C" fn(*mut IMetricSet_1_0),
    pub get_params: unsafe extern "C" fn(*mut IMetricSet_1_0) -> *mut TMetricSetParams_1_0,
    pub get_metric: unsafe extern "C" fn(*mut IMetricSet_1_0, u32) -> *mut IMetric_1_0,
    pub get_information: unsafe extern "C" fn(*mut IMetricSet_1_0, u32) -> *mut u8,
    pub get_complementary_metric_set:
        unsafe extern "C" fn(*mut IMetricSet_1_0, u32) -> *mut IMetricSet_1_0,
    pub activate: unsafe extern "C" fn(*mut IMetricSet_1_0) -> i32,
    pub deactivate: unsafe extern "C" fn(*mut IMetricSet_1_0) -> i32,
    pub add_custom_metric: unsafe extern "C" fn(*mut IMetricSet_1_0) -> *mut IMetric_1_0,
}

/// IMetricSet_1_0 with vtable pointer
#[repr(C)]
#[allow(non_camel_case_types)]
struct IMetricSet_1_0 {
    vtbl: *const IMetricSet_1_0_Vtbl,
}

/// IMetric_1_0 vtable
#[repr(C)]
#[allow(non_camel_case_types)]
struct IMetric_1_0_Vtbl {
    pub destructor: unsafe extern "C" fn(*mut IMetric_1_0),
    pub get_params: unsafe extern "C" fn(*mut IMetric_1_0) -> *mut TMetricParams_1_0,
}

/// IMetric_1_0 with vtable pointer
#[repr(C)]
#[allow(non_camel_case_types)]
struct IMetric_1_0 {
    vtbl: *const IMetric_1_0_Vtbl,
}

/// IMetricSet_1_1 vtable (extends IMetricSet_1_0 with CalculateMetrics)
#[repr(C)]
#[allow(non_camel_case_types)]
struct IMetricSet_1_1_Vtbl {
    pub base: IMetricSet_1_0_Vtbl,
    pub set_api_filtering: unsafe extern "C" fn(*mut IMetricSet_1_1, u32) -> i32,
    pub calculate_metrics: unsafe extern "C" fn(
        *mut IMetricSet_1_1,
        *const u8,
        u32,
        *mut TTypedValue_1_0,
        u32,
        *mut u32,
        bool,
    ) -> i32,
    pub calculate_io_measurement_information:
        unsafe extern "C" fn(*mut IMetricSet_1_1, *mut TTypedValue_1_0, u32) -> i32,
}

/// IMetricSet_1_1 with vtable pointer
#[repr(C)]
#[allow(non_camel_case_types)]
struct IMetricSet_1_1 {
    vtbl: *const IMetricSet_1_1_Vtbl,
}

// Function pointer types

/// OpenMetricsDevice function signature
///
/// Opens a metrics device for the specified adapter.
///
/// # Parameters
/// - `device`: Output pointer to receive the device interface
///
/// # Returns
/// - `CC_OK` on success
/// - Error code on failure
type OpenMetricsDeviceFn = unsafe extern "C" fn(device: *mut *mut IMetricsDevice_1_0) -> i32;

/// CloseMetricsDevice function signature
///
/// Closes and releases a metrics device.
///
/// # Parameters
/// - `device`: Pointer to the device interface to close
///
/// # Returns
/// - `CC_OK` on success
/// - Error code on failure
type CloseMetricsDeviceFn = unsafe extern "C" fn(device: *mut IMetricsDevice_1_0) -> i32;

// Intel Metrics Discovery API wrapper

/// Intel Metrics Discovery API client
///
/// Provides safe access to Intel GPU metrics through the Metrics Discovery API.
pub struct IntelMetricsApi {
    _library: Library,
    open_device: OpenMetricsDeviceFn,
    close_device: CloseMetricsDeviceFn,
}

impl IntelMetricsApi {
    /// Try to load Intel Metrics Discovery API from known locations
    ///
    /// Searches for igdmd64.dll in the Intel Graphics Driver directory.
    pub fn new() -> Result<Self> {
        // Search for igdmd64.dll in DriverStore
        let driver_store = PathBuf::from(r"C:\Windows\System32\DriverStore\FileRepository");

        if !driver_store.exists() {
            error!("DriverStore directory not found");
            return Err(GpuError::DriverNotInstalled);
        }

        // Search for Intel Graphics Driver directories
        let entries = std::fs::read_dir(&driver_store).map_err(|e| {
            error!("Failed to read DriverStore: {}", e);
            GpuError::DriverNotInstalled
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Look for Intel Graphics Driver directories
                if name.starts_with("iigd_dch") || name.starts_with("igdlh64") {
                    let dll_path = path.join("igdmd64.dll");
                    if dll_path.exists() {
                        info!("Found Intel Metrics Discovery API at: {:?}", dll_path);
                        return Self::load_from_path(&dll_path);
                    }
                }
            }
        }

        error!("Intel Metrics Discovery API (igdmd64.dll) not found");
        Err(GpuError::DriverNotInstalled)
    }

    /// Load the API from a specific path
    fn load_from_path(path: &Path) -> Result<Self> {
        let library = unsafe {
            Library::new(path).map_err(|e| {
                error!("Failed to load igdmd64.dll: {}", e);
                GpuError::DriverNotInstalled
            })?
        };

        info!("Successfully loaded Intel Metrics Discovery API");

        // Load function symbols as raw function pointers
        // SAFETY: Function pointers are 'static by nature and valid as long as the library is loaded.
        // The library is owned by this struct, ensuring the functions remain valid.
        let open_device: OpenMetricsDeviceFn = unsafe {
            *library
                .get::<OpenMetricsDeviceFn>(b"OpenMetricsDevice")
                .map_err(|e| {
                    error!("Failed to load OpenMetricsDevice: {}", e);
                    GpuError::DriverNotInstalled
                })?
        };

        let close_device: CloseMetricsDeviceFn = unsafe {
            *library
                .get::<CloseMetricsDeviceFn>(b"CloseMetricsDevice")
                .map_err(|e| {
                    error!("Failed to load CloseMetricsDevice: {}", e);
                    GpuError::DriverNotInstalled
                })?
        };

        debug!("All Intel MD API functions loaded successfully");

        Ok(Self {
            _library: library,
            open_device,
            close_device,
        })
    }

    /// Open a metrics device
    ///
    /// # Returns
    /// Pointer to the metrics device interface on success.
    ///
    /// # Safety
    /// The returned pointer is valid until `close_device()` is called.
    /// Prefer using `IntelMetricsDevice` wrapper for automatic lifetime management.
    ///
    /// # Errors
    /// Returns error if the API call fails or returns a null pointer.
    fn open_device(&self) -> Result<*mut IMetricsDevice_1_0> {
        let mut device: *mut IMetricsDevice_1_0 = ptr::null_mut();

        let result = unsafe { (self.open_device)(&mut device) };

        if result != CC_OK && result != MD_SUCCESS {
            error!("OpenMetricsDevice failed with code: {}", result);
            return Err(Self::map_md_error(result, "OpenMetricsDevice"));
        }

        if device.is_null() {
            error!("OpenMetricsDevice returned null device");
            return Err(GpuError::GpuNotFound);
        }

        debug!("Metrics device opened successfully");
        Ok(device)
    }

    /// Map Intel MD API error codes to GpuError
    fn map_md_error(code: i32, context: &str) -> GpuError {
        match code {
            CC_OK => GpuError::GpuNotActive, // Should not be called with CC_OK
            CC_ERROR_GENERAL | MD_ERROR_GENERAL => GpuError::DriverNotInstalled,
            CC_ERROR_INVALID_PARAMETER | MD_ERROR_INVALID_PARAMETER => {
                GpuError::FeatureNotEnabled(format!("Invalid parameter in {}", context))
            }
            CC_ERROR_NOT_SUPPORTED | MD_ERROR_NOT_SUPPORTED => {
                GpuError::FeatureNotEnabled(format!("Not supported: {}", context))
            }
            _ => GpuError::FeatureNotEnabled(format!("MD API error {} in {}", code, context)),
        }
    }

    /// Close a metrics device
    ///
    /// # Safety
    /// The device pointer must be valid and obtained from `open_device()`
    unsafe fn close_device(&self, device: *mut IMetricsDevice_1_0) -> Result<()> {
        if device.is_null() {
            return Ok(());
        }

        let result = (self.close_device)(device);

        if result != CC_OK && result != MD_SUCCESS {
            warn!("CloseMetricsDevice failed with code: {}", result);
            return Err(GpuError::GpuNotActive);
        }

        debug!("Metrics device closed successfully");
        Ok(())
    }
}

// High-level wrapper

/// Intel Metrics Device wrapper
///
/// Provides RAII-style management of Intel Metrics Discovery device.
pub struct IntelMetricsDevice {
    api: IntelMetricsApi,
    device: *mut IMetricsDevice_1_0,
}

impl IntelMetricsDevice {
    /// Create a new Intel Metrics Device
    pub fn new() -> Result<Self> {
        let api = IntelMetricsApi::new()?;
        let device = api.open_device()?;

        Ok(Self { api, device })
    }

    /// Get a global symbol value by name
    ///
    /// Global symbols provide device-specific information like frequency ranges,
    /// EU counts, and sometimes current metrics.
    ///
    /// # Parameters
    /// - `symbol_name`: Name of the symbol (e.g., "GpuCurrentFrequencyMHz", "GpuMaxFrequencyMHz")
    ///
    /// # Returns
    /// TTypedValue_1_0 if found, or error if not found
    fn get_global_symbol(&self, symbol_name: &str) -> Result<TTypedValue_1_0> {
        if self.device.is_null() {
            return Err(GpuError::GpuNotFound);
        }

        unsafe {
            let vtbl = (*self.device).vtbl;
            if vtbl.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            let symbol_name_cstr = std::ffi::CString::new(symbol_name)
                .map_err(|_| GpuError::FeatureNotEnabled("Invalid symbol name".to_string()))?;

            let typed_value_ptr =
                ((*vtbl).get_global_symbol_value_by_name)(self.device, symbol_name_cstr.as_ptr());

            if typed_value_ptr.is_null() {
                return Err(GpuError::FeatureNotEnabled(format!(
                    "Global symbol '{}' not found",
                    symbol_name
                )));
            }

            // Copy the value to avoid lifetime issues
            let typed_value = std::ptr::read(typed_value_ptr as *const TTypedValue_1_0);
            Ok(typed_value)
        }
    }

    /// Find a metric by name in all concurrent groups and metric sets
    ///
    /// Searches through all concurrent groups and their metric sets to find a metric
    /// with the specified name.
    ///
    /// # Parameters
    /// - `metric_name`: Name of the metric to find (e.g., "GpuTemperature", "GpuPower")
    ///
    /// # Returns
    /// Tuple of (concurrent_group, metric_set, metric) if found, or error if not found
    fn find_metric(
        &self,
        metric_name: &str,
    ) -> Result<(
        *mut IConcurrentGroup_1_0,
        *mut IMetricSet_1_0,
        *mut IMetric_1_0,
    )> {
        if self.device.is_null() {
            return Err(GpuError::GpuNotFound);
        }

        unsafe {
            let vtbl = (*self.device).vtbl;
            if vtbl.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            // Get device parameters to know how many concurrent groups exist
            let params = ((*vtbl).get_params)(self.device);
            if params.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            let concurrent_groups_count = (*params).concurrent_groups_count;
            debug!(
                "Searching for metric '{}' in {} concurrent groups",
                metric_name, concurrent_groups_count
            );

            // Iterate through all concurrent groups
            for group_idx in 0..concurrent_groups_count {
                let concurrent_group = ((*vtbl).get_concurrent_group)(self.device, group_idx);
                if concurrent_group.is_null() {
                    continue;
                }

                let group_vtbl = (*concurrent_group).vtbl;
                if group_vtbl.is_null() {
                    continue;
                }

                // Get concurrent group parameters
                let group_params = ((*group_vtbl).get_params)(concurrent_group);
                if group_params.is_null() {
                    continue;
                }

                let metric_sets_count = (*group_params).metric_sets_count;

                // Iterate through all metric sets in this concurrent group
                for set_idx in 0..metric_sets_count {
                    let metric_set = ((*group_vtbl).get_metric_set)(concurrent_group, set_idx);
                    if metric_set.is_null() {
                        continue;
                    }

                    let set_vtbl = (*metric_set).vtbl;
                    if set_vtbl.is_null() {
                        continue;
                    }

                    // Get metric set parameters
                    let set_params = ((*set_vtbl).get_params)(metric_set);
                    if set_params.is_null() {
                        continue;
                    }

                    let metrics_count = (*set_params).metrics_count;

                    // Iterate through all metrics in this metric set
                    for metric_idx in 0..metrics_count {
                        let metric = ((*set_vtbl).get_metric)(metric_set, metric_idx);
                        if metric.is_null() {
                            continue;
                        }

                        let metric_vtbl = (*metric).vtbl;
                        if metric_vtbl.is_null() {
                            continue;
                        }

                        // Get metric parameters
                        let metric_params = ((*metric_vtbl).get_params)(metric);
                        if metric_params.is_null() {
                            continue;
                        }

                        // Check if this is the metric we're looking for
                        let symbol_name = (*metric_params).symbol_name;
                        if !symbol_name.is_null() {
                            let name_cstr = std::ffi::CStr::from_ptr(symbol_name);
                            if let Ok(name_str) = name_cstr.to_str() {
                                if name_str.contains(metric_name) {
                                    debug!(
                                        "Found metric '{}' in group {}, set {}",
                                        metric_name, group_idx, set_idx
                                    );
                                    return Ok((concurrent_group, metric_set, metric));
                                }
                            }
                        }
                    }
                }
            }
        }

        warn!("Metric '{}' not found in any metric set", metric_name);
        Err(GpuError::FeatureNotEnabled(format!(
            "Metric '{}' not found",
            metric_name
        )))
    }

    /// Get GPU temperature in Celsius
    ///
    /// Searches for temperature-related metrics in the Intel Metrics Discovery API.
    /// Common metric names: "GpuTemperature", "Temperature", "GT Temperature"
    ///
    /// # Implementation Note
    /// This method attempts to collect temperature via IoStream API.
    /// Temperature metrics require opening an IoStream, reading reports, and calculating values.
    pub fn get_temperature(&self) -> Result<f32> {
        // Try to find temperature metric
        let (concurrent_group, metric_set, _metric) = self.find_metric("Temperature")?;

        unsafe {
            let set_vtbl = (*metric_set).vtbl;
            if set_vtbl.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            let group_vtbl = (*concurrent_group).vtbl;
            if group_vtbl.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            // Activate the metric set
            let result = ((*set_vtbl).activate)(metric_set);
            if result != CC_OK && result != MD_SUCCESS {
                error!("Failed to activate temperature metric set: {}", result);
                let _ = ((*set_vtbl).deactivate)(metric_set);
                return Err(GpuError::FeatureNotEnabled(
                    "Failed to activate temperature metric set".to_string(),
                ));
            }

            // Try to open IoStream
            let mut timer_period_ns: u32 = 0;
            let mut buffer_size: u32 = 0;

            let open_result = ((*group_vtbl).open_io_stream)(
                concurrent_group,
                metric_set,
                0, // process ID (0 = system-wide)
                &mut timer_period_ns,
                &mut buffer_size,
            );

            if open_result != CC_OK && open_result != MD_SUCCESS {
                warn!(
                    "Failed to open IoStream for temperature: error code {}",
                    open_result
                );
                let _ = ((*set_vtbl).deactivate)(metric_set);
                return Err(GpuError::FeatureNotEnabled(
                    "Failed to open IoStream for temperature".to_string(),
                ));
            }

            debug!(
                "IoStream opened: timer_period={}ns, buffer_size={}",
                timer_period_ns, buffer_size
            );

            // Wait for data to be collected
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Read data from IoStream
            let mut report_count: u32 = 1;
            let mut report_buffer: Vec<u8> = vec![0; buffer_size as usize];

            let read_result = ((*group_vtbl).read_io_stream)(
                concurrent_group,
                &mut report_count,
                report_buffer.as_mut_ptr(),
                buffer_size,
            );

            // Close IoStream
            let _ = ((*group_vtbl).close_io_stream)(concurrent_group);
            let _ = ((*set_vtbl).deactivate)(metric_set);

            if read_result != CC_OK && read_result != MD_SUCCESS {
                warn!("Failed to read IoStream: error code {}", read_result);
                return Err(GpuError::FeatureNotEnabled(
                    "Failed to read temperature data".to_string(),
                ));
            }

            if report_count == 0 {
                warn!("No reports available from IoStream");
                return Err(GpuError::FeatureNotEnabled(
                    "No temperature data available".to_string(),
                ));
            }

            debug!("Read {} reports from IoStream", report_count);

            // Try to calculate metrics using IMetricSet_1_1
            let set_1_1 = metric_set as *mut IMetricSet_1_1;
            let set_1_1_vtbl = (*set_1_1).vtbl;

            if set_1_1_vtbl.is_null() {
                warn!("IMetricSet_1_1 vtable is null");
                return Err(GpuError::FeatureNotEnabled(
                    "CalculateMetrics not available".to_string(),
                ));
            }

            // Get metric set parameters to know how many metrics to allocate
            let set_params = ((*set_vtbl).get_params)(metric_set);
            if set_params.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            let metrics_count = (*set_params).metrics_count;
            let mut calculated_values: Vec<TTypedValue_1_0> = vec![
                TTypedValue_1_0 {
                    value_type: 0,
                    value_data: TTypedValueData { value_uint32: 0 },
                };
                metrics_count as usize
            ];
            let mut calculated_count: u32 = 0;

            let calc_result = ((*set_1_1_vtbl).calculate_metrics)(
                set_1_1,
                report_buffer.as_ptr(),
                buffer_size,
                calculated_values.as_mut_ptr(),
                metrics_count,
                &mut calculated_count,
                false, // useInformation
            );

            if calc_result != CC_OK && calc_result != MD_SUCCESS {
                warn!("CalculateMetrics failed: error code {}", calc_result);
                return Err(GpuError::FeatureNotEnabled(
                    "Failed to calculate temperature metrics".to_string(),
                ));
            }

            debug!("Calculated {} metric values", calculated_count);

            // Search for temperature value in calculated metrics
            for metric_idx in 0..metrics_count {
                let metric = ((*set_vtbl).get_metric)(metric_set, metric_idx);
                if metric.is_null() {
                    continue;
                }

                let metric_vtbl = (*metric).vtbl;
                if metric_vtbl.is_null() {
                    continue;
                }

                let metric_params = ((*metric_vtbl).get_params)(metric);
                if metric_params.is_null() {
                    continue;
                }

                let symbol_name = (*metric_params).symbol_name;
                if !symbol_name.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(symbol_name);
                    if let Ok(name_str) = name_cstr.to_str() {
                        if name_str.contains("Temperature") || name_str.contains("Temp") {
                            // Found temperature metric
                            if (metric_idx as usize) < calculated_values.len() {
                                let typed_value = &calculated_values[metric_idx as usize];
                                match typed_value.value_type {
                                    0 => {
                                        // UINT32
                                        let temp = typed_value.value_data.value_uint32 as f32;
                                        info!(
                                            "Got temperature from metric '{}': {:.1}°C",
                                            name_str, temp
                                        );
                                        return Ok(temp);
                                    }
                                    1 => {
                                        // UINT64
                                        let temp = typed_value.value_data.value_uint64 as f32;
                                        info!(
                                            "Got temperature from metric '{}': {:.1}°C",
                                            name_str, temp
                                        );
                                        return Ok(temp);
                                    }
                                    2 => {
                                        // FLOAT
                                        let temp = typed_value.value_data.value_float;
                                        info!(
                                            "Got temperature from metric '{}': {:.1}°C",
                                            name_str, temp
                                        );
                                        return Ok(temp);
                                    }
                                    _ => continue,
                                }
                            }
                        }
                    }
                }
            }
        }

        warn!("Temperature value not found in calculated metrics");
        Err(GpuError::FeatureNotEnabled(
            "Temperature metric not found in results".to_string(),
        ))
    }

    /// Get GPU power usage in Watts
    ///
    /// Searches for power-related metrics in the Intel Metrics Discovery API.
    /// Common metric names: "GpuPower", "Power", "GT Power"
    ///
    /// # Implementation Note
    /// This method attempts to collect power via IoStream API.
    /// Power metrics require opening an IoStream, reading reports, and calculating values.
    pub fn get_power(&self) -> Result<f32> {
        // Try to find power metric
        let (concurrent_group, metric_set, _metric) = self.find_metric("Power")?;

        unsafe {
            let set_vtbl = (*metric_set).vtbl;
            if set_vtbl.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            let group_vtbl = (*concurrent_group).vtbl;
            if group_vtbl.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            // Activate the metric set
            let result = ((*set_vtbl).activate)(metric_set);
            if result != CC_OK && result != MD_SUCCESS {
                error!("Failed to activate power metric set: {}", result);
                let _ = ((*set_vtbl).deactivate)(metric_set);
                return Err(GpuError::FeatureNotEnabled(
                    "Failed to activate power metric set".to_string(),
                ));
            }

            // Try to open IoStream
            let mut timer_period_ns: u32 = 0;
            let mut buffer_size: u32 = 0;

            let open_result = ((*group_vtbl).open_io_stream)(
                concurrent_group,
                metric_set,
                0, // process ID (0 = system-wide)
                &mut timer_period_ns,
                &mut buffer_size,
            );

            if open_result != CC_OK && open_result != MD_SUCCESS {
                warn!(
                    "Failed to open IoStream for power: error code {}",
                    open_result
                );
                let _ = ((*set_vtbl).deactivate)(metric_set);
                return Err(GpuError::FeatureNotEnabled(
                    "Failed to open IoStream for power".to_string(),
                ));
            }

            debug!(
                "IoStream opened: timer_period={}ns, buffer_size={}",
                timer_period_ns, buffer_size
            );

            // Wait for data to be collected
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Read data from IoStream
            let mut report_count: u32 = 1;
            let mut report_buffer: Vec<u8> = vec![0; buffer_size as usize];

            let read_result = ((*group_vtbl).read_io_stream)(
                concurrent_group,
                &mut report_count,
                report_buffer.as_mut_ptr(),
                buffer_size,
            );

            // Close IoStream
            let _ = ((*group_vtbl).close_io_stream)(concurrent_group);
            let _ = ((*set_vtbl).deactivate)(metric_set);

            if read_result != CC_OK && read_result != MD_SUCCESS {
                warn!("Failed to read IoStream: error code {}", read_result);
                return Err(GpuError::FeatureNotEnabled(
                    "Failed to read power data".to_string(),
                ));
            }

            if report_count == 0 {
                warn!("No reports available from IoStream");
                return Err(GpuError::FeatureNotEnabled(
                    "No power data available".to_string(),
                ));
            }

            debug!("Read {} reports from IoStream", report_count);

            // Try to calculate metrics using IMetricSet_1_1
            let set_1_1 = metric_set as *mut IMetricSet_1_1;
            let set_1_1_vtbl = (*set_1_1).vtbl;

            if set_1_1_vtbl.is_null() {
                warn!("IMetricSet_1_1 vtable is null");
                return Err(GpuError::FeatureNotEnabled(
                    "CalculateMetrics not available".to_string(),
                ));
            }

            // Get metric set parameters to know how many metrics to allocate
            let set_params = ((*set_vtbl).get_params)(metric_set);
            if set_params.is_null() {
                return Err(GpuError::GpuNotFound);
            }

            let metrics_count = (*set_params).metrics_count;
            let mut calculated_values: Vec<TTypedValue_1_0> = vec![
                TTypedValue_1_0 {
                    value_type: 0,
                    value_data: TTypedValueData { value_uint32: 0 },
                };
                metrics_count as usize
            ];
            let mut calculated_count: u32 = 0;

            let calc_result = ((*set_1_1_vtbl).calculate_metrics)(
                set_1_1,
                report_buffer.as_ptr(),
                buffer_size,
                calculated_values.as_mut_ptr(),
                metrics_count,
                &mut calculated_count,
                false, // useInformation
            );

            if calc_result != CC_OK && calc_result != MD_SUCCESS {
                warn!("CalculateMetrics failed: error code {}", calc_result);
                return Err(GpuError::FeatureNotEnabled(
                    "Failed to calculate power metrics".to_string(),
                ));
            }

            debug!("Calculated {} metric values", calculated_count);

            // Search for power value in calculated metrics
            for metric_idx in 0..metrics_count {
                let metric = ((*set_vtbl).get_metric)(metric_set, metric_idx);
                if metric.is_null() {
                    continue;
                }

                let metric_vtbl = (*metric).vtbl;
                if metric_vtbl.is_null() {
                    continue;
                }

                let metric_params = ((*metric_vtbl).get_params)(metric);
                if metric_params.is_null() {
                    continue;
                }

                let symbol_name = (*metric_params).symbol_name;
                if !symbol_name.is_null() {
                    let name_cstr = std::ffi::CStr::from_ptr(symbol_name);
                    if let Ok(name_str) = name_cstr.to_str() {
                        if name_str.contains("Power") {
                            // Found power metric
                            if (metric_idx as usize) < calculated_values.len() {
                                let typed_value = &calculated_values[metric_idx as usize];
                                match typed_value.value_type {
                                    0 => {
                                        // UINT32
                                        let power = typed_value.value_data.value_uint32 as f32;
                                        info!(
                                            "Got power from metric '{}': {:.1}W",
                                            name_str, power
                                        );
                                        return Ok(power);
                                    }
                                    1 => {
                                        // UINT64
                                        let power = typed_value.value_data.value_uint64 as f32;
                                        info!(
                                            "Got power from metric '{}': {:.1}W",
                                            name_str, power
                                        );
                                        return Ok(power);
                                    }
                                    2 => {
                                        // FLOAT
                                        let power = typed_value.value_data.value_float;
                                        info!(
                                            "Got power from metric '{}': {:.1}W",
                                            name_str, power
                                        );
                                        return Ok(power);
                                    }
                                    _ => continue,
                                }
                            }
                        }
                    }
                }
            }
        }

        warn!("Power value not found in calculated metrics");
        Err(GpuError::FeatureNotEnabled(
            "Power metric not found in results".to_string(),
        ))
    }

    /// Get GPU max frequency in MHz
    ///
    /// Returns the maximum frequency the GPU can reach.
    pub fn get_max_frequency(&self) -> Result<u32> {
        let symbol_names = ["GpuMaxFrequencyMHz", "MaxFrequency", "GpuMaxFreq"];

        for symbol_name in &symbol_names {
            if let Ok(typed_value) = self.get_global_symbol(symbol_name) {
                unsafe {
                    match typed_value.value_type {
                        0 => {
                            let freq = typed_value.value_data.value_uint32;
                            if freq > 0 && freq < 10000 {
                                debug!("Got max frequency: {} MHz", freq);
                                return Ok(freq);
                            }
                        }
                        1 => {
                            let freq = typed_value.value_data.value_uint64 as u32;
                            if freq > 0 && freq < 10000 {
                                debug!("Got max frequency: {} MHz", freq);
                                return Ok(freq);
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }

        Err(GpuError::FeatureNotEnabled(
            "Max frequency not available".to_string(),
        ))
    }

    /// Get memory frequency in MHz
    pub fn get_memory_frequency(&self) -> Result<u32> {
        if let Ok(typed_value) = self.get_global_symbol("MemoryFrequencyMHz") {
            unsafe {
                match typed_value.value_type {
                    0 => Ok(typed_value.value_data.value_uint32),
                    1 => Ok(typed_value.value_data.value_uint64 as u32),
                    _ => Err(GpuError::FeatureNotEnabled(
                        "Invalid memory frequency type".to_string(),
                    )),
                }
            }
        } else {
            Err(GpuError::FeatureNotEnabled(
                "Memory frequency not available".to_string(),
            ))
        }
    }

    // NOTE: get_utilization() method removed
    // Utilization is obtained from PDH (Performance Data Helper) instead.
    // Intel MD API's utilization metric is complex (200+ lines), unreliable,
    // and provides no benefit over PDH. See enhance_with_pdh() method.

    /// Get GPU frequency in MHz
    ///
    /// Tries to get frequency from global symbols first (simpler approach),
    /// then falls back to metric collection if needed.
    /// Common symbol names: "GpuCurrentFrequencyMHz", "GpuMaxFrequencyMHz", "GpuMinFrequencyMHz"
    pub fn get_frequency(&self) -> Result<u32> {
        // Try global symbols first (simpler and faster)
        let symbol_names = [
            "GpuCurrentFrequencyMHz",
            "GpuFrequencyMHz",
            "GpuCoreFrequencyMHz",
        ];

        for symbol_name in &symbol_names {
            if let Ok(typed_value) = self.get_global_symbol(symbol_name) {
                unsafe {
                    match typed_value.value_type {
                        0 => {
                            // VALUE_TYPE_UINT32
                            let freq = typed_value.value_data.value_uint32;
                            if freq > 0 && freq < 10000 {
                                // Sanity check: 0-10GHz
                                debug!(
                                    "Got frequency from global symbol '{}': {} MHz",
                                    symbol_name, freq
                                );
                                return Ok(freq);
                            }
                        }
                        1 => {
                            // VALUE_TYPE_UINT64
                            let freq = typed_value.value_data.value_uint64 as u32;
                            if freq > 0 && freq < 10000 {
                                debug!(
                                    "Got frequency from global symbol '{}': {} MHz",
                                    symbol_name, freq
                                );
                                return Ok(freq);
                            }
                        }
                        2 => {
                            // VALUE_TYPE_FLOAT
                            let freq = typed_value.value_data.value_float as u32;
                            if freq > 0 && freq < 10000 {
                                debug!(
                                    "Got frequency from global symbol '{}': {} MHz",
                                    symbol_name, freq
                                );
                                return Ok(freq);
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }

        // Fallback: try metric collection (more complex, requires IoStream)
        warn!("Frequency not available via global symbols, metric collection not yet implemented");
        Err(GpuError::FeatureNotEnabled(
            "Frequency metric not available".to_string(),
        ))
    }
}

impl Drop for IntelMetricsDevice {
    fn drop(&mut self) {
        if !self.device.is_null() {
            unsafe {
                let _ = self.api.close_device(self.device);
            }
        }
    }
}

// Intel Windows Provider implementation

/// Intel GPU provider for Windows
///
/// Uses a three-tier approach for GPU metrics:
/// 1. WMI (Windows Management Instrumentation) - Basic GPU detection and info
/// 2. Intel Metrics Discovery API - Temperature, power, frequency (if available)
/// 3. PDH (Performance Data Helper) - Utilization and memory usage fallback
pub struct IntelWindowsProvider;

impl IntelWindowsProvider {
    pub fn new() -> Self {
        Self
    }

    /// Get basic Intel GPU info using WMI
    ///
    /// Uses the platform-agnostic IntelProvider which queries WMI for:
    /// - GPU name, vendor, driver version
    /// - Total memory size
    fn get_basic_gpu_info(&self) -> Result<GpuInfo> {
        let intel_provider = super::super::intel::IntelProvider::new();
        let gpus = intel_provider.detect_gpus()?;
        gpus.into_iter().next().ok_or(GpuError::GpuNotFound)
    }

    /// Enhance GPU info with Intel Metrics Discovery API
    ///
    /// # Metrics from Intel MD API
    /// - Temperature (primary source)
    /// - Power usage (primary source)
    /// - Core frequency (primary source)
    /// - Max frequency (primary source)
    /// - Memory frequency (primary source)
    ///
    /// # Note
    /// Utilization is NOT collected here - PDH is more reliable for that metric.
    fn enhance_with_md_api(&self, gpu: &mut GpuInfo) {
        if let Ok(device) = IntelMetricsDevice::new() {
            debug!("✓ Intel Metrics Discovery API available, collecting metrics");

            // Temperature - primary source
            if let Ok(temp) = device.get_temperature() {
                gpu.temperature = Some(temp);
                info!("✓ Temperature from Intel MD API: {:.1}°C", temp);
            } else {
                debug!("✗ Temperature not available from Intel MD API");
            }

            // Power usage - primary source
            if let Ok(power) = device.get_power() {
                gpu.power_usage = Some(power);
                info!("✓ Power usage from Intel MD API: {:.1}W", power);
            } else {
                debug!("✗ Power usage not available from Intel MD API");
            }

            // Core frequency - primary source
            if let Ok(freq) = device.get_frequency() {
                gpu.core_clock = Some(freq);
                info!("✓ Core clock from Intel MD API: {} MHz", freq);
            } else {
                debug!("✗ Core clock not available from Intel MD API");
            }

            // Max frequency - primary source
            if let Ok(max_freq) = device.get_max_frequency() {
                gpu.max_clock_speed = Some(max_freq);
                info!("✓ Max clock from Intel MD API: {} MHz", max_freq);
            } else {
                debug!("✗ Max clock not available from Intel MD API");
            }

            // Memory frequency - primary source
            if let Ok(mem_freq) = device.get_memory_frequency() {
                gpu.memory_clock = Some(mem_freq);
                info!("✓ Memory clock from Intel MD API: {} MHz", mem_freq);
            } else {
                debug!("✗ Memory clock not available from Intel MD API");
            }

            debug!("Intel MD API metrics collection complete");
        } else {
            debug!("✗ Intel Metrics Discovery API not available (igdmd64.dll not found)");
        }
    }

    /// Get GPU utilization and memory usage via PDH
    ///
    /// # Metrics from PDH
    /// - GPU utilization (primary source - more reliable than Intel MD API)
    /// - Memory usage (primary source - Intel MD API doesn't provide this)
    ///
    /// # Strategy
    /// PDH is the PRIMARY source for utilization and memory metrics.
    /// Intel MD API's utilization metric is complex and unreliable (requires IoStream).
    fn get_utilization(&self, gpu: &mut GpuInfo) {
        debug!("→ Collecting PDH metrics (utilization & memory)");

        // Open PDH query
        let query = match super::pdh::open_query() {
            Ok(q) => q,
            Err(e) => {
                warn!("✗ Failed to open PDH query: {:?}", e);
                return;
            }
        };

        // Build counter paths for GPU utilization and memory
        let utilization_path = r"\GPU Engine(*engtype_3D)\Utilization Percentage";
        let memory_path = r"\GPU Process Memory(*)\Dedicated Usage";

        // Expand wildcard paths and add counters
        let util_paths = super::pdh::expand_wildcard_path(utilization_path);
        let mem_paths = super::pdh::expand_wildcard_path(memory_path);

        let mut util_counters = Vec::new();
        let mut mem_counters = Vec::new();

        // Add utilization counters
        for path in &util_paths {
            if let Some(counter) = super::pdh::add_counter(query, path) {
                util_counters.push(counter);
            }
        }

        // Add memory counters
        for path in &mem_paths {
            if let Some(counter) = super::pdh::add_counter(query, path) {
                mem_counters.push(counter);
            }
        }

        if util_counters.is_empty() && mem_counters.is_empty() {
            warn!("✗ No PDH counters available");
            super::pdh::close_query(query);
            return;
        }

        // Collect first snapshot
        if let Err(e) = super::pdh::collect_query_data(query) {
            warn!("✗ First PDH collection failed: {:?}", e);
            super::pdh::close_query(query);
            return;
        }

        // Wait for PDH collection interval
        std::thread::sleep(std::time::Duration::from_millis(
            super::pdh::PDH_COLLECTION_INTERVAL_MS,
        ));

        // Collect second snapshot
        if let Err(e) = super::pdh::collect_query_data(query) {
            warn!("✗ Second PDH collection failed: {:?}", e);
            super::pdh::close_query(query);
            return;
        }

        // Calculate total GPU utilization
        // Note: PDH returns multiple GPU Engine counters (one per execution unit).
        // We need to SUM all values, not average them, as each counter represents
        // a portion of the total GPU utilization.
        if !util_counters.is_empty() {
            let mut total_util = 0.0;

            for counter in &util_counters {
                if let Ok(value) = super::pdh::get_counter_value(*counter) {
                    total_util += value;
                }
            }

            // Total utilization is the sum of all engine utilizations
            gpu.utilization = Some(total_util as f32);
            info!("✓ Utilization from PDH: {:.2}%", total_util);
        }

        // Calculate total memory usage
        if !mem_counters.is_empty() {
            let mut total_mem_bytes = 0.0;
            let mut valid_count = 0;

            for counter in &mem_counters {
                if let Ok(value) = super::pdh::get_counter_value(*counter) {
                    total_mem_bytes += value;
                    valid_count += 1;
                }
            }

            if valid_count > 0 {
                let mem_mb = (total_mem_bytes / (1024.0 * 1024.0)) as u64;
                if let Some(total_mb) = gpu.memory_total {
                    let mem_percent = (mem_mb as f32 / (total_mb * 1024) as f32) * 100.0;
                    gpu.memory_util = Some(mem_percent.min(100.0));
                    info!(
                        "✓ Memory utilization from PDH: {:.2}% ({} MB / {} MB)",
                        mem_percent,
                        mem_mb,
                        total_mb * 1024
                    );
                } else {
                    debug!("  Cannot calculate memory %: total memory unknown");
                }
            } else {
                debug!("✗ No valid memory values from PDH");
            }
        }

        // Close PDH query
        super::pdh::close_query(query);
        debug!("PDH metrics collection complete");
    }
}

impl Default for IntelWindowsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuProvider for IntelWindowsProvider {
    /// Detect Intel GPUs on Windows
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        debug!("Detecting Intel GPUs on Windows");

        let mut gpu = self.get_basic_gpu_info()?;

        // Enhance with Intel Metrics Discovery API
        self.enhance_with_md_api(&mut gpu);

        // Get utilization and memory via PDH
        self.get_utilization(&mut gpu);

        info!("Successfully detected Intel GPU: {:?}", gpu.name_gpu);
        Ok(vec![gpu])
    }

    /// Update Intel GPU information
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        debug!("Updating Intel GPU information on Windows");

        // Get fresh basic info
        let basic_info = self.get_basic_gpu_info()?;

        // Update basic fields
        gpu.name_gpu = basic_info.name_gpu;
        gpu.vendor = basic_info.vendor;
        gpu.driver_version = basic_info.driver_version;
        gpu.memory_total = basic_info.memory_total;
        gpu.active = basic_info.active;

        // Enhance with Intel Metrics Discovery API
        self.enhance_with_md_api(gpu);

        // Get utilization and memory via PDH
        self.get_utilization(gpu);

        if !gpu.is_valid() {
            warn!("GPU data validation failed");
            return Err(GpuError::GpuNotActive);
        }

        info!("Successfully updated Intel GPU information");
        Ok(())
    }

    /// Get the vendor for this provider
    fn get_vendor(&self) -> Vendor {
        Vendor::Intel(IntelGpuType::Unknown)
    }
}
