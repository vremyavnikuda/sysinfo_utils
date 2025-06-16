use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::vendor::Vendor;
use log::{error, info, warn};
use once_cell::sync::Lazy;
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use windows::{
    core::*,
    Win32::{Foundation::*, System::LibraryLoader::*},
};

#[allow(non_snake_case)]
type ADLMainControlCreate =
    unsafe extern "C" fn(Option<unsafe extern "C" fn(usize) -> *mut c_void>, i32) -> i32;
type ADLMainControlDestroy = unsafe extern "C" fn() -> i32;
type ADLAdapterNumberOfAdaptersGet = unsafe extern "C" fn(*mut i32) -> i32;
type ADLAdapterAdapterInfoGet = unsafe extern "C" fn(*mut AdapterInfo, i32) -> i32;
type ADLOverdrive5TemperatureGet = unsafe extern "C" fn(i32, i32, *mut ADLTemperature) -> i32;
type ADLOverdrive5CurrentActivityGet = unsafe extern "C" fn(i32, *mut ADLPMActivity) -> i32;
type ADLOverdrive5PowerControlGet = unsafe extern "C" fn(i32, *mut i32, *mut i32) -> i32;

#[repr(C)]
#[allow(non_snake_case)]
struct AdapterInfo {
    strAdapterName: [u8; 256],
    strDisplayName: [u8; 256],
    iAdapterIndex: i32,
    iBusNumber: i32,
    iDeviceNumber: i32,
    iFunctionNumber: i32,
    iVendorID: i32,
    strDriverPath: [u8; 256],
    strDriverPathExt: [u8; 256],
    strPNPString: [u8; 256],
    iExist: i32,
    strUDID: [u8; 256],
    iChipFamily: i32,
    iChipRevision: i32,
    iExtInfo: i32,
    iMemorySize: i32,
    iMemoryType: i32,
    iMemoryBusWidth: i32,
    iMemoryBitWidth: i32,
    iCoreClock: i32,
    iMemoryClock: i32,
    iEngineClock: i32,
    iMemoryClock2: i32,
    iEngineClock2: i32,
    iMemoryClock3: i32,
    iEngineClock3: i32,
    iMemoryClock4: i32,
    iEngineClock4: i32,
    iMemoryClock5: i32,
    iEngineClock5: i32,
    iMemoryClock6: i32,
    iEngineClock6: i32,
    iMemoryClock7: i32,
    iEngineClock7: i32,
    iMemoryClock8: i32,
    iEngineClock8: i32,
    iMemoryClock9: i32,
    iEngineClock9: i32,
    iMemoryClock10: i32,
    iEngineClock10: i32,
    iMemoryClock11: i32,
    iEngineClock11: i32,
    iMemoryClock12: i32,
    iEngineClock12: i32,
    iMemoryClock13: i32,
    iEngineClock13: i32,
    iMemoryClock14: i32,
    iEngineClock14: i32,
    iMemoryClock15: i32,
    iEngineClock15: i32,
    iMemoryClock16: i32,
    iEngineClock16: i32,
    iMemoryClock17: i32,
    iEngineClock17: i32,
    iMemoryClock18: i32,
    iEngineClock18: i32,
    iMemoryClock19: i32,
    iEngineClock19: i32,
    iMemoryClock20: i32,
    iEngineClock20: i32,
    iMemoryClock21: i32,
    iEngineClock21: i32,
    iMemoryClock22: i32,
    iEngineClock22: i32,
    iMemoryClock23: i32,
    iEngineClock23: i32,
    iMemoryClock24: i32,
    iEngineClock24: i32,
    iMemoryClock25: i32,
    iEngineClock25: i32,
    iMemoryClock26: i32,
    iEngineClock26: i32,
    iMemoryClock27: i32,
    iEngineClock27: i32,
    iMemoryClock28: i32,
    iEngineClock28: i32,
    iMemoryClock29: i32,
    iEngineClock29: i32,
    iMemoryClock30: i32,
    iEngineClock30: i32,
    iMemoryClock31: i32,
    iEngineClock31: i32,
    iMemoryClock32: i32,
    iEngineClock32: i32,
    iMemoryClock33: i32,
    iEngineClock33: i32,
    iMemoryClock34: i32,
    iEngineClock34: i32,
    iMemoryClock35: i32,
    iEngineClock35: i32,
    iMemoryClock36: i32,
    iEngineClock36: i32,
    iMemoryClock37: i32,
    iEngineClock37: i32,
    iMemoryClock38: i32,
    iEngineClock38: i32,
    iMemoryClock39: i32,
    iEngineClock39: i32,
    iMemoryClock40: i32,
    iEngineClock40: i32,
    iMemoryClock41: i32,
    iEngineClock41: i32,
    iMemoryClock42: i32,
    iEngineClock42: i32,
    iMemoryClock43: i32,
    iEngineClock43: i32,
    iMemoryClock44: i32,
    iEngineClock44: i32,
    iMemoryClock45: i32,
    iEngineClock45: i32,
    iMemoryClock46: i32,
    iEngineClock46: i32,
    iMemoryClock47: i32,
    iEngineClock47: i32,
    iMemoryClock48: i32,
    iEngineClock48: i32,
    iMemoryClock49: i32,
    iEngineClock49: i32,
    iMemoryClock50: i32,
    iEngineClock50: i32,
    iMemoryClock51: i32,
    iEngineClock51: i32,
    iMemoryClock52: i32,
    iEngineClock52: i32,
    iMemoryClock53: i32,
    iEngineClock53: i32,
    iMemoryClock54: i32,
    iEngineClock54: i32,
    iMemoryClock55: i32,
    iEngineClock55: i32,
    iMemoryClock56: i32,
    iEngineClock56: i32,
    iMemoryClock57: i32,
    iEngineClock57: i32,
    iMemoryClock58: i32,
    iEngineClock58: i32,
    iMemoryClock59: i32,
    iEngineClock59: i32,
    iMemoryClock60: i32,
    iEngineClock60: i32,
    iMemoryClock61: i32,
    iEngineClock61: i32,
    iMemoryClock62: i32,
    iEngineClock62: i32,
    iMemoryClock63: i32,
    iEngineClock63: i32,
    iMemoryClock64: i32,
    iEngineClock64: i32,
    iMemoryClock65: i32,
    iEngineClock65: i32,
    iMemoryClock66: i32,
    iEngineClock66: i32,
    iMemoryClock67: i32,
    iEngineClock67: i32,
    iMemoryClock68: i32,
    iEngineClock68: i32,
    iMemoryClock69: i32,
    iEngineClock69: i32,
    iMemoryClock70: i32,
    iEngineClock70: i32,
    iMemoryClock71: i32,
    iEngineClock71: i32,
    iMemoryClock72: i32,
    iEngineClock72: i32,
    iMemoryClock73: i32,
    iEngineClock73: i32,
    iMemoryClock74: i32,
    iEngineClock74: i32,
    iMemoryClock75: i32,
    iEngineClock75: i32,
    iMemoryClock76: i32,
    iEngineClock76: i32,
    iMemoryClock77: i32,
    iEngineClock77: i32,
    iMemoryClock78: i32,
    iEngineClock78: i32,
    iMemoryClock79: i32,
    iEngineClock79: i32,
    iMemoryClock80: i32,
    iEngineClock80: i32,
    iMemoryClock81: i32,
    iEngineClock81: i32,
    iMemoryClock82: i32,
    iEngineClock82: i32,
    iMemoryClock83: i32,
    iEngineClock83: i32,
    iMemoryClock84: i32,
    iEngineClock84: i32,
    iMemoryClock85: i32,
    iEngineClock85: i32,
    iMemoryClock86: i32,
    iEngineClock86: i32,
    iMemoryClock87: i32,
    iEngineClock87: i32,
    iMemoryClock88: i32,
    iEngineClock88: i32,
    iMemoryClock89: i32,
    iEngineClock89: i32,
    iMemoryClock90: i32,
    iEngineClock90: i32,
    iMemoryClock91: i32,
    iEngineClock91: i32,
    iMemoryClock92: i32,
    iEngineClock92: i32,
    iMemoryClock93: i32,
    iEngineClock93: i32,
    iMemoryClock94: i32,
    iEngineClock94: i32,
    iMemoryClock95: i32,
    iEngineClock95: i32,
    iMemoryClock96: i32,
    iEngineClock96: i32,
    iMemoryClock97: i32,
    iEngineClock97: i32,
    iMemoryClock98: i32,
    iEngineClock98: i32,
    iMemoryClock99: i32,
    iEngineClock99: i32,
    iMemoryClock100: i32,
    iEngineClock100: i32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct ADLTemperature {
    iSize: i32,
    iTemperature: i32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct ADLPMActivity {
    iSize: i32,
    iEngineClock: i32,
    iMemoryClock: i32,
    iActivityPercent: i32,
    iCurrentPerformanceLevel: i32,
    iCurrentBusSpeed: i32,
    iCurrentBusLanes: i32,
    iMaximumBusLanes: i32,
    iReserved: i32,
}

const ADL_OK: i32 = 0;

unsafe extern "C" fn adl_main_memory_alloc(size: usize) -> *mut c_void {
    let ptr = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(size, 8));
    ptr as *mut c_void
}

#[allow(dead_code)]
unsafe extern "C" fn adl_main_memory_free(ptr: *mut *mut c_void) {
    if !(*ptr).is_null() {
        std::alloc::dealloc(
            *ptr as *mut u8,
            std::alloc::Layout::from_size_align_unchecked(1, 8),
        );
        *ptr = std::ptr::null_mut();
    }
}

fn load_adl_functions() -> Result<(
    HMODULE,
    ADLMainControlCreate,
    ADLMainControlDestroy,
    ADLAdapterNumberOfAdaptersGet,
    ADLAdapterAdapterInfoGet,
    ADLOverdrive5TemperatureGet,
    ADLOverdrive5CurrentActivityGet,
    ADLOverdrive5PowerControlGet,
)> {
    let dll = unsafe {
        LoadLibraryA(PCSTR::from_raw("atiadlxx.dll\0".as_ptr()))
            .map_err(|_| GpuError::DriverNotInstalled)?
    };

    let main_control_create = unsafe {
        std::mem::transmute(GetProcAddress(
            dll,
            PCSTR::from_raw("ADL_Main_Control_Create\0".as_ptr()),
        ))
    };
    let main_control_destroy = unsafe {
        std::mem::transmute(GetProcAddress(
            dll,
            PCSTR::from_raw("ADL_Main_Control_Destroy\0".as_ptr()),
        ))
    };
    let adapter_number_of_adapters_get = unsafe {
        std::mem::transmute(GetProcAddress(
            dll,
            PCSTR::from_raw("ADL_Adapter_NumberOfAdapters_Get\0".as_ptr()),
        ))
    };
    let adapter_adapter_info_get = unsafe {
        std::mem::transmute(GetProcAddress(
            dll,
            PCSTR::from_raw("ADL_Adapter_AdapterInfo_Get\0".as_ptr()),
        ))
    };
    let overdrive5_temperature_get = unsafe {
        std::mem::transmute(GetProcAddress(
            dll,
            PCSTR::from_raw("ADL_Overdrive5_Temperature_Get\0".as_ptr()),
        ))
    };
    let overdrive5_current_activity_get = unsafe {
        std::mem::transmute(GetProcAddress(
            dll,
            PCSTR::from_raw("ADL_Overdrive5_CurrentActivity_Get\0".as_ptr()),
        ))
    };
    let overdrive5_power_control_get = unsafe {
        std::mem::transmute(GetProcAddress(
            dll,
            PCSTR::from_raw("ADL_Overdrive5_PowerControl_Get\0".as_ptr()),
        ))
    };

    Ok((
        dll,
        main_control_create,
        main_control_destroy,
        adapter_number_of_adapters_get,
        adapter_adapter_info_get,
        overdrive5_temperature_get,
        overdrive5_current_activity_get,
        overdrive5_power_control_get,
    ))
}

static AMD_CLIENT: Lazy<Arc<Mutex<Option<AmdClientImpl>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

struct AmdClientImpl {
    dll: HMODULE,
    main_control_create: ADLMainControlCreate,
    main_control_destroy: ADLMainControlDestroy,
    adapter_number_of_adapters_get: ADLAdapterNumberOfAdaptersGet,
    adapter_adapter_info_get: ADLAdapterAdapterInfoGet,
    overdrive5_temperature_get: ADLOverdrive5TemperatureGet,
    overdrive5_current_activity_get: ADLOverdrive5CurrentActivityGet,
    overdrive5_power_control_get: ADLOverdrive5PowerControlGet,
}

impl AmdClientImpl {
    fn new() -> Result<Self> {
        let (
            dll,
            main_control_create,
            main_control_destroy,
            adapter_number_of_adapters_get,
            adapter_adapter_info_get,
            overdrive5_temperature_get,
            overdrive5_current_activity_get,
            overdrive5_power_control_get,
        ) = load_adl_functions()?;

        Ok(Self {
            dll,
            main_control_create,
            main_control_destroy,
            adapter_number_of_adapters_get,
            adapter_adapter_info_get,
            overdrive5_temperature_get,
            overdrive5_current_activity_get,
            overdrive5_power_control_get,
        })
    }
}

impl Drop for AmdClientImpl {
    fn drop(&mut self) {
        unsafe {
            (self.main_control_destroy)();
            let _ = FreeLibrary(self.dll);
        }
    }
}

pub fn detect_amd_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    info!("Starting AMD GPU detection using ADL");

