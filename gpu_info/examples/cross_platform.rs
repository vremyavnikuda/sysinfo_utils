//! Example demonstrating the use of the new GPU provider manager on different platforms
#[cfg(target_os = "linux")]
use gpu_info::providers::linux::{AmdLinuxProvider, NvidiaLinuxProvider};
#[cfg(target_os = "macos")]
use gpu_info::providers::macos::MacosProvider;
use gpu_info::{vendor::Vendor, GpuProviderManager};
fn main() {
    println!("GPU Provider Manager Example");
    let mut provider_manager = GpuProviderManager::new();
    #[cfg(target_os = "windows")]
    {
        println!("Registering Windows GPU providers...");
        provider_manager.register_provider(Vendor::Nvidia, NvidiaProvider::new());
        provider_manager.register_provider(Vendor::Amd, AmdProvider::new());
        provider_manager.register_provider(Vendor::Intel(Default::default()), IntelProvider::new());
    }
    #[cfg(target_os = "linux")]
    {
        println!("Registering Linux GPU providers...");
        provider_manager.register_provider(Vendor::Nvidia, NvidiaLinuxProvider::new());
        provider_manager.register_provider(Vendor::Amd, AmdLinuxProvider::new());
    }
    #[cfg(target_os = "macos")]
    {
        println!("Registering macOS GPU providers...");
        provider_manager.register_provider(Vendor::Unknown, MacosProvider::new());
    }
    println!("Registered vendors:");
    for vendor in provider_manager.get_registered_vendors() {
        println!("  - {:?}", vendor);
    }
    println!("Detecting GPUs...");
    let gpus = provider_manager.detect_all_gpus();
    if gpus.is_empty() {
        println!("No GPUs detected");
    } else {
        println!("Found {} GPU(s):", gpus.len());
        for (i, gpu) in gpus.iter().enumerate() {
            println!("- GPU #{}: {} ({})", i, gpu.format_name_gpu(), gpu.vendor);
            println!("-- Temperature: {}°C", gpu.format_temperature());
            println!("-- Utilization: {}%", gpu.format_utilization());
            println!("-- Power Usage: {}W", gpu.format_power_usage());
            println!("-- Memory: {}GB", gpu.format_memory_total());
        }
    }
    println!("Provider Manager Capabilities:");
    println!(
        "  - Supports NVIDIA: {}",
        provider_manager.is_vendor_supported(&Vendor::Nvidia)
    );
    println!(
        "  - Supports AMD: {}",
        provider_manager.is_vendor_supported(&Vendor::Amd)
    );
    #[cfg(target_os = "windows")]
    {
        match Vendor::Intel(Default::default()) {
            intel_vendor => {
                println!(
                    "  - Supports Intel: {}",
                    provider_manager.is_vendor_supported(&intel_vendor)
                );
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        println!(
            "  - Supports Linux NVIDIA: {}",
            provider_manager.is_vendor_supported(&Vendor::Nvidia)
        );
        println!(
            "  - Supports Linux AMD: {}",
            provider_manager.is_vendor_supported(&Vendor::Amd)
        );
    }
}
