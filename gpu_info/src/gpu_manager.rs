use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::query::GpuQuery;
use crate::vendor::Vendor;
use log::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Manager for working with multiple GPUs in the system.
///
/// # Thread Safety
///
/// `GpuManager` is `Send` and `Sync`, meaning it can be safely shared between threads.
/// The internal cache uses thread-safe primitives to ensure concurrent access is safe.
///
/// For multi-threaded applications, wrap `GpuManager` in `Arc` for shared ownership:
///
/// ```
/// use gpu_info::GpuManager;
/// use std::sync::Arc;
///
/// let manager = Arc::new(GpuManager::new());
///
/// // Clone Arc for each thread
/// let manager_clone = Arc::clone(&manager);
/// std::thread::spawn(move || {
///     let gpu = manager_clone.get_gpu_cached(0);
///     // ...
/// });
/// ```
///
/// # Caching
///
/// The manager maintains an internal cache with configurable TTL (default: 500ms).
/// Cache operations are thread-safe and use `Arc<GpuInfo>` for zero-copy reads.
#[derive(Debug, Clone)]
pub struct GpuManager {
    /// List of all detected GPUs
    gpus: Vec<GpuInfo>,
    /// Index of the primary GPU (used by default)
    primary_gpu_index: usize,
    /// GPU information cache with unified caching utilities
    ///
    /// This cache eliminates duplication by using the common caching infrastructure.
    cache: crate::cache_utils::MultiGpuInfoCache,
}
impl Default for GpuManager {
    fn default() -> Self {
        Self::new()
    }
}
impl GpuManager {
    /// Creates a new GPU manager with automatic detection
    pub fn new() -> Self {
        let mut manager = Self {
            gpus: Vec::new(),
            primary_gpu_index: 0,
            cache: crate::cache_utils::MultiGpuInfoCache::new(Duration::from_millis(500)),
        };
        manager.detect_all_gpus();
        manager
    }
    /// Creates a manager with configurable cache TTL
    pub fn with_cache_ttl(cache_ttl: Duration) -> Self {
        let mut manager = Self {
            gpus: Vec::new(),
            primary_gpu_index: 0,
            cache: crate::cache_utils::MultiGpuInfoCache::new(cache_ttl),
        };
        manager.detect_all_gpus();
        manager
    }
    /// Creates a manager with configurable cache TTL and maximum size
    pub fn with_cache_config(cache_ttl: Duration, max_entries: usize) -> Self {
        let mut manager = Self {
            gpus: Vec::new(),
            primary_gpu_index: 0,
            cache: crate::cache_utils::MultiGpuInfoCache::with_max_entries(cache_ttl, max_entries),
        };
        manager.detect_all_gpus();
        manager
    }
    /// Detects all GPUs in the system
    pub fn detect_all_gpus(&mut self) {
        self.gpus.clear();
        info!("Starting multi-GPU detection");
        #[cfg(target_os = "windows")]
        {
            self.detect_windows_gpus();
        }
        #[cfg(target_os = "linux")]
        {
            self.detect_linux_gpus();
        }
        #[cfg(target_os = "macos")]
        {
            self.detect_macos_gpus();
        }
        if self.gpus.is_empty() {
            warn!("No GPUs detected in the system");
            self.gpus.push(GpuInfo::unknown());
        } else {
            info!("Detected {} GPU(s) in the system", self.gpus.len());
            self.select_primary_gpu();
        }
    }
    #[cfg(target_os = "windows")]
    fn detect_windows_gpus(&mut self) {
        use crate::providers::{amd, intel, nvidia};
        // NVIDIA GPUs
        if let Ok(nvidia_gpus) = nvidia::detect_nvidia_gpus() {
            for (index, gpu) in nvidia_gpus.into_iter().enumerate() {
                info!("Found NVIDIA GPU #{}: {:?}", index, gpu.name_gpu);
                self.gpus.push(gpu);
            }
        }
        // AMD GPUs
        if let Ok(amd_gpus) = amd::detect_amd_gpus() {
            for (index, gpu) in amd_gpus.into_iter().enumerate() {
                info!("Found AMD GPU #{}: {:?}", index, gpu.name_gpu);
                self.gpus.push(gpu);
            }
        }
        // Intel GPUs
        let intel_gpus = intel::detect_intel_gpus();
        for (index, gpu) in intel_gpus.into_iter().enumerate() {
            info!("Found Intel GPU #{}: {:?}", index, gpu.name_gpu);
            self.gpus.push(gpu);
        }
    }
    #[cfg(target_os = "linux")]
    fn detect_linux_gpus(&mut self) {
        use crate::gpu_info::GpuProvider;
        use crate::providers::linux::{AmdLinuxProvider, NvidiaLinuxProvider};
        // NVIDIA GPUs
        let nvidia_provider = NvidiaLinuxProvider::new();
        match nvidia_provider.detect_gpus() {
            Ok(nvidia_gpus) => {
                for gpu in nvidia_gpus {
                    info!("Found NVIDIA GPU: {:?}", gpu.name_gpu);
                    self.gpus.push(gpu);
                }
            }
            Err(e) => {
                warn!("Failed to detect NVIDIA GPUs: {}", e);
            }
        }
        // AMD GPUs
        let amd_provider = AmdLinuxProvider::new();
        match amd_provider.detect_gpus() {
            Ok(amd_gpus) => {
                for gpu in amd_gpus {
                    info!("Found AMD GPU: {:?}", gpu.name_gpu);
                    self.gpus.push(gpu);
                }
            }
            Err(e) => {
                warn!("Failed to detect AMD GPUs: {}", e);
            }
        }
    }
    #[cfg(target_os = "macos")]
    fn detect_macos_gpus(&mut self) {
        use crate::macos;
        let gpus = macos::get_all_gpus();
        for gpu in gpus {
            info!("Found macOS GPU: {:?}", gpu.name_gpu);
            self.gpus.push(gpu);
        }
    }
    /// Selects the primary GPU (priority to discrete GPUs)
    fn select_primary_gpu(&mut self) {
        for (index, gpu) in self.gpus.iter().enumerate() {
            match gpu.vendor {
                Vendor::Nvidia | Vendor::Amd => {
                    self.primary_gpu_index = index;
                    info!(
                        "Selected primary GPU: {} (index {})",
                        gpu.name_gpu.as_deref().unwrap_or("Unknown"),
                        index
                    );
                    return;
                }
                _ => {
                    continue;
                }
            }
        }
        if !self.gpus.is_empty() {
            self.primary_gpu_index = 0;
            info!(
                "Selected primary GPU: {} (index 0)",
                self.gpus[0].name_gpu.as_deref().unwrap_or("Unknown")
            );
        }
    }
    /// Returns the number of detected GPUs
    pub fn gpu_count(&self) -> usize {
        self.gpus.len()
    }
    /// Returns information about all GPUs
    pub fn get_all_gpus(&self) -> &Vec<GpuInfo> {
        &self.gpus
    }
    /// Returns a copy of all GPUs
    pub fn get_all_gpus_owned(&self) -> Vec<GpuInfo> {
        self.gpus.clone()
    }
    /// Returns the primary GPU
    pub fn get_primary_gpu(&self) -> Option<&GpuInfo> {
        self.gpus.get(self.primary_gpu_index)
    }
    /// Returns a copy of the primary GPU
    pub fn get_primary_gpu_owned(&self) -> Option<GpuInfo> {
        self.gpus.get(self.primary_gpu_index).cloned()
    }
    /// Returns GPU by index
    pub fn get_gpu_by_index(&self, index: usize) -> Option<&GpuInfo> {
        self.gpus.get(index)
    }
    /// Returns a copy of GPU by index
    pub fn get_gpu_by_index_owned(&self, index: usize) -> Option<GpuInfo> {
        self.gpus.get(index).cloned()
    }
    /// Returns GPUs by vendor
    pub fn get_gpus_by_vendor(&self, vendor: Vendor) -> Vec<&GpuInfo> {
        self.gpus
            .iter()
            .filter(|gpu| gpu.vendor == vendor)
            .collect()
    }
    /// Returns copies of GPUs by vendor
    pub fn get_gpus_by_vendor_owned(&self, vendor: Vendor) -> Vec<GpuInfo> {
        self.gpus
            .iter()
            .filter(|gpu| gpu.vendor == vendor)
            .cloned()
            .collect()
    }
    /// Sets the primary GPU
    ///
    /// # Errors
    ///
    /// Returns [`GpuError::GpuNotFound`] if the index is out of bounds.
    pub fn set_primary_gpu(&mut self, index: usize) -> Result<()> {
        if index >= self.gpus.len() {
            return Err(GpuError::GpuNotFound);
        }
        self.primary_gpu_index = index;
        info!(
            "Primary GPU changed to index {}: {}",
            index,
            self.gpus[index].name_gpu.as_deref().unwrap_or("Unknown")
        );
        Ok(())
    }
    /// Updates information about all GPUs
    ///
    /// # Errors
    ///
    /// Returns the first error encountered during GPU updates. All GPUs are
    /// attempted to be updated even if some fail.
    pub fn refresh_all_gpus(&mut self) -> Result<()> {
        debug!("Refreshing information for all {} GPUs", self.gpus.len());
        let mut errors = Vec::new();
        for (index, gpu) in self.gpus.iter_mut().enumerate() {
            if let Err(e) = Self::update_single_gpu_static(gpu) {
                error!("Failed to update GPU #{}: {}", index, e);
                errors.push((index, e));
            }
        }
        self.cache.clear_all();
        if let Some((_, err)) = errors.into_iter().next() {
            Err(err)
        } else {
            Ok(())
        }
    }
    /// Updates information about a specific GPU
    ///
    /// # Errors
    ///
    /// Returns an error in the following cases:
    /// - [`GpuError::GpuNotFound`] - The index is out of bounds
    /// - Provider-specific errors if the GPU update fails
    pub fn refresh_gpu(&mut self, index: usize) -> Result<()> {
        let gpu = self.gpus.get_mut(index).ok_or(GpuError::GpuNotFound)?;
        Self::update_single_gpu_static(gpu)?;
        self.cache.set(index, gpu.clone());
        Ok(())
    }
    /// Updates information about the primary GPU
    ///
    /// # Errors
    ///
    /// Returns an error if the primary GPU update fails.
    pub fn refresh_primary_gpu(&mut self) -> Result<()> {
        self.refresh_gpu(self.primary_gpu_index)
    }
    /// Internal function for updating a single GPU
    fn update_single_gpu_static(gpu: &mut GpuInfo) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            use crate::providers::{amd, intel, nvidia};
            match gpu.vendor {
                Vendor::Nvidia => nvidia::update_nvidia_info(gpu),
                Vendor::Amd => amd::update_amd_info(gpu),
                Vendor::Intel(_) => intel::update_intel_info(gpu),
                _ => {
                    warn!("GPU update not implemented for vendor: {:?}", gpu.vendor);
                    Ok(())
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            use crate::gpu_info::GpuProvider;
            use crate::providers::linux::{AmdLinuxProvider, NvidiaLinuxProvider};
            match gpu.vendor {
                Vendor::Nvidia => {
                    let nvidia_provider = NvidiaLinuxProvider::new();
                    nvidia_provider.update_gpu(gpu)
                }
                Vendor::Amd => {
                    let amd_provider = AmdLinuxProvider::new();
                    amd_provider.update_gpu(gpu)
                }
                _ => {
                    warn!("GPU update not implemented for vendor: {:?}", gpu.vendor);
                    Ok(())
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            crate::macos::update_gpu_info(gpu)
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            warn!("GPU update not implemented for vendor: {:?}", gpu.vendor);
            Ok(())
        }
    }
    /// Returns GPU with caching (zero-copy)
    ///
    /// Returns `Arc<GpuInfo>` for efficient sharing without cloning.
    /// Use `get_gpu_cached_owned()` if you need to mutate the data.
    ///
    /// This method automatically updates GPU metrics if cache is expired.
    ///
    /// # Time Complexity
    ///
    /// - Cache hit: O(1), ~0.1-0.5ms
    /// - Cache miss: O(1) + FFI call time, ~1-200ms depending on vendor
    ///
    /// # Memory
    ///
    /// Returns `Arc<GpuInfo>` (8 bytes pointer) - zero-copy for read-only access.
    pub fn get_gpu_cached(&self, index: usize) -> Option<Arc<GpuInfo>> {
        if let Some(cached_gpu) = self.cache.get(&index) {
            debug!("Returning cached GPU #{}", index);
            return Some(cached_gpu);
        }

        if let Some(mut gpu) = self.get_gpu_by_index_owned(index) {
            if let Err(e) = Self::update_single_gpu_static(&mut gpu) {
                warn!("Failed to update GPU #{} metrics: {}", index, e);
            }
            self.cache.set(index, gpu.clone());
            debug!("Populated cache for GPU #{} with updated metrics", index);
            self.cache.get(&index)
        } else {
            None
        }
    }

    /// Returns GPU with caching (owned copy)
    ///
    /// Returns a cloned copy of cached GPU information.
    /// Use this when you need to mutate the GPU info.
    /// For read-only access, prefer `get_gpu_cached()` which is more efficient.
    ///
    /// # Time Complexity
    ///
    /// - Cache hit: O(1), ~0.1-0.5ms + clone time
    /// - Cache miss: O(1) + FFI call time, ~1-200ms depending on vendor
    ///
    /// # Memory
    ///
    /// Allocates a new `GpuInfo` (~128 bytes) on each call.
    pub fn get_gpu_cached_owned(&self, index: usize) -> Option<GpuInfo> {
        self.cache.get_owned(&index).or_else(|| {
            if let Some(gpu) = self.get_gpu_by_index_owned(index) {
                self.cache.set(index, gpu.clone());
                Some(gpu)
            } else {
                None
            }
        })
    }

    /// Returns primary GPU with caching (zero-copy)
    ///
    /// Returns `Arc<GpuInfo>` for efficient sharing without cloning.
    /// Use `get_primary_gpu_cached_owned()` if you need to mutate the data.
    ///
    /// # Time Complexity
    ///
    /// - Cache hit: O(1), ~0.1-0.5ms
    /// - Cache miss: O(1) + FFI call time, ~1-200ms depending on vendor
    pub fn get_primary_gpu_cached(&self) -> Option<Arc<GpuInfo>> {
        self.get_gpu_cached(self.primary_gpu_index)
    }

    /// Returns primary GPU with caching (owned copy)
    ///
    /// Returns a cloned copy of cached primary GPU information.
    /// Use this when you need to mutate the GPU info.
    /// For read-only access, prefer `get_primary_gpu_cached()` which is more efficient.
    pub fn get_primary_gpu_cached_owned(&self) -> Option<GpuInfo> {
        self.get_gpu_cached_owned(self.primary_gpu_index)
    }
    /// Returns GPU statistics
    pub fn get_gpu_statistics(&self) -> GpuStatistics {
        let mut stats = GpuStatistics::default();
        for gpu in &self.gpus {
            match gpu.vendor {
                Vendor::Nvidia => {
                    stats.nvidia_count += 1;
                }
                Vendor::Amd => {
                    stats.amd_count += 1;
                }
                Vendor::Intel(_) => {
                    stats.intel_count += 1;
                }
                Vendor::Apple => {
                    stats.apple_count += 1;
                }
                Vendor::Unknown => {
                    stats.unknown_count += 1;
                }
            }
            if let Some(temp) = gpu.temperature {
                stats.total_temperature += temp;
                stats.temperature_readings += 1;
            }
            if let Some(power) = gpu.power_usage {
                stats.total_power_usage += power;
                stats.power_readings += 1;
            }
        }
        stats.total_gpus = self.gpus.len();
        stats
    }
    /// Checks if all GPUs are active
    pub fn all_gpus_active(&self) -> bool {
        self.gpus.iter().all(|gpu| gpu.active.unwrap_or(false))
    }
    /// Returns a list of active GPU indices
    pub fn get_active_gpu_indices(&self) -> Vec<usize> {
        self.gpus
            .iter()
            .enumerate()
            .filter(|(_, gpu)| gpu.active.unwrap_or(false))
            .map(|(index, _)| index)
            .collect()
    }
    /// Gets cache statistics
    pub fn get_cache_stats(&self) -> Option<crate::cache_utils::CacheStats> {
        self.cache.get_stats()
    }

    /// Creates a query builder for filtering GPUs.
    ///
    /// The query is lazy - no filtering happens until a terminal method
    /// (`collect()`, `first()`, `count()`, `exists()`) is called.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::{GpuManager, Vendor};
    ///
    /// let manager = GpuManager::new();
    ///
    /// // Find all NVIDIA GPUs
    /// let nvidia_gpus = manager.query().vendor(Vendor::Nvidia).collect();
    ///
    /// // Count GPUs with temperature above 70°C
    /// let hot_count = manager.query().min_temperature(70.0).count();
    ///
    /// // Check if any active GPU exists
    /// let has_active = manager.query().active_only().exists();
    /// ```
    pub fn query(&self) -> GpuQuery<'_> {
        GpuQuery::new(self)
    }

    /// Returns an iterator over GPU references.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let manager = GpuManager::new();
    ///
    /// for gpu in manager.iter() {
    ///     println!("GPU: {:?}", gpu.name_gpu());
    /// }
    /// ```
    pub fn iter(&self) -> std::slice::Iter<'_, GpuInfo> {
        self.gpus.iter()
    }

