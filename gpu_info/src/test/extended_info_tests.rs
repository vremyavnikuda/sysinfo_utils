//! Comprehensive tests for extended GPU information functionality
//!
//! These tests cover extended GPU metrics, health scoring, performance analysis,
//! and conversion between basic and extended GPU information structures.

#[cfg(test)]
mod tests {
    use crate::extended_info::{
        ConnectionInfo, EncoderInfo, ExtendedGpuInfo, FanInfo, GpuInfoExtensions,
        IndividualFanInfo, MemoryInfo, OverclockingInfo, PerformanceState, ThermalInfo,
        ThrottleReason,
    };
    use crate::gpu_info::GpuInfo;
    use crate::vendor::Vendor;

    /// Test basic ExtendedGpuInfo creation
    #[test]
    fn test_extended_gpu_info_creation() {
        let basic_gpu = create_test_gpu();
        let extended_gpu = ExtendedGpuInfo::from_basic(basic_gpu.clone());

        assert_eq!(extended_gpu.base_info.vendor, basic_gpu.vendor);
        assert_eq!(extended_gpu.base_info.name_gpu, basic_gpu.name_gpu);
        assert_eq!(extended_gpu.base_info.temperature, basic_gpu.temperature);

        println!("Extended GPU info created from basic info");
    }

    /// Test unknown ExtendedGpuInfo creation
    #[test]
    fn test_extended_gpu_info_unknown() {
        let unknown_gpu = ExtendedGpuInfo::unknown();
        assert_eq!(unknown_gpu.base_info.vendor, Vendor::Unknown);
        assert!(unknown_gpu.base_info.name_gpu.is_none());
        assert!(unknown_gpu.base_info.temperature.is_none());
    }

    /// Test health score calculation with good conditions
    #[test]
    fn test_health_score_good_conditions() {
        let mut extended_gpu = create_test_extended_gpu();
        extended_gpu.thermal_info.gpu_temperature = Some(65.0);
        extended_gpu.thermal_info.is_throttling = Some(false);
        extended_gpu.base_info.memory_util = Some(70.0);
        extended_gpu.memory_info.ecc_errors = Some(0);
        let health_score = extended_gpu.health_score();
        println!("Health score with good conditions: {:.1}%", health_score);
        assert!(
            health_score >= 90.0,
            "Expected high health score, got {:.1}%",
            health_score
        );
        assert!(!extended_gpu.needs_attention());
    }

    /// Test health score calculation with poor conditions
    #[test]
    fn test_health_score_poor_conditions() {
        let mut extended_gpu = create_test_extended_gpu();
        extended_gpu.thermal_info.gpu_temperature = Some(90.0);
        extended_gpu.thermal_info.is_throttling = Some(true);
        extended_gpu.base_info.memory_util = Some(98.0);
        extended_gpu.memory_info.ecc_errors = Some(5);
        let health_score = extended_gpu.health_score();
        println!("Health score with poor conditions: {:.1}%", health_score);
        assert!(
            health_score < 50.0,
            "Expected low health score, got {:.1}%",
            health_score
        );
        assert!(extended_gpu.needs_attention());
    }

    /// Test health score calculation with mixed conditions
    #[test]
    fn test_health_score_mixed_conditions() {
        let mut extended_gpu = create_test_extended_gpu();
        extended_gpu.thermal_info.gpu_temperature = Some(78.0);
        extended_gpu.thermal_info.is_throttling = Some(false);
        extended_gpu.base_info.memory_util = Some(85.0);
        extended_gpu.memory_info.ecc_errors = Some(0);
        let health_score = extended_gpu.health_score();
        println!("Health score with mixed conditions: {:.1}%", health_score);
        assert!(
            (50.0..=90.0).contains(&health_score),
            "Expected moderate health score, got {:.1}%",
            health_score
        );
    }

    /// Test cooling efficiency calculation
    #[test]
    fn test_cooling_efficiency() {
        let mut extended_gpu = create_test_extended_gpu();
        extended_gpu.thermal_info.gpu_temperature = Some(70.0);
        extended_gpu.fan_info.fan_speed_percent = Some(50.0);
        let efficiency = extended_gpu.cooling_efficiency();
        assert!(efficiency.is_some());
        let eff_value = efficiency.unwrap();
        println!("Cooling efficiency: {:.1}%", eff_value);
        assert!((0.0..=100.0).contains(&eff_value));
        extended_gpu.fan_info.fan_speed_percent = Some(0.0);
        let efficiency_zero = extended_gpu.cooling_efficiency();
        assert!(efficiency_zero.is_none());
        extended_gpu.thermal_info.gpu_temperature = None;
        let efficiency_missing = extended_gpu.cooling_efficiency();
        assert!(efficiency_missing.is_none());
        println!("Cooling efficiency tests passed");
    }

