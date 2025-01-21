use log::trace;

use crate::{system_os::Type, Info};

pub fn current_platform() -> Info{
    trace!("emscripten::current_platform() is called");

    let info = Info::with_type(Type::Emscripten);
    trace!("Returning system information: {:?}", info);
    info
}

#[cfg(test)]
mod test{
    use crate::system_os::Type;

    use super::*;
    use pretty_assertions::assert_eq;
    #[test]
    fn os_type(){
        let version = current_platform();
        assert_eq!(Type::Emscripten, version.system_type());
    }
}