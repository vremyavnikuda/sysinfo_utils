//gpu_info/src/mode/manager.rs
//! GPU management and detection implementation
use crate::amd::detect_amd_gpus;
use crate::amd::update_amd_info;
use crate::intel::detect_intel_gpus;
use crate::intel::update_intel_info;
use crate::mode::gpu::{GpuInfo, GpuVendor};
use crate::nvidia::{detect_nvidia_gpus, update_nvidia_info};
use log::warn;
use serde::{Deserialize, Serialize};
use serde_json;
use sysinfo::System;

/// A central controller for GPU detection and management
///
/// Handles GPU detection, status monitoring, and system integration.
///
/// # Lifecycle
/// 1. Initialize with `init()`
/// 2. Detect GPUs with `detect_gpus()`
/// 3. Refresh metrics with `refresh()`.
///
/// # Example
/// ```
/// use gpu_info::GpuManager;
///
/// let mut manager = GpuManager::init();
/// manager.refresh();
///
/// if let Some(gpu) = manager.gpu.first() {
///     println!("Active GPU: {}", gpu.get_name());
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuManager {
    /// List of detected GPUs
    pub gpu: Vec<GpuInfo>,
    /// Index of currently active GPU
    pub active_gpu: usize,
}

impl GpuManager {
    /// Creates init GpuManager with automatic GPU detection
    ///
    /// # Panics
    /// May panic if system calls fail (platform-dependent)
    pub fn init() -> Self {
        let mut manager = GpuManager {
            gpu: Vec::new(),
            active_gpu: 0,
        };
        manager.detect_gpus();
        manager
    }

    /// Detects available GPUs using vendor-specific methods
    ///
    /// # Implementation Details
    /// — nvidia: Uses `nvidia-smi` CLI tool
    /// — amd: Parses sysfs files
    /// — Intel: Checks specific sysfs paths.
    ///
    /// # Platform Notes
    /// — Requires root permissions for some sysfs paths
    /// — nvidia detection depends on `nvidia-smi` availability.
    ///
    /// # Panics
    /// May panic if system calls fail (platform-dependent)
    ///
    // TODO: - detect_gpus()
    // TODO: - Убедиться, что sysfs доступен для amd и Intel
    // TODO: - Проверить доступность `nvidia-smi` в системе
    // TODO: - Убедиться, что sysfs корректно определяет производителя GPU
    /// Обнаружение всех установленных GPU
    pub fn detect_gpus(&mut self) {
        self.gpu.clear();

        self.gpu.extend(detect_nvidia_gpus());
        self.gpu.extend(detect_amd_gpus());
        self.gpu.extend(detect_intel_gpus());
    }

    //TODO:Qualcomm add
    //fn qualcomm_info(&mut self){}
    /// Switches active GPU
    ///
    /// # Arguments
    /// * `index` — Zero-based GPU index
    ///
    /// # Errors
    /// Returns `Err (String)` if index is out of bounds.
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let mut manager = GpuManager::init();
    /// assert!(manager.switch_gpu(0).is_ok());
    /// ```
    // TODO: пока что просто меняет индекс активного GPU (нуждается в доработке)
    pub fn switch_gpu(&mut self, index: usize) -> Result<(), String> {
        if index >= self.gpu.len() {
            return Err("Invalid GPU index".into());
        }

        // TODO:Здесь должна быть логика переключения GPU. Это зависит от конкретной системы и драйверов
        self.active_gpu = index;
        Ok(())
    }

    /// Updates metrics for all detected GPUs
    ///
    /// # Refresh Rate
    /// — nvidia: Real-time (~1 sec latency)
    /// — amd/Intel: Depends on sysfs update frequency.
    ///
    /// # Platform Notes
    /// May block on system calls during execution
    // TODO: проверить работоспособность в системах Linux,Windows,Mac OS
    pub fn refresh(&mut self) {
        for gpu in self.gpu.iter_mut() {
            match gpu.vendor {
                GpuVendor::Nvidia => update_nvidia_info(gpu),
                GpuVendor::AMD => update_amd_info(gpu),
                GpuVendor::Intel => update_intel_info(gpu),
                _ => {
                    warn!("Unknown vendor, skipping metrics refresh");
                }
            }
        }
    }

