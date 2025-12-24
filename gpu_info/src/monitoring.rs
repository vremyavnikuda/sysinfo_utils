use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::gpu_manager::GpuManager;
use log::{debug, error, info, warn};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
/// GPU monitoring system with alert and history support
#[derive(Debug)]
pub struct GpuMonitor {
    /// GPU manager
    gpu_manager: Arc<Mutex<GpuManager>>,

    /// Monitoring configuration
    config: MonitorConfig,

    /// Metrics history
    history: Arc<Mutex<GpuHistory>>,

    /// Alert handlers
    alert_handlers: Arc<Mutex<Vec<Box<dyn AlertHandler + Send + Sync>>>>,

    /// Monitoring state
    is_running: Arc<Mutex<bool>>,

    /// Monitoring statistics
    stats: Arc<Mutex<MonitorStats>>,

    /// Monitoring thread handle
    thread_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}
/// Monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Polling interval
    pub polling_interval: Duration,

    /// History size (number of entries)
    pub history_size: usize,

    /// Threshold values for alerts
    pub thresholds: GpuThresholds,

    /// Enable automatic alerts
    pub enable_alerts: bool,

    /// Log metrics
    pub log_metrics: bool,

    /// Save metrics to file
    pub save_to_file: Option<String>,
}
/// Threshold values for alerts
#[derive(Debug, Clone)]
pub struct GpuThresholds {
    /// Temperature warning threshold (°C)
    pub temperature_warning: f32,

    /// Critical temperature threshold (°C)
    pub temperature_critical: f32,

    /// Memory usage warning threshold (%)
    pub memory_warning: f32,

    /// Critical memory usage threshold (%)
    pub memory_critical: f32,

    /// Power usage warning threshold (W)
    pub power_warning: f32,

    /// Critical power usage threshold (W)
    pub power_critical: f32,

    /// GPU utilization warning threshold (%)
    pub utilization_warning: f32,

    /// Minimum fan speed for warning (%)
    pub fan_speed_min: f32,
}
/// GPU metrics history
#[derive(Debug)]
pub struct GpuHistory {
    /// History entries for each GPU
    pub gpu_histories: Vec<SingleGpuHistory>,

    /// Maximum history size
    pub max_size: usize,
}
/// Metrics history for a single GPU
#[derive(Debug)]
pub struct SingleGpuHistory {
    /// Timestamps
    pub timestamps: VecDeque<Instant>,

    /// Temperatures
    pub temperatures: VecDeque<f32>,

    /// GPU utilization
    pub utilizations: VecDeque<f32>,

    /// Power usage
    pub power_usage: VecDeque<f32>,

    /// Memory utilization
    pub memory_utilizations: VecDeque<f32>,

    /// Core clock frequencies
    pub core_clocks: VecDeque<u32>,

    /// Fan speeds
    pub fan_speeds: VecDeque<f32>,
}
/// Monitoring statistics
#[derive(Debug, Default, Clone)]
pub struct MonitorStats {
    /// Monitoring start time
    pub start_time: Option<Instant>,

    /// Total number of collected metrics
    pub total_measurements: u64,

    /// Number of alerts
    pub total_alerts: u64,

    /// Number of errors
    pub total_errors: u64,

    /// Average collection time
    pub avg_collection_time: Duration,

