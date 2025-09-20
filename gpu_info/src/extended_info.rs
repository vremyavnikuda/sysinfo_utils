use crate::gpu_info::{GpuInfo, Result};
use crate::vendor::Vendor;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
/// Расширенная информация о GPU с дополнительными метриками
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ExtendedGpuInfo {
    /// Базовая информация о GPU
    pub base_info: GpuInfo,

    /// Информация о вентиляторах
    pub fan_info: FanInfo,

    /// Информация о видеокодеках
    pub encoder_info: EncoderInfo,

    /// Информация о памяти
    pub memory_info: MemoryInfo,

    /// Информация о шинах и подключении
    pub connection_info: ConnectionInfo,

    /// Тепловая информация
    pub thermal_info: ThermalInfo,

    /// Информация о производительности
    pub performance_info: PerformanceInfo,
}
/// Информация о системе охлаждения
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FanInfo {
    /// Скорость вентилятора в RPM
    pub fan_speed_rpm: Option<u32>,

    /// Скорость вентилятора в процентах (0-100)
    pub fan_speed_percent: Option<f32>,

    /// Количество вентиляторов
    pub fan_count: Option<u8>,

    /// Информация о каждом вентиляторе
    pub individual_fans: Vec<IndividualFanInfo>,

    /// Автоматический режим управления вентиляторами
    pub auto_fan_control: Option<bool>,

    /// Целевая температура для авто-управления
    pub target_temperature: Option<f32>,
}
/// Информация об отдельном вентиляторе
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndividualFanInfo {
    /// Индекс вентилятора
    pub index: u8,

    /// Скорость в RPM
    pub speed_rpm: Option<u32>,

    /// Скорость в процентах
    pub speed_percent: Option<f32>,

    /// Максимальная скорость
    pub max_speed_rpm: Option<u32>,
}
/// Информация о видеокодеках
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EncoderInfo {
    /// Загрузка видеокодера (%)
    pub encoder_utilization: Option<f32>,

    /// Загрузка видеодекодера (%)
    pub decoder_utilization: Option<f32>,

    /// Поддерживаемые кодеки
    pub supported_codecs: Vec<String>,

    /// Текущий активный кодек
    pub active_codec: Option<String>,

    /// Количество активных сессий кодирования
    pub active_encoding_sessions: Option<u32>,

    /// Количество активных сессий декодирования
    pub active_decoding_sessions: Option<u32>,
}
/// Расширенная информация о памяти
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MemoryInfo {
    /// Общий объем видеопамяти в MB
    pub total_memory_mb: Option<u64>,

    /// Используемая память в MB
    pub used_memory_mb: Option<u64>,

    /// Свободная память в MB
    pub free_memory_mb: Option<u64>,

    /// Пропускная способность памяти в GB/s
    pub memory_bandwidth_gb_s: Option<f32>,

    /// Использование пропускной способности памяти (%)
    pub memory_bandwidth_utilization: Option<f32>,

    /// Тип памяти (GDDR6, HBM2, etc.)
    pub memory_type: Option<String>,

    /// Ширина шины памяти в битах
    pub memory_bus_width: Option<u32>,

    /// ECC статус (если поддерживается)
    pub ecc_enabled: Option<bool>,

    /// Количество ошибок ECC
    pub ecc_errors: Option<u64>,
}
/// Информация о подключении
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ConnectionInfo {
    /// Поколение PCIe (3, 4, 5, etc.)
    pub pcie_generation: Option<u8>,

    /// Ширина PCIe линии (x1, x4, x8, x16)
    pub pcie_width: Option<u8>,

    /// Текущая пропускная способность PCIe в GB/s
    pub pcie_throughput_gb_s: Option<f32>,

    /// Максимальная пропускная способность PCIe в GB/s
    pub pcie_max_throughput_gb_s: Option<f32>,

    /// Использование PCIe пропускной способности (%)
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
/// Расширенная тепловая информация
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ThermalInfo {
    /// Температура GPU в °C
    pub gpu_temperature: Option<f32>,

    /// Температура памяти в °C
    pub memory_temperature: Option<f32>,

    /// Температура VRM в °C
    pub vrm_temperature: Option<f32>,

    /// Максимальная безопасная температура
    pub max_safe_temperature: Option<f32>,

    /// Температура начала троттлинга
    pub throttle_temperature: Option<f32>,

    /// Критическая температура отключения
    pub critical_temperature: Option<f32>,

    /// Текущее состояние троттлинга
    pub is_throttling: Option<bool>,

    /// Причина троттлинга
    pub throttle_reason: Option<ThrottleReason>,
}
/// Причины троттлинга
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ThrottleReason {
    None,
    Temperature,
    PowerLimit,
    VoltageLimit,
    Unknown,
}
/// Информация о производительности
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PerformanceInfo {
    /// Базовая частота ядра в MHz
    pub base_core_clock: Option<u32>,

    /// Boost частота ядра в MHz
    pub boost_core_clock: Option<u32>,

    /// Частота шейдеров в MHz (для NVIDIA)
    pub shader_clock: Option<u32>,

    /// Базовая частота памяти в MHz
    pub base_memory_clock: Option<u32>,

    /// Boost частота памяти в MHz
    pub boost_memory_clock: Option<u32>,

    /// Текущее состояние производительности
    pub performance_state: Option<PerformanceState>,

    /// Доступные состояния производительности
    pub available_performance_states: Vec<PerformanceState>,

    /// Возможности разгона
    pub overclocking_info: OverclockingInfo,
}
/// Состояния производительности GPU
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PerformanceState {
    /// Максимальная производительность
    Maximum,
    /// Высокая производительность
    High,
    /// Средняя производительность
    Medium,
    /// Энергосбережение
    PowerSaver,
    /// Адаптивная
    Adaptive,
    /// Неизвестно
    Unknown,
}
/// Информация о разгоне
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OverclockingInfo {
    /// Поддержка разгона
    pub overclocking_supported: Option<bool>,

    /// Текущее смещение частоты ядра в MHz
    pub core_clock_offset: Option<i32>,

    /// Текущее смещение частоты памяти в MHz
    pub memory_clock_offset: Option<i32>,

    /// Максимальное безопасное смещение ядра
    pub max_core_clock_offset: Option<i32>,

    /// Максимальное безопасное смещение памяти
    pub max_memory_clock_offset: Option<i32>,

    /// Текущий лимит напряжения в mV
    pub voltage_limit: Option<u32>,

    /// Максимальный лимит напряжения
    pub max_voltage_limit: Option<u32>,
}
impl ExtendedGpuInfo {
    /// Создает ExtendedGpuInfo из базового GpuInfo
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
    /// Создает неизвестный ExtendedGpuInfo
    pub fn unknown() -> Self {
        Self::default()
    }
    /// Возвращает базовую информацию
    pub fn base(&self) -> &GpuInfo {
        &self.base_info
    }
    /// Возвращает общую эффективность охлаждения
    pub fn cooling_efficiency(&self) -> Option<f32> {
        if let (Some(temp), Some(fan_speed)) = (
            self.thermal_info.gpu_temperature,
            self.fan_info.fan_speed_percent,
        ) {
            if fan_speed > 0.0 {
                Some(100.0 - (temp * fan_speed / 100.0))
            } else {
                None
            }
        } else {
            None
        }
    }
    /// Возвращает общий health score GPU (0-100)
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
            score.max(0.0).min(100.0)
        }
    }
    /// Проверяет, требует ли GPU внимания
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
/// Трейт для преобразования базового GpuInfo в расширенный
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
