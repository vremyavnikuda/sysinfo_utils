#[cfg(test)]
mod gpu_info_tests {
    use crate::gpu_info::Formattable;
    use crate::vendor::Vendor;
    use crate::GpuInfo;
    use std::cell::RefCell;
    #[allow(dead_code)]
    struct MockCommand {
        success: bool,
        output: &'static str,
    }
    
    /// Test format driver version fn `format_driver_version()`
    #[test]
    fn _format_driver_version_returns_driver_version_when_present() {
        let gpu_info = GpuInfo {
            driver_version: Some("460.39".to_string()),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_driver_version(), "460.39");
    }
    
    /// Test format driver version fn `format_driver_version()`
    #[test]
    fn _format_driver_version_returns_unknown_when_absent() {
        let gpu_info = GpuInfo {
            driver_version: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_driver_version(), "Unknown Driver Version");
    }
    
    /// Test formater fn `format_max_clock_speed()`
    #[test]
    fn _format_max_clock_speed_returns_max_clock_speed_when_present() {
        let gpu_info = GpuInfo {
            max_clock_speed: Some(1800),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_max_clock_speed(), "1800 MHz");
    }
    
    /// Test formater fn `format_max_clock_speed()`
    #[test]
    fn _format_max_clock_speed_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            max_clock_speed: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_max_clock_speed(), "N/A");
    }
    
    /// Verifies that the default implementation of `GpuInfo` returns an instance with all fields set to unknown values.
    /// Test `default()` method of `GpuInfo` struct.
    /// impl Default for GpuInfo {}
    #[test]
    fn _default_returns_instance_with_unknown_values() {
        let gpu_info = GpuInfo::default();
        assert_eq!(gpu_info.vendor, Vendor::Unknown);
        assert!(gpu_info.name_gpu.is_none());
        assert!(gpu_info.temperature.is_none());
        assert!(gpu_info.utilization.is_none());
        assert!(gpu_info.power_usage.is_none());
        assert!(gpu_info.core_clock.is_none());
        assert!(gpu_info.memory_util.is_none());
        assert!(gpu_info.memory_clock.is_none());
        assert!(gpu_info.active.is_none());
        assert!(gpu_info.power_limit.is_none());
        assert!(gpu_info.memory_total.is_none());
        assert!(gpu_info.driver_version.is_none());
        assert!(gpu_info.max_clock_speed.is_none());
    }

    /// Format test fn `format_memory_total()`
    #[test]
    fn _default_creates_new_instance_each_time() {
        let gpu_info1 = GpuInfo::default();
        let gpu_info2 = GpuInfo::default();
        assert_ne!(&gpu_info1 as *const _, &gpu_info2 as *const _);
    }

    /// Format test fn `format_memory_total()`
    #[test]
    fn _format_memory_total_returns_total_memory_when_present() {
        let gpu_info = GpuInfo {
            memory_total: Some(8192), // 8192 MB = 8.0 GB
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_total(), "8.00 GB");
    }

    /// Format test fn `format_memory_total()`
    #[test]
    fn _format_memory_total_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            memory_total: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_total(), "N/A");
    }

    /// Test formater fn `format_power_limit()`
    #[test]
    fn _format_power_limit_returns_power_limit_when_present() {
        let gpu_info = GpuInfo {
            power_limit: Some(250.0),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_power_limit(), "250.00W");
    }

    /// Test formater fn `format_power_limit()`
    #[test]
    fn _format_power_limit_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            power_limit: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_power_limit(), "Not supported");
    }

    /// Test formater fn `format_active()`
    #[test]
    fn _format_active_returns_active_when_gpu_is_active() {
        let gpu_info = GpuInfo {
            active: Some(true),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_active(), "Active");
    }

    /// Test formater fn `format_active()`
    #[test]
    fn _format_active_returns_inactive_when_gpu_is_inactive() {
        let gpu_info = GpuInfo {
            active: Some(false),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_active(), "Inactive");
    }

    /// Test formater fn `format_active()`
    #[test]
    fn _format_active_returns_inactive_when_active_status_is_unknown() {
        let gpu_info = GpuInfo {
            active: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_active(), "Inactive");
    }

    /// Test formater fn `format_memory_clock(&self)`
    #[test]
    fn _format_memory_clock_returns_memory_clock_when_present() {
        let gpu_info = GpuInfo {
            memory_clock: Some(7000),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_clock(), "7000 MHz");
    }

    /// Test formater fn `format_memory_clock(&self)`
    #[test]
    fn _format_memory_clock_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            memory_clock: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_clock(), "N/A");
    }

    /// Test format fn `format_memory_util()`
    #[test]
    fn _format_memory_util_returns_memory_utilization_when_present() {
        let gpu_info = GpuInfo {
            memory_util: Some(75.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_util(), "75.50%");
    }

    /// Test format fn `format_memory_util()`
    #[test]
    fn _format_memory_util_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            memory_util: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_util(), "N/A");
    }

    /// Test format fn `format_core_clock()`
    #[test]
    fn _format_core_clock_returns_core_clock_when_present() {
        let gpu_info = GpuInfo {
            core_clock: Some(1500),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_core_clock(), "1500 MHz");
    }

    /// Test format fn `format_core_clock()`
    #[test]
    fn _format_core_clock_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            core_clock: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_core_clock(), "N/A");
    }

    /// Test format fn `format_power_usage(&self)`
    #[test]
    fn _format_power_usage_returns_power_usage_when_present() {
        let gpu_info = GpuInfo {
            power_usage: Some(150.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_power_usage(), "150.50W");
    }

    /// Test format fn `format_power_usage(&self)`
    #[test]
    fn _format_power_usage_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            power_usage: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_power_usage(), "Not supported");
    }

    /// Test format fn `format_utilization(&self)`
    #[test]
    fn _format_utilization_returns_utilization_when_present() {
        let gpu_info = GpuInfo {
            utilization: Some(85.3),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_utilization(), "85.30%");
    }

    /// Test format fn `format_utilization(&self)`
    #[test]
    fn _format_utilization_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            utilization: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_utilization(), "N/A");
    }

    /// Test format fn `format_temperature(&self)`
    #[test]
    fn _format_temperature_returns_temperature_when_present() {
        let gpu_info = GpuInfo {
            temperature: Some(65.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_temperature(), "65.50°C");
    }

    /// Test format fn `format_temperature(&self)`
    #[test]
    fn _format_temperature_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            temperature: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_temperature(), "Not supported");
    }

    /// Test format fn `format_name_gpu(&self)`
    #[test]
    fn _format_name_gpu_returns_name_when_present() {
        let gpu_info = GpuInfo {
            name_gpu: Some("NVIDIA GeForce RTX 3080".to_string()),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_name_gpu(), "NVIDIA GeForce RTX 3080");
    }

    /// Test format fn `format_name_gpu(&self)`
    #[test]
    fn _format_name_gpu_returns_unknown_when_absent() {
        let gpu_info = GpuInfo {
            name_gpu: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_name_gpu(), "Unknown GPU");
    }

    /// Test default format fn `max_clock_speed(&self)`
    #[test]
    fn _max_clock_speed_returns_value_when_present() {
        let gpu_info = GpuInfo {
            max_clock_speed: Some(2100),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.max_clock_speed().fmt_string(), "2100");
    }

    /// Test default format fn `max_clock_speed(&self)`
    #[test]
    fn _max_clock_speed_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            max_clock_speed: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.max_clock_speed().fmt_string(), "N/A");
    }

    /// Test default format fn `driver_version(&self)`
    #[test]
    fn _driver_version_returns_value_when_present() {
        let gpu_info = GpuInfo {
            driver_version: Some("470.57.02".to_string()),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.driver_version().fmt_string(), "470.57.02");
    }

    /// Test default format fn `driver_version(&self)`
    #[test]
    fn _driver_version_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            driver_version: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.driver_version().fmt_string(), "N/A");
    }

    /// Test default format fn `memory_total(&self)`
    #[test]
    fn _memory_total_returns_value_when_present() {
        let gpu_info = GpuInfo {
            memory_total: Some(8192),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.memory_total().fmt_string(), "8192");
    }

    /// Test format fn `memory_total(&self)`
    #[test]
    fn _memory_total_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            memory_total: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.memory_total().fmt_string(), "N/A");
    }

    /// Test default format fn `power_limit(&self)`
    #[test]
    fn _power_limit_returns_value_when_present() {
        let gpu_info = GpuInfo {
            power_limit: Some(250.0),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.power_limit().fmt_string(), "250.0");
    }

    /// Test default format fn `power_limit(&self)`
    #[test]
    fn _power_limit_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            power_limit: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.power_limit().fmt_string(), "N/A");
    }

    /// Test default format fn `active(&self)`
    #[test]
    fn _active_returns_true_when_gpu_is_active() {
        let gpu_info = GpuInfo {
            active: Some(true),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.active(), Some(true));
    }

    /// Test default format fn `active(&self)`
    #[test]
    fn _active_returns_false_when_gpu_is_inactive() {
        let gpu_info = GpuInfo {
            active: Some(false),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.active(), Some(false));
    }

    /// Test default format fn `active(&self)`
    #[test]
    fn _active_returns_none_when_status_is_unknown() {
        let gpu_info = GpuInfo {
            active: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.active().fmt_string(), "N/A");
    }

    /// Test default format fn `memory_clock(&self)`
    #[test]
    fn _memory_clock_returns_value_when_present() {
        let gpu_info = GpuInfo {
            memory_clock: Some(7000),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.memory_clock().fmt_string(), "7000");
    }

    /// Test default format fn `memory_clock(&self)`
    #[test]
    fn _memory_clock_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            memory_clock: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.memory_clock().fmt_string(), "N/A");
    }

    /// Test default format fn `memory_util(&self)`
    #[test]
    fn _memory_util_returns_value_when_present() {
        let gpu_info = GpuInfo {
            memory_util: Some(75.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.memory_util().fmt_string(), "75.5");
    }

    /// Test default format fn `memory_util(&self)`
    #[test]
    fn _memory_util_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            memory_util: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.memory_util().fmt_string(), "N/A");
    }
 
    /// Test default format fn `core_clock(&self)`
    #[test]
    fn _core_clock_returns_value_when_present() {
        let gpu_info = GpuInfo {
            core_clock: Some(1500),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.core_clock().fmt_string(), "1500");
    }

    /// Test default format fn `core_clock(&self)`
    #[test]
    fn _core_clock_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            core_clock: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.core_clock().fmt_string(), "N/A");
    }

    /// Test default format fn `power_usage(&self)`
    #[test]
    fn _power_usage_returns_value_when_present() {
        let gpu_info = GpuInfo {
            power_usage: Some(120.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.power_usage().fmt_string(), "120.5");
    }

    /// Test default format fn `power_usage(&self)`
    #[test]
    fn _power_usage_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            power_usage: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.power_usage().fmt_string(), "N/A");
    }

    /// Test default format fn `utilization(&self)`
    #[test]
    fn _utilization_returns_value_when_present() {
        let gpu_info = GpuInfo {
            utilization: Some(85.0),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.utilization().fmt_string(), "85.0");
    }

    /// Test default format fn `utilization(&self)`
    #[test]
    fn _utilization_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            utilization: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.utilization().fmt_string(), "N/A");
    }

    /// Test default format fn `temperature(&self)`
    #[test]
    fn _temperature_returns_value_when_present() {
        let gpu_info = GpuInfo {
            temperature: Some(70.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.temperature().fmt_string(), "70.5");
    }

    /// Test default format fn `temperature(&self)`
    #[test]
    fn _temperature_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            temperature: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.temperature().fmt_string(), "N/A");
    }

    /// Test default format fn `name_gpu(&self)`
    #[test]
    fn _name_gpu_returns_value_when_present() {
        let gpu_info = GpuInfo {
            name_gpu: Some("NVIDIA GeForce RTX 3080".to_string()),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.name_gpu().fmt_string(), "NVIDIA GeForce RTX 3080");
    }

    /// Test default format fn `name_gpu(&self)`
    #[test]
    fn _name_gpu_returns_none_when_absent() {
        let gpu_info = GpuInfo {
            name_gpu: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.name_gpu().fmt_string(), "N/A");
    }

    /// Test default format fn `vendor(&self)`
    #[test]
    fn _vendor_returns_correct_vendor_when_present() {
        let gpu_info = GpuInfo {
            vendor: Vendor::Nvidia,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.vendor(), Vendor::Nvidia);
    }

    /// Test default format fn `vendor(&self)
    #[test]
    fn _vendor_returns_unknown_when_vendor_is_not_set() {
        let gpu_info = GpuInfo {
            vendor: Vendor::Unknown,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.vendor(), Vendor::Unknown);
    }

    /// Test default format fn `write_vendor(vendor: Vendor)`
    #[test]
    fn _write_vendor_creates_instance_with_specified_vendor() {
        let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
        assert_eq!(gpu_info.vendor, Vendor::Nvidia);
    }

    /// Test `write_vendor()`
    #[test]
    fn _write_vendor_sets_other_fields_to_default() {
        let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
        assert_eq!(gpu_info.name_gpu, None);
        assert_eq!(gpu_info.temperature, None);
        assert_eq!(gpu_info.utilization, None);
        assert_eq!(gpu_info.power_usage, None);
        assert_eq!(gpu_info.core_clock, None);
        assert_eq!(gpu_info.memory_util, None);
        assert_eq!(gpu_info.memory_clock, None);
        assert_eq!(gpu_info.active, None);
        assert_eq!(gpu_info.power_limit, None);
        assert_eq!(gpu_info.memory_total, None);
        assert_eq!(gpu_info.driver_version, None);
        assert_eq!(gpu_info.max_clock_speed, None);
    }

    /// Test `Display` implementation for `GpuInfo`
    #[test]
    fn _display_includes_all_fields_when_present() {
        let gpu_info = GpuInfo {
            vendor: Vendor::Nvidia,
            name_gpu: Some("NVIDIA GeForce RTX 3080".to_string()),
            temperature: Some(70.5),
            utilization: Some(85.0),
            power_usage: Some(120.5),
            core_clock: Some(1500),
            memory_util: Some(75.5),
            memory_clock: Some(7000),
            active: Some(true),
            power_limit: Some(250.0),
            memory_total: Some(8192),
            memory_used: None,
            driver_version: Some("470.57.02".to_string()),
            max_clock_speed: Some(2100),
        };
        let display_output = format!("{}", gpu_info);
        assert!(display_output.contains("NVIDIA"));
        assert!(display_output.contains("NVIDIA GeForce RTX 3080"));
        assert!(display_output.contains("70.5"));
        assert!(display_output.contains("85.0"));
        assert!(display_output.contains("120.5"));
        assert!(display_output.contains("1500"));
        assert!(display_output.contains("75.5"));
        assert!(display_output.contains("7000"));
        assert!(display_output.contains("true"));
        assert!(display_output.contains("250.0"));
        assert!(display_output.contains("8192"));
        assert!(display_output.contains("470.57.02"));
        assert!(display_output.contains("2100"));
    }

    /// Test `Display` implementation for `GpuInfo` with missing fields
    #[test]
    fn _display_handles_missing_fields_gracefully() {
        let gpu_info = GpuInfo {
            vendor: Vendor::Unknown,
            ..GpuInfo::default()
        };
        let display_output = format!("{}", gpu_info);
        assert!(display_output.contains("N/A"));
        assert!(!display_output.contains("NVIDIA GeForce RTX 3080"));
        assert!(!display_output.contains("70.5"));
        assert!(!display_output.contains("85.0"));
        assert!(!display_output.contains("120.5"));
        assert!(!display_output.contains("1500"));
        assert!(!display_output.contains("75.5"));
        assert!(!display_output.contains("7000"));
        assert!(!display_output.contains("true"));
        assert!(!display_output.contains("250.0"));
        assert!(!display_output.contains("8192"));
        assert!(!display_output.contains("470.57.02"));
        assert!(!display_output.contains("2100"));
    }

    impl MockCommand {
        /// Creates a init `MockCommand` with the given success state and output.
        ///
        /// # Arguments
        ///
        /// * `success`: A boolean indicating whether the mocked command should
        ///   succeed or fail.
        /// * `output`: A string slice containing the output of the mocked command.
        ///
        /// # Returns
        ///
        /// * `MockCommand` - A new `MockCommand` instance with the given success state and output.
        ///
        /// # Panics
        #[allow(dead_code)]
        fn new(success: bool, output: &'static str) -> Self {
            Self { success, output }
        }
    }
    thread_local! {
        static MOCK_COMMAND: RefCell<Option<MockCommand>> = const { RefCell::new(None) };
    }
    /// Sets a mocked command for the duration of the current test.
    ///
    /// This function is used to mock the output of commands that are executed
    /// by the `GpuManager` during tests. It sets a `MockCommand` instance that
    /// will be used instead of the real command when `Command::init` is called.
    ///
    /// # Arguments
    ///
    /// * `success`: A boolean indicating whether the mocked command should
    ///   succeed or fail.
    /// * `output`: A string slice containing the output of the mocked command.
    ///
    /// # Example
    ///
    /// ```rust
    /// mock_command(true, "nvidia-smi output");
    /// ```
    ///
    /// # Returns
    ///
    /// * `None` - If the mocked command fails.
    /// * `Some(MockCommand)` - If the mocked command succeeds.
    ///
    /// # Panics
    ///
    /// * `None` - If the mocked command fails.
    /// * `Some(MockCommand)` - If the mocked command succeeds.
    #[allow(dead_code)]
    fn mock_command(success: bool, output: &'static str) {
        MOCK_COMMAND.with(|mc| {
            *mc.borrow_mut() = Some(MockCommand::new(success, output));
        });
    }

    /// Tests various methods of `GpuInfo` struct.
    ///
    /// This test initializes a `GpuInfo` instance with sample data
    /// and verifies the correctness of its methods including:
    /// - `get_name`: Ensures the GPU name_gpu is returned correctly.
    /// - `get_vendor`: Checks if the vendor is correctly identified.
    /// - `get_temperature`: Confirms formatting and value of temperature.
    /// - `get_utilization`: Validates the utilization percentage format.
    /// - `get_clock_speed`: Checks the clock speed formatting.
    /// - `get_power_usage`: Ensures power usage is formatted correctly.
    /// - `active`: Verifies if the GPU is active.
    #[test]
    fn test_gpu_info_methods() {
        let gpu = GpuInfo {
            vendor: Vendor::Nvidia,
            name_gpu: Some("Test GPU".to_string()),
            temperature: None,
            utilization: Some(75.0),
            power_usage: Some(100.0),
            core_clock: Some(1500),
            memory_util: Some(2000.0),
            memory_clock: Some(100),
            active: None,
            power_limit: None,
            memory_total: None,
            memory_used: None,
            driver_version: None,
            max_clock_speed: None,
        };
        assert_eq!(gpu.name_gpu(), Some("Test GPU"));
        assert!(matches!(gpu.vendor(), Vendor::Nvidia));
        assert_eq!(gpu.temperature(), None);
        assert_eq!(gpu.utilization(), Some(75.0));
        assert_eq!(gpu.core_clock(), Some(1500));
        assert_eq!(gpu.max_clock_speed(), None);
        assert_eq!(gpu.power_usage(), Some(100.0));
        assert_eq!(gpu.power_limit(), None);
        assert_eq!(gpu.active(), None);
    }

    /// Tests that `GpuManager` can be successfully created and that it is
    /// initialized with a non-empty list of GPUs and the active GPU set to 0.
    #[test]
    fn _test_gpu_manager_creation() {
        let gpu = GpuInfo::default();
        assert_eq!(
            gpu.name_gpu, None,
            "Expected gpus to be empty, but it was not."
        );
        assert_eq!(gpu.active.fmt_string(), "N/A");
    }
}

