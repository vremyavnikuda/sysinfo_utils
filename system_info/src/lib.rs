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

pub use crate::{BitDepth::*, info::Info, os_type::Type, version::Version};

#[cfg(not(any(
    target_os = "aix",
    target_os = "android",
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "windows"
)))]
#[path = "unknown/mod.rs"]
mod imp;

mod system_uname;
mod system_version;
mod android;
mod dragonfly;
mod emscripten;
mod freebsd;
mod illumos;
mod macos;
use system_info::Info;

pub use crate::{
    bit_depth::BitDepth,
    system_matcher::SystemMatcher,
    system_version::SystemVersion,
};
/// Returns information about the current operating system (type, version, edition, etc.).
///
/// # Examples
///
/// ```
/// use system_info;
///
/// let info = system_info::get();
///
/// // Print full information:
/// println!("OS information: {info}");
///
/// // Print information separately:
/// println!("Type: {}", info.os_type());
/// println!("Version: {}", info.version());
/// println!("Edition: {:?}", info.edition());
/// println!("Codename: {:?}", info.codename());
/// println!("BitDepth: {}", info.bit_depth());
/// println!("Architecture: {:?}", info.architecture());
/// ```
pub fn get() -> Info{
    imp::current_platform()
}