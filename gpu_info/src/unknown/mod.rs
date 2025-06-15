//gpu_info/src/unknown/mod
use crate::gpu_info::GpuInfo;
use log::warn;

pub fn init() -> Vec<GpuInfo> {
    warn!("Unknown platform: no GPU info available.");
    Vec::new()
}

pub fn update(_gpu: &mut GpuInfo) {
    warn!("Unknown platform: cannot update GPU info.");
}