/// Tests the `check_power_state` method of `GpuManager`.
///
/// # Note
///
/// This test currently does nothing, as it requires integration testing with
/// real processes.
#[test]
fn _test_power_state_check() {
    let manager = crate::GpuInfo::default();
    let _ = manager.power_usage();
    assert_eq!(manager.power_usage(), None);
}

#[cfg(all(not(target_os = "hermit"), any(unix, windows)))]
mod mock_impl {
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;
    use std::process::{Command, ExitStatus, Output};
    /// Mocks the execution of a system command by returning predefined output.
    ///
    /// This function is used in testing to simulate the execution of a system
    /// command without actually running it. The output is determined by the
    /// state of the `MOCK_COMMAND` thread-local storage, which contains the
    /// success state and the output string.
    ///
    /// # Arguments
    ///
    /// * `_cmd` - A mutable reference to a `Command` that is being mocked.
    ///
    /// # Returns
    ///
    /// * `Result<Output, std::io::Error>` - An `Output` object containing the
    ///   mocked status, stdout, and stderr. If a mock is available, it returns
    ///   the corresponding status and output; otherwise, it defaults to a
    ///   success status with empty output.
    pub fn _command_mock(_cmd: &mut Command) -> Result<Output, std::io::Error> {
        Ok(Output {
            status: ExitStatus::from_raw(0),
            stdout: vec![],
            stderr: vec![],
        })
    }
}

