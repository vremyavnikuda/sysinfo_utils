//! Retrieves detailed GPU information in a cross-platform manner.
//gpu_info/src/gpu_info.rs
use crate::vendor::Vendor;
use std::fmt::{ Display, Formatter };

/// All information gathered from the system about the current GPU.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuInfo {
    pub vendor: Vendor, // производитель GPU
    pub name_gpu: Option<String>, // полное название GPU
    pub temperature: Option<f32>, // текущая температура GPU
    pub utilization: Option<f32>, // текущее использование GPU (%)
    pub power_usage: Option<f32>, // текущее потребление энергии (Вт)
    pub core_clock: Option<u32>, // текущая частота ядра GPU (МГц)
    pub memory_util: Option<f32>, // использование памяти GPU (%)
    pub memory_clock: Option<u32>, // частота памяти GPU (МГц)
    pub active: Option<bool>, // активен ли GPU
    pub power_limit: Option<f32>, // лимит потребления энергии (Вт)
    pub memory_total: Option<u32>, // общий объем памяти GPU (МБ)
    pub driver_version: Option<String>, // версия драйвера
    pub max_clock_speed: Option<u32>, // максимальная частота GPU (МГц)
}

impl GpuInfo {
    /// Создает новый экземпляр GpuInfo с неизвестными значениями
    pub fn unknown() -> Self {
        Self {
            vendor: Vendor::Unknown,
            name_gpu: None,
            temperature: None,
            utilization: None,
            power_usage: None,
            core_clock: None,
            memory_util: None,
            memory_clock: None,
            active: None,
            power_limit: None,
            memory_total: None,
            driver_version: None,
            max_clock_speed: None,
        }
    }

    pub fn write_vendor(vendor: Vendor) -> Self {
        Self {
            vendor,
            ..Default::default()
        }
    }

    pub fn vendor(&self) -> Vendor {
        self.vendor
    }

    pub fn name_gpu(&self) -> Option<&str> {
        self.name_gpu.as_ref().map(String::as_ref)
    }

    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    pub fn utilization(&self) -> Option<f32> {
        self.utilization
    }

    pub fn power_usage(&self) -> Option<f32> {
        self.power_usage
    }

    pub fn core_clock(&self) -> Option<u32> {
        self.core_clock
    }

    pub fn memory_util(&self) -> Option<f32> {
        self.memory_util
    }

    pub fn memory_clock(&self) -> Option<u32> {
        self.memory_clock
    }

    pub fn active(&self) -> Option<bool> {
        self.active
    }

    pub fn power_limit(&self) -> Option<f32> {
        self.power_limit
    }

    pub fn memory_total(&self) -> Option<u32> {
        self.memory_total
    }

    pub fn driver_version(&self) -> Option<&str> {
        self.driver_version.as_ref().map(String::as_ref)
    }

    pub fn max_clock_speed(&self) -> Option<u32> {
        self.max_clock_speed
    }
}

impl Default for GpuInfo {
    fn default() -> Self {
        Self::unknown()
    }
}

impl Display for GpuInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.vendor)?;
        if let Some(name) = &self.name_gpu {
            write!(f, "{}", name)?;
        }
        if let Some(temp) = &self.temperature {
            write!(f, "{}", temp)?;
        }
        if let Some(util) = &self.utilization {
            write!(f, "{}", util)?;
        }
        if let Some(power) = &self.power_usage {
            write!(f, "{}", power)?;
        }
        if let Some(core) = &self.core_clock {
            write!(f, "{}", core)?;
        }
        if let Some(mem_util) = &self.memory_util {
            write!(f, "{}", mem_util)?;
        }
        if let Some(mem_clock) = &self.memory_clock {
            write!(f, "{}", mem_clock)?;
        }
        if let Some(active) = &self.active {
            write!(f, "{}", active)?;
        }
        if let Some(power_limit) = &self.power_limit {
            write!(f, "{}", power_limit)?;
        }
        if let Some(mem_total) = &self.memory_total {
            write!(f, "{}", mem_total)?;
        }
        if let Some(driver) = &self.driver_version {
            write!(f, "{}", driver)?;
        }
        if let Some(max_clock) = &self.max_clock_speed {
            write!(f, "{}", max_clock)?;
        }
        Ok(())
    }
}
