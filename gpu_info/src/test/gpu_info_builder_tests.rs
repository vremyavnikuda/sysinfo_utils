#[cfg(test)]
mod tests {
    use crate::gpu_info::GpuInfo;
    use crate::vendor::Vendor;

    #[test]
    fn test_builder_basic() {
        let gpu = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("NVIDIA GeForce RTX 3080")
            .temperature(65.0)
            .utilization(75.0)
            .build();
        assert_eq!(gpu.vendor(), Vendor::Nvidia);
        assert_eq!(gpu.name_gpu(), Some("NVIDIA GeForce RTX 3080"));
        assert_eq!(gpu.temperature(), Some(65.0));
        assert_eq!(gpu.utilization(), Some(75.0));
    }

    #[test]
    fn test_builder_all_fields() {
        let gpu = GpuInfo::builder()
            .vendor(Vendor::Amd)
            .name("AMD Radeon RX 6800 XT")
            .temperature(70.0)
            .utilization(80.0)
            .power_usage(250.0)
            .core_clock(2250)
            .memory_util(60.0)
            .memory_clock(2000)
            .active(true)
            .power_limit(300.0)
            .memory_total(16)
            .driver_version("23.11.1")
            .max_clock_speed(2500)
            .build();
        assert_eq!(gpu.vendor(), Vendor::Amd);
        assert_eq!(gpu.name_gpu(), Some("AMD Radeon RX 6800 XT"));
        assert_eq!(gpu.temperature(), Some(70.0));
        assert_eq!(gpu.utilization(), Some(80.0));
        assert_eq!(gpu.power_usage(), Some(250.0));
        assert_eq!(gpu.core_clock(), Some(2250));
        assert_eq!(gpu.memory_util(), Some(60.0));
        assert_eq!(gpu.memory_clock(), Some(2000));
        assert_eq!(gpu.active(), Some(true));
        assert_eq!(gpu.power_limit(), Some(300.0));
        assert_eq!(gpu.memory_total(), Some(16));
        assert_eq!(gpu.driver_version(), Some("23.11.1"));
        assert_eq!(gpu.max_clock_speed(), Some(2500));
    }

    #[test]
    fn test_builder_defaults() {
        let gpu = GpuInfo::builder().build();
        assert_eq!(gpu.vendor(), Vendor::Unknown);
        assert_eq!(gpu.name_gpu(), None);
        assert_eq!(gpu.temperature(), None);
        assert_eq!(gpu.utilization(), None);
        assert_eq!(gpu.power_usage(), None);
    }

    #[test]
    fn test_builder_partial() {
        let gpu = GpuInfo::builder()
            .vendor(Vendor::Intel(crate::vendor::IntelGpuType::Integrated))
            .name("Intel UHD Graphics 630")
            .active(true)
            .build();
        assert_eq!(
            gpu.vendor(),
            Vendor::Intel(crate::vendor::IntelGpuType::Integrated)
        );
        assert_eq!(gpu.name_gpu(), Some("Intel UHD Graphics 630"));
        assert_eq!(gpu.active(), Some(true));
        assert_eq!(gpu.temperature(), None);
        assert_eq!(gpu.utilization(), None);
    }

    #[test]
    fn test_builder_method_chaining() {
        let gpu = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("Test GPU")
            .temperature(50.0)
            .utilization(25.0)
            .power_usage(100.0)
            .build();
        assert_eq!(gpu.vendor(), Vendor::Nvidia);
        assert_eq!(gpu.name_gpu(), Some("Test GPU"));
        assert_eq!(gpu.temperature(), Some(50.0));
        assert_eq!(gpu.utilization(), Some(25.0));
        assert_eq!(gpu.power_usage(), Some(100.0));
    }

    #[test]
    fn test_builder_string_conversion() {
        let gpu = GpuInfo::builder()
            .name(String::from("Owned String"))
            .driver_version("Borrowed str")
            .build();
        assert_eq!(gpu.name_gpu(), Some("Owned String"));
        assert_eq!(gpu.driver_version(), Some("Borrowed str"));
    }

    #[test]
    fn test_builder_vs_unknown() {
        let builder_gpu = GpuInfo::builder().build();
        let unknown_gpu = GpuInfo::unknown();
        assert_eq!(builder_gpu.vendor(), unknown_gpu.vendor());
        assert_eq!(builder_gpu.name_gpu(), unknown_gpu.name_gpu());
        assert_eq!(builder_gpu.temperature(), unknown_gpu.temperature());
    }

    #[test]
    fn test_builder_vs_write_vendor() {
        let builder_gpu = GpuInfo::builder().vendor(Vendor::Nvidia).build();
        let vendor_gpu = GpuInfo::write_vendor(Vendor::Nvidia);
        assert_eq!(builder_gpu.vendor(), vendor_gpu.vendor());
        assert_eq!(builder_gpu.name_gpu(), vendor_gpu.name_gpu());
    }

