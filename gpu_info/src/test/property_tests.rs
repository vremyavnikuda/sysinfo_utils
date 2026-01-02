//! Property-based tests for GPU info types.
//!
//! These tests use `proptest` to verify properties hold across many random inputs.
//! Each property test validates specific requirements from the design document.

use proptest::prelude::*;

/// Configuration for property tests - reduced case count for faster execution.
fn proptest_config() -> ProptestConfig {
    ProptestConfig {
        cases: 20,
        ..ProptestConfig::default()
    }
}

use crate::gpu_info::GpuInfo;
use crate::vendor::{IntelGpuType, Vendor};

/// Strategy for generating arbitrary `IntelGpuType` values.
fn arb_intel_gpu_type() -> impl Strategy<Value = IntelGpuType> {
    prop_oneof![
        Just(IntelGpuType::Integrated),
        Just(IntelGpuType::Discrete),
        Just(IntelGpuType::Unknown),
    ]
}

/// Strategy for generating arbitrary `Vendor` values.
///
/// **Validates: Requirements 30.6** - Implement `proptest::Arbitrary` for property-based testing
pub fn arb_vendor() -> impl Strategy<Value = Vendor> {
    prop_oneof![
        Just(Vendor::Nvidia),
        Just(Vendor::Amd),
        arb_intel_gpu_type().prop_map(Vendor::Intel),
        Just(Vendor::Apple),
        Just(Vendor::Unknown),
    ]
}

/// Strategy for generating valid temperature values (0.0 - 1000.0°C).
fn arb_valid_temperature() -> impl Strategy<Value = f32> {
    0.0f32..=1000.0f32
}

/// Strategy for generating invalid temperature values (> 1000.0°C).
fn arb_invalid_temperature() -> impl Strategy<Value = f32> {
    1000.1f32..5000.0f32
}

/// Strategy for generating valid utilization values (0.0 - 100.0%).
fn arb_valid_utilization() -> impl Strategy<Value = f32> {
    0.0f32..=100.0f32
}

/// Strategy for generating invalid utilization values (> 100.0%).
fn arb_invalid_utilization() -> impl Strategy<Value = f32> {
    100.1f32..1000.0f32
}

/// Strategy for generating valid power usage values (0.0 - 1000.0W).
fn arb_valid_power_usage() -> impl Strategy<Value = f32> {
    0.0f32..=1000.0f32
}

/// Strategy for generating invalid power usage values (> 1000.0W).
fn arb_invalid_power_usage() -> impl Strategy<Value = f32> {
    1000.1f32..10000.0f32
}

/// Strategy for generating arbitrary GPU names.
fn arb_gpu_name() -> impl Strategy<Value = Option<String>> {
    prop_oneof![Just(None), "[a-zA-Z0-9 ]{1,50}".prop_map(Some),]
}

/// Strategy for generating arbitrary `GpuInfo` with valid values.
///
/// **Validates: Requirements 30.6** - Implement `proptest::Arbitrary` for property-based testing
pub fn arb_gpu_info_valid() -> impl Strategy<Value = GpuInfo> {
    (
        arb_vendor(),
        arb_gpu_name(),
        prop::option::of(arb_valid_temperature()),
        prop::option::of(arb_valid_utilization()),
        prop::option::of(arb_valid_power_usage()),
    )
        .prop_map(|(vendor, name, temp, util, power)| {
            let mut builder = GpuInfo::builder().vendor(vendor);
            if let Some(n) = name {
                builder = builder.name(n);
            }
            if let Some(t) = temp {
                builder = builder.temperature(t);
            }
            if let Some(u) = util {
                builder = builder.utilization(u);
            }
            if let Some(p) = power {
                builder = builder.power_usage(p);
            }
            builder.build()
        })
}

