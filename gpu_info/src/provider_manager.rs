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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vendor::Vendor;
    struct MockProvider;
    impl MockProvider {
        fn new() -> Self {
            Self
        }
    }
    impl Default for MockProvider {
        fn default() -> Self {
            Self::new()
        }
    }
    impl GpuProvider for MockProvider {
        fn detect_gpus(&self) -> Result<Vec<GpuInfo>> {
            Ok(vec![GpuInfo::write_vendor(Vendor::Nvidia)])
        }
        fn update_gpu(&self, gpu: &mut GpuInfo) -> Result<()> {
            gpu.name_gpu = Some("Mock GPU".to_string());
            Ok(())
        }
        fn get_vendor(&self) -> Vendor {
            Vendor::Nvidia
        }
    }
    #[test]
    fn test_provider_manager_creation() {
        let manager = GpuProviderManager::new();
        assert!(manager.get_registered_vendors().is_empty());
    }
    #[test]
    fn test_provider_registration() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new());
        assert_eq!(manager.get_registered_vendors().len(), 1);
        assert!(manager.is_vendor_supported(&Vendor::Nvidia));
    }
    #[test]
    fn test_gpu_detection() {
        let mut manager = GpuProviderManager::new();
        manager.register_provider(Vendor::Nvidia, MockProvider::new());
        let gpus = manager.detect_all_gpus();
        assert_eq!(gpus.len(), 1);
        assert_eq!(gpus[0].vendor, Vendor::Nvidia);
    }
}
