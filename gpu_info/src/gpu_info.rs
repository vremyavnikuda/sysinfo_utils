//! GPU Monitoring and Management Library
//!
//! Provides cross-platform GPU monitoring capabilities with support for:
//! - NVIDIA GPUs (via `nvidia-smi`)
//! - AMD GPUs (via sysfs interface)
//! - Intel integrated graphics (via sysfs)
//!
//! # Features
//! - Real-time metrics collection
//! - Multi-GPU support
//! - Waybar integration
//! - Power state monitoring
//! - Vendor-specific data collection
//!
//! # Platform Support
//! - Linux: Full support
//! - Windows: Partial NVIDIA support
//FIXME: обратить внимание на поддержку Mac OS and Windows
//! - macOS: Not currently supported
//!
//! # Examples
//! ## Basic Usage
//! ```rust
//! use gpu_info::{GpuManager, GpuInfo};
//!
//! let mut manager = GpuManager::new();
//! manager.refresh();
//!
//! for (idx, gpu) in manager.gpus.iter().enumerate() {
//!     println!("GPU {}: {}", idx, gpu.get_name());
//!     println!("Temperature: {}", gpu.get_temperature());
//! }
//! ```
//!
//! ## Waybar Integration
//! ```rust,no_run
//! let manager = GpuManager::new();
//! println!("{}", manager.generate_waybar_json());
//! ```
use std::{path::Path, process::Command};

use serde::{Deserialize, Serialize};
use sysinfo::System;
/// Represents GPU hardware vendors
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Enum representing different GPU vendors.
pub enum GpuVendor {
    /// NVIDIA Corporation devices
    Nvidia,
    /// Advanced Micro Devices (AMD) devices
    AMD,
    /// Intel Corporation integrated graphics
    Intel,
    /// Unknown or unsupported GPU vendor
    Unknown,
}

impl Default for GpuVendor {
    /// Default vendor is Unknown
    fn default() -> Self {
        GpuVendor::Unknown
    }
}
/// Contains detailed metrics and information about a GPU device
///
/// # Examples
/// ```
/// use gpu_info::{GpuInfo, GpuVendor};
///
/// let gpu = GpuInfo {
///     name: "RTX 4090".into(),
///     vendor: GpuVendor::Nvidia,
///     temperature: Some(65.0),
///     utilization: Some(45.5),
///     ..Default::default()
/// };
///
/// assert_eq!(gpu.get_utilization(), "󰾆 Utilization: 45.5%");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    /// GPU model name (e.g., "NVIDIA GeForce RTX 4090")
    pub name: String,
    /// Hardware vendor classification
    pub vendor: GpuVendor,
    /// Current temperature in Celsius (if available)
    pub temperature: Option<f32>,
    /// GPU utilization percentage (0-100)
    pub utilization: Option<f32>,
    /// Current clock speed in MHz
    pub clock_speed: Option<u64>,
    /// Maximum supported clock speed in MHz
    pub max_clock_speed: Option<u64>,
    /// Current power draw in watts
    pub power_usage: Option<f32>,
    /// Maximum power limit in watts
    pub max_power_usage: Option<f32>,
    /// Indicates if the GPU is currently active
    pub is_active: bool,
}

impl GpuInfo {
    /// Returns the GPU model name as a string slice
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns reference to vendor classification
    pub fn get_vendor(&self) -> &GpuVendor {
        &self.vendor
    }

    /// Formats temperature with icon and units
    ///
    /// # Returns
    /// - Formatted string: " Temperature: XX°C"
    /// - "N/A" if temperature unavailable
    pub fn get_temperature(&self) -> String {
        match self.temperature {
            Some(temp) => format!(" Temperature: {}°C", temp),
            None => " Temperature: N/A".to_string(),
        }
    }

    /// Formats GPU utilization percentage with icon
    ///
    /// # Returns
    /// - Formatted string: "󰾆 Utilization: XX%"
    /// - "N/A" if utilization data unavailable
    pub fn get_utilization(&self) -> String {
        match self.utilization {
            Some(util) => format!("󰾆 Utilization: {}%", util),
            None => "󰾆 Utilization: N/A".to_string(),
        }
    }

    /// Formats clock speeds with icon and units
    ///
    /// # Returns
    /// String in format " Clock Speed: CURRENT/MAX MHz"
    /// Uses 0 for missing values
    pub fn get_clock_speed(&self) -> String {
        let current = self.clock_speed.unwrap_or(0);
        let max = self.max_clock_speed.unwrap_or(0);
        format!(" Clock Speed: {}/{} MHz", current, max)
    }