    /// Test thermal info functionality
    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_thermal_info() {
        let mut thermal_info = ThermalInfo::default();
        thermal_info.gpu_temperature = Some(75.5);
        thermal_info.memory_temperature = Some(68.0);
        thermal_info.vrm_temperature = Some(82.0);
        thermal_info.max_safe_temperature = Some(83.0);
        thermal_info.throttle_temperature = Some(84.0);
        thermal_info.critical_temperature = Some(95.0);
        thermal_info.is_throttling = Some(false);
        thermal_info.throttle_reason = Some(ThrottleReason::None);
        assert_eq!(thermal_info.gpu_temperature, Some(75.5));
        assert_eq!(thermal_info.throttle_reason, Some(ThrottleReason::None));
        assert_eq!(thermal_info.is_throttling, Some(false));
    }

    /// Test throttle reason enum
    #[test]
    fn test_throttle_reason() {
        let reasons = vec![
            ThrottleReason::None,
            ThrottleReason::Temperature,
            ThrottleReason::PowerLimit,
            ThrottleReason::VoltageLimit,
            ThrottleReason::Unknown,
        ];
        for reason in reasons {
            println!("Throttle reason: {:?}", reason);
            assert_eq!(reason, reason);
        }
    }

    /// Test performance state enum
    #[test]
    fn test_performance_state() {
        let states = vec![
            PerformanceState::Maximum,
            PerformanceState::High,
            PerformanceState::Medium,
            PerformanceState::PowerSaver,
            PerformanceState::Adaptive,
            PerformanceState::Unknown,
        ];
        for state in states {
            println!("Performance state: {:?}", state);
            assert_eq!(state, state);
        }
    }

    /// Test fan info functionality
    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_fan_info() {
        let individual_fan = IndividualFanInfo {
            index: 0,
            speed_rpm: Some(2500),
            speed_percent: Some(75.0),
            max_speed_rpm: Some(3500),
        };
        let mut fan_info = FanInfo::default();
        fan_info.fan_speed_rpm = Some(2500);
        fan_info.fan_speed_percent = Some(75.0);
        fan_info.fan_count = Some(2);
        fan_info.individual_fans = vec![individual_fan.clone()];
        fan_info.auto_fan_control = Some(true);
        fan_info.target_temperature = Some(70.0);
        assert_eq!(fan_info.fan_speed_rpm, Some(2500));
        assert_eq!(fan_info.individual_fans.len(), 1);
        assert_eq!(fan_info.individual_fans[0].index, 0);
        assert_eq!(fan_info.individual_fans[0].speed_rpm, Some(2500));
    }

    /// Test memory info functionality
    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_memory_info() {
        let mut memory_info = MemoryInfo::default();
        memory_info.total_memory_mb = Some(8192);
        memory_info.used_memory_mb = Some(6144);
        memory_info.free_memory_mb = Some(2048);
        memory_info.memory_bandwidth_gb_s = Some(448.0);
        memory_info.memory_bandwidth_utilization = Some(65.0);
        memory_info.memory_type = Some("GDDR6".to_string());
        memory_info.memory_bus_width = Some(256);
        memory_info.ecc_enabled = Some(false);
        memory_info.ecc_errors = Some(0);
        assert_eq!(memory_info.total_memory_mb, Some(8192));
        assert_eq!(memory_info.memory_type, Some("GDDR6".to_string()));
        assert_eq!(memory_info.ecc_enabled, Some(false));
        println!("Memory info test passed");
    }

    /// Test connection info functionality
    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_connection_info() {
        let mut connection_info = ConnectionInfo::default();
        connection_info.pcie_generation = Some(4);
        connection_info.pcie_width = Some(16);
        connection_info.pcie_throughput_gb_s = Some(31.5);
        connection_info.pcie_max_throughput_gb_s = Some(31.5);
        connection_info.pcie_utilization = Some(25.0);
        connection_info.bus_id = Some("0000:01:00.0".to_string());
        connection_info.device_id = Some("0x2204".to_string());
        connection_info.vendor_id = Some("0x10DE".to_string());
        assert_eq!(connection_info.pcie_generation, Some(4));
        assert_eq!(connection_info.pcie_width, Some(16));
        assert_eq!(connection_info.bus_id, Some("0000:01:00.0".to_string()));
    }

