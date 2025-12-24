//! Fluent query API for filtering GPUs.
//!
//! This module provides a builder-style API for querying and filtering GPUs
//! from a [`GpuManager`]. The query is lazy - no filtering happens until
//! a terminal method is called.
//!
//! # Examples
//!
//! ```
//! use gpu_info::{GpuManager, Vendor};
//!
//! let manager = GpuManager::new();
//!
//! // Find all NVIDIA GPUs with temperature above 50°C
//! let hot_nvidia_gpus = manager
//!     .query()
//!     .vendor(Vendor::Nvidia)
//!     .min_temperature(50.0)
//!     .collect();
//!
//! // Count AMD GPUs
//! let amd_count = manager
//!     .query()
//!     .vendor(Vendor::Amd)
//!     .count();
//!
//! // Get first active GPU
//! let active_gpu = manager
//!     .query()
//!     .active_only()
//!     .first();
//! ```

use crate::gpu_info::GpuInfo;
use crate::gpu_manager::GpuManager;
use crate::vendor::Vendor;
use std::sync::Arc;

/// Query builder for filtering GPUs.
///
/// Created by calling [`GpuManager::query()`]. The query is lazy - no filtering
/// happens until a terminal method (`collect()`, `first()`, `count()`) is called.
///
/// # Examples
///
/// ```
/// use gpu_info::{GpuManager, Vendor};
///
/// let manager = GpuManager::new();
///
/// // Chain multiple filters
/// let gpus = manager
///     .query()
///     .vendor(Vendor::Nvidia)
///     .min_temperature(50.0)
///     .max_temperature(80.0)
///     .collect();
/// ```
#[derive(Debug, Clone)]
pub struct GpuQuery<'a> {
    manager: &'a GpuManager,
    vendor_filter: Option<Vendor>,
    min_temp: Option<f32>,
    max_temp: Option<f32>,
    min_util: Option<f32>,
    max_util: Option<f32>,
    active_only: bool,
    has_temperature: Option<bool>,
    has_power: Option<bool>,
}

impl<'a> GpuQuery<'a> {
    /// Creates a new query builder for the given manager.
    ///
    /// This is typically called via [`GpuManager::query()`] rather than directly.
    pub fn new(manager: &'a GpuManager) -> Self {
        Self {
            manager,
            vendor_filter: None,
            min_temp: None,
            max_temp: None,
            min_util: None,
            max_util: None,
            active_only: false,
            has_temperature: None,
            has_power: None,
        }
    }

    /// Filters GPUs by vendor.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::{GpuManager, Vendor};
    ///
    /// let manager = GpuManager::new();
    /// let nvidia_gpus = manager.query().vendor(Vendor::Nvidia).collect();
    /// ```
    pub fn vendor(mut self, vendor: Vendor) -> Self {
        self.vendor_filter = Some(vendor);
        self
    }

    /// Filters GPUs with temperature at or above the specified value.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let manager = GpuManager::new();
    /// let hot_gpus = manager.query().min_temperature(70.0).collect();
    /// ```
    pub fn min_temperature(mut self, temp: f32) -> Self {
        self.min_temp = Some(temp);
        self
    }

    /// Filters GPUs with temperature at or below the specified value.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let manager = GpuManager::new();
    /// let cool_gpus = manager.query().max_temperature(60.0).collect();
    /// ```
    pub fn max_temperature(mut self, temp: f32) -> Self {
        self.max_temp = Some(temp);
        self
    }

    /// Filters GPUs with utilization at or above the specified percentage.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let manager = GpuManager::new();
    /// let busy_gpus = manager.query().min_utilization(80.0).collect();
    /// ```
    pub fn min_utilization(mut self, util: f32) -> Self {
        self.min_util = Some(util);
        self
    }

    /// Filters GPUs with utilization at or below the specified percentage.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let manager = GpuManager::new();
    /// let idle_gpus = manager.query().max_utilization(20.0).collect();
    /// ```
    pub fn max_utilization(mut self, util: f32) -> Self {
        self.max_util = Some(util);
        self
    }

    /// Filters to only include active GPUs.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let manager = GpuManager::new();
    /// let active_gpus = manager.query().active_only().collect();
    /// ```
    pub fn active_only(mut self) -> Self {
        self.active_only = true;
        self
    }

    /// Filters GPUs that have temperature data available.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let manager = GpuManager::new();
    /// let gpus_with_temp = manager.query().with_temperature().collect();
    /// ```
    pub fn with_temperature(mut self) -> Self {
        self.has_temperature = Some(true);
        self
    }

    /// Filters GPUs that have power usage data available.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::GpuManager;
    ///
    /// let manager = GpuManager::new();
    /// let gpus_with_power = manager.query().with_power().collect();
    /// ```
    pub fn with_power(mut self) -> Self {
        self.has_power = Some(true);
        self
    }

