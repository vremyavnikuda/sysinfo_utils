//! Linux GPU providers module.
//!
//! This module exports all GPU provider implementations for Linux.
//! These providers use Linux-specific APIs like sysfs and hwmon for
//! GPU detection and metrics collection.
//!
//! # Providers
//!
//! - `AmdLinuxProvider` - AMD GPU provider using sysfs/hwmon
//! - `IntelLinuxProvider` - Intel GPU provider using sysfs
//! - `NvidiaLinuxProvider` - NVIDIA GPU provider using NVML

/// AMD GPU provider for Linux using sysfs and hwmon.
pub mod amd;

/// Intel GPU provider for Linux using sysfs.
pub mod intel;

/// NVIDIA GPU provider for Linux using NVML.
pub mod nvidia;

#[cfg(target_os = "linux")]
pub use self::amd::AmdLinuxProvider;
#[cfg(target_os = "linux")]
pub use self::intel::IntelLinuxProvider;
#[cfg(target_os = "linux")]
pub use self::nvidia::NvidiaLinuxProvider;
