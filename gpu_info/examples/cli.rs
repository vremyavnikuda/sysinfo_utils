use gpu_info::GpuManager;

fn main() {
    let mut manager = GpuManager::new();
    manager.refresh();

    if let Some(gpu) = manager.gpus.first() {
        println!("{}", gpu.name_gpu());
        println!("{}", gpu.get_temperature_gpu());
        println!("{}", gpu.get_utilization_gpu());
        println!("{}", gpu.get_power_usage_gpu());
        println!("{}", gpu.get_clock_speed_gpu());
    } else {
        println!("No GPUs detected.");
    }
}