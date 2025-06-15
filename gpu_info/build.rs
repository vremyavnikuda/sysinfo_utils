fn main() {
    #[cfg(target_os = "windows")]
    {
        let project_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let lib_path = format!("{}/src/libs", project_path);

        // Проверяем наличие локальной библиотеки
        let dll_path = format!("{}/nvml.dll", lib_path);
        if std::path::Path::new(&dll_path).exists() {
            println!("cargo:rustc-link-search=native={}", lib_path);
            println!("cargo:rustc-link-lib=static=nvml");
            println!("cargo:warning=Using local nvml.dll from {}", dll_path);
        } else {
            println!(
                "cargo:warning=Local nvml.dll not found at {}, will try system library",
                dll_path
            );
            println!("cargo:rustc-link-lib=dylib=nvml");
        }
    }
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=dylib=nvidia-ml");
    }
}
