//! Windows Performance Data Helper (PDH) API - Internal Utility Module
//!
//! This is an internal utility module that provides low-level PDH API access.
//! It should NOT be used directly by external code. Instead, use `IntelWindowsProvider`
//! which provides a unified interface for all Intel GPU metrics.
//!
//! # Architecture
//! - This module contains only PDH FFI bindings and basic data collection
//! - All business logic and fallback strategies are in `intel.rs`
//! - PDH is used as a fallback when Intel Metrics Discovery API is unavailable

use crate::gpu_info::{GpuError, Result};
use log::{debug, error, warn};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::Performance::{
    PdhAddCounterW, PdhCloseQuery, PdhCollectQueryData, PdhExpandWildCardPathW,
    PdhGetFormattedCounterValue, PdhOpenQueryW, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
};

/// PDH collection interval in milliseconds
///
/// PDH requires two data collection points to calculate rates accurately.
/// 500ms provides a good balance between accuracy and responsiveness.
pub(crate) const PDH_COLLECTION_INTERVAL_MS: u64 = 500;

// PDH Query handle type (re-exported for use in intel.rs)
pub(crate) type PdhQuery = isize;
pub(crate) type PdhCounter = isize;

/// Open a new PDH query
pub(crate) fn open_query() -> Result<PdhQuery> {
    unsafe {
        let mut query: PdhQuery = 0;
        let result = PdhOpenQueryW(PCWSTR::null(), 0, &mut query);
        if result != 0 {
            error!("Failed to open PDH query: error code 0x{:X}", result);
            return Err(GpuError::DriverNotInstalled);
        }
        debug!("PDH query opened successfully");
        Ok(query)
    }
}

/// Close a PDH query
pub(crate) fn close_query(query: PdhQuery) {
    unsafe {
        let _ = PdhCloseQuery(query);
        debug!("PDH query closed");
    }
}

/// Collect query data
pub(crate) fn collect_query_data(query: PdhQuery) -> Result<()> {
    unsafe {
        let result = PdhCollectQueryData(query);
        if result != 0 {
            warn!("Failed to collect PDH data: error code 0x{:X}", result);
            return Err(GpuError::GpuNotActive);
        }
        Ok(())
    }
}

/// Expand wildcard path to get all matching counter paths
pub(crate) fn expand_wildcard_path(wildcard_path: &str) -> Vec<String> {
    unsafe { expand_wildcard_path_unsafe(wildcard_path) }
}

unsafe fn expand_wildcard_path_unsafe(wildcard_path: &str) -> Vec<String> {
    let wide_path: Vec<u16> = OsStr::new(wildcard_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut buffer_size: u32 = 0;
    let result = PdhExpandWildCardPathW(
        PCWSTR::null(),
        PCWSTR(wide_path.as_ptr()),
        PWSTR::null(),
        &mut buffer_size,
        0,
    );
    if result != 0x800007D2 && result != 0 {
        warn!("PdhExpandWildCardPathW failed: 0x{:X}", result);
        return Vec::new();
    }
    if buffer_size == 0 {
        return Vec::new();
    }
    let mut buffer: Vec<u16> = vec![0; buffer_size as usize];
    let result = PdhExpandWildCardPathW(
        PCWSTR::null(),
        PCWSTR(wide_path.as_ptr()),
        PWSTR(buffer.as_mut_ptr()),
        &mut buffer_size,
        0,
    );
    if result != 0 {
        warn!("PdhExpandWildCardPathW second call failed: 0x{:X}", result);
        return Vec::new();
    }
    let mut paths = Vec::new();
    let mut start = 0;
    for i in 0..buffer.len() {
        if buffer[i] == 0 {
            if i > start {
                if let Ok(path) = String::from_utf16(&buffer[start..i]) {
                    paths.push(path);
                }
            }
            start = i + 1;
            if i + 1 < buffer.len() && buffer[i + 1] == 0 {
                break;
            }
        }
    }
    paths
}

/// Add a counter to the query
pub(crate) fn add_counter(query: PdhQuery, path: &str) -> Option<PdhCounter> {
    unsafe { add_counter_unsafe(query, path) }
}

unsafe fn add_counter_unsafe(query: PdhQuery, path: &str) -> Option<PdhCounter> {
    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut hcounter: PdhCounter = 0;
    let result = PdhAddCounterW(query, PCWSTR(wide_path.as_ptr()), 0, &mut hcounter);
    if result != 0 {
        warn!("Failed to add counter '{}': error code {}", path, result);
        return None;
    }
    debug!("Added PDH counter: {}", path);
    Some(hcounter)
}

/// Get formatted counter value
pub(crate) fn get_counter_value(counter: PdhCounter) -> Result<f64> {
    unsafe { get_counter_value_unsafe(counter) }
}

unsafe fn get_counter_value_unsafe(counter: PdhCounter) -> Result<f64> {
    let mut value: PDH_FMT_COUNTERVALUE = std::mem::zeroed();
    let result = PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, None, &mut value);
    if result != 0 {
        return Err(GpuError::GpuNotActive);
    }
    Ok(value.Anonymous.doubleValue)
}
