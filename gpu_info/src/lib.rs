// src/lib.rs
//missing_docs
#![deny(missing_debug_implementations)]

//! `gpu_info` crate provides functionality to detect and manage GPU information.
//!
//! ## Modules
//!
//! - [manager]: Contains structures and functions to detect and manage GPU information.
//! - [amd]: Contains functionality specific to AMD GPUs.
//! - [intel]: Contains functionality specific to Intel GPUs.
//! - [nvidia]: Contains functionality specific to NVIDIA GPUs.
//! - [qualcomm]: Contains functionality specific to Qualcomm GPUs.
//! - [unknown]: Contains functionality for unsupported GPU vendors.
//!
//! ## Examples
//!
//! ```rust
//! use gpu_info::{GpuManager, GpuInfo, GpuVendor};
//!
//! let mut manager = GpuManager::new();
//! manager.refresh();
//!
//! for (idx, gpu) in manager.gpus.iter().enumerate() {
//!     println!("GPU {}: {}", idx, gpu.get_name());
//!     println!("Temperature: {}", gpu.get_temperature());
//! }
//! ```

/// Module documentation for `gpu_info` module.
///
/// The `gpu_info` module provides structures and functions to detect and manage GPU information.
///
/// ## Structures
///
/// - `GpuManager`: Manages the detection and information of GPUs.
/// - `Gpu`: Represents a GPU with its properties.
///
/// ## Enums
///
/// - `GpuVendor`: Enum representing different GPU vendors (Nvidia, AMD, Intel).
pub mod mode;

use mode::{gpu, manager, vendor};
// Re-export main types to the crate root
pub use gpu::GpuInfo;
pub use manager::GpuManager;
pub use vendor::GpuVendor;

/// Contains functionality specific to AMD GPUs.
pub mod amd;

/// Contains functionality specific to Intel GPUs.
pub mod intel;

/// Contains functionality specific to NVIDIA GPUs.
pub mod nvidia;

/// Contains functionality specific to Qualcomm GPUs.
pub mod qualcomm;

pub(crate) mod test;
//pub mod UNKNOWN;
