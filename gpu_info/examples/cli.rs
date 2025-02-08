use gpu_info::GpuManager;

//TODO: рассмотреть возможность разделения на подбиблиотеки (NVIDIA, AMD, Intel)
fn main() {
    let mut manager = GpuManager::new();
    manager.refresh();

    if let Some(gpu) = manager.gpus.first() {
        println!(
            "{}\n{}\n{}\n{}\n{}",
            gpu.name_gpu(),
            gpu.get_temperature_gpu(),
            gpu.get_utilization_gpu(),
            gpu.get_power_usage_gpu(),
            gpu.get_clock_speed_gpu()
        );
    }
}
