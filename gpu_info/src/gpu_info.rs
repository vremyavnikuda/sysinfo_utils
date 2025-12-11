use crate::vendor::Vendor;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use std::time::Duration;
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
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}
pub type Result<T> = std::result::Result<T, GpuError>;
/// Trait for unified GPU provider interface
pub trait GpuProvider: Send + Sync {
    /// Detect all GPUs provided by this provider
    fn detect_gpus(&self) -> Result<Vec<GpuInfo>>;
    /// Update the information for a specific GPU
    fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()>;
    /// Get the vendor associated with this provider
    fn get_vendor(&self) -> Vendor;
}
/// Handle empty vector result by converting to Result
///
/// This function eliminates duplication in provider implementations where
/// they need to convert an empty Vec to a GpuError::GpuNotFound error.
///
/// # Arguments
/// * `items` - Vector of items to check
///
/// # Returns
/// * `Ok(Vec<T>)` - If vector is not empty
/// * `Err(GpuError::GpuNotFound)` - If vector is empty
///
/// # Example
/// ```rust
/// use gpu_info::gpu_info::{handle_empty_result, GpuError, Result};
/// let items = vec![1, 2, 3];
/// let result: Result<Vec<i32>> = handle_empty_result(items);
/// assert!(result.is_ok());
/// let empty_items: Vec<i32> = vec![];
/// let result: Result<Vec<i32>> = handle_empty_result(empty_items);
/// assert!(matches!(result, Err(GpuError::GpuNotFound)));
/// ```
pub fn handle_empty_result<T>(items: Vec<T>) -> Result<Vec<T>> {
    if items.is_empty() {
        Err(GpuError::GpuNotFound)
    } else {
        Ok(items)
    }
}
/// Update GPU information using the common pattern
///
/// This function eliminates duplication in provider implementations where
/// they need to update a single GPU from a list of GPUs obtained from API.
///
/// # Arguments
/// * `gpu` - Mutable reference to GPU to update
/// * `api_gpus_fn` - Function that returns a vector of GPUs from API
///
/// # Returns
/// * `Ok(())` - If GPU was successfully updated
/// * `Err(GpuError)` - If update failed
///
/// # Example
/// ```rust
/// use gpu_info::gpu_info::{update_gpu_from_api, GpuInfo, Result};
/// fn update_example(gpu: &mut GpuInfo) -> Result<()> {
///     update_gpu_from_api(gpu, || vec![GpuInfo::unknown()])
/// }
/// ```
pub fn update_gpu_from_api<F>(gpu: &mut GpuInfo, api_gpus_fn: F) -> Result<()>
where
    F: FnOnce() -> Vec<GpuInfo>,
{
    let gpus = api_gpus_fn();
    if let Some(updated_gpu) = gpus.first() {
        *gpu = updated_gpu.clone();
        Ok(())
    } else {
        Err(GpuError::GpuNotActive)
    }
}
/// Trait for formatting values into string representation.
///
/// This trait defines a method for formatting GPU information fields
/// into human-readable strings, with "N/A" for unavailable values.
pub trait Formattable: Debug {
    fn fmt_string(&self) -> String;
}
/// All information gathered from the system about the current GPU.
#[derive(Debug, Clone, PartialEq)]
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
// Macros are defined in crate::macros and imported via #[macro_use]
/// Implementation of Formattable for Option<f32> with one decimal place formatting.
impl Formattable for Option<f32> {
    fn fmt_string(&self) -> String {
        match self {
            Some(value) => format!("{:.1}", value),
            None => String::from("N/A"),
        }
    }
}
// Use macro to implement Formattable for other Option<T> types
impl_formattable_for_option!(u32);
impl_formattable_for_option!(bool);
impl_formattable_for_option!(String);
impl Formattable for Option<&str> {
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
    /// use gpu_info::vendor::Vendor;
    /// use gpu_info::GpuInfo;
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
    /// pub enum Vendor {
    ///     Nvidia,
    /// }
    /// ```
    /// * `Vendor` - The vendor of the GPU. If the vendor cannot be determined,
    ///   returns `Vendor::Unknown`.
    /// # Example
    /// ```
    /// let gpu = gpu_info::get();
    /// println!("Vendor: {}", gpu.vendor())
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
    /// - The GPU driver doesn't report this information
    /// - The system doesn't have a dedicated GPU
    /// - The GPU information couldn't be accessed due to permission issues
    ///
    /// # Returns
    ///
    /// * `Some(&str)` - The full name of the GPU as a string slice.
    /// * `None` - If the GPU name could not be determined or is not available
    ///
    /// # Example
    /// ```rust
    /// let gpu = gpu_info::get();
    /// println!("GPU Name: {:?}", gpu.name_gpu());
    /// ```
    /// # Performance
    /// This is a lightweight accessor method that simply returns to stored data. It performs no I/O operations or complex calculations.
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
    /// ```rust
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
    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }
    /// Returns the current utilization of the GPU as a percentage.
    ///
    /// # Returns
    /// * `Some(f32)` - The current utilization of the GPU (0.0-100.0).
    /// * `None` - If the utilization of the GPU is unknown.
    ///
    /// # Example
    /// ```rust
    /// let gpu = gpu_info::get();
    /// println!("Utilization: {:?}", gpu.utilization());
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
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
    /// ```rust
    /// let gpu = gpu_info::get();
    /// println!("Max Clock Speed: {:?}", gpu.max_clock_speed());
    /// ```
    pub fn max_clock_speed(&self) -> Option<u32> {
        self.max_clock_speed
    }
    // format_name_gpu: Returns formatted name of the GPU. If unknown, returns "Unknown GPU".
    impl_format_string!(format_name_gpu, name_gpu, "Unknown GPU");

    // format_utilization: Returns formatted utilization (%). If unknown, returns 0.0.
    impl_format_numeric!(format_utilization, utilization, f32, 0.0);

    // format_power_usage: Returns formatted power usage (W). If unknown, returns 0.0.
    impl_format_numeric!(format_power_usage, power_usage, f32, 0.0);

    // format_core_clock: Returns formatted core clock (MHz). If unknown, returns 0.
    impl_format_numeric!(format_core_clock, core_clock, u32, 0);

    // format_memory_util: Returns formatted memory utilization (%). If unknown, returns 0.0.
    impl_format_numeric!(format_memory_util, memory_util, f32, 0.0);

    // format_memory_clock: Returns formatted memory clock (MHz). If unknown, returns 0.
    impl_format_numeric!(format_memory_clock, memory_clock, u32, 0);

    // format_power_limit: Returns formatted power limit (W). If unknown, returns 0.0.
    impl_format_numeric!(format_power_limit, power_limit, f32, 0.0);

    // format_memory_total: Returns formatted total memory (GB). If unknown, returns 0.
    impl_format_numeric!(format_memory_total, memory_total, u32, 0);

    // format_max_clock_speed: Returns formatted max clock speed (MHz). If unknown, returns 0.
    impl_format_numeric!(format_max_clock_speed, max_clock_speed, u32, 0);

    // format_driver_version: Returns formatted driver version. If unknown, returns default.
    impl_format_string!(
        format_driver_version,
        driver_version,
        "Unknown Driver Version"
    );

    // format_temperature: Returns formatted temperature (C). If unknown, returns 0.0.
    impl_format_numeric!(format_temperature, temperature, f32, 0.0);

    /// Returns formatted active status of the GPU.
    /// Returns "Active" if GPU is active, "Inactive" otherwise.
    ///
    /// Note: This method has custom logic and cannot be generated by macro.
    pub fn format_active(&self) -> String {
        if self.active == Some(true) {
            "Active".to_string()
        } else {
            "Inactive".to_string()
        }
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
            if mem > 131072 {
                // 128 GB in MB
                return Err(GpuError::InvalidMemory(mem));
            }
        }
        Ok(())
    }
    pub fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    /// Creates a new builder for constructing `GpuInfo` instances.
    ///
    /// The builder pattern provides an ergonomic API for creating GPU information
    /// objects with method chaining. All fields are optional and will default to
    /// `None` or unknown values if not set.
    ///
    /// # Returns
    ///
    /// A new `GpuInfoBuilder` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::{GpuInfo, vendor::Vendor};
    ///
    /// let gpu = GpuInfo::builder()
    ///     .vendor(Vendor::Nvidia)
    ///     .name("NVIDIA GeForce RTX 3080")
    ///     .temperature(65.0)
    ///     .utilization(75.0)
    ///     .power_usage(250.0)
    ///     .memory_total(10)
    ///     .build();
    ///
    /// assert_eq!(gpu.vendor(), Vendor::Nvidia);
    /// assert_eq!(gpu.temperature(), Some(65.0));
    /// ```
    pub fn builder() -> GpuInfoBuilder {
        GpuInfoBuilder::new()
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
        writeln!(f, "GPU Information:")?;
        writeln!(f, "  Vendor: {}", self.vendor)?;
        writeln!(f, "  Name: {}", self.name_gpu.fmt_string())?;
        writeln!(f, "  Temperature: {}", self.temperature.fmt_string())?;
        writeln!(f, "  Utilization: {}", self.utilization.fmt_string())?;
        writeln!(f, "  Power Usage: {}", self.power_usage.fmt_string())?;
        writeln!(f, "  Core Clock: {}", self.core_clock.fmt_string())?;
        writeln!(f, "  Memory Utilization: {}", self.memory_util.fmt_string())?;
        writeln!(f, "  Memory Clock: {}", self.memory_clock.fmt_string())?;
        writeln!(f, "  Active: {}", self.active.fmt_string())?;
        writeln!(f, "  Power Limit: {}", self.power_limit.fmt_string())?;
        writeln!(f, "  Memory Total: {}", self.memory_total.fmt_string())?;
        writeln!(f, "  Driver Version: {}", self.driver_version.fmt_string())?;
        writeln!(
            f,
            "  Max Clock Speed: {}",
            self.max_clock_speed.fmt_string()
        )?;
        Ok(())
    }
}