proptest! {
    #![proptest_config(proptest_config())]

    /// **Validation Round-Trip**
    ///
    /// *For any* `GpuInfo` instance where `validate()` returns `Ok(())`,
    /// calling `is_valid()` SHALL return `true`.
    ///
    /// **Validates: Requirements 14.1, 14.2**
    #[test]
    fn prop_validation_round_trip(gpu in arb_gpu_info_valid()) {
        let validation_result = gpu.validate();
        let is_valid = gpu.is_valid();
        if validation_result.is_ok() {
            prop_assert!(is_valid, "is_valid() should return true when validate() returns Ok");
        }
        prop_assert_eq!(is_valid, validation_result.is_ok());
    }

    /// **Valid Temperature Acceptance**
    ///
    /// *For any* temperature in the valid range (0-1000°C),
    /// validation SHALL succeed.
    ///
    /// **Validates: Requirements 14.1, 14.3**
    #[test]
    fn prop_valid_temperature_accepted(temp in arb_valid_temperature()) {
        let gpu = GpuInfo::builder()
            .temperature(temp)
            .build();
        prop_assert!(
            gpu.validate().is_ok(),
            "Temperature {:.1}°C should be valid (range: 0-1000°C)",
            temp
        );
    }

    /// **Invalid Temperature Rejection**
    ///
    /// *For any* temperature outside the valid range (> 1000°C),
    /// validation SHALL fail with `InvalidTemperature` error.
    ///
    /// **Validates: Requirements 14.1, 14.5**
    #[test]
    fn prop_invalid_temperature_rejected(temp in arb_invalid_temperature()) {
        let gpu = GpuInfo::builder()
            .temperature(temp)
            .build();
        let result = gpu.validate();
        prop_assert!(
            result.is_err(),
            "Temperature {:.1}°C should be invalid (> 1000°C)",
            temp
        );
        if let Err(e) = result {
            let error_str = format!("{}", e);
            prop_assert!(
                error_str.contains("temperature"),
                "Error should mention temperature: {}",
                error_str
            );
        }
    }

    /// **Valid Utilization Acceptance**
    ///
    /// *For any* utilization in the valid range (0-100%),
    /// validation SHALL succeed.
    ///
    /// **Validates: Requirements 14.1, 14.3**
    #[test]
    fn prop_valid_utilization_accepted(util in arb_valid_utilization()) {
        let gpu = GpuInfo::builder()
            .utilization(util)
            .build();
        prop_assert!(
            gpu.validate().is_ok(),
            "Utilization {:.1}% should be valid (range: 0-100%)",
            util
        );
    }

    /// **Invalid Utilization Rejection**
    ///
    /// *For any* utilization outside the valid range (> 100%),
    /// validation SHALL fail with `InvalidUtilization` error.
    ///
    /// **Validates: Requirements 14.1, 14.5**
    #[test]
    fn prop_invalid_utilization_rejected(util in arb_invalid_utilization()) {
        let gpu = GpuInfo::builder()
            .utilization(util)
            .build();
        let result = gpu.validate();
        prop_assert!(
            result.is_err(),
            "Utilization {:.1}% should be invalid (> 100%)",
            util
        );
    }

    /// **Valid Power Usage Acceptance**
    ///
    /// *For any* power usage in the valid range (0-1000W),
    /// validation SHALL succeed.
    ///
    /// **Validates: Requirements 14.1, 14.3**
    #[test]
    fn prop_valid_power_usage_accepted(power in arb_valid_power_usage()) {
        let gpu = GpuInfo::builder()
            .power_usage(power)
            .build();
        prop_assert!(
            gpu.validate().is_ok(),
            "Power usage {:.1}W should be valid (range: 0-1000W)",
            power
        );
    }

    /// **Invalid Power Usage Rejection**
    ///
    /// *For any* power usage outside the valid range (> 1000W),
    /// validation SHALL fail with `InvalidPowerUsage` error.
    ///
    /// **Validates: Requirements 14.1, 14.5**
    #[test]
    fn prop_invalid_power_usage_rejected(power in arb_invalid_power_usage()) {
        let gpu = GpuInfo::builder()
            .power_usage(power)
            .build();
        let result = gpu.validate();
        prop_assert!(
            result.is_err(),
            "Power usage {:.1}W should be invalid (> 1000W)",
            power
        );
    }

    /// **Vendor Preservation**
    ///
    /// *For any* vendor, creating a `GpuInfo` with that vendor
    /// SHALL preserve the vendor value.
    ///
    /// **Validates: Requirements 1.2, 1.4**
    #[test]
    fn prop_vendor_preserved(vendor in arb_vendor()) {
        let gpu = GpuInfo::builder()
            .vendor(vendor)
            .build();
        prop_assert_eq!(gpu.vendor(), vendor, "Vendor should be preserved");
    }

    /// **Default Validity**
    ///
    /// *For any* type implementing `Default`, `Default::default()`
    /// SHALL produce a valid, usable instance that passes validation.
    ///
    /// **Validates: Requirements 25.3**
    #[test]
    fn prop_default_is_valid(_seed in 0u32..1000u32) {
        let gpu = GpuInfo::default();
        prop_assert!(
            gpu.is_valid(),
            "Default GpuInfo should be valid"
        );
    }

    /// **Clone Equivalence**
    ///
    /// *For any* `GpuInfo`, cloning SHALL produce an equivalent instance.
    ///
    /// **Validates: Requirements 27.1, 27.5**
    #[test]
    fn prop_clone_equivalence(gpu in arb_gpu_info_valid()) {
        let cloned = gpu.clone();
        prop_assert_eq!(gpu.vendor(), cloned.vendor());
        prop_assert_eq!(gpu.name_gpu(), cloned.name_gpu());
        prop_assert_eq!(gpu.temperature(), cloned.temperature());
        prop_assert_eq!(gpu.utilization(), cloned.utilization());
        prop_assert_eq!(gpu.power_usage(), cloned.power_usage());
    }

    /// **Hash Consistency**
    ///
    /// *For any* two `GpuInfo` instances that are equal,
    /// their hash values SHALL be equal.
    ///
    /// **Validates: Requirements 24.3, 24.4**
    #[test]
    fn prop_hash_consistency(vendor in arb_vendor(), name in arb_gpu_name()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let gpu1 = {
            let mut builder = GpuInfo::builder().vendor(vendor);
            if let Some(ref n) = name {
                builder = builder.name(n.clone());
            }
            builder.build()
        };
        let gpu2 = {
            let mut builder = GpuInfo::builder().vendor(vendor);
            if let Some(ref n) = name {
                builder = builder.name(n.clone());
            }
            builder.build()
        };
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        gpu1.hash(&mut hasher1);
        gpu2.hash(&mut hasher2);
        prop_assert_eq!(
            hasher1.finish(),
            hasher2.finish(),
            "GPUs with same vendor and name should have same hash"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the arbitrary strategies compile and produce values.
    #[test]
    fn test_strategies_compile() {
        let _ = arb_vendor();
        let _ = arb_gpu_info_valid();
    }
}
