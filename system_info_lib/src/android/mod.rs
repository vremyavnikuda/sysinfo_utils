use crate::system_os::Type;
use crate::Info;
use log::trace;

pub fn current_platform() -> Info {
    trace!("android::current_platform() is called");

    let info = Info::with_type(Type::Android);
    trace!("Returning system information: {:?}", info);
    info
}

#[cfg(test)]
mod android_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn system_type() {
        let version = current_platform();
        assert_eq!(Type::Android, version.system_type());
    }
}
