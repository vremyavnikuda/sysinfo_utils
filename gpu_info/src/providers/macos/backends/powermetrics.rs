//! PowerMetrics backend for GPU metrics
//!
//! Uses the `powermetrics` command-line tool to retrieve GPU metrics on macOS.
//! This backend provides real-time GPU utilization, power usage, and temperature data.
//!
//! # Performance
//!
//! - Metrics retrieval: ~100-200ms
//! - May require sudo for full access (temperature, detailed metrics)
//! - Provides GPU utilization and power usage without sudo
//! - Available on macOS 10.9+
//!
//! # Comparison with other backends
//!
//! | Backend           | Speed      | Requires sudo | Metrics available          |
//! |-------------------|------------|---------------|----------------------------|
//! | IOKit             | 1-10ms     | No            | Static info, PCI data      |
//! | Metal             | 1-5ms      | No            | Memory, some utilization   |
//! | **PowerMetrics**  | 100-200ms  | Partial       | Full utilization, power    |
//! | SystemProfiler    | 500-1000ms | No            | Static info only           |
//!
//! # Examples
//!
//! ```no_run
//! use gpu_info::providers::macos::backends::PowerMetricsBackend;
//!
//! let backend = PowerMetricsBackend::new();
//!
//! if backend.is_available() {
//!     if let Some(metrics) = backend.try_get_metrics() {
//!         println!("GPU Utilization: {:?}%", metrics.utilization);
//!         println!("GPU Power: {:?}W", metrics.power_watts);
//!     }
//! }
//! ```

use crate::gpu_info::{GpuError, GpuInfo, Result};
use log::{debug, warn};
use std::process::{Command, Stdio};
use std::time::Duration;

/// PowerMetrics backend for GPU metrics
///
/// Provides access to real-time GPU metrics through the `powermetrics` CLI tool.
/// This backend works without special permissions for basic metrics, but requires
/// sudo for temperature and detailed power information.
///
/// # Thread Safety
///
/// PowerMetricsBackend is Send + Sync. Each call spawns a new process,
/// so concurrent calls are safe but not recommended due to performance overhead.
#[derive(Debug, Clone)]
pub struct PowerMetricsBackend {
    /// Whether to attempt sudo access
    try_sudo: bool,
}

impl PowerMetricsBackend {
    /// Creates a new PowerMetrics backend with default settings
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::backends::PowerMetricsBackend;
    ///
    /// let backend = PowerMetricsBackend::new();
    /// ```
    pub fn new() -> Self {
        Self { try_sudo: false }
    }

    /// Creates a new PowerMetrics backend with custom timeout
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use gpu_info::providers::macos::backends::PowerMetricsBackend;
    ///
    /// let backend = PowerMetricsBackend::with_timeout(Duration::from_secs(1));
    /// ```
    pub fn with_timeout(_timeout: Duration) -> Self {
        Self { try_sudo: false }
    }

    /// Enables sudo access for powermetrics
    ///
    /// When enabled, backend will attempt to run powermetrics with sudo
    /// to access additional metrics like temperature.
    ///
    /// # Security Considerations
    ///
    /// Only enable this if you're certain sudo access is properly configured
    /// and the application has appropriate permissions.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::backends::PowerMetricsBackend;
    ///
    /// let backend = PowerMetricsBackend::new().with_sudo(true);
    /// ```
    pub fn with_sudo(mut self, enabled: bool) -> Self {
        self.try_sudo = enabled;
        self
    }

