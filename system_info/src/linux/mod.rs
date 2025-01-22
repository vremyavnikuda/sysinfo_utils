mod system_file_release;
mod system_release_lsb;

use crate::{architecture, bit_depth, system_os::Type, Info};
use log::trace;

//TODO: Implement the `current_platform` function for the Linux platform.
pub fn current_platform() -> Info {
    trace!("linux::current_platform() is called");

    let mut info = system_release_lsb::get()
        .or_else(system_file_release::get)
        .unwrap_or_else(|| Info::with_type(Type::Linux));
    info.bit_depth = bit_depth::get();
    info.architecture = architecture::get();

    trace!("Returns {:?}", info);
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_type() {
        let version = current_platform();
        match version.system_type() {
            Type::AlmaLinux
            | Type::Alpaquita
            | Type::Alpine
            | Type::Amazon
            | Type::Arch
            | Type::Artix
            | Type::CachyOS
            | Type::CentOS
            | Type::Debian
            | Type::EndeavourOS
            | Type::Fedora
            | Type::Garuda
            | Type::Gentoo
            | Type::Kali
            | Type::Linux
            | Type::Mabox
            | Type::Manjaro
            | Type::Mariner
            | Type::NixOS
            | Type::Nobara
            | Type::Uos
            | Type::OpenCloudOS
            | Type::OpenEuler
            | Type::OpenSUSE
            | Type::OracleLinux
            | Type::Pop
            | Type::Raspbian
            | Type::Redhat
            | Type::RedHatEnterprise
            | Type::RockyLinux
            | Type::Solus
            | Type::SUSE
            | Type::Ubuntu
            | Type::Ultramarine
            | Type::Void
            | Type::Mint => (),
            os_type => {
                panic!("Unexpected OS type: {}", os_type);
            }
        }
    }
}