    /// Last collection time
    pub last_collection_time: Option<Instant>,
}
/// Alert types for GPU monitoring.
///
/// This enum is marked `#[non_exhaustive]` to allow adding new alert types
/// in future versions without breaking existing code.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AlertType {
    /// High temperature alert - triggered when GPU temperature exceeds warning threshold.
    HighTemperature {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Current temperature in degrees Celsius.
        temperature: f32,
    },

    /// Critical temperature alert - triggered when GPU temperature exceeds critical threshold.
    CriticalTemperature {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Current temperature in degrees Celsius.
        temperature: f32,
    },

    /// High memory usage alert - triggered when GPU memory usage exceeds warning threshold.
    HighMemoryUsage {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Current memory usage as a percentage (0-100).
        usage: f32,
    },

    /// Critical memory usage alert - triggered when GPU memory usage exceeds critical threshold.
    CriticalMemoryUsage {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Current memory usage as a percentage (0-100).
        usage: f32,
    },

    /// High power usage alert - triggered when GPU power consumption exceeds warning threshold.
    HighPowerUsage {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Current power consumption in watts.
        power: f32,
    },

    /// Critical power usage alert - triggered when GPU power consumption exceeds critical threshold.
    CriticalPowerUsage {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Current power consumption in watts.
        power: f32,
    },

    /// High GPU utilization alert - triggered when GPU utilization exceeds warning threshold.
    HighUtilization {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Current GPU utilization as a percentage (0-100).
        utilization: f32,
    },

    /// Low fan speed alert - triggered when GPU fan speed falls below minimum threshold.
    LowFanSpeed {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Current fan speed as a percentage (0-100).
        fan_speed: f32,
    },

    /// GPU inactive alert - triggered when a GPU becomes inactive or unresponsive.
    GpuInactive {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
    },

    /// Data collection error alert - triggered when GPU metrics cannot be collected.
    CollectionError {
        /// Index of the GPU that triggered the alert.
        gpu_index: usize,
        /// Error message describing the collection failure.
        error: String,
    },
}
/// Trait for handling alerts
pub trait AlertHandler: std::fmt::Debug {
    /// Handle an alert
    fn handle_alert(&self, alert: &AlertType) -> Result<()>;
    /// Get handler name
    fn name(&self) -> &str;
}
/// Simple alert logger
#[derive(Debug)]
pub struct LogAlertHandler;
impl AlertHandler for LogAlertHandler {
    fn handle_alert(&self, alert: &AlertType) -> Result<()> {
        match alert {
            AlertType::HighTemperature {
                gpu_index,
                temperature,
            } => {
                warn!("GPU #{} high temperature: {:.1}°C", gpu_index, temperature);
            }
            AlertType::CriticalTemperature {
                gpu_index,
                temperature,
            } => {
                error!(
                    "GPU #{} CRITICAL temperature: {:.1}°C",
                    gpu_index, temperature
                );
            }
            AlertType::HighMemoryUsage { gpu_index, usage } => {
                warn!("GPU #{} high memory usage: {:.1}%", gpu_index, usage);
            }
            AlertType::CriticalMemoryUsage { gpu_index, usage } => {
                error!("GPU #{} CRITICAL memory usage: {:.1}%", gpu_index, usage);
            }
            AlertType::HighPowerUsage { gpu_index, power } => {
                warn!("GPU #{} high power usage: {:.1}W", gpu_index, power);
            }
            AlertType::CriticalPowerUsage { gpu_index, power } => {
                error!("GPU #{} CRITICAL power usage: {:.1}W", gpu_index, power);
            }
            AlertType::HighUtilization {
                gpu_index,
                utilization,
            } => {
                info!("GPU #{} high utilization: {:.1}%", gpu_index, utilization);
            }
            AlertType::LowFanSpeed {
                gpu_index,
                fan_speed,
            } => {
                warn!("GPU #{} low fan speed: {:.1}%", gpu_index, fan_speed);
            }
            AlertType::GpuInactive { gpu_index } => {
                error!("GPU #{} became inactive", gpu_index);
            }
            AlertType::CollectionError { gpu_index, error } => {
                error!("GPU #{} collection error: {}", gpu_index, error);
            }
        }
        Ok(())
    }
    fn name(&self) -> &str {
        "LogAlertHandler"
    }
}
impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            polling_interval: Duration::from_secs(1),
            history_size: 300,
            thresholds: GpuThresholds::default(),
            enable_alerts: true,
            log_metrics: false,
            save_to_file: None,
        }
    }
}

impl MonitorConfig {
    /// Creates a new `MonitorConfig` with default values.
    ///
    /// # Default Values
    ///
    /// - `polling_interval`: 1 second
    /// - `history_size`: 300 entries
    /// - `thresholds`: Default thresholds
    /// - `enable_alerts`: true
    /// - `log_metrics`: false
    /// - `save_to_file`: None
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the polling interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - The interval between metric collections.
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use gpu_info::MonitorConfig;
    /// use std::time::Duration;
    ///
    /// let config = MonitorConfig::new()
    ///     .with_polling_interval(Duration::from_millis(500));
    /// ```
    pub fn with_polling_interval(mut self, interval: Duration) -> Self {
        self.polling_interval = interval;
        self
    }

