//src/windows/mod.rs
mod winapi;

use log::trace;

use crate::Info;

pub fn current_platform() -> Info {
    trace!("windows::current_platform is called");
    let info = winapi::get();
    trace!("Returning {:?}", info);
    info
}

#[cfg(test)]
mod windows_tests {
    use crate::system_os::Type;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn system_type() {
        let version = current_platform();
        assert_eq!(Type::Windows, version.system_type());
        assert!(version.edition().is_some());
    }
}
