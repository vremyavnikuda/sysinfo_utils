//src/illumos/mod.rs
use log::trace;

use crate::{bit_depth, system_info::Info, system_os::Type, system_uname::uname, SystemVersion};

pub fn current_platform() -> Info {
    trace!("illumos::current_platform() is called");

    let version = uname("-r")
        .map(SystemVersion::from_string)
        .unwrap_or_else(|| SystemVersion::Unknown);

    let info = Info {
        system_type: get_os(),
        version,
        bit_depth: bit_depth::get(),
        ..Default::default()
    };

    trace!("Returning {:?}", info);
    info
}

fn get_os() -> Type {
    match uname("-o").as_deref() {
        Some("illumos") => Type::Illumos,
        _ => Type::Unknown,
    }
}

#[cfg(test)]
mod illumos_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn system_type() {
        let version = current_platform();
        assert_eq!(Type::Illumos, version.system_type());
    }
}
