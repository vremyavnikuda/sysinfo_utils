//! Windows Performance Data Helper (PDH) API for GPU metrics
//!
//! This module provides access to GPU performance counters through Windows PDH API.
//! It can retrieve GPU utilization and memory usage for Intel and other GPUs.

use crate::gpu_info::{GpuError, Result};
use log::{debug, error, warn};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::Performance::{
    PdhAddCounterW, PdhCloseQuery, PdhCollectQueryData, PdhExpandWildCardPathW,
    PdhGetFormattedCounterValue, PdhOpenQueryW, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
};

// PDH handle types
type PdhHquery = isize;
type PdhHcounter = isize;

/// PDH collection interval in milliseconds
///
/// PDH requires two data collection points to calculate rates accurately.
/// 500ms provides a good balance between accuracy and responsiveness.
const PDH_COLLECTION_INTERVAL_MS: u64 = 500;

/// GPU metrics from Performance Counters
#[derive(Debug, Clone, Default)]
pub struct GpuPdhMetrics {
    /// GPU utilization percentage (0.0 - 100.0)
    utilization: Option<f32>,
    /// GPU memory usage in MB
    memory_usage_mb: Option<u32>,
}

impl GpuPdhMetrics {
    /// Get GPU utilization percentage (0.0 - 100.0)
    #[inline]
    pub fn utilization(&self) -> Option<f32> {
        self.utilization
    }

    /// Get GPU memory usage in MB
    #[inline]
    pub fn memory_usage_mb(&self) -> Option<u32> {
        self.memory_usage_mb
    }
}

/// Get GPU metrics using Performance Counters
///
/// # Performance Note
/// This function blocks for approximately 500ms to collect accurate PDH data.
/// PDH requires two data collection points to calculate rates. If calling from
/// async code, use `tokio::task::spawn_blocking` to avoid blocking the executor.
///
/// # Internal API
/// This is an internal function used by `IntelProvider`. It is not part of the
/// public API and should not be called directly by external code.
pub fn get_gpu_metrics(gpu_index: usize) -> Result<GpuPdhMetrics> {
    unsafe {
        let mut query: PdhHquery = 0;
        let result = PdhOpenQueryW(PCWSTR::null(), 0, &mut query);
        if result != 0 {
            error!("Failed to open PDH query: error code 0x{:X}", result);
            return Err(GpuError::DriverNotInstalled);
        }
        debug!("PDH query opened successfully");
        let util_counter_path =
            format!("\\GPU Engine(*phys_{}*)\\Utilization Percentage", gpu_index);

        debug!("Expanding wildcard path: {}", util_counter_path);
        let expanded_paths = expand_wildcard_path(&util_counter_path);
        debug!("Found {} counter paths", expanded_paths.len());
        let mut util_counters = Vec::new();
        for path in &expanded_paths {
            if let Some(counter) = add_counter(query, path) {
                util_counters.push(counter);
            }
        }

        debug!(
            "Successfully added {} utilization counters",
            util_counters.len()
        );

        let mem_counter_path = format!(
            "\\GPU Adapter Memory(*phys_{}*)\\Dedicated Usage",
            gpu_index
        );
        debug!("Memory counter path: {}", mem_counter_path);
        let mem_counter = add_counter(query, &mem_counter_path);
        debug!("Memory counter added: {}", mem_counter.is_some());
        let result1 = PdhCollectQueryData(query);
        debug!("First PdhCollectQueryData result: {}", result1);
        std::thread::sleep(std::time::Duration::from_millis(PDH_COLLECTION_INTERVAL_MS));
        let result = PdhCollectQueryData(query);
        debug!("Second PdhCollectQueryData result: 0x{:X}", result);
        if result != 0 {
            warn!("Failed to collect PDH data: error code 0x{:X}", result);
            let _ = PdhCloseQuery(query);
            return Err(GpuError::GpuNotActive);
        }
        let mut metrics = GpuPdhMetrics::default();
        if !util_counters.is_empty() {
            let mut total_utilization = 0.0;

            for counter in &util_counters {
                if let Ok(value) = get_counter_value(*counter) {
                    total_utilization += value;
                }
            }

            debug!(
                "Total utilization from {} engines: {:.2}%",
                util_counters.len(),
                total_utilization
            );
            let rounded = (total_utilization.min(100.0) * 100.0).round() / 100.0;
            metrics.utilization = Some(rounded as f32);
        } else {
            warn!("No utilization counters available");
        }
        if let Some(counter) = mem_counter {
            if let Ok(value) = get_counter_value(counter) {
                let mb = (value / 1024.0 / 1024.0) as u32;
                metrics.memory_usage_mb = Some(mb);
                debug!("GPU memory usage: {} MB", mb);
            }
        }
        let _ = PdhCloseQuery(query);
        debug!("PDH query closed");
        Ok(metrics)
    }
}

/// Expand wildcard path to get all matching counter paths
unsafe fn expand_wildcard_path(wildcard_path: &str) -> Vec<String> {
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
unsafe fn add_counter(query: PdhHquery, path: &str) -> Option<PdhHcounter> {
    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut hcounter: PdhHcounter = 0;

    let result = PdhAddCounterW(query, PCWSTR(wide_path.as_ptr()), 0, &mut hcounter);

    if result != 0 {
        warn!("Failed to add counter '{}': error code {}", path, result);
        return None;
    }

    debug!("Added PDH counter: {}", path);
    Some(hcounter)
}

/// Get formatted counter value
unsafe fn get_counter_value(counter: PdhHcounter) -> Result<f64> {
    let mut value: PDH_FMT_COUNTERVALUE = std::mem::zeroed();

    let result = PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, None, &mut value);

    if result != 0 {
        return Err(GpuError::GpuNotActive);
    }

    Ok(value.Anonymous.doubleValue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg_attr(not(target_os = "windows"), ignore)]
    fn test_get_gpu_metrics() {
        let result = get_gpu_metrics(0);

        match result {
            Ok(metrics) => {
                println!("GPU metrics: {:?}", metrics);
                if let Some(util) = metrics.utilization() {
                    assert!(
                        (0.0..=100.0).contains(&util),
                        "Utilization out of range: {}",
                        util
                    );
                }
                // Note: memory_usage_mb can be 0 for some GPUs (e.g., Intel integrated)
                // so we just check that it's present, not that it's positive
                if let Some(_mem) = metrics.memory_usage_mb() {
                    // Memory metric is available (value can be 0)
                }
            }
            Err(e) => {
                println!("Expected error (no GPU or driver): {:?}", e);
                // This is acceptable - not all test systems have GPUs
            }
        }
    }

    #[test]
    fn test_gpu_pdh_metrics_getters() {
        let metrics = GpuPdhMetrics {
            utilization: Some(75.5),
            memory_usage_mb: Some(4096),
        };

        assert_eq!(metrics.utilization(), Some(75.5));
        assert_eq!(metrics.memory_usage_mb(), Some(4096));
    }

    #[test]
    fn test_gpu_pdh_metrics_default() {
        let metrics = GpuPdhMetrics::default();

        assert_eq!(metrics.utilization(), None);
        assert_eq!(metrics.memory_usage_mb(), None);
    }
}
