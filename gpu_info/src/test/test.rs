#[cfg(test)]
mod gpu_info_tests {
    use crate::mode::gpu::GpuVendor;
    use crate::{GpuInfo, GpuManager};
    use std::cell::RefCell;
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::process::{Command, ExitStatus};
    use tempfile::TempDir;

    struct MockCommand {
        success: bool,
        output: &'static str,
    }

    impl MockCommand {
        /// Creates a new `MockCommand` with the given success state and output.
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
    /// will be used instead of the real command when `Command::new` is called.
    ///
    /// # Arguments
    ///
    /// * `success`: A boolean indicating whether the mocked command should
    ///   succeed or fail.
    /// * `output`: A string slice containing the output of the mocked command.
    ///
    /// # Examples
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
    /// - `get_name`: Ensures the GPU name is returned correctly.
    /// - `get_vendor`: Checks if the vendor is correctly identified.
    /// - `get_temperature`: Confirms formatting and value of temperature.
    /// - `get_utilization`: Validates the utilization percentage format.
    /// - `get_clock_speed`: Checks the clock speed formatting.
    /// - `get_power_usage`: Ensures power usage is formatted correctly.
    /// - `is_active`: Verifies if the GPU is active.
    #[test]
    fn _test_gpu_info_methods() {
        let gpu = GpuInfo {
            name: "Test GPU".to_string(),
            vendor: GpuVendor::Nvidia,
            temperature: Some(75.0),
            utilization: Some(50.0),
            clock_speed: Some(1500),
            max_clock_speed: Some(2000),
            power_usage: Some(100.0),
            max_power_usage: Some(150.0),
            is_active: true,
        };

        assert_eq!(gpu.name_gpu(), "Test GPU");
        assert!(matches!(gpu.vendor_gpu(), GpuVendor::Nvidia));
        assert_eq!(gpu.temperature_gpu(), "Temperature: 75°C");
        assert_eq!(gpu.utilization_gpu(), "Utilization: 50%");
        assert_eq!(gpu.clock_speed_gpu(), "Clock Speed: 1500/2000 MHz");
        assert_eq!(gpu.power_usage_gpu(), "Power Usage: 100.00/150 W");
        assert!(gpu.is_active_gpu());
    }

    /// Tests that `GpuManager` can be successfully created and that it is
    /// initialized with a non-empty list of GPUs and the active GPU set to 0.
    #[test]
    fn _test_gpu_manager_creation() {
        let manager = GpuManager::new();
        assert!(
            !manager.gpus.is_empty(),
            "Expected gpus to be empty, but it was not."
        );
        assert_eq!(manager.active_gpu, 0);
    }

    /// Tests that `GpuManager` can parse a line of NVIDIA GPU information
    /// produced by `nvidia-smi` and create a `GpuInfo` instance from it.
    ///
    /// # Assertions
    ///
    /// - The `GpuManager` instance is not empty after calling `detect_gpus()`.
    /// - The first GPU in the list is an NVIDIA GPU.
    /// - The name of the first GPU starts with either "NVIDIA", "AMD", or "INTEL".
    #[test]
    fn _test_nvidia_parsing() {
        mock_command(true, "NVIDIA GPU,75,50,1500,2000,100,150\n");

        let mut manager = GpuManager::new();
        manager.detect_gpus();

        assert!(!manager.gpus.is_empty());
        let gpu = &manager.gpus[0];
        assert!(
            gpu.name.starts_with("NVIDIA")
                || gpu.name.starts_with("AMD")
                || gpu.name.starts_with("INTEL")
        );
        assert!(matches!(gpu.vendor, GpuVendor::Nvidia));
    }

    /// Tests the functionality of switching active GPUs in `GpuManager`.
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
        let mut manager = GpuManager {
            gpus: vec![
                GpuInfo {
                    name: "GPU1".to_string(),
                    vendor: GpuVendor::Nvidia,
                    ..Default::default()
                },
                GpuInfo {
                    name: "GPU2".to_string(),
                    vendor: GpuVendor::AMD,
                    ..Default::default()
                },
            ],
            active_gpu: 0,
        };

        assert!(manager.switch_gpu(1).is_ok());
        assert_eq!(manager.active_gpu, 1);
        assert!(manager.switch_gpu(2).is_err());
    }

    /// Tests the `generate_waybar_json` method of `GpuManager`.
    ///
    /// This test initializes a `GpuManager` with a single `GpuInfo` instance
    /// and verifies that the generated JSON string contains the expected
    /// temperature and tooltip information.
    ///
    /// # Assertions
    ///
    /// - The JSON string contains a "text" field with the temperature "65°C".
    /// - The JSON string contains a "tooltip" field with the expected format,
    ///   including the GPU name and utilization percentage.
    #[test]
    fn _test_waybar_json_generation() {
        let manager = GpuManager {
            gpus: vec![GpuInfo {
                name: "Test GPU".to_string(),
                temperature: Some(65.0),
                utilization: Some(30.0),
                ..Default::default()
            }],
            active_gpu: 0,
        };

        let json = manager.generate_waybar_json();
        assert!(json.contains("\"text\":\"65°C\""));
        assert!(json.contains("\"tooltip\":\"Test GPU - Temp: 65°C\\nUtilization: 30%\""));
    }

    /// Tests the `check_power_state` method of `GpuManager`.
    ///
    /// # Note
    ///
    /// This test currently does nothing, as it requires integration testing with
    /// real processes.
    #[test]
    fn _test_power_state_check() {
        let manager = GpuManager::new();
        // Требует интеграционного тестирования с реальными процессами
        let _ = manager.check_power_state();
    }