    /// Formats power usage with icon and precision
    ///
    /// # Returns
    /// String in format "󱪉 Power Usage: CURRENT/MAX W"
    /// - CURRENT: 2 decimal places
    /// - MAX: 0 decimal places
    pub fn get_power_usage(&self) -> String {
        let current = self.power_usage.unwrap_or(0.0);
        let max = self.max_power_usage.unwrap_or(0.0);
        format!("󱪉 Power Usage: {:.2}/{} W", current, max)
    }

    /// Indicates if the GPU is currently active
    ///
    /// # Note
    /// Activation state detection depends on vendor implementation
    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

/// Main controller for GPU detection and management
///
/// # Lifecycle
/// 1. Initialize with `new()`
/// 2. Detect GPUs with `detect_gpus()`
/// 3. Refresh metrics with `refresh()`
///
/// # Example
/// ```
/// use gpu_info::GpuManager;
///
/// let mut manager = GpuManager::new();
/// manager.refresh();
///
/// if let Some(gpu) = manager.gpus.first() {
///     println!("Active GPU: {}", gpu.get_name());
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuManager {
    /// List of detected GPUs
    pub gpus: Vec<GpuInfo>,
    /// Index of currently active GPU
    pub active_gpu: usize,
}

impl GpuManager {
    /// Creates new GpuManager with automatic GPU detection
    ///
    /// # Panics
    /// May panic if system calls fail (platform-dependent)
    ///
    /// # Examples
    /// ```
    /// let manager = GpuManager::new();
    /// assert!(!manager.gpus.is_empty());
    /// ```
    pub fn new() -> Self {
        let mut manager = GpuManager {
            gpus: Vec::new(),
            active_gpu: 0,
        };
        manager.detect_gpus();
        manager
    }

    /// Detects available GPUs using vendor-specific methods
    ///
    /// # Implementation Details
    /// - NVIDIA: Uses `nvidia-smi` CLI tool
    /// - AMD: Parses sysfs files
    /// - Intel: Checks specific sysfs paths
    ///
    /// # Platform Notes
    /// - Requires root permissions for some sysfs paths
    /// - NVIDIA detection depends on `nvidia-smi` availability
    /// 
    //FIXME: необходимо убериться что команда nvidia-smi доступна в системе
    // FIXME: необходимо убедиться что sysfs доступен для AMD и Intel
    // FIXME: необходимо убедиться что sysfs корректно определяет производителя GPU

    pub fn detect_gpus(&mut self) {
        self.gpus.clear();

        if let Ok(output) = Command::new("nvidia-smi").arg("--query-gpu=name,temperature.gpu,utilization.gpu,clocks.current.graphics,clocks.max.graphics,power.draw,power.max_limit").arg("--format=csv,noheader,nounits").output() {
            if output.status.success() {
                self.parse_nvidia_info(&String::from_utf8_lossy(&output.stdout));
            }
        }

        if Path::new("/sys/class/drm/card0/device/vendor").exists() {
            self.parse_amd_info();
        }

        if Path::new("/sys/class/drm/card0/device/intel_info").exists() {
            self.parse_intel_info();
        }
    }

    /// Switches active GPU
    ///
    /// # Arguments
    /// * `index` - Zero-based GPU index
    ///
    /// # Errors
    /// Returns `Err(String)` if index is out of bounds
    ///
    /// # Example
    /// ```
    /// let mut manager = gpu_info::GpuManager::new();
    /// assert!(manager.switch_gpu(0).is_ok());
    /// ```
    // FIXME: пока что просто меняет индекс активного GPU (нуждается в доработке)
    pub fn switch_gpu(&mut self, index: usize) -> Result<(), String> {
        if index >= self.gpus.len() {
            return Err("Invalid GPU index".into());
        }

        // TODO:Здесь должна быть логика переключения GPU. Это зависит от конкретной системы и драйверов
        self.active_gpu = index;
        Ok(())
    }

