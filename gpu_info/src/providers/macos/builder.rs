//! Builder for MacosProvider with fluent API
//!
//! Provides a type-safe builder pattern with method chaining for creating
//! configured MacosProvider instances.

use super::config::{MacosBackend, MacosConfig};
use super::provider::MacosProvider;
use crate::gpu_info::Result;
use std::time::Duration;

/// Builder for constructing a [`MacosProvider`] with custom configuration
///
/// Uses the builder pattern with method chaining for ergonomic API.
/// All configuration options have sensible defaults.
///
/// # Examples
///
/// Basic usage with defaults:
/// ```no_run
/// # use gpu_info::providers::macos::MacosProviderBuilder;
/// let provider = MacosProviderBuilder::new()
///     .build()
///     .expect("Failed to create provider");
/// ```
///
/// Custom configuration:
/// ```no_run
/// # use gpu_info::providers::macos::{MacosProviderBuilder, MacosBackend};
/// # use std::time::Duration;
/// let provider = MacosProviderBuilder::new()
///     .cache_ttl(Duration::from_secs(120))
///     .backend(MacosBackend::Hybrid)
///     .fallback(true)
///     .profiler_timeout(Duration::from_secs(3))
///     .build()
///     .expect("Failed to create provider");
/// ```
#[derive(Debug, Clone)]
pub struct MacosProviderBuilder {
    config: MacosConfig,
}

impl MacosProviderBuilder {
    /// Creates a new builder with default configuration
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::MacosProviderBuilder;
    ///
    /// let builder = MacosProviderBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            config: MacosConfig::default(),
        }
    }

    /// Sets the TTL (time-to-live) for cached static GPU information
    ///
    /// Static information like GPU name, vendor, and total memory is cached
    /// to avoid expensive repeated queries. This sets how long that cache remains valid.
    ///
    /// Default: 60 seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::MacosProviderBuilder;
    /// use std::time::Duration;
    ///
    /// let builder = MacosProviderBuilder::new()
    ///     .cache_ttl(Duration::from_secs(120));
    /// ```
    #[inline]
    pub fn cache_ttl(mut self, duration: Duration) -> Self {
        self.config.cache_ttl = duration;
        self
    }

    /// Sets the preferred backend for GPU operations
    ///
    /// The backend determines which system API or tool is used. Different backends
    /// have different performance and capability trade-offs.
    ///
    /// Default: [`MacosBackend::Hybrid`]
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::{MacosProviderBuilder, MacosBackend};
    ///
    /// let builder = MacosProviderBuilder::new()
    ///     .backend(MacosBackend::SystemProfiler);
    /// ```
    #[inline]
    pub fn backend(mut self, backend: MacosBackend) -> Self {
        self.config.preferred_backend = backend;
        self
    }

    /// Enables or disables the fallback chain
    ///
    /// When enabled, if the preferred backend fails, the provider will automatically
    /// try other available backends. When disabled, failures will propagate as errors.
    ///
    /// Default: true
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::MacosProviderBuilder;
    ///
    /// let builder = MacosProviderBuilder::new()
    ///     .fallback(false); // Strict mode - no fallbacks
    /// ```
    #[inline]
    pub fn fallback(mut self, enabled: bool) -> Self {
        self.config.fallback_enabled = enabled;
        self
    }

    /// Sets the timeout for the system_profiler command
    ///
    /// The system_profiler tool can be slow on some systems. This sets the maximum
    /// time to wait for it to complete before giving up.
    ///
    /// Default: 2 seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::MacosProviderBuilder;
    /// use std::time::Duration;
    ///
    /// let builder = MacosProviderBuilder::new()
    ///     .profiler_timeout(Duration::from_secs(5));
    /// ```
    #[inline]
    pub fn profiler_timeout(mut self, duration: Duration) -> Self {
        self.config.profiler_timeout = duration;
        self
    }

    /// Builds the [`MacosProvider`] with the configured options
    ///
    /// This is a consuming method that takes ownership of the builder.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider cannot be initialized with the given configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gpu_info::providers::macos::MacosProviderBuilder;
    ///
    /// let provider = MacosProviderBuilder::new()
    ///     .build()
    ///     .expect("Failed to create provider");
    /// ```
    pub fn build(self) -> Result<MacosProvider> {
        MacosProvider::with_config(self.config)
    }
}

impl Default for MacosProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = MacosProviderBuilder::new();
        assert_eq!(builder.config.cache_ttl, Duration::from_secs(60));
        assert_eq!(builder.config.preferred_backend, MacosBackend::Hybrid);
        assert!(builder.config.fallback_enabled);
        assert_eq!(builder.config.profiler_timeout, Duration::from_secs(2));
    }

    #[test]
    fn test_builder_chain() {
        let builder = MacosProviderBuilder::new()
            .cache_ttl(Duration::from_secs(120))
            .backend(MacosBackend::SystemProfiler)
            .fallback(false)
            .profiler_timeout(Duration::from_secs(5));

        assert_eq!(builder.config.cache_ttl, Duration::from_secs(120));
        assert_eq!(
            builder.config.preferred_backend,
            MacosBackend::SystemProfiler
        );
        assert!(!builder.config.fallback_enabled);
        assert_eq!(builder.config.profiler_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_builder_partial_chain() {
        let builder = MacosProviderBuilder::new()
            .cache_ttl(Duration::from_secs(90))
            .backend(MacosBackend::PowerMetrics);

        assert_eq!(builder.config.cache_ttl, Duration::from_secs(90));
        assert_eq!(builder.config.preferred_backend, MacosBackend::PowerMetrics);
        assert!(builder.config.fallback_enabled); // unchanged
        assert_eq!(builder.config.profiler_timeout, Duration::from_secs(2)); // unchanged
    }

    #[test]
    fn test_default_trait() {
        let builder1 = MacosProviderBuilder::new();
        let builder2 = MacosProviderBuilder::default();

        assert_eq!(builder1.config.cache_ttl, builder2.config.cache_ttl);
        assert_eq!(
            builder1.config.preferred_backend,
            builder2.config.preferred_backend
        );
    }

    #[test]
    fn test_clone() {
        let builder1 = MacosProviderBuilder::new()
            .cache_ttl(Duration::from_secs(100))
            .backend(MacosBackend::SystemProfiler);

        let builder2 = builder1.clone();

        assert_eq!(builder1.config.cache_ttl, builder2.config.cache_ttl);
        assert_eq!(
            builder1.config.preferred_backend,
            builder2.config.preferred_backend
        );
    }
}
