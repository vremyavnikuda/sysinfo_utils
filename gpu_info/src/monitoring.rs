use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::gpu_manager::GpuManager;
use log::{debug, error, info, warn};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
/// Система мониторинга GPU с поддержкой алертов и истории
#[derive(Debug)]
pub struct GpuMonitor {
    /// Менеджер GPU
    gpu_manager: Arc<Mutex<GpuManager>>,

    /// Настройки мониторинга
    config: MonitorConfig,

    /// История метрик
    history: Arc<Mutex<GpuHistory>>,

    /// Обработчики алертов
    alert_handlers: Arc<Mutex<Vec<Box<dyn AlertHandler + Send + Sync>>>>,

    /// Состояние мониторинга
    is_running: Arc<Mutex<bool>>,

    /// Статистика мониторинга
    stats: Arc<Mutex<MonitorStats>>,
}
/// Конфигурация мониторинга
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Интервал опроса
    pub polling_interval: Duration,

    /// Размер истории (количество записей)
    pub history_size: usize,

    /// Пороговые значения для алертов
    pub thresholds: GpuThresholds,

    /// Включить автоматические алерты
    pub enable_alerts: bool,

    /// Логировать метрики
    pub log_metrics: bool,

    /// Сохранять метрики в файл
    pub save_to_file: Option<String>,
}
/// Пороговые значения для алертов
#[derive(Debug, Clone)]
pub struct GpuThresholds {
    /// Предупреждение о температуре (°C)
    pub temperature_warning: f32,

    /// Критическая температура (°C)
    pub temperature_critical: f32,

    /// Предупреждение об использовании памяти (%)
    pub memory_warning: f32,

    /// Критическое использование памяти (%)
    pub memory_critical: f32,

    /// Предупреждение о потреблении энергии (W)
    pub power_warning: f32,

    /// Критическое потребление энергии (W)
    pub power_critical: f32,

    /// Предупреждение о загрузке GPU (%)
    pub utilization_warning: f32,

    /// Минимальная скорость вентилятора для предупреждения (%)
    pub fan_speed_min: f32,
}
/// История метрик GPU
#[derive(Debug)]
pub struct GpuHistory {
    /// Записи истории для каждого GPU
    pub gpu_histories: Vec<SingleGpuHistory>,

    /// Максимальный размер истории
    pub max_size: usize,
}
/// История метрик для одного GPU
#[derive(Debug)]
pub struct SingleGpuHistory {
    /// Временные метки
    pub timestamps: VecDeque<Instant>,

    /// Температуры
    pub temperatures: VecDeque<f32>,

    /// Загрузка GPU
    pub utilizations: VecDeque<f32>,

    /// Потребление энергии
    pub power_usage: VecDeque<f32>,

    /// Использование памяти
    pub memory_utilizations: VecDeque<f32>,

    /// Частоты ядра
    pub core_clocks: VecDeque<u32>,

    /// Скорости вентиляторов
    pub fan_speeds: VecDeque<f32>,
}
/// Статистика мониторинга
#[derive(Debug, Default, Clone)]
pub struct MonitorStats {
    /// Время запуска мониторинга
    pub start_time: Option<Instant>,

    /// Общее количество собранных метрик
    pub total_measurements: u64,

    /// Количество алертов
    pub total_alerts: u64,

    /// Количество ошибок
    pub total_errors: u64,

    /// Среднее время сбора метрик
    pub avg_collection_time: Duration,

