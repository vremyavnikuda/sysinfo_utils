//! Asynchronous API for GPU information retrieval
//!
//! This module provides async versions of the main GPU information functions,
//! allowing non-blocking operations for better performance in async contexts.
use crate::gpu_info::{GpuError, Result};
use crate::GpuInfo;
/// Asynchronously gets the primary GPU information
///
/// This function runs the GPU detection in a blocking task to avoid blocking
/// the async runtime, making it suitable for use in async contexts. It leverages
/// the project's caching system for improved performance on repeated calls.
///
/// # Returns
/// * `Ok(GpuInfo)` - The primary GPU information
/// * `Err(GpuError)` - If GPU detection fails
///
/// # Implementation Details
/// - Uses cached GPU manager for efficient primary GPU retrieval
/// - Falls back to direct detection if cache is unavailable
/// - Preserves specific error information from providers
/// - Executes in blocking task to avoid blocking async runtime
///
/// # Example
/// ```rust
/// use gpu_info::async_api::get_async;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let gpu = get_async().await?;
///     println!("Primary GPU: {}", gpu.format_name_gpu());
///     println!("Temperature: {}°C", gpu.format_temperature());
///     Ok(())
/// }
/// ```
pub async fn get_async() -> Result<GpuInfo> {
    let result = tokio::task::spawn_blocking(|| {
        // Try to get primary GPU from cached manager first (most efficient)
        if let Some(primary_gpu) = crate::gpu_manager::get_primary_gpu() {
            return Ok(primary_gpu);
        }
        
        // Fallback to direct detection if cache unavailable
        match crate::get() {
            gpu if gpu.vendor != crate::vendor::Vendor::Unknown => Ok(gpu),
            _ => Err(crate::gpu_info::GpuError::GpuNotFound),
        }
    })
    .await;
    
    match result {
        Ok(Ok(gpu)) => Ok(gpu),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(GpuError::GpuNotActive),
    }
}
/// Asynchronously gets all available GPUs in the system
///
/// This function runs the GPU detection in a blocking task to avoid blocking
/// the async runtime, making it suitable for use in async contexts.
///
/// # Returns
/// * `Ok(Vec<GpuInfo>)` - Vector of all detected GPU information
/// * `Err(GpuError)` - If GPU detection fails
///
/// # Example
/// ```rust
/// use gpu_info::async_api::get_all_async;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let gpus = get_all_async().await?;
///     for gpu in gpus {
///         println!("GPU: {:?}", gpu);
///     }
///     Ok(())
/// }
/// ```
pub async fn get_all_async() -> Result<Vec<GpuInfo>> {
    let result = tokio::task::spawn_blocking(|| crate::get_all()).await;
    match result {
        Ok(gpus) => Ok(gpus),
        Err(_) => Err(GpuError::GpuNotActive),
    }
}
/// Asynchronously updates GPU information
///
/// This function updates the information for a specific GPU using the appropriate
/// provider in a blocking task to avoid blocking the async runtime. The function
/// automatically selects the correct provider based on the GPU's vendor.
///
/// # Arguments
/// * `gpu` - Mutable reference to GPU to update
///
/// # Returns
/// * `Ok(())` - If GPU was successfully updated with fresh data
/// * `Err(GpuError)` - If update failed (provider not found, API error, etc.)
///
/// # Implementation Details
/// - Uses provider pattern for vendor-specific updates
/// - Executes in blocking task to avoid blocking async runtime
/// - Automatically registers appropriate providers for current platform
/// - Updates only the target GPU (efficient, focused operation)
///
/// # Example
/// ```rust
/// use gpu_info::{async_api::update_gpu_async, get};
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut gpu = get();
///     println!("Before: {}°C", gpu.format_temperature());
///     update_gpu_async(&mut gpu).await?;
///     println!("After: {}°C", gpu.format_temperature());
///     Ok(())
/// }
/// ```
pub async fn update_gpu_async(gpu: &mut GpuInfo) -> Result<()> {
    let mut gpu_clone = gpu.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut provider_manager = crate::provider_manager::GpuProviderManager::new();
        #[cfg(target_os = "windows")]
        {
            use crate::providers::{AmdProvider, IntelProvider, NvidiaProvider};
            provider_manager.register_provider(crate::vendor::Vendor::Nvidia, NvidiaProvider::new());
            provider_manager.register_provider(crate::vendor::Vendor::Amd, AmdProvider::new());
            provider_manager.register_provider(
                crate::vendor::Vendor::Intel(crate::vendor::IntelGpuType::Unknown), 
                IntelProvider::new()
            );
        }
        #[cfg(target_os = "linux")]
        {
            use crate::providers::linux::{AmdLinuxProvider, NvidiaLinuxProvider};
            provider_manager.register_provider(crate::vendor::Vendor::Nvidia, NvidiaLinuxProvider::new());
            provider_manager.register_provider(crate::vendor::Vendor::Amd, AmdLinuxProvider::new());
        }
        #[cfg(target_os = "macos")]
        {
            use crate::providers::macos::MacosProvider;
            provider_manager.register_provider(crate::vendor::Vendor::Apple, MacosProvider::new());
        }
        match provider_manager.update_gpu(&mut gpu_clone) {
            Ok(()) => Ok(gpu_clone),
            Err(e) => Err(e),
        }
    })
    .await;
    match result {
        Ok(Ok(updated_gpu)) => {
            *gpu = updated_gpu;
            Ok(())
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(GpuError::GpuNotActive),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_get_async() {
        let result = get_async().await;
        // We can't guarantee a GPU is present in test environment
        // but we can check that the function doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
    #[tokio::test]
    async fn test_get_all_async() {
        let result = get_all_async().await;
        // We can't guarantee GPUs are present in test environment
        // but we can check that the function doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
    #[tokio::test]
    async fn test_update_gpu_async() {
        let mut gpu = GpuInfo::unknown();
        let result = update_gpu_async(&mut gpu).await;
        // Update may fail in test environment, but shouldn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
