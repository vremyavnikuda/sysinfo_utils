use crate::{bit_depth, system_os::Type, system_uname::uname, Info, SystemVersion};
use log::{error, trace};
use std::process::Command;

/// Возвращает информацию о текущей платформе, включая тип системы, версию и разрядность.
///
/// Эта функция определяет текущую платформу, используя:
/// - `uname` для получения версии системы.
/// - Вспомогательную функцию `get_os` для определения типа системы.
/// - Функцию `bit_depth::get` для определения разрядности системы.
///
/// Возвращаемая структура `Info` содержит следующие данные:
/// - `system_type`: тип операционной системы (например, FreeBSD, MidnightBSD).
/// - `version`: версия системы, определенная с помощью `uname -r`.
/// - `bit_depth`: разрядность системы.
///
/// # Пример
/// ```
/// let platform_info = current_platform();
/// println!("System type: {:?}", platform_info.system_type);
/// println!("System version: {:?}", platform_info.version);
/// println!("Bit depth: {:?}", platform_info.bit_depth);
/// ```
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
                Ok(ok) => ok,
                Err(error) => {
                    error!("Failed to invoke '/sbin/sysctl': {:?}", error);
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
mod freebsd_tests {
    use super::*;
    use crate::system_os::Type;
    use pretty_assertions::assert_eq;

    /// Проверяет, что текущая платформа правильно определяется как FreeBSD.
    #[test]
    fn current_platform_freebsd() {
        let platform = current_platform();
        assert_eq!(platform.system_type, Type::FreeBSD);
    }
}
