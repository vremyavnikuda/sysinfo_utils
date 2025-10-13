//! Prelude module for convenient imports.
//!
//! The prelude module provides a convenient way to import the most commonly used
//! types and functions from the `system_info_lib` crate. Instead of importing
//! each type individually, you can import everything at once with a single line.
//!
//! # Examples
//!
//! ```
//! use system_info_lib::prelude::*;
//!
//! // Now you have access to all commonly used types
//! let info = get();
//! println!("OS: {}", info.system_type());
//! println!("Version: {}", info.version());
//!
//! // Use the builder pattern
//! let custom_info = Info::builder()
//!     .system_type(Type::Linux)
//!     .version(SystemVersion::Semantic(5, 15, 0))
//!     .bit_depth(BitDepth::X64)
//!     .build();
//! ```
//!
//! # What's included
//!
//! - [`BitDepth`] - System bit depth representation
//! - [`Info`] - Main system information structure
//! - [`InfoBuilder`] - Builder for creating `Info` instances
//! - [`Type`] - Operating system type enumeration
//! - [`SystemVersion`] - System version representation
//! - [`get`] - Function to retrieve current system information

pub use crate::{
    get, BitDepth, Info, InfoBuilder, SystemVersion, Type,
};
