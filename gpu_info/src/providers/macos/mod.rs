//! macOS GPU provider with multi-backend support
//!
//! This module provides a comprehensive GPU information provider for macOS with
//! multiple backend options for optimal performance and reliability.
//!
//! # Backends
//!
//! - **SystemProfiler**: Always available, ~500-1000ms (slowest but most compatible)
//! - **IOKit**: Requires `macos-iokit` feature, ~1-10ms (fast PCI detection)
//! - **Metal**: Requires `macos-metal` feature, ~1-5ms (real-time metrics)
//! - **PowerMetrics**: CLI tool, ~100ms (metrics fallback)
//! - **Hybrid**: Auto-selects best available backend (recommended)
//!
//! # Feature Flags
//!
//! - `macos-iokit`: Enables IOKit backend for fast detection
//! - `macos-metal`: Enables Metal backend for real-time metrics
//! - `macos-full`: Enables all macOS features
//!
//! # Examples
//!
//! Basic usage:
//! ```no_run
//! use gpu_info::providers::macos::MacosProvider;
//! use gpu_info::gpu_info::GpuProvider;
//!
//! let provider = MacosProvider::new().expect("Failed to create provider");
//! let gpus = provider.detect_gpus().expect("Failed to detect GPUs");
//! println!("Found {} GPUs", gpus.len());
//! ```
//!
//! With builder:
//! ```no_run
//! use gpu_info::providers::macos::{MacosProviderBuilder, MacosBackend};
//! use std::time::Duration;
//!
//! let provider = MacosProviderBuilder::new()
//!     .cache_ttl(Duration::from_secs(120))
//!     .backend(MacosBackend::Hybrid)
//!     .fallback(true)
//!     .build()
//!     .expect("Failed to create provider");
//! ```

// Core modules
pub mod backends;
pub mod builder;
pub mod cache;
pub mod config;
pub mod metrics;
pub mod provider;
pub mod router;

// Legacy module (will be migrated in future tasks)
#[allow(clippy::module_inception)]
pub mod macos;

// Re-export public API
pub use self::builder::MacosProviderBuilder;
pub use self::config::{MacosBackend, MacosConfig};
pub use self::metrics::MacosMetrics;
pub use self::provider::MacosProvider;