    /// Returns a mutable iterator over GPU references.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let mut manager = GpuManager::new();
    ///
    /// for gpu in manager.iter_mut() {
    ///     // Modify GPU info if needed
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, GpuInfo> {
        self.gpus.iter_mut()
    }
}

/// Allows iterating over GPUs with `for gpu in &manager`.
///
/// # Examples
///
/// ```
/// use gpu_info::GpuManager;
///
/// let manager = GpuManager::new();
///
/// for gpu in &manager {
///     println!("GPU: {:?}", gpu.name_gpu());
/// }
/// ```
impl<'a> IntoIterator for &'a GpuManager {
    type Item = &'a GpuInfo;
    type IntoIter = std::slice::Iter<'a, GpuInfo>;

    fn into_iter(self) -> Self::IntoIter {
        self.gpus.iter()
    }
}

/// Allows mutable iteration over GPUs with `for gpu in &mut manager`.
impl<'a> IntoIterator for &'a mut GpuManager {
    type Item = &'a mut GpuInfo;
    type IntoIter = std::slice::IterMut<'a, GpuInfo>;

    fn into_iter(self) -> Self::IntoIter {
        self.gpus.iter_mut()
    }
}

/// Allows constructing a `GpuManager` from an iterator of `GpuInfo`.
///
/// This is useful for creating a manager with pre-defined GPUs,
/// such as in testing scenarios or when loading GPU data from a file.
///
/// # Examples
///
/// ```
/// use gpu_info::{GpuManager, GpuInfo, Vendor};
/// use std::iter::FromIterator;
///
/// let gpus = vec![
///     GpuInfo::mock_nvidia(),
///     GpuInfo::mock_amd(),
/// ];
///
/// let manager = GpuManager::from_iter(gpus);
/// assert_eq!(manager.gpu_count(), 2);
/// ```
///
/// Using `collect()`:
///
/// ```
/// use gpu_info::{GpuManager, GpuInfo};
///
/// let manager: GpuManager = vec![
///     GpuInfo::mock_nvidia(),
///     GpuInfo::mock_intel(),
/// ].into_iter().collect();
///
/// assert_eq!(manager.gpu_count(), 2);
/// ```
impl FromIterator<GpuInfo> for GpuManager {
    fn from_iter<I: IntoIterator<Item = GpuInfo>>(iter: I) -> Self {
        let gpus: Vec<GpuInfo> = iter.into_iter().collect();
        let gpu_count = gpus.len();

        let manager = Self {
            gpus,
            primary_gpu_index: 0,
            cache: crate::cache_utils::MultiGpuInfoCache::new(Duration::from_millis(500)),
        };

        // Pre-populate cache with the provided GPUs
        for (i, gpu) in manager.gpus.iter().enumerate() {
            manager.cache.set(i, gpu.clone());
        }

        debug!("Created GpuManager from iterator with {} GPU(s)", gpu_count);
        manager
    }
}

