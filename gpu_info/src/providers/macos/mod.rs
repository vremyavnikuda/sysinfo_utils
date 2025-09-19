//! macOS GPU providers module
//! 
//! This module exports all GPU provider implementations for macOS.

pub mod macos;

#[cfg(target_os = "macos")]
pub use self::macos::MacosProvider;