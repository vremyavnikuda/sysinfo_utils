use std::fmt::{Debug, Display, Formatter};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::vendor::Vendor;

#[derive(Debug, thiserror::Error)]
pub enum GpuError {
    #[error("Invalid temperature value: {0}")]
    InvalidTemperature(f32),
    #[error("Invalid utilization value: {0}")]
    InvalidUtilization(f32),
    #[error("Invalid power usage value: {0}")]
    InvalidPowerUsage(f32),
    #[error("Invalid clock speed value: {0}")]
    InvalidClockSpeed(u32),
    #[error("Invalid memory value: {0}")]
    InvalidMemory(u32),
    #[error("GPU not found")]
    GpuNotFound,
    #[error("Driver not installed")]
    DriverNotInstalled,
    #[error("GPU not active")]
    GpuNotActive,
}

pub type Result<T> = std::result::Result<T, GpuError>;

// TODO: add to DOC
/// Trait fmt_string (форматируем результат<T> в строковое представление)  defines a method for formatting GPU information.
pub trait Formattable: Debug {
    fn fmt_string(&self) -> String;
}

/// All information gathered from the system about the current GPU.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GpuInfo {
    /// The GPU vendor (e.g., NVIDIA, AMD, Intel).
    pub vendor: Vendor, // GPU manufacturer
    /// The full name of the GPU (e.g., NVIDIA GeForce RTX 3080, AMD Radeon RX 6800 XT ).
    pub name_gpu: Option<String>, // full GPU name
    /// The current temperature of the GPU in degrees Celsius.
    pub temperature: Option<f32>, // current GPU temperature
    /// The current utilization of the GPU as a percentage.
    pub utilization: Option<f32>, // current GPU utilization (%)
    /// The current power usage of the GPU in watts.
    pub power_usage: Option<f32>, // current power consumption (W)
    /// The current core clock speed of the GPU in MHz.
    pub core_clock: Option<u32>, // current GPU core clock (MHz)
    /// The current memory utilization of the GPU as a percentage.
    pub memory_util: Option<f32>, // GPU memory utilization (%)
    /// The current memory clock speed of the GPU in MHz.
    pub memory_clock: Option<u32>, // GPU memory clock (MHz)
    /// Whether the GPU is currently active (in use).
    pub active: Option<bool>, // whether the GPU is active
    /// The power limit of the GPU in watts.
    pub power_limit: Option<f32>, // power consumption limit (W)
    /// The total memory of the GPU in gigabytes.
    pub memory_total: Option<u32>, // total GPU memory (GB)
    /// The driver version of the GPU.
    pub driver_version: Option<String>, // driver version
    /// The maximum clock speed of the GPU in MHz.
    pub max_clock_speed: Option<u32>, // maximum GPU clock speed (MHz)
}

impl Formattable for Option<f32> {
    /// Formats the Option<f32> value into a string representation.
    ///
    /// # Returns
    /// * If `Some(value)`, returns the string representation of the value with one decimal place.
    /// * If `None`, returns "N/A" indicating the absence of a value.
    fn fmt_string(&self) -> String {
        match self {
            Some(value) => format!("{:.1}", value),
            None => String::from("N/A"),
        }
    }
}

impl Formattable for Option<u32> {
    /// Formats the Option<u32> value into a string representation.
    ///
    /// # Returns
    /// * If `Some(value)`, returns the string representation of the value.
    /// * If `None`, returns "N/A" indicating the absence of a value.
    fn fmt_string(&self) -> String {
        match self {
            Some(value) => format!("{}", value),
            None => String::from("N/A"),
        }
    }
}

impl Formattable for Option<bool> {
    /// Formats the Option value into a string representation.
    ///
    /// # Returns
    /// * If `Some(value)`, returns the string representation of the value.
    /// * If `None`, returns "N/A" indicating the absence of a value.
    ///
    fn fmt_string(&self) -> String {
        match self {
            Some(value) => value.to_string(),
            None => String::from("N/A"),
        }
    }
}

impl Formattable for Option<String> {
    /// Formats the Option value into a string representation.
    ///
    /// # Returns
    /// * If `Some(value)`, returns the string representation of the value.
    /// * If `None`, returns "N/A" indicating the absence of a value.
    fn fmt_string(&self) -> String {
        match self {
            Some(value) => value.to_string(),
            None => String::from("N/A"),
        }
    }
}

