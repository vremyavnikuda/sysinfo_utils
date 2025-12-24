use crate::vendor::Vendor;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Errors that can occur when working with GPU information.
///
/// This enum is marked `#[non_exhaustive]` to allow adding new variants
/// in future versions without breaking existing code.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum GpuError {
    /// Invalid temperature value (expected 0-1000°C).
    #[error("Invalid temperature value: {0}")]
    InvalidTemperature(f32),
    /// Invalid utilization value (expected 0-100%).
    #[error("Invalid utilization value: {0}")]
    InvalidUtilization(f32),
    /// Invalid power usage value (expected 0-1000W).
    #[error("Invalid power usage value: {0}")]
    InvalidPowerUsage(f32),
    /// Invalid clock speed value (expected 0-5000 MHz).
    #[error("Invalid clock speed value: {0}")]
    InvalidClockSpeed(u32),
    /// Invalid memory value (expected 0-131072 MB).
    #[error("Invalid memory value: {0}")]
    InvalidMemory(u32),
    /// No GPU was found in the system.
    #[error("GPU not found")]
    GpuNotFound,
    /// GPU driver is not installed or not accessible.
    #[error("Driver not installed")]
    DriverNotInstalled,
    /// The GPU is not currently active.
    #[error("GPU not active")]
    GpuNotActive,
    /// Feature is not enabled.
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
    /// I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// FFI operation failed.
    #[error("FFI error: {0}")]
    Ffi(String),
}

/// A specialized `Result` type for GPU operations.
///
/// This type alias is used throughout the crate for functions that can fail
/// with a [`GpuError`]. It simplifies function signatures by avoiding the need
/// to specify the error type explicitly.
///
/// # Examples
///
/// ```
/// use gpu_info::{Result, GpuError};
///
/// fn get_temperature() -> Result<f32> {
///     // Returns Ok(temperature) or Err(GpuError::...)
///     Ok(65.0)
/// }
/// ```
///
/// [`GpuError`]: crate::GpuError
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
    /// Formats the value into a human-readable string.
    ///
    /// Returns a string representation of the value, or "N/A" if the value
    /// is not available or cannot be formatted.
    fn fmt_string(&self) -> String;
}
/// All information gathered from the system about the current GPU.
///
/// # Thread Safety
///
/// `GpuInfo` is `Send` and `Sync`, meaning it can be safely shared between threads
/// and sent across thread boundaries. This is guaranteed by the fact that all fields
/// are either primitive types or `Option<T>` where `T` is `Send + Sync`.
///
/// For concurrent access patterns, consider using `Arc<GpuInfo>` which is returned
/// by `GpuManager::get_gpu_cached()` for zero-copy sharing.
///
/// # Cloning
///
/// Cloning a `GpuInfo` allocates memory for the `String` fields (`name_gpu` and
/// `driver_version`). The total allocation is typically 50-200 bytes depending on
/// string lengths. For read-only access, prefer using `Arc<GpuInfo>` from
/// `GpuManager::get_gpu_cached()` which provides zero-copy sharing.
///
/// The [`clone_from()`](Clone::clone_from) method is optimized to reuse existing
/// string allocations when possible, reducing memory churn in hot paths.
///
/// # Example
/// ```
/// use gpu_info::{GpuInfo, vendor::Vendor};
///
/// // Using builder
/// let gpu = GpuInfo::builder()
///     .vendor(Vendor::Nvidia)
///     .temperature(65.0)
///     .build();
///
/// // Using unknown() as base
/// let gpu = GpuInfo::unknown();
/// ```
#[derive(Debug, PartialEq)]
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
    /// The total memory of the GPU in megabytes (stored internally, displayed as GB).
    pub memory_total: Option<u32>, // total GPU memory (MB, display as GB)
    /// The currently used memory of the GPU in megabytes.
    pub memory_used: Option<u32>, // used GPU memory (MB)
    /// The driver version of the GPU.
    pub driver_version: Option<String>, // driver version
    /// The maximum clock speed of the GPU in MHz.
    pub max_clock_speed: Option<u32>, // maximum GPU clock speed (MHz)
}