    /// Test encoder info functionality
    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_encoder_info() {
        let mut encoder_info = EncoderInfo::default();
        encoder_info.encoder_utilization = Some(15.0);
        encoder_info.decoder_utilization = Some(5.0);
        encoder_info.supported_codecs = vec!["H.264".to_string(), "H.265".to_string()];
        encoder_info.active_codec = Some("H.264".to_string());
        encoder_info.active_encoding_sessions = Some(2);
        encoder_info.active_decoding_sessions = Some(1);
        assert_eq!(encoder_info.encoder_utilization, Some(15.0));
        assert_eq!(encoder_info.supported_codecs.len(), 2);
        assert_eq!(encoder_info.active_codec, Some("H.264".to_string()));
    }

    /// Test overclocking info functionality
    #[test]
    #[allow(clippy::field_reassign_with_default)]
    fn test_overclocking_info() {
        let mut oc_info = OverclockingInfo::default();
        oc_info.overclocking_supported = Some(true);
        oc_info.core_clock_offset = Some(150);
        oc_info.memory_clock_offset = Some(500);
        oc_info.max_core_clock_offset = Some(300);
        oc_info.max_memory_clock_offset = Some(1000);
        oc_info.voltage_limit = Some(1100);
        oc_info.max_voltage_limit = Some(1200);
        assert_eq!(oc_info.overclocking_supported, Some(true));
        assert_eq!(oc_info.core_clock_offset, Some(150));
        assert_eq!(oc_info.max_voltage_limit, Some(1200));
    }

    /// Test Display implementation
    #[test]
    fn test_display_implementation() {
        let extended_gpu = create_test_extended_gpu();
        let display_string = format!("{}", extended_gpu);
        assert!(display_string.contains("Extended GPU Information"));
        assert!(display_string.contains("Name:"));
        assert!(display_string.contains("Vendor:"));
        assert!(display_string.contains("Health Score:"));

        println!("Display output:\n{}", display_string);
    }

    /// Test GpuInfoExtensions trait
    #[test]
    fn test_gpu_info_extensions() {
        let basic_gpu = create_test_gpu();
        let extended_gpu = basic_gpu.clone().to_extended();
        assert_eq!(extended_gpu.base_info.vendor, basic_gpu.vendor);
        assert_eq!(extended_gpu.base_info.name_gpu, basic_gpu.name_gpu);
        let mut basic_gpu_mut = basic_gpu;
        let enhance_result = basic_gpu_mut.enhance();
        assert!(enhance_result.is_ok());
        println!("GPU info extensions test passed");
    }

    /// Load test: Create many extended GPU instances
    #[test]
    fn test_extended_gpu_load_creation() {
        const INSTANCES: usize = 1000;
        let mut extended_gpus = Vec::with_capacity(INSTANCES);
        let start = std::time::Instant::now();
        for i in 0..INSTANCES {
            let mut basic_gpu = create_test_gpu();
            basic_gpu.temperature = Some(50.0 + (i as f32 % 40.0));
            let extended_gpu = ExtendedGpuInfo::from_basic(basic_gpu);
            extended_gpus.push(extended_gpu);
        }
        let creation_time = start.elapsed();
        let mut total_health_score = 0.0;
        let mut high_health_count = 0;
        for gpu in &extended_gpus {
            let health = gpu.health_score();
            total_health_score += health;
            if health > 80.0 {
                high_health_count += 1;
            }
        }
        let avg_health = total_health_score / INSTANCES as f32;
        println!("Load test results:");
        println!(
            "  Created {} extended GPU instances in {:?}",
            INSTANCES, creation_time
        );
        println!("  Average health score: {:.1}%", avg_health);
        println!("  High health count: {}", high_health_count);
        assert_eq!(extended_gpus.len(), INSTANCES);
        assert!(
            creation_time.as_millis() < 1000,
            "Creation took too long: {:?}",
            creation_time
        );
    }