/// Allows extending a `GpuManager` with additional GPUs.
///
/// This is useful for adding GPUs discovered from different sources
/// or for dynamically adding mock GPUs in tests.
///
/// # Examples
///
/// ```
/// use gpu_info::{GpuManager, GpuInfo};
///
/// let mut manager = GpuManager::from_iter(vec![GpuInfo::mock_nvidia()]);
/// assert_eq!(manager.gpu_count(), 1);
///
/// // Extend with more GPUs
/// manager.extend(vec![GpuInfo::mock_amd(), GpuInfo::mock_intel()]);
/// assert_eq!(manager.gpu_count(), 3);
/// ```
///
/// Using `extend()` with an iterator:
///
/// ```
/// use gpu_info::{GpuManager, GpuInfo, Vendor};
///
/// let mut manager = GpuManager::from_iter(std::iter::empty());
///
/// // Add GPUs one by one
/// manager.extend(std::iter::once(GpuInfo::mock_nvidia()));
/// manager.extend(std::iter::once(GpuInfo::mock_amd()));
///
/// assert_eq!(manager.gpu_count(), 2);
/// ```
impl Extend<GpuInfo> for GpuManager {
    fn extend<I: IntoIterator<Item = GpuInfo>>(&mut self, iter: I) {
        let start_index = self.gpus.len();

        for (i, gpu) in iter.into_iter().enumerate() {
            let cache_index = start_index + i;
            self.cache.set(cache_index, gpu.clone());
            self.gpus.push(gpu);
        }

        debug!("Extended GpuManager: now has {} GPU(s)", self.gpus.len());
    }
}