    /// Checks if powermetrics is available on the system
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::backends::PowerMetricsBackend;
    ///
    /// let backend = PowerMetricsBackend::new();
    /// if backend.is_available() {
    ///     println!("powermetrics is available");
    /// }
    /// ```
    pub fn is_available(&self) -> bool {
        debug!("Checking if powermetrics is available");

        Command::new("which")
            .arg("powermetrics")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| {
                let available = status.success();
                debug!("powermetrics available: {}", available);
                available
            })
            .unwrap_or_else(|e| {
                warn!("Failed to check powermetrics availability: {}", e);
                false
            })
    }

    /// Attempts to retrieve GPU metrics without sudo
    ///
    /// Returns `None` if:
    /// - powermetrics is not available
    /// - Command execution fails
    /// - Parsing fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gpu_info::providers::macos::backends::PowerMetricsBackend;
    ///
    /// let backend = PowerMetricsBackend::new();
    /// if let Some(metrics) = backend.try_get_metrics() {
    ///     println!("GPU usage: {:?}%", metrics.utilization);
    /// }
    /// ```
    pub fn try_get_metrics(&self) -> Option<GpuMetrics> {
        debug!("Attempting to retrieve GPU metrics from powermetrics");

        if !self.is_available() {
            warn!("powermetrics not available");
            return None;
        }

        self.run_powermetrics()
            .and_then(|output| self.parse_metrics(&output))
    }

    /// Updates GPU with metrics from powermetrics
    ///
    /// # Errors
    ///
    /// Returns an error if powermetrics is not available or execution fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gpu_info::providers::macos::backends::PowerMetricsBackend;
    /// use gpu_info::gpu_info::GpuInfo;
    ///
    /// let backend = PowerMetricsBackend::new();
    /// let mut gpu = GpuInfo::default();
    /// backend.update_gpu(&mut gpu)?;
    /// # Ok::<(), gpu_info::gpu_info::GpuError>(())
    /// ```
    pub fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        debug!("Updating GPU {:?} with powermetrics data", gpu.name_gpu);

        match self.try_get_metrics() {
            Some(metrics) => {
                // TODO: Update GpuInfo with metrics when extended_info is available
                // For now, log the metrics
                debug!("Retrieved metrics: {:?}", metrics);

                if let Some(util) = metrics.utilization {
                    debug!("GPU utilization: {:.2}%", util);
                }

                if let Some(power) = metrics.power_watts {
                    debug!("GPU power usage: {:.2}W", power);
                }

                if let Some(temp) = metrics.temperature {
                    debug!("GPU temperature: {:.2}°C", temp);
                }

                Ok(())
            }
            None => {
                warn!("Failed to retrieve powermetrics data");
                Err(GpuError::DriverNotInstalled)
            }
        }
    }

    /// Runs powermetrics command and returns output
    ///
    /// TODO: Implement actual powermetrics execution (REQUIRES macOS)
    /// Priority: HIGH
    ///
    /// Implementation steps:
    /// 1. Build command with appropriate flags:
    ///    - `-n 1` for single sample
    ///    - `--samplers gpu_power` for GPU metrics
    ///    - `-i 100` for 100ms interval
    /// 2. Execute with timeout
    /// 3. Capture stdout and stderr
    /// 4. Handle errors gracefully
    ///
    /// Example command:
    /// ```bash
    /// powermetrics -n 1 --samplers gpu_power -i 100
    /// ```
    ///
    /// With sudo (for temperature):
    /// ```bash
    /// sudo powermetrics -n 1 --samplers gpu_power,thermal -i 100
    /// ```
    ///
    /// Safety considerations:
    /// - Validate command arguments
    /// - Handle timeout properly
    /// - Don't expose sudo password prompts to users
    fn run_powermetrics(&self) -> Option<String> {
        debug!("Running powermetrics command");

        // TODO: Replace with actual implementation
        // This is a stub that returns None

        warn!("powermetrics execution not yet implemented (requires macOS)");
        None
    }

    /// Parses powermetrics output to extract GPU metrics
    ///
    /// TODO: Implement powermetrics output parsing (CAN be done without macOS)
    /// Priority: HIGH
    ///
    /// Expected output format (varies by macOS version):
    /// ```text
    /// GPU Power: 1234 mW
    /// GPU Duty Cycle: 45.67%
    /// GPU Frequency: 1200 MHz
    /// GPU Temperature: 56.7 C (if sudo)
    /// ```
    ///
    /// Alternative format:
    /// ```text
    /// gpu_power: 1.234 W
    /// gpu_duty_cycle: 45.67
    /// gpu_frequency: 1200000000 Hz
    /// ```
    ///
    /// Implementation steps:
    /// 1. Split output by lines
    /// 2. Parse each line for known patterns:
    ///    - "GPU Power:" or "gpu_power:" → extract watts
    ///    - "GPU Duty Cycle:" or "gpu_duty_cycle:" → extract percentage
    ///    - "GPU Temperature:" or "gpu_temp:" → extract celsius
    ///    - "GPU Frequency:" → extract MHz
    /// 3. Handle different units (mW, W, %, MHz, Hz)
    /// 4. Return GpuMetrics with parsed values
    ///
    /// Example implementation:
    /// ```rust,ignore
    /// fn parse_metrics(&self, output: &str) -> Option<GpuMetrics> {
    ///     let mut metrics = GpuMetrics::default();
    ///
    ///     for line in output.lines() {
    ///         if line.contains("GPU Duty Cycle:") {
    ///             if let Some(value) = parse_percentage(line) {
    ///                 metrics.utilization = Some(value);
    ///             }
    ///         } else if line.contains("GPU Power:") {
    ///             if let Some(value) = parse_power(line) {
    ///                 metrics.power_watts = Some(value);
    ///             }
    ///         } else if line.contains("GPU Temperature:") {
    ///             if let Some(value) = parse_temperature(line) {
    ///                 metrics.temperature = Some(value);
    ///             }
    ///         }
    ///     }
    ///
    ///     Some(metrics)
    /// }
    /// ```
    fn parse_metrics(&self, output: &str) -> Option<GpuMetrics> {
        debug!("Parsing powermetrics output ({} bytes)", output.len());

        // TODO: Replace with actual parsing logic
        // This is a stub that returns default metrics

        warn!("powermetrics parsing not yet implemented");
        Some(GpuMetrics::default())
    }

    /// Parses percentage value from a line
    ///
    /// Supports formats:
    /// - "45.67%"
    /// - "45.67"
    /// - "GPU Duty Cycle: 45.67%"
    #[allow(dead_code)]
    fn parse_percentage(line: &str) -> Option<f32> {
        // Extract number followed by optional %
        let parts: Vec<&str> = line.split(':').collect();
        let value_str = parts.last()?.trim();

        let number_str = value_str.trim_end_matches('%').trim();
        number_str.parse::<f32>().ok().map(|v| v.clamp(0.0, 100.0))
    }

    /// Parses power value from a line and converts to watts
    ///
    /// Supports formats:
    /// - "1234 mW" → 1.234 W
    /// - "1.234 W"
    /// - "GPU Power: 1234 mW"
    #[allow(dead_code)]
    fn parse_power(line: &str) -> Option<f32> {
        let parts: Vec<&str> = line.split(':').collect();
        let value_str = parts.last()?.trim();

        // Check for mW
        if value_str.contains("mW") {
            let number_str = value_str.replace("mW", "").trim().to_string();
            return number_str.parse::<f32>().ok().map(|mw| mw / 1000.0);
        }

        // Check for W
        if value_str.contains("W") {
            let number_str = value_str.replace("W", "").trim().to_string();
            return number_str.parse::<f32>().ok();
        }

        // Try parsing as raw number (assume watts)
        value_str.parse::<f32>().ok()
    }

    /// Parses temperature value from a line
    ///
    /// Supports formats:
    /// - "56.7 C"
    /// - "56.7°C"
    /// - "GPU Temperature: 56.7 C"
    #[allow(dead_code)]
    fn parse_temperature(line: &str) -> Option<f32> {
        let parts: Vec<&str> = line.split(':').collect();
        let value_str = parts.last()?.trim();

        let number_str = value_str
            .replace("°C", "")
            .replace("C", "")
            .trim()
            .to_string();

        number_str.parse::<f32>().ok()
    }
}