    /// Updates metrics for all detected GPUs
    ///
    /// # Refresh Rate
    /// - NVIDIA: Real-time (~1 sec latency)
    /// - AMD/Intel: Depends on sysfs update frequency
    ///
    /// # Platform Notes
    /// May block on system calls during execution
    //FIXME: проверить работоспособность в системах Linux,Windows,Mac OS
    pub fn refresh(&mut self) {
        for gpu in self.gpus.iter_mut() {
            match gpu.vendor {
                GpuVendor::Nvidia => GpuManager::update_nvidia_info(gpu),
                GpuVendor::AMD => GpuManager::update_amd_info(gpu),
                GpuVendor::Intel => GpuManager::update_intel_info(gpu),
                _ => {}
            }
        }
    }

    /// Generates Waybar-compatible JSON output
    ///
    /// # Output Format
    /// ```json
    /// {
    ///   "text": "65°C",
    ///   "tooltip": "GPU Name - Temp: 65°C\nUtilization: 45%"
    /// }
    /// ```
    ///
    /// # Dependencies
    /// Requires `serde_json` feature enabled
    pub fn generate_waybar_json(&self) -> String {
        let active_gpu = &self.gpus[self.active_gpu];

        let json = serde_json::json!({
            "text": format!("{}°C", active_gpu.temperature.unwrap_or(0.0)),
            "tooltip": format!(
                "{} - Temp: {}°C\nUtilization: {}%",
                active_gpu.name,
                active_gpu.temperature.unwrap_or(0.0),
                active_gpu.utilization.unwrap_or(0.0)
            )
        });

        json.to_string()
    }

    /// Checks NVIDIA power management state
    ///
    /// # Returns
    /// `true` if any NVIDIA processes are running
    ///
    /// # Platform Support
    /// Linux-only detection
    // FIXME: пока что проверяет только процессы с именем nvidia
    // FIXME: не работает в системах Linux and Mac OS
    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Добавить поддержку других производителей GPU(AMD, Intel)
    pub fn check_power_state(&self) -> bool {
        let sys = System::new_all();
        sys.processes().values().any(|p| {
            p.name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains("nvidia")
        })
    }

    // Private methods with documentation
    /// (Internal) Parses NVIDIA GPU information
    ///
    /// # Input Format
    /// CSV output from `nvidia-smi`:
    /// `name,temp,utilization,clock,clock_max,power,power_max`
    fn parse_nvidia_info(&mut self, data: &str) {
        for line in data.lines() {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 7 {
                let name = parts[0].trim().to_string();
                let temperature = parts[1].trim().parse().ok();
                let utilization = parts[2].trim().parse().ok();
                let clock_speed = parts[3].trim().parse().ok();
                let max_clock_speed = parts[4].trim().parse().ok();
                let power_usage = parts[5].trim().parse().ok();
                let max_power_usage = parts[6].trim().parse().ok();

                self.gpus.push(GpuInfo {
                    name: name.clone(),
                    vendor: GpuVendor::Nvidia,
                    temperature,
                    utilization,
                    clock_speed,
                    max_clock_speed,
                    power_usage,
                    max_power_usage,
                    is_active: true,
                });
                break;
            }
        }
    }

    /// (Internal) Checks if an AMD GPU is present and adds it to the list
    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Требуется документация
    fn parse_amd_info(&mut self) {
        if let Ok(output) = std::fs::read_to_string("/sys/class/drm/card0/device/vendor") {
            if output.contains("AMD") {
                self.gpus.push(GpuInfo {
                    name: "AMD GPU".to_string(),
                    vendor: GpuVendor::AMD,
                    temperature: None,
                    utilization: None,
                    clock_speed: None,
                    max_clock_speed: None,
                    power_usage: None,
                    max_power_usage: None,
                    is_active: true,
                });
            }
        }
    }

    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Требуется документация
    fn parse_intel_info(&mut self) {
        if let Ok(output) = std::fs::read_to_string("/sys/class/drm/card0/device/intel_info") {
            if output.contains("Intel") {
                self.gpus.push(GpuInfo {
                    name: "Intel GPU".to_string(),
                    vendor: GpuVendor::Intel,
                    temperature: None,
                    utilization: None,
                    clock_speed: None,
                    max_clock_speed: None,
                    power_usage: None,
                    max_power_usage: None,
                    is_active: true,
                });
            }
        }
    }