/// GPU statistics aggregated across all GPUs in the system.
///
/// This struct provides summary statistics about all detected GPUs,
/// including counts by vendor and aggregate metrics like temperature
/// and power consumption.
#[derive(Debug, Default, Clone)]
pub struct GpuStatistics {
    /// Total number of GPUs detected in the system.
    pub total_gpus: usize,
    /// Number of NVIDIA GPUs detected.
    pub nvidia_count: usize,
    /// Number of AMD GPUs detected.
    pub amd_count: usize,
    /// Number of Intel GPUs detected.
    pub intel_count: usize,
    /// Number of Apple GPUs detected.
    pub apple_count: usize,
    /// Number of GPUs with unknown vendor.
    pub unknown_count: usize,
    /// Sum of all GPU temperatures for averaging.
    pub total_temperature: f32,
    /// Number of GPUs reporting temperature readings.
    pub temperature_readings: usize,
    /// Sum of all GPU power usage values.
    pub total_power_usage: f32,
    /// Number of GPUs reporting power readings.
    pub power_readings: usize,
}
impl GpuStatistics {
    /// Returns the average temperature across all GPUs
    pub fn average_temperature(&self) -> Option<f32> {
        if self.temperature_readings > 0 {
            Some(self.total_temperature / (self.temperature_readings as f32))
        } else {
            None
        }
    }
    /// Returns the total power consumption of all GPUs
    pub fn total_power_consumption(&self) -> Option<f32> {
        if self.power_readings > 0 {
            Some(self.total_power_usage)
        } else {
            None
        }
    }
}
// Global static variable for singleton access
use std::sync::OnceLock;
static GPU_MANAGER: OnceLock<Arc<Mutex<GpuManager>>> = OnceLock::new();
/// Returns the global GpuManager instance
pub fn global_gpu_manager() -> Arc<Mutex<GpuManager>> {
    GPU_MANAGER
        .get_or_init(|| Arc::new(Mutex::new(GpuManager::new())))
        .clone()
}
/// Convenience function for getting the primary GPU (owned copy)
///
/// Returns owned `GpuInfo` for backward compatibility.
/// For more efficient access, use `GpuManager::get_primary_gpu_cached()`.
pub fn get_primary_gpu() -> Option<GpuInfo> {
    let manager = global_gpu_manager();
    let result = if let Ok(mgr) = manager.lock() {
        mgr.get_primary_gpu_cached_owned()
    } else {
        None
    };
    result
}