impl Formattable for Option<&str> {
    /// Formats the Option value into a string representation.
    ///
    /// # Returns
    /// * If `Some(value)`, returns the string representation of the value.
    /// * If `None`, returns "N/A" indicating the absence of a value.
    fn fmt_string(&self) -> String {
        match self {
            Some(value) => value.to_string(),
            None => String::from("N/A"),
        }
    }
}

impl Formattable for String {
    /// Formats the string value into a string representation.
    ///
    /// # Returns
    /// The string value itself.
    fn fmt_string(&self) -> String {
        self.clone()
    }
}

/// A struct representing the GPU information.
impl GpuInfo {
    /// Creates a `GpuInfo` instance with all fields set to unknown or default values.
    ///
    /// This function is useful for initializing a `GpuInfo` struct when no information
    /// about the GPU is available.
    ///
    /// # Returns
    /// A `GpuInfo` instance with all fields set to their default or unknown values.
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let unknown_gpu = GpuInfo::unknown();
    /// ```
    pub fn unknown() -> Self {
        return Self {
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
        };
    }

    /// Creates a new `GpuInfo` instance with the specified GPU vendor.
    ///
    /// This function initializes a `GpuInfo` struct with the given `vendor`
    /// and sets all other fields to their default values.
    ///
    /// # Arguments
    /// * `vendor` - The GPU vendor to set in the `GpuInfo` instance.
    ///
    /// # Returns
    /// A new `GpuInfo` instance with the specified vendor and default values for other fields.
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// use gpu_info::vendor::Vendor;
    ///
    /// let gpu_info = GpuInfo::write_vendor(Vendor::Nvidia);
    /// ```
    /// # Notes
    /// * This function is useful for creating a `GpuInfo` instance when the
    ///   vendor is known, but other information is not yet available.
    pub fn write_vendor(vendor: Vendor) -> Self {
        Self {
            vendor,
            ..GpuInfo::default()
        }
    }

    /// Returns the vendor of the GPU.
    ///
    /// # Returns
    /// * The vendor of the GPU.
    /// ```
    /// pub enum Vendor{
    ///     Nvidia,
    /// }
    /// ```
    /// * `Vendor` - The vendor of the GPU. If the vendor cannot be determined,
    ///   returns `Vendor::Unknown`.
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Vendor: {}",gpu.vendor())
    /// ```
    /// # Notes
    /// * Vendor information is typically set by the manufacturer
    ///   and is stored in the GPU driver, BIOS, or device firmwar
    pub fn vendor(&self) -> Vendor {
        self.vendor
    }

    /// Returns the full name of the GPU.
    ///
    /// This function retrieves the GPU's marketing name as reported by the driver.
    /// The name typically includes the manufacturer's branding and model number,
    /// such as "NVIDIA GeForce RTX 3080" or "AMD Radeon RX 6800 XT".
    ///
    /// The name may not be available if:
    /// - The GPU driver doesn't report this informationq
    /// - The system doesn't have a dedicated GPU
    /// - The GPU information couldn't be accessed due to permission issues
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - The full name of the GPU as a string slice.
    /// * `None` - If the GPU name could not be determined or is not available
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("GPU Name: {:?}", gpu.name_gpu());
    /// ```
    /// # Performance
    /// This is a lightweight accessor method that simply returns to stored data. It performs no I/O operations or complex calculations.
    ///
    pub fn name_gpu(&self) -> Option<&str> {
        self.name_gpu.as_deref()
    }

    /// Returns the current temperature of the GPU.
    ///
    /// This function retrieves the GPU's current temperature as reported by the driver.
    /// The temperature is typically measured at the GPU die and represents the core temperature.
    ///
    /// The temperature may not be available if:
    /// - The GPU doesn't have temperature sensors
    /// - The driver doesn't expose temperature information
    /// - The system doesn't have a dedicated GPU
    /// - The temperature couldn't be accessed due to permission issues
    ///
    /// # Returns
    ///
    /// * `Some(f32)` - The current temperature of the GPU in degrees Celsius.
    /// * `None` - If the GPU temperature could not be determined or is not available.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Temperature: {:?}", gpu.temperature());
    /// ```
    ///
    /// # Notes
    /// - Different GPUs may report temperatures with varying precision
    /// - Some GPUs may throttle performance at certain temperature thresholds
    /// - The temperature reading may have a small delay from real-time values
    ///
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    ///.
    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    /// Returns the current power usage of the GPU.
    ///
    /// # Returns
    /// * `Some(f32)` - The current power usage of the GPU in watts.
    /// * `None` - If the power usage of the GPU is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Power Usage: {:?}", gpu.power_usage());
    /// ```
    pub fn utilization(&self) -> Option<f32> {
        self.utilization
    }

