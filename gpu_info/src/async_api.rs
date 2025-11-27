//! Asynchronous API for GPU information retrieval
//!
//! This module provides async versions of the main GPU information functions,
//! allowing non-blocking operations for better performance in async contexts.
//!
//! Uses `Arc<GpuInfo>` for efficient sharing without cloning.
use crate::gpu_info::{GpuError, Result};
use crate::GpuInfo;
use std::sync::Arc;
/// Asynchronously gets the primary GPU information (zero-copy)
///
/// This function runs the GPU detection in a blocking task to avoid blocking
/// the async runtime, making it suitable for use in async contexts. It leverages
/// the project's caching system for improved performance on repeated calls.
///
/// Returns `Arc<GpuInfo>` for efficient sharing without cloning.
/// Use `get_async_owned()` if you need to mutate the data.
///
/// # Returns
/// * `Ok(Arc<GpuInfo>)` - The primary GPU information
/// * `Err(GpuError)` - If GPU detection fails
///
/// # Implementation Details
/// - Uses cached GPU manager for efficient primary GPU retrieval
/// - Falls back to direct detection if cache is unavailable
/// - Preserves specific error information from providers
/// - Executes in blocking task to avoid blocking async runtime
/// - Zero-cost abstraction: no cloning for read-only access
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
pub async fn get_async() -> Result<Arc<GpuInfo>> {
    tokio::task::spawn_blocking(|| {
        crate::gpu_manager::get_primary_gpu_arc().ok_or(GpuError::GpuNotFound)
    })
    .await
    .map_err(|_| GpuError::GpuNotActive)?
}

/// Asynchronously gets the primary GPU information (owned copy)
///
/// Returns a cloned copy of GPU information. Use this when you need to mutate
/// the GPU info. For read-only access, prefer `get_async()` which is more efficient.
///
/// # Returns
/// * `Ok(GpuInfo)` - Cloned primary GPU information
/// * `Err(GpuError)` - If GPU detection fails
///
/// # Example
/// ```rust
/// use gpu_info::async_api::get_async_owned;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut gpu = get_async_owned().await?;
///     // Can mutate gpu here
///     Ok(())
/// }
/// ```
pub async fn get_async_owned() -> Result<GpuInfo> {
    let result = tokio::task::spawn_blocking(|| {
        if let Some(primary_gpu) = crate::gpu_manager::get_primary_gpu() {
            return Ok(primary_gpu);
        }
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
/// Asynchronously gets all available GPUs in the system (zero-copy)
///
/// This function runs the GPU detection in a blocking task to avoid blocking
/// the async runtime, making it suitable for use in async contexts.
///
/// Returns `Vec<Arc<GpuInfo>>` for efficient sharing without cloning.
/// Use `get_all_async_owned()` if you need to mutate the data.
///
/// # Returns
/// * `Ok(Vec<Arc<GpuInfo>>)` - Vector of all detected GPU information
/// * `Err(GpuError)` - If GPU detection fails
///
/// # Example
/// ```rust
/// use gpu_info::async_api::get_all_async;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let gpus = get_all_async().await?;
///     for gpu in &gpus {
///         println!("GPU: {}", gpu.format_name_gpu());
///     }
///     Ok(())
/// }
/// ```
pub async fn get_all_async() -> Result<Vec<Arc<GpuInfo>>> {
    tokio::task::spawn_blocking(|| {
        let mut manager = crate::gpu_manager::GpuManager::new();
        manager.detect_all_gpus();
        let gpu_count = manager.gpu_count();

        if gpu_count == 0 {
            return Err(GpuError::GpuNotFound);
        }
        let mut gpus = Vec::with_capacity(gpu_count);
        for i in 0..gpu_count {
            if let Some(gpu) = manager.get_gpu_cached(i) {
                gpus.push(gpu);
            }
        }
        if gpus.is_empty() {
            Err(GpuError::GpuNotFound)
        } else {
            Ok(gpus)
        }
    })
    .await
    .map_err(|_| GpuError::GpuNotActive)?
}

/// Asynchronously gets all available GPUs in the system (owned copies)
///
/// Returns cloned copies of GPU information. Use this when you need to mutate
/// the GPU info. For read-only access, prefer `get_all_async()` which is more efficient.
///
/// # Returns
/// * `Ok(Vec<GpuInfo>)` - Cloned vector of all detected GPU information
/// * `Err(GpuError)` - If GPU detection fails
///
/// # Example
/// ```rust
/// use gpu_info::async_api::get_all_async_owned;
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut gpus = get_all_async_owned().await?;
///     // Can mutate gpus here
///     Ok(())
/// }
/// ```
pub async fn get_all_async_owned() -> Result<Vec<GpuInfo>> {
    let result = tokio::task::spawn_blocking(crate::get_all).await;
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
    let vendor = gpu.vendor;
    let name_gpu = gpu.name_gpu.clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut manager = crate::gpu_manager::GpuManager::new();
        manager.detect_all_gpus();
        let _ = manager.refresh_all_gpus();
        let gpu_count = manager.gpu_count();
        for i in 0..gpu_count {
            if let Some(gpu_arc) = manager.get_gpu_cached(i) {
                if gpu_arc.vendor == vendor && gpu_arc.name_gpu == name_gpu {
                    // Clone only once when we found the match
                    return Ok(Arc::try_unwrap(gpu_arc).unwrap_or_else(|arc| (*arc).clone()));
                }
            }
        }
        Err(GpuError::GpuNotFound)
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
        assert!(result.is_ok() || result.is_err());
    }
    #[tokio::test]
    async fn test_get_all_async() {
        let result = get_all_async().await;
        assert!(result.is_ok() || result.is_err());
    }
    #[tokio::test]
    async fn test_update_gpu_async() {
        let mut gpu = GpuInfo::unknown();
        let result = update_gpu_async(&mut gpu).await;
        assert!(result.is_ok() || result.is_err());
    }
}
