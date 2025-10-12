//! Performance metrics for macOS GPU provider
//!
//! This module tracks performance metrics for GPU operations on macOS.

use super::config::MacosBackend;
use std::time::Duration;

/// Performance metrics for macOS GPU operations
///
/// Tracks which backend was used, timing information, and cache statistics.
///
/// # Examples
///
/// ```
/// use gpu_info::providers::macos::{MacosMetrics, MacosBackend};
///
/// let metrics = MacosMetrics {
///     backend_used: MacosBackend::Hybrid,
///     detection_time_ms: 5,
///     cache_hit: false,
///     available_backends: 3,
/// };
///
/// println!("Detection took {}ms using {:?}", metrics.detection_time_ms, metrics.backend_used);
/// ```
#[derive(Debug, Clone)]
pub struct MacosMetrics {
    /// The backend that was actually used for the operation
    pub backend_used: MacosBackend,

    /// Time taken for GPU detection in milliseconds
    pub detection_time_ms: u64,

    /// Whether the result was served from cache
    pub cache_hit: bool,

    /// Number of backends available on this system
    pub available_backends: u8,
}

impl MacosMetrics {
    /// Creates a new metrics instance
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::{MacosMetrics, MacosBackend};
    /// use std::time::Duration;
    ///
    /// let duration = Duration::from_millis(10);
    /// let metrics = MacosMetrics::new(MacosBackend::Hybrid, duration, false, 3);
    /// assert_eq!(metrics.detection_time_ms, 10);
    /// ```
    #[inline]
    pub fn new(
        backend_used: MacosBackend,
        duration: Duration,
        cache_hit: bool,
        available_backends: u8,
    ) -> Self {
        Self {
            backend_used,
            detection_time_ms: duration.as_millis() as u64,
            cache_hit,
            available_backends,
        }
    }

    /// Returns true if the operation was fast (< 50ms)
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::{MacosMetrics, MacosBackend};
    /// use std::time::Duration;
    ///
    /// let fast = MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(10), false, 3);
    /// let slow = MacosMetrics::new(MacosBackend::SystemProfiler, Duration::from_millis(600), false, 3);
    ///
    /// assert!(fast.is_fast());
    /// assert!(!slow.is_fast());
    /// ```
    #[inline]
    pub fn is_fast(&self) -> bool {
        self.detection_time_ms < 50
    }

    /// Returns a performance rating from 0.0 (slowest) to 1.0 (fastest)
    ///
    /// Based on expected performance characteristics:
    /// - 0-10ms: 1.0 (excellent)
    /// - 10-50ms: 0.8 (good)
    /// - 50-200ms: 0.5 (acceptable)
    /// - 200-500ms: 0.3 (slow)
    /// - 500ms+: 0.1 (very slow)
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::{MacosMetrics, MacosBackend};
    /// use std::time::Duration;
    ///
    /// let metrics = MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(5), false, 3);
    /// assert!(metrics.performance_rating() > 0.9);
    /// ```
    pub fn performance_rating(&self) -> f32 {
        match self.detection_time_ms {
            0..=10 => 1.0,
            11..=50 => 0.8,
            51..=200 => 0.5,
            201..=500 => 0.3,
            _ => 0.1,
        }
    }

    /// Returns a human-readable performance description
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::{MacosMetrics, MacosBackend};
    /// use std::time::Duration;
    ///
    /// let metrics = MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(5), false, 3);
    /// assert_eq!(metrics.performance_description(), "Excellent");
    /// ```
    pub fn performance_description(&self) -> &'static str {
        match self.detection_time_ms {
            0..=10 => "Excellent",
            11..=50 => "Good",
            51..=200 => "Acceptable",
            201..=500 => "Slow",
            _ => "Very Slow",
        }
    }
}

impl Default for MacosMetrics {
    fn default() -> Self {
        Self {
            backend_used: MacosBackend::Hybrid,
            detection_time_ms: 0,
            cache_hit: false,
            available_backends: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let duration = Duration::from_millis(42);
        let metrics = MacosMetrics::new(MacosBackend::SystemProfiler, duration, false, 2);

        assert_eq!(metrics.backend_used, MacosBackend::SystemProfiler);
        assert_eq!(metrics.detection_time_ms, 42);
        assert!(!metrics.cache_hit);
        assert_eq!(metrics.available_backends, 2);
    }

    #[test]
    fn test_is_fast() {
        let fast = MacosMetrics::new(
            MacosBackend::Hybrid,
            Duration::from_millis(10),
            false,
            3,
        );
        let slow = MacosMetrics::new(
            MacosBackend::SystemProfiler,
            Duration::from_millis(600),
            false,
            3,
        );

        assert!(fast.is_fast());
        assert!(!slow.is_fast());
    }

    #[test]
    fn test_performance_rating() {
        let excellent =
            MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(5), false, 3);
        let good = MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(30), false, 3);
        let acceptable =
            MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(100), false, 3);
        let slow =
            MacosMetrics::new(MacosBackend::SystemProfiler, Duration::from_millis(300), false, 3);
        let very_slow =
            MacosMetrics::new(MacosBackend::SystemProfiler, Duration::from_millis(800), false, 3);

        assert_eq!(excellent.performance_rating(), 1.0);
        assert_eq!(good.performance_rating(), 0.8);
        assert_eq!(acceptable.performance_rating(), 0.5);
        assert_eq!(slow.performance_rating(), 0.3);
        assert_eq!(very_slow.performance_rating(), 0.1);
    }

    #[test]
    fn test_performance_description() {
        assert_eq!(
            MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(5), false, 3)
                .performance_description(),
            "Excellent"
        );
        assert_eq!(
            MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(30), false, 3)
                .performance_description(),
            "Good"
        );
        assert_eq!(
            MacosMetrics::new(MacosBackend::Hybrid, Duration::from_millis(100), false, 3)
                .performance_description(),
            "Acceptable"
        );
    }

    #[test]
    fn test_default_metrics() {
        let metrics = MacosMetrics::default();
        assert_eq!(metrics.backend_used, MacosBackend::Hybrid);
        assert_eq!(metrics.detection_time_ms, 0);
        assert!(!metrics.cache_hit);
        assert_eq!(metrics.available_backends, 1);
    }

    #[test]
    fn test_clone() {
        let original = MacosMetrics::new(
            MacosBackend::PowerMetrics,
            Duration::from_millis(100),
            true,
            4,
        );
        let cloned = original.clone();

        assert_eq!(original.backend_used, cloned.backend_used);
        assert_eq!(original.detection_time_ms, cloned.detection_time_ms);
        assert_eq!(original.cache_hit, cloned.cache_hit);
        assert_eq!(original.available_backends, cloned.available_backends);
    }
}