    /// Returns the current power usage of the GPU.
    ///
    /// # Returns
    /// * `Some(f32)` - The current power usage of the GPU in watts.
    /// * `None` - If the power usage of the GPU is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Power Usage: {:?}", gpu.power_usage());
    /// ```
    pub fn power_usage(&self) -> Option<f32> {
        self.power_usage
    }

    /// Returns the current core clock speed of the GPU.
    ///
    /// # Returns
    /// * `Some(u32)` - The current core clock speed of the GPU in MHz.
    /// * `None` - If the core clock speed of the GPU is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Core Clock: {:?}", gpu.core_clock());
    /// ```
    pub fn core_clock(&self) -> Option<u32> {
        self.core_clock
    }

    /// Returns the current memory utilization of the GPU as a percentage.
    ///
    /// # Returns
    /// * `Some(f32)` - The current memory utilization of the GPU in percent.
    /// * `None` - If the memory utilization of the GPU is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Memory Utilization: {:?}", gpu.memory_util());
    /// ```
    pub fn memory_util(&self) -> Option<f32> {
        self.memory_util
    }

    /// Returns the current memory clock speed of the GPU.
    ///
    /// # Returns
    /// * `Some(u32)` - The current memory clock speed of the GPU in MHz.
    /// * `None` - If the memory clock speed of the GPU is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Memory Clock: {:?}", gpu.memory_clock());
    /// ```
    pub fn memory_clock(&self) -> Option<u32> {
        self.memory_clock
    }

    /// Returns whether the GPU is currently active.
    ///
    /// # Returns
    /// * `Some(true)` - If the GPU is active.
    /// * `Some(false)` - If the GPU is inactive.
    /// * `None` - If the active status of the GPU is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Is Active: {:?}", gpu.active());
    /// ```
    pub fn active(&self) -> Option<bool> {
        self.active
    }

    /// Returns the power limit of the GPU in watts.
    ///
    /// # Returns
    /// * `Some(f32)` - The power limit of the GPU in watts.
    /// * `None` - If the power limit of the GPU is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Power Limit: {:?}", gpu.power_limit());
    /// ```
    pub fn power_limit(&self) -> Option<f32> {
        self.power_limit
    }

    /// Returns the total memory of the GPU in megabytes.
    ///
    /// # Returns
    /// * `Some(u32)` - The total memory of the GPU in megabytes.
    /// * `None` - If the total memory of the GPU is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Memory Total: {:?}", gpu.memory_total());
    /// ```
    pub fn memory_total(&self) -> Option<u32> {
        self.memory_total
    }

    /// Returns the driver version of the GPU.
    ///
    /// # Returns
    /// * `Some(&str)` - The driver version of the GPU as a string slice.
    /// * `None` - If the driver version is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Driver Version: {:?}", gpu.driver_version());
    /// ```
    pub fn driver_version(&self) -> Option<&str> {
        self.driver_version.as_deref()
    }

    /// Returns the maximum clock speed of the GPU in MHz.
    ///
    /// # Returns
    /// * `Some(u32)` - The maximum clock speed of the GPU in MHz.
    /// * `None` - If the maximum clock speed is unknown.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Max Clock Speed: {:?}", gpu.max_clock_speed());
    /// ```
    pub fn max_clock_speed(&self) -> Option<u32> {
        self.max_clock_speed
    }

    /// Returns formated the name of the GPU.
    ///
    /// If the name of the GPU is unknown, returns "Unknown GPU".
    ///
    /// # Returns
    /// * `String` - The name of the GPU.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formated Name Gpu: {}", gpu.format_name_gpu());
    /// ```
    pub fn format_name_gpu(&self) -> String {
        self.name_gpu
            .as_ref()
            .map_or_else(|| "Unknown GPU".to_string(), |s| s.clone())
    }

    /// Returns formatted temperature of the GPU.
    ///
    /// If the temperature of the GPU is unknown, returns 0.0.
    ///
    /// # Returns
    /// * `f32` - The temperature of the GPU in degrees Celsius.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Temperature: {}", gpu.format_temperature());
    /// ```
    /// # Notes
    /// * The temperature is typically measured at the GPU die and represents the core temperature.
    /// * Different GPUs may report temperatures with varying precision.
    /// * Some GPUs may throttle performance at certain temperature thresholds.
    /// * The temperature reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    ///
    pub fn format_temperature(&self) -> f32 {
        self.temperature.unwrap_or(0.0)
    }

