//src/test/test.rs
#[cfg(test)]
mod gpu_info_tests {
    use crate::{gpu_info::Formattable, vendor, GpuInfo};
    use std::cell::RefCell;
    use vendor::Vendor;

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
        assert_eq!(gpu_info.format_max_clock_speed(), 1800);
    }

    /// Test formater fn `format_max_clock_speed()`
    #[test]
    fn _format_max_clock_speed_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            max_clock_speed: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_max_clock_speed(), 0);
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
            memory_total: Some(8192),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_total(), 8192);
    }

    /// Format test fn `format_memory_total()`
    #[test]
    fn _format_memory_total_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            memory_total: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_total(), 0);
    }

    /// Test formater fn `format_power_limit()`
    #[test]
    fn _format_power_limit_returns_power_limit_when_present() {
        let gpu_info = GpuInfo {
            power_limit: Some(250.0),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_power_limit(), 250.0);
    }

    /// Test formater fn `format_power_limit()`
    #[test]
    fn _format_power_limit_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            power_limit: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_power_limit(), 0.0);
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
        assert_eq!(gpu_info.format_memory_clock(), 7000);
    }

    /// Test formater fn `format_memory_clock(&self)`
    #[test]
    fn _format_memory_clock_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            memory_clock: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_clock(), 0);
    }

    /// Test format fn `format_memory_util()`
    #[test]
    fn _format_memory_util_returns_memory_utilization_when_present() {
        let gpu_info = GpuInfo {
            memory_util: Some(75.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_util(), 75.5);
    }

    /// Test format fn `format_memory_util()`
    #[test]
    fn _format_memory_util_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            memory_util: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_memory_util(), 0.0);
    }

    /// Test format fn `format_core_clock()`
    #[test]
    fn _format_core_clock_returns_core_clock_when_present() {
        let gpu_info = GpuInfo {
            core_clock: Some(1500),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_core_clock(), 1500);
    }

    /// Test format fn `format_core_clock()`
    #[test]
    fn _format_core_clock_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            core_clock: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_core_clock(), 0);
    }

    /// Test format fn `format_power_usage(&self)`
    #[test]
    fn _format_power_usage_returns_power_usage_when_present() {
        let gpu_info = GpuInfo {
            power_usage: Some(150.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_power_usage(), 150.5);
    }

    /// Test format fn `format_power_usage(&self)`
    #[test]
    fn _format_power_usage_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            power_usage: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_power_usage(), 0.0);
    }

    /// Test format fn `format_utilization(&self)`
    #[test]
    fn _format_utilization_returns_utilization_when_present() {
        let gpu_info = GpuInfo {
            utilization: Some(85.3),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_utilization(), 85.3);
    }

    /// Test format fn `format_utilization(&self)`
    #[test]
    fn _format_utilization_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            utilization: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_utilization(), 0.0);
    }

    /// Test format fn `format_temperature(&self)`
    #[test]
    fn _format_temperature_returns_temperature_when_present() {
        let gpu_info = GpuInfo {
            temperature: Some(65.5),
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_temperature(), 65.5);
    }

    /// Test format fn `format_temperature(&self)`
    #[test]
    fn _format_temperature_returns_zero_when_absent() {
        let gpu_info = GpuInfo {
            temperature: None,
            ..GpuInfo::default()
        };
        assert_eq!(gpu_info.format_temperature(), 0.0);
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
            driver_version: Some("470.57.02".to_string()),
            max_clock_speed: Some(2100),
            ..GpuInfo::default()
        };

        let display_output = format!("{}", gpu_info);
        assert!(display_output.contains("Nvidia"));
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
        assert!(display_output.contains("Unknown"));
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
        fn new(success: bool, output: &'static str) -> Self {
            Self { success, output }
        }
    }

    thread_local! {
        static MOCK_COMMAND: RefCell<Option<MockCommand>> = RefCell::new(None);
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
    ///
    ///
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
            power_usage: Some(50.0),
            core_clock: Some(1500),
            memory_util: Some(2000.0),
            memory_clock: Some(100),
            active: None,
            power_limit: None,
            memory_total: None,
            driver_version: None,
            max_clock_speed: None,
        };

        assert_eq!(gpu.name_gpu(), Some("Test GPU"));
        assert!(matches!(gpu.vendor(), Vendor::Nvidia));
        assert_eq!(gpu.temperature(), Some(75.0));
        assert_eq!(gpu.utilization(), Some(50.0));
        assert_eq!(gpu.core_clock(), Some(1500));
        assert_eq!(gpu.max_clock_speed(), Some(2000));
        assert_eq!(gpu.power_usage(), Some(100.0));
        assert_eq!(gpu.power_limit(), Some(150.0));
        assert_eq!(gpu.active().unwrap(), true);
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

    // TODO: ПОКА ЧТО ДАННЫЙ ТЕСТ НЕ НУЖЕН ТАК КАК МЫ БОЛЬШЕ НЕ ПАРСИМ ИНФОРМАЦИЮ ,А ПОЛУЧАЕМ КОНКРЕТНЫЕ ЗНАЧЕНИЯ ПО ОТДЕЛЬНОСТИ . НО В БУДУЩЕМ ВОЗМОЖНО ПРИГОДИТСЯ
    /*#[test]
    fn _test_nvidia_parsing() {
        mock_command(true, "NVIDIA GPU,75,50,1500,2000,100,150\n");

        let gpu = GpuInfo::default();
        gpu.detect_gpus();

        assert!(!manager.gpu.is_empty());
        let gpu = &manager.gpu[0];
        assert!(
            gpu.name.starts_with("NVIDIA")
                || gpu.name.starts_with("AMD")
                || gpu.name.starts_with("INTEL")
        );
        assert!(matches!(gpu.vendor, Vendor::Nvidia));
    }*/

    /*/// Tests the functionality of switching active GPUs in `GpuManager`.
    ///
    /// This test initializes a `GpuManager` with two GPUs and verifies:
    /// - Switching to a valid GPU index updates the active GPU correctly.
    /// - Attempting to switch to an out-of-bounds index returns an error.
    ///
    /// # Assertions
    ///
    /// - Successfully switches active GPU from index 0 to 1.
    /// - Fails to switch active GPU to index 2, as it is out of bounds.
    #[test]
    fn _test_gpu_switching() {
        let mut manager = GpuInfo {
            vendor: Default::default(),
            name_gpu: None,
            temperature: None,
            utilization: None,
            power_usage: None,
            core_clock: None,
            memory_util: None,
            : vec![
                GpuInfo {
                    name_gpu: Some("GPU1".to_string()),
                    vendor: Vendor::Nvidia,
                    ..Default::default()
                },
                GpuInfo {
                    name_gpu: Some("GPU2".to_string()),
                    vendor: Vendor::Amd,
                    ..Default::default()
                },
            ],
            active: 0,
            power_limit: None,
            memory_total: None,
            driver_version: None,
            memory_clock: None,
            max_clock_speed: None,
        };

    assert!(manager.switch_gpu(1).is_ok());
    assert_eq!(manager.active_gpu, 1);
    assert!(manager.switch_gpu(2).is_err());*/
}

