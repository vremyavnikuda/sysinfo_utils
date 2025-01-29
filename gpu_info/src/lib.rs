#![feature(rustdoc_missing_doc_code_examples)]
#![deny(missing_debug_implementations, missing_docs, unsafe_code)]

//! `gpu_info` crate provides functionality to detect and manage GPU information.
//!
//! ## Modules
//!
//! - [manager]: Contains structures and functions to detect and manage GPU information.
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
//!     println!("Temperature: {}", gpu.get_temperature()); // Теперь выведет иконку
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
pub(crate) mod test;