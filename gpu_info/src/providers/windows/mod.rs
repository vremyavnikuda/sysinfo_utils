//! Windows-specific GPU provider utilities
//!
//! This module contains internal utilities for Windows GPU providers:
//! - PDH (Performance Data Helper) API for GPU metrics
//! - Intel Metrics API for advanced Intel GPU monitoring

// Internal modules (not part of public API)
pub(crate) mod intel_metrics;
pub(crate) mod pdh;
