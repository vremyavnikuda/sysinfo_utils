//! Vendor-specific GPU implementations and utilities

use serde::{Deserialize, Serialize};

/// Enumeration of GPU hardware vendors
///
/// Represents different graphics processor manufacturers supported by the library.
/// This enum is non-exhaustive and may be extended in future versions.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuVendor {
    /// NVIDIA Corporation graphics processors
    Nvidia,

    /// Advanced Micro Devices (AMD) graphics processors
    AMD,

    /// Intel Corporation integrated and discrete graphics
    Intel,

    /// Unknown or unspecified GPU vendor
    Unknown,
}

impl Default for GpuVendor {
    /// Provides default value as [`GpuVendor::Unknown`]
    fn default() -> Self {
        GpuVendor::Unknown
    }
}
