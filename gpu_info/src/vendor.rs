use std::fmt::Display;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
///
/// GPU vendor information
/// Vendor enum representing different GPU vendors
///
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
    Integrated, // Встроенная графика
    Discrete,   // Дискретная графика
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
///
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
