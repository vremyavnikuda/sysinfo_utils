//! Comprehensive tests for GPU monitoring system
//!
//! These tests cover monitoring functionality including alerts, history tracking,
//! performance under load, and error handling scenarios.
//!
//! ## Performance Considerations
//!
//! GPU monitoring tests may experience slower performance in test environments due to:
//! - Absence of real GPU hardware
//! - FFI library timeouts when no hardware is present
//! - Virtualized or containerized test environments
//! - CI/CD environments with limited resources
//!
#[cfg(test)]
mod tests {
    use crate::monitoring::{
        AlertHandler, AlertType, GpuMonitor, GpuThresholds, LogAlertHandler, MonitorConfig,
    };
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;
    #[derive(Debug)]
    struct MockAlertHandler {
        alerts_received: Arc<Mutex<Vec<AlertType>>>,
    }

    impl MockAlertHandler {
        fn new() -> Self {
            Self {
                alerts_received: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_alerts(&self) -> Vec<AlertType> {
            match self.alerts_received.lock() {
                Ok(guard) => guard.clone(),
                Err(poisoned) => poisoned.into_inner().clone(),
            }
        }
    }

    impl AlertHandler for MockAlertHandler {
        fn handle_alert(&self, alert: &AlertType) -> crate::gpu_info::Result<()> {
            match self.alerts_received.lock() {
                Ok(mut guard) => guard.push(alert.clone()),
                Err(poisoned) => poisoned.into_inner().push(alert.clone()),
            }
            Ok(())
        }

        fn name(&self) -> &str {
            "MockAlertHandler"
        }
    }

    /// Test basic monitor creation and configuration
    #[test]
    fn test_monitor_creation() {
        let config = MonitorConfig::default();
        let monitor = GpuMonitor::new(config.clone());
        assert!(!monitor.is_monitoring());
        assert_eq!(monitor.get_stats().total_measurements, 0);
        println!(
            "Monitor created with config: polling_interval={:?}, history_size={}",
            config.polling_interval, config.history_size
        );
    }

    /// Test monitor with custom configuration
    #[test]
    fn test_monitor_custom_config() {
        let config = MonitorConfig {
            polling_interval: Duration::from_millis(100),
            history_size: 50,
            thresholds: GpuThresholds {
                temperature_warning: 60.0,
                temperature_critical: 80.0,
                memory_warning: 70.0,
                memory_critical: 90.0,
                power_warning: 200.0,
                power_critical: 250.0,
                utilization_warning: 90.0,
                fan_speed_min: 20.0,
            },
            enable_alerts: true,
            log_metrics: false,
            save_to_file: None,
        };
        let monitor = GpuMonitor::new(config.clone());
        assert!(!monitor.is_monitoring());
        println!(
            "Custom monitor created with fast polling: {:?}",
            config.polling_interval
        );
    }

    /// Test alert handler registration
    #[test]
    fn test_alert_handler_registration() {
        let monitor = GpuMonitor::with_defaults();
        let mock_handler = MockAlertHandler::new();
        let result = monitor.add_alert_handler(Box::new(mock_handler));
        assert!(result.is_ok());
        println!("Alert handler registered successfully");
    }

    /// Test default thresholds
    #[test]
    fn test_default_thresholds() {
        let thresholds = GpuThresholds::default();
        assert_eq!(thresholds.temperature_warning, 75.0);
        assert_eq!(thresholds.temperature_critical, 85.0);
        assert_eq!(thresholds.memory_warning, 80.0);
        assert_eq!(thresholds.memory_critical, 95.0);
        assert_eq!(thresholds.power_warning, 250.0);
        assert_eq!(thresholds.power_critical, 300.0);
        assert_eq!(thresholds.utilization_warning, 95.0);
        assert_eq!(thresholds.fan_speed_min, 10.0);
        println!("Default thresholds verified");
    }

    /// Test log alert handler
    #[test]
    fn test_log_alert_handler() {
        let handler = LogAlertHandler;
        assert_eq!(handler.name(), "LogAlertHandler");
        let test_alerts = vec![
            AlertType::HighTemperature {
                gpu_index: 0,
                temperature: 80.0,
            },
            AlertType::CriticalTemperature {
                gpu_index: 0,
                temperature: 90.0,
            },
            AlertType::HighMemoryUsage {
                gpu_index: 1,
                usage: 85.0,
            },
            AlertType::CriticalMemoryUsage {
                gpu_index: 1,
                usage: 98.0,
            },
            AlertType::HighPowerUsage {
                gpu_index: 0,
                power: 280.0,
            },
            AlertType::CriticalPowerUsage {
                gpu_index: 0,
                power: 320.0,
            },
            AlertType::HighUtilization {
                gpu_index: 1,
                utilization: 98.0,
            },
            AlertType::LowFanSpeed {
                gpu_index: 0,
                fan_speed: 5.0,
            },
            AlertType::GpuInactive { gpu_index: 2 },
            AlertType::CollectionError {
                gpu_index: 0,
                error: "Test error".to_string(),
            },
        ];
        for alert in test_alerts {
            let result = handler.handle_alert(&alert);
            assert!(result.is_ok(), "Failed to handle alert: {:?}", alert);
        }
        println!("All alert types handled successfully");
    }

    /// Test monitoring start and stop
    #[tokio::test]
    async fn test_monitoring_lifecycle() {
        let config = MonitorConfig {
            polling_interval: Duration::from_millis(50),
            ..Default::default()
        };
        let monitor = GpuMonitor::new(config);
        assert!(!monitor.is_monitoring());
        let result = monitor.start_monitoring();
        assert!(result.is_ok());
        assert!(monitor.is_monitoring());
        sleep(Duration::from_millis(100)).await;
        assert!(monitor.is_monitoring());
        sleep(Duration::from_millis(300)).await;
        let result = monitor.stop_monitoring();
        assert!(result.is_ok());
        assert!(!monitor.is_monitoring());
        sleep(Duration::from_millis(50)).await;
        let stats = monitor.get_stats();
        println!(
            "Monitoring stats: measurements={}, errors={}",
            stats.total_measurements, stats.total_errors
        );
        assert!(
            stats.total_measurements > 0 || stats.total_errors > 0,
            "Expected either measurements or errors, got measurements={}, errors={}",
            stats.total_measurements,
            stats.total_errors
        );
    }

    /// Test monitoring performance in constrained environments
    #[tokio::test]
    async fn test_monitoring_performance_constrained() {
        let config = MonitorConfig {
            polling_interval: Duration::from_millis(200),
            enable_alerts: false,
            log_metrics: false,
            ..Default::default()
        };
        let monitor = GpuMonitor::new(config);
        if let Err(e) = monitor.start_monitoring() {
            panic!("Failed to start monitoring: {:?}", e);
        }
        sleep(Duration::from_millis(500)).await;
        if let Err(e) = monitor.stop_monitoring() {
            panic!("Failed to stop monitoring: {:?}", e);
        }
        let stats = monitor.get_stats();
        println!("Constrained environment stats:");
        println!("  Total measurements: {}", stats.total_measurements);
        println!("  Total errors: {}", stats.total_errors);
        println!("  Average collection time: {:?}", stats.avg_collection_time);
        assert!(
            stats.total_measurements > 0 || stats.total_errors > 0,
            "Some monitoring activity should occur"
        );
        if stats.total_measurements > 0 {
            assert!(
                stats.avg_collection_time < Duration::from_secs(10),
                "Collection time extremely high: {:?}",
                stats.avg_collection_time
            );
        }
    }

    /// Load test: Long-running monitoring with performance optimizations
    #[tokio::test]
    async fn test_long_running_monitoring() {
        let config = MonitorConfig {
            polling_interval: Duration::from_millis(100),
            enable_alerts: false,
            log_metrics: false,
            ..Default::default()
        };
        let monitor = GpuMonitor::new(config);
        let mock_handler = MockAlertHandler::new();
        if let Err(e) = monitor.add_alert_handler(Box::new(mock_handler)) {
            panic!("Failed to add alert handler: {:?}", e);
        }
        if let Err(e) = monitor.start_monitoring() {
            panic!("Failed to start monitoring: {:?}", e);
        }
        sleep(Duration::from_secs(1)).await;
        if let Err(e) = monitor.stop_monitoring() {
            panic!("Failed to stop monitoring: {:?}", e);
        }
        let stats = monitor.get_stats();
        println!("Long-running test stats:");
        println!("  Total measurements: {}", stats.total_measurements);
        println!("  Total errors: {}", stats.total_errors);
        println!("  Average collection time: {:?}", stats.avg_collection_time);
        assert!(
            stats.total_measurements > 0 || stats.total_errors > 0,
            "No monitoring activity detected: measurements={}, errors={}",
            stats.total_measurements,
            stats.total_errors
        );
        if stats.total_measurements > 0 {
            assert!(
                stats.avg_collection_time < Duration::from_secs(5),
                "Collection time too high: {:?}",
                stats.avg_collection_time
            );
            let collection_ms = stats.avg_collection_time.as_millis();
            if collection_ms > 1000 {
                println!("Note: Collection time is high ({:.1}s) - this may indicate no real GPUs in test environment",
                        stats.avg_collection_time.as_secs_f64());
            }
        }
    }

    /// Test alert generation under simulated conditions
    #[tokio::test]
    async fn test_alert_generation() {
        let mut config = MonitorConfig {
            polling_interval: Duration::from_millis(50),
            ..Default::default()
        };
        config.thresholds.temperature_warning = 70.0;
        config.thresholds.temperature_critical = 80.0;
        config.enable_alerts = true;
        let monitor = GpuMonitor::new(config);
        let mock_handler = Arc::new(MockAlertHandler::new());
        let _handler_clone = mock_handler.clone();
        if let Err(e) = monitor.add_alert_handler(Box::new(MockAlertHandler::new())) {
            panic!("Failed to add alert handler: {:?}", e);
        }
        if let Err(e) = monitor.start_monitoring() {
            panic!("Failed to start monitoring: {:?}", e);
        }
        sleep(Duration::from_millis(300)).await;
        if let Err(e) = monitor.stop_monitoring() {
            panic!("Failed to stop monitoring: {:?}", e);
        }
        let alerts = mock_handler.get_alerts();
        println!("Generated {} alerts during test", alerts.len());
        for (i, alert) in alerts.iter().enumerate() {
            println!("Alert {}: {:?}", i, alert);
        }
    }

    /// Test monitor statistics accuracy
    #[tokio::test]
    async fn test_statistics_accuracy() {
        let config = MonitorConfig {
            polling_interval: Duration::from_millis(20),
            ..Default::default()
        };
        let monitor = GpuMonitor::new(config);
        let initial_stats = monitor.get_stats();
        assert_eq!(initial_stats.total_measurements, 0);
        assert_eq!(initial_stats.total_errors, 0);
        assert!(initial_stats.start_time.is_none());
        if let Err(e) = monitor.start_monitoring() {
            panic!("Failed to start monitoring: {:?}", e);
        }
        let run_duration = Duration::from_millis(250);
        sleep(run_duration).await;
        if let Err(e) = monitor.stop_monitoring() {
            panic!("Failed to stop monitoring: {:?}", e);
        }
        let final_stats = monitor.get_stats();
        println!("Statistics accuracy test:");
        println!("  Measurements: {}", final_stats.total_measurements);
        println!("  Errors: {}", final_stats.total_errors);
        if let Some(start_time) = final_stats.start_time {
            println!("  Runtime: {:?}", start_time.elapsed());
        }
        assert!(final_stats.start_time.is_some());
        assert!(final_stats.total_measurements > 0 || final_stats.total_errors > 0);
        let runtime = match final_stats.start_time {
            Some(start_time) => start_time.elapsed(),
            None => panic!("Expected start_time to be Some after monitoring"),
        };
        assert!(
            runtime >= run_duration * 8 / 10,
            "Runtime too short: {:?} < {:?}",
            runtime,
            run_duration * 8 / 10
        );
        assert!(
            runtime <= Duration::from_secs(4),
            "Runtime too long: {:?} > 4s",
            runtime
        );
    }

    /// Test concurrent monitoring operations
    #[tokio::test]
    async fn test_concurrent_monitoring() {
        let config = MonitorConfig {
            polling_interval: Duration::from_millis(30),
            ..Default::default()
        };
        let monitor = Arc::new(GpuMonitor::new(config));
        let mut handles = Vec::new();
        for i in 0..5 {
            let monitor_clone = monitor.clone();
            let handle = tokio::spawn(async move {
                let start_result = monitor_clone.start_monitoring();
                sleep(Duration::from_millis(50)).await;
                let stop_result = monitor_clone.stop_monitoring();
                (i, start_result, stop_result)
            });
            handles.push(handle);
        }
        let mut successful_starts = 0;
        let mut successful_stops = 0;
        for handle in handles {
            match handle.await {
                Ok((task_id, start_result, stop_result)) => {
                    if start_result.is_ok() {
                        successful_starts += 1;
                    }
                    if stop_result.is_ok() {
                        successful_stops += 1;
                    }
                    println!(
                        "Task {} completed: start={}, stop={}",
                        task_id,
                        start_result.is_ok(),
                        stop_result.is_ok()
                    );
                }
                Err(e) => {
                    println!("Task failed: {}", e);
                }
            }
        }
        println!(
            "Concurrent test results: {} starts, {} stops",
            successful_starts, successful_stops
        );
        assert!(successful_starts > 0);
        assert!(successful_stops > 0);
    }

    /// Stress test: Rapid start/stop cycles
    #[tokio::test]
    async fn test_rapid_start_stop_cycles() {
        let config = MonitorConfig {
            polling_interval: Duration::from_millis(10),
            ..Default::default()
        };
        let monitor = GpuMonitor::new(config);
        let cycles = 20;
        let mut successful_cycles = 0;
        for i in 0..cycles {
            if monitor.start_monitoring().is_ok() {
                sleep(Duration::from_millis(5)).await;
                if monitor.stop_monitoring().is_ok() {
                    successful_cycles += 1;
                }
            }
            if i % 5 == 0 {
                println!("Completed {} start/stop cycles", i + 1);
            }
        }
        println!(
            "Rapid cycles test: {}/{} successful",
            successful_cycles, cycles
        );
        assert!(
            successful_cycles > cycles / 2,
            "Too many cycle failures: {}/{}",
            successful_cycles,
            cycles
        );
        assert!(!monitor.is_monitoring());
    }

    /// Test GPU history functionality
    #[test]
    fn test_gpu_history() {
        let monitor = GpuMonitor::with_defaults();
        let history_0 = monitor.get_gpu_history(0);
        let history_999 = monitor.get_gpu_history(999);
        println!("History for GPU 0: {:?}", history_0.is_some());
        println!("History for GPU 999: {:?}", history_999.is_some());
        assert!(history_999.is_none());
    }

    /// Integration test: Full monitoring workflow
    #[tokio::test]
    async fn test_full_monitoring_workflow() {
        println!("Starting full monitoring workflow test");
        let config = MonitorConfig {
            polling_interval: Duration::from_millis(25),
            enable_alerts: true,
            log_metrics: false,
            ..Default::default()
        };
        let monitor = GpuMonitor::new(config);
        let mock_handler = MockAlertHandler::new();
        let _handler_ref = Arc::new(Mutex::new(mock_handler));
        if let Err(e) = monitor.add_alert_handler(Box::new(LogAlertHandler)) {
            panic!("Failed to add alert handler: {:?}", e);
        }
        assert!(monitor.start_monitoring().is_ok());
        assert!(monitor.is_monitoring());
        sleep(Duration::from_millis(200)).await;
        let intermediate_stats = monitor.get_stats();
        println!(
            "Intermediate stats: measurements={}, errors={}",
            intermediate_stats.total_measurements, intermediate_stats.total_errors
        );
        sleep(Duration::from_millis(200)).await;
        assert!(monitor.stop_monitoring().is_ok());
        assert!(!monitor.is_monitoring());
        let final_stats = monitor.get_stats();
        println!("Final workflow stats:");
        println!("  Total measurements: {}", final_stats.total_measurements);
        println!("  Total alerts: {}", final_stats.total_alerts);
        println!("  Total errors: {}", final_stats.total_errors);
        println!(
            "  Average collection time: {:?}",
            final_stats.avg_collection_time
        );
        assert!(final_stats.total_measurements > 0 || final_stats.total_errors > 0);
        assert!(final_stats.start_time.is_some());
        println!("Full monitoring workflow test completed successfully");
    }
}
