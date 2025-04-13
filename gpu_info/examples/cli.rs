fn main() {
    // Создаем менеджер GPU
    let gpu = gpu_info::get();

    println!("Vendor: {:?}", gpu.vendor());
    println!("Name: {:?}", gpu.name_gpu());
    println!("Utilization: {:?}", gpu.utilization());
    println!("Temperature: {:?}", gpu.temperature());
    println!("Clock Speed: {:?}", gpu.core_clock());
    println!("Power Usage: {:?}", gpu.power_usage());
    println!("Memory Usage: {:?}", gpu.memory_util());
    println!("Memory Total: {:?}", gpu.memory_total());
    println!("Is active: {:?}", gpu.active());
}