/// Builder for constructing [`GpuInfo`] instances with method chaining.
///
/// Provides an ergonomic API for building GPU information objects.
/// All fields are optional and will default to `None` or unknown values if not set.
///
/// # Examples
///
/// ```
/// use gpu_info::{GpuInfo, vendor::Vendor};
///
/// let gpu = GpuInfo::builder()
///     .vendor(Vendor::Nvidia)
///     .name("NVIDIA GeForce RTX 3080")
///     .temperature(65.0)
///     .utilization(75.0)
///     .power_usage(250.0)
///     .core_clock(1710)
///     .memory_total(10)
///     .active(true)
///     .build();
///
/// assert_eq!(gpu.vendor(), Vendor::Nvidia);
/// assert_eq!(gpu.name_gpu(), Some("NVIDIA GeForce RTX 3080"));
/// assert_eq!(gpu.temperature(), Some(65.0));
/// ```
#[derive(Debug, Clone, Default)]
pub struct GpuInfoBuilder {
    vendor: Option<Vendor>,
    name_gpu: Option<String>,
    temperature: Option<f32>,
    utilization: Option<f32>,
    power_usage: Option<f32>,
    core_clock: Option<u32>,
    memory_util: Option<f32>,
    memory_clock: Option<u32>,
    active: Option<bool>,
    power_limit: Option<f32>,
    memory_total: Option<u32>,
    driver_version: Option<String>,
    max_clock_speed: Option<u32>,
}

