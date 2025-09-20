use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::vendor::Vendor;
use log::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use std::time::Duration;
/// Менеджер для работы с множественными GPU в системе
#[derive(Debug, Clone)]
pub struct GpuManager {
    /// Список всех обнаруженных GPU
    gpus: Vec<GpuInfo>,
    /// Индекс основного GPU (используется по умолчанию)
    primary_gpu_index: usize,
    /// GPU information cache with unified caching utilities
    ///
    /// This cache eliminates duplication by using the common caching infrastructure.
    cache: crate::cache_utils::MultiGpuInfoCache,
}
impl Default for GpuManager {
    fn default() -> Self {
        Self::new()
    }
}
impl GpuManager {
    /// Создает новый менеджер GPU с автоматическим обнаружением
    pub fn new() -> Self {
        let mut manager = Self {
            gpus: Vec::new(),
            primary_gpu_index: 0,
            cache: crate::cache_utils::MultiGpuInfoCache::new(Duration::from_millis(500)),
        };
        manager.detect_all_gpus();
        manager
    }
    /// Создает менеджер с настраиваемым TTL кэша
    pub fn with_cache_ttl(cache_ttl: Duration) -> Self {
        let mut manager = Self {
            gpus: Vec::new(),
            primary_gpu_index: 0,
            cache: crate::cache_utils::MultiGpuInfoCache::new(cache_ttl),
        };
        manager.detect_all_gpus();
        manager
    }
    /// Создает менеджер с настраиваемым TTL кэша и максимальным размером
    pub fn with_cache_config(cache_ttl: Duration, max_entries: usize) -> Self {
        let mut manager = Self {
            gpus: Vec::new(),
            primary_gpu_index: 0,
            cache: crate::cache_utils::MultiGpuInfoCache::with_max_entries(cache_ttl, max_entries),
        };
        manager.detect_all_gpus();
        manager
    }
    /// Обнаруживает все GPU в системе
    pub fn detect_all_gpus(&mut self) {
        self.gpus.clear();
        info!("Starting multi-GPU detection");
        // Обнаружение по платформам
        #[cfg(target_os = "windows")]
        {
            self.detect_windows_gpus();
        }
        #[cfg(target_os = "linux")]
        {
            self.detect_linux_gpus();
        }
        #[cfg(target_os = "macos")]
        {
            self.detect_macos_gpus();
        }
        if self.gpus.is_empty() {
            warn!("No GPUs detected in the system");
            self.gpus.push(GpuInfo::unknown());
        } else {
            info!("Detected {} GPU(s) in the system", self.gpus.len());
            self.select_primary_gpu();
        }
    }
    #[cfg(target_os = "windows")]
    fn detect_windows_gpus(&mut self) {
        use crate::providers::{amd, intel, nvidia};
        // NVIDIA GPUs
        if let Ok(nvidia_gpus) = nvidia::detect_nvidia_gpus() {
            for (index, gpu) in nvidia_gpus.into_iter().enumerate() {
                info!("Found NVIDIA GPU #{}: {:?}", index, gpu.name_gpu);
                self.gpus.push(gpu);
            }
        }
        // AMD GPUs
        if let Ok(amd_gpus) = amd::detect_amd_gpus() {
            for (index, gpu) in amd_gpus.into_iter().enumerate() {
                info!("Found AMD GPU #{}: {:?}", index, gpu.name_gpu);
                self.gpus.push(gpu);
            }
        }
        // Intel GPUs
        let intel_gpus = intel::detect_intel_gpus();
        for (index, gpu) in intel_gpus.into_iter().enumerate() {
            info!("Found Intel GPU #{}: {:?}", index, gpu.name_gpu);
            self.gpus.push(gpu);
        }
    }
    #[cfg(target_os = "linux")]
    fn detect_linux_gpus(&mut self) {
        use crate::linux;
        // NVIDIA GPUs
        if let Ok(nvidia_gpus) = linux::nvidia::detect_nvidia_gpus() {
            for gpu in nvidia_gpus {
                info!("Found NVIDIA GPU: {:?}", gpu.name_gpu);
                self.gpus.push(gpu);
            }
        }
        // AMD GPUs (если реализовано)
        #[cfg(feature = "linux_amd")]
        {
            if let Ok(amd_gpus) = linux::amd::detect_amd_gpus() {
                for gpu in amd_gpus {
                    info!("Found AMD GPU: {:?}", gpu.name_gpu);
                    self.gpus.push(gpu);
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    fn detect_macos_gpus(&mut self) {
        use crate::macos;
        let gpus = macos::get_all_gpus();
        for gpu in gpus {
            info!("Found macOS GPU: {:?}", gpu.name_gpu);
            self.gpus.push(gpu);
        }
    }
    /// Выбирает основной GPU (приоритет дискретным GPU)
    fn select_primary_gpu(&mut self) {
        for (index, gpu) in self.gpus.iter().enumerate() {
            match gpu.vendor {
                Vendor::Nvidia | Vendor::Amd => {
                    self.primary_gpu_index = index;
                    info!(
                        "Selected primary GPU: {} (index {})",
                        gpu.name_gpu.as_deref().unwrap_or("Unknown"),
                        index
                    );
                    return;
                }
                _ => continue,
            }
        }
        if !self.gpus.is_empty() {
            self.primary_gpu_index = 0;
            info!(
                "Selected primary GPU: {} (index 0)",
                self.gpus[0].name_gpu.as_deref().unwrap_or("Unknown")
            );
        }
    }
    /// Возвращает количество обнаруженных GPU
    pub fn gpu_count(&self) -> usize {
        self.gpus.len()
    }
    /// Возвращает информацию о всех GPU
    pub fn get_all_gpus(&self) -> &Vec<GpuInfo> {
        &self.gpus
    }
    /// Возвращает копию всех GPU
    pub fn get_all_gpus_owned(&self) -> Vec<GpuInfo> {
        self.gpus.clone()
    }
    /// Возвращает основной GPU
    pub fn get_primary_gpu(&self) -> Option<&GpuInfo> {
        self.gpus.get(self.primary_gpu_index)
    }
    /// Возвращает копию основного GPU
    pub fn get_primary_gpu_owned(&self) -> Option<GpuInfo> {
        self.gpus.get(self.primary_gpu_index).cloned()
    }
    /// Возвращает GPU по индексу
    pub fn get_gpu_by_index(&self, index: usize) -> Option<&GpuInfo> {
        self.gpus.get(index)
    }
    /// Возвращает копию GPU по индексу
    pub fn get_gpu_by_index_owned(&self, index: usize) -> Option<GpuInfo> {
        self.gpus.get(index).cloned()
    }
    /// Возвращает GPU по производителю
    pub fn get_gpus_by_vendor(&self, vendor: Vendor) -> Vec<&GpuInfo> {
        self.gpus
            .iter()
            .filter(|gpu| gpu.vendor == vendor)
            .collect()
    }
    /// Возвращает копии GPU по производителю
    pub fn get_gpus_by_vendor_owned(&self, vendor: Vendor) -> Vec<GpuInfo> {
        self.gpus
            .iter()
            .filter(|gpu| gpu.vendor == vendor)
            .cloned()
            .collect()
    }
    /// Устанавливает основной GPU
    pub fn set_primary_gpu(&mut self, index: usize) -> Result<()> {
        if index >= self.gpus.len() {
            return Err(GpuError::GpuNotFound);
        }
        self.primary_gpu_index = index;
        info!(
            "Primary GPU changed to index {}: {}",
            index,
            self.gpus[index].name_gpu.as_deref().unwrap_or("Unknown")
        );
        Ok(())
    }
    /// Обновляет информацию о всех GPU
    pub fn refresh_all_gpus(&mut self) -> Result<()> {
        debug!("Refreshing information for all {} GPUs", self.gpus.len());
        let mut errors = Vec::new();
        for (index, gpu) in self.gpus.iter_mut().enumerate() {
            if let Err(e) = Self::update_single_gpu_static(gpu) {
                error!("Failed to update GPU #{}: {}", index, e);
                errors.push((index, e));
            }
        }
        self.cache.clear_all();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.into_iter().next().unwrap().1)
        }
    }
    /// Обновляет информацию о конкретном GPU
    pub fn refresh_gpu(&mut self, index: usize) -> Result<()> {
        let gpu = self.gpus.get_mut(index).ok_or(GpuError::GpuNotFound)?;
        Self::update_single_gpu_static(gpu)?;
        self.cache.set(index, gpu.clone());
        Ok(())
    }
    /// Обновляет информацию об основном GPU
    pub fn refresh_primary_gpu(&mut self) -> Result<()> {
        self.refresh_gpu(self.primary_gpu_index)
    }
    /// Внутренняя функция обновления одного GPU
    fn update_single_gpu_static(gpu: &mut GpuInfo) -> Result<()> {
        // Use the new provider interface when available
        #[cfg(target_os = "windows")]
        {
            use crate::providers::{amd, intel, nvidia};
            match gpu.vendor {
                Vendor::Nvidia => nvidia::update_nvidia_info(gpu),
                Vendor::Amd => amd::update_amd_info(gpu),
                Vendor::Intel(_) => intel::update_intel_info(gpu),
                _ => {
                    warn!("GPU update not implemented for vendor: {:?}", gpu.vendor);
                    Ok(())
                }
            }
        }
        #[cfg(target_os = "linux")]
        {
            match gpu.vendor {
                Vendor::Nvidia => crate::linux::nvidia::update_nvidia_info(gpu),
                _ => {
                    warn!("GPU update not implemented for vendor: {:?}", gpu.vendor);
                    Ok(())
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            crate::macos::update_gpu_info(gpu)
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            warn!("GPU update not implemented for vendor: {:?}", gpu.vendor);
            Ok(())
        }
    }
    /// Возвращает GPU с кэшированием
    pub fn get_gpu_cached(&self, index: usize) -> Option<GpuInfo> {
        if let Some(cached_gpu) = self.cache.get(&index) {
            debug!("Returning cached GPU #{}", index);
            return Some(cached_gpu);
        }
        // If not in cache, get from source and populate cache
        if let Some(gpu) = self.get_gpu_by_index_owned(index) {
            self.cache.set(index, gpu.clone());
            debug!("Populated cache for GPU #{}", index);
            Some(gpu)
        } else {
            None
        }
    }
    /// Возвращает основной GPU с кэшированием
    pub fn get_primary_gpu_cached(&self) -> Option<GpuInfo> {
        self.get_gpu_cached(self.primary_gpu_index)
    }
    /// Возвращает статистику по GPU
    pub fn get_gpu_statistics(&self) -> GpuStatistics {
        let mut stats = GpuStatistics::default();
        for gpu in &self.gpus {
            match gpu.vendor {
                Vendor::Nvidia => stats.nvidia_count += 1,
                Vendor::Amd => stats.amd_count += 1,
                Vendor::Intel(_) => stats.intel_count += 1,
                Vendor::Apple => stats.apple_count += 1,
                Vendor::Unknown => stats.unknown_count += 1,
            }
            if let Some(temp) = gpu.temperature {
                stats.total_temperature += temp;
                stats.temperature_readings += 1;
            }
            if let Some(power) = gpu.power_usage {
                stats.total_power_usage += power;
                stats.power_readings += 1;
            }
        }
        stats.total_gpus = self.gpus.len();
        stats
    }
    /// Проверяет, все ли GPU активны
    pub fn all_gpus_active(&self) -> bool {
        self.gpus.iter().all(|gpu| gpu.active.unwrap_or(false))
    }
    /// Возвращает список индексов активных GPU
    pub fn get_active_gpu_indices(&self) -> Vec<usize> {
        self.gpus
            .iter()
            .enumerate()
            .filter(|(_, gpu)| gpu.active.unwrap_or(false))
            .map(|(index, _)| index)
            .collect()
    }
    /// Получает статистику кэша
    pub fn get_cache_stats(&self) -> Option<crate::cache_utils::CacheStats> {
        self.cache.get_stats()
    }
}
/// Статистика по GPU в системе
#[derive(Debug, Default, Clone)]
pub struct GpuStatistics {
    pub total_gpus: usize,
    pub nvidia_count: usize,
    pub amd_count: usize,
    pub intel_count: usize,
    pub apple_count: usize,
    pub unknown_count: usize,
    pub total_temperature: f32,
    pub temperature_readings: usize,
    pub total_power_usage: f32,
    pub power_readings: usize,
}
impl GpuStatistics {
    /// Возвращает среднюю температуру по всем GPU
    pub fn average_temperature(&self) -> Option<f32> {
        if self.temperature_readings > 0 {
            Some(self.total_temperature / self.temperature_readings as f32)
        } else {
            None
        }
    }
    /// Возвращает общее энергопотребление всех GPU
    pub fn total_power_consumption(&self) -> Option<f32> {
        if self.power_readings > 0 {
            Some(self.total_power_usage)
        } else {
            None
        }
    }
}
// Глобальная статическая переменная для singleton доступа
use std::sync::OnceLock;
static GPU_MANAGER: OnceLock<Arc<Mutex<GpuManager>>> = OnceLock::new();
/// Возвращает глобальный экземпляр GpuManager
pub fn global_gpu_manager() -> Arc<Mutex<GpuManager>> {
    GPU_MANAGER
        .get_or_init(|| Arc::new(Mutex::new(GpuManager::new())))
        .clone()
}
/// Convenience функция для получения основного GPU
pub fn get_primary_gpu() -> Option<GpuInfo> {
    let manager = global_gpu_manager();
    let result = if let Ok(mgr) = manager.lock() {
        mgr.get_primary_gpu_cached()
    } else {
        None
    };
    result
}
/// Convenience функция для получения всех GPU
pub fn get_all_gpus() -> Vec<GpuInfo> {
    let manager = global_gpu_manager();
    let result = if let Ok(mgr) = manager.lock() {
        mgr.get_all_gpus_owned()
    } else {
        Vec::new()
    };
    result
}
/// Convenience функция для получения количества GPU
pub fn get_gpu_count() -> usize {
    let manager = global_gpu_manager();
    let result = if let Ok(mgr) = manager.lock() {
        mgr.gpu_count()
    } else {
        0
    };
    result
}
