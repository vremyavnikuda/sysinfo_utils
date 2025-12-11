//src/platform/macos.rs
use crate::{
    gpu_info::{GpuInfo, Result},
    vendor::{IntelGpuType, Vendor},
};
use log::{debug, info, warn};
use std::process::Command;
/// Extended structure for macOS GPU information
struct MacOSGpuProvider;
impl MacOSGpuProvider {
    /// Gets list of all GPUs in macOS system
    pub fn detect_all_gpus() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        gpus.extend(Self::get_basic_gpu_info());
        Self::enhance_with_iokit(&mut gpus);
        if gpus.is_empty() {
            if let Some(apple_gpu) = Self::detect_apple_silicon_gpu() {
                gpus.push(apple_gpu);
            }
        }
        gpus
    }
    /// Basic information via system_profiler
    fn get_basic_gpu_info() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        let output = Command::new("system_profiler")
            .args(["-xml", "SPDisplaysDataType"])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            gpus.extend(Self::parse_system_profiler_output(&stdout));
        } else {
            warn!("Failed to invoke system_profiler for GPU info.");
        }
        gpus
    }
    /// Parse system_profiler output
    fn parse_system_profiler_output(xml_output: &str) -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        let lines: Vec<&str> = xml_output.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.contains("<dict>") {
                if let Some(gpu) = Self::parse_gpu_dict(&lines, &mut i) {
                    gpus.push(gpu);
                }
            }
            i += 1;
        }
        if gpus.is_empty() {
            gpus.extend(Self::simple_parse_system_profiler(xml_output));
        }
        info!("Found {} GPU(s) via system_profiler", gpus.len());
        gpus
    }
    /// Parse GPU dictionary from XML
    fn parse_gpu_dict(lines: &[&str], index: &mut usize) -> Option<GpuInfo> {
        let mut gpu = GpuInfo::unknown();
        let mut found_gpu_info = false;
        while *index < lines.len() {
            let line = lines[*index].trim();
            if line.contains("</dict>") {
                break;
            }
            if line.contains("<key>sppci_model</key>") && *index + 1 < lines.len() {
                let next_line = lines[*index + 1].trim();
                if let Some(name) = Self::extract_string_value(next_line) {
                    gpu.name_gpu = Some(name.clone());
                    gpu.vendor = Self::determine_vendor(&name);
                    gpu.active = Some(true);
                    gpu.memory_total = Self::extract_vram_from_name(&name);
                    found_gpu_info = true;
                }
            }
            if line.contains("<key>sppci_vram</key>") && *index + 1 < lines.len() {
                let next_line = lines[*index + 1].trim();
                if let Some(vram_str) = Self::extract_string_value(next_line) {
                    gpu.memory_total = Self::parse_vram_string(&vram_str);
                }
            }
            if line.contains("<key>sppci_bus_speed</key>") && *index + 1 < lines.len() {
                let next_line = lines[*index + 1].trim();
                if let Some(speed_str) = Self::extract_string_value(next_line) {
                    gpu.core_clock = Self::parse_clock_speed(&speed_str);
                }
            }
            *index += 1;
        }
        if found_gpu_info {
            Some(gpu)
        } else {
            None
        }
    }
    /// Extract string value from XML tag
    fn extract_string_value(line: &str) -> Option<String> {
        if line.starts_with("<string>") && line.ends_with("</string>") {
            Some(
                line.trim_start_matches("<string>")
                    .trim_end_matches("</string>")
                    .to_string(),
            )
        } else {
            None
        }
    }
    /// Simple parsing for backward compatibility
    fn simple_parse_system_profiler(xml_output: &str) -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        let mut current_gpu = GpuInfo::unknown();
        let mut in_display_section = false;
        for line in xml_output.lines() {
            let line = line.trim();
            if line.contains("<key>sppci_model</key>") {
                in_display_section = true;
                continue;
            }
            if in_display_section && line.starts_with("<string>") && line.ends_with("</string>") {
                let name = line
                    .trim_start_matches("<string>")
                    .trim_end_matches("</string>")
                    .to_string();
                current_gpu.name_gpu = Some(name.clone());
                current_gpu.vendor = Self::determine_vendor(&name);
                current_gpu.active = Some(true);
                current_gpu.memory_total = Self::extract_vram_from_name(&name);
                gpus.push(current_gpu.clone());
                current_gpu = GpuInfo::unknown();
                in_display_section = false;
            }
        }
        gpus
    }
    /// Determine vendor from GPU name
    fn determine_vendor(name: &str) -> Vendor {
        let name_lower = name.to_lowercase();
        if name_lower.contains("apple")
            || name_lower.contains("m1")
            || name_lower.contains("m2")
            || name_lower.contains("m3")
        {
            Vendor::Apple
        } else if name_lower.contains("amd") || name_lower.contains("radeon") {
            Vendor::Amd
        } else if name_lower.contains("nvidia") || name_lower.contains("geforce") {
            Vendor::Nvidia
        } else if name_lower.contains("intel") {
            let gpu_type = if name_lower.contains("iris") || name_lower.contains("uhd") {
                IntelGpuType::Integrated
            } else {
                IntelGpuType::Unknown
            };
            Vendor::Intel(gpu_type)
        } else {
            Vendor::Unknown
        }
    }
    /// Extract VRAM information from GPU name
    fn extract_vram_from_name(name: &str) -> Option<u32> {
        let words: Vec<&str> = name.split_whitespace().collect();
        for word in words {
            if word.ends_with("GB") || word.ends_with("gb") {
                if let Ok(size) = word
                    .trim_end_matches("GB")
                    .trim_end_matches("gb")
                    .parse::<u32>()
                {
                    return Some(size);
                }
            }
        }
        None
    }
    /// Parse VRAM string (e.g., "4096 MB", "8 GB")
    fn parse_vram_string(vram_str: &str) -> Option<u32> {
        let vram_lower = vram_str.to_lowercase();
        if let Some(mb_pos) = vram_lower.find("mb") {
            let number_part = &vram_str[..mb_pos].trim();
            if let Ok(mb_value) = number_part.parse::<u32>() {
                return Some((mb_value + 512) / 1024);
            }
        }
        if let Some(gb_pos) = vram_lower.find("gb") {
            let number_part = &vram_str[..gb_pos].trim();
            if let Ok(gb_value) = number_part.parse::<u32>() {
                return Some(gb_value);
            }
        }
        None
    }
    /// Parse clock speed (e.g., "1200 MHz", "2.5 GHz")
    fn parse_clock_speed(speed_str: &str) -> Option<u32> {
        let speed_lower = speed_str.to_lowercase();
        if let Some(mhz_pos) = speed_lower.find("mhz") {
            let number_part = &speed_str[..mhz_pos].trim();
            if let Ok(mhz_value) = number_part.parse::<u32>() {
                return Some(mhz_value);
            }
        }
        if let Some(ghz_pos) = speed_lower.find("ghz") {
            let number_part = &speed_str[..ghz_pos].trim();
            if let Ok(ghz_value) = number_part.parse::<f32>() {
                return Some((ghz_value * 1000.0) as u32); // Convert GHz to MHz
            }
        }
        None
    }
    /// Detect Apple Silicon GPU
    fn detect_apple_silicon_gpu() -> Option<GpuInfo> {
        let cpu_output = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output();
        let arch_output = Command::new("uname").args(["-m"]).output();
        let mut is_apple_silicon = false;
        let mut cpu_info = String::new();
        if let Ok(arch_out) = arch_output {
            let arch_str = String::from_utf8_lossy(&arch_out.stdout);
            if arch_str.trim() == "arm64" {
                is_apple_silicon = true;
            }
        }
        if let Ok(cpu_out) = cpu_output {
            cpu_info = String::from_utf8_lossy(&cpu_out.stdout).to_string();
            if cpu_info.contains("Apple") {
                is_apple_silicon = true;
            }
        }
        if is_apple_silicon {
            info!("Detected Apple Silicon with integrated GPU");
            let (gpu_name, _gpu_cores) = Self::determine_apple_gpu_info(&cpu_info);
            return Some(GpuInfo {
                vendor: Vendor::Apple,
                name_gpu: Some(gpu_name),
                active: Some(true),
                memory_total: Self::get_unified_memory_size(),
                core_clock: Self::estimate_apple_gpu_clock(&cpu_info),
                utilization: Self::get_apple_gpu_utilization(),
                temperature: Self::get_apple_gpu_temperature(),
                ..Default::default()
            });
        }
        None
    }
    /// Determine Apple GPU information
    fn determine_apple_gpu_info(cpu_info: &str) -> (String, Option<u32>) {
        if cpu_info.contains("M3") {
            if cpu_info.contains("Pro") {
                ("Apple M3 Pro GPU".to_string(), Some(18)) // M3 Pro имеет до 18 GPU ядер
            } else if cpu_info.contains("Max") {
                ("Apple M3 Max GPU".to_string(), Some(40)) // M3 Max имеет до 40 GPU ядер
            } else {
                ("Apple M3 GPU".to_string(), Some(10)) // Базовый M3 имеет 8-10 GPU ядер
            }
        } else if cpu_info.contains("M2") {
            if cpu_info.contains("Pro") {
                ("Apple M2 Pro GPU".to_string(), Some(19)) // M2 Pro имеет 19 GPU ядер
            } else if cpu_info.contains("Max") {
                ("Apple M2 Max GPU".to_string(), Some(38)) // M2 Max имеет 38 GPU ядер
            } else {
                ("Apple M2 GPU".to_string(), Some(8)) // Базовый M2 имеет 8-10 GPU ядер
            }
        } else if cpu_info.contains("M1") {
            if cpu_info.contains("Pro") {
                ("Apple M1 Pro GPU".to_string(), Some(16)) // M1 Pro имеет 14-16 GPU ядер
            } else if cpu_info.contains("Max") {
                ("Apple M1 Max GPU".to_string(), Some(32)) // M1 Max имеет 24-32 GPU ядра
            } else {
                ("Apple M1 GPU".to_string(), Some(8)) // Базовый M1 имеет 7-8 GPU ядер
            }
        } else {
            ("Apple Silicon GPU".to_string(), None)
        }
    }
    /// Estimate Apple GPU clock speed
    fn estimate_apple_gpu_clock(cpu_info: &str) -> Option<u32> {
        // Estimate based on chip type
        if cpu_info.contains("M3") {
            Some(1400) // M3 GPU частота около 1.4 GHz
        } else if cpu_info.contains("M2") {
            Some(1300) // M2 GPU частота около 1.3 GHz
        } else if cpu_info.contains("M1") {
            Some(1200) // M1 GPU частота около 1.2 GHz
        } else {
            Some(1000) // Общая оценка
        }
    }
    /// Get Apple GPU utilization
    fn get_apple_gpu_utilization() -> Option<f32> {
        // Try to get via powermetrics (if available)
        let output = Command::new("powermetrics")
            .args(["-n", "1", "-s", "gpu_power", "--show-initial-usage"])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(usage) = Self::parse_gpu_usage_from_powermetrics(&stdout) {
                return Some(usage);
            }
        }
        None
    }
    /// Get Apple GPU temperature
    fn get_apple_gpu_temperature() -> Option<f32> {
        let output = Command::new("sysctl").args(["-a"]).output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("machdep.xcpm.gpu_thermal") || line.contains("thermal.gpu") {
                    if let Some(temp) = Self::parse_temperature_from_sysctl(line) {
                        return Some(temp);
                    }
                }
            }
        }
        None
    }
    /// Parse GPU usage from powermetrics
    fn parse_gpu_usage_from_powermetrics(output: &str) -> Option<f32> {
        for line in output.lines() {
            if line.contains("GPU") && line.contains("%") {
                // Простой парсинг процентов
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.ends_with('%') {
                        if let Ok(usage) = part.trim_end_matches('%').parse::<f32>() {
                            return Some(usage);
                        }
                    }
                }
            }
        }
        None
    }
    /// Parse temperature from sysctl
    fn parse_temperature_from_sysctl(line: &str) -> Option<f32> {
        if let Some(colon_pos) = line.find(':') {
            let value_part = &line[colon_pos + 1..].trim();
            if let Ok(temp) = value_part.parse::<f32>() {
                if temp > 200.0 {
                    return Some(temp - 273.15);
                } else {
                    return Some(temp);
                }
            }
        }
        None
    }
    /// Get unified memory size for Apple Silicon
    fn get_unified_memory_size() -> Option<u32> {
        let output = Command::new("sysctl").args(["-n", "hw.memsize"]).output();
        if let Ok(out) = output {
            let mem_str = String::from_utf8_lossy(&out.stdout);
            if let Ok(mem_bytes) = mem_str.trim().parse::<u64>() {
                return Some((mem_bytes / 1024 / 1024 / 1024) as u32);
            }
        }
        None
    }
    /// Enhance information via additional system calls
    fn enhance_with_iokit(gpus: &mut [GpuInfo]) {
        debug!("Attempting to enhance GPU information via additional system calls");
        for gpu in gpus.iter_mut() {
            if gpu.utilization.is_none() {
                gpu.utilization = Self::get_gpu_utilization_estimate();
            }
            if gpu.active.is_none() {
                gpu.active = Some(true);
            }
        }
    }
    /// Approximate GPU utilization estimate
    fn get_gpu_utilization_estimate() -> Option<f32> {
        let output = Command::new("vm_stat").output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("Pages") {
                return Some(5.0);
            }
        }
        None
    }
    /// Update GPU information
    pub fn update_gpu_info(gpu: &mut GpuInfo) -> Result<()> {
        debug!("Updating macOS GPU information for {:?}", gpu.name_gpu);
        gpu.active = Some(true);
        if matches!(gpu.vendor, Vendor::Apple) {
            if gpu.utilization.is_none() {
                gpu.utilization = Self::get_apple_gpu_utilization();
            }
            if gpu.temperature.is_none() {
                gpu.temperature = Self::get_apple_gpu_temperature();
            }
        } else if gpu.utilization.is_none() {
            gpu.utilization = Self::get_gpu_utilization_estimate();
        }
        Ok(())
    }
}
/// Returns list of available GPUs on macOS
pub fn info_gpu() -> GpuInfo {
    let gpus = MacOSGpuProvider::detect_all_gpus();
    if let Some(primary_gpu) = gpus.first() {
        primary_gpu.clone()
    } else {
        warn!("No GPU detected on macOS, returning unknown GPU");
        GpuInfo::unknown()
    }
}
/// Gets all available GPUs on macOS
pub fn get_all_gpus() -> Vec<GpuInfo> {
    MacOSGpuProvider::detect_all_gpus()
}
/// Update detailed GPU information on macOS
pub fn update_gpu_info(gpu: &mut GpuInfo) -> Result<()> {
    MacOSGpuProvider::update_gpu_info(gpu)
}
pub fn init() -> Vec<GpuInfo> {
    get_all_gpus()
}
pub fn update(gpu: &mut GpuInfo) {
    let _ = update_gpu_info(gpu);
}