    let (
        dll,
        main_control_create,
        main_control_destroy,
        adapter_number_of_adapters_get,
        adapter_adapter_info_get,
        overdrive5_temperature_get,
        overdrive5_current_activity_get,
        overdrive5_power_control_get,
    ) = match load_adl_functions() {
        Ok(functions) => functions,
        Err(e) => {
            error!("Failed to load ADL functions: {}", e);
            return gpus;
        }
    };

    // Инициализация ADL
    unsafe {
        if main_control_create(Some(adl_main_memory_alloc), 1) != ADL_OK {
            error!("Failed to initialize ADL");
            return gpus;
        }
    }

    // Получение количества адаптеров
    let mut adapter_count = 0;
    unsafe {
        if adapter_number_of_adapters_get(&mut adapter_count) != ADL_OK || adapter_count == 0 {
            error!("No AMD GPUs found");
            main_control_destroy();
            return gpus;
        }
    }

    // Получение информации об адаптерах
    let adapter_info_size = std::mem::size_of::<AdapterInfo>() * adapter_count as usize;
    let adapter_info = unsafe {
        let ptr = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
            adapter_info_size,
            8,
        ));
        std::slice::from_raw_parts_mut(ptr as *mut AdapterInfo, adapter_count as usize)
    };

    unsafe {
        if adapter_adapter_info_get(adapter_info.as_mut_ptr(), adapter_info_size as i32) != ADL_OK {
            error!("Failed to get adapter information");
            std::alloc::dealloc(
                adapter_info.as_mut_ptr() as *mut u8,
                std::alloc::Layout::from_size_align_unchecked(adapter_info_size, 8),
            );
            main_control_destroy();
            return gpus;
        }
    }

    // Обработка каждого адаптера
    for adapter in adapter_info.iter() {
        let mut gpu_info = GpuInfo {
            vendor: Vendor::Amd,
            name_gpu: Some(
                String::from_utf8_lossy(&adapter.strAdapterName)
                    .trim_end_matches('\0')
                    .to_string(),
            ),
            temperature: None,
            utilization: None,
            power_usage: None,
            core_clock: None,
            memory_util: None,
            memory_clock: None,
            active: None,
            power_limit: None,
            memory_total: Some((adapter.iMemorySize / (1024 * 1024 * 1024)) as u32),
            driver_version: None,
            max_clock_speed: None,
        };

        // Получение температуры
        let mut temp = ADLTemperature {
            iSize: std::mem::size_of::<ADLTemperature>() as i32,
            iTemperature: 0,
        };
        unsafe {
            if overdrive5_temperature_get(adapter.iAdapterIndex, 0, &mut temp) == ADL_OK {
                gpu_info.temperature = Some(temp.iTemperature as f32 / 1000.0);
            }
        }

        // Получение активности и частот
        let mut activity = ADLPMActivity {
            iSize: std::mem::size_of::<ADLPMActivity>() as i32,
            iEngineClock: 0,
            iMemoryClock: 0,
            iActivityPercent: 0,
            iCurrentPerformanceLevel: 0,
            iCurrentBusSpeed: 0,
            iCurrentBusLanes: 0,
            iMaximumBusLanes: 0,
            iReserved: 0,
        };
        unsafe {
            if overdrive5_current_activity_get(adapter.iAdapterIndex, &mut activity) == ADL_OK {
                gpu_info.utilization = Some(activity.iActivityPercent as f32);
                gpu_info.core_clock = Some((activity.iEngineClock / 100) as u32);
                gpu_info.memory_clock = Some((activity.iMemoryClock / 100) as u32);
                gpu_info.active = Some(activity.iActivityPercent > 0);
            }
        }

        // Получение информации о питании
        let mut power = 0;
        let mut power_limit = 0;
        unsafe {
            if overdrive5_power_control_get(adapter.iAdapterIndex, &mut power, &mut power_limit)
                == ADL_OK
            {
                gpu_info.power_usage = Some(power as f32);
                gpu_info.power_limit = Some(power_limit as f32);
            }
        }

        gpus.push(gpu_info);
    }

    // Очистка
    unsafe {
        std::alloc::dealloc(
            adapter_info.as_mut_ptr() as *mut u8,
            std::alloc::Layout::from_size_align_unchecked(adapter_info_size, 8),
        );
        main_control_destroy();
        let _ = FreeLibrary(dll);
    }

    if gpus.is_empty() {
        warn!("No AMD GPUs were detected");
    } else {
        info!("Successfully detected {} AMD GPU(s)", gpus.len());
    }

    gpus
}

