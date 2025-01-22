use std::process::Command;
use log::{error, trace};
use crate::{bit_depth, system_os::Type, system_uname::uname, Info, SystemVersion};

pub fn current_platform() -> Info {
    trace!("freebsd::current_platform is called");

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


/// Executes the `/sbin/sysctl` command with the argument `hardening.version`
/// to check the hardening version of the system. If the command is successful,
/// it returns the output. If the command fails, it logs an error message and
/// returns `Type::FreeBSD`.
fn get_os() -> Type {
    match uname("-s").as_deref() {
        Some("MidnightBSD") => Type::MidnightBSD,
        Some("FreeBSD") => {
            let check_hardening = match Command::new("/sbin/sysctl")
                .arg("hardening.version")
                .output()
            {
                Ok(o) => o,
                Err(e) => {
                    error!("Failed to invoke '/sbin/sysctl': {:?}", e);
                    return Type::FreeBSD;
                }
            };
            match std::str::from_utf8(&check_hardening.stderr) {
                Ok("0\n") => Type::HardenedBSD,
                Ok(_) => Type::FreeBSD,
                Err(_) => Type::FreeBSD,
            }
        }
        _ => Type::Unknown,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn os_type() {
        let version = current_platform();
        assert_eq!(Type::FreeBSD, version.system_type());
    }
}