//! Fallback implementation for unsupported platforms.
//!
//! This module provides stub implementations for platforms that are not
//! explicitly supported. All functions return empty or no-op results.

use crate::gpu_info::GpuInfo;
use log::warn;

/// Initializes GPU detection on unsupported platforms.
///
/// This function always returns an empty vector and logs a warning,
/// as GPU detection is not available on unsupported platforms.
pub fn init() -> Vec<GpuInfo> {
    warn!("Unknown platform: no GPU info available.");
    Vec::new()
}

/// Updates GPU information on unsupported platforms.
///
/// This function is a no-op and logs a warning, as GPU updates
/// are not available on unsupported platforms.
pub fn update(_gpu: &mut GpuInfo) {
    warn!("Unknown platform: cannot update GPU info.");
}
