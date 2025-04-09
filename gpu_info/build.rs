//gpu_info/build.rs
fn main() {
    let project_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_path = format!("{}/src/libs", project_path);

    println!("cargo:rustc-link-search=native={}", lib_path);
    println!("cargo:rustc-link-lib=dylib=nvml");

    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=static=nvml");
}