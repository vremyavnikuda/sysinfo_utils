// src/nvidia/mod.rs

use crate::mode::gpu::{GpuInfo, GpuVendor};
use std::process::Command;

/// Detects all nvidia GPUs and parses their information from the `nvidia-smi` command.
///
/// The function executes the `nvidia-smi` command with the following arguments:
///
/// - `--query-gpu=name,temperature.gpu,utilization.gpu,clocks.current.graphics,clocks.max.graphics,power.draw,power.max_limit`
/// - `--format=csv,noheader,nounits`
///
/// It then parses the output and creates a `GpuInfo` instance for each GPU.
///
/// # Parameters
/// None
///
/// # Returns
/// A vector of `GpuInfo` instances, one for each detected nvidia GPU.
pub fn detect_nvidia_gpus() -> Vec<GpuInfo> {
	let mut gpus = Vec::new();

	if let Ok(output) = Command::new("nvidia-smi")
		.args(&[
			"--query-gpu=name,temperature.gpu,utilization.gpu,clocks.current.graphics,clocks.max.graphics,power.draw,power.max_limit",
			"--format=csv,noheader,nounits",
		])
		.output()
	{
		if output.status.success() {
			let data = String::from_utf8_lossy(&output.stdout);
			for line in data.lines() {
				let parts: Vec<&str> = line.split(',').collect();
				if parts.len() == 7 {
					gpus.push(GpuInfo {
						name: parts[0].trim().to_string(),
						vendor: GpuVendor::Nvidia,
						temperature: parts[1].trim().parse().ok(),
						utilization: parts[2].trim().parse().ok(),
						clock_speed: parts[3].trim().parse().ok(),
						max_clock_speed: parts[4].trim().parse().ok(),
						power_usage: parts[5].trim().parse().ok(),
						max_power_usage: parts[6].trim().parse().ok(),
						is_active: true,
					});
				}
			}
		}
	}

	gpus
}

// (Internal) Updates nvidia GPU metrics
///
/// # Data Sources
/// - temperature.gpu
/// - utilization.gpu
/// - clocks_speed.current.graphics
/// - power_usage.draw
pub fn update_nvidia_info(gpu: &mut GpuInfo) {
	let output = Command::new("nvidia-smi")
		.args(&[
			"--query-gpu=temperature.gpu,utilization.gpu,clocks.current.graphics,power.draw",
			"--format=csv,noheader,nounits",
		])
		.output()
		.expect("Failed to execute nvidia-smi command");

	let data = String::from_utf8_lossy(&output.stdout);
	let parts: Vec<&str> = data.split(',').collect();
	gpu.temperature = parts.get(0).and_then(|s| s.trim().parse().ok());
	gpu.utilization = parts.get(1).and_then(|s| s.trim().parse().ok());
	gpu.clock_speed = parts.get(2).and_then(|s| s.trim().parse().ok());
	gpu.power_usage = parts.get(3).and_then(|s| s.trim().parse().ok());
	// TODO:Возможно имеет смысл использовать этот код
	/*
	if parts.len() >= 4 {
		gpu.temperature = parts.get(0).and_then(|s| s.trim().parse().ok());
		gpu.utilization = parts.get(1).and_then(|s| s.trim().parse().ok());
		gpu.clock_speed = parts.get(2).and_then(|s| s.trim().parse().ok());
		gpu.power_usage = parts.get(3).and_then(|s| s.trim().parse().ok());
	} else {
		warn!("nvidia-smi returned unexpected format: {:?}", parts);
	}
	*/
}