    /// Returns formatted utilization of the GPU.
    ///
    /// If the utilization of the GPU is unknown, returns 0.0.
    ///
    /// # Returns
    /// * `f32` - The utilization of the GPU as a percentage.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Utilization: {}", gpu.format_utilization());
    /// ```
    /// # Notes
    /// * The utilization is typically measured as a percentage of the GPU's maximum capacity.
    /// * Different GPUs may report utilization with varying precision.
    /// * Some GPUs may throttle performance at certain utilization thresholds.
    /// * The utilization reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_utilization(&self) -> f32 {
        self.utilization.unwrap_or(0.0)
    }

    /// Returns formatted power usage of the GPU.
    ///
    /// If the power usage of the GPU is unknown, returns 0.0.
    ///
    /// # Returns
    /// * `f32` - The power usage of the GPU in watts.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Power Usage: {}", gpu.format_power_usage());
    /// ```
    /// # Notes
    /// * The power usage is typically measured in watts and represents the current power consumption of the GPU.
    /// * Different GPUs may report power usage with varying precision.
    /// * Some GPUs may throttle performance at certain power usage thresholds.
    /// * The power usage reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_power_usage(&self) -> f32 {
        self.power_usage.unwrap_or(0.0)
    }

    /// Returns formatted core clock of the GPU.
    ///
    /// If the core clock of the GPU is unknown, returns 0.
    ///
    /// # Returns
    /// * `u32` - The core clock of the GPU in MHz.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Core Clock: {}", gpu.format_core_clock());
    /// ```
    /// # Notes
    /// * The core clock is typically measured in megahertz (MHz) and represents the current clock speed of the GPU's core.
    /// * Different GPUs may report core clock with varying precision.
    /// * Some GPUs may throttle performance at certain core clock speeds.
    /// * The core clock reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_core_clock(&self) -> u32 {
        self.core_clock.unwrap_or(0)
    }

    /// Returns formatted memory utilization of the GPU.
    ///
    /// If the memory utilization of the GPU is unknown, returns 0.0.
    ///
    /// # Returns
    /// * `f32` - The memory utilization of the GPU as a percentage.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Memory Utilization: {}", gpu.format_memory_util());
    /// ```
    /// # Notes
    /// * The memory utilization is typically measured as a percentage of the GPU's maximum memory capacity.
    /// * Different GPUs may report memory utilization with varying precision.
    /// * Some GPUs may throttle performance at certain memory utilization thresholds.
    /// * The memory utilization reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_memory_util(&self) -> f32 {
        self.memory_util.unwrap_or(0.0)
    }

    /// Returns formatted memory clock of the GPU.
    ///
    /// If the memory clock of the GPU is unknown, returns 0.
    ///
    /// # Returns
    /// * `u32` - The memory clock of the GPU in MHz.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Memory Clock: {}", gpu.format_memory_clock());
    /// ```
    /// # Notes
    /// * The memory clock is typically measured in megahertz (MHz) and represents the current clock speed of the GPU's memory.
    /// * Different GPUs may report memory clock with varying precision.
    /// * Some GPUs may throttle performance at certain memory clock speeds.
    /// * The memory clock reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_memory_clock(&self) -> u32 {
        self.memory_clock.unwrap_or(0)
    }

    /// Returns formatted active status of the GPU.
    ///
    /// If the active status of the GPU is unknown, returns "Unknown".
    ///
    /// # Returns
    /// * `String` - The active status of the GPU.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Active Status: {}", gpu.format_active());
    /// ```
    /// # Notes
    /// * The active status indicates whether the GPU is currently in use or idle.
    /// * Different GPUs may report active status with varying precision.
    /// * The active status reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_active(&self) -> String {
        if self.active == Some(true) {
            "Active".to_string()
        } else {
            "Inactive".to_string()
        }
    }

    /// Returns formatted power limit of the GPU.
    ///
    /// If the power limit of the GPU is unknown, returns 0.0.
    ///
    /// # Returns
    /// * `f32` - The power limit of the GPU in watts.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Power Limit: {}", gpu.format_power_limit());
    /// ```
    /// # Notes
    /// * The power limit is typically measured in watts and represents the maximum power consumption of the GPU.
    /// * Different GPUs may report power limit with varying precision.
    /// * Some GPUs may throttle performance at certain power limit thresholds.
    /// * The power limit reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_power_limit(&self) -> f32 {
        self.power_limit.unwrap_or(0.0)
    }

    /// Returns formatted total memory of the GPU.
    ///
    /// If the total memory of the GPU is unknown, returns 0.
    ///
    /// # Returns
    /// * `u32` - The total memory of the GPU in megabytes.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Memory Total: {}", gpu.format_memory_total());
    /// ```
    /// # Notes
    /// * The total memory is typically measured in megabytes (MB) and represents the maximum memory capacity of the GPU.
    /// * Different GPUs may report total memory with varying precision.
    /// * Some GPUs may throttle performance at certain memory usage thresholds.
    /// * The total memory reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_memory_total(&self) -> u32 {
        self.memory_total.unwrap_or(0)
    }

    /// Returns formatted driver version of the GPU.
    ///
    /// If the driver version of the GPU is unknown, returns "Unknown Driver Version".
    ///
    /// # Returns
    /// * `String` - The driver version of the GPU.
    ///
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Formatted Driver Version: {}", gpu.format_driver_version());
    /// ```
    /// # Notes
    /// * The driver version is typically reported by the GPU driver and represents the current version of the driver software.
    /// * Different GPUs may report driver version with varying precision.
    /// * Some GPUs may require specific driver versions for optimal performance.
    /// * The driver version reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_driver_version(&self) -> String {
        self.driver_version
            .as_ref()
            .map_or_else(|| "Unknown Driver Version".to_string(), |s| s.clone())
    }

    /// Returns the maximum clock speed of the GPU in MHz
    ///
    /// If the maximum clock speed is not available, returns 0.
    ///
    /// # Returns
    /// * type u32
    /// * The maximum clock speed in MHz, or 0 if not available
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// let max_clock_speed = gpu.format_max_clock_speed();
    /// max_clock_speed
    /// ```
    /// # Notes
    /// * The maximum clock speed is typically reported by the GPU driver and represents the highest clock speed the GPU can achieve.
    /// * Different GPUs may report maximum clock speed with varying precision.
    /// * Some GPUs may have dynamic clock speeds that change based on workload and temperature.
    /// * The maximum clock speed reading may have a small delay from real-time values.
    /// # Performance
    /// This is a lightweight accessor method that simply returns stored data.
    /// It performs no I/O operations or complex calculations.
    pub fn format_max_clock_speed(&self) -> u32 {
        self.max_clock_speed.unwrap_or(0)
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(temp) = self.temperature {
            if !(0.0..=1000.0).contains(&temp) {
                return Err(GpuError::InvalidTemperature(temp));
            }
        }

        if let Some(util) = self.utilization {
            if !(0.0..=100.0).contains(&util) {
                return Err(GpuError::InvalidUtilization(util));
            }
        }

        if let Some(power) = self.power_usage {
            if !(0.0..=1000.0).contains(&power) {
                return Err(GpuError::InvalidPowerUsage(power));
            }
        }

        if let Some(clock) = self.core_clock {
            if clock > 5000 {
                return Err(GpuError::InvalidClockSpeed(clock));
            }
        }

        if let Some(mem) = self.memory_total {
            if mem > 128 {
                return Err(GpuError::InvalidMemory(mem));
            }
        }

        Ok(())
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

impl Default for GpuInfo {
    /// Creates a new `GpuInfo` instance with all fields set to their default values.
    ///
    /// # Returns
    /// * `GpuInfo` - A new instance of `GpuInfo` with all fields set to their default values.
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

// Кэширование результатов
pub struct GpuInfoCache {
    info: RwLock<Option<(GpuInfo, Instant)>>,
    ttl: Duration,
}

impl GpuInfoCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            info: RwLock::new(None),
            ttl,
        }
    }

    pub fn get(&self) -> Option<GpuInfo> {
        let guard = self.info.read().ok()?;
        let (info, timestamp) = guard.as_ref()?;

        if timestamp.elapsed() < self.ttl {
            Some(info.clone())
        } else {
            None
        }
    }

    pub fn set(&self, info: GpuInfo) {
        if let Ok(mut guard) = self.info.write() {
            *guard = Some((info, Instant::now()));
        }
    }
}

impl Default for GpuInfoCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
