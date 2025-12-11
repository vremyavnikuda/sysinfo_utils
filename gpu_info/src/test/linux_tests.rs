//! Linux-specific GPU detection tests.
//!
//! These tests verify that the Linux GPU detection functions work correctly:
//! - `detect_vendor()`: Vendor detection via sysfs
//! - `info_gpu()`: GPU information collection
//!
//! Note: These tests run on actual hardware and may return Unknown
//! if no GPU is present. For deterministic testing with mock data,
//! see the test_data/linux/ directory.

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use crate::{linux::*, vendor::Vendor};

    #[test]
    fn test_detect_vendor_returns_valid_vendor() {
        let vendor = detect_vendor();

        // Verify it's a valid vendor variant
        assert!(
            matches!(
                vendor,
                Vendor::Nvidia | Vendor::Amd | Vendor::Intel(_) | Vendor::Unknown
            ),
            "Expected valid vendor, got: {:?}",
            vendor
        );

        // Verify Display trait works
        let vendor_str = vendor.to_string();
        assert!(!vendor_str.is_empty(), "Vendor string should not be empty");
    }

    #[test]
    fn test_info_gpu_returns_valid_gpuinfo() {
        let gpu = info_gpu();

        // Verify vendor is valid
        assert!(
            matches!(
                gpu.vendor,
                Vendor::Nvidia | Vendor::Amd | Vendor::Intel(_) | Vendor::Unknown
            ),
            "Expected valid vendor, got: {:?}",
            gpu.vendor
        );

        // Verify temperature is physically possible if present
        if let Some(temp) = gpu.temperature {
            assert!(
                (-273.15..=1000.0).contains(&temp),
                "Temperature should be physically possible: {}°C",
                temp
            );
        }

        // Verify utilization is in valid range if present
        if let Some(util) = gpu.utilization {
            assert!(
                (0.0..=100.0).contains(&util),
                "Utilization should be 0-100%: {}%",
                util
            );
        }

        // Verify memory utilization is in valid range if present
        if let Some(mem_util) = gpu.memory_util {
            assert!(
                (0.0..=100.0).contains(&mem_util),
                "Memory utilization should be 0-100%: {}%",
                mem_util
            );
        }
    }

    #[test]
    fn test_info_gpu_handles_missing_hardware_gracefully() {
        // This test verifies that info_gpu() doesn't panic even without GPU hardware
        let gpu = info_gpu();

        // Even without a GPU, we should get Unknown vendor with minimal data
        if matches!(gpu.vendor, Vendor::Unknown) {
            assert!(
                gpu.name_gpu.is_none() || gpu.name_gpu.as_ref().map_or(true, |s| s.is_empty()),
                "Unknown GPU should have no name or empty name"
            );
        }
    }

    #[test]
    fn test_info_gpu_validates_correctly() {
        let gpu = info_gpu();

        // Use the validate() method to ensure returned data passes internal validation
        match gpu.validate() {
            Ok(_) => {
                // GPU data is valid - test passes
            }
            Err(e) => {
                panic!("GPU validation failed: {:?}", e);
            }
        }
    }

    // TODO: Add tests using mock data from test_data/linux/
    // This requires refactoring info_gpu() to accept a custom sysfs path parameter
    // for dependency injection during testing.
    //
    // Planned tests:
    // - test_nvidia_rtx_3080_mock_data()
    // - test_amd_rx_6800_mock_data()
    // - test_intel_uhd_630_mock_data()
}
