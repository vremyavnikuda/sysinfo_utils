//! Macros for reducing code duplication in GPU info implementations.
//!
//! This module contains macros used throughout the gpu_info crate for:
//! - Implementing `Formattable` trait for `Option<T>` types
//! - Generating format methods that return numeric types with defaults
//! - Generating format methods that return String with defaults
//!
//! # Usage
//!
//! These macros are used internally by `GpuInfo` to generate format methods:
//!
//! ```rust
//! use gpu_info::GpuInfo;
//!
//! let gpu = GpuInfo::builder()
//!     .temperature(65.5)
//!     .name("RTX 3080")
//!     .build();
//!
//! // format_* methods use these macros internally
//! assert_eq!(gpu.format_temperature(), 65.5);
//! assert_eq!(gpu.format_name_gpu(), "RTX 3080");
//! ```

/// Macro for implementing Formattable for Option<T> types.
///
/// This macro generates an implementation of the `Formattable` trait for `Option<$type>`,
/// where the `fmt_string` method returns the string representation of the value
/// or "N/A" if the option is `None`.
///
/// # Arguments
/// * `$type` - The inner type of the Option (e.g., `u32`, `bool`, `String`)
///
/// # Generated Code
///
/// For `impl_formattable_for_option!(u32)`, generates:
/// ```text
/// impl Formattable for Option<u32> {
///     fn fmt_string(&self) -> String {
///         match self {
///             Some(value) => value.to_string(),
///             None => String::from("N/A"),
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_formattable_for_option {
    ($type:ty) => {
        impl Formattable for Option<$type> {
            fn fmt_string(&self) -> String {
                match self {
                    Some(value) => value.to_string(),
                    None => String::from("N/A"),
                }
            }
        }
    };
}

/// Macro for generating format_* methods that return numeric types with default values.
///
/// This macro eliminates code duplication by generating methods that follow the pattern:
/// `self.field.unwrap_or(default)`
///
/// # Arguments
/// * `$method_name` - The name of the generated method (e.g., `format_temperature`)
/// * `$field` - The field to access on self (e.g., `temperature`)
/// * `$return_type` - The return type of the method (e.g., `f32`)
/// * `$default` - The default value to return if the field is None
///
/// # Generated Code
///
/// For `impl_format_numeric!(format_temperature, temperature, f32, 0.0)`, generates:
/// ```text
/// pub fn format_temperature(&self) -> f32 {
///     self.temperature.unwrap_or(0.0)
/// }
/// ```
#[macro_export]
macro_rules! impl_format_numeric {
    ($method_name:ident, $field:ident, $return_type:ty, $default:expr) => {
        pub fn $method_name(&self) -> $return_type {
            self.$field.unwrap_or($default)
        }
    };
}

/// Macro for generating format_* methods that return String with default values.
///
/// This macro handles Option<String> fields that need to return owned String values.
///
/// # Arguments
/// * `$method_name` - The name of the generated method
/// * `$field` - The field to access on self
/// * `$default` - The default string to return if the field is None
///
/// # Generated Code
///
/// For `impl_format_string!(format_name_gpu, name_gpu, "Unknown GPU")`, generates:
/// ```text
/// pub fn format_name_gpu(&self) -> String {
///     self.name_gpu
///         .as_ref()
///         .map_or_else(|| "Unknown GPU".to_string(), |s| s.clone())
/// }
/// ```
#[macro_export]
macro_rules! impl_format_string {
    ($method_name:ident, $field:ident, $default:expr) => {
        pub fn $method_name(&self) -> String {
            self.$field
                .as_ref()
                .map_or_else(|| $default.to_string(), |s| s.clone())
        }
    };
}