/// Convenience function for getting the primary GPU (zero-copy)
///
/// Returns `Arc<GpuInfo>` for efficient sharing without cloning.
pub fn get_primary_gpu_arc() -> Option<Arc<GpuInfo>> {
    let manager = global_gpu_manager();
    let mgr = manager.lock().ok()?;
    mgr.get_primary_gpu_cached()
}
/// Convenience function for getting all GPUs
pub fn get_all_gpus() -> Vec<GpuInfo> {
    let manager = global_gpu_manager();
    let result = if let Ok(mgr) = manager.lock() {
        mgr.get_all_gpus_owned()
    } else {
        Vec::new()
    };
    result
}
/// Convenience function for getting the GPU count
pub fn get_gpu_count() -> usize {
    let manager = global_gpu_manager();
    let result = if let Ok(mgr) = manager.lock() {
        mgr.gpu_count()
    } else {
        0
    };
    result
}

/// Compile-time assertion that `GpuManager` implements `Send`.
///
/// This ensures `GpuManager` can be safely transferred between threads.
const _: () = {
    const fn assert_send<T: Send>() {}
    assert_send::<GpuManager>();
};

/// Compile-time assertion that `GpuManager` implements `Sync`.
///
/// This ensures `GpuManager` can be safely shared between threads via references.
const _: () = {
    const fn assert_sync<T: Sync>() {}
    assert_sync::<GpuManager>();
};

/// Compile-time assertion that `GpuManager` implements both `Send` and `Sync`.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GpuManager>();
};