    #[test]
    fn test_gpu_info_eq_identical() {
        let gpu1 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .build();
        let gpu2 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .build();
        assert_eq!(gpu1, gpu2);
    }

    #[test]
    fn test_gpu_info_eq_different_metrics() {
        let gpu1 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .build();
        let gpu2 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(70.0)
            .build();
        assert_ne!(gpu1, gpu2);
    }

    #[test]
    fn test_gpu_info_hash_same_identity() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let gpu1 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .build();
        let gpu2 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(70.0)
            .build();
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        gpu1.hash(&mut hasher1);
        gpu2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_gpu_info_hash_different_identity() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let gpu1 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .build();
        let gpu2 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 4090")
            .build();
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        gpu1.hash(&mut hasher1);
        gpu2.hash(&mut hasher2);
        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_gpu_info_in_hashset() {
        use std::collections::HashSet;
        let gpu1 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .build();
        let gpu2 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .build();
        let gpu3 = GpuInfo::builder()
            .vendor(Vendor::Amd)
            .name("RX 6800 XT")
            .build();
        let mut set = HashSet::new();
        set.insert(gpu1.clone());
        assert!(set.contains(&gpu2));
        assert!(!set.contains(&gpu3));
        set.insert(gpu3);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_gpu_info_in_hashmap() {
        use std::collections::HashMap;
        let gpu = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .build();
        let mut map: HashMap<GpuInfo, String> = HashMap::new();
        map.insert(gpu.clone(), "Primary GPU".to_string());
        assert_eq!(map.get(&gpu), Some(&"Primary GPU".to_string()));
        let other_gpu = GpuInfo::builder()
            .vendor(Vendor::Amd)
            .name("RX 6800 XT")
            .build();
        assert_eq!(map.get(&other_gpu), None);
    }

    #[test]
    fn test_gpu_info_eq_reflexive() {
        let gpu = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .build();
        assert_eq!(gpu, gpu);
    }

    #[test]
    fn test_gpu_info_eq_symmetric() {
        let gpu1 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .build();
        let gpu2 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .build();
        assert_eq!(gpu1, gpu2);
        assert_eq!(gpu2, gpu1);
    }

    #[test]
    fn test_gpu_info_eq_transitive() {
        let gpu1 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .build();
        let gpu2 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .build();
        let gpu3 = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .build();
        assert_eq!(gpu1, gpu2);
        assert_eq!(gpu2, gpu3);
        assert_eq!(gpu1, gpu3);
    }

    #[test]
    fn test_gpu_info_clone() {
        let gpu = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .driver_version("535.154.05")
            .build();
        let cloned = gpu.clone();
        assert_eq!(gpu, cloned);
        assert_eq!(cloned.vendor(), Vendor::Nvidia);
        assert_eq!(cloned.name_gpu(), Some("RTX 3080"));
        assert_eq!(cloned.temperature(), Some(65.0));
        assert_eq!(cloned.driver_version(), Some("535.154.05"));
    }

    #[test]
    fn test_gpu_info_clone_from_reuses_allocation() {
        let source = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .temperature(65.0)
            .driver_version("535.154.05")
            .build();
        let mut dest = GpuInfo::builder()
            .vendor(Vendor::Amd)
            .name("RX 6800 XT")
            .temperature(70.0)
            .driver_version("23.11.1")
            .build();
        dest.clone_from(&source);
        assert_eq!(dest.vendor(), Vendor::Nvidia);
        assert_eq!(dest.name_gpu(), Some("RTX 3080"));
        assert_eq!(dest.temperature(), Some(65.0));
        assert_eq!(dest.driver_version(), Some("535.154.05"));
        assert_eq!(dest, source);
    }

    #[test]
    fn test_gpu_info_clone_from_with_none_strings() {
        let source = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .temperature(65.0)
            .build();
        let mut dest = GpuInfo::builder()
            .vendor(Vendor::Amd)
            .name("RX 6800 XT")
            .driver_version("23.11.1")
            .build();
        dest.clone_from(&source);
        assert_eq!(dest.name_gpu(), None);
        assert_eq!(dest.driver_version(), None);
        assert_eq!(dest.vendor(), Vendor::Nvidia);
    }

    #[test]
    fn test_gpu_info_clone_from_to_none_strings() {
        let source = GpuInfo::builder()
            .vendor(Vendor::Nvidia)
            .name("RTX 3080")
            .driver_version("535.154.05")
            .build();
        let mut dest = GpuInfo::builder().vendor(Vendor::Amd).build();
        dest.clone_from(&source);
        assert_eq!(dest.name_gpu(), Some("RTX 3080"));
        assert_eq!(dest.driver_version(), Some("535.154.05"));
    }
}