// TODO: ПОКА ЧТО НЕ ПЛАНИРУЮ РЕАЛИЗОВЫВАТЬ ПЕРЕДАЧУ ДАННЫХ В waybar_json ДЛЯ ГЕНЕРАЦИИ ДАННЫХ
// todo: но буду это делать
/*#[test]
fn _test_waybar_json_generation() {
    let manager = GpuInfo {
        vendor: Default::default(),
        name_gpu: None,
        temperature: None,
        utilization: None,
        power_usage: None,
        core_clock: None,
        memory_util: None,
        memory_clock: None,
        active: None,
        power_limit: None,
        memory_total: None,
        driver_version: None,
        gpu: vec![GpuInfo {
            name_gpu: Some("Test GPU".to_string()),
            temperature: Some(65.0),
            utilization: Some(30.0),
            ..Default::default()
        }],
        active_gpu: 0,
        max_clock_speed: None,
    };

    let json = manager.generate_waybar_json();
    assert!(json.contains("\"text\":\"65°C\""));
    assert!(json.contains("\"tooltip\":\"Test GPU - Temp: 65°C\\nUtilization: 30%\""));
}*/

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

// Реализация моков для системных команд
#[cfg(all(not(target_os = "hermit"), any(unix, doc)))]
mod mock_impl {
    use super::*;
    use std::{
        os::unix::process::ExitStatusExt,
        process::{Command, Output},
    };

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
        let mock = MOCK_COMMAND.with(|mc| mc.borrow_mut().take());