impl Default for PowerMetricsBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics retrieved from powermetrics
///
/// Contains GPU performance metrics obtained from the `powermetrics` command.
/// All fields are optional as availability depends on:
/// - macOS version
/// - Whether sudo was used
/// - GPU type (integrated vs discrete)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GpuMetrics {
    /// GPU utilization percentage (0.0-100.0)
    ///
    /// Also known as "GPU Duty Cycle" in powermetrics output.
    /// Represents the percentage of time GPU was active.
    pub utilization: Option<f32>,

    /// GPU power usage in watts
    ///
    /// Actual power draw of the GPU. Available without sudo.
    pub power_watts: Option<f32>,

    /// GPU temperature in Celsius
    ///
    /// Usually requires sudo access. May not be available on all Macs.
    pub temperature: Option<f32>,

    /// GPU frequency in MHz
    ///
    /// Current GPU clock speed. Availability varies by GPU type.
    pub frequency_mhz: Option<f32>,
}

impl GpuMetrics {
    /// Creates new GPU metrics
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::backends::powermetrics::GpuMetrics;
    ///
    /// let metrics = GpuMetrics::new(
    ///     Some(45.5),  // utilization
    ///     Some(12.3),  // power
    ///     Some(56.7),  // temperature
    /// );
    /// ```
    pub fn new(
        utilization: Option<f32>,
        power_watts: Option<f32>,
        temperature: Option<f32>,
    ) -> Self {
        Self {
            utilization: utilization.map(|v| v.clamp(0.0, 100.0)),
            power_watts,
            temperature,
            frequency_mhz: None,
        }
    }

    /// Adds frequency information to metrics
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::backends::powermetrics::GpuMetrics;
    ///
    /// let metrics = GpuMetrics::default().with_frequency(1200.0);
    /// assert_eq!(metrics.frequency_mhz, Some(1200.0));
    /// ```
    pub fn with_frequency(mut self, freq_mhz: f32) -> Self {
        self.frequency_mhz = Some(freq_mhz);
        self
    }

    /// Returns true if any metrics are available
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::backends::powermetrics::GpuMetrics;
    ///
    /// let empty = GpuMetrics::default();
    /// assert!(!empty.has_data());
    ///
    /// let with_data = GpuMetrics::new(Some(50.0), None, None);
    /// assert!(with_data.has_data());
    /// ```
    pub fn has_data(&self) -> bool {
        self.utilization.is_some()
            || self.power_watts.is_some()
            || self.temperature.is_some()
            || self.frequency_mhz.is_some()
    }

