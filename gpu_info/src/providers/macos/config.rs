//! Configuration types for macOS GPU provider
//!
//! This module defines the configuration options and backend types for the macOS provider.

use std::time::Duration;

/// Configuration for the macOS GPU provider
///
/// # Examples
///
/// ```
/// use gpu_info::providers::macos::MacosConfig;
/// use std::time::Duration;
///
/// let config = MacosConfig {
///     cache_ttl: Duration::from_secs(120),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct MacosConfig {
    /// TTL (time-to-live) for cached static GPU information
    ///
    /// Default: 60 seconds
    pub cache_ttl: Duration,

    /// Preferred backend to use for GPU operations
    ///
    /// Default: [`MacosBackend::Hybrid`]
    pub preferred_backend: MacosBackend,

    /// Enable fallback chain when preferred backend fails
    ///
    /// Default: true
    pub fallback_enabled: bool,

    /// Timeout for system_profiler command
    ///
    /// Default: 2 seconds
    pub profiler_timeout: Duration,
}

impl Default for MacosConfig {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(60),
            preferred_backend: MacosBackend::Hybrid,
            fallback_enabled: true,
            profiler_timeout: Duration::from_secs(2),
        }
    }
}

/// Types of backends available for macOS GPU detection
///
/// The backend determines which system API or tool is used to gather GPU information.
/// Different backends have different performance characteristics and capabilities.
///
/// # Performance Comparison
///
/// - [`IOKit`](MacosBackend::IOKit): 1-10ms (fastest)
/// - [`Metal`](MacosBackend::Metal): 1-5ms (real-time metrics)
/// - [`PowerMetrics`](MacosBackend::PowerMetrics): ~100ms (CLI tool)
/// - [`SystemProfiler`](MacosBackend::SystemProfiler): 500-1000ms (slowest, most compatible)
///
/// # Examples
///
/// ```
/// use gpu_info::providers::macos::MacosBackend;
///
/// let backend = MacosBackend::Hybrid; // Automatically selects best available
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MacosBackend {
    /// IOKit backend - fast PCI device detection and static information
    ///
    /// **Speed:** 1-10ms
    /// **Requires:** `macos-iokit` feature flag
    ///
    /// Uses IOKit framework for direct hardware access. Fastest option for
    /// GPU detection and retrieving static information like vendor, model, and memory.
    #[cfg(feature = "macos-iokit")]
    IOKit,

    /// Metal API backend - real-time GPU metrics
    ///
    /// **Speed:** 1-5ms
    /// **Requires:** `macos-metal` feature flag
    ///
    /// Uses Metal framework for real-time GPU utilization, memory usage,
    /// and other dynamic metrics. Most accurate for runtime information.
    #[cfg(feature = "macos-metal")]
    Metal,

    /// powermetrics CLI tool - GPU metrics without sudo (if available)
    ///
    /// **Speed:** ~100ms
    /// **Requires:** No feature flags
    ///
    /// Uses the `powermetrics` command-line tool. May require sudo access
    /// for full metrics, but provides basic information without privileges.
    PowerMetrics,

    /// system_profiler - legacy fallback method
    ///
    /// **Speed:** 500-1000ms
    /// **Requires:** No feature flags
    ///
    /// Uses the `system_profiler` command. Slowest but most compatible option.
    /// Works on all macOS versions without additional dependencies.
    SystemProfiler,

    /// Hybrid mode - automatically selects the best available backend
    ///
    /// **Speed:** Varies (uses fastest available)
    ///
    /// Intelligently routes operations to the most appropriate backend:
    /// - Static detection: IOKit > SystemProfiler
    /// - Dynamic metrics: Metal > PowerMetrics > Estimation
    /// - Temperature: IOKit/Metal > Estimation
    Hybrid,
}

impl MacosBackend {
    /// Returns a human-readable name for the backend
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "macos-iokit")]
            Self::IOKit => "IOKit",
            #[cfg(feature = "macos-metal")]
            Self::Metal => "Metal",
            Self::PowerMetrics => "PowerMetrics",
            Self::SystemProfiler => "system_profiler",
            Self::Hybrid => "Hybrid",
        }
    }

    /// Returns true if this backend is available on the current system
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::MacosBackend;
    ///
    /// if MacosBackend::SystemProfiler.is_available() {
    ///     println!("system_profiler is available");
    /// }
    /// ```
    pub fn is_available(&self) -> bool {
        match self {
            #[cfg(feature = "macos-iokit")]
            // IOKit is always available on macOS with feature flag
            Self::IOKit => true,
            #[cfg(feature = "macos-metal")]
            // Metal is always available on macOS 10.11+ with feature flag
            Self::Metal => true,
            Self::PowerMetrics => {
                // Check if powermetrics command exists
                std::process::Command
                    ::new("which")
                    .arg("powermetrics")
                    .output()
                    .map(|output| output.status.success())
                    .unwrap_or(false)
            }
            Self::SystemProfiler => {
                // system_profiler is always available on macOS
                true
            }
            // Hybrid is always available (uses available backends)
            Self::Hybrid => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = MacosConfig::default();
        assert_eq!(config.cache_ttl, Duration::from_secs(60));
        assert_eq!(config.preferred_backend, MacosBackend::Hybrid);
        assert!(config.fallback_enabled);
        assert_eq!(config.profiler_timeout, Duration::from_secs(2));
    }

    #[test]
    fn test_backend_names() {
        assert_eq!(MacosBackend::SystemProfiler.name(), "system_profiler");
        assert_eq!(MacosBackend::PowerMetrics.name(), "PowerMetrics");
        assert_eq!(MacosBackend::Hybrid.name(), "Hybrid");
    }

    #[test]
    fn test_backend_availability() {
        // system_profiler should always be available on macOS
        assert!(MacosBackend::SystemProfiler.is_available());
        assert!(MacosBackend::Hybrid.is_available());
    }

    #[test]
    fn test_config_clone() {
        let config1 = MacosConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.cache_ttl, config2.cache_ttl);
        assert_eq!(config1.preferred_backend, config2.preferred_backend);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_backend_serialization() {
        let backend = MacosBackend::Hybrid;
        let json = serde_json::to_string(&backend).expect("Failed to serialize");
        let deserialized: MacosBackend = serde_json
            ::from_str(&json)
            .expect("Failed to deserialize");
        assert_eq!(backend, deserialized);
    }
}
