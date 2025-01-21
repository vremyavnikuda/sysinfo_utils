//! # System Info Crate
//!
//! This crate provides functionality to retrieve and manage system information
//! such as architecture, bit depth, operating system, and version details.
//! It supports multiple platforms and includes specific implementations for
//! different operating systems.
//!
//! ## Modules
//!
//! - `architecture`: Handles system architecture information.
//! - `bit_depth`: Manages bit depth details.
//! - `imp`: Platform-specific implementations (e.g., AIX).
//! - `system_info`: Core system information functionalities.
//! - `system_matcher`: Utilities for matching system properties.
//! - `system_os`: Operating system-related information.
//! - `system_uname`: Uname system call wrapper.
//! - `system_version`: System version details.

#![deny(
    missing_debug_implementations,
    missing_docs,
    unsafe_code,
    missing_doc_code_examples
)]
mod architecture;
mod bit_depth;
#[cfg(target_os = "aix")]
#[path = "aix/mod.rs"]
mod imp;
mod system_info;
mod system_matcher;
mod system_os;
#[cfg(any(
    target_os = "aix",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod system_uname;
mod system_version;
mod android;
mod dragonfly;
mod emscripten;

pub use crate::{
    architecture::Architecture,
    bit_depth::BitDepth,
    system_info::{Info, get},
    system_matcher::SystemMatcher,
    system_os::SystemOS,
    system_version::SystemVersion,
};