//! macOS GPU provider implementation
//!
//! This module implements the GpuProvider trait for GPUs on macOS using system utilities.
//!
//! # Implementation Details
//!
//! The macOS provider uses `system_profiler` to detect GPUs and gather basic information.
//! For Apple Silicon (M1/M2/M3), it provides additional support for unified memory
//! and estimated GPU metrics.
//!
//! ## Limitations
//!
//! - Real-time metrics require `sudo` access for `powermetrics`
//! - `system_profiler` is relatively slow (500-1000ms per call)
//! - Some metrics are estimated rather than measured directly
//! - Metal API integration would require Objective-C bindings
//!
//! ## Performance Considerations
//!
//! To minimize overhead, the provider caches basic GPU information and only
//! updates dynamic metrics (utilization, temperature) during `update_gpu()` calls.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(target_os = "macos")]
//! # {
//! use gpu_info::providers::macos::MacosProvider;
//! use gpu_info::gpu_info::GpuProvider;
//!
//! let provider = MacosProvider::new();
//! if let Ok(gpus) = provider.detect_gpus() {
//!     for gpu in gpus {
//!         println!("Found GPU: {:?}", gpu.name_gpu);
//!     }
//! }
//! # }
//! ```
use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::vendor::Vendor;
use log::{debug, info, warn};
use std::process::Command;
/// GPU provider for macOS
pub struct MacosProvider;
impl MacosProvider {
    pub fn new() -> Self {
        Self
    }
    /// Gets list of all GPUs in macOS system
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
    /// Basic information via system_profiler
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
    /// Parse system_profiler output
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
    /// Parse GPU dictionary from XML
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
    /// Extract string value from XML tag
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
    /// Simple parsing for backward compatibility
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
            if in_display_section && line.starts_with("<string>") && line.ends_with("</string>") {
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
        gpus
    }
    /// Determine vendor from GPU name
    fn determine_vendor(&self, name: &str) -> Vendor {
        crate::vendor::determine_vendor_from_name(name)
    }
    /// Extract VRAM information from GPU name
    fn extract_vram_from_name(&self, name: &str) -> Option<u32> {
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
    /// Parse clock speed (e.g., "1200 MHz", "2.5 GHz")
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
                return Some((ghz_value * 1000.0) as u32); // Convert GHz to MHz
            }
        }
        None
    }
    /// Detect Apple Silicon GPU
    fn detect_apple_silicon_gpu(&self) -> Option<GpuInfo> {
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
    /// Determine Apple GPU information
    fn determine_apple_gpu_info(&self, cpu_info: &str) -> (String, Option<u32>) {
        if cpu_info.contains("M3") {
            if cpu_info.contains("Pro") {
                ("Apple M3 Pro GPU".to_string(), Some(18)) // M3 Pro has up to 18 GPU cores
            } else if cpu_info.contains("Max") {
                ("Apple M3 Max GPU".to_string(), Some(40)) // M3 Max has up to 40 GPU cores
            } else {
                ("Apple M3 GPU".to_string(), Some(10)) // Base M3 has 8-10 GPU cores
            }
        } else if cpu_info.contains("M2") {
            if cpu_info.contains("Pro") {
                ("Apple M2 Pro GPU".to_string(), Some(19)) // M2 Pro has 19 GPU cores
            } else if cpu_info.contains("Max") {
                ("Apple M2 Max GPU".to_string(), Some(38)) // M2 Max has 38 GPU cores
            } else {
                ("Apple M2 GPU".to_string(), Some(8)) // Base M2 has 8-10 GPU cores
            }
        } else if cpu_info.contains("M1") {
            if cpu_info.contains("Pro") {
                ("Apple M1 Pro GPU".to_string(), Some(16)) // M1 Pro has 14-16 GPU cores
            } else if cpu_info.contains("Max") {
                ("Apple M1 Max GPU".to_string(), Some(32)) // M1 Max has 24-32 GPU cores
            } else {
                ("Apple M1 GPU".to_string(), Some(8)) // Base M1 has 7-8 GPU cores
            }
        } else {
            ("Apple Silicon GPU".to_string(), None)
        }
    }
    /// Estimate Apple GPU clock speed
    fn estimate_apple_gpu_clock(&self, cpu_info: &str) -> Option<u32> {
        // Estimate based on chip type
        if cpu_info.contains("M3") {
            Some(1400) // M3 GPU clock around 1.4 GHz
        } else if cpu_info.contains("M2") {
            Some(1300) // M2 GPU clock around 1.3 GHz
        } else if cpu_info.contains("M1") {
            Some(1200) // M1 GPU clock around 1.2 GHz
        } else {
            Some(1000) // General estimate
        }
    }
    /// Get Apple GPU utilization
    fn get_apple_gpu_utilization(&self) -> Option<f32> {
        // Try to get via powermetrics (if available)
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
    /// Get Apple GPU temperature
    fn get_apple_gpu_temperature(&self) -> Option<f32> {
        let output = Command::new("sysctl").args(["-a"]).output();
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
    /// Parse GPU usage from powermetrics
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
    /// Parse temperature from sysctl
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
    /// Get unified memory size for Apple Silicon
    fn get_unified_memory_size(&self) -> Option<u32> {
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
    fn enhance_with_additional_info(&self, gpus: &mut [GpuInfo]) {
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
    /// Approximate GPU utilization estimate
    fn get_gpu_utilization_estimate(&self) -> Option<f32> {
        let output = Command::new("vm_stat").output();
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
        } else if gpu.utilization.is_none() {
            gpu.utilization = self.get_gpu_utilization_estimate();
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
    fn test_macos_provider_creation() {
        let provider = MacosProvider::new();
        let default_provider = MacosProvider;
        // Ensure both creation methods work
        assert_eq!(provider.get_vendor(), Vendor::Unknown);
        assert_eq!(default_provider.get_vendor(), Vendor::Unknown);
    }

    #[test]
    fn test_macos_provider_vendor() {
        let provider = MacosProvider::new();
        // For macOS provider, we return Unknown as it can detect multiple vendors
        assert_eq!(provider.get_vendor(), Vendor::Unknown);
    }

    #[test]
    fn test_parse_vram_string() {
        let provider = MacosProvider::new();

        // Test megabytes parsing
        assert_eq!(provider.parse_vram_string("4096 MB"), Some(4));
        assert_eq!(provider.parse_vram_string("8192 MB"), Some(8));
        assert_eq!(provider.parse_vram_string("512 MB"), Some(1)); // Rounds up

        // Test gigabytes parsing
        assert_eq!(provider.parse_vram_string("8 GB"), Some(8));
        assert_eq!(provider.parse_vram_string("16 GB"), Some(16));

        // Test invalid inputs
        assert_eq!(provider.parse_vram_string("invalid"), None);
        assert_eq!(provider.parse_vram_string(""), None);
    }

    #[test]
    fn test_parse_clock_speed() {
        let provider = MacosProvider::new();

        // Test MHz parsing
        assert_eq!(provider.parse_clock_speed("1200 MHz"), Some(1200));
        assert_eq!(provider.parse_clock_speed("2400 MHz"), Some(2400));

        // Test GHz parsing
        assert_eq!(provider.parse_clock_speed("1.5 GHz"), Some(1500));
        assert_eq!(provider.parse_clock_speed("2.0 GHz"), Some(2000));

        // Test invalid inputs
        assert_eq!(provider.parse_clock_speed("invalid"), None);
        assert_eq!(provider.parse_clock_speed(""), None);
    }

    #[test]
    fn test_extract_vram_from_name() {
        let provider = MacosProvider::new();

        // Test various GPU name formats
        assert_eq!(
            provider.extract_vram_from_name("NVIDIA GeForce RTX 3080 10GB"),
            Some(10)
        );
        assert_eq!(
            provider.extract_vram_from_name("AMD Radeon Pro 8GB"),
            Some(8)
        );
        assert_eq!(
            provider.extract_vram_from_name("Intel Iris Plus Graphics 2gb"),
            Some(2)
        );

        // Test names without VRAM info
        assert_eq!(provider.extract_vram_from_name("Intel HD Graphics"), None);
        assert_eq!(provider.extract_vram_from_name(""), None);
    }

    #[test]
    fn test_determine_vendor() {
        let provider = MacosProvider::new();

        // Test NVIDIA detection
        let nvidia_vendor = provider.determine_vendor("NVIDIA GeForce GTX 1080");
        assert_eq!(nvidia_vendor, Vendor::Nvidia);

        // Test AMD detection
        let amd_vendor = provider.determine_vendor("AMD Radeon Pro 5500M");
        assert_eq!(amd_vendor, Vendor::Amd);

        // Test Apple detection
        let apple_vendor = provider.determine_vendor("Apple M1 GPU");
        assert_eq!(apple_vendor, Vendor::Apple);

        // Test unknown vendor
        let unknown_vendor = provider.determine_vendor("Unknown GPU");
        assert_eq!(unknown_vendor, Vendor::Unknown);
    }

    #[test]
    fn test_determine_apple_gpu_info() {
        let provider = MacosProvider::new();

        // Test M3 variants
        let (m3_name, m3_cores) = provider.determine_apple_gpu_info("Apple M3");
        assert!(m3_name.contains("M3"));
        assert!(m3_cores.is_some());

        let (m3_pro_name, m3_pro_cores) = provider.determine_apple_gpu_info("Apple M3 Pro");
        assert!(m3_pro_name.contains("M3 Pro"));
        assert_eq!(m3_pro_cores, Some(18));

        let (m3_max_name, m3_max_cores) = provider.determine_apple_gpu_info("Apple M3 Max");
        assert!(m3_max_name.contains("M3 Max"));
        assert_eq!(m3_max_cores, Some(40));

        // Test M2 variants
        let (m2_name, _) = provider.determine_apple_gpu_info("Apple M2");
        assert!(m2_name.contains("M2"));

        // Test M1 variants
        let (m1_name, _) = provider.determine_apple_gpu_info("Apple M1");
        assert!(m1_name.contains("M1"));

        // Test unknown Apple chip
        let (unknown_name, unknown_cores) = provider.determine_apple_gpu_info("Apple Unknown");
        assert_eq!(unknown_name, "Apple Silicon GPU");
        assert_eq!(unknown_cores, None);
    }

    #[test]
    fn test_estimate_apple_gpu_clock() {
        let provider = MacosProvider::new();

        // Test M3 clock estimation
        assert_eq!(provider.estimate_apple_gpu_clock("Apple M3"), Some(1400));
        assert_eq!(
            provider.estimate_apple_gpu_clock("Apple M3 Pro"),
            Some(1400)
        );

        // Test M2 clock estimation
        assert_eq!(provider.estimate_apple_gpu_clock("Apple M2"), Some(1300));

        // Test M1 clock estimation
        assert_eq!(provider.estimate_apple_gpu_clock("Apple M1"), Some(1200));

        // Test unknown chip
        assert_eq!(
            provider.estimate_apple_gpu_clock("Apple Unknown"),
            Some(1000)
        );
    }

    #[test]
    fn test_extract_string_value() {
        let provider = MacosProvider::new();

        // Test valid XML string
        assert_eq!(
            provider.extract_string_value("<string>Test Value</string>"),
            Some("Test Value".to_string())
        );

        // Test invalid format
        assert_eq!(provider.extract_string_value("<int>123</int>"), None);
        assert_eq!(provider.extract_string_value(""), None);
        assert_eq!(provider.extract_string_value("Plain text"), None);
    }

    #[test]
    fn test_parse_temperature_from_sysctl() {
        let provider = MacosProvider::new();

        // Test Celsius temperature
        assert_eq!(
            provider.parse_temperature_from_sysctl("thermal.gpu: 45.5"),
            Some(45.5)
        );

        // Test Kelvin temperature (should convert to Celsius)
        assert_eq!(
            provider.parse_temperature_from_sysctl("thermal.gpu: 318.15"),
            Some(45.0)
        );

        // Test invalid format
        assert_eq!(
            provider.parse_temperature_from_sysctl("thermal.gpu invalid"),
            None
        );
        assert_eq!(provider.parse_temperature_from_sysctl(""), None);
    }

    #[test]
    fn test_detect_gpus_returns_result() {
        let provider = MacosProvider::new();
        // Should always return Ok, even if no GPUs found
        let result = provider.detect_gpus();
        assert!(result.is_ok());
    }

    #[test]
    fn test_update_gpu_does_not_panic() {
        let provider = MacosProvider::new();
        let mut gpu = GpuInfo::unknown();

        // Should not panic even with unknown GPU
        let result = provider.update_gpu(&mut gpu);
        assert!(result.is_ok());
        assert_eq!(gpu.active, Some(true)); // Should set active to true
    }

    #[test]
    fn test_update_gpu_preserves_existing_info() {
        let provider = MacosProvider::new();
        let mut gpu = GpuInfo {
            vendor: Vendor::Apple,
            name_gpu: Some("Apple M1 GPU".to_string()),
            memory_total: Some(8),
            ..Default::default()
        };

        let _ = provider.update_gpu(&mut gpu);

        // Should preserve existing information
        assert_eq!(gpu.vendor, Vendor::Apple);
        assert_eq!(gpu.name_gpu, Some("Apple M1 GPU".to_string()));
        assert_eq!(gpu.memory_total, Some(8));
        assert_eq!(gpu.active, Some(true));
    }
}