    /// Stress test: Health score calculation performance
    #[test]
    fn test_health_score_performance() {
        let extended_gpu = create_test_extended_gpu();
        const CALCULATIONS: usize = 100_000;
        let start = std::time::Instant::now();
        let mut total_score = 0.0;
        for _ in 0..CALCULATIONS {
            total_score += extended_gpu.health_score();
        }
        let calculation_time = start.elapsed();
        let avg_score = total_score / CALCULATIONS as f32;
        let calculations_per_second = CALCULATIONS as f64 / calculation_time.as_secs_f64();
        println!("  {} calculations in {:?}", CALCULATIONS, calculation_time);
        println!("  Average score: {:.1}%", avg_score);
        println!("  Calculations per second: {:.0}", calculations_per_second);
        assert!(
            calculations_per_second > 10_000.0,
            "Too slow: {:.0} calc/sec",
            calculations_per_second
        );
    }

    /// Integration test: Full extended GPU workflow
    #[test]
    fn test_full_extended_gpu_workflow() {
        let basic_gpu = create_test_gpu();
        println!("Created basic GPU: {:?}", basic_gpu.vendor);
        let mut extended_gpu = ExtendedGpuInfo::from_basic(basic_gpu);
        populate_extended_gpu_info(&mut extended_gpu);
        let health_score = extended_gpu.health_score();
        let cooling_efficiency = extended_gpu.cooling_efficiency();
        let needs_attention = extended_gpu.needs_attention();
        println!("  Health score: {:.1}%", health_score);
        println!("  Cooling efficiency: {:?}", cooling_efficiency);
        println!("  Needs attention: {}", needs_attention);
        let display_output = format!("{}", extended_gpu);
        println!("Display output length: {} characters", display_output.len());
        assert!((0.0..=100.0).contains(&health_score));
        if let Some(efficiency) = cooling_efficiency {
            assert!((0.0..=100.0).contains(&efficiency));
        }
    }

    // Helper functions

    #[allow(clippy::field_reassign_with_default)]
    fn create_test_gpu() -> GpuInfo {
        let mut gpu = GpuInfo::default();
        gpu.vendor = Vendor::Nvidia;
        gpu.name_gpu = Some("Test GPU RTX 4080".to_string());
        gpu.temperature = Some(72.0);
        gpu.utilization = Some(65.0);
        gpu.power_usage = Some(220.0);
        gpu.core_clock = Some(2100);
        gpu.memory_util = Some(75.0);
        gpu.memory_clock = Some(11000);
        gpu.active = Some(true);
        gpu.power_limit = Some(320.0);
        gpu.memory_total = Some(16384);
        gpu.driver_version = Some("531.41".to_string());
        gpu.max_clock_speed = Some(2800);
        gpu
    }

    fn create_test_extended_gpu() -> ExtendedGpuInfo {
        let basic_gpu = create_test_gpu();
        ExtendedGpuInfo::from_basic(basic_gpu)
    }

    fn populate_extended_gpu_info(extended_gpu: &mut ExtendedGpuInfo) {
        extended_gpu.fan_info.fan_speed_rpm = Some(2200);
        extended_gpu.fan_info.fan_speed_percent = Some(65.0);
        extended_gpu.fan_info.fan_count = Some(3);
        extended_gpu.thermal_info.gpu_temperature = Some(72.0);
        extended_gpu.thermal_info.memory_temperature = Some(68.0);
        extended_gpu.thermal_info.max_safe_temperature = Some(83.0);
        extended_gpu.thermal_info.is_throttling = Some(false);
        extended_gpu.thermal_info.throttle_reason = Some(ThrottleReason::None);
        extended_gpu.memory_info.total_memory_mb = Some(16384);
        extended_gpu.memory_info.used_memory_mb = Some(12288);
        extended_gpu.memory_info.memory_type = Some("GDDR6X".to_string());
        extended_gpu.memory_info.ecc_enabled = Some(false);
        extended_gpu.memory_info.ecc_errors = Some(0);
        extended_gpu.connection_info.pcie_generation = Some(4);
        extended_gpu.connection_info.pcie_width = Some(16);
        extended_gpu.connection_info.bus_id = Some("0000:01:00.0".to_string());
        extended_gpu.performance_info.base_core_clock = Some(1800);
        extended_gpu.performance_info.boost_core_clock = Some(2100);
        extended_gpu.performance_info.performance_state = Some(PerformanceState::High);
    }
}