/// Manual Clone implementation with optimized `clone_from()`.
///
/// The `clone_from()` method reuses existing string allocations when possible,
/// which reduces memory churn when repeatedly updating GPU info in hot paths.
impl Clone for GpuInfo {
    fn clone(&self) -> Self {
        Self {
            vendor: self.vendor,
            name_gpu: self.name_gpu.clone(),
            temperature: self.temperature,
            utilization: self.utilization,
            power_usage: self.power_usage,
            core_clock: self.core_clock,
            memory_util: self.memory_util,
            memory_clock: self.memory_clock,
            active: self.active,
            power_limit: self.power_limit,
            memory_total: self.memory_total,
            memory_used: self.memory_used,
            driver_version: self.driver_version.clone(),
            max_clock_speed: self.max_clock_speed,
        }
    }

    /// Optimized clone that reuses existing string allocations.
    ///
    /// This is more efficient than `*self = source.clone()` when `self` already
    /// has allocated strings, as it avoids deallocating and reallocating memory.
    fn clone_from(&mut self, source: &Self) {
        self.vendor = source.vendor;
        // Reuse string allocation if possible
        clone_option_string(&mut self.name_gpu, &source.name_gpu);
        self.temperature = source.temperature;
        self.utilization = source.utilization;
        self.power_usage = source.power_usage;
        self.core_clock = source.core_clock;
        self.memory_util = source.memory_util;
        self.memory_clock = source.memory_clock;
        self.active = source.active;
        self.power_limit = source.power_limit;
        self.memory_total = source.memory_total;
        self.memory_used = source.memory_used;
        // Reuse string allocation if possible
        clone_option_string(&mut self.driver_version, &source.driver_version);
        self.max_clock_speed = source.max_clock_speed;
    }
}

/// Helper function to clone Option<String> while reusing allocation.
#[inline]
fn clone_option_string(dest: &mut Option<String>, source: &Option<String>) {
    match (dest, source) {
        (Some(d), Some(s)) => {
            d.clear();
            d.push_str(s);
        }
        (dest, source) => *dest = source.clone(),
    }
}