    /// Collects all matching GPUs (terminal method).
    ///
    /// Returns `Arc<GpuInfo>` for zero-copy access. Use this when you need
    /// all matching GPUs.
    ///
    /// # Time Complexity
    ///
    /// O(n) where n is the number of GPUs in the manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::{GpuManager, Vendor};
    ///
    /// let manager = GpuManager::new();
    /// let nvidia_gpus = manager.query().vendor(Vendor::Nvidia).collect();
    /// println!("Found {} NVIDIA GPUs", nvidia_gpus.len());
    /// ```
    pub fn collect(self) -> Vec<Arc<GpuInfo>> {
        (0..self.manager.gpu_count())
            .filter_map(|i| self.manager.get_gpu_cached(i))
            .filter(|gpu| self.matches(gpu))
            .collect()
    }

    /// Returns the first matching GPU (terminal method).
    ///
    /// Returns `Arc<GpuInfo>` for zero-copy access. Use this when you only
    /// need one matching GPU.
    ///
    /// # Time Complexity
    ///
    /// O(n) worst case, but short-circuits on first match.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::{GpuManager, Vendor};
    ///
    /// let manager = GpuManager::new();
    /// if let Some(nvidia) = manager.query().vendor(Vendor::Nvidia).first() {
    ///     println!("Found NVIDIA GPU: {:?}", nvidia.name_gpu());
    /// }
    /// ```
    pub fn first(self) -> Option<Arc<GpuInfo>> {
        (0..self.manager.gpu_count())
            .filter_map(|i| self.manager.get_gpu_cached(i))
            .find(|gpu| self.matches(gpu))
    }

    /// Counts matching GPUs (terminal method).
    ///
    /// More efficient than `collect().len()` as it doesn't allocate a vector.
    ///
    /// # Time Complexity
    ///
    /// O(n) where n is the number of GPUs in the manager.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::{GpuManager, Vendor};
    ///
    /// let manager = GpuManager::new();
    /// let nvidia_count = manager.query().vendor(Vendor::Nvidia).count();
    /// println!("Found {} NVIDIA GPUs", nvidia_count);
    /// ```
    pub fn count(self) -> usize {
        (0..self.manager.gpu_count())
            .filter_map(|i| self.manager.get_gpu_cached(i))
            .filter(|gpu| self.matches(gpu))
            .count()
    }

    /// Checks if any GPU matches the query (terminal method).
    ///
    /// More efficient than `count() > 0` as it short-circuits on first match.
    ///
    /// # Examples
    ///
    /// ```
    /// use gpu_info::{GpuManager, Vendor};
    ///
    /// let manager = GpuManager::new();
    /// if manager.query().vendor(Vendor::Nvidia).exists() {
    ///     println!("NVIDIA GPU found!");
    /// }
    /// ```
    pub fn exists(self) -> bool {
        self.first().is_some()
    }

    /// Checks if a GPU matches all the query filters.
    fn matches(&self, gpu: &GpuInfo) -> bool {
        // Vendor filter
        if let Some(vendor) = &self.vendor_filter {
            if gpu.vendor != *vendor {
                return false;
            }
        }

        // Temperature filters
        if let Some(min_temp) = self.min_temp {
            match gpu.temperature {
                Some(temp) if temp >= min_temp => {}
                _ => return false,
            }
        }

        if let Some(max_temp) = self.max_temp {
            match gpu.temperature {
                Some(temp) if temp <= max_temp => {}
                _ => return false,
            }
        }

        // Utilization filters
        if let Some(min_util) = self.min_util {
            match gpu.utilization {
                Some(util) if util >= min_util => {}
                _ => return false,
            }
        }

        if let Some(max_util) = self.max_util {
            match gpu.utilization {
                Some(util) if util <= max_util => {}
                _ => return false,
            }
        }

        // Active filter
        if self.active_only && !gpu.active.unwrap_or(false) {
            return false;
        }

        // Has temperature filter
        if let Some(has_temp) = self.has_temperature {
            if has_temp && gpu.temperature.is_none() {
                return false;
            }
        }

        // Has power filter
        if let Some(has_power) = self.has_power {
            if has_power && gpu.power_usage.is_none() {
                return false;
            }
        }

        true
    }
}

