//! Backend implementations for macOS GPU detection
//!
//! This module contains different backend implementations for accessing GPU information
//! on macOS, each with different performance characteristics and capabilities.

// System profiler backend (always available)
pub mod system_profiler;

// IOKit backend (requires macos-iokit feature)
#[cfg(feature = "macos-iokit")]
pub mod iokit;

// Metal backend (requires macos-metal feature)
#[cfg(feature = "macos-metal")]
pub mod metal;

// PowerMetrics backend
pub mod powermetrics;

// Re-export backend types
pub use system_profiler::SystemProfilerBackend;

#[cfg(feature = "macos-iokit")]
pub use iokit::IOKitBackend;

#[cfg(feature = "macos-metal")]
pub use metal::MetalBackend;

pub use powermetrics::PowerMetricsBackend;
