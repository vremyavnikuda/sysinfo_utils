//src/platform/macos.rs
use crate::{gpu_info::GpuInfo, vendor::Vendor};
use log::warn;
use std::process::Command;

/// Возвращает список доступных GPU на macOS.
///
/// Использует `system_profiler` для получения информации о видеокартах.
/// В результате возвращает вектор `GpuInfo`, содержащий базовую информацию.
pub fn init() -> Vec<GpuInfo> {
    let mut gpus = Vec::new();

    let output = Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);

        let mut name = String::new();
        let mut vendor;

        for line in stdout.lines() {
            if line.contains("Chipset Model:") {
                name = line.split(':').nth(1).unwrap_or("").trim().to_string();
            }

            if line.contains("Vendor:") {
                let ven_str = line.split(':').nth(1).unwrap_or("").trim().to_lowercase();
                vendor = if ven_str.contains("amd") {
                    Vendor::Amd
                } else if ven_str.contains("intel") {
                    Vendor::Intel
                } else if ven_str.contains("nvidia") {
                    Vendor::Nvidia
                } else if ven_str.contains("apple") {
                    Vendor::Apple
                } else {
                    Vendor::Unknown(ven_str)
                };

                gpus.push(GpuInfo {
                    name_gpu: Some(name.clone()),
                    vendor: Some(vendor),
                    active: Some(true),
                    ..Default::default()
                });
            }
        }
    } else {
        warn!("Failed to invoke system_profiler for GPU info.");
    }

    gpus
}

/// Обновление подробной информации о GPU на macOS (не реализовано).
///
/// macOS не предоставляет простой способ получения таких метрик, как
/// температура или потребление энергии, без привязки к проприетарным API.
/// Поэтому эта функция на данный момент только помечает GPU как активный.
pub fn update(gpu: &mut GpuInfo) {
    gpu.active = Some(true);
    // Расширение возможно при использовании IOKit через FFI или сторонних библиотек
}