// TODO: there should be no tests here. Transfer them to gpu_info\src\test
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vendor::Vendor;

    #[test]
    fn test_query_builder_creation() {
        let manager = GpuManager::new();
        let query = GpuQuery::new(&manager);
        assert!(query.vendor_filter.is_none());
        assert!(query.min_temp.is_none());
        assert!(!query.active_only);
    }

    #[test]
    fn test_query_vendor_filter() {
        let manager = GpuManager::new();
        let query = manager.query().vendor(Vendor::Nvidia);
        assert_eq!(query.vendor_filter, Some(Vendor::Nvidia));
    }

    #[test]
    fn test_query_temperature_filters() {
        let manager = GpuManager::new();
        let query = manager.query().min_temperature(50.0).max_temperature(80.0);
        assert_eq!(query.min_temp, Some(50.0));
        assert_eq!(query.max_temp, Some(80.0));
    }

    #[test]
    fn test_query_utilization_filters() {
        let manager = GpuManager::new();
        let query = manager.query().min_utilization(20.0).max_utilization(90.0);
        assert_eq!(query.min_util, Some(20.0));
        assert_eq!(query.max_util, Some(90.0));
    }

    #[test]
    fn test_query_active_only() {
        let manager = GpuManager::new();
        let query = manager.query().active_only();
        assert!(query.active_only);
    }

    #[test]
    fn test_query_chaining() {
        let manager = GpuManager::new();
        let query = manager
            .query()
            .vendor(Vendor::Nvidia)
            .min_temperature(50.0)
            .max_temperature(80.0)
            .active_only();

        assert_eq!(query.vendor_filter, Some(Vendor::Nvidia));
        assert_eq!(query.min_temp, Some(50.0));
        assert_eq!(query.max_temp, Some(80.0));
        assert!(query.active_only);
    }

    #[test]
    fn test_query_collect() {
        let manager = GpuManager::new();
        let gpus = manager.query().collect();
        assert!(!gpus.is_empty() || manager.gpu_count() == 0);
    }

    #[test]
    fn test_query_count() {
        let manager = GpuManager::new();
        let count = manager.query().count();
        assert_eq!(count, manager.gpu_count());
    }

    #[test]
    fn test_query_first() {
        let manager = GpuManager::new();
        let first = manager.query().first();
        if manager.gpu_count() > 0 {
            assert!(first.is_some());
        }
    }

    #[test]
    fn test_query_exists() {
        let manager = GpuManager::new();
        let exists = manager.query().exists();
        assert_eq!(exists, manager.gpu_count() > 0);
    }

    #[test]
    fn test_matches_vendor() {
        let manager = GpuManager::new();
        let query = GpuQuery::new(&manager).vendor(Vendor::Nvidia);
        let nvidia_gpu = GpuInfo::builder().vendor(Vendor::Nvidia).build();
        let amd_gpu = GpuInfo::builder().vendor(Vendor::Amd).build();
        assert!(query.matches(&nvidia_gpu));
        assert!(!query.matches(&amd_gpu));
    }

    #[test]
    fn test_matches_temperature_range() {
        let manager = GpuManager::new();
        let query = GpuQuery::new(&manager)
            .min_temperature(50.0)
            .max_temperature(80.0);

        let cold_gpu = GpuInfo::builder().temperature(40.0).build();
        let warm_gpu = GpuInfo::builder().temperature(65.0).build();
        let hot_gpu = GpuInfo::builder().temperature(90.0).build();
        let no_temp_gpu = GpuInfo::builder().build();
        assert!(!query.matches(&cold_gpu));
        assert!(query.matches(&warm_gpu));
        assert!(!query.matches(&hot_gpu));
        assert!(!query.matches(&no_temp_gpu));
    }

    #[test]
    fn test_matches_active_only() {
        let manager = GpuManager::new();
        let query = GpuQuery::new(&manager).active_only();
        let active_gpu = GpuInfo::builder().active(true).build();
        let inactive_gpu = GpuInfo::builder().active(false).build();
        let unknown_gpu = GpuInfo::builder().build();
        assert!(query.matches(&active_gpu));
        assert!(!query.matches(&inactive_gpu));
        assert!(!query.matches(&unknown_gpu));
    }

    #[test]
    fn test_matches_with_temperature() {
        let manager = GpuManager::new();
        let query = GpuQuery::new(&manager).with_temperature();
        let with_temp = GpuInfo::builder().temperature(65.0).build();
        let without_temp = GpuInfo::builder().build();
        assert!(query.matches(&with_temp));
        assert!(!query.matches(&without_temp));
    }

    #[test]
    fn test_matches_combined_filters() {
        let manager = GpuManager::new();
        let query = GpuQuery::new(&manager)
            .vendor(Vendor::Nvidia)
            .min_temperature(50.0)
            .active_only();

        let matching = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .temperature(65.0)
            .active(true)
            .build();

        let wrong_vendor = GpuInfo::builder()
            .vendor(Vendor::Amd)
            .temperature(65.0)
            .active(true)
            .build();

        let too_cold = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .temperature(40.0)
            .active(true)
            .build();

        let inactive = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .temperature(65.0)
            .active(false)
            .build();

        assert!(query.matches(&matching));
        assert!(!query.matches(&wrong_vendor));
        assert!(!query.matches(&too_cold));
        assert!(!query.matches(&inactive));
    }
}
