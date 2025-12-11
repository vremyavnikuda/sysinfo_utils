//! GPU Provider Manager for unified GPU detection and management
//!
//! This module provides a centralized manager for all GPU providers,
//! allowing unified detection and updating of GPUs from different vendors.
use crate::gpu_info::{GpuInfo, GpuProvider, Result};
use crate::vendor::Vendor;
use log::{error, info, warn};
use std::collections::HashMap;
/// Manager for all GPU providers
pub struct GpuProviderManager {
    providers: HashMap<Vendor, Box<dyn GpuProvider>>,
}
impl GpuProviderManager {
    /// Create a new provider manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    /// Register a provider for a specific vendor
    pub fn register_provider<P: GpuProvider + 'static>(&mut self, vendor: Vendor, provider: P) {
        self.providers.insert(vendor, Box::new(provider));
    }
    /// Detect all GPUs from all registered providers
    pub fn detect_all_gpus(&self) -> Vec<GpuInfo> {
        let mut all_gpus = Vec::new();
        for (vendor, provider) in &self.providers {
            match provider.detect_gpus() {
                Ok(mut gpus) => {
                    info!("Found {} {} GPU(s)", gpus.len(), vendor);
                    all_gpus.append(&mut gpus);
                }
                Err(e) => {
                    error!("Failed to detect {} GPUs: {}", vendor, e);
                }
            }
        }
        all_gpus
    }
    /// Update a specific GPU using the appropriate provider
    pub fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
        for (vendor, provider) in &self.providers {
            let is_match = match (vendor, &gpu.vendor) {
                (Vendor::Intel(_), Vendor::Intel(_)) => true,
                _ => std::mem::discriminant(vendor) == std::mem::discriminant(&gpu.vendor),
            };
            if is_match {
                return provider.update_gpu(gpu);
            }
        }
        warn!("No provider registered for vendor: {:?}", gpu.vendor);
        Err(crate::gpu_info::GpuError::GpuNotActive)
    }
    /// Get all registered vendors
    pub fn get_registered_vendors(&self) -> Vec<Vendor> {
        self.providers.keys().cloned().collect()
    }
    /// Check if a vendor is supported
    pub fn is_vendor_supported(&self, vendor: &Vendor) -> bool {
        self.providers.contains_key(vendor)
    }
}
impl Default for GpuProviderManager {
    fn default() -> Self {
        Self::new()
    }
}
