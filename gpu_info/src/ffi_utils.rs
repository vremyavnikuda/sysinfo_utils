//! Common FFI utilities for dynamic library loading and symbol resolution
//! 
//! This module provides abstractions for loading dynamic libraries and resolving symbols
//! across different platforms, reducing code duplication in GPU vendor implementations.

use log::error;
use std::marker::PhantomData;

#[cfg(windows)]
use windows::{
    core::PCSTR,
    core::PCWSTR,
    Win32::Foundation::HMODULE,
    Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA, LoadLibraryW},
};

#[cfg(unix)]
use libloading::{Library, Symbol};

/// Represents the result of an API call with a success/error code
pub trait ApiResult<T> {
    /// Check if the result represents success
    fn is_success(&self) -> bool;
    /// Convert to Option<T> based on success status
    fn to_option(self) -> Option<T>;
    /// Get error code for logging
    fn error_code(&self) -> i32;
}

/// NVML API result wrapper
pub struct NvmlResult<T> {
    pub code: i32,
    pub value: T,
}

impl<T> ApiResult<T> for NvmlResult<T> {
    fn is_success(&self) -> bool {
         // NVML_SUCCESS
        self.code == 0
    }
    
    fn to_option(self) -> Option<T> {
        if self.is_success() {
            Some(self.value)
        } else {
            None
        }
    }
    
    fn error_code(&self) -> i32 {
        self.code
    }
}

/// ADL API result wrapper
pub struct AdlResult<T> {
    pub code: i32,
    pub value: T,
}

impl<T> ApiResult<T> for AdlResult<T> {
    fn is_success(&self) -> bool {
        // ADL_OK
        self.code == 0 
    }
    
    fn to_option(self) -> Option<T> {
        if self.is_success() {
            Some(self.value)
        } else {
            None
        }
    }
    
    fn error_code(&self) -> i32 {
        self.code
    }
}

/// Macro to handle API result conversion with error logging
#[macro_export]
macro_rules! handle_api_result {
    ($result:expr, $error_msg:expr) => {
        match $result.to_option() {
            Some(value) => value,
            None => {
                log::error!("{}: error code {}", $error_msg, $result.error_code());
                return None;
            }
        }
    };
}

/// Macro to handle API result conversion with error logging for functions returning Vec
#[macro_export]
macro_rules! handle_api_result_vec {
    ($result:expr, $error_msg:expr) => {
        match $result.to_option() {
            Some(value) => value,
            None => {
                log::error!("{}: error code {}", $error_msg, $result.error_code());
                return Vec::new();
            }
        }
    };
}

/// Cross-platform dynamic library wrapper
pub enum DynamicLibrary {
    #[cfg(windows)]
    Windows(HMODULE),
    #[cfg(unix)]
    Unix(Library),
}

impl DynamicLibrary {
    /// Load a library by name on Windows
    #[cfg(windows)]
    pub fn load_windows_a(name: &str) -> Result<Self, String> {
        let name_cstr = format!("{}\0", name);
        unsafe {
            match LoadLibraryA(PCSTR::from_raw(name_cstr.as_ptr())) {
                Ok(handle) => Ok(DynamicLibrary::Windows(handle)),
                Err(e) => Err(format!("Failed to load library {}: {}", name, e)),
            }
        }
    }

    /// Load a library by wide string path on Windows
    #[cfg(windows)]
    pub fn load_windows_w(path: &str) -> Result<Self, String> {
        let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            match LoadLibraryW(PCWSTR::from_raw(wide_path.as_ptr())) {
                Ok(handle) => Ok(DynamicLibrary::Windows(handle)),
                Err(e) => Err(format!("Failed to load library from {}: {}", path, e)),
            }
        }
    }

    /// Load a library by path on Unix systems
    #[cfg(unix)]
    pub fn load_unix(path: &str) -> Result<Self, String> {
        match Library::new(path) {
            Ok(lib) => Ok(DynamicLibrary::Unix(lib)),
            Err(e) => Err(format!("Failed to load library from {}: {}", path, e)),
        }
    }

    /// Get a symbol from the loaded library
    #[cfg(windows)]
    pub fn get_symbol<T>(&self, symbol_name: &str) -> Result<T, String> {
        match self {
            DynamicLibrary::Windows(handle) => {
                let symbol_cstr = format!("{}\0", symbol_name);
                unsafe {
                    match GetProcAddress(*handle, PCSTR::from_raw(symbol_cstr.as_ptr())) {
                        Some(proc_addr) => Ok(std::mem::transmute_copy(&proc_addr)),
                        None => Err(format!("Failed to get symbol: {}", symbol_name)),
                    }
                }
            }
        }
    }

    /// Get a symbol from the loaded library on Unix
    #[cfg(unix)]
    pub fn get_symbol<T>(&self, symbol_name: &[u8]) -> Result<Symbol<T>, String> {
        match self {
            DynamicLibrary::Unix(lib) => {
                match lib.get(symbol_name) {
                    Ok(symbol) => Ok(symbol),
                    Err(e) => Err(format!("Failed to get symbol {:?}: {}", 
                                        std::str::from_utf8(symbol_name).unwrap_or("unknown"), e)),
                }
            }
        }
    }
}