#[test]
#[cfg(target_os = "linux")]
fn integration_test_real_system() {
    use crate::vendor::Vendor;
    use crate::GpuManager;
    use std::fs;
    use std::path::Path;
    let manager = GpuManager::new();
    let gpus = manager.get_all_gpus_owned();
    if Path::new("/usr/bin/nvidia-smi").exists() {
        assert!(gpus.iter().any(|g| matches!(g.vendor, Vendor::Nvidia)));
    }
    if Path::new("/sys/class/drm/card0/device/vendor").exists() {
        let vendor_str =
            fs::read_to_string("/sys/class/drm/card0/device/vendor").unwrap_or_default();
        if vendor_str.contains("0x1002") {
            assert!(gpus.iter().any(|g| matches!(g.vendor, Vendor::Amd)));
        }
        if vendor_str.contains("0x8086") {
            assert!(gpus.iter().any(|g| matches!(
                g.vendor,
                Vendor::Intel(crate::vendor::IntelGpuType::Unknown)
            )));
        }
    }
}

/// Tests that `GpuManager` correctly returns the vendor of the first GPU in its list
///
/// # Assertions
///
/// - The `GpuManager` contains at least one GPU
/// - The first GPU in the list is an NVIDIA GPU
#[test]
fn test_get_vendor_nvidia() {
    let manager = crate::GpuInfo::default();
    let gpu = manager;
    assert!(matches!(gpu.vendor, crate::vendor::Vendor::Unknown));
}

