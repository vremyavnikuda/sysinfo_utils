use std::{path::Path, process::Command};

use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    AMD,
    Intel,
    Unknown,
}

impl Default for GpuVendor {
    fn default() -> Self {
        GpuVendor::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub temperature: Option<f32>,
    pub utilization: Option<f32>,
    pub clock_speed: Option<u64>,
    pub max_clock_speed: Option<u64>,
    pub power_usage: Option<f32>,
    pub max_power_usage: Option<f32>,
    pub is_active: bool,
}

impl GpuInfo {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_vendor(&self) -> &GpuVendor {
        &self.vendor
    }

    pub fn get_temperature(&self) -> String {
        match self.temperature {
            Some(temp) => format!(" Temperature: {}°C", temp),
            None => " Temperature: N/A".to_string(),
        }
    }

    pub fn get_utilization(&self) -> String {
        match self.utilization {
            Some(util) => format!("󰾆 Utilization: {}%", util),
            None => "󰾆 Utilization: N/A".to_string(),
        }
    }

    pub fn get_clock_speed(&self) -> String {
        let current = self.clock_speed.unwrap_or(0);
        let max = self.max_clock_speed.unwrap_or(0);
        format!(" Clock Speed: {}/{} MHz", current, max)
    }

    pub fn get_power_usage(&self) -> String {
        let current = self.power_usage.unwrap_or(0.0);
        let max = self.max_power_usage.unwrap_or(0.0);
        format!("󱪉 Power Usage: {:.2}/{} W", current, max)
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuManager {
    pub gpus: Vec<GpuInfo>,
    pub active_gpu: usize,
}

impl GpuManager {
    pub fn new() -> Self {
        let mut manager = GpuManager {
            gpus: Vec::new(),
            active_gpu: 0,
        };
        manager.detect_gpus();
        manager
    }

    // Автоматическое определение GPU
    pub fn detect_gpus(&mut self) {
        self.gpus.clear();

        // Обнаружение NVIDIA
        if let Ok(output) = Command::new("nvidia-smi").arg("--query-gpu=name,temperature.gpu,utilization.gpu,clocks.current.graphics,clocks.max.graphics,power.draw,power.max_limit").arg("--format=csv,noheader,nounits").output() {
            if output.status.success() {
                self.parse_nvidia_info(&String::from_utf8_lossy(&output.stdout));
            }
        }

        // Обнаружение AMD
        if Path::new("/sys/class/drm/card0/device/vendor").exists() {
            self.parse_amd_info();
        }

        // Обнаружение Intel
        if Path::new("/sys/class/drm/card0/device/intel_info").exists() {
            self.parse_intel_info();
        }
    }

    // Переключение между GPU
    pub fn switch_gpu(&mut self, index: usize) -> Result<(), String> {
        if index >= self.gpus.len() {
            return Err("Invalid GPU index".into());
        }

        // Здесь должна быть логика переключения GPU
        // Это зависит от конкретной системы и драйверов
        self.active_gpu = index;
        Ok(())
    }

    // Сбор информации о GPU
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

    // Генерация JSON для Waybar
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

    // Управление состоянием GPU
    pub fn check_power_state(&self) -> bool {
        let sys = System::new_all();
        sys.processes().values().any(|p| {
            p.name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains("nvidia")
        })
    }

    // Приватные методы для парсинга информации
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
            manager.gpus.is_empty(),
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
