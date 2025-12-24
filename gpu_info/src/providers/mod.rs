//! GPU providers module.
//!
//! This module exports all GPU provider implementations for different vendors
//! and platforms. Each provider implements the `GpuProvider` trait to provide
//! a unified interface for GPU detection and metrics collection.
//!
//! # Architecture
//!
//! Providers are organized by vendor (NVIDIA, AMD, Intel) with platform-specific
//! implementations in submodules:
//!
//! - `nvidia` - NVIDIA GPU provider using NVML
//! - `amd` - AMD GPU provider using ADL
//! - `intel` - Intel GPU provider using WMI/sysfs
//! - `linux` - Linux-specific provider implementations
//! - `macos` - macOS-specific provider implementations

/// AMD GPU provider implementation.
///
/// Provides GPU detection and metrics for AMD/Radeon GPUs using the
/// AMD Display Library (ADL) on Windows.
pub mod amd;

/// Intel GPU provider implementation.
///
/// Provides GPU detection and metrics for Intel integrated and discrete GPUs
/// using WMI on Windows and sysfs on Linux.
pub mod intel;

/// Linux-specific GPU provider implementations.
///
/// Contains provider implementations that use Linux-specific APIs like
/// sysfs and hwmon for GPU metrics collection.
pub mod linux;

/// macOS-specific GPU provider implementations.
///
/// Contains provider implementations that use macOS-specific APIs like
/// IOKit and Metal for GPU metrics collection.
pub mod macos;

/// NVIDIA GPU provider implementation.
///
/// Provides GPU detection and metrics for NVIDIA GPUs using the
/// NVIDIA Management Library (NVML).
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