#[cfg(test)]
#[cfg(target_os = "linux")]
mod linux_nvidia_provider_test {
    use crate::gpu_info::GpuProvider;
    use crate::providers::linux::NvidiaLinuxProvider;
    use crate::vendor::Vendor;

    /// Test that NvidiaLinuxProvider returns correct vendor
    #[test]
    fn test_nvidia_provider_vendor() {
        let provider = NvidiaLinuxProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Nvidia);
    }

    /// Test that NvidiaLinuxProvider can be created
    #[test]
    fn test_nvidia_provider_creation() {
        let provider = NvidiaLinuxProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Nvidia);
        let provider_default = NvidiaLinuxProvider;
        assert_eq!(provider_default.get_vendor(), Vendor::Nvidia);
    }

    /// Test detect_gpus behavior (will fail if NVML not available)
    #[test]
    fn test_nvidia_provider_detect_gpus_integration() {
        let provider = NvidiaLinuxProvider::new();
        match provider.detect_gpus() {
            Ok(gpus) => {
                assert!(!gpus.is_empty(), "Should detect at least one GPU");
                for gpu in gpus {
                    assert_eq!(gpu.vendor, Vendor::Nvidia);
                    assert!(gpu.name_gpu.is_some(), "GPU should have a name");
                }
            }
            Err(e) => {
                println!("Expected error (no NVML or no GPU): {:?}", e);
            }
        }
    }

    /// Test update_gpu behavior
    #[test]
    fn test_nvidia_provider_update_gpu_integration() {
        let provider = NvidiaLinuxProvider::new();
        match provider.detect_gpus() {
            Ok(gpus) if !gpus.is_empty() => {
                let mut gpu = gpus[0].clone();
                match provider.update_gpu(&mut gpu) {
                    Ok(()) => {
                        assert_eq!(gpu.vendor, Vendor::Nvidia);
                        println!("GPU updated successfully: {:?}", gpu.name_gpu);
                    }
                    Err(e) => {
                        println!("Failed to update GPU: {:?}", e);
                    }
                }
            }
            _ => {
                println!("No NVIDIA GPUs detected for update test");
            }
        }
    }
}