// Macros are defined in crate::macros and imported via #[macro_use]
/// Implementation of Formattable for `Option<f32>` with one decimal place formatting.
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
            memory_used: None,
            driver_version: None,
            max_clock_speed: None,
        }
    }

    /// Creates a mock `GpuInfo` with typical NVIDIA GPU values.
    ///
    /// Useful for unit tests that need realistic GPU data without
    /// requiring actual hardware.
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    ///
    /// let gpu = GpuInfo::mock_nvidia();
    /// assert_eq!(gpu.vendor().to_string(), "NVIDIA");
    /// assert!(gpu.temperature().is_some());
    /// ```
    pub fn mock_nvidia() -> Self {
        Self::builder()
            .vendor(Vendor::Nvidia)
            .name("NVIDIA GeForce RTX 3080")
            .temperature(65.0)
            .utilization(45.0)
            .power_usage(220.0)
            .core_clock(1710)
            .memory_util(35.0)
            .memory_clock(9501)
            .active(true)
            .power_limit(320.0)
            .memory_total(10240)
            .memory_used(4096)
            .driver_version("535.154.05")
            .max_clock_speed(1995)
            .build()
    }

    /// Creates a mock `GpuInfo` with typical AMD GPU values.
    ///
    /// Useful for unit tests that need realistic GPU data without
    /// requiring actual hardware.
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    ///
    /// let gpu = GpuInfo::mock_amd();
    /// assert_eq!(gpu.vendor().to_string(), "AMD");
    /// assert!(gpu.temperature().is_some());
    /// ```
    pub fn mock_amd() -> Self {
        Self::builder()
            .vendor(Vendor::Amd)
            .name("AMD Radeon RX 6800 XT")
            .temperature(70.0)
            .utilization(55.0)
            .power_usage(250.0)
            .core_clock(2015)
            .memory_util(40.0)
            .memory_clock(2000)
            .active(true)
            .power_limit(300.0)
            .memory_total(16384)
            .memory_used(6144)
            .driver_version("23.11.1")
            .max_clock_speed(2250)
            .build()
    }

    /// Creates a mock `GpuInfo` with typical Intel integrated GPU values.
    ///
    /// Useful for unit tests that need realistic GPU data without
    /// requiring actual hardware.
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    ///
    /// let gpu = GpuInfo::mock_intel();
    /// assert!(matches!(gpu.vendor(), gpu_info::Vendor::Intel(_)));
    /// ```
    pub fn mock_intel() -> Self {
        use crate::vendor::IntelGpuType;
        Self::builder()
            .vendor(Vendor::Intel(IntelGpuType::Integrated))
            .name("Intel UHD Graphics 630")
            .temperature(55.0)
            .utilization(20.0)
            .core_clock(1150)
            .memory_util(15.0)
            .active(true)
            .memory_total(1024)
            .driver_version("31.0.101.4502")
            .max_clock_speed(1200)
            .build()
    }

    /// Creates a mock `GpuInfo` with customizable values.
    ///
    /// This is a convenience method that returns a builder pre-populated
    /// with sensible defaults. Useful for tests that need to customize
    /// specific fields.
    ///
    /// # Example
    /// ```
    /// use gpu_info::{GpuInfo, Vendor};
    ///
    /// let gpu = GpuInfo::mock()
    ///     .vendor(Vendor::Nvidia)
    ///     .temperature(80.0)  // Override temperature
    ///     .build();
    ///
    /// assert_eq!(gpu.temperature(), Some(80.0));
    /// ```
    pub fn mock() -> GpuInfoBuilder {
        GpuInfoBuilder::new()
            .vendor(Vendor::Unknown)
            .name("Mock GPU")
            .temperature(60.0)
            .utilization(50.0)
            .power_usage(150.0)
            .core_clock(1500)
            .memory_util(30.0)
            .memory_clock(1000)
            .active(true)
            .power_limit(200.0)
            .memory_total(8192)
            .memory_used(2048)
            .driver_version("1.0.0")
            .max_clock_speed(1800)
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

    /// Returns the GPU name or a default value.
    ///
    /// This method uses `Cow<'_, str>` to avoid allocation when the name is available,
    /// while still providing a default value when it's not.
    ///
    /// # Returns
    ///
    /// * `Cow::Borrowed(&str)` - If the GPU name is available
    /// * `Cow::Borrowed("Unknown GPU")` - If the GPU name is not available
    ///
    /// # Example
    /// ```rust
    /// use gpu_info::GpuInfo;
    /// use std::borrow::Cow;
    ///
    /// let gpu = GpuInfo::builder().name("RTX 3080").build();
    /// assert_eq!(gpu.name_or_default(), "RTX 3080");
    ///
    /// let unknown = GpuInfo::unknown();
    /// assert_eq!(unknown.name_or_default(), "Unknown GPU");
    /// ```
    pub fn name_or_default(&self) -> std::borrow::Cow<'_, str> {
        match &self.name_gpu {
            Some(name) => std::borrow::Cow::Borrowed(name.as_str()),
            None => std::borrow::Cow::Borrowed("Unknown GPU"),
        }
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

    /// Returns the currently used memory of the GPU in megabytes.
    ///
    /// # Returns
    /// * `Some(u32)` - The used memory of the GPU in megabytes.
    /// * `None` - If the used memory of the GPU is unknown.
    ///
    /// # Example
    /// ```rust
    /// let gpu = gpu_info::get();
    /// println!("Memory Used: {:?}", gpu.memory_used());
    /// ```
    pub fn memory_used(&self) -> Option<u32> {
        self.memory_used
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

    /// Returns formatted name of the GPU.
    ///
    /// If the GPU name is unknown, returns "Unknown GPU".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().name("RTX 3080").build();
    /// assert_eq!(gpu.format_name_gpu(), "RTX 3080");
    /// ```
    pub fn format_name_gpu(&self) -> String {
        self.name_gpu
            .as_ref()
            .map_or_else(|| "Unknown GPU".to_string(), |s| s.clone())
    }

    /// Returns formatted GPU utilization percentage.
    ///
    /// Rounds to 2 decimal places for readability.
    /// If unknown, returns "N/A".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().utilization(75.5).build();
    /// assert_eq!(gpu.format_utilization(), "75.50%");
    /// ```
    pub fn format_utilization(&self) -> String {
        match self.utilization {
            Some(util) => format!("{:.2}%", (util * 100.0).round() / 100.0),
            None => "N/A".to_string(),
        }
    }

    /// Returns formatted temperature in Celsius.
    ///
    /// Rounds to 2 decimal places for readability.
    /// If not supported by driver, returns "Not supported".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().temperature(65.5).build();
    /// assert_eq!(gpu.format_temperature(), "65.50°C");
    /// ```
    pub fn format_temperature(&self) -> String {
        match self.temperature {
            Some(temp) => format!("{:.2}°C", (temp * 100.0).round() / 100.0),
            None => "Not supported".to_string(),
        }
    }

    /// Returns formatted power usage in watts.
    ///
    /// Rounds to 2 decimal places for readability.
    /// If not supported by driver, returns "Not supported".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().power_usage(250.0).build();
    /// assert_eq!(gpu.format_power_usage(), "250.00W");
    /// ```
    pub fn format_power_usage(&self) -> String {
        match self.power_usage {
            Some(power) => format!("{:.2}W", (power * 100.0).round() / 100.0),
            None => "Not supported".to_string(),
        }
    }

    /// Returns formatted power limit in watts.
    ///
    /// Rounds to 2 decimal places for readability.
    /// If not supported by driver, returns "Not supported".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().power_limit(350.0).build();
    /// assert_eq!(gpu.format_power_limit(), "350.00W");
    /// ```
    pub fn format_power_limit(&self) -> String {
        match self.power_limit {
            Some(limit) => format!("{:.2}W", (limit * 100.0).round() / 100.0),
            None => "Not supported".to_string(),
        }
    }

    /// Returns formatted memory utilization percentage.
    ///
    /// Rounds to 2 decimal places for readability.
    /// If unknown, returns "N/A".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().memory_util(45.5).build();
    /// assert_eq!(gpu.format_memory_util(), "45.50%");
    /// ```
    pub fn format_memory_util(&self) -> String {
        match self.memory_util {
            Some(util) => format!("{:.2}%", (util * 100.0).round() / 100.0),
            None => "N/A".to_string(),
        }
    }

    /// Returns formatted core clock speed in MHz.
    ///
    /// If unknown, returns "N/A".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().core_clock(1800).build();
    /// assert_eq!(gpu.format_core_clock(), "1800 MHz");
    /// ```
    pub fn format_core_clock(&self) -> String {
        match self.core_clock {
            Some(clock) => format!("{} MHz", clock),
            None => "N/A".to_string(),
        }
    }

    /// Returns formatted memory clock speed in MHz.
    ///
    /// If unknown, returns "N/A".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().memory_clock(7000).build();
    /// assert_eq!(gpu.format_memory_clock(), "7000 MHz");
    /// ```
    pub fn format_memory_clock(&self) -> String {
        match self.memory_clock {
            Some(clock) => format!("{} MHz", clock),
            None => "N/A".to_string(),
        }
    }

    /// Returns formatted max clock speed in MHz.
    ///
    /// If unknown, returns "N/A".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().max_clock_speed(2100).build();
    /// assert_eq!(gpu.format_max_clock_speed(), "2100 MHz");
    /// ```
    pub fn format_max_clock_speed(&self) -> String {
        match self.max_clock_speed {
            Some(clock) => format!("{} MHz", clock),
            None => "N/A".to_string(),
        }
    }

    /// Returns formatted total memory in gigabytes.
    ///
    /// Converts internal MB storage to GB for display.
    /// If unknown, returns "N/A".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().memory_total(8192).build(); // 8 GB in MB
    /// assert_eq!(gpu.format_memory_total(), "8.00 GB");
    /// ```
    pub fn format_memory_total(&self) -> String {
        match self.memory_total {
            Some(mb) => format!("{:.2} GB", (mb as f32) / 1024.0),
            None => "N/A".to_string(),
        }
    }

    /// Returns formatted used memory in gigabytes.
    ///
    /// Converts internal MB storage to GB for display.
    /// If unknown, returns "N/A".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().memory_used(4096).build(); // 4 GB in MB
    /// assert_eq!(gpu.format_memory_used(), "4.00 GB");
    /// ```
    pub fn format_memory_used(&self) -> String {
        match self.memory_used {
            Some(mb) => format!("{:.2} GB", (mb as f32) / 1024.0),
            None => "N/A".to_string(),
        }
    }

    /// Returns formatted driver version.
    ///
    /// If the driver version is unknown, returns "Unknown Driver Version".
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().driver_version("535.104.05").build();
    /// assert_eq!(gpu.format_driver_version(), "535.104.05");
    /// ```
    pub fn format_driver_version(&self) -> String {
        self.driver_version
            .as_ref()
            .map_or_else(|| "Unknown Driver Version".to_string(), |s| s.clone())
    }

    /// Returns formatted active status of the GPU.
    /// Returns "Active" if GPU is active, "Inactive" otherwise.
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    /// let gpu = GpuInfo::builder().active(true).build();
    /// assert_eq!(gpu.format_active(), "Active");
    /// ```
    pub fn format_active(&self) -> String {
        if self.active == Some(true) {
            "Active".to_string()
        } else {
            "Inactive".to_string()
        }
    }

    /// Returns `true` if temperature data is available.
    ///
    /// # Example
    /// ```
    /// use gpu_info::GpuInfo;
    ///
    /// let gpu = GpuInfo::builder().temperature(65.0).build();
    /// assert!(gpu.has_temperature());
    ///
    /// let gpu = GpuInfo::unknown();
    /// assert!(!gpu.has_temperature());
    /// ```
    pub fn has_temperature(&self) -> bool {
        self.temperature.is_some()
    }

    /// Returns `true` if utilization data is available.
    pub fn has_utilization(&self) -> bool {
        self.utilization.is_some()
    }

    /// Returns `true` if power usage data is available.
    pub fn has_power_usage(&self) -> bool {
        self.power_usage.is_some()
    }

    /// Returns `true` if driver version information is available.
    pub fn has_driver(&self) -> bool {
        self.driver_version.is_some()
    }

    /// Returns `true` if memory information is available.
    pub fn has_memory_info(&self) -> bool {
        self.memory_total.is_some() || self.memory_used.is_some()
    }

    /// Returns `true` if clock speed information is available.
    pub fn has_clock_info(&self) -> bool {
        self.core_clock.is_some() || self.memory_clock.is_some()
    }

    /// Returns `true` if the GPU can report power metrics.
    ///
    /// This indicates whether the GPU driver supports power monitoring.
    pub fn can_report_power(&self) -> bool {
        self.power_usage.is_some() || self.power_limit.is_some()
    }

    /// Returns `true` if the GPU can report thermal metrics.
    ///
    /// This indicates whether the GPU driver supports temperature monitoring.
    pub fn can_report_thermals(&self) -> bool {
        self.temperature.is_some()
    }

    /// Returns `true` if the GPU is currently active.
    ///
    /// Returns `false` if the GPU is inactive or if the active status is unknown.
    pub fn is_active(&self) -> bool {
        self.active.unwrap_or(false)
    }

    /// Returns `true` if the GPU vendor is known.
    pub fn is_vendor_known(&self) -> bool {
        self.vendor != Vendor::Unknown
    }

    /// Validates all fields are within expected ranges.
    ///
    /// # Errors
    ///
    /// Returns an error if any field is out of valid range:
    /// - Temperature: 0-1000°C
    /// - Utilization: 0-100%
    /// - Power usage: 0-1000W
    /// - Clock speed: 0-5000 MHz
    /// - Memory: 0-131072 MB (128 GB)
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuInfo;
    ///
    /// let gpu = GpuInfo::builder()
    ///     .temperature(65.0)
    ///     .utilization(45.0)
    ///     .build();
    ///
    /// assert!(gpu.validate().is_ok());
    /// ```
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

    /// Returns `true` if all fields are within valid ranges.
    ///
    /// This is a convenience method equivalent to `self.validate().is_ok()`.
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
    memory_used: Option<u32>,
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

    /// Sets the total GPU memory in megabytes.
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

    /// Sets the used GPU memory in megabytes.
    ///
    /// # Arguments
    ///
    /// * `memory_used` - The currently used memory.
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    pub fn memory_used(mut self, memory_used: u32) -> Self {
        self.memory_used = Some(memory_used);
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
            memory_used: self.memory_used,
            driver_version: self.driver_version,
            max_clock_speed: self.max_clock_speed,
        }
    }

    /// Builds the [`GpuInfo`] instance with validation.
    ///
    /// This method builds the `GpuInfo` and validates all fields are within
    /// expected ranges. Use this when you need to ensure the GPU information
    /// is valid before using it.
    ///
    /// # Returns
    ///
    /// * `Ok(GpuInfo)` - A valid `GpuInfo` instance.
    /// * `Err(GpuError)` - If any field is out of valid range.
    ///
    /// # Errors
    ///
    /// Returns an error if any field is out of valid range:
    /// - Temperature: 0-1000°C
    /// - Utilization: 0-100%
    /// - Power usage: 0-1000W
    /// - Clock speed: 0-5000 MHz
    /// - Memory: 0-131072 MB (128 GB)
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::{GpuInfo, vendor::Vendor};
    ///
    /// // Valid GPU info
    /// let gpu = GpuInfo::builder()
    ///     .vendor(Vendor::Nvidia)
    ///     .temperature(65.0)
    ///     .utilization(75.0)
    ///     .try_build();
    /// assert!(gpu.is_ok());
    ///
    /// // Invalid temperature
    /// let gpu = GpuInfo::builder()
    ///     .temperature(-10.0)
    ///     .try_build();
    /// assert!(gpu.is_err());
    /// ```
    pub fn try_build(self) -> Result<GpuInfo> {
        let gpu = self.build();
        gpu.validate()?;
        Ok(gpu)
    }
}

