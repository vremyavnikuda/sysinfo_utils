//! Example demonstrating the use of the new GPU provider manager
use gpu_info::vendor::Vendor;
use gpu_info::{
    providers::{AmdProvider, IntelProvider, NvidiaProvider},
    GpuProviderManager,
};
fn main() {
    println!("GPU Provider Manager Example");
    let mut provider_manager = GpuProviderManager::new();
    #[cfg(target_os = "windows")]
    {
        provider_manager.register_provider(Vendor::Nvidia, NvidiaProvider);
        provider_manager.register_provider(Vendor::Amd, AmdProvider);
        provider_manager.register_provider(Vendor::Intel(Default::default()), IntelProvider);
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
            println!("  GPU #{}: {} ({})", i, gpu.format_name_gpu(), gpu.vendor);
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
}
