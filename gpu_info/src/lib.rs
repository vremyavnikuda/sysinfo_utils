//gpu_info/src/lib.rs
use crate::gpu_info::GpuInfo;

// #![deny(missing_debug_implementations, missing_docs, unsafe_code)]
pub mod gpu_info;
pub mod vendor;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod imp;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
#[path = "unknown/mod.rs"]
mod imp;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]

pub fn get() -> GpuInfo{
    imp::info_gpu()
}
