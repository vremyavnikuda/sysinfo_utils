//! Metric value type that distinguishes between available, unavailable, and unsupported metrics.
//!
//! This module provides a type-safe way to represent GPU metrics that may be:
//! - Available (has a value)
//! - Unavailable (temporarily not readable, error occurred)
//! - NotSupported (driver/hardware doesn't support this metric)
//!
//! `MetricValue<T>` implements `Default`, returning `Unavailable` as the default state.

use std::fmt;

/// Represents a GPU metric value with three possible states
///
/// # States
/// - `Available(T)` - Metric is supported and has a value
/// - `Unavailable` - Metric is supported but temporarily not readable (error, not initialized)
/// - `NotSupported` - Driver/hardware doesn't support this metric
///
/// # Example
/// ```
/// use gpu_info::MetricValue;
///
/// let temp: MetricValue<f32> = MetricValue::Available(65.5);
/// let power: MetricValue<f32> = MetricValue::NotSupported;
///
/// assert_eq!(temp.format_with_unit("°C"), "65.5°C");
/// assert_eq!(power.format_with_unit("W"), "N/A (not supported by driver)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[non_exhaustive]
pub enum MetricValue<T> {
    /// Metric is available and has a value
    Available(T),
    /// Metric is supported but temporarily unavailable (error, not initialized)
    #[default]
    Unavailable,
    /// Metric is not supported by the driver/hardware
    NotSupported,
}

impl<T> MetricValue<T> {
    /// Returns `true` if the metric is available
    pub fn is_available(&self) -> bool {
        matches!(self, MetricValue::Available(_))
    }

    /// Returns `true` if the metric is not supported
    pub fn is_not_supported(&self) -> bool {
        matches!(self, MetricValue::NotSupported)
    }

    /// Returns `true` if the metric is unavailable
    pub fn is_unavailable(&self) -> bool {
        matches!(self, MetricValue::Unavailable)
    }

    /// Returns the value if available, otherwise `None`
    pub fn value(&self) -> Option<&T> {
        match self {
            MetricValue::Available(v) => Some(v),
            _ => None,
        }
    }

    /// Converts to `Option<T>`, losing the distinction between Unavailable and NotSupported
    pub fn into_option(self) -> Option<T> {
        match self {
            MetricValue::Available(v) => Some(v),
            _ => None,
        }
    }

    /// Maps the value if available
    pub fn map<U, F>(self, f: F) -> MetricValue<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            MetricValue::Available(v) => MetricValue::Available(f(v)),
            MetricValue::Unavailable => MetricValue::Unavailable,
            MetricValue::NotSupported => MetricValue::NotSupported,
        }
    }
}

impl<T: fmt::Display> MetricValue<T> {
    /// Formats the metric value with a unit
    ///
    /// # Example
    /// ```
    /// use gpu_info::MetricValue;
    ///
    /// let temp = MetricValue::Available(65.5);
    /// assert_eq!(temp.format_with_unit("°C"), "65.5°C");
    ///
    /// let power: MetricValue<f32> = MetricValue::NotSupported;
    /// assert_eq!(power.format_with_unit("W"), "N/A (not supported by driver)");
    /// ```
    pub fn format_with_unit(&self, unit: &str) -> String {
        match self {
            MetricValue::Available(v) => format!("{}{}", v, unit),
            MetricValue::Unavailable => "N/A".to_string(),
            MetricValue::NotSupported => "N/A (not supported by driver)".to_string(),
        }
    }

    /// Formats the metric value without a unit
    pub fn format_value(&self) -> String {
        match self {
            MetricValue::Available(v) => format!("{}", v),
            MetricValue::Unavailable => "N/A".to_string(),
            MetricValue::NotSupported => "N/A (not supported by driver)".to_string(),
        }
    }
}

impl<T> From<Option<T>> for MetricValue<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => MetricValue::Available(v),
            None => MetricValue::Unavailable,
        }
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for MetricValue<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            MetricValue::Available(v) => v.serialize(serializer),
            MetricValue::Unavailable => serializer.serialize_none(),
            MetricValue::NotSupported => serializer.serialize_none(),
        }
    }
}

// TODO: there should be no tests here. Transfer them to gpu_info\src\test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_value_available() {
        let metric = MetricValue::Available(42);
        assert!(metric.is_available());
        assert!(!metric.is_unavailable());
        assert!(!metric.is_not_supported());
        assert_eq!(metric.value(), Some(&42));
    }

    #[test]
    fn test_metric_value_unavailable() {
        let metric: MetricValue<i32> = MetricValue::Unavailable;
        assert!(!metric.is_available());
        assert!(metric.is_unavailable());
        assert!(!metric.is_not_supported());
        assert_eq!(metric.value(), None);
    }

    #[test]
    fn test_metric_value_not_supported() {
        let metric: MetricValue<i32> = MetricValue::NotSupported;
        assert!(!metric.is_available());
        assert!(!metric.is_unavailable());
        assert!(metric.is_not_supported());
        assert_eq!(metric.value(), None);
    }

    #[test]
    fn test_format_with_unit() {
        let temp = MetricValue::Available(65.5);
        assert_eq!(temp.format_with_unit("°C"), "65.5°C");
        let unavailable: MetricValue<f32> = MetricValue::Unavailable;
        assert_eq!(unavailable.format_with_unit("°C"), "N/A");
        let not_supported: MetricValue<f32> = MetricValue::NotSupported;
        assert_eq!(
            not_supported.format_with_unit("°C"),
            "N/A (not supported by driver)"
        );
    }

    #[test]
    fn test_from_option() {
        let some_value = MetricValue::from(Some(42));
        assert!(some_value.is_available());
        assert_eq!(some_value.value(), Some(&42));
        let none_value: MetricValue<i32> = MetricValue::from(None);
        assert!(none_value.is_unavailable());
    }

    #[test]
    fn test_map() {
        let metric = MetricValue::Available(10);
        let doubled = metric.map(|v| v * 2);
        assert_eq!(doubled.value(), Some(&20));
        let unavailable: MetricValue<i32> = MetricValue::Unavailable;
        let mapped = unavailable.map(|v| v * 2);
        assert!(mapped.is_unavailable());
    }

    #[test]
    fn test_default() {
        let metric: MetricValue<i32> = MetricValue::default();
        assert!(metric.is_unavailable());
        assert_eq!(metric, MetricValue::Unavailable);
    }
}