/// Builder for dynamic library loading with error handling
pub struct LibraryLoader {
    library_name: String,
    fallback_paths: Vec<String>,
}

impl LibraryLoader {
    /// Create a new library loader for the given library name
    pub fn new(library_name: &str) -> Self {
        Self {
            library_name: library_name.to_string(),
            fallback_paths: Vec::new(),
        }
    }

    /// Add a fallback path to try if the default loading fails
    pub fn with_fallback_path(mut self, path: &str) -> Self {
        self.fallback_paths.push(path.to_string());
        self
    }

    /// Attempt to load the library, trying fallback paths if necessary
    pub fn load(self) -> Result<DynamicLibrary, String> {
        // Try loading by name first
        #[cfg(windows)]
        {
            if let Ok(lib) = DynamicLibrary::load_windows_a(&self.library_name) {
                return Ok(lib);
            }
        }

        #[cfg(unix)]
        {
            if let Ok(lib) = DynamicLibrary::load_unix(&self.library_name) {
                return Ok(lib);
            }
        }

        // Try fallback paths
        for path in &self.fallback_paths {
            #[cfg(windows)]
            {
                if path.ends_with(".dll") {
                    if let Ok(lib) = DynamicLibrary::load_windows_w(path) {
                        return Ok(lib);
                    }
                } else if let Ok(lib) = DynamicLibrary::load_windows_a(path) {
                    return Ok(lib);
                }
            }

            #[cfg(unix)]
            {
                if let Ok(lib) = DynamicLibrary::load_unix(path) {
                    return Ok(lib);
                }
            }
        }

        Err(format!("Failed to load library {} and all fallback paths", self.library_name))
    }
}

/// Symbol resolver with type safety and error handling
pub struct SymbolResolver<'a> {
    library: &'a DynamicLibrary,
}

impl<'a> SymbolResolver<'a> {
    /// Create a new symbol resolver for the given library
    pub fn new(library: &'a DynamicLibrary) -> Self {
        Self { library }
    }

    /// Resolve a symbol with error handling and logging
    #[cfg(windows)]
    pub fn resolve<T>(&self, symbol_name: &str) -> Option<T> {
        match self.library.get_symbol(symbol_name) {
            Ok(symbol) => Some(symbol),
            Err(e) => {
                error!("Failed to resolve symbol {}: {}", symbol_name, e);
                None
            }
        }
    }

    /// Resolve a symbol with error handling and logging on Unix
    #[cfg(unix)]
    pub fn resolve<T>(&self, symbol_name: &[u8]) -> Option<Symbol<T>> {
        match self.library.get_symbol(symbol_name) {
            Ok(symbol) => Some(symbol),
            Err(e) => {
                error!("Failed to resolve symbol {:?}: {}", 
                      std::str::from_utf8(symbol_name).unwrap_or("unknown"), e);
                None
            }
        }
    }

    /// Resolve multiple symbols at once, returning None if any fail
    #[cfg(unix)]
    pub fn resolve_all<T>(&self, symbol_names: &[&[u8]]) -> Option<Vec<Symbol<T>>> {
        let mut symbols = Vec::new();
        for &name in symbol_names {
            match self.resolve(name) {
                Some(symbol) => symbols.push(symbol),
                None => return None,
            }
        }
        Some(symbols)
    }
}

/// GPU API function table - generic structure for organizing API functions
pub struct ApiTable<T> {
    functions: T,
    _phantom: PhantomData<T>,
}

impl<T> ApiTable<T> {
    pub fn new(functions: T) -> Self {
        Self {
            functions,
            _phantom: PhantomData,
        }
    }

    pub fn functions(&self) -> &T {
        &self.functions
    }
}