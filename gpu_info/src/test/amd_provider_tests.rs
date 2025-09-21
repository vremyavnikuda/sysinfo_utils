//! Tests for AMD GPU provider on Linux
#[cfg(test)]
mod amd_provider_tests {
    use crate::providers::linux::amd::AmdLinuxProvider;
    use crate::gpu_info::GpuProvider;
    use crate::vendor::Vendor;
    
    #[test]
    fn test_amd_provider_creation() {
        let provider = AmdLinuxProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Amd);
    }
    
    // Note: We can't directly test get_memory_info since it's private
    // But we can test the overall functionality through the public API
}