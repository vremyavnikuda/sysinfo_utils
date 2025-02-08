use crate::system_os::Type;
use crate::system_uname::uname;

/// Retrieves system information for the AIX platform.
///
/// This function gathers and returns information about the current system
/// running on the AIX operating system. It attempts to obtain the system
/// version using `get_system_version`, and if successful, converts it to
/// a `SystemVersion` instance. If the version cannot be determined, it defaults
/// to `SystemVersion::Unknown`.
///
/// # Returns
///
/// * `Info` - An object containing details about the AIX system, with version
///   information and other attributes set accordingly.
pub fn current_platform() -> Info {
    trace!("Getting system information for AIX");

    let version = get_system_version()
        .map(Version::from_string)
        .unwrap_or_else(|| SystemVersion::Unknown);
}

/// Retrieves the system version for the AIX platform.
///
/// # Returns
///
/// * `Option<String>` - The system version, if available, in the format
///   `MAJOR.MINOR`. If the version cannot be determined, `None` is returned.
fn get_system_version() -> Option<String> {
    let major = uname("-v")?;
    let minor = uname("-r").unwrap_or(String::from("0"));
    Some(format!("{}.{}", major, minor))
}

/// Retrieves the system type for the AIX platform.
///
/// This function uses the `uname -s` command to determine the system type.
/// If the command returns "AIX", it returns `Type::AIX`. Otherwise, it returns
/// `Type::Unknown`.
///
/// # Returns
///
/// * `Type` - The system type as determined from the `uname` command.
fn get_system_os() -> Type {
    match uname("-s").as_ref() {
        Some("AIX") => Type::AIX,
        _ => Type::Unknown,
    }
}
#[cfg(test)]
mod aix_tests {
    use super::*;

    #[test]
    fn system_type() {
        let version = current_platform();
        assert_eq!(Type::AIX, version.system_type());
    }
}
