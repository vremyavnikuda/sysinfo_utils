use crate::Info;
use crate::system_os::Type;
use log::trace;

pub fn current_platform() -> Info {
    trace!("android::current_platform() is called");

    let info = Info::with_type(Type::Android);
    trace!("Returning system information: {:?}", info);
    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn os_type() {
        let version = current_platform();
        assert_eq!(Type::Android, version.system_type());
    }
}