        if let Some(mock) = mock {
            Ok(Output {
                status: std::process::ExitStatus::from_raw(if mock.success { 0 } else { 1 }),
                stdout: mock.output.as_bytes().to_vec(),
                stderr: vec![],
            })
        } else {
            Ok(Output {
                status: ExitStatus::from_raw(0),
                stdout: vec![],
                stderr: vec![],
            })
        }
    }
}

// Переопределение системных команд для тестов
// #[cfg(test)]
// impl GpuInfo {
//     /// A test-only implementation of `info_gpu` that mocks the
//     /// execution of the `nvidia-smi` command.
//     ///
//     /// This function is only available in the `test` configuration and is
//     /// used to test the `info_gpu` method without relying on the
//     /// actual `nvidia-smi` command. It takes a mutable reference to a
//     /// `GpuInfo` object and updates it with the mocked output.
//     ///
//     /// # Arguments
//     ///
//     /// * `gpu` - A mutable reference to a `GpuInfo` that is being updated.
//     fn _test_update_nvidia_info(gpu: &mut GpuInfo) {
//         mock_impl::_command_mock(&mut Command::new("nvidia-smi")).unwrap();
//         GpuInfo::update_nvidia_info(gpu)
//     }
// }

/// Tests the detection of an AMD GPU by mocking the sysfs vendor file.
///
/// This test creates a temporary directory structure mimicking the sysfs paths
/// used by AMD GPUs. It writes a mock vendor ID into the vendor file and then
/// invokes the `detect_gpus` method of `GpuManager` to ensure that an AMD GPU
/// is correctly detected and added to the manager's list of GPUs.
///
/// # Assertions
///
/// - The GPU list is not empty after detection.
/// - At least one of the detected GPUs matches the AMD vendor.
//#[test]
// fn test_amd_parsing() {
//     let tmp_dir = TempDir::new().unwrap();
//     let card_path = tmp_dir.path().join("card0/device");
//     fs::create_dir_all(&card_path).unwrap();
//
//     let mut vendor_file = File::create(card_path.join("vendor")).unwrap();
//     writeln!(vendor_file, "0x1002").unwrap(); // PCI ID AMD
//
//     let mut manager = GpuInfo::default();
//     manager.active();
//
//     assert!(!manager.gpus.is_empty());
//     assert!(manager.gpus.iter().any(|g| matches!(g.vendor, Vendor::Amd)));
// }

/// Tests the detection of an Intel GPU by mocking the sysfs intel_info file.
///
/// This test creates a temporary directory structure mimicking the sysfs paths
/// used by Intel GPUs. It writes a mock vendor ID into the intel_info file and
/// then invokes the `detect_gpus` method of `GpuManager` to ensure that an
/// Intel GPU is correctly detected and added to the manager's list of GPUs.
///
/// # Assertions
///
/// - The GPU list is not empty after detection.
/// - At least one of the detected GPUs matches the Intel vendor.
// #[test]
// fn test_intel_parsing() {
//     let tmp_dir = TempDir::new().unwrap();
//     let card_path = tmp_dir.path().join("card0/device");
//     fs::create_dir_all(&card_path).unwrap();
//
//     let mut info_file = File::create(card_path.join("intel_info")).unwrap();
//     writeln!(info_file, "Intel GPU").unwrap();
//
//     let mut manager = GpuInfo::default();
//     manager.active();
//
//     assert!(!manager.active().is_none());
//     assert!(manager
//         .vendor
//         .eq(|g| matches!(g.vendor, Vendor::Intel)));
// }

/// Tests that metrics are correctly updated for an NVIDIA GPU.
///
/// This test creates a mock `GpuManager` with a single NVIDIA GPU and
/// then invokes the `refresh` method to update the metrics. It asserts
/// that the metrics are correctly updated based on the mock `nvidia-smi`
/// output.
// #[test]
// fn test_metrics_update() {
//     mock_command(true, "75,50,1500,100\n");
//
//     let mut manager = gpu_info {
//         gpus: vec![GpuInfo {
//             vendor: Vendor::Nvidia,
//             ..Default::default()
//         }],
//         active_gpu: 0,
//     };
//
//     manager.refresh();
//     let gpu = &manager.gpus[0];
//
//     assert_eq!(gpu.temperature, Some(75.0));
//     assert_eq!(gpu.utilization, Some(50.0));
//     assert_eq!(gpu.clock_speed, Some(1500));
//     assert_eq!(gpu.power_usage, Some(100.0));
// }