impl GpuInfoBuilder {
    /// Creates a new empty builder.
    ///
    /// # Returns
    ///
    /// A new `GpuInfoBuilder` instance with all fields set to `None`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the GPU vendor.
    ///
    /// # Arguments
    ///
    /// * `vendor` - The vendor of the GPU (e.g., NVIDIA, AMD, Intel).
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn vendor(mut self, vendor: Vendor) -> Self {
        self.vendor = Some(vendor);
        self
    }

    /// Sets the GPU name.
    ///
    /// # Arguments
    ///
    /// * `name` - The full name of the GPU.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name_gpu = Some(name.into());
        self
    }

    /// Sets the GPU temperature in degrees Celsius.
    ///
    /// # Arguments
    ///
    /// * `temperature` - The current temperature of the GPU.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the GPU utilization percentage.
    ///
    /// # Arguments
    ///
    /// * `utilization` - The current utilization of the GPU (0.0-100.0).
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn utilization(mut self, utilization: f32) -> Self {
        self.utilization = Some(utilization);
        self
    }

    /// Sets the GPU power usage in watts.
    ///
    /// # Arguments
    ///
    /// * `power_usage` - The current power consumption of the GPU.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn power_usage(mut self, power_usage: f32) -> Self {
        self.power_usage = Some(power_usage);
        self
    }

    /// Sets the GPU core clock speed in MHz.
    ///
    /// # Arguments
    ///
    /// * `core_clock` - The current core clock speed.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn core_clock(mut self, core_clock: u32) -> Self {
        self.core_clock = Some(core_clock);
        self
    }

    /// Sets the GPU memory utilization percentage.
    ///
    /// # Arguments
    ///
    /// * `memory_util` - The current memory utilization (0.0-100.0).
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn memory_util(mut self, memory_util: f32) -> Self {
        self.memory_util = Some(memory_util);
        self
    }

    /// Sets the GPU memory clock speed in MHz.
    ///
    /// # Arguments
    ///
    /// * `memory_clock` - The current memory clock speed.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn memory_clock(mut self, memory_clock: u32) -> Self {
        self.memory_clock = Some(memory_clock);
        self
    }

    /// Sets whether the GPU is currently active.
    ///
    /// # Arguments
    ///
    /// * `active` - Whether the GPU is in use.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Sets the GPU power limit in watts.
    ///
    /// # Arguments
    ///
    /// * `power_limit` - The power consumption limit.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn power_limit(mut self, power_limit: f32) -> Self {
        self.power_limit = Some(power_limit);
        self
    }

    /// Sets the total GPU memory in gigabytes.
    ///
    /// # Arguments
    ///
    /// * `memory_total` - The total memory capacity.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn memory_total(mut self, memory_total: u32) -> Self {
        self.memory_total = Some(memory_total);
        self
    }

    /// Sets the GPU driver version.
    ///
    /// # Arguments
    ///
    /// * `driver_version` - The version string of the GPU driver.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn driver_version(mut self, driver_version: impl Into<String>) -> Self {
        self.driver_version = Some(driver_version.into());
        self
    }

    /// Sets the maximum GPU clock speed in MHz.
    ///
    /// # Arguments
    ///
    /// * `max_clock_speed` - The maximum clock speed.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn max_clock_speed(mut self, max_clock_speed: u32) -> Self {
        self.max_clock_speed = Some(max_clock_speed);
        self
    }

    /// Builds the [`GpuInfo`] instance.
    ///
    /// All unset fields will default to their unknown values:
    /// - `vendor`: `Vendor::Unknown`
    /// - All other fields: `None`
    ///
    /// # Returns
    ///
    /// A new `GpuInfo` instance.
    pub fn build(self) -> GpuInfo {
        GpuInfo {
            vendor: self.vendor.unwrap_or(Vendor::Unknown),
            name_gpu: self.name_gpu,
            temperature: self.temperature,
            utilization: self.utilization,
            power_usage: self.power_usage,
            core_clock: self.core_clock,
            memory_util: self.memory_util,
            memory_clock: self.memory_clock,
            active: self.active,
            power_limit: self.power_limit,
            memory_total: self.memory_total,
            driver_version: self.driver_version,
            max_clock_speed: self.max_clock_speed,
        }
    }
}

/// GPU information cache that uses unified caching utilities
///
/// This cache eliminates duplication by using the common caching infrastructure.
pub struct GpuInfoCache {
    /// Underlying cache implementation
    cache: crate::cache_utils::GpuInfoCache,
}
impl GpuInfoCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: crate::cache_utils::GpuInfoCache::new(ttl),
        }
    }
    pub fn get(&self) -> Option<Arc<GpuInfo>> {
        self.cache.get()
    }

    pub fn get_owned(&self) -> Option<GpuInfo> {
        self.cache.get_owned()
    }
    pub fn set(&self, info: GpuInfo) {
        self.cache.set(info);
    }
}
impl Default for GpuInfoCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}