    /// Sets the history size.
    ///
    /// # Arguments
    ///
    /// * `size` - The maximum number of history entries to keep.
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    pub fn with_history_size(mut self, size: usize) -> Self {
        self.history_size = size;
        self
    }

    /// Sets the alert thresholds.
    ///
    /// # Arguments
    ///
    /// * `thresholds` - The threshold values for alerts.
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    pub fn with_thresholds(mut self, thresholds: GpuThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Enables or disables automatic alerts.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable alerts.
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    pub fn with_alerts_enabled(mut self, enabled: bool) -> Self {
        self.enable_alerts = enabled;
        self
    }

    /// Enables or disables metric logging.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to log metrics.
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    pub fn with_log_metrics(mut self, enabled: bool) -> Self {
        self.log_metrics = enabled;
        self
    }

    /// Sets the file path for saving metrics.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path, or `None` to disable file saving.
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining.
    pub fn with_save_to_file(mut self, path: Option<String>) -> Self {
        self.save_to_file = path;
        self
    }

    // BORROWING CHAIN PATTERN: &mut self -> &mut Self
    // Use for in-place modification of existing config

    /// Sets the polling interval (borrowing pattern).
    ///
    /// # Arguments
    ///
    /// * `interval` - The interval between metric collections.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    ///
    /// # Example
    ///
    /// ```
    /// use gpu_info::MonitorConfig;
    /// use std::time::Duration;
    ///
    /// let mut config = MonitorConfig::default();
    /// config
    ///     .polling_interval(Duration::from_millis(500))
    ///     .history_size(100);
    /// ```
    pub fn polling_interval(&mut self, interval: Duration) -> &mut Self {
        self.polling_interval = interval;
        self
    }

    /// Sets the history size (borrowing pattern).
    ///
    /// # Arguments
    ///
    /// * `size` - The maximum number of history entries to keep.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn history_size(&mut self, size: usize) -> &mut Self {
        self.history_size = size;
        self
    }