/// Tests error handling in the `detect_gpus` and `refresh` methods
///
/// This test ensures that the `detect_gpus` and `refresh` methods correctly
/// handle errors when running the `nvidia-smi` command:
///
/// - When the command fails to execute, no NVIDIA GPUs should be detected.
/// - When the command produces invalid output, no NVIDIA GPUs should be
///   detected.
///
/// # Assertions
///
/// - The `detect_gpus` method does not detect any NVIDIA GPUs if the command
///   fails to execute.
/// - The `detect_gpus` method does not detect any NVIDIA GPUs if the command
///   produces invalid output.
//#[test]
// fn test_error_handling() {
//     mock_command(false, "");
//     let mut manager = GpuInfo::default();
//     manager.active();
//
//     assert!(manager
//         .gpu
//         .iter()
//         .all(|g| !matches!(g.vendor, Vendor::Nvidia)));
//
//     mock_command(true, "invalid,data,here\n");
//     manager.active();
//     assert!(manager.gpus.is_empty());
// }

/// Integration test that detects real GPUs on the system
///
/// This test detects the GPUs on the system and checks if the `GpuManager`
/// correctly detects the GPUs. The test is only run on Linux systems,
/// since the paths and commands used in the test are specific to Linux.
///
/// # Assertions
///
/// - If the `nvidia-smi` command is available, at least one NVIDIA GPU
///   should be detected.
/// - If the sysfs `vendor` file is available, at least one AMD or Intel GPU
///   should be detected, depending on the contents of the `vendor` file.
#[test]
#[cfg(target_os = "linux")]
fn integration_test_real_system() {
    let mut manager = GpuManager::new();
    manager.detect_gpus();

    if Path::new("/usr/bin/nvidia-smi").exists() {
        assert!(manager
            .gpus
            .iter()
            .any(|g| matches!(g.vendor, GpuVendor::Nvidia)));
    }

    if Path::new("/sys/class/drm/card0/device/vendor").exists() {
        let vendor = fs::read_to_string("/sys/class/drm/card0/device/vendor").unwrap_or_default();

        if vendor.contains("0x1002") {
            assert!(manager
                .gpu
                .iter()
                .any(|g| matches!(g.vendor, GpuVendor::AMD)));
        }

        if vendor.contains("0x8086") {
            assert!(manager
                .gpu
                .iter()
                .any(|g| matches!(g.vendor, GpuVendor::Intel)));
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
    manager.vendor;
    let gpu = manager;
    assert!(matches!(gpu.vendor, crate::vendor::Vendor::Nvidia));
}

//TODO: пока что не могу разобраться почему не проходит тест
//возможно из за того что он сравнивает с текущей системой
//#[test]
//TODO: ПОКА НОРМАЛЬНО НЕ ПРОРАБОТАНЫ ФУНКЦИИ РАБОТЫ С AMD ПРОСТО НЕ ИМЕЕТ СМЫЛА РАБОТАТЬ НАД ТЕСТАМИ
// НО ЕСЛИ ЧТО ТЕСТ БУДЕТ ВЫГДЯТЕТЬ ИМЕННО ТАК ,ОТ ЭТОГО И БУДЕМ ОТТАЛКИВАТЬСЯ КАК БУДЕТ РАБОТАТЬ ФУНКЦИИ
/*fn test_get_vendor_amd() {
    let mut manager = GpuInfo::default();
    manager.detect_gpus();
    let gpu = &manager.gpus[0];
    assert!(matches!(gpu.vendor, Vendor::Amd));
}*/
//TODO: ПОКА НОРМАЛЬНО НЕ ПРОРАБОТАНЫ ФУНКЦИИ РАБОТЫ С INTEL ПРОСТО НЕ ИМЕЕТ СМЫЛА РАБОТАТЬ НАД ТЕСТАМИ
// НО ЕСЛИ ЧТО ТЕСТ БУДЕТ ВЫГДЯТЕТЬ ИМЕННО ТАК ,ОТ ЭТОГО И БУДЕМ ОТТАЛКИВАТЬСЯ КАК БУДЕТ РАБОТАТЬ ФУНКЦИИ
//#[test]
/*fn test_get_vendor_intel() {
    let mut manager = GpuInfo::default();
    manager.detect_gpus();
    let gpu = &manager.gpus[0];
    assert!(matches!(gpu.vendor, Vendor::Intel));
}*/
// TODO: ПОКА ЧТО ДАННЫЙ ТЕСТ НЕ НУЖЕН ТАК КАК МЫ БОЛЬШЕ НЕ ПАРСИМ ИНФОРМАЦИЮ ,А ПОЛУЧАЕМ КОНКРЕТНЫЕ ЗНАЧЕНИЯ ПО ОТДЕЛЬНОСТИ
// НО В БУДУЕМ МОЖЕТ ПРИГОДИТСЯ ДАННЫЙ ТЕСТ
//#[test]
// fn test_partial_data_parsing() {
//     mock_command(true, "NVIDIA GPU,75,,1500,,100,\n");
//     let mut manager = gpu_info::GpuInfo::default();
//     manager.active;
//
//     let gpu = &manager;
//     assert_eq!(gpu.utilization, None);
//     assert_eq!(gpu.max_clock_speed, None);
//     assert_eq!(gpu.power_limit, None);
// }
#[cfg(test)]
#[cfg(target_os = "linux")]
mod linux_nvidia_test {
    use crate::imp::{update_nvidia_info, MockNvmlClient, NVML_SUCCESS};

    /// Test `update_nvidia_info()` updates GPU information
    #[test]
    fn update_nvidia_info_updates_gpu_data() {
        let mut gpu = crate::GpuInfo {
            name_gpu: Some("NVIDIA GeForce RTX 3080".to_string()),
            ..Default::default()
        };

        let mut mock_client = MockNvmlClient::new();
        mock_client.expect_init().returning(|| NVML_SUCCESS);
        mock_client.expect_shutdown().returning(|| NVML_SUCCESS);
        mock_client.expect_get_count().returning(|count| unsafe {
            *count = 1;
            NVML_SUCCESS
        });
        mock_client
            .expect_get_handle_by_index()
            .returning(|_, device| unsafe {
                *device = std::ptr::null_mut();
                NVML_SUCCESS
            });
        mock_client
            .expect_get_name()
            .returning(|_, name, length| unsafe {
                let test_name = "NVIDIA GeForce RTX 3080".as_bytes();
                std::ptr::copy_nonoverlapping(
                    test_name.as_ptr(),
                    name as *mut u8,
                    test_name.len().min(length as usize),
                );
                *name.add(test_name.len().min(length as usize)) = 0;
                NVML_SUCCESS
            });
        mock_client
            .expect_get_temperature()
            .returning(|_, _, temp| unsafe {
                *temp = 70;
                NVML_SUCCESS
            });
        mock_client
            .expect_get_utilization_rates()
            .returning(|_, util| unsafe {
                (*util).gpu = 85;
                (*util).memory = 75;
                NVML_SUCCESS
            });
        mock_client
            .expect_get_power_usage()
            .returning(|_, milliwatts| unsafe {
                *milliwatts = 120000;
                NVML_SUCCESS
            });
        mock_client
            .expect_get_clock_info()
            .returning(|_, _, clock| unsafe {
                *clock = 1500;
                NVML_SUCCESS
            });
        mock_client
            .expect_get_max_clock_info()
            .returning(|_, _, clock| unsafe {
                *clock = 2100;
                NVML_SUCCESS
            });
        mock_client
            .expect_get_power_management_limit()
            .returning(|_, limit| unsafe {
                *limit = 250000;
                NVML_SUCCESS
            });

        update_nvidia_info(&mut gpu);

        assert_eq!(gpu.temperature, Some(70.0));
        assert_eq!(gpu.utilization, Some(85.0));
        assert_eq!(gpu.memory_util, Some(75.0));
        assert_eq!(gpu.power_usage, Some(120.0));
        assert_eq!(gpu.core_clock, Some(1500));
        assert_eq!(gpu.max_clock_speed, Some(2100));
        assert_eq!(gpu.power_limit, Some(250.0));
        assert_eq!(gpu.active, Some(true));
    }
}
