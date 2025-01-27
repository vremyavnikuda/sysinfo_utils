use gpu_info::GpuManager;

//TODO: рассмотреть возможность разделения на подбиблиотеки (NVIDIA, AMD, Intel)
fn main() {
    let mut manager = GpuManager::new();
    manager.refresh();

    if let Some(gpu) = manager.gpus.first() {
        println!(
            "{}\n{}\n{}\n{}\n{}",
            gpu.get_name(),
            gpu.get_temperature(),
            gpu.get_utilization(),
            gpu.get_power_usage(),
            gpu.get_clock_speed()
        );
    }
}