    // Реализация моков для системных команд
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
                    status: ExitStatus::from_raw(if mock.success { 0 } else { 1 }),
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
    #[cfg(test)]
    impl GpuManager {
        /// A test-only implementation of `update_nvidia_info` that mocks the
        /// execution of the `nvidia-smi` command.
        ///
        /// This function is only available in the `test` configuration and is
        /// used to test the `update_nvidia_info` method without relying on the
        /// actual `nvidia-smi` command. It takes a mutable reference to a
        /// `GpuInfo` object and updates it with the mocked output.
        ///
        /// # Arguments
        ///
        /// * `gpu` - A mutable reference to a `GpuInfo` that is being updated.
        fn _test_update_nvidia_info(gpu: &mut GpuInfo) {
            mock_impl::_command_mock(&mut Command::new("nvidia-smi")).unwrap();
            GpuManager::update_nvidia_info(gpu)
        }
    }

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
    #[test]
    fn test_amd_parsing() {
        let tmp_dir = TempDir::new().unwrap();
        let card_path = tmp_dir.path().join("card0/device");
        fs::create_dir_all(&card_path).unwrap();

        let mut vendor_file = File::create(card_path.join("vendor")).unwrap();
        writeln!(vendor_file, "0x1002").unwrap(); // PCI ID AMD

        let mut manager = GpuManager::new();
        manager.detect_gpus();

        assert!(!manager.gpus.is_empty());
        assert!(manager
            .gpus
            .iter()
            .any(|g| matches!(g.vendor, GpuVendor::AMD)));
    }

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
    #[test]
    fn test_intel_parsing() {
        let tmp_dir = TempDir::new().unwrap();
        let card_path = tmp_dir.path().join("card0/device");
        fs::create_dir_all(&card_path).unwrap();

        let mut info_file = File::create(card_path.join("intel_info")).unwrap();
        writeln!(info_file, "Intel GPU").unwrap();

        let mut manager = GpuManager::new();
        manager.detect_gpus();

        assert!(!manager.gpus.is_empty());
        assert!(manager
            .gpus
            .iter()
            .any(|g| matches!(g.vendor, GpuVendor::Intel)));
    }

    /// Tests that metrics are correctly updated for an NVIDIA GPU.
    ///
    /// This test creates a mock `GpuManager` with a single NVIDIA GPU and
    /// then invokes the `refresh` method to update the metrics. It asserts
    /// that the metrics are correctly updated based on the mock `nvidia-smi`
    /// output.
    #[test]
    fn test_metrics_update() {
        mock_command(true, "75,50,1500,100\n");
        let mut manager = GpuManager {
            gpus: vec![GpuInfo {
                vendor: GpuVendor::Nvidia,
                ..Default::default()
            }],
            active_gpu: 0,
        };

        manager.refresh();
        let gpu = &manager.gpus[0];

        assert_eq!(gpu.temperature, Some(50.0));
        assert_eq!(gpu.utilization, Some(75.0));
        assert_eq!(gpu.clock_speed, Some(1500));
        assert_eq!(gpu.power_usage, Some(100.0));
    }

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
    #[test]
    fn test_error_handling() {
        mock_command(false, "");
        let mut manager = GpuManager::new();
        manager.detect_gpus();

        assert!(manager
            .gpus
            .iter()
            .all(|g| !matches!(g.vendor, GpuVendor::Nvidia)));

        mock_command(true, "invalid,data,here\n");
        manager.detect_gpus();
        assert!(manager.gpus.is_empty());
    }

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
            let vendor =
                fs::read_to_string("/sys/class/drm/card0/device/vendor").unwrap_or_default();

            if vendor.contains("0x1002") {
                assert!(manager
                    .gpus
                    .iter()
                    .any(|g| matches!(g.vendor, GpuVendor::AMD)));
            }

            if vendor.contains("0x8086") {
                assert!(manager
                    .gpus
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
        let mut manager = GpuManager::new();
        manager.detect_gpus();
        let gpu = &manager.gpus[0];
        assert!(matches!(gpu.vendor, GpuVendor::Nvidia));
    }

    //TODO: пока что не могу разобраться почему не проходит тест
    //возможно из за того что он сравнивает с текущей системой
    #[test]
    fn test_get_vendor_amd() {
        let mut manager = GpuManager::new();
        manager.detect_gpus();
        let gpu = &manager.gpus[0];
        assert!(matches!(gpu.vendor, GpuVendor::AMD));
    }
    //TODO: пока что не могу разобраться почему не проходит тест
    //возможно из за того что он сравнивает с текущей системой
    #[test]
    fn test_get_vendor_intel() {
        let mut manager = GpuManager::new();
        manager.detect_gpus();
        let gpu = &manager.gpus[0];
        assert!(matches!(gpu.vendor, GpuVendor::Intel));
    }

    /// Tests that partial data is correctly parsed from `nvidia-smi` output
    ///
    /// This test ensures that the `GpuManager` correctly parses the `nvidia-smi`
    /// output even when some of the fields are missing (e.g. `utilization.gpu` or
    /// `clocks.max.graphics`). The test creates a mock `GpuManager` with a single
    /// NVIDIA GPU and then invokes the `detect_gpus` method to detect the GPU.
    /// It asserts that the `utilization`, `max_clock_speed`, and `max_power_usage`
    /// fields are correctly set to `None` for the detected GPU.
    #[test]
    fn test_partial_data_parsing() {
        mock_command(true, "NVIDIA GPU,75,,1500,,100,\n");
        let mut manager = GpuManager::new();
        manager.detect_gpus();

        let gpu = &manager.gpus[0];
        assert_eq!(gpu.utilization, None);
        assert_eq!(gpu.max_clock_speed, None);
        assert_eq!(gpu.max_power_usage, None);
    }
}
