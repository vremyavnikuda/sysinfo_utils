//! Windows-specific GPU provider utilities
//!
//! # Architecture
//!
//! This module provides Windows-specific GPU monitoring through a layered approach:
//!
//! ## Public API
//! - **`intel`** - `IntelWindowsProvider` - Unified interface for Intel GPU metrics
//!   - Single entry point for all Intel GPU operations
//!   - Implements three-tier fallback strategy (WMI → Intel MD API → PDH)
//!   - Use this for all Intel GPU monitoring
//!
//! ## Internal Utilities (Not Public API)
//! - **`pdh`** - Low-level PDH (Performance Data Helper) FFI bindings
//!   - Used internally by `IntelWindowsProvider` as fallback
//!   - Do NOT use directly - use `IntelWindowsProvider` methods instead
//!   - Provides utilization and memory usage via Windows Performance Counters
//!
//! # Usage
//!
//! ```rust,no_run
//! use gpu_info::providers::windows::intel::IntelWindowsProvider;
//! use gpu_info::gpu_info::GpuProvider;
//!
//! let provider = IntelWindowsProvider::new();
//! let gpus = provider.detect_gpus()?;
//! // All metrics (WMI + Intel MD API + PDH) collected automatically
//! # Ok::<(), gpu_info::gpu_info::GpuError>(())
//! ```
//!
//! # Design Principles
//!
//! - **Centralization**: All Intel GPU logic in `intel.rs`, PDH is just a utility
//! - **Encapsulation**: PDH module is `pub(crate)`, not exposed to external code
//! - **Fallback Strategy**: Graceful degradation when advanced APIs unavailable

// Public module - use this for Intel GPU monitoring
#[cfg(target_os = "windows")]
pub mod intel;

// Internal utility module - do NOT use directly
pub(crate) mod pdh;