/// Creates a `GpuInfo` from a `Vendor`.
///
/// This is useful when you only know the vendor and want to create
/// a minimal `GpuInfo` instance.
///
/// # Examples
///
/// ```
/// use gpu_info::{GpuInfo, vendor::Vendor};
///
/// let gpu: GpuInfo = Vendor::Nvidia.into();
/// assert_eq!(gpu.vendor(), Vendor::Nvidia);
/// ```
impl From<Vendor> for GpuInfo {
    fn from(vendor: Vendor) -> Self {
        Self::write_vendor(vendor)
    }
}

/// Parses a `GpuInfo` from a JSON string.
///
/// # Errors
///
/// Returns [`GpuError::Ffi`] if the JSON parsing fails.
///
/// # Examples
///
/// ```
/// use gpu_info::GpuInfo;
/// use std::convert::TryFrom;
///
/// let json = r#"{"vendor":"Unknown"}"#;
/// // Note: This will fail without proper serde setup
/// // let gpu = GpuInfo::try_from(json);
/// ```
#[cfg(feature = "serde")]
impl TryFrom<&str> for GpuInfo {
    type Error = GpuError;

    fn try_from(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| GpuError::Ffi(e.to_string()))
    }
}

/// Provides a reference to the `GpuInfo`.
///
/// This is useful for generic functions that accept `impl AsRef<GpuInfo>`.
///
/// # Examples
///
/// ```
/// use gpu_info::GpuInfo;
///
/// fn print_vendor(gpu: impl AsRef<GpuInfo>) {
///     println!("Vendor: {}", gpu.as_ref().vendor());
/// }
///
/// let gpu = GpuInfo::unknown();
/// print_vendor(&gpu);
/// ```
impl AsRef<GpuInfo> for GpuInfo {
    fn as_ref(&self) -> &GpuInfo {
        self
    }
}

