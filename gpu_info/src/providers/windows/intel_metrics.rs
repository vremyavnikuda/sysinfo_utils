//! Intel Metrics API for GPU monitoring
//!
//! This module provides access to Intel GPU metrics through Intel GPA Framework.
//! It attempts to load metrics-direct.dll from Intel GPA installation.

use crate::gpu_info::{GpuError, Result};
use libloading::Library;
use log::{error, info, warn};
use std::path::PathBuf;

/// Intel GPU metrics from GPA Framework
///
/// Note: Fields are public for internal convenience since this is a `pub(crate)` module.
/// This struct is not part of the public API.
#[derive(Debug, Clone, Default)]
pub struct IntelGpuMetrics {
    /// GPU temperature in Celsius
    pub temperature: Option<f32>,
    /// GPU power usage in Watts
    pub power_usage: Option<f32>,
    /// GPU core clock in MHz
    pub core_clock: Option<u32>,
    /// GPU memory clock in MHz
    pub memory_clock: Option<u32>,
}

/// Intel Metrics API wrapper
pub struct IntelMetricsApi {
    _library: Library,
}

impl IntelMetricsApi {
    /// Try to load Intel Metrics API from known locations
    pub fn new() -> Result<Self> {
        let possible_paths = vec![
            PathBuf::from(r"C:\Program Files\IntelSWTools\GPA\Streams\metrics-direct.dll"),
            PathBuf::from(r"C:\Program Files (x86)\IntelSWTools\GPA\Streams\metrics-direct.dll"),
        ];

        for path in possible_paths {
            if path.exists() {
                info!("Found Intel Metrics API at: {:?}", path);

                match unsafe { Library::new(&path) } {
                    Ok(lib) => {
                        info!("Successfully loaded Intel Metrics API");
                        return Ok(Self { _library: lib });
                    }
                    Err(e) => {
                        warn!("Failed to load Intel Metrics API from {:?}: {}", path, e);
                    }
                }
            }
        }

        error!("Intel Metrics API (metrics-direct.dll) not found");
        Err(GpuError::DriverNotInstalled)
    }

    /// Get GPU metrics
    ///
    /// Note: This is a placeholder implementation.
    /// The actual API requires reverse engineering or official documentation.
    pub fn get_metrics(&self) -> Result<IntelGpuMetrics> {
        // TODO: Implement actual API calls
        warn!("Intel Metrics API integration not yet implemented");
        warn!("This requires reverse engineering metrics-direct.dll or official documentation");

        Ok(IntelGpuMetrics::default())
    }
}

/// Try to get Intel GPU metrics
pub fn get_intel_metrics() -> Result<IntelGpuMetrics> {
    let api = IntelMetricsApi::new()?;
    api.get_metrics()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intel_metrics_api_load() {
        let result = IntelMetricsApi::new();
        println!("Intel Metrics API load result: {:?}", result.is_ok());
    }

    #[test]
    fn test_get_intel_metrics() {
        if let Ok(metrics) = get_intel_metrics() {
            println!("Intel GPU metrics: {:?}", metrics);
        }
    }
}
