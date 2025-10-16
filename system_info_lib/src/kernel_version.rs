/// Returns the kernel version of the operating system.
///
/// On Unix-like systems (Linux, macOS, BSD), this calls `uname -r`.
/// On Windows, kernel version is not separately available.
///
/// # Returns
///
/// * `Option<String>` - The kernel version, if it can be determined.
///
/// # Examples
///
/// ```
/// # #[cfg(any(target_os = "linux", target_os = "macos"))]
/// # {
/// use system_info_lib;
///
/// let info = system_info_lib::get();
/// if let Some(version) = info.kernel_version() {
///     println!("Kernel version: {}", version);
/// }
/// # }
/// ```
#[cfg(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub fn get() -> Option<String> {
    use log::error;
    use std::process::Command;

    Command::new("uname")
        .arg("-r")
        .output()
        .map_err(|e| {
            error!("Failed to invoke 'uname -r': {:?}", e);
        })
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim_end().to_owned())
            } else {
                error!("'uname -r' invocation failed: {:?}", out);
                None
            }
        })
}

#[cfg(target_os = "windows")]
pub fn get() -> Option<String> {
    None
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "android",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "windows"
)))]
pub fn get() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "android",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    fn test_get_kernel_version() {
        let version = get();
        assert!(
            version.is_some(),
            "Kernel version should be available on Unix-like systems"
        );
        let version_str = version.unwrap();
        assert!(
            !version_str.is_empty(),
            "Kernel version should not be empty"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_get_kernel_version_windows() {
        let version = get();
        assert!(
            version.is_none(),
            "Kernel version is not separately tracked on Windows"
        );
    }
}
