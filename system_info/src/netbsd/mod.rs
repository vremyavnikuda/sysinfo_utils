use log::trace;

use crate::{
    SystemVersion, architecture, bit_depth, system_info::Info, system_os::Type, system_uname::uname,
};

pub fn current_platform() -> Info {
    trace!("netbsd::current_platform is called");

    let version = uname("-s")
        .map(SystemVersion::from_string)
        .unwrap_or_else(|| SystemVersion::Unknown);

    let info = Info {
        system_type: Type::NetBSD,
        version,
        bit_depth: bit_depth::get(),
        architecture: architecture::get(),
        ..Default::default()
    };

    trace!("Returning {:?}", info);
    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn os_type() {
        let version = current_platform();
        assert_eq!(Type::NetBSD, version.system_type());
    }
}
