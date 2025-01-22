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
extern crate core;

#[cfg(target_os = "aix")]
#[path = "aix/mod.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android/mod.rs"]
mod imp;

#[cfg(target_os = "dragonfly")]
#[path = "dragonfly/mod.rs"]
mod imp;

#[cfg(target_os = "emscripten")]
#[path = "emscripten/mod.rs"]
mod imp;

#[cfg(target_os = "freebsd")]
#[path = "freebsd/mod.rs"]
mod imp;

#[cfg(target_os = "illumos")]
#[path = "illumos/mod.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod imp;

#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod imp;

#[cfg(target_os = "netbsd")]
#[path = "netbsd/mod.rs"]
mod imp;

#[cfg(target_os = "openbsd")]
#[path = "openbsd/mod.rs"]
mod imp;

#[cfg(target_os = "redox")]
#[path = "redox/mod.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

mod architecture;
mod bit_depth;
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
mod freebsd;

use system_info::Info;

pub use crate::{
    bit_depth::BitDepth,
    system_matcher::SystemMatcher,
    system_version::SystemVersion,
};
pub fn get() -> Info{
    imp::current_platform()
}