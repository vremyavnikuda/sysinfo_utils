use std::fmt::Display;
use std::str::FromStr;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
/// GPU vendor information.
///
/// Represents the manufacturer of a GPU. This enum is `#[non_exhaustive]`
/// to allow adding new vendors in future versions without breaking changes.
///
/// # Examples
///
/// ```
/// use gpu_info::vendor::Vendor;
/// use std::str::FromStr;
///
/// let vendor = Vendor::from_str("nvidia").unwrap();
/// assert_eq!(vendor, Vendor::Nvidia);
///
/// let vendor: Vendor = "AMD".parse().unwrap();
/// assert_eq!(vendor, Vendor::Amd);
/// ```
pub enum Vendor {
    /// NVIDIA Corporation GPUs (GeForce, Quadro, Tesla, etc.)
    Nvidia,
    /// AMD (Advanced Micro Devices) GPUs (Radeon, FirePro, etc.)
    Amd,
    /// Intel Corporation GPUs (integrated or discrete Arc)
    Intel(IntelGpuType),
    /// Apple Silicon GPUs (M1, M2, M3, etc.)
    Apple,
    /// Unknown or unrecognized GPU vendor
    Unknown,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
/// Intel GPU type classification.
///
/// Distinguishes between integrated graphics (built into the CPU)
/// and discrete graphics (separate GPU like Intel Arc).
///
/// This enum is `#[non_exhaustive]` to allow adding new Intel GPU
/// types in future versions without breaking changes.
pub enum IntelGpuType {
    /// Integrated graphics (Intel UHD, Iris, HD Graphics)
    Integrated,
    /// Discrete graphics (Intel Arc series)
    Discrete,
    /// Unknown Intel GPU type
    #[default]
    Unknown,
}
/// Converts a string to a Vendor enum
impl Default for Vendor {
    fn default() -> Self {
        Vendor::Unknown
    }
}

/// Display trait implementation for Vendor enum
impl Display for Vendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Vendor::Nvidia => write!(f, "NVIDIA"),
            Vendor::Amd => write!(f, "AMD"),
            Vendor::Intel(gpu_type) => write!(f, "INTEL ({})", gpu_type),
            Vendor::Apple => write!(f, "APPLE"),
            Vendor::Unknown => write!(f, "UNKNOWN"),
        }
    }
}
impl Display for IntelGpuType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            IntelGpuType::Integrated => write!(f, "Integrated"),
            IntelGpuType::Discrete => write!(f, "Discrete"),
            IntelGpuType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Error type for parsing a `Vendor` from a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseVendorError {
    /// The invalid input string
    pub input: String,
}

impl std::fmt::Display for ParseVendorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown GPU vendor: '{}'", self.input)
    }
}

impl std::error::Error for ParseVendorError {}

/// Parses a `Vendor` from a string.
///
/// The parsing is case-insensitive and supports common vendor names
/// and aliases.
///
/// # Supported Values
///
/// - NVIDIA: "nvidia", "geforce", "quadro", "tesla"
/// - AMD: "amd", "radeon", "ati"
/// - Intel: "intel", "arc", "iris", "uhd"
/// - Apple: "apple", "m1", "m2", "m3"
///
/// # Examples
///
/// ```
/// use gpu_info::vendor::Vendor;
/// use std::str::FromStr;
///
/// assert_eq!(Vendor::from_str("nvidia").unwrap(), Vendor::Nvidia);
/// assert_eq!(Vendor::from_str("AMD").unwrap(), Vendor::Amd);
/// assert_eq!(Vendor::from_str("GeForce").unwrap(), Vendor::Nvidia);
/// assert_eq!(Vendor::from_str("Radeon").unwrap(), Vendor::Amd);
///
/// // Unknown vendor returns error
/// assert!(Vendor::from_str("unknown_vendor").is_err());
/// ```
impl FromStr for Vendor {
    type Err = ParseVendorError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        let trimmed = lower.trim();

        // NVIDIA
        if trimmed == "nvidia"
            || trimmed == "geforce"
            || trimmed == "quadro"
            || trimmed == "tesla"
            || trimmed.starts_with("nvidia ")
        {
            return Ok(Vendor::Nvidia);
        }

        // AMD
        if trimmed == "amd"
            || trimmed == "radeon"
            || trimmed == "ati"
            || trimmed.starts_with("amd ")
            || trimmed.starts_with("radeon ")
        {
            return Ok(Vendor::Amd);
        }

