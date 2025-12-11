use crate::gpu_info::{GpuInfo, Result};
#[cfg(target_os = "windows")]
use crate::vendor::Vendor;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
/// Extended GPU information with additional metrics
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExtendedGpuInfo {
    /// Basic GPU information
    pub base_info: GpuInfo,

    /// Fan information
    pub fan_info: FanInfo,

    /// Video encoder/decoder information
    pub encoder_info: EncoderInfo,

    /// Memory information
    pub memory_info: MemoryInfo,

    /// Bus and connection information
    pub connection_info: ConnectionInfo,

    /// Thermal information
    pub thermal_info: ThermalInfo,

    /// Performance information
    pub performance_info: PerformanceInfo,
}
/// Cooling system information
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FanInfo {
    /// Fan speed in RPM
    pub fan_speed_rpm: Option<u32>,

    /// Fan speed in percent (0-100)
    pub fan_speed_percent: Option<f32>,

    /// Number of fans
    pub fan_count: Option<u8>,

    /// Information about each individual fan
    pub individual_fans: Vec<IndividualFanInfo>,

    /// Automatic fan control mode
    pub auto_fan_control: Option<bool>,

    /// Target temperature for auto control
    pub target_temperature: Option<f32>,
}
/// Individual fan information
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndividualFanInfo {
    /// Fan index
    pub index: u8,

    /// Speed in RPM
    pub speed_rpm: Option<u32>,

    /// Speed in percent
    pub speed_percent: Option<f32>,

    /// Maximum speed
    pub max_speed_rpm: Option<u32>,
}
/// Video encoder/decoder information
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EncoderInfo {
    /// Video encoder utilization (%)
    pub encoder_utilization: Option<f32>,

    /// Video decoder utilization (%)
    pub decoder_utilization: Option<f32>,

    /// Supported codecs
    pub supported_codecs: Vec<String>,

    /// Currently active codec
    pub active_codec: Option<String>,

    /// Number of active encoding sessions
    pub active_encoding_sessions: Option<u32>,

    /// Number of active decoding sessions
    pub active_decoding_sessions: Option<u32>,
}
/// Extended memory information
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MemoryInfo {
    /// Total video memory in MB
    pub total_memory_mb: Option<u64>,

    /// Used memory in MB
    pub used_memory_mb: Option<u64>,

    /// Free memory in MB
    pub free_memory_mb: Option<u64>,

    /// Memory bandwidth in GB/s
    pub memory_bandwidth_gb_s: Option<f32>,

    /// Memory bandwidth utilization (%)
    pub memory_bandwidth_utilization: Option<f32>,

    /// Memory type (GDDR6, HBM2, etc.)
    pub memory_type: Option<String>,

    /// Memory bus width in bits
    pub memory_bus_width: Option<u32>,

    /// ECC status (if supported)
    pub ecc_enabled: Option<bool>,

    /// Number of ECC errors
    pub ecc_errors: Option<u64>,
}
/// Connection information
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConnectionInfo {
    /// PCIe generation (3, 4, 5, etc.)
    pub pcie_generation: Option<u8>,

    /// PCIe lane width (x1, x4, x8, x16)
    pub pcie_width: Option<u8>,

    /// Current PCIe throughput in GB/s
    pub pcie_throughput_gb_s: Option<f32>,

    /// Maximum PCIe throughput in GB/s
    pub pcie_max_throughput_gb_s: Option<f32>,

    /// PCIe bandwidth utilization (%)
    pub pcie_utilization: Option<f32>,

    /// Bus ID
    pub bus_id: Option<String>,

    /// Device ID
    pub device_id: Option<String>,

    /// Vendor ID
    pub vendor_id: Option<String>,

    /// Subsystem ID
    pub subsystem_id: Option<String>,
}
/// Extended thermal information
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ThermalInfo {
    /// GPU temperature in °C
    pub gpu_temperature: Option<f32>,

    /// Memory temperature in °C
    pub memory_temperature: Option<f32>,

    /// VRM temperature in °C
    pub vrm_temperature: Option<f32>,

    /// Maximum safe temperature
    pub max_safe_temperature: Option<f32>,

    /// Throttling start temperature
    pub throttle_temperature: Option<f32>,

    /// Critical shutdown temperature
    pub critical_temperature: Option<f32>,

    /// Current throttling state
    pub is_throttling: Option<bool>,

    /// Throttling reason
    pub throttle_reason: Option<ThrottleReason>,
}
/// Throttling reasons
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ThrottleReason {
    None,
    Temperature,
    PowerLimit,
    VoltageLimit,
    Unknown,
}
/// Performance information
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PerformanceInfo {
    /// Base core clock in MHz
    pub base_core_clock: Option<u32>,

    /// Boost core clock in MHz
    pub boost_core_clock: Option<u32>,

    /// Shader clock in MHz (for NVIDIA)
    pub shader_clock: Option<u32>,

    /// Base memory clock in MHz
    pub base_memory_clock: Option<u32>,

    /// Boost memory clock in MHz
    pub boost_memory_clock: Option<u32>,

    /// Current performance state
    pub performance_state: Option<PerformanceState>,

    /// Available performance states
    pub available_performance_states: Vec<PerformanceState>,

    /// Overclocking capabilities
    pub overclocking_info: OverclockingInfo,
}
/// GPU performance states
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PerformanceState {
    /// Maximum performance
    Maximum,
    /// High performance
    High,
    /// Medium performance
    Medium,
    /// Power saver
    PowerSaver,
    /// Adaptive
    Adaptive,
    /// Unknown
    Unknown,
}
/// Overclocking information
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OverclockingInfo {
    /// Overclocking support
    pub overclocking_supported: Option<bool>,

    /// Current core clock offset in MHz
    pub core_clock_offset: Option<i32>,

    /// Current memory clock offset in MHz
    pub memory_clock_offset: Option<i32>,

    /// Maximum safe core clock offset
    pub max_core_clock_offset: Option<i32>,

    /// Maximum safe memory clock offset
    pub max_memory_clock_offset: Option<i32>,

    /// Current voltage limit in mV
    pub voltage_limit: Option<u32>,

    /// Maximum voltage limit
    pub max_voltage_limit: Option<u32>,
}
impl ExtendedGpuInfo {
    /// Creates ExtendedGpuInfo from basic GpuInfo
    pub fn from_basic(gpu_info: GpuInfo) -> Self {
        Self {
            base_info: gpu_info,
            fan_info: FanInfo::default(),
            encoder_info: EncoderInfo::default(),
            memory_info: MemoryInfo::default(),
            connection_info: ConnectionInfo::default(),
            thermal_info: ThermalInfo::default(),
            performance_info: PerformanceInfo::default(),
        }
    }
    /// Creates an unknown ExtendedGpuInfo
    pub fn unknown() -> Self {
        Self {
            base_info: GpuInfo::unknown(),
            fan_info: FanInfo::default(),
            encoder_info: EncoderInfo::default(),
            memory_info: MemoryInfo::default(),
            connection_info: ConnectionInfo::default(),
            thermal_info: ThermalInfo::default(),
            performance_info: PerformanceInfo::default(),
        }
    }
    /// Returns basic information
    pub fn base(&self) -> &GpuInfo {
        &self.base_info
    }
    /// Returns overall cooling efficiency
    pub fn cooling_efficiency(&self) -> Option<f32> {
        if let (Some(temp), Some(fan_speed)) = (
            self.thermal_info.gpu_temperature,
            self.fan_info.fan_speed_percent,
        ) {
            if fan_speed > 0.0 {
                Some(100.0 - (temp * fan_speed) / 100.0)
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Returns overall GPU health score (0-100)
    pub fn health_score(&self) -> f32 {
        let mut score: f32 = 100.0;
        let mut factors = 0;
        if let Some(temp) = self.thermal_info.gpu_temperature {
            factors += 1;
            if temp > 85.0 {
                score -= 30.0;
            } else if temp > 75.0 {
                score -= 15.0;
            } else if temp > 65.0 {
                score -= 5.0;
            }
        }
        if let Some(is_throttling) = self.thermal_info.is_throttling {
            factors += 1;
            if is_throttling {
                score -= 25.0;
            }
        }
        if let Some(mem_util) = self.base_info.memory_util {
            factors += 1;
            if mem_util > 95.0 {
                score -= 10.0;
            }
        }
        if let Some(ecc_errors) = self.memory_info.ecc_errors {
            factors += 1;
            if ecc_errors > 0 {
                score -= 20.0;
            }
        }
        if factors == 0 {
            50.0
        } else {
            score.clamp(0.0, 100.0)
        }
    }
    /// Checks if GPU needs attention
    pub fn needs_attention(&self) -> bool {
        self.health_score() < 70.0
    }
}
impl Display for ExtendedGpuInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Extended GPU Information")?;
        writeln!(f, "Name: {}", self.base_info.format_name_gpu())?;
        writeln!(f, "Vendor: {}", self.base_info.vendor)?;
        writeln!(f, "Health Score: {:.1}%", self.health_score())?;
        if let Some(temp) = self.thermal_info.gpu_temperature {
            writeln!(f, "Temperature: {:.1}°C", temp)?;
        }
        if let Some(util) = self.base_info.utilization {
            writeln!(f, "Utilization: {:.1}%", util)?;
        }
        if let Some(mem_util) = self.base_info.memory_util {
            writeln!(f, "Memory Utilization: {:.1}%", mem_util)?;
        }
        if let Some(fan_speed) = self.fan_info.fan_speed_percent {
            writeln!(f, "Fan Speed: {:.1}%", fan_speed)?;
        }
        Ok(())
    }
}
impl Default for ExtendedGpuInfo {
    fn default() -> Self {
        Self::unknown()
    }
}
/// Trait for converting basic GpuInfo to extended
pub trait GpuInfoExtensions {
    fn to_extended(self) -> ExtendedGpuInfo;
    fn enhance(&mut self) -> Result<()>;
}
impl GpuInfoExtensions for GpuInfo {
    fn to_extended(self) -> ExtendedGpuInfo {
        ExtendedGpuInfo::from_basic(self)
    }
    fn enhance(&mut self) -> Result<()> {
        match self.vendor {
            #[cfg(target_os = "windows")]
            Vendor::Nvidia => Ok(()),
            #[cfg(target_os = "windows")]
            Vendor::Amd => Ok(()),
            _ => Ok(()),
        }
    }
}
