use std::{fs::File, io::Read};

use log::{error, trace};

use crate::{BitDepth, SystemVersion, bit_depth, system_info::Info, system_os::Type};

const UNAME_FILE: &str = "sys:uname";

pub fn current_platform() -> Info {
    trace!("redox::current_platform is called");

    let version = get_version()
        .map(SystemVersion::from_string)
        .unwrap_or_else(|| SystemVersion::Unknown);
    let info = Info {
        system_type: Type::Redox,
        version,
        bit_depth: BitDepth::Unknown,
        ..Default::default()
    };
    trace!("Returning {:?}", info);
    info
}

fn get_version() -> Option<String> {
    let mut file = match File::open(UNAME_FILE) {
        Ok(file) => file,
        Err(e) => {
            error!("Unable to open {} file: {:?}", UNAME_FILE, e);
            return None;
        }
    };

    let mut version = String::new();
    if let Err(e) = file.read_to_string(&mut version) {
        error!("Unable to read {} file: {:?}", UNAME_FILE, e);
        return None;
    }
    Some(version)
}

#[cfg(test)]
mod redox_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn system_type() {
        let version = current_platform();
        assert_eq!(Type::Redox, version.system_type());
    }
}
