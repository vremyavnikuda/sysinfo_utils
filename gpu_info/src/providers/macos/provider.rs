//! Main MacosProvider implementation
//!
//! Orchestrates all backends, caching, and routing for macOS GPU operations.

use super::backends::{PowerMetricsBackend, SystemProfilerBackend};
use super::cache::GpuCache;
use super::config::{MacosBackend, MacosConfig};
use super::metrics::MacosMetrics;
use super::router::{BackendRouter, Operation};
use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use log::{debug, info, warn};
use std::time::Instant;

#[cfg(feature = "macos-iokit")]
use super::backends::IOKitBackend;

#[cfg(feature = "macos-metal")]
use super::backends::MetalBackend;

/// Main GPU provider for macOS
///
/// Supports multiple backends with automatic fallback and smart caching.
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// use gpu_info::providers::macos::MacosProvider;
/// use gpu_info::gpu_info::GpuProvider;
///
/// let provider = MacosProvider::new().expect("Failed to create provider");
/// let gpus = provider.detect_gpus().expect("Failed to detect GPUs");
/// println!("Found {} GPUs", gpus.len());
/// ```
///
/// With custom configuration:
/// ```no_run
/// use gpu_info::providers::macos::{MacosProvider, MacosProviderBuilder, MacosBackend};
/// use std::time::Duration;
///
/// let provider = MacosProviderBuilder::new()
///     .cache_ttl(Duration::from_secs(120))
///     .backend(MacosBackend::Hybrid)
///     .build()
///     .expect("Failed to create provider");
/// ```
pub struct MacosProvider {
    /// Configuration
    config: MacosConfig,
    /// Backend router
    router: BackendRouter,
    /// Static information cache
    cache: GpuCache,
    /// Last operation metrics
    last_metrics: Option<MacosMetrics>,
    /// Backends
    system_profiler: SystemProfilerBackend,
    powermetrics: PowerMetricsBackend,
    #[cfg(feature = "macos-iokit")]
    iokit: Option<IOKitBackend>,
    #[cfg(feature = "macos-metal")]
    metal: Option<MetalBackend>,
}

impl MacosProvider {
    /// Creates a new provider with default configuration
    ///
    /// # Errors
    ///
    /// Returns an error if provider initialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gpu_info::providers::macos::MacosProvider;
    ///
    /// let provider = MacosProvider::new().expect("Failed to create provider");
    /// ```
    pub fn new() -> Result<Self> {
        Self::with_config(MacosConfig::default())
    }

    /// Creates a provider with custom configuration
    ///
    /// # Errors
    ///
    /// Returns an error if provider initialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gpu_info::providers::macos::{MacosProvider, MacosConfig, MacosBackend};
    /// use std::time::Duration;
    ///
    /// let config = MacosConfig {
    ///     cache_ttl: Duration::from_secs(120),
    ///     preferred_backend: MacosBackend::Hybrid,
    ///     fallback_enabled: true,
    ///     profiler_timeout: Duration::from_secs(2),
    /// };
    ///
    /// let provider = MacosProvider::with_config(config).expect("Failed to create provider");
    /// ```
    pub fn with_config(config: MacosConfig) -> Result<Self> {
        info!("Initializing MacosProvider with config: {:?}", config);

        let router = BackendRouter::new(config.clone());
        let cache = GpuCache::new(config.cache_ttl);

        // Initialize backends
        let system_profiler = SystemProfilerBackend::new();
        let powermetrics = PowerMetricsBackend::new();

        #[cfg(feature = "macos-iokit")]
        let iokit = IOKitBackend::new().ok();

        #[cfg(feature = "macos-metal")]
        let metal = MetalBackend::new().ok();

        info!(
            "Available backends: {} total",
            router.available_count()
        );

        Ok(Self {
            config,
            router,
            cache,
            last_metrics: None,
            system_profiler,
            powermetrics,
            #[cfg(feature = "macos-iokit")]
            iokit,
            #[cfg(feature = "macos-metal")]
            metal,
        })
    }

    /// Returns the current configuration
    pub fn config(&self) -> &MacosConfig {
        &self.config
    }

    /// Returns metrics from the last operation
    pub fn last_metrics(&self) -> Option<&MacosMetrics> {
        self.last_metrics.as_ref()
    }

    /// Returns list of available backends
    pub fn available_backends(&self) -> Vec<MacosBackend> {
        self.router.available_backends().to_vec()
    }

    /// Checks if a specific backend is available
    pub fn is_backend_available(&self, backend: MacosBackend) -> bool {
        self.router.is_backend_available(backend)
    }

    /// Clears the static information cache
    pub fn clear_cache(&mut self) {
        info!("Clearing GPU cache");
        self.cache.clear();
    }

