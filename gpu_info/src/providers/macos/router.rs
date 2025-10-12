//! Smart backend routing for macOS GPU operations
//!
//! Intelligently selects the best backend for each type of operation
//! based on availability, performance, and configuration.

use super::config::{MacosBackend, MacosConfig};

/// Type of GPU operation to perform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// Detect GPUs in the system
    DetectGpu,
    /// Get dynamic metrics (utilization, temperature, etc.)
    GetDynamicMetrics,
    /// Get static information (name, vendor, memory)
    GetStaticInfo,
    /// Get temperature specifically
    GetTemperature,
}

/// Smart router for selecting optimal backends
///
/// The router considers:
/// - User preferences from configuration
/// - Backend availability
/// - Operation type
/// - Fallback chain when enabled
pub struct BackendRouter {
    /// Available backends on this system
    available_backends: Vec<MacosBackend>,
    /// Configuration preferences
    config: MacosConfig,
}

impl BackendRouter {
    /// Creates a new router with the given configuration
    ///
    /// Automatically detects which backends are available.
    pub fn new(config: MacosConfig) -> Self {
        let available_backends = Self::detect_available_backends();

        Self {
            available_backends,
            config,
        }
    }

    /// Detects which backends are currently available
    fn detect_available_backends() -> Vec<MacosBackend> {
        let mut backends = Vec::new();

        // Check IOKit
        #[cfg(feature = "macos-iokit")]
        {
            if MacosBackend::IOKit.is_available() {
                backends.push(MacosBackend::IOKit);
            }
        }

        // Check Metal
        #[cfg(feature = "macos-metal")]
        {
            if MacosBackend::Metal.is_available() {
                backends.push(MacosBackend::Metal);
            }
        }

        // Check PowerMetrics
        if MacosBackend::PowerMetrics.is_available() {
            backends.push(MacosBackend::PowerMetrics);
        }

        // SystemProfiler is always available
        backends.push(MacosBackend::SystemProfiler);

        backends
    }

    /// Selects the optimal backend for the given operation
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::router::{BackendRouter, Operation};
    /// use gpu_info::providers::macos::MacosConfig;
    ///
    /// let router = BackendRouter::new(MacosConfig::default());
    /// let backend = router.select_backend(Operation::DetectGpu);
    /// ```
    pub fn select_backend(&self, operation: Operation) -> MacosBackend {
        // If preferred backend is not Hybrid, try to use it
        if self.config.preferred_backend != MacosBackend::Hybrid {
            if self.is_backend_available(self.config.preferred_backend) {
                return self.config.preferred_backend;
            }

            // If fallback is disabled, still return preferred even if unavailable
            if !self.config.fallback_enabled {
                return self.config.preferred_backend;
            }
        }

        // Hybrid mode or fallback - select based on operation
        match operation {
            Operation::DetectGpu | Operation::GetStaticInfo => self.select_for_detection(),
            Operation::GetDynamicMetrics => self.select_for_metrics(),
            Operation::GetTemperature => self.select_for_temperature(),
        }
    }

    /// Selects the best backend for GPU detection
    fn select_for_detection(&self) -> MacosBackend {
        // Priority: IOKit > SystemProfiler
        #[cfg(feature = "macos-iokit")]
        {
            if self.is_backend_available(MacosBackend::IOKit) {
                return MacosBackend::IOKit;
            }
        }

        MacosBackend::SystemProfiler
    }

    /// Selects the best backend for dynamic metrics
    fn select_for_metrics(&self) -> MacosBackend {
        // Priority: Metal > PowerMetrics > SystemProfiler
        #[cfg(feature = "macos-metal")]
        {
            if self.is_backend_available(MacosBackend::Metal) {
                return MacosBackend::Metal;
            }
        }

        if self.is_backend_available(MacosBackend::PowerMetrics) {
            return MacosBackend::PowerMetrics;
        }

        MacosBackend::SystemProfiler
    }

    /// Selects the best backend for temperature reading
    fn select_for_temperature(&self) -> MacosBackend {
        // Priority: IOKit/Metal > PowerMetrics > SystemProfiler
        #[cfg(feature = "macos-iokit")]
        {
            if self.is_backend_available(MacosBackend::IOKit) {
                return MacosBackend::IOKit;
            }
        }

        #[cfg(feature = "macos-metal")]
        {
            if self.is_backend_available(MacosBackend::Metal) {
                return MacosBackend::Metal;
            }
        }

        if self.is_backend_available(MacosBackend::PowerMetrics) {
            return MacosBackend::PowerMetrics;
        }

        MacosBackend::SystemProfiler
    }

    /// Checks if a backend is available
    pub fn is_backend_available(&self, backend: MacosBackend) -> bool {
        self.available_backends.contains(&backend)
    }

    /// Returns all available backends
    pub fn available_backends(&self) -> &[MacosBackend] {
        &self.available_backends
    }

    /// Returns the number of available backends
    pub fn available_count(&self) -> u8 {
        self.available_backends.len() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let config = MacosConfig::default();
        let router = BackendRouter::new(config);

        // Should have at least SystemProfiler
        assert!(!router.available_backends.is_empty());
        assert!(router.is_backend_available(MacosBackend::SystemProfiler));
    }

    #[test]
    fn test_select_backend_with_hybrid() {
        let config = MacosConfig {
            preferred_backend: MacosBackend::Hybrid,
            ..Default::default()
        };
        let router = BackendRouter::new(config);

        // Should select appropriate backends for different operations
        let detection = router.select_backend(Operation::DetectGpu);
        let metrics = router.select_backend(Operation::GetDynamicMetrics);

        // Both should be valid backends
        assert!(router.is_backend_available(detection));
        assert!(router.is_backend_available(metrics));
    }

    #[test]
    fn test_available_count() {
        let router = BackendRouter::new(MacosConfig::default());
        let count = router.available_count();

        // Should have at least 1 (SystemProfiler)
        assert!(count >= 1);
    }

    #[test]
    fn test_backend_selection_consistency() {
        let router = BackendRouter::new(MacosConfig::default());

        // Same operation should return same backend
        let backend1 = router.select_backend(Operation::DetectGpu);
        let backend2 = router.select_backend(Operation::DetectGpu);

        assert_eq!(backend1, backend2);
    }
}
