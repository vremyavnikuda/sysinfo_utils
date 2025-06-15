use crate::gpu_info::{GpuError, GpuInfo, Result};
use crate::vendor::{IntelGpuType, Vendor};
use log::{error, info, warn};
use std::process::Command;

fn determine_intel_gpu_type(name: &str) -> IntelGpuType {
    let name = name.to_lowercase();
    if name.contains("iris") || name.contains("uhd") || name.contains("hd graphics") {
        info!("Detected Intel integrated GPU: {}", name);
        IntelGpuType::Integrated
    } else if name.contains("arc") || name.contains("discrete") {
        info!("Detected Intel discrete GPU: {}", name);
        IntelGpuType::Discrete
    } else {
        warn!("Unknown Intel GPU type detected: {}", name);
        IntelGpuType::Unknown
    }
}

fn get_intel_gpu_info() -> Result<String> {
    let output = Command::new("powershell")
        .args([
            "Get-WmiObject",
            "Win32_VideoController",
            "|",
            "Where-Object",
            "{ $_.Name -like '*Intel*' }",
            "|",
            "Select-Object",
            "Name, AdapterRAM, DriverVersion, CurrentRefreshRate, MaxRefreshRate, Status",
            "|",
            "Format-List",
        ])
        .output()
        .map_err(|e| {
            error!("Failed to execute PowerShell command: {}", e);
            GpuError::DriverNotInstalled
        })?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_gpu_info(output_str: &str) -> Option<GpuInfo> {
    let gpu_name = output_str
        .lines()
        .find(|line| line.contains("Name"))
        .map(|line| line.split(":").nth(1).unwrap_or("").trim().to_string())
        .unwrap_or_else(|| {
            warn!("Failed to get GPU name, using default");
            "Intel GPU".to_string()
        });

    let gpu_type = determine_intel_gpu_type(&gpu_name);

    let driver_version = output_str
        .lines()
        .find(|line| line.contains("DriverVersion"))
        .map(|line| line.split(":").nth(1).unwrap_or("").trim().to_string());

    let memory_total = output_str
        .lines()
        .find(|line| line.contains("AdapterRAM"))
        .and_then(|line| line.split(":").nth(1)?.trim().parse::<u32>().ok());

    let core_clock = output_str
        .lines()
        .find(|line| line.contains("CurrentRefreshRate"))
        .and_then(|line| line.split(":").nth(1)?.trim().parse::<u32>().ok());

    let max_clock_speed = output_str
        .lines()
        .find(|line| line.contains("MaxRefreshRate"))
        .and_then(|line| line.split(":").nth(1)?.trim().parse::<u32>().ok());

    let status = output_str
        .lines()
        .find(|line| line.contains("Status"))
        .map(|line| line.split(":").nth(1).unwrap_or("").trim() == "OK");

    info!("Found Intel GPU: {} (Type: {:?})", gpu_name, gpu_type);
    if let Some(ver) = &driver_version {
        info!("Driver version: {}", ver);
    }
    if let Some(mem) = memory_total {
        info!("Total memory: {} GB", mem);
    }
    if let Some(clock) = core_clock {
        info!("Current clock speed: {} MHz", clock);
    }
    if let Some(max_clock) = max_clock_speed {
        info!("Max clock speed: {} MHz", max_clock);
    }

    Some(GpuInfo {
        name_gpu: Some(gpu_name),
        vendor: Vendor::Intel(gpu_type),
        driver_version,
        memory_total,
        core_clock,
        max_clock_speed,
        active: status,
        temperature: None,
        utilization: None,
        power_usage: None,
        power_limit: None,
        memory_util: None,
        memory_clock: None,
    })
}

pub fn detect_intel_gpus() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();
    info!("Starting Intel GPU detection");

    match get_intel_gpu_info() {
        Ok(output_str) => {
            if output_str.contains("Intel") {
                if let Some(gpu) = parse_gpu_info(&output_str) {
                    gpus.push(gpu);
                }
            } else {
                warn!("No Intel GPU found in system");
            }
        }
        Err(e) => {
            error!("Failed to get Intel GPU information: {}", e);
        }
    }

    if gpus.is_empty() {
        warn!("No Intel GPUs were detected");
    } else {
        info!("Successfully detected {} Intel GPU(s)", gpus.len());
    }

    gpus
}

pub fn update_intel_info(gpu: &mut GpuInfo) -> Result<()> {
    info!("Updating Intel GPU information");

    let output_str = get_intel_gpu_info()?;

    if let Some(updated_gpu) = parse_gpu_info(&output_str) {
        *gpu = updated_gpu;
    }

    if !gpu.is_valid() {
        warn!("GPU data validation failed");
        return Err(GpuError::GpuNotActive);
    }

    info!("Successfully updated Intel GPU information");
    Ok(())
}
