#![deny(missing_debug_implementations, missing_docs, unsafe_code)]
#![feature(rustdoc_missing_doc_code_examples)]

//! `gpu_info` crate provides functionality to detect and manage GPU information.
//!
//! ## Modules
//!
//! - [gpu_info](gpu_info): Contains structures and functions to detect and manage GPU information.
//!
//! ## Examples
//!
//! ```rust
//! use gpu_info::gpu_info::GpuInfo;
//! use gpu_info::gpu_info::GpuManager;
//!
//! fn main() {
//!     let mut manager = GpuManager::new();
//!     manager.detect_gpus();
//!     for gpu in manager.gpus {
//!         println!("Detected GPU: {}", gpu.name);
//!     }
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
pub mod gpu_info;