    /// Generates Waybar-compatible JSON output
    ///
    /// # Output Format
    /// ```json
    /// {
    ///   "text": "65°C",
    ///   "tooltip": "GPU Name - Temp: 65°C, Utilization: 45%"
    /// }
    /// ```
    ///
    /// # Dependencies
    /// Requires `serde_json` feature enabled
    pub fn generate_waybar_json(&self) -> String {
        let active_gpu = &self.gpu[self.active_gpu];

        if self.gpu.is_empty() {
            return serde_json::json!({"text": "No GPU detected"}).to_string();
        }

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

    /// Checks nvidia power management state
    ///
    /// # Returns
    /// `true` if any nvidia processes are running.
    ///
    /// # Platform Support
    /// Linux-only detection
    // TODO: пока что проверяет только процессы с именем nvidia
    // TODO: не работает в системах Linux and Mac OS
    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Добавить поддержку других производителей GPU(amd, Intel)
    pub fn check_power_state(&self) -> bool {
        let mut sys = System::new_all();
        sys.refresh_all();

        let gpu_processes = ["nvidia", "amdgpu", "radeon", "intel", "i915", "rtx"];

        sys.processes().values().any(|p| {
            let name = p.name().to_string_lossy().to_ascii_lowercase();
            gpu_processes.iter().any(|&gpu| name.contains(gpu))
        })
    }

    /// Parses a string of nvidia GPU information and creates a `GpuInfo` struct from it.
    ///
    /// The input string is expected to be in a specific format, with comma-separated values for the GPU's:
    /// 1. Name
    /// 2. Temperature
    /// 3. Utilization
    /// 4. Clock speed
    /// 5. Maximum clock speed
    /// 6. Power usage
    /// 7. Maximum power usage
    ///
    /// The function then adds the parsed `GpuInfo` to a list of GPUs (`self.gpus`).
    ///
    /// # Parameters
    ///
    /// * `data`: A string containing the nvidia GPU information.
    ///
    /// # Returns
    ///
    /// None
    pub fn parse_nvidia_info(&mut self, data: &str) {
        let mut gpu = GpuInfo {
            name: String::new(),
            vendor: GpuVendor::Nvidia,
            temperature: None,
            utilization: None,
            clock_speed: None,
            max_clock_speed: None,
            power_usage: None,
            max_power_usage: None,
            is_active: true,
        };

        for (i, part) in data.split(',').enumerate() {
            match i {
                0 => gpu.name = part.trim().to_string(),
                1 => gpu.temperature = part.trim().parse().ok(),
                2 => gpu.utilization = part.trim().parse().ok(),
                3 => gpu.clock_speed = part.trim().parse().ok(),
                4 => gpu.max_clock_speed = part.trim().parse().ok(),
                5 => gpu.power_usage = part.trim().parse().ok(),
                6 => gpu.max_power_usage = part.trim().parse().ok(),
                _ => break,
            }
        }
        self.gpu.push(gpu);
    }

    /// (Internal) Checks if an amd GPU is present and adds it to the list.
    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Требуется документация
    // TODO: не работает определение метрик amd GPU
    pub fn parse_amd_info(&mut self) {
        if let Ok(output) = std::fs::read_to_string("/sys/class/drm/card0/device/vendor") {
            if output.contains("amd") {
                self.gpu.push(GpuInfo {
                    name: "amd GPU".to_string(),
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
    /// (Internal) Checks if an Intel GPU is present and adds it to the list.
    ///
    /// # Platform Support
    /// Linux-only detection
    ///
    /// # Errors
    /// Returns `Err (String)` if the Intel GPU information cannot be read.
    pub fn parse_intel_info(&mut self) {
        if let Ok(output) = std::fs::read_to_string("/sys/class/drm/card0/device/intel_info") {
            if output.contains("Intel") {
                self.gpu.push(GpuInfo {
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

    // Delegate methods to the active GPU's GpuInfo methods
    /// Returns the vendor of the active GPU.
    ///
    /// Retrieves the vendor of the currently active GPU, falling back to `GpuVendor::Unknown` if no active GPU is available.
    ///
    /// # Returns
    /// - `GpuVendor`: The vendor of the active GPU, or `GpuVendor::Unknown` if no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::{GpuManager, GpuVendor};
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let vendor = manager.vendor_gpu();
    /// println!("GPU Vendor: {:?}", vendor);
    /// ```
    ///
    /// # Notes
    /// - This method clones the `GpuVendor` value from the active GPU’s `vendor_gpu()` method.
    /// - Ensure `refresh()` is called prior to this method to get the latest GPU data.
    /// - If the `active_gpu` index is invalid or no GPUs are detected, it returns `GpuVendor::Unknown`.
    pub fn vendor_gpu(&self) -> GpuVendor {
        self.gpu
            .get(self.active_gpu)
            .map_or(GpuVendor::Unknown, |gpu| gpu.vendor_gpu().clone())
    }

    /// Returns the name of the active GPU.
    ///
    /// Retrieves the name of the currently active GPU, falling back to "N/A" if no active GPU is available.
    ///
    /// # Returns
    /// - `String`: The name of the active GPU, or "N/A" if no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let name = manager.name_gpu();
    /// println!("GPU Name: {}", name);
    /// ```
    ///
    /// # Notes
    /// - This method converts the active GPU’s name to a `String` using its `name_gpu()` method.
    /// - If the `active_gpu` index is invalid or no GPUs are detected, it returns "N/A".
    pub fn name_gpu(&self) -> String {
        self.gpu
            .get(self.active_gpu)
            .map_or("N/A".to_string(), |gpu| gpu.name_gpu().to_string())
    }

    /// Returns the temperature of the active GPU in Celsius.
    ///
    /// Retrieves the temperature of the currently active GPU, falling back to `Some(0.0)` if no active GPU is available.
    ///
    /// # Returns
    /// - `Option<f32>`: The temperature in Celsius if available, or `Some(0.0)` if no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// if let Some(temp) = manager.temperature_gpu() {
    ///     println!("GPU Temperature: {}°C", temp);
    /// }
    /// ```
    ///
    /// # Notes
    /// - This method delegates to the active GPU’s `temperature_gpu()` method.
    /// - Returns `Some(0.0)` as a fallback instead of `None` to indicate no data is available.
    pub fn temperature_gpu(&self) -> Option<f32> {
        self.gpu
            .get(self.active_gpu)
            .map_or(Some(0.0), |gpu| gpu.temperature_gpu())
    }

    /// Returns the formatted temperature of the active GPU.
    ///
    /// Provides a human-readable string of the active GPU’s temperature, falling back to "Temperature: N/A" if no active GPU is available.
    ///
    /// # Returns
    /// - `String`: A formatted string like "Temperature: XX°C" or "Temperature: N/A".
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let temp_str = manager.format_get_temperature_gpu();
    /// println!("{}", temp_str);
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_get_temperature_gpu()` method.
    /// - If the `active_gpu` index is invalid or no GPUs are detected, it returns "Temperature: N/A".
    pub fn format_get_temperature_gpu(&self) -> String {
        self.gpu
            .get(self.active_gpu)
            .map_or("Temperature: N/A".to_string(), |gpu| {
                gpu.format_get_temperature_gpu()
            })
    }

    /// Returns the utilization percentage of the active GPU.
    ///
    /// Retrieves the utilization of the currently active GPU as a percentage, falling back to `0.0` if no active GPU is available.
    ///
    /// # Returns
    /// - `f32`: The utilization percentage (0-100), or `0.0` if no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let utilization = manager.utilization_gpu();
    /// println!("GPU Utilization: {}%", utilization);
    /// ```
    ///
    /// # Notes
    /// - This method delegates to the active GPU’s `utilization_gpu()` method.
    /// - Returns `0.0` as a fallback when no data is available.
    pub fn utilization_gpu(&self) -> f32 {
        self.gpu
            .get(self.active_gpu)
            .map_or(0.0, |gpu| gpu.utilization_gpu())
    }

    /// Returns the formatted utilization of the active GPU.
    ///
    /// Provides a human-readable string of the active GPU’s utilization, falling back to "Utilization: N/A" if no active GPU is available.
    ///
    /// # Returns
    /// - `String`: A formatted string like "Utilization: XX%" or "Utilization: N/A".
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let util_str = manager.format_get_utilization_gpu();
    /// println!("{}", util_str);
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_get_utilization_gpu()` method.
    /// - If the `active_gpu` index is invalid or no GPUs are detected, it returns "Utilization: N/A".
    pub fn format_get_utilization_gpu(&self) -> String {
        self.gpu
            .get(self.active_gpu)
            .map_or("Utilization: N/A".to_string(), |gpu| {
                gpu.format_get_utilization_gpu()
            })
    }

    /// Returns the current clock speed of the active GPU in MHz.
    ///
    /// Retrieves the current clock speed of the active GPU, falling back to `0` if no active GPU is available.
    ///
    /// # Returns
    /// - `u64`: The current clock speed in MHz, or `0` if no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let clock = manager.clock_speed_gpu();
    /// println!("GPU Clock Speed: {} MHz", clock);
    /// ```
    ///
    /// # Notes
    /// - This method delegates to the active GPU’s `clock_speed_gpu()` method.
    /// - Returns `0` as a fallback when no data is available.
    pub fn clock_speed_gpu(&self) -> u64 {
        self.gpu
            .get(self.active_gpu)
            .map_or(0, |gpu| gpu.clock_speed_gpu())
    }

    /// Returns the formatted current clock speed of the active GPU.
    ///
    /// Retrieves the current clock speed of the active GPU as an optional value, returning `None` if no active GPU or data is available.
    ///
    /// # Returns
    /// - `Option<u64>`: The current clock speed in MHz if available, or `None` if no GPU or data is detected.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// if let Some(clock) = manager.format_clock_speed_gpu() {
    ///     println!("GPU Clock Speed: {} MHz", clock);
    /// }
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_clock_speed_gpu()` method.
    /// - Uses `and_then` to propagate `None` if the GPU or its clock speed data is unavailable.
    pub fn format_clock_speed_gpu(&self) -> Option<u64> {
        self.gpu
            .get(self.active_gpu)
            .and_then(|gpu| gpu.format_clock_speed_gpu())
    }

    /// Returns the maximum clock speed of the active GPU in MHz.
    ///
    /// Retrieves the maximum supported clock speed of the active GPU, falling back to `0` if no active GPU is available.
    ///
    /// # Returns
    /// - `u64`: The maximum clock speed in MHz, or `0` if no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let max_clock = manager.max_clock_speed();
    /// println!("GPU Max Clock Speed: {} MHz", max_clock);
    /// ```
    ///
    /// # Notes
    /// - This method delegates to the active GPU’s `max_clock_speed()` method.
    /// - Returns `0` as a fallback when no data is available.
    pub fn max_clock_speed(&self) -> u64 {
        self.gpu
            .get(self.active_gpu)
            .map_or(0, |gpu| gpu.max_clock_speed())
    }

    /// Returns the formatted maximum clock speed of the active GPU.
    ///
    /// Retrieves the maximum clock speed of the active GPU as an optional value, returning `None` if no active GPU or data is available.
    ///
    /// # Returns
    /// - `Option<u64>`: The maximum clock speed in MHz if available, or `None` if no GPU or data is detected.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// if let Some(max_clock) = manager.format_max_clock_speed_gpu() {
    ///     println!("GPU Max Clock Speed: {} MHz", max_clock);
    /// }
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_max_clock_speed_gpu()` method.
    /// - Uses `and_then` to propagate `None` if the GPU or its max clock speed data is unavailable.
    pub fn format_max_clock_speed_gpu(&self) -> Option<u64> {
        self.gpu
            .get(self.active_gpu)
            .and_then(|gpu| gpu.format_max_clock_speed_gpu())
    }

    /// Returns the formatted clock speeds (current/max) of the active GPU.
    ///
    /// Provides a human-readable string of the active GPU’s current and maximum clock speeds, falling back to "Clock Speed: N/A" if no active GPU is available.
    ///
    /// # Returns
    /// - `String`: A formatted string like "Clock Speed: CURRENT/MAX MHz" or "Clock Speed: N/A".
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let clock_str = manager.format_get_clock_speed_gpu();
    /// println!("{}", clock_str);
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_get_clock_speed_gpu()` method.
    /// - If the `active_gpu` index is invalid or no GPUs are detected, it returns "Clock Speed: N/A".
    pub fn format_get_clock_speed_gpu(&self) -> String {
        self.gpu
            .get(self.active_gpu)
            .map_or("Clock Speed: N/A".to_string(), |gpu| {
                gpu.format_get_clock_speed_gpu()
            })
    }

    /// Returns the power usage of the active GPU in watts.
    ///
    /// Retrieves the current power consumption of the active GPU, falling back to `0.0` if no active GPU is available.
    ///
    /// # Returns
    /// - `f32`: The power usage in watts, or `0.0` if no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let power = manager.power_usage_gpu();
    /// println!("GPU Power Usage: {} W", power);
    /// ```
    ///
    /// # Notes
    /// - This method delegates to the active GPU’s `power_usage_gpu()` method.
    /// - Returns `0.0` as a fallback when no data is available.
    pub fn power_usage_gpu(&self) -> f32 {
        self.gpu
            .get(self.active_gpu)
            .map_or(0.0, |gpu| gpu.power_usage_gpu())
    }

    /// Returns the formatted power usage of the active GPU.
    ///
    /// Provides a human-readable string of the active GPU’s current power usage, falling back to "N/A" if no active GPU is available.
    ///
    /// # Returns
    /// - `String`: A formatted string like "XX.XX" (watts with two decimal places) or "N/A".
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let power_str = manager.format_power_usage_gpu();
    /// println!("GPU Power: {}", power_str);
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_power_usage_gpu()` method.
    /// - If the `active_gpu` index is invalid or no GPUs are detected, it returns "N/A".
    pub fn format_power_usage_gpu(&self) -> String {
        self.gpu
            .get(self.active_gpu)
            .map_or("N/A".to_string(), |gpu| gpu.format_power_usage_gpu())
    }

    /// Returns the formatted maximum power usage of the active GPU.
    ///
    /// Retrieves the maximum power usage of the active GPU as an optional value, returning `None` if no active GPU or data is available.
    ///
    /// # Returns
    /// - `Option<f32>`: The maximum power usage in watts if available, or `None` if no GPU or data is detected.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// if let Some(max_power) = manager.format_max_power_usage_gpu() {
    ///     println!("GPU Max Power Usage: {} W", max_power);
    /// }
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_max_power_usage_gpu()` method.
    /// - Uses `and_then` to propagate `None` if the GPU or its max power usage data is unavailable.
    pub fn format_max_power_usage_gpu(&self) -> Option<f32> {
        self.gpu
            .get(self.active_gpu)
            .and_then(|gpu| gpu.format_max_power_usage_gpu())
    }

    /// Returns the formatted power usage (current/max) of the active GPU.
    ///
    /// Provides a human-readable string of the active GPU’s current and maximum power usage, falling back to "Power Usage: N/A" if no active GPU is available.
    ///
    /// # Returns
    /// - `String`: A formatted string like "Power Usage: CURRENT/MAX W" or "Power Usage: N/A".
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let power_str = manager.format_get_power_usage_gpu();
    /// println!("{}", power_str);
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_get_power_usage_gpu()` method.
    /// - If the `active_gpu` index is invalid or no GPUs are detected, it returns "Power Usage: N/A".
    pub fn format_get_power_usage_gpu(&self) -> String {
        self.gpu
            .get(self.active_gpu)
            .map_or("Power Usage: N/A".to_string(), |gpu| {
                gpu.format_get_power_usage_gpu()
            })
    }

    /// Returns whether the active GPU is currently active.
    ///
    /// Checks if the currently active GPU is in an active state, falling back to `false` if no active GPU is available.
    ///
    /// # Returns
    /// - `bool`: `true` if the GPU is active, `false` if inactive or no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let is_active = manager.bool_is_active_gpu();
    /// println!("GPU Active: {}", is_active);
    /// ```
    ///
    /// # Notes
    /// - This method delegates to the active GPU’s `bool_is_active_gpu()` method.
    /// - Returns `false` as a fallback when no data is available.
    pub fn bool_is_active_gpu(&self) -> bool {
        self.gpu
            .get(self.active_gpu)
            .map_or(false, |gpu| gpu.bool_is_active_gpu())
    }

    /// Returns the formatted active state of the active GPU.
    ///
    /// Provides a human-readable string indicating whether the active GPU is active, falling back to "Inactive" if no active GPU is available.
    ///
    /// # Returns
    /// - `String`: "Active" if the GPU is active, "Inactive" if inactive or no GPU is detected at the active index.
    ///
    /// # Examples
    /// ```rust
    /// use gpu_info::GpuManager;
    /// let mut manager = GpuManager::init();
    /// manager.refresh();
    /// let active_str = manager.format_is_active_gpu();
    /// println!("GPU State: {}", active_str);
    /// ```
    ///
    /// # Notes
    /// - Delegates to the active GPU’s `format_is_active_gpu()` method.
    /// - If the `active_gpu` index is invalid or no GPUs are detected, it returns "Inactive".
    pub fn format_is_active_gpu(&self) -> String {
        self.gpu
            .get(self.active_gpu)
            .map_or("Inactive".to_string(), |gpu| gpu.format_is_active_gpu())
    }
}