    /// Updates metrics after an operation
    fn update_metrics(&mut self, backend_used: MacosBackend, duration: std::time::Duration) {
        let cache_hit = duration.as_millis() < 10; // Assume cache hit if very fast
        let metrics = MacosMetrics::new(
            backend_used,
            duration,
            cache_hit,
            self.router.available_count(),
        );

        debug!(
            "Operation completed: backend={}, time={}ms, cache_hit={}",
            backend_used.name(),
            metrics.detection_time_ms,
            metrics.cache_hit
        );

        self.last_metrics = Some(metrics);
    }

    /// Detects GPUs using the selected backend
    fn detect_gpus_with_backend(&self, backend: MacosBackend) -> Result<Vec<GpuInfo>> {
        match backend {
            #[cfg(feature = "macos-iokit")]
            MacosBackend::IOKit => {
                if let Some(ref iokit) = self.iokit {
                    return iokit.detect_gpus();
                }
                warn!("IOKit backend not available, falling back");
                self.system_profiler.detect_gpus()
            }
            #[cfg(feature = "macos-metal")]
            MacosBackend::Metal => {
                if let Some(ref metal) = self.metal {
                    return metal.detect_gpus();
                }
                warn!("Metal backend not available, falling back");
                self.system_profiler.detect_gpus()
            }
            MacosBackend::PowerMetrics => {
                // PowerMetrics is primarily for metrics, not detection
                warn!("PowerMetrics not suitable for detection, using SystemProfiler");
                self.system_profiler.detect_gpus()
            }
            MacosBackend::SystemProfiler => self.system_profiler.detect_gpus(),
            MacosBackend::Hybrid => {
                // This should not happen as router should resolve Hybrid
                self.system_profiler.detect_gpus()
            }
        }
    }

    /// Updates GPU metrics using the selected backend
    fn update_gpu_with_backend(&self, gpu: &mut GpuInfo, backend: MacosBackend) -> Result<()> {
        match backend {
            #[cfg(feature = "macos-metal")]
            MacosBackend::Metal => {
                if let Some(ref metal) = self.metal {
                    return metal.update_gpu(gpu);
                }
                // Fallback to powermetrics
                self.powermetrics.update_gpu(gpu)
            }
            MacosBackend::PowerMetrics => self.powermetrics.update_gpu(gpu),
            _ => {
                // Other backends don't provide real-time metrics yet
                // Just return Ok without updating
                Ok(())
            }
        }
    }
}

impl GpuProvider for MacosProvider {
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        let start = Instant::now();
        let backend = self.router.select_backend(Operation::DetectGpu);

        debug!("Detecting GPUs with backend: {}", backend.name());

        let gpus = self.detect_gpus_with_backend(backend)?;

        // Note: We can't update self.last_metrics here because &self is immutable
        // This will be fixed when we refactor to use interior mutability

        info!("Detected {} GPUs in {:?}", gpus.len(), start.elapsed());

        Ok(gpus)
    }

    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        let start = Instant::now();
        let backend = self.router.select_backend(Operation::GetDynamicMetrics);

        debug!(
            "Updating GPU metrics with backend: {}",
            backend.name()
        );

        let result = self.update_gpu_with_backend(gpu, backend);

        debug!("GPU metrics updated in {:?}", start.elapsed());

        result
    }

    fn get_vendor(&self) -> crate::vendor::Vendor {
        // For macOS, the vendor is typically determined during detection
        // Return Apple as default for Apple Silicon, Unknown otherwise
        #[cfg(target_arch = "aarch64")]
        {
            crate::vendor::Vendor::Apple
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            crate::vendor::Vendor::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let result = MacosProvider::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_provider_with_custom_config() {
        let config = MacosConfig {
            cache_ttl: std::time::Duration::from_secs(120),
            ..Default::default()
        };

        let result = MacosProvider::with_config(config);
        assert!(result.is_ok());

        if let Ok(provider) = result {
            assert_eq!(provider.config().cache_ttl, std::time::Duration::from_secs(120));
        }
    }

    #[test]
    fn test_available_backends() {
        let provider = MacosProvider::new().expect("Failed to create provider");
        let backends = provider.available_backends();

        // Should have at least SystemProfiler
        assert!(!backends.is_empty());
        assert!(backends.contains(&MacosBackend::SystemProfiler));
    }

    #[test]
    fn test_is_backend_available() {
        let provider = MacosProvider::new().expect("Failed to create provider");

        // SystemProfiler should always be available
        assert!(provider.is_backend_available(MacosBackend::SystemProfiler));
    }

    #[test]
    fn test_clear_cache() {
        let mut provider = MacosProvider::new().expect("Failed to create provider");
        provider.clear_cache(); // Should not panic
    }

    #[test]
    fn test_config_access() {
        let config = MacosConfig::default();
        let provider = MacosProvider::with_config(config.clone()).expect("Failed to create provider");

        assert_eq!(provider.config().cache_ttl, config.cache_ttl);
        assert_eq!(provider.config().preferred_backend, config.preferred_backend);
    }
}