    /// Последнее время сбора
    pub last_collection_time: Option<Instant>,
}
/// Типы алертов
#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    /// Высокая температура
    HighTemperature { gpu_index: usize, temperature: f32 },

    /// Критическая температура
    CriticalTemperature { gpu_index: usize, temperature: f32 },

    /// Высокое использование памяти
    HighMemoryUsage { gpu_index: usize, usage: f32 },

    /// Критическое использование памяти
    CriticalMemoryUsage { gpu_index: usize, usage: f32 },

    /// Высокое потребление энергии
    HighPowerUsage { gpu_index: usize, power: f32 },

    /// Критическое потребление энергии
    CriticalPowerUsage { gpu_index: usize, power: f32 },

    /// Высокая загрузка GPU
    HighUtilization { gpu_index: usize, utilization: f32 },

    /// Низкая скорость вентилятора
    LowFanSpeed { gpu_index: usize, fan_speed: f32 },

    /// GPU стал неактивным
    GpuInactive { gpu_index: usize },

    /// Ошибка сбора данных
    CollectionError { gpu_index: usize, error: String },
}
/// Трейт для обработки алертов
pub trait AlertHandler: std::fmt::Debug {
    /// Обработать алерт
    fn handle_alert(&self, alert: &AlertType) -> Result<()>;
    /// Получить имя обработчика
    fn name(&self) -> &str;
}
/// Простой логгер алертов
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
    /// Создает новый монитор GPU
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
        }
    }
    /// Создает монитор с настройками по умолчанию
    pub fn with_defaults() -> Self {
        Self::new(MonitorConfig::default())
    }
    /// Добавляет обработчик алертов
    pub fn add_alert_handler(&self, handler: Box<dyn AlertHandler + Send + Sync>) -> Result<()> {
        if let Ok(mut handlers) = self.alert_handlers.lock() {
            info!("Added alert handler: {}", handler.name());
            handlers.push(handler);
            Ok(())
        } else {
            Err(GpuError::GpuNotActive)
        }
    }
    /// Запускает мониторинг в отдельном потоке
    pub fn start_monitoring(&self) -> Result<()> {
        if let Ok(mut is_running) = self.is_running.lock() {
            if *is_running {
                warn!("Monitoring is already running");
                return Ok(());
            }
            *is_running = true;
        }
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
        if let Ok(mut stats) = self.stats.lock() {
            stats.start_time = Some(Instant::now());
        }
        let gpu_manager = Arc::clone(&self.gpu_manager);
        let history = Arc::clone(&self.history);
        let alert_handlers = Arc::clone(&self.alert_handlers);
        let is_running = Arc::clone(&self.is_running);
        let stats = Arc::clone(&self.stats);
        let config = self.config.clone();
        thread::spawn(move || {
            Self::monitoring_loop(
                gpu_manager,
                history,
                alert_handlers,
                is_running,
                stats,
                config,
            );
        });
        Ok(())
    }
    /// Останавливает мониторинг
    pub fn stop_monitoring(&self) -> Result<()> {
        if let Ok(mut is_running) = self.is_running.lock() {
            if !*is_running {
                warn!("Monitoring is not running");
                return Ok(());
            }
            *is_running = false;
            info!("Stopping GPU monitoring");
        }
        Ok(())
    }
    /// Проверяет, запущен ли мониторинг
    pub fn is_monitoring(&self) -> bool {
        self.is_running.lock().map(|r| *r).unwrap_or(false)
    }
    /// Возвращает статистику мониторинга
    pub fn get_stats(&self) -> MonitorStats {
        self.stats.lock().map(|s| s.clone()).unwrap_or_default()
    }
    /// Возвращает историю для конкретного GPU
    pub fn get_gpu_history(&self, gpu_index: usize) -> Option<SingleGpuHistory> {
        if let Ok(history) = self.history.lock() {
            history.gpu_histories.get(gpu_index).cloned()
        } else {
            None
        }
    }
    /// Основной цикл мониторинга
    fn monitoring_loop(
        gpu_manager: Arc<Mutex<GpuManager>>,
        history: Arc<Mutex<GpuHistory>>,
        alert_handlers: Arc<Mutex<Vec<Box<dyn AlertHandler + Send + Sync>>>>,
        is_running: Arc<Mutex<bool>>,
        stats: Arc<Mutex<MonitorStats>>,
        config: MonitorConfig,
    ) {
        let mut consecutive_errors = 0;
        const MAX_CONSECUTIVE_ERRORS: u32 = 10;
        while Self::should_continue_monitoring(&is_running) {
            let collection_start = Instant::now();
            let collection_result = if let Ok(mut manager) = gpu_manager.lock() {
                manager.refresh_all_gpus()
            } else {
                Err(GpuError::GpuNotActive)
            };
            match collection_result {
                Ok(()) => {
                    consecutive_errors = 0;
                    if let Ok(manager) = gpu_manager.lock() {
                        let gpus = manager.get_all_gpus();
                        Self::update_history(&history, &gpus, collection_start);
                        if config.enable_alerts {
                            Self::check_alerts(&gpus, &config.thresholds, &alert_handlers);
                        }
                        if config.log_metrics {
                            Self::log_metrics(&gpus);
                        }
                        Self::update_stats(&stats, collection_start);
                    }
                }
                Err(e) => {
                    consecutive_errors += 1;
                    error!("GPU data collection failed: {}", e);
                    if let Ok(mut s) = stats.lock() {
                        s.total_errors += 1;
                    }
                    if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        warn!("Too many consecutive errors, taking a longer break");
                        thread::sleep(Duration::from_secs(10));
                        consecutive_errors = 0;
                    }
                }
            }
            thread::sleep(config.polling_interval);
        }
        info!("GPU monitoring loop ended");
    }
    /// Проверяет, нужно ли продолжать мониторинг
    fn should_continue_monitoring(is_running: &Arc<Mutex<bool>>) -> bool {
        is_running.lock().map(|r| *r).unwrap_or(false)
    }
    /// Обновляет историю метрик
    fn update_history(history: &Arc<Mutex<GpuHistory>>, gpus: &[GpuInfo], timestamp: Instant) {
        if let Ok(mut hist) = history.lock() {
            for (gpu_index, gpu) in gpus.iter().enumerate() {
                if let Some(gpu_history) = hist.gpu_histories.get_mut(gpu_index) {
                    gpu_history.add_measurement(gpu, timestamp);
                }
            }
        }
    }
    /// Проверяет алерты
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
    /// Логирует метрики
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
    /// Обновляет статистику
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
    /// Создает новую историю для указанного количества GPU
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
    /// Создает новую историю для одного GPU
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
    /// Добавляет новое измерение
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
    /// Возвращает среднюю температуру за указанный период
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
    /// Возвращает максимальную температуру за указанный период
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