pub fn update_amd_info(gpu: &mut GpuInfo) -> Result<()> {
    info!("Updating AMD GPU information using ADL");

    let mut client_guard = AMD_CLIENT.lock().map_err(|_| GpuError::GpuNotActive)?;
    if client_guard.is_none() {
        *client_guard = Some(AmdClientImpl::new()?);
    }
    let client = client_guard.as_ref().ok_or(GpuError::GpuNotActive)?;

    // Инициализация ADL
    unsafe {
        if (client.main_control_create)(Some(adl_main_memory_alloc), 1) != ADL_OK {
            return Err(GpuError::DriverNotInstalled.into());
        }
    }

    // Получение количества адаптеров
    let mut adapter_count = 0;
    unsafe {
        if (client.adapter_number_of_adapters_get)(&mut adapter_count) != ADL_OK
            || adapter_count == 0
        {
            return Err(GpuError::GpuNotActive.into());
        }
    }

    // Получение информации об адаптерах
    let adapter_info_size = std::mem::size_of::<AdapterInfo>() * adapter_count as usize;
    let adapter_info = unsafe {
        let ptr = std::alloc::alloc(std::alloc::Layout::from_size_align_unchecked(
            adapter_info_size,
            8,
        ));
        std::slice::from_raw_parts_mut(ptr as *mut AdapterInfo, adapter_count as usize)
    };

    unsafe {
        if (client.adapter_adapter_info_get)(adapter_info.as_mut_ptr(), adapter_info_size as i32)
            != ADL_OK
        {
            std::alloc::dealloc(
                adapter_info.as_mut_ptr() as *mut u8,
                std::alloc::Layout::from_size_align_unchecked(adapter_info_size, 8),
            );
            return Err(GpuError::GpuNotActive.into());
        }
    }

    // Обновление информации для первого найденного адаптера
    if let Some(adapter) = adapter_info.first() {
        // Обновление температуры
        let mut temp = ADLTemperature {
            iSize: std::mem::size_of::<ADLTemperature>() as i32,
            iTemperature: 0,
        };
        unsafe {
            if (client.overdrive5_temperature_get)(adapter.iAdapterIndex, 0, &mut temp) == ADL_OK {
                gpu.temperature = Some(temp.iTemperature as f32 / 1000.0);
            }
        }

        // Обновление активности и частот
        let mut activity = ADLPMActivity {
            iSize: std::mem::size_of::<ADLPMActivity>() as i32,
            iEngineClock: 0,
            iMemoryClock: 0,
            iActivityPercent: 0,
            iCurrentPerformanceLevel: 0,
            iCurrentBusSpeed: 0,
            iCurrentBusLanes: 0,
            iMaximumBusLanes: 0,
            iReserved: 0,
        };
        unsafe {
            if (client.overdrive5_current_activity_get)(adapter.iAdapterIndex, &mut activity)
                == ADL_OK
            {
                gpu.utilization = Some(activity.iActivityPercent as f32);
                gpu.core_clock = Some((activity.iEngineClock / 100) as u32);
                gpu.memory_clock = Some((activity.iMemoryClock / 100) as u32);
                gpu.active = Some(activity.iActivityPercent > 0);
            }
        }

        // Обновление информации о питании
        let mut power = 0;
        let mut power_limit = 0;
        unsafe {
            if (client.overdrive5_power_control_get)(
                adapter.iAdapterIndex,
                &mut power,
                &mut power_limit,
            ) == ADL_OK
            {
                gpu.power_usage = Some(power as f32);
                gpu.power_limit = Some(power_limit as f32);
            }
        }
    }

    // Очистка
    unsafe {
        std::alloc::dealloc(
            adapter_info.as_mut_ptr() as *mut u8,
            std::alloc::Layout::from_size_align_unchecked(adapter_info_size, 8),
        );
    }

    if !gpu.is_valid() {
        warn!("GPU data validation failed");
        return Err(GpuError::GpuNotActive.into());
    }

    info!("Successfully updated AMD GPU information");
    Ok(())
}
