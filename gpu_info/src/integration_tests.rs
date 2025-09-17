/// Интеграционные тесты для GPU библиотеки с синтетическими данными
/// 
/// Эти тесты проверяют работу библиотеки с реалистичными тестовыми данными
/// для различных типов GPU, симулируя реальное поведение системы.

#[test]
fn test_gpu_detection_with_synthetic_data() {
    let provider = crate::test_provider::create_test_provider();
    
    match provider.detect_test_gpus() {
        Ok(gpus) => {
            assert!(!gpus.is_empty(), "Should detect at least one test GPU");
            let nvidia_count = gpus.iter().filter(|g| matches!(g.vendor, crate::vendor::Vendor::Nvidia)).count();
            let amd_count = gpus.iter().filter(|g| matches!(g.vendor, crate::vendor::Vendor::Amd)).count();
            let intel_count = gpus.iter().filter(|g| matches!(g.vendor, crate::vendor::Vendor::Intel(_))).count();
            println!("Detected GPUs: {} NVIDIA, {} AMD, {} Intel", nvidia_count, amd_count, intel_count);
            for (i, gpu) in gpus.iter().enumerate() {
                println!("GPU #{}: {} - {}", i, gpu.vendor, gpu.format_name_gpu());
                assert!(gpu.name_gpu.is_some());
                assert_ne!(gpu.vendor, crate::vendor::Vendor::Unknown);
                if let Some(temp) = gpu.temperature {
                    assert!(temp > 0.0 && temp < 100.0, "Temperature should be realistic: {}°C", temp);
                }
                if let Some(util) = gpu.utilization {
                    assert!(util >= 0.0 && util <= 100.0, "Utilization should be 0-100%: {}%", util);
                }
            }
        }
        Err(e) => {
            println!("Test data not available: {}", e);
        }
    }
}

/// Тест GPU Manager с синтетическими данными
#[test] 
fn test_gpu_manager_with_test_data() {
    use crate::test_provider::mock::MockGpuProvider;
    let mock_gpus = vec![
        MockGpuProvider::create_mock_nvidia(),
        MockGpuProvider::create_mock_amd(),
        MockGpuProvider::create_mock_intel(),
    ];
    let mock_provider = MockGpuProvider::new(mock_gpus);
    let gpus = mock_provider.get_gpus();
    assert_eq!(gpus.len(), 3);
    assert!(matches!(gpus[0].vendor, crate::vendor::Vendor::Nvidia));
    assert!(matches!(gpus[1].vendor, crate::vendor::Vendor::Amd));
    assert!(matches!(gpus[2].vendor, crate::vendor::Vendor::Intel(_)));
    for gpu in &gpus {
        assert!(gpu.temperature.is_some());
        assert!(gpu.utilization.is_some());
        assert!(gpu.name_gpu.is_some());
        println!("Mock GPU: {} - {}°C, {}% util", 
                gpu.format_name_gpu(),
                gpu.format_temperature(),
                gpu.format_utilization());
    }
}

/// Тест мониторинга с симуляцией
#[test]
fn test_gpu_monitoring_simulation() {
    use crate::test_provider::mock::MockGpuProvider;
    use std::thread;
    use std::time::Duration;
    let mut gpu = MockGpuProvider::create_mock_nvidia();
    let provider = crate::test_provider::create_test_provider();
    let initial_temp = gpu.temperature.unwrap();
    for i in 0..5 {
        provider.simulate_gpu_update(&mut gpu).unwrap();
        let current_temp = gpu.temperature.unwrap();
        println!("Update #{}: Temperature {}°C -> {}°C", 
                i + 1, initial_temp, current_temp);
        
        assert!(current_temp >= 40.0 && current_temp <= 90.0);
        thread::sleep(Duration::from_millis(10));
    }
}

/// Тест валидации GPU данных
#[test]
fn test_gpu_data_validation() {
    use crate::test_provider::mock::MockGpuProvider;
    let gpu = MockGpuProvider::create_mock_nvidia();
    assert!(gpu.is_valid(), "Mock NVIDIA GPU should have valid data");
    assert!(gpu.temperature.unwrap() > 0.0);
    assert!(gpu.utilization.unwrap() >= 0.0 && gpu.utilization.unwrap() <= 100.0);
    assert!(gpu.power_usage.unwrap() > 0.0);
    assert!(gpu.core_clock.unwrap() > 0);
    assert!(gpu.memory_total.unwrap() > 0);
}

/// Тест форматирования выводов
#[test]
fn test_gpu_formatting() {
    use crate::test_provider::mock::MockGpuProvider;
    let gpu = MockGpuProvider::create_mock_amd();
    assert!(!gpu.format_name_gpu().is_empty());
    assert!(gpu.format_temperature() >= 0.0);
    assert!(gpu.format_utilization() >= 0.0);
    assert!(gpu.format_power_usage() >= 0.0);
    assert!(gpu.format_core_clock() > 0);
    assert!(gpu.format_memory_total() > 0);
    assert!(!gpu.format_driver_version().is_empty());
    let display_output = format!("{}", gpu);
    assert!(!display_output.is_empty());
    println!("AMD GPU Display: {}", display_output);
}

/// Бенчмарк тест для производительности
#[test]
fn test_gpu_performance_benchmark() {
    use crate::test_provider::mock::MockGpuProvider;
    use std::time::Instant;
    let mut gpu = MockGpuProvider::create_mock_nvidia();
    let provider = crate::test_provider::create_test_provider();
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        provider.simulate_gpu_update(&mut gpu).unwrap();
    }
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() / iterations as u128;
    println!("Performance: {} iterations in {:?} (avg: {}ns per update)", 
             iterations, duration, avg_time);
    
    assert!(avg_time < 1_000_000, "GPU updates should be fast ({}ns)", avg_time);
}

/// Интеграционный тест со всеми компонентами
#[test]
fn test_full_integration() {
    use crate::test_provider::mock::MockGpuProvider;
    let test_gpus = vec![
        MockGpuProvider::create_mock_nvidia(),
        MockGpuProvider::create_mock_amd(),
        MockGpuProvider::create_mock_intel(),
    ];
    for (i, gpu) in test_gpus.iter().enumerate() {
        println!("\nTesting GPU #{}: {}", i, gpu.format_name_gpu());
        println!("Vendor: {}", gpu.vendor);
        println!("Temperature: {}°C", gpu.format_temperature());
        println!("Utilization: {}%", gpu.format_utilization());
        println!("Power: {}W", gpu.format_power_usage());
        println!("Memory: {}/{} GB ({}%)", 
                gpu.memory_total.unwrap_or(0), 
                gpu.memory_total.unwrap_or(0),
                gpu.format_memory_util());
        
        assert!(gpu.active.unwrap_or(false), "GPU should be active");
        assert!(gpu.is_valid(), "All GPU metrics should be valid");
    }
    println!("\n All integration tests passed!");
}