        // Intel
        if trimmed == "intel" || trimmed.starts_with("intel ") {
            let gpu_type = if trimmed.contains("arc") {
                IntelGpuType::Discrete
            } else if trimmed.contains("iris")
                || trimmed.contains("uhd")
                || trimmed.contains("hd graphics")
            {
                IntelGpuType::Integrated
            } else {
                IntelGpuType::Unknown
            };
            return Ok(Vendor::Intel(gpu_type));
        }

        // Intel Arc
        if trimmed == "arc" || trimmed.starts_with("arc ") {
            return Ok(Vendor::Intel(IntelGpuType::Discrete));
        }

        // Intel integrated
        if trimmed == "iris" || trimmed == "uhd" || trimmed.starts_with("iris ") {
            return Ok(Vendor::Intel(IntelGpuType::Integrated));
        }

        // Apple
        if trimmed == "apple"
            || trimmed == "m1"
            || trimmed == "m2"
            || trimmed == "m3"
            || trimmed == "m4"
            || trimmed.starts_with("apple ")
        {
            return Ok(Vendor::Apple);
        }

        // Unknown - return error instead of Unknown variant
        Err(ParseVendorError {
            input: s.to_string(),
        })
    }
}
/// Determine GPU vendor from GPU name
///
/// This function eliminates duplication in vendor determination logic
/// by providing a unified way to classify GPU vendors based on their names.
///
/// # Arguments
/// * `name` - GPU name string to classify
///
/// # Returns
/// * `Vendor` - Classified vendor enum
///
/// # Examples
/// ```
/// use gpu_info::vendor::{determine_vendor_from_name, IntelGpuType, Vendor};
/// assert_eq!(
///     determine_vendor_from_name("NVIDIA GeForce RTX 3080"),
///     Vendor::Nvidia
/// );
/// assert_eq!(
///     determine_vendor_from_name("AMD Radeon RX 6800 XT"),
///     Vendor::Amd
/// );
/// assert_eq!(
///     determine_vendor_from_name("Intel Iris Xe Graphics"),
///     Vendor::Intel(IntelGpuType::Integrated)
/// );
/// ```
pub fn determine_vendor_from_name(name: &str) -> Vendor {
    let name_lower = name.to_lowercase();
    // Apple Silicon
    if name_lower.contains("apple")
        || name_lower.contains("m1")
        || name_lower.contains("m2")
        || name_lower.contains("m3")
    {
        return Vendor::Apple;
    }
    // AMD
    if name_lower.contains("amd") || name_lower.contains("radeon") {
        return Vendor::Amd;
    }
    // NVIDIA
    if name_lower.contains("nvidia") || name_lower.contains("geforce") {
        return Vendor::Nvidia;
    }
    // Intel
    if name_lower.contains("intel") {
        let gpu_type = if name_lower.contains("iris")
            || name_lower.contains("uhd")
            || name_lower.contains("hd graphics")
        {
            IntelGpuType::Integrated
        } else if name_lower.contains("arc") || name_lower.contains("discrete") {
            IntelGpuType::Discrete
        } else {
            IntelGpuType::Unknown
        };
        return Vendor::Intel(gpu_type);
    }
    Vendor::Unknown
}
/// Determine Intel GPU type from GPU name
///
/// This function provides a unified way to classify Intel GPU types
/// based on their names, eliminating duplication.
///
/// # Arguments
/// * `name` - Intel GPU name string to classify
///
/// # Returns
/// * `IntelGpuType` - Classified Intel GPU type
///
/// # Examples
/// ```
/// use gpu_info::vendor::{determine_intel_gpu_type_from_name, IntelGpuType};
/// assert_eq!(
///     determine_intel_gpu_type_from_name("Intel Iris Xe Graphics"),
///     IntelGpuType::Integrated
/// );
/// assert_eq!(
///     determine_intel_gpu_type_from_name("Intel Arc A770"),
///     IntelGpuType::Discrete
/// );
/// ```
pub fn determine_intel_gpu_type_from_name(name: &str) -> IntelGpuType {
    let name_lower = name.to_lowercase();
    if name_lower.contains("iris")
        || name_lower.contains("uhd")
        || name_lower.contains("hd graphics")
    {
        IntelGpuType::Integrated
    } else if name_lower.contains("arc") || name_lower.contains("discrete") {
        IntelGpuType::Discrete
    } else {
        IntelGpuType::Unknown
    }
}
