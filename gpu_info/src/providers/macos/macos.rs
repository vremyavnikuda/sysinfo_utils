//! macOS GPU provider implementation
//! 
//! This module implements the GpuProvider trait for GPUs on macOS using system utilities.

use crate::gpu_info::{GpuInfo, Result, GpuProvider};
use crate::vendor::Vendor;
use log::{debug, info, warn};
use std::process::Command;

/// GPU provider for macOS
pub struct MacosProvider;

impl MacosProvider {
    pub fn new() -> Self {
        Self
    }
    
    /// Получает список всех GPU в системе macOS
    fn detect_all_gpus(&self) -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        gpus.extend(self.get_basic_gpu_info());
        self.enhance_with_additional_info(&mut gpus);
        if gpus.is_empty() {
            if let Some(apple_gpu) = self.detect_apple_silicon_gpu() {
                gpus.push(apple_gpu);
            }
        }
        
        gpus
    }
    
    /// Базовая информация через system_profiler
    fn get_basic_gpu_info(&self) -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        let output = Command::new("system_profiler")
            .args(["-xml", "SPDisplaysDataType"])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            gpus.extend(self.parse_system_profiler_output(&stdout));
        } else {
            warn!("Failed to invoke system_profiler for GPU info.");
        }
        
        gpus
    }
    
    /// Парсинг вывода system_profiler
    fn parse_system_profiler_output(&self, xml_output: &str) -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        let lines: Vec<&str> = xml_output.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.contains("<dict>") {
                if let Some(gpu) = self.parse_gpu_dict(&lines, &mut i) {
                    gpus.push(gpu);
                }
            }
            i += 1;
        }
        if gpus.is_empty() {
            gpus.extend(self.simple_parse_system_profiler(xml_output));
        }
        
        info!("Found {} GPU(s) via system_profiler", gpus.len());
        gpus
    }
    
    /// Парсинг словаря GPU из XML
    fn parse_gpu_dict(&self, lines: &[&str], index: &mut usize) -> Option<GpuInfo> {
        let mut gpu = GpuInfo::unknown();
        let mut found_gpu_info = false;
        while *index < lines.len() {
            let line = lines[*index].trim();
            if line.contains("</dict>") {
                break;
            }
            if line.contains("<key>sppci_model</key>") && *index + 1 < lines.len() {
                let next_line = lines[*index + 1].trim();
                if let Some(name) = self.extract_string_value(next_line) {
                    gpu.name_gpu = Some(name.clone());
                    gpu.vendor = self.determine_vendor(&name);
                    gpu.active = Some(true);
                    gpu.memory_total = self.extract_vram_from_name(&name);
                    found_gpu_info = true;
                }
            }
            if line.contains("<key>sppci_vram</key>") && *index + 1 < lines.len() {
                let next_line = lines[*index + 1].trim();
                if let Some(vram_str) = self.extract_string_value(next_line) {
                    gpu.memory_total = self.parse_vram_string(&vram_str);
                }
            }
            if line.contains("<key>sppci_bus_speed</key>") && *index + 1 < lines.len() {
                let next_line = lines[*index + 1].trim();
                if let Some(speed_str) = self.extract_string_value(next_line) {
                    gpu.core_clock = self.parse_clock_speed(&speed_str);
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
    
    /// Извлечение строкового значения из XML тега
    fn extract_string_value(&self, line: &str) -> Option<String> {
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
    
    /// Простой парсинг для обратной совместимости
    fn simple_parse_system_profiler(&self, xml_output: &str) -> Vec<GpuInfo> {
        let mut gpus = Vec::new();
        let mut current_gpu = GpuInfo::unknown();
        let mut in_display_section = false;
        for line in xml_output.lines() {
            let line = line.trim();
            if line.contains("<key>sppci_model</key>") {
                in_display_section = true;
                continue;
            }
            if in_display_section {
                if line.starts_with("<string>") && line.ends_with("</string>") {
                    let name = line
                        .trim_start_matches("<string>")
                        .trim_end_matches("</string>")
                        .to_string();
                    
                    current_gpu.name_gpu = Some(name.clone());
                    current_gpu.vendor = self.determine_vendor(&name);
                    current_gpu.active = Some(true);
                    current_gpu.memory_total = self.extract_vram_from_name(&name);
                    gpus.push(current_gpu.clone());
                    current_gpu = GpuInfo::unknown();
                    in_display_section = false;
                }
            }
        }
        
        gpus
    }
    
    /// Определение производителя по имени GPU
    fn determine_vendor(&self, name: &str) -> Vendor {
        crate::vendor::determine_vendor_from_name(name)
    }
    
    /// Извлечение информации о VRAM из названия GPU
    fn extract_vram_from_name(&self, name: &str) -> Option<u32> {
        let words: Vec<&str> = name.split_whitespace().collect();
        for word in words {
            if word.ends_with("GB") || word.ends_with("gb") {
                if let Ok(size) = word.trim_end_matches("GB").trim_end_matches("gb").parse::<u32>() {
                    return Some(size);
                }
            }
        }
        None
    }
    
    /// Парсинг строки VRAM (например, "4096 MB", "8 GB")
    fn parse_vram_string(&self, vram_str: &str) -> Option<u32> {
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
    
    /// Парсинг частоты (например, "1200 MHz", "2.5 GHz")
    fn parse_clock_speed(&self, speed_str: &str) -> Option<u32> {
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
                return Some((ghz_value * 1000.0) as u32); // Конвертация GHz в MHz
            }
        }
        
        None
    }
    
    /// Детекция Apple Silicon GPU
    fn detect_apple_silicon_gpu(&self) -> Option<GpuInfo> {
        let cpu_output = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output();
        let arch_output = Command::new("uname")
            .args(["-m"])
            .output();
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
            let (gpu_name, _gpu_cores) = self.determine_apple_gpu_info(&cpu_info);
            return Some(GpuInfo {
                vendor: Vendor::Apple,
                name_gpu: Some(gpu_name),
                active: Some(true),
                memory_total: self.get_unified_memory_size(),
                core_clock: self.estimate_apple_gpu_clock(&cpu_info),
                utilization: self.get_apple_gpu_utilization(),
                temperature: self.get_apple_gpu_temperature(),
                ..Default::default()
            });
        }
        None
    }
    
    /// Определение информации о Apple GPU
    fn determine_apple_gpu_info(&self, cpu_info: &str) -> (String, Option<u32>) {
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
    
    /// Оценка частоты Apple GPU
    fn estimate_apple_gpu_clock(&self, cpu_info: &str) -> Option<u32> {
        // Оценка основанная на типе чипа
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
    
    /// Получение использования Apple GPU
    fn get_apple_gpu_utilization(&self) -> Option<f32> {
        // Попытаемся получить через powermetrics (если доступно)
        let output = Command::new("powermetrics")
            .args(["-n", "1", "-s", "gpu_power", "--show-initial-usage"])
            .output();
            
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(usage) = self.parse_gpu_usage_from_powermetrics(&stdout) {
                return Some(usage);
            }
        }
        None
    }
    
    /// Получение температуры Apple GPU
    fn get_apple_gpu_temperature(&self) -> Option<f32> {
        let output = Command::new("sysctl")
            .args(["-a"])
            .output();
            
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("machdep.xcpm.gpu_thermal") || line.contains("thermal.gpu") {
                    if let Some(temp) = self.parse_temperature_from_sysctl(line) {
                        return Some(temp);
                    }
                }
            }
        }
        None
    }
    
    /// Парсинг использования GPU из powermetrics
    fn parse_gpu_usage_from_powermetrics(&self, output: &str) -> Option<f32> {
        for line in output.lines() {
            if line.contains("GPU") && line.contains("%") {
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
    
    /// Парсинг температуры из sysctl
    fn parse_temperature_from_sysctl(&self, line: &str) -> Option<f32> {
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
    
    /// Получение размера unified memory для Apple Silicon
    fn get_unified_memory_size(&self) -> Option<u32> {
        let output = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output();
            
        if let Ok(out) = output {
            let mem_str = String::from_utf8_lossy(&out.stdout);
            if let Ok(mem_bytes) = mem_str.trim().parse::<u64>() {
                return Some((mem_bytes / 1024 / 1024 / 1024) as u32);
            }
        }
        None
    }
    
    /// Расширение информации через дополнительные системные вызовы
    fn enhance_with_additional_info(&self, gpus: &mut Vec<GpuInfo>) {
        debug!("Attempting to enhance GPU information via additional system calls");
        for gpu in gpus.iter_mut() {
            if gpu.utilization.is_none() {
                gpu.utilization = self.get_gpu_utilization_estimate();
            }
            if gpu.active.is_none() {
                gpu.active = Some(true);
            }
        }
    }
    
    /// Примерная оценка загрузки GPU
    fn get_gpu_utilization_estimate(&self) -> Option<f32> {
        let output = Command::new("vm_stat")
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if stdout.contains("Pages") {
                return Some(5.0); 
            }
        }
        None
    }
}

impl Default for MacosProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuProvider for MacosProvider {
    /// Detect all GPUs on macOS
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
        debug!("Detecting GPUs on macOS");
        Ok(self.detect_all_gpus())
    }
    
    /// Update GPU information on macOS
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        debug!("Updating macOS GPU information for {:?}", gpu.name_gpu);
        gpu.active = Some(true);
        if matches!(gpu.vendor, Vendor::Apple) {
            if gpu.utilization.is_none() {
                gpu.utilization = self.get_apple_gpu_utilization();
            }
            if gpu.temperature.is_none() {
                gpu.temperature = self.get_apple_gpu_temperature();
            }
        } else {
            if gpu.utilization.is_none() {
                gpu.utilization = self.get_gpu_utilization_estimate();
            }
        }
        Ok(())
    }
    
    /// Get the vendor for this provider
    fn get_vendor(&self) -> Vendor {
        // This is a generic provider that can detect multiple vendors
        Vendor::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vendor::Vendor;
    
    #[test]
    fn test_macos_provider_vendor() {
        let provider = MacosProvider::new();
        // For macOS provider, we return Unknown as it can detect multiple vendors
        assert_eq!(provider.get_vendor(), Vendor::Unknown);
    }
}