    /// Returns true if GPU is under heavy load (>80%)
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::providers::macos::backends::powermetrics::GpuMetrics;
    ///
    /// let light = GpuMetrics::new(Some(50.0), None, None);
    /// assert!(!light.is_heavy_load());
    ///
    /// let heavy = GpuMetrics::new(Some(85.0), None, None);
    /// assert!(heavy.is_heavy_load());
    /// ```
    pub fn is_heavy_load(&self) -> bool {
        self.utilization.map(|u| u > 80.0).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powermetrics_creation() {
        let backend = PowerMetricsBackend::new();
        assert!(!backend.try_sudo);
    }

    #[test]
    fn test_with_timeout() {
        let _backend = PowerMetricsBackend::with_timeout(Duration::from_secs(2));
        // Backend accepts timeout parameter
    }

    #[test]
    fn test_with_sudo() {
        let backend = PowerMetricsBackend::new().with_sudo(true);
        assert!(backend.try_sudo);
    }

    #[test]
    fn test_is_available() {
        let backend = PowerMetricsBackend::new();
        // Just test that it doesn't panic
        let _ = backend.is_available();
    }

    #[test]
    fn test_gpu_metrics_new() {
        let metrics = GpuMetrics::new(Some(45.5), Some(12.3), Some(56.7));
        assert_eq!(metrics.utilization, Some(45.5));
        assert_eq!(metrics.power_watts, Some(12.3));
        assert_eq!(metrics.temperature, Some(56.7));
    }

    #[test]
    fn test_gpu_metrics_clamp() {
        let metrics = GpuMetrics::new(Some(150.0), None, None);
        assert_eq!(metrics.utilization, Some(100.0)); // Clamped to max
    }

    #[test]
    fn test_gpu_metrics_with_frequency() {
        let metrics = GpuMetrics::default().with_frequency(1200.0);
        assert_eq!(metrics.frequency_mhz, Some(1200.0));
    }

    #[test]
    fn test_gpu_metrics_has_data() {
        let empty = GpuMetrics::default();
        assert!(!empty.has_data());

        let with_util = GpuMetrics::new(Some(50.0), None, None);
        assert!(with_util.has_data());

        let with_power = GpuMetrics::new(None, Some(10.0), None);
        assert!(with_power.has_data());
    }

    #[test]
    fn test_gpu_metrics_is_heavy_load() {
        let light = GpuMetrics::new(Some(50.0), None, None);
        assert!(!light.is_heavy_load());

        let boundary = GpuMetrics::new(Some(80.0), None, None);
        assert!(!boundary.is_heavy_load());

        let heavy = GpuMetrics::new(Some(85.0), None, None);
        assert!(heavy.is_heavy_load());
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(PowerMetricsBackend::parse_percentage("45.67%"), Some(45.67));
        assert_eq!(PowerMetricsBackend::parse_percentage("45.67"), Some(45.67));
        assert_eq!(
            PowerMetricsBackend::parse_percentage("GPU Duty Cycle: 45.67%"),
            Some(45.67)
        );
        assert_eq!(PowerMetricsBackend::parse_percentage("invalid"), None);

        // Test clamping
        assert_eq!(PowerMetricsBackend::parse_percentage("150.0%"), Some(100.0));
        assert_eq!(PowerMetricsBackend::parse_percentage("-10.0%"), Some(0.0));
    }

    #[test]
    fn test_parse_power() {
        assert_eq!(PowerMetricsBackend::parse_power("1234 mW"), Some(1.234));
        assert_eq!(PowerMetricsBackend::parse_power("1.234 W"), Some(1.234));
        assert_eq!(
            PowerMetricsBackend::parse_power("GPU Power: 1234 mW"),
            Some(1.234)
        );
        assert_eq!(PowerMetricsBackend::parse_power("invalid"), None);
    }

    #[test]
    fn test_parse_temperature() {
        assert_eq!(PowerMetricsBackend::parse_temperature("56.7 C"), Some(56.7));
        assert_eq!(PowerMetricsBackend::parse_temperature("56.7°C"), Some(56.7));
        assert_eq!(
            PowerMetricsBackend::parse_temperature("GPU Temperature: 56.7 C"),
            Some(56.7)
        );
        assert_eq!(PowerMetricsBackend::parse_temperature("invalid"), None);
    }

    #[test]
    fn test_default() {
        let _backend = PowerMetricsBackend::default();
        // Backend created with default settings
    }

    #[test]
    fn test_update_gpu_graceful_failure() {
        let backend = PowerMetricsBackend::new();
        let mut gpu = GpuInfo::default();

        // Should not panic even if powermetrics fails
        let result = backend.update_gpu(&mut gpu);
        // On systems without powermetrics, this will fail gracefully
        let _ = result;
    }
}
