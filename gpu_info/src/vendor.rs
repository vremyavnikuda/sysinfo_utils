//gpu_info/src/vendor.rs
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
    Intel,
    Apple,
    Unknown,
}

/// Converts a string to a Vendor enum
impl Default for Vendor {
    fn default() -> Self {
        Vendor::Unknown
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
            Vendor::Intel => write!(f, "INTEL"),
            Vendor::Apple => write!(f, "APPLE"),
            Vendor::Unknown => write!(f, "UNKNOWN"),
        }
    }
}
