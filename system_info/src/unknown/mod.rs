use log::trace;

use crate::system_info::Info;

pub fn current_platform() -> Info {
    trace!("unknown::current_platform is called");
    Info::unknown()
}

#[cfg(test)]
mod tests {
    use crate::system_os::Type;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn os_type() {
        let version = current_platform();
        assert_eq!(Type::Unknown, version.system_type());
    }
}