    /// Sets the alert thresholds (borrowing pattern).
    ///
    /// # Arguments
    ///
    /// * `thresholds` - The threshold values for alerts.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn thresholds(&mut self, thresholds: GpuThresholds) -> &mut Self {
        self.thresholds = thresholds;
        self
    }

    /// Enables or disables automatic alerts (borrowing pattern).
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable alerts.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn alerts_enabled(&mut self, enabled: bool) -> &mut Self {
        self.enable_alerts = enabled;
        self
    }

    /// Enables or disables metric logging (borrowing pattern).
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to log metrics.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn log_metrics(&mut self, enabled: bool) -> &mut Self {
        self.log_metrics = enabled;
        self
    }

    /// Sets the file path for saving metrics (borrowing pattern).
    ///
    /// # Arguments
    ///
    /// * `path` - The file path, or `None` to disable file saving.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining.
    pub fn save_to_file(&mut self, path: Option<String>) -> &mut Self {
        self.save_to_file = path;
        self
    }
}
impl Default for GpuThresholds {
    fn default() -> Self {
        Self {
            temperature_warning: 75.0,
            temperature_critical: 85.0,
            memory_warning: 80.0,
            memory_critical: 95.0,
            power_warning: 250.0,
            power_critical: 300.0,
            utilization_warning: 95.0,
            fan_speed_min: 10.0,
        }
    }
}
impl GpuMonitor {
    /// Creates a new GPU monitor
    pub fn new(config: MonitorConfig) -> Self {
        let gpu_manager = Arc::new(Mutex::new(GpuManager::new()));
        let gpu_count = if let Ok(mgr) = gpu_manager.lock() {
            mgr.gpu_count()
        } else {
            0
        };
        let history = Arc::new(Mutex::new(GpuHistory::new(gpu_count, config.history_size)));
        Self {
            gpu_manager,
            config,
            history,
            alert_handlers: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(Mutex::new(false)),
            stats: Arc::new(Mutex::new(MonitorStats::default())),
            thread_handle: Arc::new(Mutex::new(None)),
        }
    }
    /// Creates a monitor with default settings
    pub fn with_defaults() -> Self {
        Self::new(MonitorConfig::default())
    }
    /// Adds an alert handler
    pub fn add_alert_handler(&self, handler: Box<dyn AlertHandler + Send + Sync>) -> Result<()> {
        if let Ok(mut handlers) = self.alert_handlers.lock() {
            info!("Added alert handler: {}", handler.name());
            handlers.push(handler);
            Ok(())
        } else {
            Err(GpuError::GpuNotActive)
        }
    }
    /// Starts monitoring in a separate thread
    pub fn start_monitoring(&self) -> Result<()> {
        // Check if already running
        if let Ok(mut is_running) = self.is_running.lock() {
            if *is_running {
                warn!("Monitoring is already running");
                return Ok(());
            }
            *is_running = true;
        }

        // Ensure we have at least one alert handler
        if let Ok(handlers) = self.alert_handlers.lock() {
            if handlers.is_empty() {
                drop(handlers);
                self.add_alert_handler(Box::new(LogAlertHandler))?;
            }
        }

        info!(
            "Starting GPU monitoring with interval: {:?}",
            self.config.polling_interval
        );

        // Initialize stats
        if let Ok(mut stats) = self.stats.lock() {
            stats.start_time = Some(Instant::now());
        }

        // Clone all necessary data for the thread
        let gpu_manager = Arc::clone(&self.gpu_manager);
        let history = Arc::clone(&self.history);
        let alert_handlers = Arc::clone(&self.alert_handlers);
        let is_running = Arc::clone(&self.is_running);
        let stats = Arc::clone(&self.stats);
        let config = self.config.clone();

        // Start the monitoring thread
        let handle = thread::spawn(move || {
            Self::monitoring_loop(
                gpu_manager,
                history,
                alert_handlers,
                is_running,
                stats,
                config,
            );
        });

        // Store the thread handle
        if let Ok(mut thread_handle) = self.thread_handle.lock() {
            *thread_handle = Some(handle);
        }

        // Give the thread a moment to start
        thread::sleep(Duration::from_millis(10));

        Ok(())
    }
    /// Stops monitoring
    pub fn stop_monitoring(&self) -> Result<()> {
        // Set the running flag to false
        if let Ok(mut is_running) = self.is_running.lock() {
            if !*is_running {
                warn!("Monitoring is not running");
                return Ok(());
            }
            *is_running = false;
            info!("Stopping GPU monitoring");
        }

        // Wait for the thread to finish
        if let Ok(mut thread_handle) = self.thread_handle.lock() {
            if let Some(handle) = thread_handle.take() {
                // Give the thread time to notice the stop signal
                thread::sleep(Duration::from_millis(50));

                // Try to join the thread with a timeout
                match handle.join() {
                    Ok(()) => {
                        debug!("Monitoring thread joined successfully");
                    }
                    Err(_) => {
                        warn!("Failed to join monitoring thread cleanly");
                    }
                }
            }
        }

        Ok(())
    }
    /// Checks if monitoring is running
    pub fn is_monitoring(&self) -> bool {
        self.is_running.lock().map(|r| *r).unwrap_or(false)
    }
    /// Returns monitoring statistics
    pub fn get_stats(&self) -> MonitorStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }
    /// Returns history for a specific GPU
    pub fn get_gpu_history(&self, gpu_index: usize) -> Option<SingleGpuHistory> {
        if let Ok(history) = self.history.lock() {
            history.gpu_histories.get(gpu_index).cloned()
        } else {
            None
        }
    }
    /// Main monitoring loop
    fn monitoring_loop(
        gpu_manager: Arc<Mutex<GpuManager>>,
        history: Arc<Mutex<GpuHistory>>,
        alert_handlers: Arc<Mutex<Vec<Box<dyn AlertHandler + Send + Sync>>>>,
        is_running: Arc<Mutex<bool>>,
        stats: Arc<Mutex<MonitorStats>>,
        config: MonitorConfig,
    ) {
        info!(
            "GPU monitoring loop started with interval: {:?}",
            config.polling_interval
        );
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 10;
        let mut iteration_count = 0;

        while Self::should_continue_monitoring(&is_running) {
            iteration_count += 1;
            debug!("Monitoring iteration #{}", iteration_count);

            let collection_start = Instant::now();
            let collection_result = if let Ok(mut manager) = gpu_manager.lock() {
                let refresh_result = manager.refresh_all_gpus();
                if refresh_result.is_err() && manager.gpu_count() == 0 {
                    debug!("No GPUs found, attempting detection...");
                }
                refresh_result
            } else {
                Err(GpuError::GpuNotActive)
            };

            match collection_result {
                Ok(()) => {
                    consecutive_errors = 0;
                    if let Ok(manager) = gpu_manager.lock() {
                        let gpus = manager.get_all_gpus();
                        debug!("Successfully collected data for {} GPUs", gpus.len());
                        Self::update_history(&history, gpus, collection_start);
                        if config.enable_alerts {
                            Self::check_alerts(gpus, &config.thresholds, &alert_handlers);
                        }
                        if config.log_metrics {
                            Self::log_metrics(gpus);
                        }
                        Self::update_stats(&stats, collection_start);
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    debug!(
                        "GPU data collection failed (attempt {}): {}",
                        consecutive_errors, e
                    );
                    if let Ok(mut s) = stats.lock() {
                        s.total_errors += 1;
                    }
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        warn!(
                            "Too many consecutive errors ({}), taking a longer break",
                            consecutive_errors
                        );
                        thread::sleep(Duration::from_secs(1));
                        consecutive_errors = 0;
                    }
                }
            }
            thread::sleep(config.polling_interval);
        }
        info!(
            "GPU monitoring loop ended after {} iterations",
            iteration_count
        );
    }
    /// Checks if monitoring should continue
    fn should_continue_monitoring(is_running: &Arc<Mutex<bool>>) -> bool {
        is_running.lock().map(|r| *r).unwrap_or(false)
    }
    /// Updates metrics history
    fn update_history(history: &Arc<Mutex<GpuHistory>>, gpus: &[GpuInfo], timestamp: Instant) {
        if let Ok(mut hist) = history.lock() {
            for (gpu_index, gpu) in gpus.iter().enumerate() {
                if let Some(gpu_history) = hist.gpu_histories.get_mut(gpu_index) {
                    gpu_history.add_measurement(gpu, timestamp);
                }
            }
        }
    }
    /// Checks alerts
    fn check_alerts(
        gpus: &[GpuInfo],
        thresholds: &GpuThresholds,
        alert_handlers: &Arc<Mutex<Vec<Box<dyn AlertHandler + Send + Sync>>>>,
    ) {
        let mut alerts = Vec::new();
        for (gpu_index, gpu) in gpus.iter().enumerate() {
            if let Some(temp) = gpu.temperature {
                if temp >= thresholds.temperature_critical {
                    alerts.push(AlertType::CriticalTemperature {
                        gpu_index,
                        temperature: temp,
                    });
                } else if temp >= thresholds.temperature_warning {
                    alerts.push(AlertType::HighTemperature {
                        gpu_index,
                        temperature: temp,
                    });
                }
            }
            if let Some(mem_util) = gpu.memory_util {
                if mem_util >= thresholds.memory_critical {
                    alerts.push(AlertType::CriticalMemoryUsage {
                        gpu_index,
                        usage: mem_util,
                    });
                } else if mem_util >= thresholds.memory_warning {
                    alerts.push(AlertType::HighMemoryUsage {
                        gpu_index,
                        usage: mem_util,
                    });
                }
            }
            if let Some(power) = gpu.power_usage {
                if power >= thresholds.power_critical {
                    alerts.push(AlertType::CriticalPowerUsage { gpu_index, power });
                } else if power >= thresholds.power_warning {
                    alerts.push(AlertType::HighPowerUsage { gpu_index, power });
                }
            }
            if let Some(util) = gpu.utilization {
                if util >= thresholds.utilization_warning {
                    alerts.push(AlertType::HighUtilization {
                        gpu_index,
                        utilization: util,
                    });
                }
            }
            if gpu.active == Some(false) {
                alerts.push(AlertType::GpuInactive { gpu_index });
            }
        }
        if !alerts.is_empty() {
            if let Ok(handlers) = alert_handlers.lock() {
                for alert in &alerts {
                    for handler in handlers.iter() {
                        if let Err(e) = handler.handle_alert(alert) {
                            error!("Alert handler '{}' failed: {}", handler.name(), e);
                        }
                    }
                }
            }
        }
    }
    /// Logs metrics
    fn log_metrics(gpus: &[GpuInfo]) {
        for (index, gpu) in gpus.iter().enumerate() {
            debug!(
                "GPU #{}: {}°C, {:.1}% util, {:.1}W, {:.1}% mem",
                index,
                gpu.temperature
                    .map(|t| format!("{:.1}", t))
                    .unwrap_or_else(|| "N/A".to_string()),
                gpu.utilization.unwrap_or(0.0),
                gpu.power_usage.unwrap_or(0.0),
                gpu.memory_util.unwrap_or(0.0)
            );
        }
    }
    /// Updates statistics
    fn update_stats(stats: &Arc<Mutex<MonitorStats>>, collection_start: Instant) {
        if let Ok(mut s) = stats.lock() {
            s.total_measurements += 1;
            s.last_collection_time = Some(collection_start);
            let collection_duration = collection_start.elapsed();
            if s.total_measurements == 1 {
                s.avg_collection_time = collection_duration;
            } else {
                let alpha = 0.1;
                let new_avg_nanos = s.avg_collection_time.as_nanos() as f64 * (1.0 - alpha)
                    + collection_duration.as_nanos() as f64 * alpha;
                s.avg_collection_time = Duration::from_nanos(new_avg_nanos as u64);
            }
        }
    }
}
impl GpuHistory {
    /// Creates a new history for the specified number of GPUs
    pub fn new(gpu_count: usize, max_size: usize) -> Self {
        let gpu_histories = (0..gpu_count)
            .map(|_| SingleGpuHistory::new(max_size))
            .collect();
        Self {
            gpu_histories,
            max_size,
        }
    }
}
impl SingleGpuHistory {
    /// Creates a new history for a single GPU
    pub fn new(max_size: usize) -> Self {
        Self {
            timestamps: VecDeque::with_capacity(max_size),
            temperatures: VecDeque::with_capacity(max_size),
            utilizations: VecDeque::with_capacity(max_size),
            power_usage: VecDeque::with_capacity(max_size),
            memory_utilizations: VecDeque::with_capacity(max_size),
            core_clocks: VecDeque::with_capacity(max_size),
            fan_speeds: VecDeque::with_capacity(max_size),
        }
    }
    /// Adds a new measurement
    pub fn add_measurement(&mut self, gpu: &GpuInfo, timestamp: Instant) {
        self.timestamps.push_back(timestamp);
        self.temperatures.push_back(gpu.temperature.unwrap_or(0.0));
        self.utilizations.push_back(gpu.utilization.unwrap_or(0.0));
        self.power_usage.push_back(gpu.power_usage.unwrap_or(0.0));
        self.memory_utilizations
            .push_back(gpu.memory_util.unwrap_or(0.0));
        self.core_clocks.push_back(gpu.core_clock.unwrap_or(0));
        self.fan_speeds.push_back(0.0);
        let max_size = self.timestamps.capacity();
        while self.timestamps.len() > max_size {
            self.timestamps.pop_front();
            self.temperatures.pop_front();
            self.utilizations.pop_front();
            self.power_usage.pop_front();
            self.memory_utilizations.pop_front();
            self.core_clocks.pop_front();
            self.fan_speeds.pop_front();
        }
    }
    /// Returns average temperature for the specified period
    pub fn avg_temperature(&self, duration: Duration) -> Option<f32> {
        let cutoff = Instant::now() - duration;
        let values: Vec<f32> = self
            .timestamps
            .iter()
            .zip(self.temperatures.iter())
            .filter(|(timestamp, _)| **timestamp >= cutoff)
            .map(|(_, temp)| *temp)
            .collect();
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f32>() / values.len() as f32)
        }
    }
    /// Returns maximum temperature for the specified period
    pub fn max_temperature(&self, duration: Duration) -> Option<f32> {
        let cutoff = Instant::now() - duration;
        self.timestamps
            .iter()
            .zip(self.temperatures.iter())
            .filter(|(timestamp, _)| **timestamp >= cutoff)
            .map(|(_, temp)| *temp)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }
}
impl Clone for SingleGpuHistory {
    fn clone(&self) -> Self {
        Self {
            timestamps: self.timestamps.clone(),
            temperatures: self.temperatures.clone(),
            utilizations: self.utilizations.clone(),
            power_usage: self.power_usage.clone(),
            memory_utilizations: self.memory_utilizations.clone(),
            core_clocks: self.core_clocks.clone(),
            fan_speeds: self.fan_speeds.clone(),
        }
    }
}
