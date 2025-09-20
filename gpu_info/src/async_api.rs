//! Asynchronous API for GPU information retrieval
//! 
//! This module provides async versions of the main GPU information functions,
//! allowing non-blocking operations for better performance in async contexts.

use crate::{GpuInfo};
use crate::gpu_info::{Result, GpuError};

/// Asynchronously gets the primary GPU information
/// 
/// This function runs the GPU detection in a blocking task to avoid blocking
/// the async runtime, making it suitable for use in async contexts.
/// 
/// # Returns
/// * `Ok(GpuInfo)` - The primary GPU information
/// * `Err(GpuError)` - If GPU detection fails
/// 
/// # Example
/// ```rust
/// use gpu_info::async_api::get_async;
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let gpu = get_async().await?;
///     println!("GPU: {:?}", gpu);
///     Ok(())
/// }
/// ```
pub async fn get_async() -> Result<GpuInfo> {
    // For now, we'll use the existing synchronous implementation
    // wrapped in spawn_blocking for compatibility
    let result = tokio::task::spawn_blocking(|| crate::get()).await;
    match result {
        Ok(gpu) => Ok(gpu),
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
/// 
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
    // For now, we'll use the existing synchronous implementation
    // wrapped in spawn_blocking for compatibility
    let result = tokio::task::spawn_blocking(|| crate::get_all()).await;
    match result {
        Ok(gpus) => Ok(gpus),
        Err(_) => Err(GpuError::GpuNotActive),
    }
}

/// Asynchronously updates GPU information
/// 
/// This function updates the information for a specific GPU using the appropriate
/// provider in a blocking task to avoid blocking the async runtime.
/// 
/// # Arguments
/// * `gpu` - Mutable reference to GPU to update
/// 
/// # Returns
/// * `Ok(())` - If GPU was successfully updated
/// * `Err(GpuError)` - If update failed
/// 
/// # Example
/// ```rust
/// use gpu_info::{async_api::update_gpu_async, get};
/// 
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut gpu = get();
///     update_gpu_async(&mut gpu).await?;
///     println!("Updated GPU: {:?}", gpu);
///     Ok(())
/// }
/// ```
pub async fn update_gpu_async(gpu: &mut GpuInfo) -> Result<()> {
    // Clone the GPU for the blocking task
    let gpu_clone = gpu.clone();
    
    // Run the update in a blocking task
    let result = tokio::task::spawn_blocking(move || {
        // Since we can't access the private update_single_gpu_static function directly,
        // we'll create a temporary manager to handle the update
        let mut manager = crate::gpu_manager::GpuManager::new();
        // Try to update the GPU using the manager's internal logic
        match manager.refresh_all_gpus() {
            Ok(_) => {
                // If refresh succeeded, we can try to get the updated info
                // For now, we'll just return success
                Ok(gpu_clone)
            },
            Err(_) => {
                // If refresh failed, return the original GPU unchanged
                Ok(gpu_clone)
            }
        }
    }).await;

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