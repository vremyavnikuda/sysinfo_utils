//! GPU providers module
//!
//! This module exports all GPU provider implementations.
pub mod amd;
pub mod intel;
pub mod linux;
pub mod macos;
pub mod nvidia;
#[cfg(target_os = "windows")]
pub use self::amd::AmdProvider;
#[cfg(target_os = "windows")]
pub use self::intel::IntelProvider;
#[cfg(target_os = "linux")]
pub use self::linux::AmdLinuxProvider;
#[cfg(target_os = "linux")]
pub use self::linux::NvidiaLinuxProvider;
#[cfg(target_os = "macos")]
pub use self::macos::MacosProvider;
#[cfg(target_os = "windows")]
pub use self::nvidia::NvidiaProvider;
