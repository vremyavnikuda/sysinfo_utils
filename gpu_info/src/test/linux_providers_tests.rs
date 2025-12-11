#[cfg(test)]
mod tests {
    use crate::{
        gpu_info::GpuProvider,
        providers::linux::{
            amd::AmdLinuxProvider, intel::IntelLinuxProvider, nvidia::NvidiaLinuxProvider,
        },
        vendor::Vendor,
    };
    use std::path::Path;

    #[test]
    fn test_nvidia_linux_provider_vendor() {
        let provider = NvidiaLinuxProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Nvidia);
    }

    #[test]
    fn test_amd_linux_provider_vendor() {
        let provider = AmdLinuxProvider::new();
        assert_eq!(provider.get_vendor(), Vendor::Amd);
    }

    #[test]
    fn test_amd_get_memory_info_with_nonexistent_paths() {
        let provider = AmdLinuxProvider::new();
        let temp_dir = std::env::temp_dir();
        let result = provider.get_memory_info(&temp_dir);
        assert_eq!(result, (None, None));
    }

    #[test]
    fn test_intel_linux_provider_creation() {
        let provider = IntelLinuxProvider::new();
        let default_provider = IntelLinuxProvider;
        assert!(matches!(provider.get_vendor(), Vendor::Intel(_)));
        assert!(matches!(default_provider.get_vendor(), Vendor::Intel(_)));
    }

    #[test]
    fn test_intel_get_memory_info_with_nonexistent_paths() {
        let provider = IntelLinuxProvider::new();
        let temp_dir = std::env::temp_dir();
        let result = provider.get_memory_info(&temp_dir);
        assert_eq!(result, (None, None));
    }

    #[test]
    fn test_intel_read_hex_file_invalid_path() {
        let provider = IntelLinuxProvider::new();
        let result = provider.read_hex_file(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }
}