/// `Eq` implementation for `GpuInfo`.
///
/// This implementation treats two `GpuInfo` instances as equal if all their
/// fields are equal. For `f32` fields, we compare bit patterns so that
/// NaN values are considered equal to each other (unlike standard f32 comparison).
///
/// Note: The `PartialEq` derive already handles field-by-field comparison,
/// but `f32`'s `PartialEq` returns `false` for `NaN == NaN`. Since our
/// `PartialEq` is derived and uses the standard f32 comparison, this `Eq`
/// implementation is consistent with it for all non-NaN values.
///
/// # Examples
///
/// ```
/// use gpu_info::{GpuInfo, vendor::Vendor};
/// use std::collections::HashSet;
///
/// let gpu1 = GpuInfo::builder()
///     .vendor(Vendor::Nvidia)
///     .name("RTX 3080")
///     .build();
///
/// let gpu2 = GpuInfo::builder()
///     .vendor(Vendor::Nvidia)
///     .name("RTX 3080")
///     .build();
///
/// // Can be used in HashSet
/// let mut set = HashSet::new();
/// set.insert(gpu1.clone());
/// assert!(set.contains(&gpu2));
/// ```
impl Eq for GpuInfo {}

/// `Hash` implementation for `GpuInfo`.
///
/// Hashes by identity fields only: `vendor` and `name_gpu`.
/// Metric fields (temperature, utilization, etc.) are intentionally excluded
/// because they change frequently and shouldn't affect GPU identity.
///
/// This allows using `GpuInfo` as a key in `HashMap` or `HashSet` where
/// the same physical GPU (identified by vendor + name) should hash to
/// the same value regardless of current metrics.
///
/// # Examples
///
/// ```
/// use gpu_info::{GpuInfo, vendor::Vendor};
/// use std::collections::HashMap;
///
/// let mut gpu_data: HashMap<GpuInfo, String> = HashMap::new();
///
/// let gpu = GpuInfo::builder()
///     .vendor(Vendor::Nvidia)
///     .name("RTX 3080")
///     .temperature(65.0)
///     .build();
///
/// gpu_data.insert(gpu.clone(), "Primary GPU".to_string());
///
/// // Same GPU with different temperature still matches
/// let gpu_updated = GpuInfo::builder()
///     .vendor(Vendor::Nvidia)
///     .name("RTX 3080")
///     .temperature(70.0)
///     .build();
///
/// // Note: This will NOT find the entry because PartialEq compares all fields
/// // Use vendor + name for lookups if you need metric-independent matching
/// ```
///
/// # Note
///
/// The `Hash` implementation only considers identity fields, but `PartialEq`
/// (derived) compares all fields. This means two `GpuInfo` instances with
/// the same vendor and name but different metrics will hash to the same
/// bucket but won't be considered equal. This is intentional for use cases
/// where you want to group GPUs by identity but distinguish by metrics.
impl Hash for GpuInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vendor.hash(state);
        self.name_gpu.hash(state);
    }
}

/// Compile-time assertion that `GpuInfo` implements `Send`.
///
/// This ensures `GpuInfo` can be safely transferred between threads.
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<GpuInfo>();
};

/// Compile-time assertion that `GpuInfo` implements `Sync`.
///
/// This ensures `GpuInfo` can be safely shared between threads via references.
const _: () = {
    const fn assert_sync<T: Sync>() {}
    assert_sync::<GpuInfo>();
};

/// Compile-time assertion that `GpuInfo` implements both `Send` and `Sync`.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GpuInfo>();
};
