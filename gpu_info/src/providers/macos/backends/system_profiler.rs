//! System Profiler backend for macOS GPU detection
//!
//! Uses the `system_profiler` command-line tool to detect GPUs.
//! This is the most compatible method but also the slowest (~500-1000ms).

use crate::gpu_info::{GpuInfo, Result};
use log::debug;

/// Backend that uses system_profiler for GPU detection
///
/// # Performance
///
/// - Detection speed: 500-1000ms
/// - Always available on macOS
/// - No additional dependencies required
///
/// # Examples
///
/// ```no_run
/// use gpu_info::providers::macos::backends::SystemProfilerBackend;
///
/// let backend = SystemProfilerBackend::new();
/// if let Ok(gpus) = backend.detect_gpus() {
///     println!("Found {} GPUs", gpus.len());
/// }
/// ```
pub struct SystemProfilerBackend;

impl SystemProfilerBackend {
    /// Creates a new SystemProfiler backend
    pub fn new() -> Self {
        Self
    }

    /// Detects all GPUs using system_profiler
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - system_profiler command fails
    /// - Output parsing fails
    pub fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        debug!("Detecting GPUs using system_profiler");

        // TODO: Implement actual detection logic
        // This will be migrated from the old macos.rs implementation

        Ok(vec![])
    }

    /// Updates dynamic GPU metrics
    ///
    /// # Errors
    ///
    /// Returns an error if metrics cannot be retrieved
    pub fn update_gpu(&self, _gpu: &mut GpuInfo) -> Result<()> {
        // TODO: Implement update logic
        Ok(())
    }
}

impl Default for SystemProfilerBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = SystemProfilerBackend::new();
        // Backend should be created successfully
        let _ = backend;
    }

    #[test]
    fn test_default() {
        let backend = SystemProfilerBackend::default();
        let _ = backend;
    }
}
