//! GPU providers module
//!
//! This module exports all GPU provider implementations.

// Public provider modules (cross-platform)
pub mod amd;
pub mod intel;
pub mod linux;
pub mod macos;
pub mod nvidia;

// Internal Windows-specific utility modules (not part of public API)
#[cfg(target_os = "windows")]
pub(crate) mod windows;

// Platform-specific provider re-exports
#[cfg(target_os = "windows")]
pub use self::amd::AmdProvider;
#[cfg(target_os = "windows")]
pub use self::intel::IntelProvider;
#[cfg(target_os = "windows")]
pub use self::nvidia::NvidiaProvider;

#[cfg(target_os = "linux")]
pub use self::linux::AmdLinuxProvider;
#[cfg(target_os = "linux")]
pub use self::linux::NvidiaLinuxProvider;

#[cfg(target_os = "macos")]
pub use self::macos::MacosProvider;