    // (Internal) Updates NVIDIA GPU metrics
    ///
    /// # Data Sources
    /// - temperature.gpu
    /// - utilization.gpu
    /// - clocks.current.graphics
    /// - power.draw
    fn update_nvidia_info(gpu: &mut GpuInfo) {
        if let Ok(output) = Command::new("nvidia-smi")
            .arg("--query-gpu=temperature.gpu,utilization.gpu,clocks.current.graphics,power.draw")
            .arg("--format=csv,noheader,nounits")
            .output()
        {
            if output.status.success() {
                let data = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = data.split(',').collect();
                if parts.len() >= 4 {
                    gpu.temperature = parts[0].trim().parse().ok();
                    gpu.utilization = parts[1].trim().parse().ok();
                    gpu.clock_speed = parts[2].trim().parse().ok();
                    gpu.power_usage = parts[3].trim().parse().ok();
                }
            }
        }
    }

    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Требуется документация
    fn update_amd_info(gpu: &mut GpuInfo) {
        if let Ok(temp) =
            std::fs::read_to_string("/sys/class/drm/card0/device/hwmon/hwmon0/temp1_input")
        {
            gpu.temperature = temp.trim().parse::<f32>().ok().map(|t| t / 1000.0);
        }
        if let Ok(util) = std::fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
            gpu.utilization = util.trim().parse().ok();
        }
        if let Ok(clock) = std::fs::read_to_string("/sys/class/drm/card0/device/pp_dpm_sclk") {
            gpu.clock_speed = clock
                .lines()
                .last()
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|s| s.parse().ok());
        }
        if let Ok(power) =
            std::fs::read_to_string("/sys/class/drm/card0/device/hwmon/hwmon0/power1_average")
        {
            gpu.power_usage = power.trim().parse::<f32>().ok().map(|p| p / 1000000.0);
        }
    }

    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Требуется документация
    fn update_intel_info(gpu: &mut GpuInfo) {
        if let Ok(temp) =
            std::fs::read_to_string("/sys/class/drm/card0/device/hwmon/hwmon0/temp1_input")
        {
            gpu.temperature = temp.trim().parse::<f32>().ok().map(|t| t / 1000.0);
        }
        if let Ok(util) = std::fs::read_to_string("/sys/class/drm/card0/device/gpu_busy_percent") {
            gpu.utilization = util.trim().parse().ok();
        }
        if let Ok(clock) = std::fs::read_to_string("/sys/class/drm/card0/device/gt_max_freq_mhz") {
            gpu.clock_speed = clock.trim().parse().ok();
        }
        if let Ok(power) =
            std::fs::read_to_string("/sys/class/drm/card0/device/hwmon/hwmon0/power1_average")
        {
            gpu.power_usage = power.trim().parse::<f32>().ok().map(|p| p / 1000000.0);
        }
    }
}

// TODO: Добавить документацию
// TODO: Добавить тесты для всех платформ
// TODO: Добавить интеграционные тесты
// TODO: Добавить обработку ошибок
// TODO: Добавить доработки для других платформ( Windows, macOS)
#[cfg(test)]
mod gpu_info_tests {
    use super::*;
    use std::cell::RefCell;
    use std::process::ExitStatus;

    struct MockCommand {
        success: bool,
        output: &'static str,
    }

    impl MockCommand {
        fn new(success: bool, output: &'static str) -> Self {
            Self { success, output }
        }
    }

    thread_local! {
        static MOCK_COMMAND: RefCell<Option<MockCommand>> = RefCell::new(None);
    }

    fn mock_command(success: bool, output: &'static str) {
        MOCK_COMMAND.with(|mc| {
            *mc.borrow_mut() = Some(MockCommand::new(success, output));
        });
    }

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

        assert_eq!(gpu.get_name(), "Test GPU");
        assert!(matches!(gpu.get_vendor(), GpuVendor::Nvidia));
        assert_eq!(gpu.get_temperature(), " Temperature: 75°C");
        assert_eq!(gpu.get_utilization(), "󰾆 Utilization: 50%");
        assert_eq!(gpu.get_clock_speed(), " Clock Speed: 1500/2000 MHz");
        assert_eq!(gpu.get_power_usage(), "󱪉 Power Usage: 100.00/150 W");
        assert!(gpu.is_active());
    }

    #[test]
    fn _test_gpu_manager_creation() {
        let manager = GpuManager::new();
        assert!(
            !manager.gpus.is_empty(),
            "Expected gpus to be empty, but it was not."
        );
        assert_eq!(manager.active_gpu, 0);
    }

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
        fn _test_update_nvidia_info(gpu: &mut GpuInfo) {
            mock_impl::_command_mock(&mut Command::new("nvidia-smi")).unwrap();
            super::GpuManager::update_nvidia_info(gpu)
        }
    }
}
