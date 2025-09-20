use std::fmt::Display;
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
///
/// GPU vendor information
/// Vendor enum representing different GPU vendors
pub enum Vendor {
    Nvidia,
    Amd,
    Intel(IntelGpuType),
    Apple,
    Unknown,
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum IntelGpuType {
    Integrated,
    Discrete,
    Unknown,
}
/// Converts a string to a Vendor enum
impl Default for Vendor {
    fn default() -> Self {
        Vendor::Unknown
    }
}
impl Default for IntelGpuType {
    fn default() -> Self {
        IntelGpuType::Unknown
    }
}
///
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
