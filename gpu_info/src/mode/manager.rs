//! GPU management and detection implementation

use crate::mode::gpu::{GpuInfo, GpuVendor};
use log::{ warn};
use serde::{Deserialize, Serialize};
use serde_json;
use std::{
    fs,
    process::Command,
};
use sysinfo::System;

/// A central controller for GPU detection and management
///
/// Handles GPU detection, status monitoring, and system integration.
///
/// # Lifecycle
/// 1. Initialize with `new()`
/// 2. Detect GPUs with `detect_gpus()`
/// 3. Refresh metrics with `refresh()`.
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
    /// — NVIDIA: Uses `nvidia-smi` CLI tool
    /// — AMD: Parses sysfs files
    /// — Intel: Checks specific sysfs paths.
    ///
    /// # Platform Notes
    /// — Requires root permissions for some sysfs paths
    /// — NVIDIA detection depends on `nvidia-smi` availability.
    ///
    /// # Panics
    /// May panic if system calls fail (platform-dependent)
    ///
    // TODO: - detect_gpus()
    // TODO: - Убедиться, что sysfs доступен для AMD и Intel
    // TODO: - Проверить доступность `nvidia-smi` в системе
    // TODO: - Убедиться, что sysfs корректно определяет производителя GPU
    /// Обнаружение всех установленных GPU
	pub fn detect_gpus(&mut self) {
		self.gpus.clear();

		// NVIDIA (nvidia-smi)
		if let Ok(output) = Command::new("nvidia-smi")
			.arg("--query-gpu=name,temperature.gpu,utilization.gpu,clocks.current.graphics,clocks.max.graphics,power.draw,power.max_limit")
			.arg("--format=csv,noheader,nounits")
			.output()
		{
			if output.status.success() {
				self.parse_nvidia_info(&String::from_utf8_lossy(&output.stdout));
			}
		}

		// AMD (sysfs)
		if let Ok(entries) = fs::read_dir("/sys/class/drm/") {
			for entry in entries.flatten() {
				let path = entry.path();
				if path.join("device/vendor").exists() {
					if let Ok(vendor) = fs::read_to_string(path.join("device/vendor")) {
						if vendor.trim() == "0x1002" { // AMD Vendor ID
							self.parse_amd_info();
						}
					}
				}
			}
		}

		// Intel (sysfs)
		if let Ok(entries) = fs::read_dir("/sys/class/drm/") {
			for entry in entries.flatten() {
				let path = entry.path();
				if path.join("device/subsystem_vendor").exists() {
					if let Ok(vendor) = fs::read_to_string(path.join("device/subsystem_vendor")) {
						if vendor.trim() == "0x8086" { // Intel Vendor ID
							self.parse_intel_info();
						}
					}
				}
			}
		}
	}

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
    /// let mut manager = GpuManager::new();
    /// assert!(manager.switch_gpu(0).is_ok());
    /// ```
    // TODO: пока что просто меняет индекс активного GPU (нуждается в доработке)
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
    /// — NVIDIA: Real-time (~1 sec latency)
    /// — AMD/Intel: Depends on sysfs update frequency.
    ///
    /// # Platform Notes
    /// May block on system calls during execution
    // TODO: проверить работоспособность в системах Linux,Windows,Mac OS
    pub fn refresh(&mut self) {
        for gpu in self.gpus.iter_mut() {
            match gpu.vendor {
                GpuVendor::Nvidia => GpuManager::update_nvidia_info(gpu),
                GpuVendor::AMD => GpuManager::update_amd_info(gpu),
                GpuVendor::Intel => GpuManager::update_intel_info(gpu),
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
        let active_gpu = &self.gpus[self.active_gpu];

        if self.gpus.is_empty() {
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

    /// Checks NVIDIA power management state
    ///
    /// # Returns
    /// `true` if any NVIDIA processes are running.
    ///
    /// # Platform Support
    /// Linux-only detection
    // TODO: пока что проверяет только процессы с именем nvidia
    // TODO: не работает в системах Linux and Mac OS
    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Добавить поддержку других производителей GPU(AMD, Intel)
    pub fn check_power_state(&self) -> bool {
        let mut sys = System::new_all();
        sys.refresh_all();

        let gpu_processes = ["nvidia", "amdgpu", "radeon", "intel", "i915", "rtx"];

        sys.processes().values().any(|p| {
            let name = p.name().to_string_lossy().to_ascii_lowercase();
            gpu_processes.iter().any(|&gpu| name.contains(gpu))
        })
    }

    /// Parses a string of NVIDIA GPU information and creates a `GpuInfo` struct from it.
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
    /// * `data`: A string containing the NVIDIA GPU information.
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
        self.gpus.push(gpu);
    }

    /// (Internal) Checks if an AMD GPU is present and adds it to the list.
    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Требуется документация
    // TODO: не работает определение метрик AMD GPU
    pub fn parse_amd_info(&mut self) {
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
    pub(crate) fn update_nvidia_info(gpu: &mut GpuInfo) {
        let output = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=temperature.gpu,utilization.gpu,clocks.current.graphics,power.draw",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .expect("Failed to execute nvidia-smi command");

        let data = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = data.split(',').collect();
        gpu.temperature = parts.get(0).and_then(|s| s.trim().parse().ok());
        gpu.utilization = parts.get(1).and_then(|s| s.trim().parse().ok());
        gpu.clock_speed = parts.get(2).and_then(|s| s.trim().parse().ok());
        gpu.power_usage = parts.get(3).and_then(|s| s.trim().parse().ok());
        // TODO:Возможно имеет смысл использовать этот код
        /*
        if parts.len() >= 4 {
            gpu.temperature = parts.get(0).and_then(|s| s.trim().parse().ok());
            gpu.utilization = parts.get(1).and_then(|s| s.trim().parse().ok());
            gpu.clock_speed = parts.get(2).and_then(|s| s.trim().parse().ok());
            gpu.power_usage = parts.get(3).and_then(|s| s.trim().parse().ok());
        } else {
            warn!("nvidia-smi returned unexpected format: {:?}", parts);
        }
        */
    }

    // TODO: Требует интеграционного тестирования с реальными процессами
    // TODO: Требует обработки ошибок
    // TODO: Требует доработки для других платформ( Windows, macOS)
    // TODO: Требуется документация
    /// [upd_amd_info](Self::update_amd_info)
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
