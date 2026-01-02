//! Tests for MetricValue type

use crate::metric_value::MetricValue;

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
fn test_format_value() {
    let metric = MetricValue::Available(42);
    assert_eq!(metric.format_value(), "42");
    let unavailable: MetricValue<i32> = MetricValue::Unavailable;
    assert_eq!(unavailable.format_value(), "N/A");
    let not_supported: MetricValue<i32> = MetricValue::NotSupported;
    assert_eq!(
        not_supported.format_value(),
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
fn test_into_option() {
    let available = MetricValue::Available(42);
    assert_eq!(available.into_option(), Some(42));
    let unavailable: MetricValue<i32> = MetricValue::Unavailable;
    assert_eq!(unavailable.into_option(), None);
    let not_supported: MetricValue<i32> = MetricValue::NotSupported;
    assert_eq!(not_supported.into_option(), None);
}

#[test]
fn test_map() {
    let metric = MetricValue::Available(10);
    let doubled = metric.map(|v| v * 2);
    assert_eq!(doubled.value(), Some(&20));
    let unavailable: MetricValue<i32> = MetricValue::Unavailable;
    let mapped = unavailable.map(|v| v * 2);
    assert!(mapped.is_unavailable());
    let not_supported: MetricValue<i32> = MetricValue::NotSupported;
    let mapped = not_supported.map(|v| v * 2);
    assert!(mapped.is_not_supported());
}

#[test]
fn test_default() {
    let metric: MetricValue<i32> = MetricValue::default();
    assert!(metric.is_unavailable());
}

#[test]
fn test_clone() {
    let original = MetricValue::Available(String::from("test"));
    let cloned = original.clone();
    assert_eq!(original, cloned);
    assert_eq!(cloned.value(), Some(&String::from("test")));
}

#[test]
fn test_copy() {
    let original = MetricValue::Available(42);
    let copied = original;
    assert_eq!(original, copied);
    assert_eq!(copied.value(), Some(&42));
}
